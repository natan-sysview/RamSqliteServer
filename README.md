# RamSQLite Server

RamSQLite Server es un proyecto open source en fase de diseño. Su objetivo es que varios procesos locales compartan bases SQLite nombradas mantenidas en RAM por un servidor escrito en Rust.

## Estado actual

Todavía no existe código de aplicación. El repositorio contiene la propuesta, las especificaciones, el diseño técnico y el plan de implementación del MVP.

## Alcance previsto del MVP

- Servidor local escrito en Rust.
- Múltiples bases SQLite nombradas en memoria.
- Clientes iniciales para C# y Python.
- Consultas SQL parametrizadas y transacciones.
- Carga de archivos SQLite existentes a RAM.
- Sincronización completa a disco bajo petición del cliente.
- Límites de admisión conservadores sin expulsión automática de bases.
- Compatibilidad prevista con macOS, Linux y Windows.

## Planificación

- [Guía breve de SDD](GUIA-SDD.md)
- [Propuesta del MVP](openspec/changes/initial-ramsqlite-mvp/proposal.md)
- [Diseño técnico](openspec/changes/initial-ramsqlite-mvp/design.md)
- [Plan de implementación](openspec/changes/initial-ramsqlite-mvp/tasks.md)

Los artefactos dentro de `openspec/changes/` representan trabajo activo. Las especificaciones se promoverán a `openspec/specs/` cuando el MVP esté implementado, verificado y archivado.

## Licencia

Apache License 2.0. Consulta [LICENSE](LICENSE).
