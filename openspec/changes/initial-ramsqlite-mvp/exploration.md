## Exploración: MVP inicial de RamSQLite

### Estado actual

Este es un repositorio de planificación recién inicializado: no contiene código de aplicación, tiempo de ejecución, protocolo, pruebas ni una pila de implementación seleccionada. RamSQLite Server es solo un producto previsto: un coordinador hospedado en Rust para una base SQLite compartida y efímera que puedan usar varios procesos cliente, siendo C# el primer consumidor esperado.

SQLite ya admite el uso seguro desde varios hilos en su modo serializado predeterminado; esto no es motivo para crear un servidor. Una base `:memory:` común es distinta por conexión, mientras que una URI de memoria compartida solo se comparte entre conexiones del mismo proceso. La documentación de SQLite confirma, por lo tanto, la brecha entre procesos, pero también advierte que el modo de caché compartida está desaconsejado y mantiene la restricción de un solo escritor. [Bases en memoria de SQLite](https://www.sqlite.org/inmemorydb.html) · [Concurrencia de SQLite](https://www.sqlite.org/threadsafe.html) · [Caché compartida de SQLite](https://www.sqlite.org/sharedcache.html)

### Áreas afectadas

- `openspec/config.yaml` — establece que aún no se ha seleccionado una pila, arquitectura ni comando de pruebas; la propuesta debe preservar ese límite.
- `openspec/changes/initial-ramsqlite-mvp/exploration.md` — registra la exploración inicial del producto.
- Futuras áreas del servidor Rust y cliente C# — todavía no se han creado; su protocolo y API no deben asumirse en esta fase.

### Enfoques

1. **Coordinador local efímero con benchmarks primero** — Definir un MVP que ejecute un proceso servidor local, aloje una base de datos volátil con nombre explícito y atienda solicitudes SQL parametrizadas de varios procesos cliente locales mediante un contrato deliberadamente pequeño.
   - Ventajas: valida el valor real del ciclo de vida y la concurrencia entre procesos; mantiene verificable la hipótesis; evita afirmar que la RAM por sí sola acelera cualquier SQL.
   - Desventajas: necesita un protocolo, ciclo de vida del proceso, modelo de errores y arnés de benchmarks antes de resultar útil; añade latencia de IPC.
   - Esfuerzo: medio.

2. **Servidor de base de datos compatible con protocolos** — Exponer SQLite desde el inicio a través de un protocolo de red ampliamente admitido, como el protocolo wire de PostgreSQL o HTTP.
   - Ventajas: menos trabajo de integración específico por cliente; herramientas y controladores conocidos.
   - Desventajas: superficie de compatibilidad y seguridad mucho mayor; se solapa con ofertas existentes como `sqld` de libSQL, que expone acceso compatible con SQLite de forma remota. [README del servidor libSQL](https://github.com/tursodatabase/libsql/blob/main/libsql-server/README.md)
   - Esfuerzo: alto.

3. **Usar un servidor existente en vez de construir RamSQLite** — Evaluar libSQL/sqld o rqlite para la necesidad de despliegue.
   - Ventajas: proporciona interfaces de servidor y operación probadas; rqlite añade replicación tolerante a fallos.
   - Desventajas: no se dirige al problema acotado de coordinación local, desechable y en RAM; introduce complejidad de persistencia y distribución que el MVP propuesto no necesita. [Proyecto libSQL](https://github.com/tursodatabase/libsql) · [Proyecto rqlite](https://github.com/rqlite/rqlite)
   - Esfuerzo: evaluación baja / sin producto nuevo.

### Recomendación

Avanzar a una propuesta para un **MVP de validación, solo local y con benchmarks primero**, no para un servidor de base de datos de propósito general. Su afirmación de producto debería ser: “un proceso posee una base SQLite efímera con nombre y la expone de forma segura a varios procesos locales”. No debe prometer aceleraciones universales de SQL: SQLite ya es rápido y el servidor añade sobrecarga de IPC. La propuesta debe exigir evidencia de que este modelo resuelve un caso C# multiproceso mejor que un archivo SQLite con WAL o un servidor existente.

El primer MVP debe ser deliberadamente acotado: transporte solo por loopback, un propietario de la base, ciclo de vida explícito de crear/conectar/cerrar, ejecutar/consultar con parámetros más límites de transacción, comportamiento de un solo escritor documentado, apagado limpio que elimine todos los datos y un cliente de humo en C#. El acceso remoto, autenticación, replicación, durabilidad, transparencia para ORM, emulación de protocolos y agrupamiento de alta disponibilidad quedan fuera de alcance.

### Riesgos

- IPC y la serialización pueden hacer que el servidor sea más lento que SQLite directo dentro del proceso para muchas cargas; el rendimiento debe medirse contra `:memory:` directo y una base SQLite local con WAL.
- SQLite sigue permitiendo solo un escritor a la vez, así que el coordinador no puede prometer honestamente rendimiento de escritura en paralelo.
- El uso “transparente” desde C# requiere un proveedor/controlador personalizado o un protocolo compatible; ambos amplían materialmente el MVP.
- Una base en memoria se pierde cuando termina su servidor propietario, por lo que el ciclo de vida y el comportamiento ante fallos deben ser explícitos.
- Productos existentes pueden satisfacer requisitos de servidor más amplios, dejando la diferenciación ligada a la simplicidad solo local y a una experiencia C# de alta calidad.

### Lista para propuesta

Sí — proponer un MVP de validación acotado con criterios explícitos de rechazo: detener o pivotar si una carga C# multiproceso representativa no muestra una ventaja significativa de usabilidad o de rendimiento/latencia frente a un archivo SQLite con WAL, o si la integración cliente requerida no es lo bastante pequeña para una versión inicial.
