use fc_types::bar::Bar;
use fc_types::tick::Tick;
use std::error::Error;
use std::sync::mpsc::Receiver;

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
    fn start(&mut self) -> Result<(), Box<dyn Error>>;
    fn receiver(&self) -> Option<&Receiver<DataEvent>>;
    fn stop(&mut self) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &str;
}
