# Resultado reproducible: carga v1

**Decisión: pivotar.** Esta primera medición no justifica presentar
RamSQLite como una optimización general de SQL. Sí valida el servidor, el
contrato local y la coordinación de procesos; el siguiente trabajo debe medir
casos donde compartir una base nombrada entre procesos sea el objetivo, no
competir con SQLite directo dentro de cada proceso.

## Reproducir

```bash
cargo build -p ramsqlite-server
clients/python/.venv/bin/python -m pytest benchmarks/tests
clients/python/.venv/bin/python benchmarks/run.py --results /tmp/ramsqlite-v1-results.json
```

El JSON indicado conserva los valores de una ejecución. El archivo no se
versiona porque sus tiempos dependen de la máquina; esta página conserva una
captura trazable y el método. Antes de comparar cambios, repetir la carga con
el mismo binario, carga y entorno, y guardar el JSON generado fuera del
repositorio.

## Carga y límites

| Elemento | Valor |
|---|---|
| Versión de carga | `v1` (`benchmarks/workload-v1.json`) |
| Procesos | 2, creados con `spawn` |
| Bases | `ventas`, `inventario` |
| Semilla | 100 filas por base |
| Por proceso y base | 20 consultas `COUNT` + 10 inserciones |
| Operaciones agregadas | 120 |
| Límites RamSQLite | 32 bases; 1 GiB total; 256 MiB por base; reserva inicial 1 MiB; frame máximo 8 MiB |
| Persistencia | No se solicita durante esta carga |

`sqlite_memory` usa una base `:memory:` **privada por proceso**, ya que SQLite
directo no comparte una conexión en memoria entre procesos. `sqlite_wal` usa
un archivo por base bajo un directorio temporal, con `journal_mode=WAL` y
`busy_timeout=5000` ms. RamSQLite ejecuta el binario Rust real por TCP
loopback y una base en memoria por nombre.

## Captura inicial

Fecha de ejecución: **2026-09-10**. Revisión medida:
`8f8aaa2edf58ee74c37c528ba165aefc2d7bb256`.

| Entorno | Valor |
|---|---|
| Sistema | macOS 15.5 (24F74), `arm64` |
| Rust / Cargo | 1.88.0 / 1.88.0 |
| Python / pytest | 3.13.4 / 9.1.1 |
| SQLite de Python | 3.50.1 |
| Perfil Rust | `dev` sin optimización de lanzamiento |

| Comparador | Operaciones | Errores | Tiempo agregado (ns) | Rendimiento (ops/s) | Latencia aproximada (ms/op) |
|---|---:|---:|---:|---:|---:|
| RamSQLite | 120 | 0 | 12,961,708 | 9,258.04 | 0.1080 |
| SQLite `:memory:` directo | 120 | 0 | 137,791 | 870,884.17 | 0.0011 |
| SQLite WAL | 120 | 0 | 848,709 | 141,391.22 | 0.0071 |

El rendimiento se calcula como `operaciones / max(tiempo_de_cada_proceso)`.
La latencia es un **promedio aproximado** `tiempo_agregado / operaciones`; no
es un percentil ni una latencia de ida y vuelta individual. El runner v1 no
recoge distribuciones por operación, por lo que no permite afirmar p50/p95 ni
atribuir el costo entre IPC, serialización y SQLite.

## Interpretación y siguiente decisión

- Los tres comparadores terminaron las 120 operaciones sin error.
- En esta carga pequeña, RamSQLite fue más lento que `:memory:` directo y WAL.
  Eso es esperable: añade IPC TCP, JSON y un trabajador propietario antes de
  ejecutar SQL.
- `:memory:` directo no necesita IPC y por ello no es un sustituto compartido
  entre procesos; sigue siendo la mejor referencia cuando un solo proceso es
  suficiente.
- WAL resuelve una parte del caso multiproceso sin introducir el contrato de
  RamSQLite. Esta captura no demuestra una ventaja de rendimiento o latencia
  relevante sobre WAL.

Por tanto, el resultado es **pivotar**, no continuar como promesa de velocidad
SQL universal ni detener el experimento. Para reevaluar, la próxima carga debe
medir una necesidad real de coordinación: coste de reabrir/cargar una base
compartida, varias bases grandes, tamaño de datos documentado, contención de
un escritor y percentiles de latencia. Si esas mediciones tampoco muestran
ventaja de usabilidad o rendimiento frente a WAL, la recomendación pasa a
**detener** el MVP antes de estabilizar el protocolo.

## Verificación de esta captura

- `clients/python/.venv/bin/python -m pytest benchmarks/tests` — 2 aprobadas,
  0 fallidas.
- `clients/python/.venv/bin/python benchmarks/run.py --results
  /tmp/ramsqlite-v1-results.json` — exit 0; tres comparadores y 120
  operaciones sin errores cada uno.
