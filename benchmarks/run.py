#!/usr/bin/env python3
"""Ejecuta una carga multiproceso equivalente contra los tres comparadores del MVP."""

from __future__ import annotations

import argparse
import json
import multiprocessing
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "clients" / "python" / "src"))
from ramsqlite_client import RamSqliteClient  # noqa: E402


SCHEMA = "CREATE TABLE IF NOT EXISTS mediciones (id INTEGER PRIMARY KEY, etiqueta TEXT NOT NULL, valor INTEGER NOT NULL)"


def load_workload(path: Path) -> dict[str, Any]:
    workload = json.loads(path.read_text(encoding="utf-8"))
    required = {
        "version", "worker_count", "named_databases", "seed_rows_per_database",
        "query_operations_per_worker", "write_operations_per_worker",
    }
    if set(workload) != required or workload["version"] != "v1":
        raise ValueError("La carga debe ser exactamente el formato versionado v1.")
    if workload["worker_count"] < 2 or len(workload["named_databases"]) < 2:
        raise ValueError("La carga debe usar al menos dos procesos y dos bases nombradas.")
    if len(set(workload["named_databases"])) != len(workload["named_databases"]):
        raise ValueError("Los nombres de bases deben ser únicos.")
    if any(not isinstance(workload[key], int) or workload[key] < 1 for key in required - {"version", "named_databases"}):
        raise ValueError("Los tamaños y cantidades de la carga deben ser enteros positivos.")
    return workload


def reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def seed(connection: sqlite3.Connection, rows: int) -> None:
    connection.execute(SCHEMA)
    connection.executemany(
        "INSERT INTO mediciones(id, etiqueta, valor) VALUES (?, ?, ?)",
        [(index, f"semilla-{index}", index) for index in range(rows)],
    )
    connection.commit()


def direct_worker(mode: str, root: str, workload: dict[str, Any], worker: int) -> tuple[int, int]:
    connections: dict[str, sqlite3.Connection] = {}
    for name in workload["named_databases"]:
        target = ":memory:" if mode == "memory" else str(Path(root) / f"{name}.sqlite")
        connection = sqlite3.connect(target, timeout=5)
        if mode == "wal":
            connection.execute("PRAGMA journal_mode=WAL")
            connection.execute("PRAGMA busy_timeout=5000")
        if mode == "memory":
            seed(connection, workload["seed_rows_per_database"])
        connections[name] = connection
    started = time.perf_counter_ns()
    try:
        for name, connection in connections.items():
            for _ in range(workload["query_operations_per_worker"]):
                connection.execute("SELECT COUNT(*) FROM mediciones WHERE valor >= ?", [0]).fetchone()
            for index in range(workload["write_operations_per_worker"]):
                row_id = workload["seed_rows_per_database"] + worker * 10_000 + index
                connection.execute("INSERT INTO mediciones(id, etiqueta, valor) VALUES (?, ?, ?)", [row_id, f"{name}-{worker}", row_id])
            connection.commit()
    finally:
        for connection in connections.values():
            connection.close()
    operations = len(workload["named_databases"]) * (workload["query_operations_per_worker"] + workload["write_operations_per_worker"])
    return operations, time.perf_counter_ns() - started


def ram_worker(port: int, workload: dict[str, Any], worker: int) -> tuple[int, int]:
    clients = {name: RamSqliteClient("127.0.0.1", port, name) for name in workload["named_databases"]}
    started = time.perf_counter_ns()
    for name, client in clients.items():
        for _ in range(workload["query_operations_per_worker"]):
            client.execute("SELECT COUNT(*) AS total FROM mediciones WHERE valor >= ?", [0])
        for index in range(workload["write_operations_per_worker"]):
            row_id = workload["seed_rows_per_database"] + worker * 10_000 + index
            client.execute("INSERT INTO mediciones(id, etiqueta, valor) VALUES (?, ?, ?)", [row_id, f"{name}-{worker}", row_id])
    operations = len(clients) * (workload["query_operations_per_worker"] + workload["write_operations_per_worker"])
    return operations, time.perf_counter_ns() - started


def worker_entry(queue: multiprocessing.Queue[tuple[str, int, int]], action: Callable[..., tuple[int, int]], *args: Any) -> None:
    try:
        operations, elapsed = action(*args)
        queue.put(("ok", operations, elapsed))
    except Exception as error:  # El resultado debe conservar errores de cada proceso.
        queue.put(("error", 0, 0))
        print(f"worker error: {error}", file=sys.stderr)


def run_workers(action: Callable[..., tuple[int, int]], arguments: list[tuple[Any, ...]]) -> dict[str, Any]:
    context = multiprocessing.get_context("spawn")
    queue = context.Queue()
    processes = [context.Process(target=worker_entry, args=(queue, action, *args)) for args in arguments]
    for process in processes:
        process.start()
    for process in processes:
        process.join()
    results = [queue.get() for _ in processes]
    errors = sum(1 for result in results if result[0] != "ok") + sum(process.exitcode != 0 for process in processes)
    operations = sum(result[1] for result in results)
    elapsed = max((result[2] for result in results), default=0)
    return {
        "operations": operations,
        "elapsed_ns": elapsed,
        "throughput_operations_per_second": operations * 1_000_000_000 / elapsed if elapsed else 0,
        "errors": errors,
    }


def seed_ram(port: int, workload: dict[str, Any]) -> None:
    for name in workload["named_databases"]:
        client = RamSqliteClient("127.0.0.1", port, name)
        client.create()
        client.execute(SCHEMA)
        for index in range(workload["seed_rows_per_database"]):
            client.execute("INSERT INTO mediciones(id, etiqueta, valor) VALUES (?, ?, ?)", [index, f"semilla-{index}", index])


def wait_for_server(port: int) -> None:
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        try:
            RamSqliteClient("127.0.0.1", port, "salud").create()
            return
        except Exception:
            time.sleep(0.05)
    raise TimeoutError("El servidor no abrió el puerto loopback.")


def run_benchmark(workload: dict[str, Any], binary: Path) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="ramsqlite-benchmark-") as temporary:
        temp = Path(temporary)
        wal_root = temp / "wal"
        wal_root.mkdir()
        for name in workload["named_databases"]:
            connection = sqlite3.connect(wal_root / f"{name}.sqlite")
            connection.execute("PRAGMA journal_mode=WAL")
            seed(connection, workload["seed_rows_per_database"])
            connection.close()
        port = reserve_port()
        process = subprocess.Popen([str(binary)], cwd=ROOT, env={**os.environ, "RAMSQLITE_LISTEN": f"127.0.0.1:{port}", "RAMSQLITE_DATA_ROOT": str(temp / "data")})
        try:
            wait_for_server(port)
            seed_ram(port, workload)
            comparators = {
                "ramsqlite": run_workers(ram_worker, [(port, workload, worker) for worker in range(workload["worker_count"])]),
                "sqlite_memory": run_workers(direct_worker, [("memory", str(temp), workload, worker) for worker in range(workload["worker_count"])]),
                "sqlite_wal": run_workers(direct_worker, [("wal", str(wal_root), workload, worker) for worker in range(workload["worker_count"])]),
            }
        finally:
            process.terminate()
            process.wait(timeout=5)
    return {"workload_version": workload["version"], "named_databases": workload["named_databases"], "worker_count": workload["worker_count"], "comparators": comparators}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workload", type=Path, default=ROOT / "benchmarks" / "workload-v1.json")
    parser.add_argument("--results", type=Path, help="Archivo JSON para una ejecución; no se versiona.")
    parser.add_argument("--server-binary", type=Path, default=ROOT / "target" / "debug" / ("ramsqlite-server.exe" if os.name == "nt" else "ramsqlite-server"))
    parser.add_argument("--validate-only", action="store_true")
    arguments = parser.parse_args()
    workload = load_workload(arguments.workload)
    if arguments.validate_only:
        print(f"Carga {workload['version']} válida: {workload['worker_count']} procesos y {len(workload['named_databases'])} bases.")
        return
    if not arguments.server_binary.is_file():
        raise SystemExit("No existe el binario del servidor; ejecute `cargo build -p ramsqlite-server`.")
    result = run_benchmark(workload, arguments.server_binary)
    serialized = json.dumps(result, indent=2, sort_keys=True)
    if arguments.results:
        arguments.results.write_text(serialized + "\n", encoding="utf-8")
    print(serialized)


if __name__ == "__main__":
    main()
