#[derive(Debug)]
pub enum InteractionCommand {
    ZoomAtCursor { factor: f64, screen_x: f64 },
    PanBy { time_delta: i64 },
    UpdateCrosshair { screen_x: f64, screen_y: f64 },
    DeactivateCrosshair,
    SwitchTimeframe { timeframe: String },
    ResizePane { pane_index: usize, new_height: f64 },
}

/// Commands that control the viewport state of the chart.
#[derive(Debug)]
pub enum ViewportCommand {
    SetTimeRange { start: u64, end: u64 },
    SetValueRange { min: f64, max: f64 },
    SetCrosshairPosition { x: f64, y: f64, time: u64, price: f64 },
    DeactivateCrosshair,
    ZoomAtCursor { factor: f64, screen_x: f64 },
    PanBy { time_delta: i64 },
    RequestRedraw,
}

pub trait InteractionHandler: Send {
    fn handle_event(&self, command: InteractionCommand) -> Vec<ViewportCommand>;
}
