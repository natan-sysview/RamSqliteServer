# Propuesta: MVP inicial de RamSQLite

## Intención

Validar un servidor Rust local que coordine bases SQLite nombradas entre procesos C# y Python. Su valor es compartir datos y recursos entre procesos, no acelerar cualquier SQL.

## Alcance

### Incluido
- Administrar N bases nombradas.
- Crear una base o cargar un archivo SQLite existente en RAM.
- SQL parametrizado y transacciones para procesos locales en una o distintas bases.
- Límites configurables de memoria total, por base y número de bases; rechazar claramente toda creación o carga que los exceda.
- Guardar o sincronizar una base completa a SSD únicamente cuando el cliente lo solicite.
- Clientes C# y Python con prueba multiproceso.
- Benchmarks contra SQLite directo en `:memory:` y SQLite local con WAL.

### Fuera de alcance
- Acceso remoto, autenticación, replicación, clústeres, HTTP/PostgreSQL y transparencia para ORMs.
- Escrituras paralelas ilimitadas o aceleración SQL universal.
- Persistencia, expulsión o eliminación automáticas.
- Almacenamiento escalonado RAM/SSD por tabla; será trabajo futuro configurado por el cliente.

## Capacidades

### Nuevas capacidades
- `administracion-de-bases-en-ram`: crear, cargar, conectar, listar y cerrar múltiples bases nombradas.
- `control-de-capacidad`: aplicar límites configurables y rechazar admisiones sin desalojar bases existentes.
- `ejecucion-sql-local`: SQL parametrizado, consultas, transacciones y errores definidos.
- `persistencia-solicitada-por-cliente`: guardar o sincronizar una base completa por solicitud explícita.
- `clientes-csharp-python`: clientes y escenarios multiproceso.
- `evaluacion-comparativa`: métricas reproducibles frente a SQLite base.

### Capacidades modificadas

Ninguna; aún no existen especificaciones principales.

## Enfoque

Contrato local mínimo; el servidor posee cada base y documenta el límite de un escritor. Los benchmarks decidirán continuar o pivotar.

## Áreas afectadas

| Área | Impacto | Descripción |
|---|---|---|
| Servidor Rust | Nuevo | Registro, límites y persistencia. |
| Clientes C# y Python | Nuevo | API local y guardado. |

## Riesgos

| Riesgo | Probabilidad | Mitigación |
|---|---|---|
| IPC empeora la latencia | Alta | Medir primero. |
| Un escritor limita cargas | Alta | Documentar y medir. |
| Agotamiento de RAM | Media | Límites y rechazo seguro. |

## Plan de reversión

Si los benchmarks o la integración no muestran valor, detener o pivotar; no habrá migraciones ni datos de producción que revertir.

## Dependencias

- Elegir transporte, concurrencia y pruebas.
- Definir cargas C# y Python.

## Criterios de éxito

- [ ] C# y Python comparten una base nombrada y usan bases distintas en paralelo.
- [ ] El servidor rechaza admisiones que exceden capacidad sin desalojar datos.
- [ ] Un cliente carga o crea una base y solicita sincronizarla a disco.
- [ ] Resultados reproducibles contra `:memory:` y WAL justifican continuar.
