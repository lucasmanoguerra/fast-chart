use std::fmt;

/// Errors that can occur during rendering.
#[derive(Debug)]
pub enum RenderError {
    /// The GPU surface was lost (e.g. window minimized, device lost).
    SurfaceLost,
    /// GPU ran out of memory.
    OutOfMemory,
    /// Any other rendering failure.
    Other(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::SurfaceLost => write!(f, "Surface lost"),
            RenderError::OutOfMemory => write!(f, "Out of memory"),
            RenderError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Snapshot of chart state needed by the renderer for one frame.
///
/// The application layer fills this struct from [`ChartState`](crate::ChartState).
/// The renderer interprets it without knowing about domain types.
pub struct FrameState {
    /// Surface width in physical pixels.
    pub width: u32,
    /// Surface height in physical pixels.
    pub height: u32,
    /// Monotonic timestamp for animation interpolation.
    pub timestamp_ns: u64,
}

/// Port for rendering the chart to a GPU or software surface.
///
/// Implement this trait to connect a concrete rendering backend (e.g. wgpu)
/// to the application layer without coupling core to any graphics API.
pub trait ChartRenderer: Send {
    /// Notify the renderer of a surface resize.
    fn resize(&mut self, width: u32, height: u32);

    /// Draw a complete frame. The renderer receives the frame state
    /// and is responsible for producing the visual output.
    fn draw_frame(&mut self, state: &FrameState) -> Result<(), RenderError>;

    /// Present the rendered frame to the surface.
    fn present(&mut self) -> Result<(), RenderError>;
}
