# Domain Model Specification

## Purpose

Define the core domain types for a trading chart application. These types encode all domain concepts with zero external dependencies, forming the innermost layer of the hexagonal architecture.

## Requirements

### Requirement: Bar

A `Bar` MUST represent one OHLCV interval: open, high, low, close prices and volume. A `Bar` SHALL carry a timestamp of the interval open. All price fields MUST be `f64`. Volume MUST be `u64`.

#### Scenario: Bar captures OHLCV correctly

- GIVEN open=100.0, high=105.0, low=99.5, close=102.0, volume=15000
- WHEN a `Bar` is constructed
- THEN all five fields are accessible and match the input values

#### Scenario: Bar rejects invalid OHLC

- GIVEN a `Bar` with high < low
- WHEN the `Bar` is constructed
- THEN construction SHOULD return a `Result::Err` with a `ChartError`

### Requirement: Tick

A `Tick` MUST represent exchange-level data: bid price, ask price, last price, and volume. All price fields MUST be `f64`. Volume MUST be `u64`.

#### Scenario: Tick captures bid/ask spread

- GIVEN bid=100.0, ask=100.05, last=100.02, volume=500
- WHEN a `Tick` is constructed
- THEN all four fields are accessible and match input values

#### Scenario: Tick with zero volume

- GIVEN a `Tick` with volume=0
- WHEN the `Tick` is constructed
- THEN construction SHALL succeed

### Requirement: TimeSeries\<T\>

`TimeSeries<T>` MUST be an ordered collection of type `T` indexed by time. It SHALL support O(1) push via a ring buffer. The ring buffer capacity SHALL be a compile-time const generic parameter `N`.

#### Scenario: Push maintains order

- GIVEN an empty `TimeSeries<Bar, 100>` ring buffer
- WHEN three Bars with sequential timestamps are pushed
- THEN iteration returns them in insertion order, oldest first

#### Scenario: Ring buffer overwrites oldest

- GIVEN a `TimeSeries<Bar, 3>` filled to capacity with three Bars
- WHEN a fourth Bar is pushed
- THEN the oldest Bar is dropped and the newest Bar is accessible

### Requirement: SeriesType

A `SeriesType` enum MUST distinguish series display style. Variants: `Candle`, `Bar`, `Line`, `Area`, `Baseline`.

#### Scenario: Enum contains all five variants

- GIVEN the `SeriesType` enum
- WHEN iterating its variants
- THEN exactly five variants are present: Candle, Bar, Line, Area, Baseline

#### Scenario: Default series type

- GIVEN a chart initialization without explicit series type
- THEN the default SHALL be `Candle`

### Requirement: Viewport

A `Viewport` MUST define the visible region: start and end times, a value range (min/max price), and a zoom level.

#### Scenario: Viewport constrains visible range

- GIVEN a `Viewport` with start=9:30, end=16:00, price_min=100, price_max=110
- WHEN a time outside 9:30–16:00 is queried
- THEN the Viewport SHALL report it as out of range

#### Scenario: Zoom updates width

- GIVEN a `Viewport` at zoom level 1.0 covering 6.5 hours
- WHEN zoom level changes to 2.0
- THEN the visible time range SHALL halve

### Requirement: Scale

A `Scale` MUST map domain values to screen coordinates. `LinearScale` SHALL map price to y-pixel. `TimeScale` SHALL map epoch to x-pixel.

#### Scenario: LinearScale maps price to y

- GIVEN price_min=100, price_max=110, canvas_height=500
- WHEN price=105 is mapped
- THEN y SHALL be 250 (midpoint)

#### Scenario: TimeScale maps timestamp to x

- GIVEN time range 9:30–16:00 (23400s), canvas_width=800
- WHEN 12:45 (11700s from start) is mapped
- THEN x SHALL be 400 (midpoint)

### Requirement: Crosshair

A `Crosshair` MUST capture cursor position: x-coordinate (time), y-coordinate (price), and the values of all visible series at that point.

#### Scenario: Crosshair tracks cursor

- GIVEN mouse at pixel (400, 250) and active scales
- WHEN the crosshair is updated
- THEN it SHALL report the corresponding time and price values

#### Scenario: Crosshair outside viewport

- GIVEN mouse position beyond the viewport bounds
- WHEN the crosshair is updated
- THEN crosshair SHALL remain in its last known valid state

### Requirement: Indicator trait

An `Indicator` trait MUST provide `fn calculate(&self, series: &TimeSeries<Bar>) -> TimeSeries<f64>` and `fn name(&self) -> &str`.

#### Scenario: Indicator produces output series

- GIVEN an SMA indicator and a `TimeSeries<Bar>` of 50 bars
- WHEN `calculate` is called
- THEN a `TimeSeries<f64>` of SMA values is returned

#### Scenario: Indicator with insufficient data

- GIVEN an SMA(period=20) indicator and a series with 5 bars
- WHEN `calculate` is called
- THEN the output TimeSeries SHALL be empty or contain only `f64::NAN` values

### Requirement: ChartError

`ChartError` MUST be a typed error enum covering domain-layer failures. Variants SHALL include at least: `InvalidPriceData`, `InsufficientData`, and `OutOfRange`.

#### Scenario: Error variants are typed

- GIVEN a `ChartError::InvalidPriceData` with a descriptive message
- WHEN the error is formatted via `Display`
- THEN the formatted string includes the reason

#### Scenario: Error implements standard traits

- GIVEN any `ChartError` variant
- THEN it SHALL implement `std::error::Error`, `Debug`, and `Display`
