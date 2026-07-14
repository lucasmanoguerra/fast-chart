# Indicator Engine Specification

## Purpose

Define the SIMD-accelerated indicator calculation framework. Provides an extensible `Indicator` trait and a suite of built-in technical indicators.

## Requirements

### Requirement: Indicator trait

An `Indicator` trait MUST define `fn calculate(&self, series: &TimeSeries<Bar>) -> TimeSeries<f64>` and `fn name(&self) -> &str`. The trait SHALL be object-safe.

#### Scenario: Indicator produces named output

- GIVEN an indicator with `name()` returning "SMA(14)"
- WHEN `calculate` is called on a series
- THEN the output `TimeSeries<f64>` SHALL be associated with that name

#### Scenario: Object-safe dispatch

- GIVEN a `Vec<Box<dyn Indicator>>`
- WHEN each indicator's `calculate` is called in a loop
- THEN all indicators produce output without type erasure errors

### Requirement: SIMD acceleration

Rolling window calculations SHOULD use `core::simd` when the target supports it. SIMD SHALL be enabled for f64 operations on x86-64 and ARM NEON.

#### Scenario: SIMD SMA matches reference

- GIVEN an SMA(14) implementation using `core::simd`
- WHEN calculated on 1000 bars
- THEN each output value SHALL match a scalar reference implementation within f64 precision tolerance

#### Scenario: SIMD fallback on unsupported arch

- GIVEN a target without SIMD f64 support
- WHEN the indicator is invoked
- THEN execution falls back to scalar implementation without error

### Requirement: SMA(period)

A SIMD simple moving average MUST accept a configurable period. It SHALL compute the arithmetic mean of the last N closing prices.

#### Scenario: SMA on 200 bars, period 20

- GIVEN 200 bars with known closing prices
- WHEN SMA(20) is calculated
- THEN bars 0–18 are empty/NAN and bars 19–199 contain the correct 20-bar averages

#### Scenario: Period equals series length

- GIVEN 50 bars and SMA(50)
- WHEN calculated
- THEN exactly one valid output value exists at the last position

### Requirement: EMA(period)

A SIMD exponential moving average MUST accept a configurable period. The smoothing factor SHALL be `2 / (period + 1)`.

#### Scenario: EMA converges with SMA seed

- GIVEN 100 bars of random walk data
- WHEN EMA(20) is calculated
- THEN the output length matches the input; first NAN positions equal (period - 1)

#### Scenario: EMA of constant value

- GIVEN 50 bars where close=100.0 for all
- WHEN EMA(10) is calculated
- THEN all output values after the warm-up period approach 100.0

### Requirement: RSI(period)

A SIMD relative strength index MUST accept a configurable period. Output SHALL be bounded between 0.0 and 100.0.

#### Scenario: RSI of uptrend

- GIVEN 100 bars where close increases monotonically
- WHEN RSI(14) is calculated
- THEN the final value SHALL be 100.0 (overbought)

#### Scenario: RSI range validity

- GIVEN any input series of 200 bars
- WHEN RSI(14) is calculated
- THEN every output value SHALL be in range [0.0, 100.0]

### Requirement: MACD(fast, slow, signal)

MACD MUST compute: MACD line = EMA(fast) − EMA(slow), Signal = EMA(signal) of MACD line, Histogram = MACD − Signal.

#### Scenario: MACD components match definition

- GIVEN 200 bars with varying prices
- WHEN MACD(12, 26, 9) is calculated
- THEN MACD line, signal, and histogram are all returned with correct vector lengths

#### Scenario: MACD on flat series

- GIVEN 100 bars where all closes equal
- WHEN MACD(12, 26, 9) is calculated
- THEN all three output components SHALL be zero after warm-up

### Requirement: Bollinger Bands(period, stddev)

Bollinger Bands MUST compute: middle = SMA(period), upper = middle + k × stddev, lower = middle − k × stddev.

#### Scenario: Bands widen during volatility

- GIVEN 50 bars with low volatility followed by 50 bars with high volatility
- WHEN Bollinger(20, 2) is calculated
- THEN the band width during the high-volatility period is larger

#### Scenario: Price touches bands

- GIVEN a price series where the last close equals middle + 2×stddev
- WHEN Bollinger(20, 2) is calculated
- THEN upper is exactly equal to that close

### Requirement: Stochastic(k_period, d_period)

Stochastic oscillator MUST compute %K and %D lines. %K = ((close − lowest) / (highest − lowest)) × 100.

#### Scenario: Stochastic of new high

- GIVEN prices making a new 14-bar high on the last bar
- WHEN Stochastic(14, 3) is calculated
- THEN %K SHALL be 100.0

#### Scenario: %D is smoothed %K

- GIVEN a Stochastic(14, 3) calculation
- THEN %D(3) SHALL be the SMA(3) of the %K line

### Requirement: Ichimoku(tenkan, kijun, senkou)

Ichimoku Cloud MUST compute: Tenkan-sen, Kijun-sen, Senkou Span A, Senkou Span B, Chikou Span.

#### Scenario: All five lines produced

- GIVEN 200 bars
- WHEN Ichimoku(9, 26, 52) is calculated
- THEN all five cloud lines are returned with correct length

#### Scenario: Senkou A = (Tenkan + Kijun) / 2, shifted

- GIVEN Ichimoku(9, 26, 52) calculation
- THEN Senkou Span A at any point equals (Tenkan + Kijun)/2 of 26 periods prior

### Requirement: Thread-safe IndicatorRegistry

An `IndicatorRegistry` MUST be a thread-safe map of named indicators to pane assignments. It SHALL support concurrent reads and writes via `RwLock` or similar.

#### Scenario: Register and retrieve indicator

- GIVEN an empty registry
- WHEN an SMA(14) is inserted under name "sma14"
- THEN `get("sma14")` returns `Some(&dyn Indicator)`

#### Scenario: Concurrent access

- GIVEN a registry shared across threads
- WHEN one thread writes a new indicator and another reads simultaneously
- THEN both operations succeed without data races

### Requirement: Scalar fallback

When `core::simd` is unavailable, every indicator SHALL have a scalar implementation that produces bit-identical results.

#### Scenario: SIMD and scalar match exactly

- GIVEN identical input data
- WHEN SMA(14) is computed with both SIMD and scalar paths
- THEN all output values match within 1e-12
