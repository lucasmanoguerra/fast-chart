# API Reference — fc-drawing

## Tipos principales

### `DrawingBounds`
Límites rectangulares de un dibujo. Campos: `time_start`, `time_end`, `value_min`, `value_max`. Métodos: `from_point()`, `from_points()`, `time_width()`, `price_height()`, `contains(time, value)`, `combine(other)`.

### `HitResult`
Resultado de hit-testing: `Hit { distance }` o `Miss`.

## Traits

### `Drawing`
Trait que todos los objetos de dibujo implementan. Métodos:
- `type_name() -> &str` — nombre del tipo
- `id() -> DrawingId` — ID único
- `bounds() -> DrawingBounds` — límites rectangulares
- `hit_test(time, value, tolerance_time, tolerance_value) -> HitResult`
- `move_by(dt, dv)` — trasladar el dibujo
- `is_selectable() -> bool`
- `as_any() -> &dyn Any` — downcast

## Funciones

### `default_aabb_hit_test(drawing, time, value, tol_time, tol_value) -> HitResult`
Hit test genérico AABB: verifica si el punto está dentro de los bounds con tolerancia.

## Tipos de dibujo (re-exports desde fc-domain)

`TrendLine`, `Arrow`, `Ray`, `Segment`, `TextDrawing`, `ImageDrawing`, `LabelDrawing`, `HorizontalLine`, `VerticalLine`, `Rectangle`, `FibonacciRetracement`, `FibonacciExtension`, `Pitchfork`, `Ellipse`, `Path`.

Todos implementan `Drawing` y están disponibles a través de `fc_domain::drawing`.
