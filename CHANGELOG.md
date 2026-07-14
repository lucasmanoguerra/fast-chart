# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Multi-pane layout with draggable dividers
- Per-pane scissor rect rendering for markers and price lines
- Pluggable formatter for axis labels and crosshair tooltip
- Per-pane marker and price line renderers
- Configurable pane heights with min/max constraints

### Changed

- Removed dead `renderer` field from `ChartController`
- Removed `RendererBridge` adapter (no longer needed)
- Wired pane-level markers and price lines into GPU renderer
- Wired pane formatter into axis and crosshair labels

### Fixed

- Corrected pane resize on high-DPI displays
- Fixed marker rendering clipping across pane boundaries

## [0.1.0] - 2025-07-13

### Added

- Initial release
- 8-layer GPU rendering pipeline (wgpu)
- OHLCV candlestick chart rendering
- Line series rendering
- Grid background rendering
- Interactive crosshair with OHLC tooltip
- Kinetic scrolling with configurable friction
- 7 built-in indicators: SMA, EMA, RSI, MACD, Bollinger, Stochastic, Ichimoku
- Multi-pane layout system
- Viewport management (zoom, pan, auto-fit)
- Price/time coordinate conversion
- Trade markers
- Horizontal price lines
- TOML configuration with hot-reload
- Hexagonal architecture (Domain → Core → App)
- 324 tests passing

[Unreleased]: https://github.com/lucasmanoguerra/fast-chart/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lucasmanoguerra/fast-chart/releases/tag/v0.1.0
