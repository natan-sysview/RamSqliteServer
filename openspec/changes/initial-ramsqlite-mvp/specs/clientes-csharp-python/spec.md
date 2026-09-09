# Clientes C# y Python Especificación

## Propósito

Definir los clientes iniciales que permiten a procesos C# y Python usar el contrato local de RamSQLite.

## Requisitos

### Requisito: Disponibilidad de clientes iniciales

El sistema MUST proporcionar un cliente para C# y otro para Python que se conecten únicamente a un servidor local. Cada cliente MUST permitir seleccionar una base nombrada y solicitar las operaciones del MVP aplicables.

#### Escenario: Conexión desde C#

- GIVEN un servidor local y una base abierta
- WHEN un proceso C# usa el cliente con el nombre de la base
- THEN puede establecer una conexión y recibir el resultado de una operación válida

#### Escenario: Conexión desde Python

- GIVEN un servidor local y una base abierta
- WHEN un proceso Python usa el cliente con el nombre de la base
- THEN puede establecer una conexión y recibir el resultado de una operación válida

### Requisito: Interoperabilidad multiproceso

El sistema MUST demostrar que un proceso C# y un proceso Python pueden conectarse a la misma base nombrada y observar datos confirmados por el otro. El sistema MUST demostrar que pueden usar bases nombradas distintas sin mezclar datos.

#### Escenario: Compartir datos entre lenguajes

- GIVEN clientes C# y Python conectados a la misma base
- WHEN el cliente C# confirma una inserción válida
- THEN una consulta posterior del cliente Python observa el dato confirmado

#### Escenario: Aislar bases por nombre

- GIVEN clientes conectados a dos bases con nombres diferentes
- WHEN cada cliente escribe un valor distinto
- THEN ninguna consulta observa el valor de la otra base

### Requisito: Errores consumibles

Los clientes MUST exponer errores de conexión, capacidad, base inexistente, SQL y persistencia de forma distinguible para que las aplicaciones los puedan manejar. Los clientes SHOULD conservar el contexto de operación sin filtrar contenido de otras bases.

#### Escenario: Mostrar un rechazo de capacidad

- GIVEN que el servidor rechaza una carga por capacidad
- WHEN un cliente C# o Python recibe la respuesta
- THEN la aplicación puede identificar el error como capacidad insuficiente

