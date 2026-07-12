use crate::app::chart_controller::ChartState;
use std::error::Error;

pub trait ChartRenderer: Send {
    fn render(&mut self, state: &ChartState) -> Result<(), Box<dyn Error>>;
    fn resize(&mut self, width: u32, height: u32);
}
