# fc-input

Eventos de entrada platform-agnostic para el sistema de interacción del chart. `InputEvent` es el vocabulario crudo e independiente del dispositivo que los adaptores (winit, web, etc.) traducen desde eventos específicos de plataforma. El `InteractionEngine` los consume para producir `ChartCommand`s.

## Uso

```rust
use fc_input::{InputEvent, KeyCode, ModifierState, ModifierFlags};

// Eventos de mouse
let move_event = InputEvent::mouse_move(100.0, 200.0);
let click = InputEvent::mouse_left_down(50.0, 75.0);
let scroll = InputEvent::wheel(0.0, -3.0, 400.0, 300.0);

// Eventos de teclado
let key = InputEvent::key_down(KeyCode::ArrowRight);
let ctrl_z = InputEvent::key_down_with_modifiers(
    KeyCode::Letter('z'),
    ModifierState { shift: false, ctrl: true, alt: false, super_key: false },
);

// Touch y stylus
let touch = InputEvent::touch_start(1, 100.0, 200.0);
let pinch = InputEvent::pinch(400.0, 300.0, 200.0, 1.5);
let stylus = InputEvent::stylus_move(100.0, 200.0, 0.75);

// ModifierFlags — bitflags para hot paths
let flags = ModifierFlags::from_state(ctrl_z_state);
assert!(flags.contains(ModifierFlags::CTRL));
```

## Dependencias

- `bitflags` — bitflags eficientes para ModifierFlags

## Estructura

| Módulo | Descripción |
|--------|-------------|
| (raíz) | `InputEvent`, `KeyCode`, `MouseButton`, `ModifierState`, `ModifierFlags`, eventos de mouse/touch/keyboard/stylus |
| `engine` | `InteractionEngine` — consume InputEvent, produce ChartCommand |
| `gesture` | Detección de gestos (drag, pinch, tap) |
| `keyboard` | Manejo de atajos de teclado |
| `crosshair` | Interacción del crosshair con el input |
| `pan` | Lógica de pan (arrastre) |
| `zoom` | Lógica de zoom (rueda, pinch) |
