# Guía breve de planificación de RamSQLite

Esta guía explica dónde se guarda cada cosa y qué necesitas revisar. No hace falta leer todos los artefactos SDD.

## Camino rápido

1. Revisa el resumen de cada fase en el chat.
2. Abre `openspec/changes/initial-ramsqlite-mvp/proposal.md` si quieres validar qué incluye el MVP.
3. Antes de programar, revisaremos juntos `tasks.md` y elegiremos cómo dividir el trabajo.

## Dos directorios distintos

| Directorio | Para qué sirve | Estado actual |
|---|---|---|
| `openspec/changes/` | Trabajo en curso: investigación, propuesta, requisitos, diseño y tareas. Puede cambiar. | Contiene el MVP `initial-ramsqlite-mvp`. |
| `openspec/specs/` | Reglas oficiales de un producto ya implementado, probado y aprobado. | Está vacío hasta que terminemos el MVP. |

Cuando un cambio termina, sus especificaciones se promueven a `openspec/specs/` y su carpeta de trabajo se conserva en `openspec/changes/archive/` como historial.

## Qué hay en el cambio actual

```text
openspec/changes/initial-ramsqlite-mvp/
├── exploration.md  # Por qué vale la pena explorar RamSQLite
├── proposal.md     # Alcance del MVP y lo que queda fuera
├── specs/          # Requisitos comprobables por capacidad
├── design.md       # Cómo se organizará técnicamente
└── tasks.md        # Pasos pequeños para implementar
```

## Qué debes leer

| Si quieres saber... | Abre... |
|---|---|
| Qué hará la primera versión | `proposal.md` |
| Qué pasa si no hay RAM | `specs/control-de-capacidad/spec.md` |
| Cómo se carga o guarda una base | `specs/persistencia-solicitada-por-cliente/spec.md` |
| Cómo se conectan clientes C#/Python | `design.md` |
| En qué orden construiremos el producto | `tasks.md` |

## Qué sigue

El SDD ya está planeado. Antes de crear código, revisaremos `tasks.md` y elegiremos una estrategia para dividir el MVP en cuatro entregas revisables.
