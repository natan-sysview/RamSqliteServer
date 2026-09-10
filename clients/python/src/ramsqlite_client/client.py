"""Cliente JSON con longitud prefijada para el servidor local RamSQLite."""

from __future__ import annotations

from enum import Enum
import ipaddress
import json
import socket
from typing import Any
from uuid import uuid4


class ErrorCode(str, Enum):
    """Códigos de error que una aplicación puede tratar de forma distinguible."""

    CONNECTION = "conexion"
    CAPACITY = "capacidad"
    DATABASE_NOT_FOUND = "base_no_existe"
    SQL = "sql"
    PARAMETERS = "parametros"
    TRANSACTION = "transaccion"
    PERSISTENCE = "persistencia"
    PROTOCOL = "protocolo"
    UNKNOWN = "desconocido"


class RamSqliteError(Exception):
    """Error local o remoto de RamSQLite con un código estable."""

    def __init__(self, code: ErrorCode, message: str, cause: Exception | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.__cause__ = cause


class RamSqliteClient:
    """Cliente sin estado de transporte para una base nombrada en un servidor loopback."""

    MAXIMUM_FRAME_BYTES = 4 * 1024 * 1024

    def __init__(
        self,
        host: str,
        port: int,
        database: str,
        connection_id: str | None = None,
        timeout: float = 5.0,
    ) -> None:
        try:
            address = ipaddress.ip_address(host)
        except ValueError as exception:
            raise ValueError("El host debe ser una dirección IP loopback.") from exception
        if not address.is_loopback:
            raise ValueError("El cliente solo admite direcciones loopback.")
        if not 1 <= port <= 65535:
            raise ValueError("El puerto debe estar entre 1 y 65535.")
        if not database or database.isspace():
            raise ValueError("El nombre de la base es obligatorio.")
        if timeout <= 0:
            raise ValueError("El tiempo de espera debe ser positivo.")

        self.host = host
        self.port = port
        self.database = database
        self.connection_id = connection_id or uuid4().hex
        self.timeout = timeout

    def create(self, estimated_bytes: int | None = None) -> dict[str, Any]:
        request = self._request("crear")
        request["bytes_estimados"] = estimated_bytes
        return self._send(request)

    def execute(self, sql: str, parameters: list[Any] | tuple[Any, ...] | None = None) -> dict[str, Any]:
        if not sql or sql.isspace():
            raise ValueError("La sentencia SQL es obligatoria.")
        request = self._request("ejecutar")
        request.update(
            id_conexion=self.connection_id,
            sql=sql,
            parametros=list(parameters or ()),
        )
        return self._send(request)

    def begin_transaction(self) -> dict[str, Any]:
        return self._transaction("iniciar_transaccion")

    def commit_transaction(self) -> dict[str, Any]:
        return self._transaction("confirmar_transaccion")

    def rollback_transaction(self) -> dict[str, Any]:
        return self._transaction("revertir_transaccion")

    def _transaction(self, operation: str) -> dict[str, Any]:
        request = self._request(operation)
        request["id_conexion"] = self.connection_id
        return self._send(request)

    def _request(self, operation: str) -> dict[str, Any]:
        return {
            "operacion": operation,
            "id_solicitud": uuid4().hex,
            "base": self.database,
        }

    def _send(self, request: dict[str, Any]) -> dict[str, Any]:
        try:
            payload = json.dumps(request, separators=(",", ":")).encode("utf-8")
        except (TypeError, ValueError) as exception:
            raise RamSqliteError(ErrorCode.PROTOCOL, "La solicitud no se puede codificar como JSON.", exception) from exception
        if len(payload) > self.MAXIMUM_FRAME_BYTES:
            raise RamSqliteError(ErrorCode.PROTOCOL, "La solicitud supera el tamaño permitido.")

        try:
            with socket.create_connection((self.host, self.port), timeout=self.timeout) as connection:
                connection.sendall(len(payload).to_bytes(4, "big") + payload)
                response_size = int.from_bytes(self._receive_exactly(connection, 4), "big")
                if response_size > self.MAXIMUM_FRAME_BYTES:
                    raise RamSqliteError(ErrorCode.PROTOCOL, "La respuesta supera el tamaño permitido.")
                response = json.loads(self._receive_exactly(connection, response_size))
        except RamSqliteError:
            raise
        except (OSError, TimeoutError) as exception:
            raise RamSqliteError(ErrorCode.CONNECTION, "No fue posible conectar al servidor local.", exception) from exception
        except (UnicodeDecodeError, json.JSONDecodeError) as exception:
            raise RamSqliteError(ErrorCode.PROTOCOL, "El servidor devolvió JSON inválido.", exception) from exception

        error = response.get("error")
        if error is not None:
            code = ErrorCode._value2member_map_.get(error.get("codigo"), ErrorCode.UNKNOWN)
            raise RamSqliteError(code, error.get("mensaje", "El servidor rechazó la solicitud."))
        result = response.get("resultado")
        if result is None:
            raise RamSqliteError(ErrorCode.PROTOCOL, "La respuesta no contiene resultado.")
        if not isinstance(result, dict):
            raise RamSqliteError(ErrorCode.PROTOCOL, "El resultado del servidor no es un objeto JSON.")
        return result

    @staticmethod
    def _receive_exactly(connection: socket.socket, size: int) -> bytes:
        buffer = bytearray()
        while len(buffer) < size:
            chunk = connection.recv(size - len(buffer))
            if not chunk:
                raise RamSqliteError(ErrorCode.CONNECTION, "El servidor cerró la conexión antes de completar la respuesta.")
            buffer.extend(chunk)
        return bytes(buffer)
