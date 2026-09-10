# Benchmarks del MVP

`workload-v1.json` fija datos, operaciones y dos procesos sobre `ventas` e
`inventario`. `run.py` ejecuta el binario real del servidor y compara esa carga
contra SQLite directo en `:memory:` y archivos SQLite en modo WAL.

```bash
cargo build -p ramsqlite-server
clients/python/.venv/bin/python benchmarks/run.py --results /tmp/ramsqlite-v1.json
```

El JSON de cada ejecución no se versiona porque sus tiempos dependen de la
máquina. La [captura reproducible v1](resultados-v1.md) registra el entorno,
las métricas, sus límites y la decisión actual. La medición de `:memory:` usa
una base privada por proceso porque SQLite directo no comparte una conexión en
memoria entre procesos.
