# Tareas: MVP inicial de RamSQLite

## Pronóstico de carga de revisión

| Campo | Valor |
|---|---|
| Líneas estimadas | 1,400–2,200 |
| Riesgo de 400 líneas | Alto |
| PRs encadenados | Sí |
| División sugerida | PR 1 → PR 2 → PR 3 → PR 4 |
| Estrategia | auto-chain |
| Cadena | feature-branch-chain |

Decision needed before apply: No
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Unidades sugeridas

| Unidad | Meta | PR | Prueba enfocada | Harness | Reversión |
|---|---|---|---|---|---|
| 1 | Crate, protocolo y admisión | PR 1, base: rama tracker | `cargo test -p ramsqlite-server admission` | crear/listar por loopback | `server/` base |
| 2 | SQL, transacciones y copias | PR 2, base: rama PR 1 | `cargo test -p ramsqlite-server integration` | cliente JSON temporal | motor SQLite/copia |
| 3 | Clientes interoperables | PR 3, base: rama PR 2 | `dotnet test`; `pytest clients/python/tests` | C# inserta, Python lee | `clients/` |
| 4 | Benchmarks y documentación | PR 4, base: rama PR 3 | runner de benchmarks | tres comparadores | `benchmarks/`, `docs/` |

## Fase 1: Fundamentos y seguridad

- [x] 1.1 Crear `server/Cargo.toml` y `server/src/` con configuración loopback, raíz única, límites y protocolo JSON de longitud prefijada.
- [x] 1.2 Escribir RED en `server/tests/admission.rs` para nombres duplicados, máximos, cuota estimada y conservación de bases tras rechazo.
- [x] 1.3 Implementar registro nombrado, reserva conservadora, listado de capacidad y cierre explícito en `server/src/registry.rs`.
- [x] 1.4 Escribir RED en `server/tests/routes.rs` para ruta absoluta, `..` y enlace fuera de `data_root`, esperando `ruta_no_autorizada`.
- [x] 1.5 Implementar resolución segura de rutas relativas y rechazo de destinos no loopback en `server/src/config.rs` y `server/src/protocol.rs`.

## Fase 2: Motor por base

- [x] 2.1 Escribir RED en `server/tests/sql.rs` para parámetros incompatibles, SQL inválido, aislamiento por base y conflicto de escritor sin corrupción.
- [x] 2.2 Implementar trabajador FIFO acotado, conexión SQLite en RAM, SQL parametrizado y errores tipados en `server/src/database.rs`.
- [x] 2.3 Escribir RED en `server/tests/persistence.rs` para archivo inválido, fallo de guardado y persistencia parcial no compatible.
- [x] 2.4 Implementar carga y sincronización completa con Backup API, preservando RAM ante fallos, en `server/src/backup.rs`.
- [x] 2.5 Integrar receptor TCP y ciclo de conexión/transacción en `server/src/main.rs`; probar dos procesos y dos bases.

## Fase 3: Clientes y portabilidad

- [ ] 3.1 Crear `clients/csharp/` con cliente TCP, errores tipados y prueba de humo .NET contra servidor local.
- [ ] 3.2 Crear `clients/python/` con cliente TCP, errores tipados y prueba de humo `pytest`.
- [ ] 3.3 Añadir `tests/e2e/` para C# y Python sobre misma/distinta base; ejecutar la matriz macOS/Linux/Windows en CI.

## Fase 4: Evidencia y contrato

- [ ] 4.1 Crear `benchmarks/` con datos/cargas versionadas para RamSQLite, `:memory:` y WAL, incluidas escrituras y múltiples bases.
- [ ] 4.2 Registrar latencia, rendimiento, errores, entorno, límites y conclusión continuar/pivotar/detener en resultados reproducibles.
- [ ] 4.3 Documentar `docs/protocolo-local.md`: mensajes, parámetros, errores, raíz de archivos, un escritor y límites del MVP.
