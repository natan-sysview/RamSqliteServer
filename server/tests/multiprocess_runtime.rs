use ramsqlite_server::protocol::{self, Request, Response};
use serde_json::{Value, json};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Child, Command},
    thread,
    time::Duration,
};

const MAX_FRAME_BYTES: usize = 4096;

struct ServerProcess {
    child: Child,
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn unused_loopback_address() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserva un puerto loopback");
    listener.local_addr().expect("lee el puerto reservado")
}

fn exchange(address: SocketAddr, request: Request) -> Response {
    let mut last_error = None;
    for _ in 0..50 {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                protocol::write_frame(&mut stream, &request).expect("escribe solicitud enmarcada");
                let bytes = protocol::read_frame(&mut stream, MAX_FRAME_BYTES)
                    .expect("lee respuesta enmarcada");
                return serde_json::from_slice(&bytes).expect("interpreta respuesta JSON");
            }
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
    panic!("el servidor no aceptó conexiones: {last_error:?}");
}

fn execute(id: &str, base: &str, sql: &str, parameters: Vec<Value>) -> Request {
    Request::Ejecutar {
        id_solicitud: id.into(),
        base: base.into(),
        id_conexion: "verificador".into(),
        sql: sql.into(),
        parametros: parameters,
    }
}

fn run_probe_from_environment() {
    let address = std::env::var("RAMSQLITE_PROBE_ADDRESS")
        .expect("dirección de prueba")
        .parse()
        .expect("dirección loopback válida");
    let base = std::env::var("RAMSQLITE_PROBE_BASE").expect("base de prueba");
    let value = std::env::var("RAMSQLITE_PROBE_VALUE").expect("valor de prueba");

    let create = exchange(
        address,
        Request::Crear {
            id_solicitud: format!("crear-{base}"),
            base: base.clone(),
            bytes_estimados: None,
        },
    );
    assert!(create.error.is_none(), "crea la base del subproceso");
    assert!(
        exchange(
            address,
            execute(
                &format!("tabla-{base}"),
                &base,
                "CREATE TABLE datos (valor TEXT NOT NULL)",
                vec![],
            ),
        )
        .error
        .is_none(),
        "crea la tabla del subproceso"
    );
    assert!(
        exchange(
            address,
            execute(
                &format!("insertar-{base}"),
                &base,
                "INSERT INTO datos(valor) VALUES (?)",
                vec![json!(value)],
            ),
        )
        .error
        .is_none(),
        "inserta el dato del subproceso"
    );
}

#[test]
fn cliente_subproceso() {
    if std::env::var_os("RAMSQLITE_PROBE").is_some() {
        run_probe_from_environment();
    }
}

#[test]
fn dos_clientes_en_procesos_independientes_usan_bases_aisladas() {
    let tempdir = tempfile::tempdir().expect("crea raíz temporal");
    let address = unused_loopback_address();
    let server = Command::new(env!("CARGO_BIN_EXE_ramsqlite-server"))
        .env("RAMSQLITE_LISTEN", address.to_string())
        .env("RAMSQLITE_DATA_ROOT", tempdir.path())
        .spawn()
        .expect("inicia el binario real del servidor");
    let _server = ServerProcess { child: server };
    let test_binary = std::env::current_exe().expect("ubica el binario de pruebas");

    for (base, value) in [("ventas", "teclado"), ("inventario", "monitor")] {
        let status = Command::new(&test_binary)
            .arg("--exact")
            .arg("cliente_subproceso")
            .arg("--nocapture")
            .env("RAMSQLITE_PROBE", "1")
            .env("RAMSQLITE_PROBE_ADDRESS", address.to_string())
            .env("RAMSQLITE_PROBE_BASE", base)
            .env("RAMSQLITE_PROBE_VALUE", value)
            .status()
            .expect("ejecuta cliente en proceso independiente");
        assert!(status.success(), "el cliente {base} termina correctamente");
    }

    let ventas = exchange(
        address,
        execute(
            "verificar-ventas",
            "ventas",
            "SELECT valor FROM datos",
            vec![],
        ),
    );
    let inventario = exchange(
        address,
        execute(
            "verificar-inventario",
            "inventario",
            "SELECT valor FROM datos",
            vec![],
        ),
    );
    assert_eq!(
        ventas.resultado.expect("resultado de ventas")["filas"][0]["valor"],
        "teclado"
    );
    assert_eq!(
        inventario.resultado.expect("resultado de inventario")["filas"][0]["valor"],
        "monitor"
    );
}
