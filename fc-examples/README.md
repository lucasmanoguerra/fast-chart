# fc-examples

Aplicaciones de ejemplo para la librería de gráficos de trading fast-chart. Incluye demos completas que muestran el uso de la librería: velas simples, layouts multi-panel, herramientas de dibujo, animaciones, temas personalizados y API de builders.

## Uso

```bash
# Ejecutar el ejemplo principal (chart interactivo con wgpu)
cargo run -p fc-examples

# O ejecutar un ejemplo específico
cargo run -p fc-examples --example simple_candle
cargo run -p fc-examples --example multi_pane
cargo run -p fc-examples --example drawing_tools
cargo run -p fc-examples --example animation_demo
cargo run -p fc-examples --example builder_api
cargo run -p fc-examples --example custom_theme
```

## Ejemplos disponibles

| Ejemplo | Descripción |
|---------|-------------|
| `simple_candle` | Chart de velas básico con data simulada |
| `multi_pane` | Layout con múltiples paneles (candles + volume + RSI) |
| `drawing_tools` | Demo de herramientas de dibujo (líneas, fib, rectángulos) |
| `animation_demo` | Transiciones animadas con easing |
| `builder_api` | Construcción declarativa con ChartBuilder |
| `custom_theme` | Tema personalizado con hot-swap |

## Dependencias

- `fc-app`, `fc-primitives`, `fc-domain`, `fc-render`
- `winit`, `wgpu`, `pollster`, `env_logger`, `log`, `bytemuck`, `glyphon`
- `serde`, `toml`, `notify` (config hot-reload)
