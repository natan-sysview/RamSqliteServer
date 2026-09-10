# Protocolo local de RamSQLite (MVP)

RamSQLite expone SQLite en RAM a procesos locales mediante TCP loopback. Este
contrato es para los clientes C# y Python iniciales; no es HTTP, PostgreSQL ni
un protocolo compatible con ORMs. La versión MVP admite una solicitud y una
respuesta por conexión.

## Inicio rápido

1. Inicie el servidor con `cargo run -p ramsqlite-server`.
2. Conéctese únicamente a `127.0.0.1:7432` (o al valor local de
   `RAMSQLITE_LISTEN`).
3. Envíe un frame JSON `crear` y después frames `ejecutar` para la misma
   `base` y un `id_conexion` estable.

```json
{"operacion":"crear","id_solicitud":"req-1","base":"ventas"}
```

```json
{"operacion":"ejecutar","id_solicitud":"req-2","base":"ventas","id_conexion":"cliente-a","sql":"SELECT ? AS saludo","parametros":["hola"]}
```

## Frame y respuesta

Cada frame contiene un prefijo de **4 bytes sin signo en big-endian** que
indica el número de bytes UTF-8 del JSON que le sigue. El límite predeterminado
por frame es 8 MiB. El cliente debe escribir el prefijo y el JSON completos, y
leer exactamente el prefijo y el cuerpo de la respuesta.

Toda respuesta tiene esta forma; conserva el `id_solicitud` de la petición:

```json
{
  "id_solicitud":"req-2",
  "resultado":{"filas":[{"saludo":"hola"}],"filas_afectadas":0},
  "error":null
}
```

Si hay error, `resultado` es `null` y `error` contiene `codigo` y `mensaje`.
No se deben interpretar los mensajes para controlar la aplicación: use el
código estable.

```json
{
  "id_solicitud":"req-2",
  "resultado":null,
  "error":{"codigo":"sql","mensaje":"la sentencia SQL no es válida"}
}
```

## Operaciones

Todos los mensajes usan `operacion` en `snake_case` e `id_solicitud` no vacío.

| Operación | Campos obligatorios | Campos opcionales | Resultado correcto |
|---|---|---|---|
| `crear` | `base` | `bytes_estimados` | `{ "base": "…" }` |
| `listar` | — | — | límites, uso y bases admitidas |
| `cerrar` | `base` | — | `{ "base": "…" }` |
| `cargar` | `base`, `ruta` | `bytes_estimados` | `{ "base": "…" }` |
| `sincronizar` | `base`, `ruta` | — | `{ "base": "…" }` |
| `sincronizar_parcial` | `base`, `ruta`, `tablas` | — | siempre se rechaza en el MVP |
| `ejecutar` | `base`, `id_conexion`, `sql`, `parametros` | — | `filas` y `filas_afectadas` |
| `iniciar_transaccion` | `base`, `id_conexion` | — | `{ "base": "…" }` |
| `confirmar_transaccion` | `base`, `id_conexion` | — | `{ "base": "…" }` |
| `revertir_transaccion` | `base`, `id_conexion` | — | `{ "base": "…" }` |

`listar` devuelve un objeto con `databases`, `used_bytes`, `max_databases`,
`max_total_bytes` y `max_database_bytes`. Cada elemento de `databases` incluye
`name` y `reserved_bytes`.

## SQL y parámetros

`sql` se prepara en SQLite; los valores se envían exclusivamente en el arreglo
`parametros`, nunca concatenados al texto SQL. Se aceptan `null`, booleanos,
números JSON y textos. Los booleanos se almacenan como enteros 0/1. Arreglos y
objetos JSON se rechazan con `parametros`.

Una consulta devuelve `filas` como objetos JSON por nombre de columna y
`filas_afectadas: 0`. Una sentencia de modificación devuelve `filas: []` y el
número de filas afectadas. Un BLOB de SQLite se representa como
`{ "blob_bytes": <tamaño> }`; el MVP no transporta el contenido del BLOB.

## Transacciones y concurrencia

Una base nombrada tiene un trabajador FIFO y una conexión SQLite `:memory:`
propietaria. Bases distintas pueden avanzar en paralelo. En una misma base hay
como máximo **un escritor activo**:

1. Use el mismo `id_conexion` no vacío para iniciar, ejecutar y terminar una
   transacción.
2. `iniciar_transaccion` abre una transacción de escritura inmediata.
3. Solo su dueño puede confirmar o revertirla.
4. Mientras exista, una escritura incompatible de otra conexión recibe
   `transaccion`; no se corrompen datos.

Cierre explícitamente una base con `cerrar` cuando ya no se use. No hay cierre,
expulsión ni persistencia automáticos.

## Archivos y persistencia

El servidor posee una única raíz de datos. Por defecto es `./data`; puede
cambiarse al iniciar mediante `RAMSQLITE_DATA_ROOT`. `ruta` siempre es relativa
a esa raíz: no se permiten rutas absolutas, `..` ni enlaces simbólicos que
salgan de ella.

- `cargar` abre una SQLite existente y válida bajo la raíz, y la copia a RAM.
- `sincronizar` copia la base completa de RAM a un destino autorizado mediante
  SQLite Backup API.
- El guardado ocurre solo tras `sincronizar`; un fallo devuelve `persistencia`
  y mantiene la base de RAM disponible.
- `sincronizar_parcial` se rechaza: mover tablas entre RAM y SSD no forma parte
  del MVP.

## Errores estables

| Código | Significado y acción del cliente |
|---|---|
| `capacidad` | Se excedió máximo de bases, cuota por base o total; no se desalojó otra base. |
| `base_duplicada` / `nombre_invalido` | Corrija el nombre antes de crear. |
| `base_no_existe` / `base_no_disponible` | Cree o cargue la base, o revise que siga abierta. |
| `conexion_invalida` | Use un `id_conexion` no vacío. |
| `sql` | Corrija la sentencia; no se aplicó como éxito. |
| `parametros` | Use únicamente tipos de parámetro admitidos y cantidad compatible. |
| `transaccion` | Respete el dueño de la transacción o termine/reintente la operación. |
| `carga` | El archivo no es una SQLite válida; la base no se publicó. |
| `persistencia` | Falló la copia completa; la base de RAM sigue disponible. |
| `ruta_no_autorizada` / `ruta_invalida` | Use una ruta relativa y confinada a la raíz. |
| `destino_no_local` | El servidor y clientes solo aceptan loopback. |
| `protocolo` | El cliente detectó JSON, frame o tamaño inválido; una solicitud malformada puede cerrar la conexión sin respuesta JSON. |

Otros códigos no especificados deben tratarse como errores no recuperables del
protocolo para esa solicitud y registrarse sin incluir SQL ni datos ajenos.

## Límites del MVP

| Límite predeterminado | Valor |
|---|---:|
| Escucha | `127.0.0.1:7432` |
| Bases abiertas | 32 |
| Reserva total admitida | 1 GiB |
| Reserva por base | 256 MiB |
| Reserva de una nueva base | 1 MiB |
| Cola por base | 32 solicitudes pendientes |
| Frame | 8 MiB |

Las reservas son estimaciones de admisión, no un límite físico perfecto de la
memoria de SQLite después de ejecutar consultas. Cuando no hay capacidad, se
rechaza la nueva creación o carga sin modificar las bases ya admitidas.

## Fuera de alcance

El MVP no ofrece acceso remoto, autenticación, replicación, persistencia
periódica, expulsión LRU, tablas escalonadas RAM/SSD, BLOB binario completo ni
escrituras paralelas ilimitadas. La compatibilidad futura debe añadirse con una
nueva versión documentada del contrato.
