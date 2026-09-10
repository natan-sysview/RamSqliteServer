using System.Buffers.Binary;
using System.Net;
using System.Net.Sockets;
using System.Text.Json;

namespace RamSqlite.Client;

public enum RamSqliteErrorCode
{
    Connection,
    Capacity,
    DatabaseNotFound,
    Sql,
    Parameters,
    Transaction,
    Persistence,
    Protocol,
    Unknown,
}

public sealed class RamSqliteException : Exception
{
    public RamSqliteException(RamSqliteErrorCode code, string message, Exception? innerException = null)
        : base(message, innerException) => Code = code;

    public RamSqliteErrorCode Code { get; }
}

public sealed class RamSqliteClient
{
    private const int MaximumFrameBytes = 4 * 1024 * 1024;
    private readonly IPEndPoint endpoint;
    private readonly string connectionId;

    public RamSqliteClient(IPAddress address, int port, string database, string? connectionId = null)
    {
        if (!IPAddress.IsLoopback(address))
            throw new ArgumentException("El cliente solo admite direcciones loopback.", nameof(address));
        if (port is < 1 or > ushort.MaxValue)
            throw new ArgumentOutOfRangeException(nameof(port));
        if (string.IsNullOrWhiteSpace(database))
            throw new ArgumentException("El nombre de la base es obligatorio.", nameof(database));

        endpoint = new IPEndPoint(address, port);
        Database = database;
        this.connectionId = connectionId ?? Guid.NewGuid().ToString("N");
    }

    public string Database { get; }

    public async Task CreateAsync(ulong? estimatedBytes = null, CancellationToken cancellationToken = default)
    {
        var request = Request("crear");
        request["bytes_estimados"] = estimatedBytes;
        await SendAsync(request, cancellationToken);
    }

    public Task<JsonElement> ExecuteAsync(
        string sql,
        IReadOnlyList<object?>? parameters = null,
        CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(sql);
        var request = Request("ejecutar");
        request["id_conexion"] = connectionId;
        request["sql"] = sql;
        request["parametros"] = parameters ?? Array.Empty<object?>();
        return SendAsync(request, cancellationToken);
    }

    public async Task BeginTransactionAsync(CancellationToken cancellationToken = default)
    {
        var request = Request("iniciar_transaccion");
        request["id_conexion"] = connectionId;
        await SendAsync(request, cancellationToken);
    }

    public async Task CommitTransactionAsync(CancellationToken cancellationToken = default)
    {
        var request = Request("confirmar_transaccion");
        request["id_conexion"] = connectionId;
        await SendAsync(request, cancellationToken);
    }

    public async Task RollbackTransactionAsync(CancellationToken cancellationToken = default)
    {
        var request = Request("revertir_transaccion");
        request["id_conexion"] = connectionId;
        await SendAsync(request, cancellationToken);
    }

    private Dictionary<string, object?> Request(string operation) => new()
    {
        ["operacion"] = operation,
        ["id_solicitud"] = Guid.NewGuid().ToString("N"),
        ["base"] = Database,
    };

    private async Task<JsonElement> SendAsync(
        Dictionary<string, object?> request,
        CancellationToken cancellationToken)
    {
        var payload = JsonSerializer.SerializeToUtf8Bytes(request);
        if (payload.Length > MaximumFrameBytes)
            throw new RamSqliteException(RamSqliteErrorCode.Protocol, "La solicitud supera el tamaño permitido.");

        try
        {
            using var tcpClient = new TcpClient(endpoint.AddressFamily);
            await tcpClient.ConnectAsync(endpoint, cancellationToken);
            await using var stream = tcpClient.GetStream();
            var prefix = new byte[sizeof(uint)];
            BinaryPrimitives.WriteUInt32BigEndian(prefix, checked((uint)payload.Length));
            await stream.WriteAsync(prefix, cancellationToken);
            await stream.WriteAsync(payload, cancellationToken);

            await ReadExactlyAsync(stream, prefix, cancellationToken);
            var responseLength = BinaryPrimitives.ReadUInt32BigEndian(prefix);
            if (responseLength > MaximumFrameBytes)
                throw new RamSqliteException(RamSqliteErrorCode.Protocol, "La respuesta supera el tamaño permitido.");

            var response = new byte[checked((int)responseLength)];
            await ReadExactlyAsync(stream, response, cancellationToken);
            using var document = JsonDocument.Parse(response);
            var root = document.RootElement;
            if (root.TryGetProperty("error", out var error) && error.ValueKind != JsonValueKind.Null)
                throw ServerError(error);
            if (!root.TryGetProperty("resultado", out var result) || result.ValueKind == JsonValueKind.Null)
                throw new RamSqliteException(RamSqliteErrorCode.Protocol, "La respuesta no contiene resultado.");
            return result.Clone();
        }
        catch (RamSqliteException)
        {
            throw;
        }
        catch (SocketException exception)
        {
            throw new RamSqliteException(RamSqliteErrorCode.Connection, "No fue posible conectar al servidor local.", exception);
        }
        catch (IOException exception)
        {
            throw new RamSqliteException(RamSqliteErrorCode.Connection, "La conexión con el servidor local se interrumpió.", exception);
        }
        catch (JsonException exception)
        {
            throw new RamSqliteException(RamSqliteErrorCode.Protocol, "El servidor devolvió JSON inválido.", exception);
        }
    }

    private static async Task ReadExactlyAsync(NetworkStream stream, byte[] buffer, CancellationToken cancellationToken)
    {
        var received = 0;
        while (received < buffer.Length)
        {
            var read = await stream.ReadAsync(buffer.AsMemory(received), cancellationToken);
            if (read == 0)
                throw new IOException("El servidor cerró la conexión antes de completar la respuesta.");
            received += read;
        }
    }

    private static RamSqliteException ServerError(JsonElement error)
    {
        var code = error.TryGetProperty("codigo", out var value) ? value.GetString() : null;
        var message = error.TryGetProperty("mensaje", out var text) ? text.GetString() : null;
        return new RamSqliteException(code switch
        {
            "capacidad" => RamSqliteErrorCode.Capacity,
            "base_no_existe" => RamSqliteErrorCode.DatabaseNotFound,
            "sql" => RamSqliteErrorCode.Sql,
            "parametros" => RamSqliteErrorCode.Parameters,
            "transaccion" => RamSqliteErrorCode.Transaction,
            "persistencia" => RamSqliteErrorCode.Persistence,
            _ => RamSqliteErrorCode.Unknown,
        }, message ?? "El servidor rechazó la solicitud.");
    }
}
