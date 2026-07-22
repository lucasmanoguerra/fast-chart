# API Reference — fc-animation

## Tipos principales

### `Easing`
Funciones de easing para curvas de animación. Variantes: `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, `Spring { stiffness, damping }`.

### `AnimatedValue`
Valor animado que interpola entre dos `f64` a lo largo del tiempo. Constructor: `new(from, to, duration_ms, easing)`. Métodos: `update(dt_ms)`, `current()`, `state()`, `is_complete()`, `complete()`, `retarget(new_target)`.

### `AnimationState`
Snapshot del estado de una animación: `elapsed_ms`, `duration_ms`, `complete`. Método: `progress() -> f64`.

### `AnimationTrack`
Track de animación con valores `from`, `to`, `duration_ms` y `easing`.

### `AnimationEngine`
Motor que gestiona múltiples animaciones nombradas. Métodos: `new()`, `animate(name, animation)`, `remove(name)`, `value(name)`, `is_complete(name)`, `update(dt_ms)`, `gc()`, `active_count()`, `has_active()`.

## Funciones

### `apply_easing(t, easing) -> f64`
Aplica una función de easing a un progreso `[0.0, 1.0]`. Retorna el valor suavizado en el mismo rango (spring puede excederlo).
