use rusqlite::{Connection, backup::Backup};
use std::{path::Path, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("no se pudo cargar el archivo SQLite")]
    Load,
    #[error("no se pudo guardar la copia SQLite")]
    Save,
}

pub fn load_into_memory(path: &Path) -> Result<Connection, BackupError> {
    let source = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| BackupError::Load)?;
    source
        .query_row("PRAGMA schema_version", [], |_| Ok(()))
        .map_err(|_| BackupError::Load)?;
    let mut destination = Connection::open_in_memory().map_err(|_| BackupError::Load)?;
    copy(&source, &mut destination).map_err(|_| BackupError::Load)?;
    Ok(destination)
}

pub fn save_from_memory(source: &Connection, path: &Path) -> Result<(), BackupError> {
    let mut destination = Connection::open(path).map_err(|_| BackupError::Save)?;
    copy(source, &mut destination).map_err(|_| BackupError::Save)
}

fn copy(source: &Connection, destination: &mut Connection) -> rusqlite::Result<()> {
    let backup = Backup::new(source, destination)?;
    backup.run_to_completion(128, Duration::from_millis(1), None)
}
