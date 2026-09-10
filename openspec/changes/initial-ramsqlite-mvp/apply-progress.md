# Progreso de implementación: MVP inicial de RamSQLite

## Estado acumulado

**Modo:** Standard (`strict_tdd: false`)  
**Estrategia:** `feature-branch-chain`  
**Unidad actual:** PR 3 — clientes locales C# y Python
**Progreso:** 13 de 16 tareas completadas

## Tareas completadas

- [x] 1.1 Workspace Cargo, configuración, límites y protocolo JSON con longitud prefijada.
- [x] 1.2 Pruebas RED de admisión, duplicados, cuotas y conservación.
- [x] 1.3 Registro nombrado con reservas, listado observable y cierre explícito.
- [x] 1.4 Pruebas RED de rutas absolutas, `..` y enlaces fuera de la raíz.
- [x] 1.5 Resolución segura de rutas y rechazo de conexiones no loopback.
- [x] 2.1 Pruebas de integración TCP para parámetros incompatibles, SQL inválido, aislamiento por base y conflicto de escritor sin corrupción.
- [x] 2.2 Trabajador FIFO acotado, SQLite `:memory:`, SQL parametrizado, transacciones y errores tipados.
- [x] 2.3 Pruebas de integración TCP para archivo SQLite inválido, fallo de sincronización y solicitud parcial no compatible.
- [x] 2.4 Carga SQLite y sincronización completa explícita mediante SQLite Backup API, sin persistencia automática.
- [x] 2.5 Receptor TCP ejecutado como proceso real; dos clientes en procesos independientes usan dos bases nombradas sin mezclar datos.
- [x] 3.1 Cliente C# TCP local, errores tipados y prueba de humo .NET contra el binario real del servidor.
- [x] 3.2 Cliente Python TCP local, errores tipados y prueba de humo `pytest` contra el binario real del servidor.
- [x] 3.3 Prueba E2E de procesos C# y Python sobre la misma base y bases aisladas; workflow CI para macOS, Linux y Windows.

## Evidencia RED exigida por las tareas

| Tarea | Comando | Resultado RED |
|---|---|---|
| 1.2 | `cargo test -p ramsqlite-server --test admission` | Exit 101: `registry` aún no existía; las pruebas no podían compilar antes de la implementación. |
| 1.4 | `cargo test -p ramsqlite-server --test routes` | Exit 101: `ConfigError` y `resolve_existing_path` aún no existían. |
| 2.1 | `cargo test -p ramsqlite-server --test sql` | Exit 101: las variantes `Ejecutar`, `IniciarTransaccion` y `RevertirTransaccion` aún no existían en el protocolo. |
| 2.3 | `cargo test -p ramsqlite-server --test persistence` | Exit 101: `Registry::with_data_root` y las operaciones `Cargar`, `Sincronizar` y `SincronizarParcial` aún no existían. |

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
| PR 2 — copias SQLite | Prueba enfocada | `cargo test -p ramsqlite-server --test persistence`: exit 0, 4 aprobadas, 0 fallidas. |
| PR 2 — copias SQLite | Harness de ejecución | `cargo test -p ramsqlite-server --test persistence`: exit 0, 4 aprobadas, 0 fallidas; cada escenario abre TCP loopback real y verifica carga, sincronización o conservación de RAM. |
| PR 2 — copias SQLite | Puertas de calidad | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` y `git diff --check`: exit 0; 17 pruebas de integración aprobadas, 0 fallidas. |
| PR 2 — copias SQLite | Reversión | Revertir `server/Cargo.toml`, `server/src/backup.rs`, `server/src/database.rs`, `server/src/lib.rs`, `server/src/main.rs`, `server/src/protocol.rs`, `server/src/registry.rs`, `server/src/server.rs` y `server/tests/persistence.rs`; elimina carga/sincronización sin afectar el motor SQL ya existente. |
| PR 2 — ciclo multiproceso | Prueba enfocada | `cargo test -p ramsqlite-server --test multiprocess_runtime`: exit 0, 2 aprobadas, 0 fallidas. |
| PR 2 — ciclo multiproceso | Harness de ejecución | `cargo test -p ramsqlite-server --test multiprocess_runtime`: exit 0; inicia `ramsqlite-server` como proceso hijo con raíz temporal y loopback, luego inicia dos procesos cliente que crean, escriben y verifican `ventas` e `inventario` sin mezclar filas. |
| PR 2 — ciclo multiproceso | Puertas de calidad | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` y `git diff --check`: exit 0; 19 pruebas de integración aprobadas, 0 fallidas. |
| PR 2 — ciclo multiproceso | Reversión | Revertir `server/src/main.rs` y `server/tests/multiprocess_runtime.rs`; elimina la configuración de prueba por entorno y el harness de procesos sin modificar el motor SQLite, protocolo ni persistencia. |
| PR 3 — cliente C# | Prueba enfocada | `dotnet test clients/csharp/tests/RamSqlite.Client.Tests/RamSqlite.Client.Tests.csproj`: exit 0, 1 aprobada, 0 fallidas. |
| PR 3 — cliente C# | Harness de ejecución | La misma prueba compila el binario Rust real, lo inicia en un puerto TCP loopback temporal con una raíz temporal, crea la base `humo`, ejecuta DDL/DML/consulta parametrizada y recibe un error SQL tipado; la finalización mata el proceso y borra la raíz temporal. Exit 0, 1 aprobada. |
| PR 3 — cliente C# | Puertas de calidad | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace`: exit 0; 19 pruebas Rust aprobadas, 0 fallidas. Las pruebas loopback requieren ejecutarse fuera del sandbox de archivos para poder abrir sockets TCP locales. |
| PR 3 — cliente C# | Reversión | Revertir `.gitignore` y eliminar `clients/csharp/`; retira el cliente y la prueba C# sin tocar el servidor Rust ni las tareas de Python, CI o benchmarks. |
| PR 3 — cliente Python | Prueba enfocada | `clients/python/.venv/bin/python -m pytest clients/python/tests`: exit 0, 1 aprobada, 0 fallidas. |
| PR 3 — cliente Python | Harness de ejecución | La misma prueba compila el binario Rust real, lo inicia en un puerto TCP loopback temporal con una raíz temporal, crea la base `humo`, ejecuta DDL/DML/consulta parametrizada y recibe un error SQL tipado; la limpieza termina el proceso y elimina sus recursos temporales. Exit 0, 1 aprobada. |
| PR 3 — cliente Python | Puertas de calidad | `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo test --workspace` y `git diff --check`: exit 0; 19 pruebas Rust aprobadas, 0 fallidas. Las pruebas loopback requieren ejecutarse fuera del sandbox de archivos para poder abrir sockets TCP locales. |
| PR 3 — cliente Python | Reversión | Revertir `.gitignore` y eliminar `clients/python/`; retira el cliente y la prueba Python sin tocar el servidor Rust, el cliente C# ni las tareas E2E, CI o benchmarks. |
| PR 3 — interoperabilidad y CI | Prueba enfocada | `clients/python/.venv/bin/python -m pytest tests/e2e`: exit 0, 2 aprobadas, 0 fallidas. |
| PR 3 — interoperabilidad y CI | Harness de ejecución | La prueba inicia el binario Rust real con raíz temporal y loopback, ejecuta el escritor C# como proceso `dotnet run`, y usa el proceso Python de `pytest` para confirmar tanto la fila compartida como el aislamiento de `solo-csharp` y `solo-python`. Exit 0, 2 aprobadas. |
| PR 3 — interoperabilidad y CI | Puertas de calidad | `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo test --workspace`: exit 0, 19 pruebas aprobadas. `dotnet test clients/csharp/tests/RamSqlite.Client.Tests/RamSqlite.Client.Tests.csproj`: exit 0, 1 aprobada. `clients/python/.venv/bin/python -m pytest clients/python/tests tests/e2e`: exit 0, 3 aprobadas. |
| PR 3 — interoperabilidad y CI | CI | `.github/workflows/ci.yml` ejecuta las puertas Rust, C#, Python y E2E en `ubuntu-latest`, `macos-latest` y `windows-latest`. La matriz solo puede ejecutarse en GitHub Actions, no localmente. |
| PR 3 — interoperabilidad y CI | Reversión | Revertir `.github/workflows/ci.yml`, las reglas E2E de `.gitignore` y eliminar `tests/e2e/`; retira el harness y CI sin modificar el servidor ni los clientes publicados. |

## Límite de revisión

La unidad PR 2 pertenece a `feat/initial-ramsqlite-mvp-02-sql-engine` y parte de la rama de fundación PR 1. La implementación cohesiva supera el máximo de 400 líneas: añade el trabajador propietario, el contrato de SQL/transacciones, la integración del registro/receptor, la dependencia SQLite y cuatro pruebas TCP. Se requiere `size:exception`; no se comprimieron ni eliminaron pruebas para reducir artificialmente el cambio.

La subunidad `sqlite-backup-persistence` añade 426 líneas y elimina 27 respecto de `0cc4ea8`, sobre el presupuesto de 400. El excedente es coherente: incorpora cuatro escenarios TCP de persistencia, el módulo Backup API y el contrato de tres operaciones; no se redujo artificialmente.

## Desviaciones

Ninguna desviación funcional. El protocolo expone valores de parámetros como valores JSON seguros (nulo, booleano, número y texto); arreglos y objetos devuelven `parametros` sin interpolarse en SQL. Las copias Backup API, carga y sincronización se implementaron en las tareas 2.3–2.4. La sincronización solo ocurre por solicitud explícita y no incluye migración parcial de tablas. El binario admite `RAMSQLITE_LISTEN` y `RAMSQLITE_DATA_ROOT` para las pruebas de proceso real; sin esas variables conserva `127.0.0.1:7432` y `./data`.

## Pendiente

Tareas 3.3–4.3.
