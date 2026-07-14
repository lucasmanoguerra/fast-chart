use crate::app::chart_controller::ChartState;
use std::error::Error;

/// Port for rendering the chart to a GPU or software surface.
///
/// Implement this trait to connect a concrete rendering backend (e.g. wgpu)
/// to the application layer without coupling core to any graphics API.
pub trait ChartRenderer: Send {
    fn render(&mut self, state: &ChartState) -> Result<(), Box<dyn Error>>;
    fn resize(&mut self, width: u32, height: u32);
}
