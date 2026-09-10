use ramsqlite_server::{
    config::Limits,
    protocol::{self, Request, Response},
    registry::Registry,
    server,
};
use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

fn exchange(address: std::net::SocketAddr, request: Request) -> Response {
    let mut stream = TcpStream::connect(address).unwrap();
    protocol::write_frame(&mut stream, &request).unwrap();
    let bytes = protocol::read_frame(&mut stream, 4096).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn crea_y_lista_una_base_por_loopback_real() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let registry = Arc::new(Registry::new(Limits::default()));
    let worker_registry = Arc::clone(&registry);
    let worker = std::thread::spawn(move || {
        for _ in 0..2 {
            let (stream, _) = listener.accept().unwrap();
            server::handle_connection(stream, Arc::clone(&worker_registry), 4096).unwrap();
        }
    });

    let created = exchange(
        address,
        Request::Crear {
            id_solicitud: "1".into(),
            base: "ventas".into(),
            bytes_estimados: Some(1024),
        },
    );
    assert!(created.error.is_none());
    let listed = exchange(
        address,
        Request::Listar {
            id_solicitud: "2".into(),
        },
    );
    assert_eq!(listed.resultado.unwrap()["databases"][0]["name"], "ventas");
    worker.join().unwrap();
}
