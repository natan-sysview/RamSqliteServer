use ramsqlite_server::{
    config::{Config, ConfigError, resolve_existing_path},
    protocol::require_loopback,
};
use std::fs;

#[test]
fn rechaza_rutas_absolutas_y_parentes() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        resolve_existing_path(root.path(), "/tmp/base.db"),
        Err(ConfigError::UnauthorizedPath)
    );
    assert_eq!(
        resolve_existing_path(root.path(), "../base.db"),
        Err(ConfigError::UnauthorizedPath)
    );
}

#[cfg(unix)]
#[test]
fn rechaza_enlace_que_escapa_de_la_raiz() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let link = root.path().join("escape.db");
    symlink(outside.path(), &link).unwrap();

    assert_eq!(
        resolve_existing_path(root.path(), "escape.db"),
        Err(ConfigError::UnauthorizedPath)
    );
}

#[test]
fn acepta_archivo_existente_dentro_de_la_raiz() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("base.db");
    fs::write(&file, b"sqlite").unwrap();
    assert_eq!(
        resolve_existing_path(root.path(), "base.db").unwrap(),
        file.canonicalize().unwrap()
    );
}

#[test]
fn rechaza_configuracion_y_par_no_loopback() {
    let mut config = Config::local(tempfile::tempdir().unwrap().path().to_owned());
    config.listen = "0.0.0.0:7432".parse().unwrap();
    assert_eq!(config.validate(), Err(ConfigError::NonLoopback));
    assert_eq!(
        require_loopback("192.0.2.1:7432".parse().unwrap())
            .unwrap_err()
            .codigo,
        "destino_no_local"
    );
}
