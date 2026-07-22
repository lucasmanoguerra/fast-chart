# API Reference — fc-input

## Tipos principales

### `InputEvent`
Enum de eventos de entrada platform-agnostic. Variantes: `MouseMove`, `MouseDown`, `MouseUp`, `Wheel`, `MouseDoubleClick`, `TouchStart`, `TouchMove`, `TouchEnd`, `Pinch`, `KeyDown`, `KeyUp`, `StylusMove`, `StylusDown`, `StylusUp`.

### `KeyCode`
Códigos de teclas: `ArrowLeft/Right/Up/Down`, `Home`, `End`, `PageUp/Down`, `Plus`, `Minus`, `Escape`, `Delete`, `Letter(char)`, `Key0-Key9`, `F1-F12`, `Shift`, `Control`, `Alt`, `Super`, `Unknown`.

### `MouseButton`
Botones de mouse: `Left`, `Right`, `Middle`, `Other(u16)`.

### `ModifierState`
Estado de modificadores (struct con campos bool: `shift`, `ctrl`, `alt`, `super_key`).

### `ModifierFlags`
Bitflags para modificadores: `SHIFT`, `CTRL`, `ALT`, `SUPER`. Más eficiente que `ModifierState` en hot paths.

## Eventos de payload

### `MouseMoveEvent`
Posición del cursor: `x`, `y`, `modifiers`.

### `MouseButtonEvent`
Botón presionado: `button`, `x`, `y`, `modifiers`.

### `WheelEvent`
Scroll: `delta_x`, `delta_y`, `x`, `y`, `modifiers`.

### `TouchEvent`
Contacto touch: `id`, `x`, `y`, `pressure`.

### `PinchEvent`
Gesto pinch: `center_x`, `center_y`, `distance`, `scale`.

### `KeyEvent`
Tecla: `key`, `modifiers`, `repeat`.

### `StylusEvent`
Lápiz digitalizador: `x`, `y`, `pressure`, `tilt_x`, `tilt_y`, `modifiers`.

## Funciones convenience

`InputEvent` ofrece constructores conveniently: `mouse_move()`, `mouse_left_down()`, `mouse_left_up()`, `wheel()`, `key_down()`, `touch_start()`, `pinch()`, `stylus_move()`, y variantes con modificadores.
