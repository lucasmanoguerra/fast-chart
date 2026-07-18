# Getting Started with fast-chart

## What is fast-chart?

fast-chart is a Rust library for GPU-accelerated financial charting. It produces
`DrawCommand`s that your application executes on any rendering backend (wgpu,
glow, Skia, software). The library owns chart logic; you own the GPU surface.

Target quality: indistinguishable from TradingView Lightweight Charts, with Rust's
safety, performance, and extensibility guarantees.

## Architecture in 30 Seconds

```
fast-chart-domain     Pure types, zero dependencies
                      Bar, Tick, Viewport, TimeSeries, Indicators, Drawings

fast-chart            Application layer + rendering abstractions
                      ChartController, Pane, LayoutManager, DrawCommand, Ports

fast-chart-renderer-wgpu   Optional wgpu backend (implements RendererBackend)
```

**Dependency direction**: `fast-chart-domain` ← `fast-chart` ← `fast-chart-renderer-wgpu`

The core library crate has zero GPU dependencies.

## Adding to Cargo.toml

```toml
[dependencies]
fast-chart = { path = "../fast-chart" }

# Optional: wgpu renderer
fast-chart-renderer-wgpu = { path = "../fast-chart-renderer-wgpu" }
```

Feature flags:

| Feature    | Default | Description                           |
|------------|---------|---------------------------------------|
| `serde`    | off     | Enable serde derives on domain types  |

## Quick Start

### 1. Create a Data Provider

```rust
use fast_chart::DataProvider;
use fast_chart::{Bar, Tick};

struct MyDataProvider {
    receiver: Option<std::sync::mpsc::Receiver<fast_chart::DataEvent>>,
}

impl DataProvider for MyDataProvider {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.receiver = Some(rx);
        // Spawn your data feed here...
        Ok(())
    }

    fn receiver(&self) -> Option<&std::sync::mpsc::Receiver<fast_chart::DataEvent>> {
        self.receiver.as_ref()
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn name(&self) -> &str { "MyFeed" }
}
```

### 2. Create an Interaction Handler

```rust
use fast_chart::{InteractionCommand, InteractionHandler, ViewportCommand};

struct MyHandler;

impl InteractionHandler for MyHandler {
    fn handle_event(&self, command: InteractionCommand) -> Vec<ViewportCommand> {
        match command {
            InteractionCommand::ZoomAtCursor { factor, screen_x } => {
                vec![ViewportCommand::ZoomAtCursor { factor, screen_x }]
            }
            InteractionCommand::PanBy { time_delta } => {
                vec![ViewportCommand::PanBy { time_delta }]
            }
            InteractionCommand::UpdateCrosshair { screen_x, screen_y } => {
                vec![ViewportCommand::SetCrosshairPosition {
                    x: screen_x,
                    y: screen_y,
                    time: 0,
                    price: 0.0,
                }]
            }
            _ => vec![],
        }
    }
}
```

### 3. Wire It Up

```rust
use fast_chart::ChartController;

let provider = Box::new(MyDataProvider { receiver: None });
let handler = Box::new(MyHandler);
let mut controller = ChartController::new(provider, handler);

// Poll for data events in your render loop
controller.poll_events();

// Access chart state for rendering
let state = controller.state();
// state.time_series, state.viewport, state.crosshair, etc.
```

## Domain Types

### Bar (OHLCV)

```rust
use fast_chart::Bar;

let bar = Bar::new(
    1700000000,   // timestamp (ms since epoch)
    100.0,        // open
    105.0,        // high
    99.0,         // low
    102.0,        // close
    5000,         // volume
).unwrap();

assert!(bar.is_bullish()); // close > open
```

### TimeSeries (Ring Buffer)

```rust
use fast_chart::{Bar, TimeSeries};

let mut series: TimeSeries<Bar, 100_000> = TimeSeries::new();

// Push data — oldest is evicted when full
series.push(bar);
series.push(another_bar);

// Access
let latest = series.latest();           // Option<&Bar>
let third = series.get(2);              // Option<&Bar>
let all: Vec<&Bar> = series.iter().collect();
```

### Viewport

```rust
use fast_chart::Viewport;

let mut vp = Viewport::default();
vp.zoom(2.0, 500.0);       // zoom 2x centered at x=500
vp.pan(1000, 2000);         // pan to time range [1000, 2000]
```

## Theme Configuration

### Built-in Themes

```rust
use fast_chart::theme::ChartTheme;

let dark = ChartTheme::dark();
let light = ChartTheme::light();
```

### Custom Theme via Builder

```rust
use fast_chart::theme::{ChartThemeBuilder, ThemeToken, Rgba};

let theme = ChartThemeBuilder::new()
    .with("background", Rgba::rgb(0.05, 0.05, 0.1))
    .with_token(ThemeToken::Bullish, Rgba::rgb(0.0, 0.9, 0.5))
    .with_token(ThemeToken::Bearish, Rgba::rgb(0.9, 0.2, 0.2))
    .build();
```

### Hot-Swap at Runtime

```rust
use fast_chart::theme::{ThemeHandle, ThemeToken, Rgba};

let handle = ThemeHandle::new(ChartTheme::dark());

// From any thread
handle.set_color(ThemeToken::Bullish, Rgba::rgb(0.0, 1.0, 0.0));

// Read current theme
let theme = handle.read();
let color = theme.get_color(ThemeToken::Bullish);
```

## Indicators

16 built-in indicators, all computed from the domain layer:

```rust
use fast_chart::{Ema, Rsi, Macd, Bollinger, Atr};

let ema = Ema::new(20);        // 20-period EMA
let rsi = Rsi::new(14);        // 14-period RSI
let macd = Macd::new(12, 26, 9);
let bb = Bollinger::new(20, 2.0);
let atr = Atr::new(14);

// Each implements the Indicator trait
let value = ema.calculate(&series);
```

Full list: SMA, EMA, VWAP, RSI, MACD, Bollinger, ATR, ADX, CCI, Stochastic,
Williams %R, Parabolic SAR, Ichimoku, Supertrend, Heikin Ashi, Renko, Kagi.

## Drawing Tools

```rust
use fast_chart::{
    TrendLine, Rectangle, FibonacciRetracement,
    Pitchfork, Ellipse, HorizontalLine, VerticalLine,
};

let trend = TrendLine::new(
    ChartPoint { timestamp: 1000, price: 100.0 },
    ChartPoint { timestamp: 5000, price: 120.0 },
);

let fib = FibonacciRetracement::new(
    ChartPoint { timestamp: 1000, price: 100.0 },
    ChartPoint { timestamp: 5000, price: 120.0 },
);
```

## Multi-Pane Layout

```rust
use fast_chart::app::layout::HorizontalSplit;

// Two panes: candles on top, RSI on bottom
let layout = HorizontalSplit {
    ratios: vec![0.7, 0.3],
};
```

## Rendering Pipeline

The library produces `DrawCommand`s — you execute them:

```rust
use fast_chart::DrawCommand;

let commands: Vec<DrawCommand> = vec![
    DrawCommand::line(0.0, 0.0, 100.0, 50.0, [1.0; 4], 2.0, 5),
    DrawCommand::filled_rect(10.0, 10.0, 80.0, 40.0, [0.0, 1.0, 0.0, 0.5], 3),
    DrawCommand::text(50.0, 25.0, "BTC", [1.0; 4], 14.0, 10),
];

// Your backend executes them
backend.execute(&commands);
```

## Custom Series Renderer

Implement `SeriesRenderer` to create your own series type:

```rust
use fast_chart::{SeriesRenderer, DrawCommand, Rect, SeriesHit};

struct MyCustomSeries { /* ... */ }

impl SeriesRenderer for MyCustomSeries {
    fn update(&mut self, data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
        // Transform your data into DrawCommands
        vec![DrawCommand::polyline(
            vec![[10.0, 10.0], [50.0, 50.0], [90.0, 20.0]],
            [1.0, 0.5, 0.0, 1.0],
            2.0,
            650,
        )]
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit> {
        None // implement hit testing
    }

    fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }
}
```

## RendererBackend (GPU)

If using the wgpu backend:

```rust
use fast_chart_renderer_wgpu::{WgpuBackend, WgpuRenderer};

// Initialize wgpu
let instance = wgpu::Instance::default();
let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default())
    .await.unwrap();
let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
    .await.unwrap();

let mut backend = WgpuBackend::new(device, queue);
backend.set_surface(surface, config);

// Each frame
backend.begin_frame();
backend.clear([0.05, 0.05, 0.1, 1.0]);
backend.execute(&draw_commands);
backend.end_frame();
```

## Next Steps

- [Architecture Overview](architecture/overview.md) — full system design
- [Domain Crate](architecture/domain-crate.md) — pure types deep dive
- [Render Pipeline](architecture/render-pipeline.md) — pass system and dirty rendering
- [Theme System](architecture/theme-system.md) — design tokens and hot-swap
- [Migration Guide](migration-guide.md) — upgrading from earlier phases
