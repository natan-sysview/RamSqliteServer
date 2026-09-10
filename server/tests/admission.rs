use ramsqlite_server::{
    config::Limits,
    registry::{AdmissionError, Registry},
};

fn limits(max_databases: usize, total: u64, per_database: u64) -> Limits {
    Limits {
        max_databases,
        max_total_bytes: total,
        max_database_bytes: per_database,
        new_database_reservation_bytes: 10,
        ..Limits::default()
    }
}

#[test]
fn rechaza_duplicados_sin_reemplazar_la_base() {
    let registry = Registry::new(limits(2, 100, 100));
    registry.create("ventas", Some(10)).unwrap();

    assert_eq!(
        registry.create("ventas", Some(20)),
        Err(AdmissionError::Duplicate)
    );
    assert_eq!(registry.snapshot().databases[0].reserved_bytes, 10);
}

#[test]
fn aplica_maximo_y_conserva_bases_tras_rechazo() {
    let registry = Registry::new(limits(1, 100, 100));
    registry.create("ventas", None).unwrap();

    assert_eq!(
        registry.create("otra", None),
        Err(AdmissionError::MaximumDatabases)
    );
    assert_eq!(registry.snapshot().databases.len(), 1);
}

#[test]
fn aplica_cuotas_por_base_y_total() {
    let registry = Registry::new(limits(3, 15, 12));
    assert_eq!(
        registry.create("grande", Some(13)),
        Err(AdmissionError::PerDatabase)
    );
    registry.create("ventas", Some(10)).unwrap();
    assert_eq!(
        registry.create("inventario", Some(6)),
        Err(AdmissionError::Total)
    );
    assert_eq!(registry.snapshot().used_bytes, 10);
}

#[test]
fn cerrar_libera_la_reserva() {
    let registry = Registry::new(limits(1, 10, 10));
    registry.create("ventas", Some(10)).unwrap();
    registry.close("ventas").unwrap();
    registry.create("inventario", Some(10)).unwrap();
    assert_eq!(registry.snapshot().databases[0].name, "inventario");
}
