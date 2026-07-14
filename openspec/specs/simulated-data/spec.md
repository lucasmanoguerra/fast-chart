# Simulated Data Specification

## Purpose

Define a synthetic market data provider for development and testing. Generates realistic OHLC bars and ticks with configurable parameters and reproducible randomness.

## Requirements

### Requirement: SimulatedDataProvider implementation

`SimulatedDataProvider` MUST implement the `DataProvider` trait. It SHALL generate synthetic data on demand and push it through the async channel.

#### Scenario: Provider implements DataProvider

- GIVEN a `SimulatedDataProvider`
- WHEN its type is checked against the `DataProvider` trait
- THEN it SHALL satisfy the trait bounds

#### Scenario: Provider pushes through channel

- GIVEN a `SimulatedDataProvider` configured for streaming
- WHEN `start_stream` is called
- THEN `DataEvent::Bar` messages appear on the receiver channel

### Requirement: OHLC bar generation

The provider MUST generate OHLC bars from a drift-diffusion model. Each bar SHALL have: open = previous close, close = open + drift + noise, high = max(open, close) + random wick, low = min(open, close) − random wick.

#### Scenario: Bar prices are consistent

- GIVEN a generated bar
- THEN high >= max(open, close) AND low <= min(open, close) AND open >= low AND open <= high AND close >= low AND close <= high

#### Scenario: Consecutive bars link correctly

- GIVEN bar N with close 100.0
- WHEN bar N+1 is generated
- THEN bar N+1 open == 100.0

### Requirement: Tick-level data generation

The provider SHOULD generate tick-level data with bid/ask spread and random order arrivals. Ticks SHALL follow a Poisson arrival process.

#### Scenario: Tick spread is positive

- GIVEN any generated tick
- THEN ask > bid

#### Scenario: Tick arrival interval distribution

- GIVEN 1000 generated ticks with lambda=10 per second
- WHEN inter-arrival times are measured
- THEN they follow an exponential distribution with mean ≈ 100ms

### Requirement: Configurable market parameters

The provider SHALL accept configuration for: initial price, drift (mean return), volatility (standard deviation), session hours, tick volume, and gap frequency.

#### Scenario: Parameters affect output

- GIVEN volatility=0.01 and volatility=0.10
- WHEN 1000 bars are generated for each
- THEN the high-volatility series has wider OHLC ranges

#### Scenario: Session hours respected

- GIVEN session hours 9:30–16:00
- WHEN bars are generated
- THEN no bar has a timestamp outside 9:30–16:00

#### Scenario: Gap generation

- GIVEN gap_frequency=0.1 (10% of intervals)
- WHEN examining generated bars
- THEN approximately 10% of consecutive bars have open ≠ previous close, with a gap

### Requirement: Seeded RNG

The provider MUST accept a seed for reproducible generation. The same seed SHALL produce identical data sequences.

#### Scenario: Reproducible output

- GIVEN seed=42 for two provider instances
- WHEN both generate 100 bars
- THEN every bar's OHLCV matches exactly between instances

#### Scenario: Different seeds differ

- GIVEN seed=42 and seed=99
- WHEN both generate 100 bars
- THEN the resulting sequences are statistically distinct

### Requirement: Burst and streaming modes

The provider SHALL support burst mode (generate N bars immediately) and streaming mode (push bars on a configurable time interval).

#### Scenario: Burst mode returns N bars

- GIVEN burst mode configured for 500 bars
- WHEN `generate_burst(500)` is called
- THEN 500 `DataEvent::Bar` messages are available immediately on the channel

#### Scenario: Streaming mode at interval

- GIVEN streaming mode with interval=1000ms
- WHEN the stream starts
- THEN one `DataEvent::Bar` arrives approximately every 1000ms
