# Chart Configuration Specification

## Purpose

Define the chart configuration system: a `ChartConfig` struct that holds all visual and behavioral settings, deserialized from a TOML/YAML config file with hot-reload support.

## Requirements

### Requirement: ChartConfig struct

A `ChartConfig` struct MUST aggregate all configurable settings. It SHALL contain fields for color scheme, font configuration, default timeframe, symbol name, and pane layout.

#### Scenario: Config constructed with defaults

- GIVEN `ChartConfig::default()`
- WHEN examining its fields
- THEN all fields SHALL have sensible default values

#### Scenario: Config merges user settings

- GIVEN a `ChartConfig` with partial user overrides
- WHEN merged with defaults
- THEN user-specified fields override defaults; unspecified fields remain at default

### Requirement: Color scheme

The color scheme SHALL define: background color, grid line color, candle up color, candle down color, indicator line colors, and crosshair color. Each SHALL be an sRGB hex value.

#### Scenario: Default colors are readable

- GIVEN the default color scheme
- THEN background is dark, grid is semi-transparent, candle up is green, candle down is red

#### Scenario: Custom colors applied

- GIVEN a config with background="#1a1a2e" and candle_up="#00ff88"
- WHEN the config is loaded
- THEN rendering uses the custom colors

### Requirement: Font configuration

Font configuration SHALL specify: font family, font size for axis labels, font size for tooltip text, and font size for pane titles.

#### Scenario: Default font parameters

- GIVEN the default font config
- THEN axis labels default to 11px, tooltips to 12px, pane titles to 13px

#### Scenario: Custom font applied

- GIVEN font_family="Fira Code", axis_size=12
- WHEN the config is loaded
- THEN axis labels render in Fira Code at 12px

### Requirement: Default timeframe

The config SHALL specify the initial timeframe on startup. Supported values: `1m`, `5m`, `15m`, `60m`, `1D`, `1W`.

#### Scenario: Startup shows default timeframe

- GIVEN config with timeframe="1D"
- WHEN the chart starts
- THEN daily bars are displayed initially

#### Scenario: Invalid timeframe falls back

- GIVEN config with timeframe="7m" (unsupported)
- WHEN the config is loaded
- THEN the timeframe SHALL fall back to "1m"

### Requirement: Symbol name

The config SHALL store the symbol/instrument name as a string (e.g., "BTC/USD", "AAPL").

#### Scenario: Symbol displayed in UI

- GIVEN config with symbol="BTC/USD"
- WHEN the chart renders
- THEN the symbol name appears in the pane title

#### Scenario: Empty symbol default

- GIVEN config with empty symbol string
- WHEN the chart starts
- THEN the title SHALL display "N/A"

### Requirement: Pane layout configuration

The config SHALL define the number of panes, height ratios, and indicator assignments per pane.

#### Scenario: Two panes with RSI

- GIVEN config specifying 1 price pane + 1 indicator pane with "RSI(14)"
- WHEN the layout initializes
- THEN the indicator pane contains an RSI(14) overlay

#### Scenario: Multiple indicator assignments

- GIVEN config assigning MACD and RSI to the same pane
- WHEN the layout initializes
- THEN both indicators render in that pane

### Requirement: serde deserialization

`ChartConfig` MUST implement `serde::Deserialize`. Config SHALL load from TOML or YAML format.

#### Scenario: TOML config loads

- GIVEN a valid TOML config file with all fields
- WHEN deserialized via `serde`
- THEN a complete `ChartConfig` is produced

#### Scenario: Missing optional fields

- GIVEN a TOML config with only symbol defined
- WHEN deserialized
- THEN missing fields SHALL use their default values

### Requirement: Hot-reload capability

The config SHALL support runtime mutation. A file watcher SHOULD detect config file changes and trigger a reload event.

#### Scenario: File change triggers reload

- GIVEN a running chart with config file at `chart.toml`
- WHEN the file is modified and saved
- THEN within 500ms, the `ConfigReloadEvent` fires with the new config

#### Scenario: Invalid config on reload

- GIVEN a running chart with valid config
- WHEN the config file is replaced with invalid TOML
- THEN the previous config SHALL remain active and an error SHALL be logged
