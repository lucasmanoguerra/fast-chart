# fc-drawing

Trait unificado de dibujo y tipado para la librería de gráficos de trading fast-chart. Define el trait `Drawing` que todos los objetos de dibujo implementan, junto con hit-testing AABB y cálculo de bounds.

## Uso

```rust
use fc_drawing::{Drawing, DrawingBounds, HitResult, default_aabb_hit_test};

// Todos los tipos de dibujo implementan Drawing
let trend = TrendLine::new(100, 105.0, 200, 110.0);
assert_eq!(trend.type_name(), "TrendLine");

// Bounds
let bounds = trend.bounds();
assert!(bounds.time_width() > 0);

// Hit test
let hit = default_aabb_hit_test(&trend, 150, 107.5, 2.0, 2.0);
assert!(matches!(hit, HitResult::Hit { .. }));
```

## Dependencias

- `fc-domain` — tipos de dominio (DrawingSet, Crosshair, etc.)
- `serde` (opcional)

## Estructura

| Módulo | Descripción |
|--------|-------------|
| `trait_def` | Trait `Drawing` — interfaz común para todos los dibujos |
| `hit` | `HitResult`, `default_aabb_hit_test()` — hit testing AABB |
| `bounds` | `DrawingBounds` — cálculo de límites rectangulares |
| `impls` | Implementaciones de Drawing para los 15 tipos de dibujo |
