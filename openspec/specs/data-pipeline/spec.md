# Data Pipeline Specification

## Purpose

Define the data ingestion and aggregation pipeline. Data flows from provider through a multi-resolution cascade of ring buffers, pushed via async channels to the render loop.

## Requirements

### Requirement: DataProvider port trait

A `DataProvider` trait MUST define an async interface for bar and tick data retrieval. It SHALL provide `fn subscribe(&self) -> mpsc::Receiver<DataEvent>` for streaming data.

#### Scenario: Provider subscription returns receiver

- GIVEN a `DataProvider` implementation
- WHEN `subscribe` is called
- THEN it returns a valid `tokio::mpsc::Receiver<DataEvent>`

#### Scenario: Provider delivers data events

- GIVEN an active subscription
- WHEN new bar data is available
- THEN a `DataEvent::Bar` is sent through the channel

### Requirement: TimeSeries ring buffer

`TimeSeries<T, const N: usize>` MUST be backed by `MaybeUninit` to avoid zero-initialization overhead. Push MUST be O(1). The buffer SHALL expose indexed and iterator access.

#### Scenario: Push into uninitialized buffer

- GIVEN a `TimeSeries<Bar, 1000>` with `MaybeUninit`
- WHEN the first Bar is pushed
- THEN its memory SHALL be initialized at index 0 without initializing the remaining 999 slots

#### Scenario: Iterator yields in order

- GIVEN 10 pushed Bars with sequential timestamps
- WHEN iterating
- THEN items SHALL be yielded oldest-first in insertion order

### Requirement: Multi-resolution aggregation cascade

The pipeline SHALL cascade: `TickAggregator → M1Aggregator → M5Aggregator → M15Aggregator → H1Aggregator → D1Aggregator`. Each aggregator MUST produce OHLC bars from its source resolution.

#### Scenario: Tick bars aggregate to M1

- GIVEN 60 seconds of tick data at irregular intervals
- WHEN the M1Aggregator processes them
- THEN exactly one OHLC bar is produced with correct open/high/low/close

#### Scenario: Cascade feeds correctly

- GIVEN tick data flowing into TickAggregator
- WHEN M1 bars are produced and fed into M5Aggregator
- THEN after 5 minutes, the M5Aggregator produces one valid OHLC bar

### Requirement: Retention limits per tier

Each aggregation tier SHALL have a configurable retention limit (max bars kept). When exceeded, the oldest bars SHALL be evicted.

#### Scenario: Eviction on overflow

- GIVEN M1 tier with retention limit of 1440 bars (24h)
- WHEN bar 1441 is pushed
- THEN bar at index 0 (oldest) is evicted and the newest bar is stored

#### Scenario: Configurable limits

- GIVEN default retention limits
- WHEN configuring the pipeline with custom limits
- THEN each tier SHALL enforce its new limit independently

### Requirement: tokio mpsc channel push model

Data SHALL flow from `DataProvider` to the render loop via `tokio::mpsc` channels. The render loop SHALL receive data on the sync side.

#### Scenario: Channel delivers bars to render loop

- GIVEN a simulated provider pushing 100 bars/second
- WHEN the render loop polls the receiver
- THEN each bar is delivered without loss (within channel capacity)

#### Scenario: Backpressure on overflow

- GIVEN channel capacity is 1024
- WHEN the provider produces faster than the render loop consumes
- THEN the channel SHALL apply backpressure via `send().await`

### Requirement: rkyv zero-copy persistence

Hot ring buffers SHALL be persistable via rkyv zero-copy archive. The archive SHALL use the rkyv `Archive` derive on domain types.

#### Scenario: Round-trip archive

- GIVEN a `TimeSeries<Bar, 1000>` with 50 bars
- WHEN serialized to bytes via rkyv and deserialized
- THEN all 50 bars are recovered with identical values

#### Scenario: Zero-copy access

- GIVEN an archived `TimeSeries<Bar, 1000>` in memory-mapped storage
- WHEN accessed via `rkyv::archived_value`
- THEN bars are readable without deserialization overhead
