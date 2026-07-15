# fast-chart Architecture Overview

## Vision

fast-chart is a Rust **library** for GPU-accelerated financial charting. Applications embed it; the library owns chart logic and produces render commands. The host application owns the GPU surface and executes them.

**Target**: visual quality indistinguishable from TradingView Lightweight Charts, with Rust's safety, performance, and extensibility guarantees.

---

## Crate Structure

```
fast-chart-domain          Pure types, zero dependencies
                           Bar, Tick, Viewport, LinearScale, TimeScale,
                           Crosshair, PriceScale, Marker, Drawing tools,
                           Indicator trait + 16 implementations, Error types

fast-chart                 Library crate — chart logic, rendering abstractions
                           ChartController, Pane, LayoutManager,
                           Ports (ChartRenderer, DataProvider, InteractionHandler)

fast-chart-examples        Example applications (wgpu + winit)
                           Simulated data provider, GPU renderer, input handler
```

**Dependency direction**: `fast-chart-domain` ← `fast-chart` → `fast-chart-examples`

The library crate (`fast-chart`) has **zero GPU dependencies** — only depends on `fast-chart-domain`.

---

## Hexagonal Architecture (Ports & Adapters)

```
                    ┌─────────────────────────────┐
                    │      fast-chart-domain       │
                    │   (Pure types, zero deps)    │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │        fast-chart            │
                    │   (Library: logic + ports)   │
                    │                              │
                    │  ┌─────────┐  ┌──────────┐  │
                    │  │   app/  │  │  ports/  │  │
                    │  │         │  │          │  │
                    │  │ ChartController       │  │
                    │  │ Pane   │  │ ChartRenderer│
                    │  │ LayoutManager         │  │
                    │  │ ViewportManager       │  │
                    │  │ FrameCounter          │  │
                    │  │ IndicatorService      │  │
                    │  └─────────┘  └──────────┘  │
                    └──────────────┬──────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                     │
   ┌──────────▼────────┐ ┌────────▼────────┐ ┌─────────▼─────────┐
   │  fast-chart-      │ │   Data Provider │ │  Interaction      │
   │  examples         │ │   Adapter       │ │  Handler          │
   │  (GPU renderer)   │ │   (Simulated)   │ │  (Winit)          │
   └───────────────────┘ └─────────────────┘ └───────────────────┘
```

### Ports (Interfaces)

| Port | Location | Purpose |
|------|----------|---------|
| `ChartRenderer` | `fast-chart/src/ports/render.rs` | Abstract rendering surface (resize) |
| `DataProvider` | `fast-chart/src/ports/data_provider.rs` | Market data source (bars, ticks) |
| `InteractionHandler` | `fast-chart/src/ports/interaction.rs` | Input event processing |

### Adapters (Implementations)

| Adapter | Location | Implements |
|---------|----------|------------|
| `GpuRenderer` | `fast-chart-examples/src/adapters/gpu_renderer.rs` | `ChartRenderer` |
| `SimulatedDataProvider` | `fast-chart-examples/src/adapters/data/simulated.rs` | `DataProvider` |
| `WinitInteractionHandler` | `fast-chart-examples/src/adapters/input/handler.rs` | `InteractionHandler` |

---

## Data Flow

```
Market Data → DataProvider → DataEvent (mpsc) → ChartController → ChartState
                                                                    │
User Input → InteractionHandler → InteractionCommand → ChartController
                                                                    │
                                                            ChartState
                                                                │
                                                    RedrawRequested
                                                                │
                                                    GpuRenderer::render()
                                                                │
                                                            GPU Output
```

---

## Key Types

### Domain Layer (`fast-chart-domain`)

- **`Bar`** — OHLCV price bar with validation
- **`Tick`** — Bid/ask quote
- **`TimeSeries<T, N>`** — Ring buffer for fixed-capacity time series
- **`Viewport`** — Visible time/value window with zoom/pan
- **`LinearScale`** / **`TimeScale`** — Value/time to pixel coordinate mapping
- **`Crosshair`** — Cursor position with OHLC magnet snapping
- **`PriceScale`** — Price formatting and auto-fit
- **`Marker`** — Point annotations (buy/sell signals)
- **`PriceLine`** — Horizontal price levels
- **`Indicator` trait** — Technical indicator computation (16 implementations)
- **Drawing tools** — TrendLine, Rectangle, Fibonacci, Pitchfork, Ellipse, Path

### Application Layer (`fast-chart`)

- **`ChartController`** — Central orchestrator: data polling, input routing, state management
- **`Pane`** — Independent chart pane with viewport, series, and overlays
- **`LayoutManager`** — Multi-pane vertical stack with draggable dividers
- **`ViewportManager`** — Zoom/pan/auto-fit with coordinate mapping
- **`IndicatorService`** — Indicator registry and calculation dispatch
- **`FrameCounter`** — FPS monitoring

---

## Testing Strategy

- **Domain**: 363 unit tests + 10 doc tests — pure logic, no I/O
- **Library**: 120 unit tests + 33 integration tests — mocked ports
- **Examples**: 71 unit tests — adapter logic
- **Total**: 564+ tests, all passing

Each layer is testable in isolation through port abstractions.

---

## Design Principles

1. **Hexagonal Architecture** — Domain never depends on adapters
2. **Zero GPU deps in library** — Renderer-agnostic by design
3. **Gateway pattern** — `fast-chart` re-exports all domain types
4. **Port abstractions** — All I/O behind traits
5. **Ring buffer series** — Fixed-capacity, zero-allocation time series
6. **Validation at construction** — `Bar::new()` returns `Result`
