# API Reference — fc-sessions

## Tipos principales

### `Session`
Definición de sesión de trading. Campos: `name`, `open_hour/minute`, `close_hour/minute`, `color`, `line_style`, `width`, `active`. Constructor: `new(name, open_hour, open_minute, close_hour, close_minute)`. Métodos: `duration_minutes()`, `contains_utc(hour, minute)`.

### `ExchangeSessions`
Sesiones predefinidas de exchanges: `us_regular()` (9:30–16:00 ET), `us_premarket()` (4:00–9:30 ET), `us_afterhours()` (16:00–20:00 ET), `london()` (8:00–16:30 UTC), `tokyo()` (9:00–15:00 JST).

### `SessionLineStyle`
Estilo de línea de sesión: `Solid`, `Dashed`, `Dotted`.

### `SessionLineConfig`
Configuración del renderizado: `sessions` (Vec<Session>), `default_color`, `show_labels`.

### `SessionLineRenderer`
Renderizador de líneas de sesión. Constructor: `new(config)`. Método: `render(visible_start, visible_end, y_top, y_bottom, time_to_x) -> Vec<SessionLine>`.

### `SessionLine`
Línea renderizada: `x`, `y_top`, `y_bottom`, `color`, `line_style`, `width`, `session_name`.
