# fc-theme

Sistema de theming para charts con design tokens. Incluye temas predefinidos (dark/light), builder para temas personalizados, hot-swap de colores en runtime, y `ThemeHandle` para sharing thread-safe.

## Uso

```rust
use fc_theme::{ChartTheme, ThemeToken, ChartThemeBuilder, ThemeHandle, Rgba};

// Temas predefinidos
let dark = ChartTheme::dark();
let light = ChartTheme::light();

// Builder personalizado
let theme = ChartThemeBuilder::new()
    .with("background", Rgba::rgb(0.1, 0.1, 0.1))
    .with_token(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0))
    .build();

// Hot-swap en runtime
let mut theme = ChartTheme::dark();
theme.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));
assert_eq!(theme.get_color(ThemeToken::Bullish), Rgba::rgb(0.0, 1.0, 0.0));

// ThemeHandle — sharing thread-safe
let handle = ThemeHandle::new(ChartTheme::dark());
handle.set_color(ThemeToken::Bearish, Rgba::rgb(1.0, 0.0, 0.0));
let snapshot = handle.snapshot();
```

## Dependencias

- `fc-primitives` — Rgba, LineStyle

## Estructura

| Módulo | Descripción |
|--------|-------------|
| (raíz) | `ChartTheme`, `ThemeToken`, `ChartThemeBuilder`, `ThemeHandle`, `ThemeError`, `parse_token()` |
