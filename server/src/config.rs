use std::{
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("solo se permiten direcciones loopback")]
    NonLoopback,
    #[error("ruta no autorizada")]
    UnauthorizedPath,
    #[error("no se pudo resolver la ruta: {0}")]
    InvalidPath(String),
}

impl ConfigError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NonLoopback => "destino_no_local",
            Self::UnauthorizedPath => "ruta_no_autorizada",
            Self::InvalidPath(_) => "ruta_invalida",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Limits {
    pub max_databases: usize,
    pub max_total_bytes: u64,
    pub max_database_bytes: u64,
    pub new_database_reservation_bytes: u64,
    pub max_frame_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_databases: 32,
            max_total_bytes: 1024 * 1024 * 1024,
            max_database_bytes: 256 * 1024 * 1024,
            new_database_reservation_bytes: 1024 * 1024,
            max_frame_bytes: 8 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub listen: SocketAddr,
    pub data_root: PathBuf,
    pub limits: Limits,
}

impl Config {
    pub fn local(data_root: PathBuf) -> Self {
        Self {
            listen: "127.0.0.1:7432".parse().expect("dirección local válida"),
            data_root,
            limits: Limits::default(),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.listen.ip().is_loopback() {
            Ok(())
        } else {
            Err(ConfigError::NonLoopback)
        }
    }
}

fn safe_relative(path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(ConfigError::UnauthorizedPath);
    }
    Ok(())
}

pub fn resolve_existing_path(
    root: &Path,
    relative: impl AsRef<Path>,
) -> Result<PathBuf, ConfigError> {
    let relative = relative.as_ref();
    safe_relative(relative)?;
    let root = root
        .canonicalize()
        .map_err(|error| ConfigError::InvalidPath(error.to_string()))?;
    let target = root
        .join(relative)
        .canonicalize()
        .map_err(|error| ConfigError::InvalidPath(error.to_string()))?;
    if target.starts_with(&root) {
        Ok(target)
    } else {
        Err(ConfigError::UnauthorizedPath)
    }
}

pub fn resolve_destination_path(
    root: &Path,
    relative: impl AsRef<Path>,
) -> Result<PathBuf, ConfigError> {
    let relative = relative.as_ref();
    safe_relative(relative)?;
    let root = root
        .canonicalize()
        .map_err(|error| ConfigError::InvalidPath(error.to_string()))?;
    let target = root.join(relative);
    let resolved = if target.exists() {
        target.canonicalize()
    } else {
        target
            .parent()
            .unwrap_or(&root)
            .canonicalize()
            .map(|parent| parent.join(target.file_name().expect("ruta relativa normal")))
    }
    .map_err(|error| ConfigError::InvalidPath(error.to_string()))?;
    if resolved.starts_with(&root) {
        Ok(resolved)
    } else {
        Err(ConfigError::UnauthorizedPath)
    }
}
