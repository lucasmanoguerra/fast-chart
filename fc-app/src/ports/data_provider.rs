use fc_primitives::bar::Bar;
use fc_primitives::tick::Tick;
use std::fmt;
use std::sync::mpsc::Receiver;

/// Errors that can occur during data provider operations.
#[derive(Debug)]
pub enum DataError {
    /// Failed to establish a connection to the data source.
    ConnectionFailed(String),
    /// Failed to subscribe to a symbol or timeframe.
    SubscriptionFailed(String),
    /// The data source disconnected unexpectedly.
    Disconnected,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            DataError::SubscriptionFailed(msg) => write!(f, "Subscription failed: {msg}"),
            DataError::Disconnected => write!(f, "Disconnected"),
        }
    }
}

impl std::error::Error for DataError {}

/// An event emitted by a [`DataProvider`] when new market data arrives.
#[derive(Debug)]
pub enum DataEvent {
    BarClosed(Bar),
    TickUpdate(Tick),
    SymbolChanged(String),
    TimeframeChanged(String),
}

/// Port for market data sources.
///
/// Implement this trait to feed bars, ticks, or symbol/timeframe changes
/// into the [`ChartController`](crate::ChartController). Communication
/// happens through an [`mpsc::Receiver`] to decouple the data source
/// from the render loop.
pub trait DataProvider: Send {
    fn start(&mut self) -> Result<(), DataError>;
    fn receiver(&self) -> Option<&Receiver<DataEvent>>;
    fn stop(&mut self) -> Result<(), DataError>;
    fn name(&self) -> &str;
}
