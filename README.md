<div align="center">

# Fast Chart

**GPU-accelerated trading chart library for Rust**

High-performance · Low-latency · Hexagonal Architecture · 7 Indicators · 8-Layer GPU Pipeline

[![Crates.io](https://img.shields.io/crates/v/fc-types.svg)](https://crates.io/crates/fc-types)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-324%20passing-brightgreen)]()
[![wgpu](https://img.shields.io/badge/wgpu-22-purple)](https://wgpu.rs)

[Installation](#installation) · [Quick Start](#quick-start) · [Architecture](#architecture) · [Indicators](#indicators) · [Contributing](#contributing)

</div>

---

## What is Fast Chart?

Fast Chart is a **professional-grade trading chart library** written in Rust, designed for applications that demand **real-time rendering**, **sub-millisecond latency**, and **deterministic behavior**.

Built on [wgpu](https://wgpu.rs) for GPU-accelerated rendering with a clean hexagonal architecture that keeps domain logic pure and testable.

**Before**: "I need to build a trading terminal, but every charting library is either slow, in JavaScript, or locked to a specific framework."

**After**: You have a Rust-native, GPU-accelerated charting engine with 7 built-in indicators, multi-pane layouts, and a pluggable data layer — ready to integrate into any Rust GUI framework.

## Features

| Feature | Description |
|---------|-------------|
| **GPU Rendering** | 8-layer wgpu pipeline: Grid → Candlestick → Line → Divider → Crosshair → Markers → Price Lines → Text |
| **7 Indicators** | SMA, EMA, RSI, MACD, Bollinger Bands, Stochastic, Ichimoku Cloud |
| **Hexagonal Architecture** | Domain → Core → App layers with strict dependency direction |
| **Multi-Pane Layout** | Vertical pane stack with draggable dividers and per-pane scissor rects |
| **Kinetic Scrolling** | Momentum-based pan with configurable friction |
| **Interactive Crosshair** | Real-time cursor tracking with OHLC tooltip |
| **Viewport Management** | Zoom, pan, auto-fit with price/time coordinate conversion |
| **Pluggable Data** | `DataProvider` trait — plug any data source (WebSocket, REST, file) |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fc-types = "0.1"
fast-chart-core = "0.1"
```

Or for the full demo application:

```bash
git clone https://github.com/lucasmanoguerra/fast-chart.git
cd fast-chart
cargo run --release -p fast-chart-app
```

## Quick Start

```rust
use fc_types::bar::Bar;
use fast_chart_core::app::chart_controller::ChartController;

// Create a chart controller with your data provider
let mut controller = ChartController::new(
    Box::new(my_data_provider),
    Box::new(my_interaction_handler),
);

// Process incoming data
controller.tick();

// Access chart state
let state = controller.state();
println!("Bars loaded: {}", state.time_series.len());
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  fast-chart-app                  │
│  (adapters: GPU, input, data, config)           │
├─────────────────────────────────────────────────┤
│                 fast-chart-core                  │
│  (app: controller, layout, pane, viewport)      │
│  (ports: DataProvider, InteractionHandler)       │
├─────────────────────────────────────────────────┤
│                fc-types                 │
│  (Bar, Viewport, Crosshair, Indicators, etc.)   │
└─────────────────────────────────────────────────┘
```

**Dependency rule:** Domain depends on nothing. Core depends only on Domain. App depends on Core.

### Why Hexagonal?

- **Domain** is pure Rust — no GPU, no windowing, no async. Testable in isolation.
- **Core** contains application logic — orchestrates data flow, manages state. Still wgpu-free.
- **App** is the adapter layer — wgpu rendering, winit windows, config hot-reload. Swappable.

## Rendering Pipeline (8 Layers)

| Layer | Renderer | Description |
|-------|----------|-------------|
| 1 | `GridRenderer` | Background grid lines |
| 2 | `CandleRenderer` | OHLC candlestick bars (world-space, GPU-projected) |
| 3 | `LineRenderer` | Close-price line series (NDC-space, CPU-projected) |
| 4 | Divider lines | Pane separators |
| 5 | `CrosshairRenderer` | Cursor crosshair lines |
| 6 | `MarkerRenderer` | Trade annotations (per-pane scissor) |
| 7 | `PriceLineRenderer` | Horizontal price levels (per-pane scissor) |
| 8 | `GpuTextRenderer` | Axis labels + crosshair tooltip (glyphon) |

## Indicators

| Indicator | Description | Parameters |
|-----------|-------------|------------|
| **SMA** | Simple Moving Average | `period` |
| **EMA** | Exponential Moving Average | `period` |
| **RSI** | Relative Strength Index | `period` |
| **MACD** | Moving Average Convergence Divergence | `fast`, `slow`, `signal` |
| **Bollinger** | Bollinger Bands | `period`, `std_dev` |
| **Stochastic** | Stochastic Oscillator | `k_period`, `d_period` |
| **Ichimoku** | Ichimoku Cloud | `tenkan`, `kijun`, `senkou_b` |

## Project Structure

```
fast-chart/
├── fc-types/        # Zero-dependency domain types
│   └── src/
│       ├── bar.rs            # OHLCV bar
│       ├── viewport.rs       # Viewport (time + value range)
│       ├── crosshair.rs      # Crosshair state + magnet
│       ├── indicator.rs      # Indicator trait
│       ├── indicators/       # 7 built-in indicators
│       ├── series.rs         # Ring-buffer time series
│       ├── marker.rs         # Trade markers
│       ├── price_line.rs     # Horizontal price lines
│       └── price_scale.rs    # Price formatting
├── fast-chart-core/          # Application logic (wgpu-free)
│   └── src/
│       ├── app/
│       │   ├── chart_controller.rs  # Central orchestrator
│       │   ├── layout_manager.rs    # Multi-pane layout
│       │   ├── pane.rs              # Pane (viewport, markers, scales)
│       │   └── viewport_management.rs  # Coordinate conversion
│       └── ports/
│           ├── data_provider.rs     # Data source trait
│           ├── interaction.rs       # User input trait
│           └── render.rs            # Renderer trait
├── fast-chart-app/           # wgpu rendering + winit window
│   └── src/
│       ├── adapters/
│       │   ├── gpu_renderer.rs      # Main GPU orchestrator
│       │   ├── rendering/           # 8 sub-renderers
│       │   ├── data/                # Data adapters
│       │   └── input/               # Input handling
│       └── config/                  # TOML config + hot-reload
└── Cargo.toml                # Workspace root
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `wgpu` 22 | GPU rendering |
| `winit` 0.30 | Window management |
| `glyphon` 0.6 | Text rendering |
| `bytemuck` | GPU buffer casting |
| `pollster` | Async block_on |
| `serde` + `toml` | Configuration |
| `notify` | Config hot-reload |

## Testing

```bash
# Run all 324 tests
cargo test --workspace

# Run domain tests only
cargo test -p fc-types

# Run core tests only
cargo test -p fast-chart-core

# Run app tests only
cargo test -p fast-chart-app
```

## Contributing

1. Create a feature branch: `git checkout -b feat/my-feature`
2. Make changes and add tests
3. Ensure all tests pass: `cargo test --workspace`
4. Commit with conventional commits: `feat: add new indicator`
5. Push and create a PR to `main`

See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.

## Roadmap

- [ ] Drawing tools (trendlines, fibonacci, horizontal rays)
- [ ] Real-time WebSocket data adapter
- [ ] Custom indicator API (user-defined indicators)
- [ ] Theme system (dark/light/custom)
- [ ] Export to PNG/SVG
- [ ] WASM target support

## License

MIT — see [LICENSE](LICENSE) for details.
