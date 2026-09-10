from __future__ import annotations

import os
from pathlib import Path
import socket
import subprocess
import sys
import time

import pytest

sys.path.insert(0, str(Path(__file__).parents[1] / "src"))

from ramsqlite_client import ErrorCode, RamSqliteClient, RamSqliteError


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


@pytest.fixture(scope="module")
def server(tmp_path_factory: pytest.TempPathFactory) -> int:
    root = repository_root()
    subprocess.run(["cargo", "build", "--package", "ramsqlite-server"], cwd=root, check=True)
    port = reserve_port()
    data_root = tmp_path_factory.mktemp("ramsqlite-python-data")
    executable = root / "target" / "debug" / ("ramsqlite-server.exe" if os.name == "nt" else "ramsqlite-server")
    process = subprocess.Popen(
        [str(executable)],
        cwd=root,
        env={
            **os.environ,
            "RAMSQLITE_LISTEN": f"127.0.0.1:{port}",
            "RAMSQLITE_DATA_ROOT": str(data_root),
        },
    )
    deadline = time.monotonic() + 5
    client = RamSqliteClient("127.0.0.1", port, "humo")
    while time.monotonic() < deadline:
        try:
            client.create()
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


def test_cliente_tcp_crea_ejecuta_sql_parametrizado_y_expone_error_sql(server: int) -> None:
    client = RamSqliteClient("127.0.0.1", server, "humo")
    client.execute("CREATE TABLE productos (nombre TEXT NOT NULL)")
    client.execute("INSERT INTO productos(nombre) VALUES (?)", ["teclado"])

    result = client.execute("SELECT nombre FROM productos WHERE nombre = ?", ["teclado"])
    assert result["filas"] == [{"nombre": "teclado"}]

    with pytest.raises(RamSqliteError) as caught:
        client.execute("ESTO NO ES SQL")
    assert caught.value.code is ErrorCode.SQL
