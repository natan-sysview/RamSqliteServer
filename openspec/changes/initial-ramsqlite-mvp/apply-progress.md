# Progreso de implementación: MVP inicial de RamSQLite

## Estado acumulado

**Modo:** Standard (`strict_tdd: false`)  
**Estrategia:** `feature-branch-chain`  
**Unidad actual:** PR 2 — motor SQL en RAM
**Progreso:** 7 de 16 tareas completadas

## Tareas completadas

- [x] 1.1 Workspace Cargo, configuración, límites y protocolo JSON con longitud prefijada.
- [x] 1.2 Pruebas RED de admisión, duplicados, cuotas y conservación.
- [x] 1.3 Registro nombrado con reservas, listado observable y cierre explícito.
- [x] 1.4 Pruebas RED de rutas absolutas, `..` y enlaces fuera de la raíz.
- [x] 1.5 Resolución segura de rutas y rechazo de conexiones no loopback.
- [x] 2.1 Pruebas de integración TCP para parámetros incompatibles, SQL inválido, aislamiento por base y conflicto de escritor sin corrupción.
- [x] 2.2 Trabajador FIFO acotado, SQLite `:memory:`, SQL parametrizado, transacciones y errores tipados.

## Evidencia RED exigida por las tareas

| Tarea | Comando | Resultado RED |
|---|---|---|
| 1.2 | `cargo test -p ramsqlite-server --test admission` | Exit 101: `registry` aún no existía; las pruebas no podían compilar antes de la implementación. |
| 1.4 | `cargo test -p ramsqlite-server --test routes` | Exit 101: `ConfigError` y `resolve_existing_path` aún no existían. |
| 2.1 | `cargo test -p ramsqlite-server --test sql` | Exit 101: las variantes `Ejecutar`, `IniciarTransaccion` y `RevertirTransaccion` aún no existían en el protocolo. |

## Evidencia de unidades de trabajo

| Unidad | Evidencia | Resultado exacto |
|---|---|---|
| PR 1 | Prueba enfocada | `cargo test -p ramsqlite-server --test admission`: exit 0, 4 aprobadas, 0 fallidas. |
| PR 1 | Seguridad de rutas | `cargo test -p ramsqlite-server --test routes`: exit 0, 4 aprobadas, 0 fallidas. |
| PR 1 | Harness de ejecución | `cargo test -p ramsqlite-server --test loopback`: exit 0, 1 aprobada, 0 fallidas; abrió un socket TCP real en `127.0.0.1:0`, creó `ventas` y la recuperó con `listar`. |
| PR 1 | Calidad estática | `cargo fmt --all -- --check` y `cargo clippy -p ramsqlite-server --all-targets -- -D warnings`: exit 0. |
| PR 2 | Prueba enfocada | `cargo test -p ramsqlite-server --test sql`: exit 0, 4 aprobadas, 0 fallidas. |
| PR 2 | Harness de ejecución | `cargo test -p ramsqlite-server --test sql`: exit 0, 4 aprobadas, 0 fallidas; cada caso abre TCP loopback real, envía JSON con longitud prefijada y verifica respuestas del trabajador SQLite. |
| PR 2 | Puertas de calidad | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace`: exit 0; 13 pruebas de integración aprobadas, 0 fallidas. |
| PR 2 | Reversión | Revertir `Cargo.lock`, `server/Cargo.toml`, `server/src/database.rs`, `server/src/lib.rs`, `server/src/protocol.rs`, `server/src/registry.rs`, `server/src/server.rs` y `server/tests/sql.rs`; restaura la base de admisión de PR 1 sin tocar propuesta, especificaciones ni diseño. |

## Límite de revisión

La unidad PR 2 pertenece a `feat/initial-ramsqlite-mvp-02-sql-engine` y parte de la rama de fundación PR 1. La implementación cohesiva supera el máximo de 400 líneas: añade el trabajador propietario, el contrato de SQL/transacciones, la integración del registro/receptor, la dependencia SQLite y cuatro pruebas TCP. Se requiere `size:exception`; no se comprimieron ni eliminaron pruebas para reducir artificialmente el cambio.

## Desviaciones

Ninguna desviación funcional. El protocolo expone valores de parámetros como valores JSON seguros (nulo, booleano, número y texto); arreglos y objetos devuelven `parametros` sin interpolarse en SQL. Las copias Backup API, carga y sincronización permanecen para las tareas 2.3–2.4.

## Pendiente

Tareas 2.3–4.3.
