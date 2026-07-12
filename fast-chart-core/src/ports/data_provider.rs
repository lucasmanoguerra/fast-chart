use fast_chart_domain::bar::Bar;
use fast_chart_domain::tick::Tick;
use std::error::Error;
use std::sync::mpsc::Receiver;

pub enum DataEvent {
    BarClosed(Bar),
    TickUpdate(Tick),
    SymbolChanged(String),
    TimeframeChanged(String),
}

pub trait DataProvider: Send {
    fn start(&mut self) -> Result<(), Box<dyn Error>>;
    fn receiver(&self) -> Option<&Receiver<DataEvent>>;
    fn stop(&mut self) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &str;
}
