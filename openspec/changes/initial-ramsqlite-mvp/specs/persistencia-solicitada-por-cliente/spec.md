# Persistencia solicitada por cliente Especificación

## Propósito

Definir el guardado explícito de una base completa en RAM a un archivo SQLite local.

## Requisitos

### Requisito: Guardado bajo solicitud explícita

El sistema MUST guardar o sincronizar una base completa únicamente tras una solicitud explícita de un cliente conectado a esa base. El sistema MUST NOT ejecutar persistencia periódica, automática ni implícita al admitir una base.

#### Escenario: Guardar una base a SSD

- GIVEN una base abierta con cambios en RAM y una ruta local autorizada
- WHEN un cliente solicita sincronizar esa base en la ruta
- THEN el servidor confirma la finalización del guardado completo

#### Escenario: No guardar sin solicitud

- GIVEN una base abierta con cambios en RAM
- WHEN no existe una solicitud de guardado
- THEN el servidor no crea ni actualiza un archivo de respaldo por sí mismo

### Requisito: Integridad de guardado y errores

El sistema MUST informar éxito solo si el archivo de destino representa una copia SQLite utilizable de la base solicitada. Si el destino no puede escribirse o la operación falla, el sistema MUST devolver un error y MUST mantener la base en RAM disponible y sin alteraciones causadas por el fallo.

#### Escenario: Fallo al escribir el destino

- GIVEN una base abierta y una ruta de destino no escribible
- WHEN un cliente solicita sincronizarla
- THEN el servidor devuelve un error de persistencia
- AND la base continúa disponible en RAM

### Requisito: Alcance de persistencia del MVP

El sistema MUST persistir una base completa, no un subconjunto de tablas. El sistema MUST NOT mover automáticamente tablas entre RAM y SSD.

#### Escenario: Solicitud de persistencia parcial

- GIVEN una base abierta
- WHEN un cliente solicita persistir solo tablas seleccionadas
- THEN el servidor rechaza la solicitud como no compatible con el MVP

