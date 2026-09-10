use ramsqlite_server::{
    config::Limits,
    protocol::{self, Request, Response},
    registry::Registry,
    server,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    net::{TcpListener, TcpStream},
    path::Path,
    sync::Arc,
};
use tempfile::TempDir;

fn run_server(requests: usize, root: &Path) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let registry = Arc::new(Registry::with_data_root(
        Limits::default(),
        root.to_path_buf(),
    ));
    let worker = std::thread::spawn(move || {
        for _ in 0..requests {
            let (stream, _) = listener.accept().unwrap();
            server::handle_connection(stream, Arc::clone(&registry), 4096).unwrap();
        }
    });
    (address, worker)
}

fn exchange(address: std::net::SocketAddr, request: Request) -> Response {
    let mut stream = TcpStream::connect(address).unwrap();
    protocol::write_frame(&mut stream, &request).unwrap();
    let bytes = protocol::read_frame(&mut stream, 4096).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn create(address: std::net::SocketAddr, name: &str, id: &str) {
    let response = exchange(
        address,
        Request::Crear {
            id_solicitud: id.into(),
            base: name.into(),
            bytes_estimados: None,
        },
    );
    assert!(response.error.is_none());
}

fn execute(id: &str, base: &str, sql: &str, parameters: Vec<Value>) -> Request {
    Request::Ejecutar {
        id_solicitud: id.into(),
        base: base.into(),
        id_conexion: "cliente".into(),
        sql: sql.into(),
        parametros: parameters,
    }
}

#[test]
fn rechaza_archivo_invalido_sin_publicar_una_base_parcial() {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("invalida.sqlite"), "no es sqlite").unwrap();
    let (address, worker) = run_server(2, root.path());

    let rejected = exchange(
        address,
        Request::Cargar {
            id_solicitud: "1".into(),
            base: "ventas".into(),
            ruta: "invalida.sqlite".into(),
            bytes_estimados: None,
        },
    );
    assert_eq!(rejected.error.unwrap().codigo, "carga");

    let listed = exchange(
        address,
        Request::Listar {
            id_solicitud: "2".into(),
        },
    );
    assert!(
        listed.resultado.unwrap()["databases"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    worker.join().unwrap();
}

#[test]
fn informa_fallo_de_guardado_y_conserva_la_base_en_ram() {
    let root = TempDir::new().unwrap();
    fs::create_dir(root.path().join("destino-directorio")).unwrap();
    let (address, worker) = run_server(5, root.path());
    create(address, "ventas", "1");
    assert!(
        exchange(
            address,
            execute("2", "ventas", "CREATE TABLE datos (valor TEXT)", vec![]),
        )
        .error
        .is_none()
    );
    assert!(
        exchange(
            address,
            execute(
                "3",
                "ventas",
                "INSERT INTO datos(valor) VALUES (?)",
                vec![json!("permanece")],
            ),
        )
        .error
        .is_none()
    );

    let rejected = exchange(
        address,
        Request::Sincronizar {
            id_solicitud: "4".into(),
            base: "ventas".into(),
            ruta: "destino-directorio".into(),
        },
    );
    assert_eq!(rejected.error.unwrap().codigo, "persistencia");

    let still_available = exchange(
        address,
        execute("5", "ventas", "SELECT valor FROM datos", vec![]),
    );
    assert_eq!(
        still_available.resultado.unwrap()["filas"][0]["valor"],
        "permanece"
    );
    worker.join().unwrap();
}

#[test]
fn rechaza_persistencia_parcial_no_compatible_con_el_mvp() {
    let root = TempDir::new().unwrap();
    let (address, worker) = run_server(2, root.path());
    create(address, "ventas", "1");

    let rejected = exchange(
        address,
        Request::SincronizarParcial {
            id_solicitud: "2".into(),
            base: "ventas".into(),
            ruta: "ventas.sqlite".into(),
            tablas: vec!["ventas".into()],
        },
    );
    assert_eq!(rejected.error.unwrap().codigo, "persistencia");
    worker.join().unwrap();
}

#[test]
fn carga_sqlite_valido_y_sincroniza_una_copia_completa() {
    let root = TempDir::new().unwrap();
    let source = root.path().join("origen.sqlite");
    let source_connection = Connection::open(&source).unwrap();
    source_connection
        .execute_batch("CREATE TABLE datos (valor TEXT); INSERT INTO datos VALUES ('cargado');")
        .unwrap();
    drop(source_connection);

    let (address, worker) = run_server(4, root.path());
    assert!(
        exchange(
            address,
            Request::Cargar {
                id_solicitud: "1".into(),
                base: "ventas".into(),
                ruta: "origen.sqlite".into(),
                bytes_estimados: None,
            },
        )
        .error
        .is_none()
    );
    let loaded = exchange(
        address,
        execute("2", "ventas", "SELECT valor FROM datos", vec![]),
    );
    assert_eq!(loaded.resultado.unwrap()["filas"][0]["valor"], "cargado");
    assert!(
        exchange(
            address,
            Request::Sincronizar {
                id_solicitud: "3".into(),
                base: "ventas".into(),
                ruta: "respaldo.sqlite".into(),
            },
        )
        .error
        .is_none()
    );
    let listed = exchange(
        address,
        Request::Listar {
            id_solicitud: "4".into(),
        },
    );
    assert_eq!(listed.resultado.unwrap()["databases"][0]["name"], "ventas");
    worker.join().unwrap();

    let copied = Connection::open(root.path().join("respaldo.sqlite")).unwrap();
    let value: String = copied
        .query_row("SELECT valor FROM datos", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "cargado");
}
