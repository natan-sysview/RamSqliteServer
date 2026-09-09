# Administración de bases en RAM Especificación

## Propósito

Definir el ciclo de vida de múltiples bases SQLite nombradas, efímeras y compartibles dentro de un servidor local.

## Requisitos

### Requisito: Identidad y creación de bases

El sistema MUST crear una base vacía asociada a un nombre no vacío y único en el servidor. El sistema MUST rechazar nombres duplicados y MUST NOT reemplazar una base existente.

#### Escenario: Crear una base nombrada

- GIVEN un servidor en ejecución sin una base llamada `ventas`
- WHEN un cliente local solicita crear `ventas`
- THEN el servidor confirma que `ventas` está disponible para conexión

#### Escenario: Rechazar un nombre duplicado

- GIVEN una base llamada `ventas` ya abierta
- WHEN un cliente solicita crear otra base llamada `ventas`
- THEN el servidor devuelve un error de nombre ya existente
- AND la base original permanece disponible

### Requisito: Carga y descubrimiento de bases

El sistema MUST permitir crear una base nombrada a partir de un archivo SQLite válido. El sistema MUST listar las bases abiertas con su nombre y estado. El sistema MUST rechazar una carga cuyo archivo no sea una base SQLite válida sin crear una base parcial.

#### Escenario: Cargar una base existente

- GIVEN un archivo SQLite válido y una base con nombre libre
- WHEN un cliente solicita cargarlo con ese nombre
- THEN el servidor registra la base y confirma que puede conectarse

#### Escenario: Rechazar un archivo inválido

- GIVEN una ruta a un archivo que no es una base SQLite válida
- WHEN un cliente solicita cargarlo como una base nombrada
- THEN el servidor devuelve un error de carga
- AND no aparece una base con ese nombre en el listado

### Requisito: Conexión y cierre explícitos

El sistema MUST permitir que clientes locales se conecten a una base abierta por nombre. El sistema MUST cerrar una base solo mediante una solicitud explícita y MUST impedir nuevas conexiones después de confirmarlo. El sistema MUST NOT cerrar otra base como consecuencia.

#### Escenario: Compartir una base entre procesos

- GIVEN una base `ventas` abierta
- WHEN dos clientes locales independientes se conectan a `ventas`
- THEN ambos reciben acceso a la misma base nombrada

#### Escenario: Cerrar una base sin afectar a las demás

- GIVEN las bases `ventas` e `inventario` abiertas
- WHEN un cliente solicita cerrar `ventas`
- THEN `ventas` deja de aceptar conexiones nuevas
- AND `inventario` continúa disponible

