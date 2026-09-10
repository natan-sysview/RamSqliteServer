# Progreso de implementación: MVP inicial de RamSQLite

## Estado acumulado

**Modo:** Standard (`strict_tdd: false`)  
**Estrategia:** `feature-branch-chain`  
**Unidad actual:** PR 1 — crate, protocolo y admisión  
**Progreso:** 5 de 16 tareas completadas

## Tareas completadas

- [x] 1.1 Workspace Cargo, configuración, límites y protocolo JSON con longitud prefijada.
- [x] 1.2 Pruebas RED de admisión, duplicados, cuotas y conservación.
- [x] 1.3 Registro nombrado con reservas, listado observable y cierre explícito.
- [x] 1.4 Pruebas RED de rutas absolutas, `..` y enlaces fuera de la raíz.
- [x] 1.5 Resolución segura de rutas y rechazo de conexiones no loopback.

## Evidencia RED exigida por las tareas

| Tarea | Comando | Resultado RED |
|---|---|---|
| 1.2 | `cargo test -p ramsqlite-server --test admission` | Exit 101: `registry` aún no existía; las pruebas no podían compilar antes de la implementación. |
| 1.4 | `cargo test -p ramsqlite-server --test routes` | Exit 101: `ConfigError` y `resolve_existing_path` aún no existían. |

## Evidencia de unidad de trabajo

| Evidencia | Resultado exacto |
|---|---|
| Prueba enfocada | `cargo test -p ramsqlite-server --test admission`: exit 0, 4 aprobadas, 0 fallidas. |
| Seguridad de rutas | `cargo test -p ramsqlite-server --test routes`: exit 0, 4 aprobadas, 0 fallidas. |
| Harness de ejecución | `cargo test -p ramsqlite-server --test loopback`: exit 0, 1 aprobada, 0 fallidas; abrió un socket TCP real en `127.0.0.1:0`, creó `ventas` y la recuperó con `listar`. |
| Calidad estática | `cargo fmt --all -- --check` y `cargo clippy -p ramsqlite-server --all-targets -- -D warnings`: exit 0. |
| Reversión | Revertir `Cargo.toml`, `Cargo.lock`, `server/`, la regla `/target/` de `.gitignore` y los cambios de capacidades/pruebas en `openspec/config.yaml`; no afecta artefactos previos de propuesta, especificaciones ni diseño. |

## Límite de revisión

Esta unidad es la primera rebanada de la cadena y permanece aislada en `feat/initial-ramsqlite-mvp-01-foundation`. La implementación cohesiva supera 400 líneas por incluir crate, contrato, pruebas RED, seguridad de rutas y harness real; se reporta `size:exception` en vez de reducir pruebas o comprimir el código.

## Desviaciones

Ninguna desviación funcional. El receptor de esta unidad implementa únicamente `crear`, `listar` y `cerrar`; SQL, transacciones y copias permanecen para la fase 2.

## Pendiente

Tareas 2.1–4.3.
