# API Reference — fc-theme

## Tipos principales

### `ChartTheme`
Tema completo con todos los design tokens (33 colores). Campos públicos: `background`, `pane_background`, `grid_line`, `text_primary`, `text_secondary`, `bullish`, `bearish`, `crosshair_line`, `watermark`, etc. Métodos: `dark()`, `light()`, `preset(name)`, `set_color(token, color)`, `get_color(token)`, `set_colors(updates)`.

### `ThemeToken`
Token type-safe para colores del tema. 33 variantes: `Background`, `PaneBackground`, `GridLine`, `TextPrimary`, `TextSecondary`, `Bullish`, `Bearish`, `CrosshairLine`, `SelectionBorder`, `MarkerUp`, `DrawingLine`, `Divider`, `Watermark`, etc.

### `ChartThemeBuilder`
Builder para temas personalizados. Métodos: `new()` (desde dark), `from_theme(theme)`, `with(name, color)`, `with_token(token, color)`, `build()`.

### `ThemeHandle`
Handle thread-safe para tema compartido (Arc<RwLock<ChartTheme>>). Métodos: `new(theme)`, `set(theme)`, `set_color(token, color)`, `read()`, `write()`, `snapshot()`. Implementa `Clone` (comparte estado).

### `ThemeError`
Errores de acceso al tema: `LockPoisoned`.

## Funciones

### `parse_token(name: &str) -> Option<ThemeToken>`
Parsea un nombre de token string a `ThemeToken`. Retorna `None` para tokens desconocidos.

## Re-exports

- `Rgba` — desde fc-primitives
- `LineStyle` — desde fc-primitives
