use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "operacion", rename_all = "snake_case")]
pub enum Request {
    Crear {
        id_solicitud: String,
        base: String,
        bytes_estimados: Option<u64>,
    },
    Listar {
        id_solicitud: String,
    },
    Cerrar {
        id_solicitud: String,
        base: String,
    },
    Ejecutar {
        id_solicitud: String,
        base: String,
        id_conexion: String,
        sql: String,
        parametros: Vec<serde_json::Value>,
    },
    IniciarTransaccion {
        id_solicitud: String,
        base: String,
        id_conexion: String,
    },
    ConfirmarTransaccion {
        id_solicitud: String,
        base: String,
        id_conexion: String,
    },
    RevertirTransaccion {
        id_solicitud: String,
        base: String,
        id_conexion: String,
    },
}

impl Request {
    pub fn id(&self) -> &str {
        match self {
            Self::Crear { id_solicitud, .. }
            | Self::Listar { id_solicitud }
            | Self::Cerrar { id_solicitud, .. }
            | Self::Ejecutar { id_solicitud, .. }
            | Self::IniciarTransaccion { id_solicitud, .. }
            | Self::ConfirmarTransaccion { id_solicitud, .. }
            | Self::RevertirTransaccion { id_solicitud, .. } => id_solicitud,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub id_solicitud: String,
    pub resultado: Option<serde_json::Value>,
    pub error: Option<ProtocolError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProtocolError {
    pub codigo: String,
    pub mensaje: String,
}

pub fn require_loopback(address: SocketAddr) -> Result<(), ProtocolError> {
    if address.ip().is_loopback() {
        Ok(())
    } else {
        Err(ProtocolError {
            codigo: "destino_no_local".into(),
            mensaje: "solo se permiten conexiones loopback".into(),
        })
    }
}

pub fn read_frame<R: Read>(reader: &mut R, max: usize) -> io::Result<Vec<u8>> {
    let mut prefix = [0; 4];
    reader.read_exact(&mut prefix)?;
    let size = u32::from_be_bytes(prefix) as usize;
    if size > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "mensaje demasiado grande",
        ));
    }
    let mut payload = vec![0; size];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

pub fn write_frame<W: Write, T: Serialize>(writer: &mut W, value: &T) -> io::Result<()> {
    let payload = serde_json::to_vec(value).map_err(io::Error::other)?;
    let size = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "mensaje demasiado grande"))?;
    writer.write_all(&size.to_be_bytes())?;
    writer.write_all(&payload)
}
