from __future__ import annotations

import os
from pathlib import Path
import socket
import subprocess
import sys
import time

import pytest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "clients" / "python" / "src"))

from ramsqlite_client import ErrorCode, RamSqliteClient, RamSqliteError


WRITER_PROJECT = ROOT / "tests" / "e2e" / "csharp" / "RamSqlite.E2E.Writer.csproj"


def reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


@pytest.fixture(scope="module")
def server(tmp_path_factory: pytest.TempPathFactory) -> int:
    subprocess.run(["cargo", "build", "--package", "ramsqlite-server"], cwd=ROOT, check=True)
    port = reserve_port()
    executable = ROOT / "target" / "debug" / ("ramsqlite-server.exe" if os.name == "nt" else "ramsqlite-server")
    process = subprocess.Popen(
        [str(executable)],
        cwd=ROOT,
        env={
            **os.environ,
            "RAMSQLITE_LISTEN": f"127.0.0.1:{port}",
            "RAMSQLITE_DATA_ROOT": str(tmp_path_factory.mktemp("ramsqlite-e2e-data")),
        },
    )
    deadline = time.monotonic() + 5
    readiness_client = RamSqliteClient("127.0.0.1", port, "salud")
    while time.monotonic() < deadline:
        try:
            readiness_client.create()
            break
        except RamSqliteError as error:
            if error.code is not ErrorCode.CONNECTION:
                process.kill()
                process.wait()
                raise
            time.sleep(0.05)
    else:
        process.kill()
        process.wait()
        raise TimeoutError("El servidor real no abrió el puerto loopback.")

    yield port

    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def write_from_csharp(port: int, database: str, value: str) -> None:
    subprocess.run(
        [
            "dotnet",
            "run",
            "--project",
            str(WRITER_PROJECT),
            "--",
            "127.0.0.1",
            str(port),
            database,
            value,
        ],
        cwd=ROOT,
        check=True,
    )


def query_values(port: int, database: str) -> list[str]:
    result = RamSqliteClient("127.0.0.1", port, database).execute(
        "SELECT valor FROM registros ORDER BY valor"
    )
    return [row["valor"] for row in result["filas"]]


def test_csharp_confirma_insercion_y_python_observa_dato_compartido(server: int) -> None:
    write_from_csharp(server, "compartida", "confirmado-desde-csharp")

    assert query_values(server, "compartida") == ["confirmado-desde-csharp"]


def test_clientes_aislan_las_bases_por_nombre(server: int) -> None:
    write_from_csharp(server, "solo-csharp", "dato-csharp")

    python_client = RamSqliteClient("127.0.0.1", server, "solo-python")
    python_client.create()
    python_client.execute("CREATE TABLE registros (valor TEXT NOT NULL)")
    python_client.execute("INSERT INTO registros(valor) VALUES (?)", ["dato-python"])

    assert query_values(server, "solo-csharp") == ["dato-csharp"]
    assert query_values(server, "solo-python") == ["dato-python"]
