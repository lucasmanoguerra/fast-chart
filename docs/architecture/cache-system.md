# Cache System

## Overview

5 specialized sub-caches minimize redundant computation and GPU uploads.
Each cache tracks invalidation independently.

---

## Sub-Caches

### GeometryCache

Caches computed vertex geometry for series. Avoids recomputing candle bodies,
line segments, and area fills when only the viewport changes slightly.

### TextCache

Caches rendered text glyphs and layouts. Price labels, time axis labels, and
indicator values are cached to avoid re-rasterization.

### AxisCache

Caches price scale and time scale tick positions and labels. Recomputes only
when the visible range changes significantly.

### GridCache

Caches grid line positions. Horizontal (price) and vertical (time) grid lines
are pre-computed and reused across frames.

### IndicatorCache

Caches indicator calculation results. SMA, RSI, MACD values are cached per
input series and only recomputed when new data arrives.

---

## Invalidation

Each cache invalidates independently based on what changed:

| Change | Invalidates |
|--------|-------------|
| New data bar | GeometryCache, IndicatorCache |
| Zoom/scroll | GeometryCache, AxisCache, GridCache |
| Resize | All caches |
| Theme change | TextCache, GridCache |
| Pane height change | GeometryCache, AxisCache |

```rust
pub mod cache {
    pub use axis::AxisCache;
    pub use geometry::GeometryCache;
    pub use grid::GridCache;
    pub use indicator::IndicatorCache;
    pub use text::TextCache;
}
```

---

## Usage

Caches are managed internally by the render pipeline. The pipeline checks
cache validity before each pass and only recomputes when necessary.

For custom renderers, caches can be used directly:

```rust
let mut geo_cache = GeometryCache::new();
if geo_cache.is_valid(series_version) {
    // Use cached geometry
} else {
    // Recompute and update cache
    geo_cache.update(series_version, computed_vertices);
}
```
