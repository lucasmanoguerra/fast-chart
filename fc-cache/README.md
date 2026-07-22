# fc-cache

Cachés especializados para optimizar el rendimiento de renderizado de gráficos. Incluye un caché genérico con eviction FIFO y hit-rate tracking, y cachés de dominio específicos para geometría, texto, ejes, grilla e indicadores.

## Uso

```rust
use fc_cache::{Cache, GeometryCache, TextCache, AxisCache};

// Caché genérico
let mut cache: Cache<String, Vec<f32>> = Cache::new(1000);
cache.insert("geometry_1".into(), vec![1.0, 2.0, 3.0]);
let hit = cache.get(&"geometry_1".into()); // Some
let miss = cache.get(&"missing".into());   // None
let (hits, misses) = cache.hit_rate();

// Caché de geometría
let mut geo = GeometryCache::new(500);
geo.insert(series_id, vertices);
```

## Dependencias

- Sin dependencias externas — solo std

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `cache` | `Cache<K, V>` — caché genérico con FIFO eviction y hit-rate |
| `axis` | `AxisCache` — caché de labels y posiciones del eje |
| `geometry` | `GeometryCache` — caché de geometría de series |
| `grid` | `GridCache` — caché de líneas de grilla |
| `indicator` | `IndicatorCache` — caché de cálculos de indicadores |
| `text` | `TextCache` — caché de mediciones y renderizado de texto |
