# fc-animation

Motor de animación para transiciones suaves en gráficos financieros. Proporciona funciones de easing e interpolación de valores para transiciones animadas: ticks de precio, cambios de escala, zoom, scroll, opacidad, etc.

## Uso

```rust
use fc_animation::{AnimatedValue, AnimationEngine, Easing};

// Animación individual
let mut anim = AnimatedValue::new(0.0, 100.0, 200.0, Easing::EaseInOut);
anim.update(100.0); // mitad del tiempo
assert!((anim.current() - 50.0).abs() < 1e-10);

// Motor de animación con múltiples tracks
let mut engine = AnimationEngine::new();
engine.animate("scroll", AnimatedValue::new(0.0, 100.0, 500.0, Easing::EaseInOut));
engine.animate("fade", AnimatedValue::new(1.0, 0.0, 300.0, Easing::EaseOut));

engine.update(200.0);
assert!(engine.value("scroll").is_some());
engine.gc(); // limpiar animaciones completadas
```

## Dependencias

- Sin dependencias externas — solo std

## Estructura

| Módulo | Descripción |
|--------|-------------|
| (raíz) | `Easing`, `AnimatedValue`, `AnimationEngine`, `AnimationTrack`, `AnimationState`, `apply_easing()` |
