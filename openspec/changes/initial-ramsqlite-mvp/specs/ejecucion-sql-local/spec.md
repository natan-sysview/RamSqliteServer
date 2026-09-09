# Ejecución SQL local Especificación

## Propósito

Definir la ejecución SQL compartida para clientes locales, con parámetros, transacciones y errores observables.

## Requisitos

### Requisito: Ejecución SQL parametrizada

El sistema MUST permitir a un cliente conectado ejecutar sentencias SQL parametrizadas sobre la base solicitada. El sistema MUST devolver filas para consultas y un resultado de ejecución para sentencias que modifican datos. El sistema MUST NOT interpretar valores de parámetros como texto SQL.

#### Escenario: Consultar con parámetros

- GIVEN una conexión local a una base con datos
- WHEN el cliente envía una consulta SQL con valores parametrizados válidos
- THEN el servidor devuelve las filas que coinciden con los parámetros

#### Escenario: Rechazar parámetros incompatibles

- GIVEN una sentencia válida cuyo parámetro requerido no tiene un valor compatible
- WHEN el cliente solicita ejecutarla
- THEN el servidor devuelve un error de parámetros
- AND no modifica la base

### Requisito: Aislamiento por base y conexión

El sistema MUST ejecutar cada solicitud sobre la base nombrada a la que se conectó el cliente. El sistema MUST NOT permitir que una solicitud dirigida a una base lea o modifique otra base sin una conexión explícita a ella.

#### Escenario: Operar bases distintas en paralelo

- GIVEN clientes conectados a `ventas` e `inventario`
- WHEN cada cliente ejecuta una modificación válida en su propia base
- THEN cada modificación solo es visible en su base correspondiente

### Requisito: Transacciones y concurrencia de escritura

El sistema MUST permitir iniciar, confirmar y revertir transacciones para una conexión. El sistema MUST documentar y comunicar que una base admite como máximo un escritor activo a la vez; una solicitud de escritura que no pueda progresar MUST recibir un resultado definido y MUST NOT corromper datos.

#### Escenario: Confirmar una transacción

- GIVEN una transacción activa que insertó datos válidos
- WHEN el cliente solicita confirmarla
- THEN los datos confirmados son visibles para una consulta posterior permitida

#### Escenario: Conflicto de escritores

- GIVEN una base con una transacción de escritura activa
- WHEN otro cliente intenta iniciar una escritura incompatible
- THEN el servidor devuelve un resultado definido de espera o conflicto
- AND la transacción activa conserva su integridad

### Requisito: Errores SQL definidos

El sistema MUST devolver errores distinguibles para base inexistente, conexión inválida, SQL inválido y conflicto de transacción. El sistema MUST NOT exponer el estado interno de otro cliente en esos errores.

#### Escenario: SQL inválido

- GIVEN una conexión válida
- WHEN el cliente envía una sentencia SQL inválida
- THEN recibe un error SQL sin cambio de datos

