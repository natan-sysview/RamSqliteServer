using System.Diagnostics;
using System.Net;
using System.Net.Sockets;
using RamSqlite.Client;
using Xunit;

namespace RamSqlite.Client.Tests;

public sealed class SmokeTests : IAsyncLifetime
{
    private readonly string dataRoot = Path.Combine(Path.GetTempPath(), $"ramsqlite-csharp-{Guid.NewGuid():N}");
    private Process? server;
    private int port;

    public async Task InitializeAsync()
    {
        var repositoryRoot = FindRepositoryRoot();
        await RunAsync("cargo", "build --package ramsqlite-server", repositoryRoot);
        port = ReserveLoopbackPort();
        Directory.CreateDirectory(dataRoot);
        var executable = OperatingSystem.IsWindows() ? "ramsqlite-server.exe" : "ramsqlite-server";
        server = Process.Start(new ProcessStartInfo(Path.Combine(repositoryRoot, "target", "debug", executable))
        {
            UseShellExecute = false,
            Environment =
            {
                ["RAMSQLITE_LISTEN"] = $"127.0.0.1:{port}",
                ["RAMSQLITE_DATA_ROOT"] = dataRoot,
            },
        }) ?? throw new InvalidOperationException("No se pudo iniciar el servidor real.");

        var deadline = DateTime.UtcNow.AddSeconds(5);
        while (DateTime.UtcNow < deadline)
        {
            try
            {
                await new RamSqliteClient(IPAddress.Loopback, port, "humo").CreateAsync();
                return;
            }
            catch (RamSqliteException exception) when (exception.Code == RamSqliteErrorCode.Connection)
            {
                await Task.Delay(50);
            }
        }
        throw new TimeoutException("El servidor real no abrió el puerto loopback.");
    }

    public async Task DisposeAsync()
    {
        if (server is { HasExited: false })
        {
            server.Kill(entireProcessTree: true);
            await server.WaitForExitAsync();
        }
        Directory.Delete(dataRoot, recursive: true);
    }

    [Fact]
    public async Task Cliente_tcp_crea_ejecuta_sql_y_expone_error_sql()
    {
        var client = new RamSqliteClient(IPAddress.Loopback, port, "humo");
        await client.ExecuteAsync("CREATE TABLE productos (nombre TEXT NOT NULL)");
        await client.ExecuteAsync("INSERT INTO productos(nombre) VALUES (?)", ["teclado"]);

        var rows = await client.ExecuteAsync("SELECT nombre FROM productos WHERE nombre = ?", ["teclado"]);
        Assert.Equal("teclado", rows.GetProperty("filas")[0].GetProperty("nombre").GetString());

        var error = await Assert.ThrowsAsync<RamSqliteException>(
            () => client.ExecuteAsync("ESTO NO ES SQL"));
        Assert.Equal(RamSqliteErrorCode.Sql, error.Code);
    }

    private static async Task RunAsync(string fileName, string arguments, string workingDirectory)
    {
        using var process = Process.Start(new ProcessStartInfo(fileName, arguments)
        {
            WorkingDirectory = workingDirectory,
            UseShellExecute = false,
        }) ?? throw new InvalidOperationException($"No se pudo iniciar {fileName}.");
        await process.WaitForExitAsync();
        Assert.True(process.ExitCode == 0, $"{fileName} terminó con {process.ExitCode}.");
    }

    private static int ReserveLoopbackPort()
    {
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        return ((IPEndPoint)listener.LocalEndpoint).Port;
    }

    private static string FindRepositoryRoot()
    {
        for (var directory = new DirectoryInfo(AppContext.BaseDirectory); directory is not null; directory = directory.Parent)
            if (File.Exists(Path.Combine(directory.FullName, "Cargo.toml")))
                return directory.FullName;
        throw new DirectoryNotFoundException("No se encontró la raíz del repositorio.");
    }
}
