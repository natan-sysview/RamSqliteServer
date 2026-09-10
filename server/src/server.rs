use crate::{
    protocol::{self, ProtocolError, Request, Response},
    registry::Registry,
};
use serde_json::json;
use std::{
    io,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

pub fn serve(listener: TcpListener, registry: Arc<Registry>, max_frame: usize) -> io::Result<()> {
    for incoming in listener.incoming() {
        let stream = incoming?;
        let registry = Arc::clone(&registry);
        std::thread::spawn(move || {
            if let Err(error) = handle_connection(stream, registry, max_frame) {
                eprintln!("conexión finalizada: {error}");
            }
        });
    }
    Ok(())
}

pub fn handle_connection(
    mut stream: TcpStream,
    registry: Arc<Registry>,
    max_frame: usize,
) -> io::Result<()> {
    protocol::require_loopback(stream.peer_addr()?)
        .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error.mensaje))?;
    let payload = protocol::read_frame(&mut stream, max_frame)?;
    let request: Request = serde_json::from_slice(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let id = request.id().to_owned();
    let outcome = match request {
        Request::Crear {
            base,
            bytes_estimados,
            ..
        } => registry
            .create(&base, bytes_estimados)
            .map(|_| json!({"base": base}))
            .map_err(|error| ProtocolError {
                codigo: error.code().into(),
                mensaje: error.to_string(),
            }),
        Request::Listar { .. } => Ok(json!(registry.snapshot())),
        Request::Cerrar { base, .. } => registry
            .close(&base)
            .map(|_| json!({"base": base}))
            .map_err(|error| ProtocolError {
                codigo: error.code().into(),
                mensaje: error.to_string(),
            }),
    };
    let response = match outcome {
        Ok(result) => Response {
            id_solicitud: id,
            resultado: Some(result),
            error: None,
        },
        Err(error) => Response {
            id_solicitud: id,
            resultado: None,
            error: Some(error),
        },
    };
    protocol::write_frame(&mut stream, &response)
}
