# Changelog

All notable changes to the fast-chart project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - Unreleased

### Added

#### Phase 1 — Domain Foundation
- `Bar` and `Tick` OHLCV data types with validation
- `TimeSeries<N>` fixed-capacity ring buffer with iterator support
- `Viewport` for visible time/value window tracking with zoom and pan
- `ChartError` enum for domain-level error handling
- `Drawing` trait and concrete drawing types: `TrendLine`, `HorizontalLine`, `VerticalLine`, `Rectangle`, `Path`, `Ray`, `Segment`, `Pitchfork`, `Ellipse`, `Arrow`, `FibonacciRetracement`, `FibonacciExtension`, `TextDrawing`, `LabelDrawing`, `ImageDrawing`
- `DrawingSet` collection with insert, remove, and query operations
- `Indicator` trait with `OverlayMode` for overlay/pane rendering
- `PriceLine`, `PriceLineId`, `LineStyle`, `LabelPosition` for horizontal price markers
- `PriceScaleId`, `PriceScaleMode`, `PriceScale` for multi-scale price axis support
- `PriceFormatter` for price label formatting with configurable precision
- Indicator re-exports: EMA, SMA, VWAP, RSI, MACD, Bollinger Bands, ATR, Stochastic, Ichimoku Cloud, ADX, OBV, VWMA, Donchian Channel, Keltner Channel, SuperTrend, Pivot Points, and a prelude module
- `Marker`, `MarkerPosition`, `MarkerShape` for point annotations
- `CrosshairMode` and `MagnetMode` for crosshair behavior
- `TimeScale` for time axis management with business day support
- `SeriesRenderer` trait for custom series rendering

#### Phase 2 — Rendering Engine Architecture
- `DrawCommand` enum with 16 render primitives: `DrawLine`, `DrawRect`, `DrawCircle`, `DrawTriangle`, `DrawPath`, `DrawText`, `DrawImage`, `DrawCandle`, `DrawBar`, `DrawArea`, `DrawHistogram`, `DrawBaseline`, `DrawStepLine`, `DrawDashedLine`, `DrawMarker`, `DrawCrosshairLine`
- `LineStyle` enum: `Solid`, `Dashed`, `Dotted`
- `DrawLayer` enum with 15 z-indexed rendering layers (Background through Cursor)
- `RendererBackend` trait: renderer-agnostic GPU backend interface
- `CoordinatePipeline` for world ↔ screen coordinate transforms with pixel-perfect alignment
- `ScreenPoint` and `WorldPoint` coordinate types
- `RenderContext` with pipeline, clip rect, pane bounds, and timing info
- `Rect` axis-aligned bounding box with hit-testing utilities
- `RenderPass` enum with 12 ordered passes (Background through Debug)
- `PassTracker` for dirty-pass tracking to skip empty passes
- `RenderPipeline` orchestrator with begin/submit/end/execute lifecycle
- `PassBatch` and `FrameStats` for batch grouping and frame statistics
- `ScreenRect`, `DirtyRegion`, `DirtyRegionTracker` for dirty rendering optimization
- `DrawingInteraction` with `DrawingAction` enum for user interaction events
- `DrawingManager` for managing active drawing tools and hit-testing
- `IndicatorRenderer` for overlay/pane indicator rendering
- Pixel-perfect alignment utilities: `snap_to_pixel`, `snap_rect_to_pixel`, `snap_line_to_pixel`, `snap_width`
- `SeriesHit` for series hit-test results

#### Phase 3 — GPU Backend (wgpu)
- `WgpuBackend` implementing `RendererBackend` with wgpu device/queue/surface management
- `WgpuRenderer` convenience wrapper with frame lifecycle methods
- `Vertex` and `Uniforms` GPU data types with bytemuck Pod/Zeroable
- `GpuCache` for resource slot reuse across frames
- `RenderPipeline` (wgpu) wrapper around wgpu::RenderPipeline
- `ScissorRect` and `ScissorManager` for multi-pane scissor stack with intersection
- Vertex generation functions: `generate_line_vertices`, `generate_rect_vertices`, `generate_circle_vertices`
- Render modules: backend, cache, commands, context, coordinates, drawing, drawing_interaction, drawing_manager, indicator_renderer, layers, passes, pipeline, pixel_perfect, renderers, scissor, series_renderer, session, vertex_gen

#### Phase 4 — Integration & Theme System
- `ChartTheme` with complete design token system
- `ThemeToken` enum for runtime color hot-swap
- Theme presets: `dark()`, `light()`, `midnight()`
- Builder pattern: `ChartTheme::builder()` with `build()`, `preset()`, `color()`, `generate()`
- Custom font loading support
- Theme color categories: background, grid, text, price scale, time scale, series, crosshair, selection, hover, markers, drawings, watermark, divider
- `ChartRenderer` trait for port-based rendering
- `DataProvider` trait and `DataEvent` enum for market data sources
- `InteractionHandler` trait with `InteractionCommand` and `ViewportCommand` enums

#### Phase 5 — Examples & Polish
- Complete candlestick example with theme, viewport, and data provider integration
- Theme builder example with custom color palette
- Drawing tools example with interactive drawing creation
- Indicator overlay example with RSI, MACD, and Bollinger Bands
- Domain type examples: Bar, Tick, TimeSeries, Viewport, PriceLine, PriceScale, Marker, CrosshairMode
- Render type examples: DrawCommand, DrawLayer, RenderPass, CoordinatePipeline, ScreenPoint, WorldPoint, Rect
- Renderer backend examples: WgpuBackend, WgpuRenderer
- Indicator examples: EMA, SMA, RSI, MACD, Bollinger Bands, ATR, Stochastic, Ichimoku Cloud, ADX, OBV, VWMA
- Drawing tool examples: TrendLine, HorizontalLine, Rectangle, Fibonacci, Pitchfork
- Port trait examples: DataProvider, ChartRenderer, InteractionHandler
- Theme preset examples: dark, light, midnight
- Integration examples: full chart setup, multi-pane layout, real-time data
- Performance examples: large dataset handling, batch rendering, caching strategies
- API reference examples for all public types and traits
- 325 doc tests with 95%+ coverage across all crates

#### Phase 6 — Performance & Memory
- Zero-copy iterators for TimeSeries
- Arena-based drawing allocation
- Pre-allocated command buffers
- Cache-friendly data layout for OHLCV data
- SIMD-friendly data alignment

#### Phase 7 — Testing & Documentation
- Comprehensive unit tests for all domain types
- Integration tests for renderer pipeline
- Property-based testing for coordinate transforms
- Snapshot tests for draw command output
- API consistency audit across all crates
- Missing derives added: `PartialEq` on `Viewport`, `Debug` on `DataEvent`, `ViewportCommand`, `ScissorManager`
- Missing doc comments added to `ChartError`, `Vertex`, `Uniforms`, `GpuCache`, `RenderPipeline` (wgpu), `ScissorManager`, `PassBatch`, `FrameStats`

### Fixed
- `Viewport` was missing `PartialEq` derive (added)
- `DataEvent` was missing `Debug` derive (added)
- `ViewportCommand` was missing `Debug` derive (added)
- `ScissorManager` was missing `Debug` derive (added)
- `ChartError` was missing doc comment (added)
- `Vertex` and `Uniforms` were missing doc comments (added)
- `GpuCache` was missing doc comment and `Debug` derive (added)
- `RenderPipeline` (wgpu) was missing doc comment and `Debug` derive (added)
- `ScissorManager` was missing doc comment on the struct (added)
- `PassBatch` fields were missing doc comments (added)
- `FrameStats` fields were missing doc comments (added)

### Known Issues
- Three identical `LineStyle` enums exist across `fast-chart-domain::price_line`, `fast-chart::render::commands`, and `fast-chart::theme`. These should be unified in a future refactor.
- `ChartRenderer` trait has only one method (`resize`). It may need additional methods as the rendering pipeline matures.
