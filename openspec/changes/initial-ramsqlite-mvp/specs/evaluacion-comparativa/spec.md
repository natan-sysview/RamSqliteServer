# Evaluación comparativa Especificación

## Propósito

Definir evidencia reproducible para decidir si RamSQLite aporta valor frente a alternativas SQLite directas.

## Requisitos

### Requisito: Comparadores y cargas representativas

El sistema MUST medir RamSQLite contra SQLite directo en `:memory:` y una base SQLite local con WAL. Cada comparación MUST usar una carga documentada de varios procesos, incluyendo consultas y al menos una carga con escrituras.

#### Escenario: Ejecutar los tres comparadores

- GIVEN una carga de referencia documentada y datos equivalentes
- WHEN se ejecuta la evaluación comparativa
- THEN se producen resultados para RamSQLite, `:memory:` y SQLite con WAL

#### Escenario: Carga con múltiples bases

- GIVEN una carga de referencia que usa más de una base nombrada
- WHEN se mide RamSQLite
- THEN el resultado identifica la carga por base y el comportamiento agregado

### Requisito: Reproducibilidad y métricas

El sistema MUST documentar la versión de la carga, tamaño de datos, configuración de límites y entorno de ejecución de cada resultado. El sistema MUST registrar latencia, rendimiento y errores por escenario. El sistema SHOULD incluir consumo de memoria si puede medirse de forma consistente.

#### Escenario: Repetir una evaluación

- GIVEN la misma carga, datos, configuración y entorno documentados
- WHEN se repite una evaluación
- THEN cada resultado conserva los metadatos necesarios para comparar sus métricas

### Requisito: Decisión explícita de continuación

El sistema MUST publicar una conclusión que indique continuar, pivotar o detener el MVP a partir de los resultados. La conclusión MUST reconocer que `:memory:` directo no necesita IPC y que RamSQLite no promete aceleración SQL universal.

#### Escenario: Resultados sin valor suficiente

- GIVEN resultados donde RamSQLite no ofrece una ventaja de usabilidad o rendimiento/latencia relevante frente a WAL
- WHEN se revisa la evaluación
- THEN la conclusión recomienda pivotar o detener en vez de afirmar éxito

