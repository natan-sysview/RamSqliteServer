# Diseño: MVP inicial de RamSQLite

## Enfoque técnico

Un proceso Rust escuchará solo en TCP loopback y administrará bases SQLite nombradas en RAM. Cada base tendrá un trabajador propietario y una conexión SQLite `:memory:`; así, bases distintas avanzan en paralelo y el límite de un escritor por base es explícito. C# y Python usarán un contrato propio, no emulación de protocolos existentes.

## Decisiones de arquitectura

### Transporte y compatibilidad

**Elección:** TCP en `127.0.0.1`/`::1`, JSON con longitud prefijada, una respuesta por solicitud y BLOB en Base64; se soportarán macOS, Linux y Windows.

**Alternativas:** sockets Unix, pipes con nombre, HTTP/PostgreSQL y conexión SQLite directa.

**Justificación:** loopback conserva el alcance local y es portable en los tres sistemas. El servidor rechazará destinos no loopback. Las pruebas y empaquetado se ejecutarán para las tres plataformas.

### Propiedad y concurrencia por base

**Elección:** registro `nombre -> Base`; cada `Base` posee una cola FIFO acotada y un trabajador bloqueante con su conexión SQLite privada.

**Alternativas:** mutex sobre una conexión compartida, caché SQLite compartida o proceso por base.

**Justificación:** el acceso queda confinado a un dueño. Una transacción se asocia a `id_conexion`; escrituras incompatibles esperan o devuelven conflicto definido. Las bases distintas progresan en paralelo.

### Admisión conservadora

**Elección:** antes de publicar una base, reservar una estimación de su tamaño más sobrecarga; para una nueva, reservar un mínimo configurable. Validar máximo de bases, cuota estimada por base y total; limitar también las colas.

**Alternativas:** límite físico perfecto por base, expulsión LRU o admitir sin control.

**Justificación:** SQLite puede crecer después de una consulta y no ofrece un techo físico fiable por conexión. La estimación segura es la política explícita del MVP: una admisión que exceda la cuota se rechaza y nunca desaloja bases existentes.

### Rutas y copia completa

**Elección:** el servidor configura y posee un único directorio raíz para todas las cargas y sincronizaciones. Las rutas del protocolo son relativas a esa raíz; al resolverlas, el servidor rechaza rutas absolutas, `..` o enlaces que escapen de ella. Cargar valida y copia el archivo a RAM; sincronizar copia RAM a un destino autorizado mediante Backup API.

**Alternativas:** rutas arbitrarias del cliente, una raíz por base, guardado periódico o persistir tablas individuales.

**Justificación:** los clientes locales no obtienen acceso general al sistema de archivos. Una carga fallida no publica una base y un guardado fallido no altera RAM.

## Flujo de datos

```text
C# / Python --TCP loopback--> receptor Rust --registro--> cola de Base
                                                     \-> trabajador SQLite :memory:
cliente <--respuesta JSON------- receptor <---------- resultado/error

cargar: raíz del servidor/archivo -> copia SQLite -> Base en RAM
sincronizar: Base en RAM -> copia SQLite -> raíz del servidor/destino
```

Solicitud: `id_solicitud`, operación, `base`, `id_conexion` opcional, SQL, parámetros tipados y ruta relativa cuando aplique. Respuesta: mismo id, filas, filas afectadas o error (`capacidad`, `ruta_no_autorizada`, `base_no_existe`, `conexion_invalida`, `sql`, `parametros`, `transaccion`, `persistencia`).

## Cambios de archivos

| Archivo | Acción | Descripción |
|---|---|---|
| `server/` | Crear | Crate Rust: receptor, registro, trabajadores, límites y validación de rutas. |
| `clients/csharp/` | Crear | Cliente C# y prueba de humo. |
| `clients/python/` | Crear | Cliente Python y prueba de humo. |
| `benchmarks/` | Crear | Cargas reproducibles y comparadores `:memory:`/WAL. |
| `docs/protocolo-local.md` | Crear | Mensajes, tipos, códigos de error y raíz de archivos. |

## Estrategia de pruebas

| Capa | Qué probar | Enfoque |
|---|---|---|
| Unidad | registro, estimaciones, límites, nombres y resolución de rutas | Rust determinista; incluir `..`, absoluta y enlace fuera de raíz |
| Integración | SQL, transacciones, carga/sincronización, límites y rechazo de rutas | servidor loopback y archivos temporales bajo raíz |
| E2E | C#/Python, misma/distinta base, rechazos y persistencia | procesos independientes en macOS, Linux y Windows |
| Benchmark | latencia, rendimiento, errores y metadatos | cargas versionadas frente a RAM directa y WAL |

## Matriz de amenazas

| Límite | Aplicabilidad | Respuesta |
|---|---|---|
| Rutas tipo documentación | N/A | El MVP no clasifica ni ejecuta archivos. |
| Selección Git | N/A | No usa Git. |
| Estado de commit | N/A | No crea commits. |
| Estado de push | N/A | No hace push. |
| Comandos PR | N/A | No crea ni ejecuta PRs. |

## Migración / lanzamiento

No requiere migración. Se publicarán artefactos y pruebas para macOS, Linux y Windows. Si los benchmarks no muestran valor frente a WAL, se detiene o pivota antes de estabilizar el contrato.

## Preguntas abiertas

Ninguna.
