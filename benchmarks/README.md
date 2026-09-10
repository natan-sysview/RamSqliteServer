# Benchmarks del MVP

`workload-v1.json` fija datos, operaciones y dos procesos sobre `ventas` e
`inventario`. `run.py` ejecuta el binario real del servidor y compara esa carga
contra SQLite directo en `:memory:` y archivos SQLite en modo WAL.

```bash
cargo build -p ramsqlite-server
clients/python/.venv/bin/python benchmarks/run.py --results /tmp/ramsqlite-v1.json
```

Los resultados no se versionan: la tarea 4.2 registrará el entorno y una
conclusión. La medición de `:memory:` usa una base privada por proceso porque
SQLite directo no comparte una conexión en memoria entre procesos.
