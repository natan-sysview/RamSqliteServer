use rusqlite::{
    Connection,
    types::{Value, ValueRef},
};
use serde::Serialize;
use serde_json::{Map, Value as JsonValue, json};
use std::sync::mpsc::{self, SyncSender};
use thiserror::Error;

const MAX_PENDING_REQUESTS: usize = 32;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("la conexión debe tener un identificador no vacío")]
    InvalidConnection,
    #[error("los parámetros solo admiten null, booleanos, números y texto")]
    Parameters,
    #[error("la sentencia SQL no es válida")]
    Sql,
    #[error("la transacción no puede continuar")]
    Transaction,
    #[error("la base no está disponible")]
    Unavailable,
}

impl DatabaseError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidConnection => "conexion_invalida",
            Self::Parameters => "parametros",
            Self::Sql => "sql",
            Self::Transaction => "transaccion",
            Self::Unavailable => "base_no_disponible",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub filas: Vec<Map<String, JsonValue>>,
    pub filas_afectadas: usize,
}

#[derive(Clone)]
pub struct Database {
    sender: SyncSender<Command>,
}

enum Command {
    Execute {
        connection_id: String,
        sql: String,
        parameters: Vec<JsonValue>,
        response: mpsc::Sender<Result<ExecutionResult, DatabaseError>>,
    },
    Begin {
        connection_id: String,
        response: mpsc::Sender<Result<(), DatabaseError>>,
    },
    Commit {
        connection_id: String,
        response: mpsc::Sender<Result<(), DatabaseError>>,
    },
    Rollback {
        connection_id: String,
        response: mpsc::Sender<Result<(), DatabaseError>>,
    },
    Shutdown,
}

impl Database {
    pub fn in_memory() -> Result<Self, DatabaseError> {
        let (sender, receiver) = mpsc::sync_channel(MAX_PENDING_REQUESTS);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        std::thread::spawn(move || match Connection::open_in_memory() {
            Ok(connection) => {
                let _ = ready_sender.send(Ok(()));
                run_worker(connection, receiver);
            }
            Err(_) => {
                let _ = ready_sender.send(Err(DatabaseError::Unavailable));
            }
        });
        ready_receiver
            .recv()
            .map_err(|_| DatabaseError::Unavailable)??;
        Ok(Self { sender })
    }

    pub fn execute(
        &self,
        connection_id: String,
        sql: String,
        parameters: Vec<JsonValue>,
    ) -> Result<ExecutionResult, DatabaseError> {
        let (response, receiver) = mpsc::channel();
        self.send(Command::Execute {
            connection_id,
            sql,
            parameters,
            response,
        })?;
        receiver.recv().map_err(|_| DatabaseError::Unavailable)?
    }

    pub fn begin(&self, connection_id: String) -> Result<(), DatabaseError> {
        self.transaction(connection_id, |connection_id, response| Command::Begin {
            connection_id,
            response,
        })
    }

    pub fn commit(&self, connection_id: String) -> Result<(), DatabaseError> {
        self.transaction(connection_id, |connection_id, response| Command::Commit {
            connection_id,
            response,
        })
    }

    pub fn rollback(&self, connection_id: String) -> Result<(), DatabaseError> {
        self.transaction(connection_id, |connection_id, response| Command::Rollback {
            connection_id,
            response,
        })
    }

    pub fn shutdown(&self) {
        let _ = self.sender.send(Command::Shutdown);
    }

    fn transaction(
        &self,
        connection_id: String,
        command: impl FnOnce(String, mpsc::Sender<Result<(), DatabaseError>>) -> Command,
    ) -> Result<(), DatabaseError> {
        let (response, receiver) = mpsc::channel();
        self.send(command(connection_id, response))?;
        receiver.recv().map_err(|_| DatabaseError::Unavailable)?
    }

    fn send(&self, command: Command) -> Result<(), DatabaseError> {
        self.sender
            .send(command)
            .map_err(|_| DatabaseError::Unavailable)
    }
}

fn run_worker(connection: Connection, receiver: mpsc::Receiver<Command>) {
    let mut worker = Worker {
        connection,
        writer: None,
    };
    while let Ok(command) = receiver.recv() {
        match command {
            Command::Execute {
                connection_id,
                sql,
                parameters,
                response,
            } => {
                let _ = response.send(worker.execute(connection_id, sql, parameters));
            }
            Command::Begin {
                connection_id,
                response,
            } => {
                let _ = response.send(worker.begin(connection_id));
            }
            Command::Commit {
                connection_id,
                response,
            } => {
                let _ = response.send(worker.finish(connection_id, "COMMIT"));
            }
            Command::Rollback {
                connection_id,
                response,
            } => {
                let _ = response.send(worker.finish(connection_id, "ROLLBACK"));
            }
            Command::Shutdown => break,
        }
    }
}

struct Worker {
    connection: Connection,
    writer: Option<String>,
}

impl Worker {
    fn begin(&mut self, connection_id: String) -> Result<(), DatabaseError> {
        self.require_connection(&connection_id)?;
        if self.writer.is_some() {
            return Err(DatabaseError::Transaction);
        }
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| DatabaseError::Transaction)?;
        self.writer = Some(connection_id);
        Ok(())
    }

    fn finish(&mut self, connection_id: String, command: &str) -> Result<(), DatabaseError> {
        self.require_owner(&connection_id)?;
        self.connection
            .execute_batch(command)
            .map_err(|_| DatabaseError::Transaction)?;
        self.writer = None;
        Ok(())
    }

    fn execute(
        &mut self,
        connection_id: String,
        sql: String,
        parameters: Vec<JsonValue>,
    ) -> Result<ExecutionResult, DatabaseError> {
        self.require_connection(&connection_id)?;
        if self
            .writer
            .as_deref()
            .is_some_and(|owner| owner != connection_id)
        {
            return Err(DatabaseError::Transaction);
        }
        let parameters = parameters
            .into_iter()
            .map(to_sql_value)
            .collect::<Result<Vec<_>, _>>()?;
        let mut statement = self
            .connection
            .prepare(&sql)
            .map_err(|_| DatabaseError::Sql)?;
        if !statement.readonly()
            && self
                .writer
                .as_deref()
                .is_some_and(|owner| owner != connection_id)
        {
            return Err(DatabaseError::Transaction);
        }
        if statement.readonly() {
            let columns = statement
                .column_names()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let rows = statement
                .query_map(rusqlite::params_from_iter(parameters), |row| {
                    let mut values = Map::new();
                    for (index, column) in columns.iter().enumerate() {
                        values.insert(column.clone(), from_sql_value(row.get_ref(index)?));
                    }
                    Ok(values)
                })
                .map_err(|_| DatabaseError::Sql)?;
            let rows = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| DatabaseError::Sql)?;
            Ok(ExecutionResult {
                filas: rows,
                filas_afectadas: 0,
            })
        } else {
            let affected = statement
                .execute(rusqlite::params_from_iter(parameters))
                .map_err(|_| DatabaseError::Sql)?;
            Ok(ExecutionResult {
                filas: Vec::new(),
                filas_afectadas: affected,
            })
        }
    }

    fn require_connection(&self, connection_id: &str) -> Result<(), DatabaseError> {
        if connection_id.trim().is_empty() {
            Err(DatabaseError::InvalidConnection)
        } else {
            Ok(())
        }
    }

    fn require_owner(&self, connection_id: &str) -> Result<(), DatabaseError> {
        self.require_connection(connection_id)?;
        if self.writer.as_deref() == Some(connection_id) {
            Ok(())
        } else {
            Err(DatabaseError::Transaction)
        }
    }
}

fn to_sql_value(value: JsonValue) -> Result<Value, DatabaseError> {
    match value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(value) => Ok(Value::Integer(i64::from(value))),
        JsonValue::Number(value) => value
            .as_i64()
            .map(Value::Integer)
            .or_else(|| value.as_f64().map(Value::Real))
            .ok_or(DatabaseError::Parameters),
        JsonValue::String(value) => Ok(Value::Text(value)),
        JsonValue::Array(_) | JsonValue::Object(_) => Err(DatabaseError::Parameters),
    }
}

fn from_sql_value(value: ValueRef<'_>) -> JsonValue {
    match value {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Integer(value) => json!(value),
        ValueRef::Real(value) => json!(value),
        ValueRef::Text(value) => JsonValue::String(String::from_utf8_lossy(value).into_owned()),
        ValueRef::Blob(value) => json!({"blob_bytes": value.len()}),
    }
}
