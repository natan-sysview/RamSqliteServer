# Control de capacidad Especificación

## Propósito

Definir la admisión segura de bases en RAM sin desalojar ni alterar bases existentes.

## Requisitos

### Requisito: Límites configurables de admisión

El sistema MUST permitir configurar un límite de memoria total, un límite de memoria por base y un máximo de bases abiertas. El sistema MUST aplicar los límites al crear o cargar una base antes de confirmarla.

#### Escenario: Admitir una base dentro de los límites

- GIVEN límites configurados con capacidad disponible suficiente
- WHEN un cliente crea o carga una base que no excede ningún límite
- THEN el servidor admite la base y actualiza su uso de capacidad

#### Escenario: Rechazar por máximo de bases

- GIVEN que el número de bases abiertas alcanzó el máximo configurado
- WHEN un cliente intenta crear o cargar otra base
- THEN el servidor devuelve un error de capacidad con el límite afectado

### Requisito: Rechazo seguro por capacidad insuficiente

El sistema MUST rechazar una admisión que exceda cualquier límite. El error MUST identificar que la causa es capacidad insuficiente. El sistema MUST NOT desalojar, cerrar, borrar ni modificar una base ya admitida para satisfacer la solicitud.

#### Escenario: Rechazar una carga que supera la memoria por base

- GIVEN una carga cuyo tamaño requerido excede el límite por base
- WHEN el cliente solicita abrirla en RAM
- THEN el servidor rechaza la solicitud antes de hacerla disponible
- AND las bases existentes conservan su estado

#### Escenario: Rechazar cuando no queda memoria total

- GIVEN bases abiertas que consumen la capacidad total admitida
- WHEN un cliente solicita una nueva base que requiere más memoria
- THEN el servidor devuelve un error de capacidad total
- AND no expulsa ninguna base existente

### Requisito: Estado de capacidad observable

El sistema MUST exponer la configuración de límites y el estado de admisión de cada base listada. El sistema SHOULD incluir el motivo de rechazo sin exponer datos SQL ni contenido de la base.

#### Escenario: Consultar capacidad tras un rechazo

- GIVEN que una solicitud fue rechazada por capacidad
- WHEN un cliente consulta el listado de bases y límites
- THEN recibe las bases actualmente admitidas y los límites configurados

