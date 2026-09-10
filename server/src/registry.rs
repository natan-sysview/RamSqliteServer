use crate::{
    config::{ConfigError, Limits, resolve_destination_path, resolve_existing_path},
    database::{Database, DatabaseError, ExecutionResult},
};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("el nombre de la base no puede estar vacío")]
    EmptyName,
    #[error("ya existe una base con ese nombre")]
    Duplicate,
    #[error("se alcanzó el máximo de bases")]
    MaximumDatabases,
    #[error("la reserva excede el límite por base")]
    PerDatabase,
    #[error("la reserva excede el límite total")]
    Total,
    #[error("la base no existe")]
    NotFound,
    #[error("la base no pudo iniciarse")]
    Unavailable,
}

impl AdmissionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyName => "nombre_invalido",
            Self::Duplicate => "base_duplicada",
            Self::MaximumDatabases | Self::PerDatabase | Self::Total => "capacidad",
            Self::NotFound => "base_no_existe",
            Self::Unavailable => "base_no_disponible",
        }
    }
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error(transparent)]
    Admission(#[from] AdmissionError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl RegistryError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Admission(error) => error.code(),
            Self::Config(error) => error.code(),
            Self::Database(error) => error.code(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub reserved_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct CapacitySnapshot {
    pub databases: Vec<DatabaseInfo>,
    pub used_bytes: u64,
    pub max_databases: usize,
    pub max_total_bytes: u64,
    pub max_database_bytes: u64,
}

struct DatabaseEntry {
    reservation: u64,
    database: Database,
}

pub struct Registry {
    limits: Limits,
    data_root: PathBuf,
    databases: Mutex<HashMap<String, DatabaseEntry>>,
}

impl Registry {
    pub fn new(limits: Limits) -> Self {
        Self::with_data_root(
            limits,
            std::env::current_dir().expect("directorio de trabajo disponible"),
        )
    }

    pub fn with_data_root(limits: Limits, data_root: PathBuf) -> Self {
        Self {
            limits,
            data_root,
            databases: Mutex::new(HashMap::new()),
        }
    }

    pub fn create(&self, name: &str, estimated_bytes: Option<u64>) -> Result<(), AdmissionError> {
        let reservation = estimated_bytes.unwrap_or(self.limits.new_database_reservation_bytes);
        let mut databases = self.databases.lock().expect("registro disponible");
        self.check_admission(&databases, name, reservation)?;
        let database = Database::in_memory().map_err(|_| AdmissionError::Unavailable)?;
        databases.insert(
            name.to_owned(),
            DatabaseEntry {
                reservation,
                database,
            },
        );
        Ok(())
    }

    pub fn load(
        &self,
        name: &str,
        relative_path: impl AsRef<Path>,
        estimated_bytes: Option<u64>,
    ) -> Result<(), RegistryError> {
        let path = resolve_existing_path(&self.data_root, relative_path)?;
        let size = std::fs::metadata(&path)
            .map_err(|error| ConfigError::InvalidPath(error.to_string()))?
            .len();
        let reservation =
            estimated_bytes.unwrap_or(size.max(self.limits.new_database_reservation_bytes));
        let mut databases = self.databases.lock().expect("registro disponible");
        self.check_admission(&databases, name, reservation)?;
        let database = Database::from_path(&path)?;
        databases.insert(
            name.to_owned(),
            DatabaseEntry {
                reservation,
                database,
            },
        );
        Ok(())
    }

    pub fn sync(&self, name: &str, relative_path: impl AsRef<Path>) -> Result<(), RegistryError> {
        let path = resolve_destination_path(&self.data_root, relative_path)?;
        self.database(name)?.sync(path).map_err(Into::into)
    }

    pub fn reject_partial_sync(&self, name: &str) -> Result<(), RegistryError> {
        self.database(name)?;
        Err(DatabaseError::Persistence.into())
    }

    pub fn close(&self, name: &str) -> Result<(), AdmissionError> {
        self.databases
            .lock()
            .expect("registro disponible")
            .remove(name)
            .map(|entry| entry.database.shutdown())
            .ok_or(AdmissionError::NotFound)
    }

    pub fn execute(
        &self,
        name: &str,
        connection_id: String,
        sql: String,
        parameters: Vec<Value>,
    ) -> Result<ExecutionResult, RegistryError> {
        self.database(name)?
            .execute(connection_id, sql, parameters)
            .map_err(Into::into)
    }

    pub fn begin(&self, name: &str, connection_id: String) -> Result<(), RegistryError> {
        self.database(name)?
            .begin(connection_id)
            .map_err(Into::into)
    }

    pub fn commit(&self, name: &str, connection_id: String) -> Result<(), RegistryError> {
        self.database(name)?
            .commit(connection_id)
            .map_err(Into::into)
    }

    pub fn rollback(&self, name: &str, connection_id: String) -> Result<(), RegistryError> {
        self.database(name)?
            .rollback(connection_id)
            .map_err(Into::into)
    }

    pub fn snapshot(&self) -> CapacitySnapshot {
        let databases = self.databases.lock().expect("registro disponible");
        let mut listed: Vec<_> = databases
            .iter()
            .map(|(name, entry)| DatabaseInfo {
                name: name.clone(),
                reserved_bytes: entry.reservation,
            })
            .collect();
        listed.sort_by(|a, b| a.name.cmp(&b.name));
        CapacitySnapshot {
            used_bytes: listed.iter().map(|entry| entry.reserved_bytes).sum(),
            databases: listed,
            max_databases: self.limits.max_databases,
            max_total_bytes: self.limits.max_total_bytes,
            max_database_bytes: self.limits.max_database_bytes,
        }
    }

    fn check_admission(
        &self,
        databases: &HashMap<String, DatabaseEntry>,
        name: &str,
        reservation: u64,
    ) -> Result<(), AdmissionError> {
        if name.trim().is_empty() {
            return Err(AdmissionError::EmptyName);
        }
        if databases.contains_key(name) {
            return Err(AdmissionError::Duplicate);
        }
        if databases.len() >= self.limits.max_databases {
            return Err(AdmissionError::MaximumDatabases);
        }
        if reservation > self.limits.max_database_bytes {
            return Err(AdmissionError::PerDatabase);
        }
        let used: u64 = databases.values().map(|entry| entry.reservation).sum();
        if used
            .checked_add(reservation)
            .is_none_or(|total| total > self.limits.max_total_bytes)
        {
            return Err(AdmissionError::Total);
        }
        Ok(())
    }

    fn database(&self, name: &str) -> Result<Database, RegistryError> {
        self.databases
            .lock()
            .expect("registro disponible")
            .get(name)
            .map(|entry| entry.database.clone())
            .ok_or(AdmissionError::NotFound.into())
    }
}
