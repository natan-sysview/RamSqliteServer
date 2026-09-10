using System.Net;
using RamSqlite.Client;

if (args.Length != 4 || !IPAddress.TryParse(args[0], out var address) || !int.TryParse(args[1], out var port))
{
    Console.Error.WriteLine("Uso: RamSqlite.E2E.Writer <host-loopback> <puerto> <base> <valor>");
    return 2;
}

var client = new RamSqliteClient(address, port, args[2]);
await client.CreateAsync();
await client.ExecuteAsync("CREATE TABLE IF NOT EXISTS registros (valor TEXT NOT NULL)");
await client.ExecuteAsync("INSERT INTO registros(valor) VALUES (?)", [args[3]]);

var result = await client.ExecuteAsync("SELECT valor FROM registros WHERE valor = ?", [args[3]]);
var rows = result.GetProperty("filas");
if (rows.GetArrayLength() != 1 || rows[0].GetProperty("valor").GetString() != args[3])
{
    Console.Error.WriteLine("El cliente C# no pudo confirmar su inserción.");
    return 1;
}

return 0;
