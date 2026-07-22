# API Reference — fc-cache

## Tipos principales

### `Cache<K, V>`
Caché genérico con bounded capacity y eviction FIFO. Métodos: `new(max_entries)`, `get(&key)`, `insert(key, value)`, `remove(&key)`, `contains(&key)`, `len()`, `is_empty()`, `clear()`, `hit_rate() -> (hits, misses)`.

### `AxisCache`
Caché para labels y posiciones de ejes. Almacena mediciones de texto y posiciones calculadas para evitar recálculo en cada frame.

### `GeometryCache`
Caché de geometría pre-calculada de series (vértices, índices). Key: ID de la serie. Evita regenerar vértices cuando los datos no cambian.

### `GridCache`
Caché de líneas de grilla horizontales y verticales. Almacena posiciones y estilos calculados.

### `IndicatorCache`
Caché de valores calculados de indicadores. Almacena series de valores intermedios para evitar recálculo completo.

### `TextCache`
Caché de mediciones de texto (width, height) y glyph layouts. Evita medir el mismo texto múltiples veces por frame.
