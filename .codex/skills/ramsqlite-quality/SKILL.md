---
name: ramsqlite-quality
description: "Trigger: Rust tests, calidad, Clippy, rustfmt, CI, cobertura, RamSQLite. Aplica las puertas de calidad del proyecto."
license: Apache-2.0
metadata:
  author: "natan-sysview"
  version: "1.0"
---

## Activation Contract

Aplica esta skill al modificar Rust, pruebas, GitHub Actions o contratos del servidor RamSQLite.

## Hard Rules

- Añade una prueba de regresión para cada defecto corregido.
- Coloca pruebas unitarias de lógica aislada en el módulo `server/src/` correspondiente con `#[cfg(test)]`; usa `server/tests/` para integración por API pública, TCP o varios componentes.
- No declares una tarea lista si fallan `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` o `cargo test --workspace`.
- Mantén las pruebas y documentación necesarias en el mismo cambio que el comportamiento que verifican.
- Cuando exista CI, ejecuta las mismas puertas en macOS, Linux y Windows; no sustituyas la matriz por una comprobación local.

## Decision Gates

| Cambio | Prueba mínima |
|---|---|
| Función o regla pura | Unitaria en el módulo afectado |
| Protocolo, red o varias capas | Integración en `server/tests/` |
| Cliente C# o Python | E2E con procesos reales |
| Rendimiento | Benchmark reproducible frente a `:memory:` y WAL |

## Execution Steps

1. Clasifica el cambio con la tabla y añade la prueba correspondiente antes de cerrar la tarea.
2. Ejecuta las tres puertas locales de formato, lints y pruebas.
3. Registra el comando y resultado en el artefacto SDD o resumen de entrega.
4. Si cambias CI, confirma que incluye la matriz macOS/Linux/Windows y las mismas puertas.

## Output Contract

Informa la clase de pruebas añadida, los tres comandos ejecutados y el resultado. Indica cualquier puerta que no pueda ejecutarse y por qué.

## References

- `../../../openspec/changes/initial-ramsqlite-mvp/design.md` — estrategia de pruebas del MVP.
- `../../../openspec/changes/initial-ramsqlite-mvp/tasks.md` — tareas y evidencia exigida.
