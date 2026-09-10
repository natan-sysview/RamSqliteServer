use ramsqlite_server::{
    config::Limits,
    protocol::{self, Request, Response},
    registry::Registry,
    server,
};
use serde_json::{Value, json};
use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

fn run_server(requests: usize) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let registry = Arc::new(Registry::new(Limits::default()));
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

fn execute(id: &str, base: &str, connection: &str, sql: &str, parameters: Vec<Value>) -> Request {
    Request::Ejecutar {
        id_solicitud: id.into(),
        base: base.into(),
        id_conexion: connection.into(),
        sql: sql.into(),
        parametros: parameters,
    }
}

#[test]
fn rechaza_parametros_incompatibles_sin_modificar_la_base() {
    let (address, worker) = run_server(4);
    create(address, "ventas", "1");
    assert!(
        exchange(
            address,
            execute(
                "2",
                "ventas",
                "cliente-a",
                "CREATE TABLE ventas (nombre TEXT)",
                vec![]
            ),
        )
        .error
        .is_none()
    );

    let rejected = exchange(
        address,
        execute(
            "3",
            "ventas",
            "cliente-a",
            "INSERT INTO ventas(nombre) VALUES (?)",
            vec![json!({"no": "escalar"})],
        ),
    );
    assert_eq!(rejected.error.unwrap().codigo, "parametros");

    let count = exchange(
        address,
        execute(
            "4",
            "ventas",
            "cliente-a",
            "SELECT COUNT(*) AS total FROM ventas",
            vec![],
        ),
    );
    assert_eq!(count.resultado.unwrap()["filas"][0]["total"], 0);
    worker.join().unwrap();
}

#[test]
fn rechaza_sql_invalido_sin_modificar_la_base() {
    let (address, worker) = run_server(3);
    create(address, "ventas", "1");
    let rejected = exchange(
        address,
        execute("2", "ventas", "cliente-a", "ESTO NO ES SQL", vec![]),
    );
    assert_eq!(rejected.error.unwrap().codigo, "sql");
    let listed = exchange(
        address,
        Request::Listar {
            id_solicitud: "3".into(),
        },
    );
    assert_eq!(listed.resultado.unwrap()["databases"][0]["name"], "ventas");
    worker.join().unwrap();
}

#[test]
fn aisla_las_operaciones_por_base_y_devuelve_filas_parametrizadas() {
    let (address, worker) = run_server(8);
    create(address, "ventas", "1");
    create(address, "inventario", "2");
    for (id, base, sql, parameters) in [
        ("3", "ventas", "CREATE TABLE datos (nombre TEXT)", vec![]),
        (
            "4",
            "inventario",
            "CREATE TABLE datos (nombre TEXT)",
            vec![],
        ),
        (
            "5",
            "ventas",
            "INSERT INTO datos(nombre) VALUES (?)",
            vec![json!("teclado")],
        ),
        (
            "6",
            "inventario",
            "INSERT INTO datos(nombre) VALUES (?)",
            vec![json!("monitor")],
        ),
    ] {
        assert!(
            exchange(address, execute(id, base, "cliente", sql, parameters))
                .error
                .is_none()
        );
    }
    let ventas = exchange(
        address,
        execute(
            "7",
            "ventas",
            "cliente",
            "SELECT nombre FROM datos WHERE nombre = ?",
            vec![json!("teclado")],
        ),
    );
    let inventario = exchange(
        address,
        execute(
            "8",
            "inventario",
            "cliente",
            "SELECT nombre FROM datos",
            vec![],
        ),
    );
    assert_eq!(ventas.resultado.unwrap()["filas"][0]["nombre"], "teclado");
    assert_eq!(
        inventario.resultado.unwrap()["filas"][0]["nombre"],
        "monitor"
    );
    worker.join().unwrap();
}

#[test]
fn rechaza_conflicto_de_escritor_sin_corromper_la_transaccion_activa() {
    let (address, worker) = run_server(7);
    create(address, "ventas", "1");
    assert!(
        exchange(
            address,
            execute(
                "2",
                "ventas",
                "cliente-a",
                "CREATE TABLE datos (valor INTEGER)",
                vec![]
            ),
        )
        .error
        .is_none()
    );
    assert!(
        exchange(
            address,
            Request::IniciarTransaccion {
                id_solicitud: "3".into(),
                base: "ventas".into(),
                id_conexion: "cliente-a".into(),
            },
        )
        .error
        .is_none()
    );
    assert!(
        exchange(
            address,
            execute(
                "4",
                "ventas",
                "cliente-a",
                "INSERT INTO datos(valor) VALUES (1)",
                vec![]
            ),
        )
        .error
        .is_none()
    );
    let conflict = exchange(
        address,
        execute(
            "5",
            "ventas",
            "cliente-b",
            "INSERT INTO datos(valor) VALUES (2)",
            vec![],
        ),
    );
    assert_eq!(conflict.error.unwrap().codigo, "transaccion");
    assert!(
        exchange(
            address,
            Request::RevertirTransaccion {
                id_solicitud: "6".into(),
                base: "ventas".into(),
                id_conexion: "cliente-a".into(),
            },
        )
        .error
        .is_none()
    );
    let count = exchange(
        address,
        execute(
            "7",
            "ventas",
            "cliente-b",
            "SELECT COUNT(*) AS total FROM datos",
            vec![],
        ),
    );
    assert_eq!(count.resultado.unwrap()["filas"][0]["total"], 0);
    worker.join().unwrap();
}
