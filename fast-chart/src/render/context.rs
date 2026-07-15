// ---------------------------------------------------------------------------
// RenderContext — context passed to all renderers during a frame
// ---------------------------------------------------------------------------

use super::coordinates::CoordinatePipeline;
use super::series_renderer::Rect;

/// The context passed to all series renderers during a frame.
///
/// Contains everything a renderer needs: coordinate pipeline, clip rect,
/// pane bounds, time/value ranges, DPI scale, and timestamp.
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// The coordinate pipeline for world↔screen transforms.
    pub pipeline: CoordinatePipeline,
    /// The clipping rectangle (pane bounds in screen space).
    pub clip_rect: Rect,
    /// The full pane bounds (may be larger than clip_rect for padding).
    pub pane_bounds: Rect,
    /// Visible time range in world units.
    pub time_range: (f64, f64),
    /// Visible price range in world units.
    pub price_range: (f64, f64),
    /// DPI scale factor.
    pub scale_factor: f32,
    /// Frame timestamp (monotonic, for animations).
    pub timestamp: u64,
}

impl RenderContext {
    /// Create a new render context.
    pub fn new(
        pipeline: CoordinatePipeline,
        clip_rect: Rect,
        pane_bounds: Rect,
        time_range: (f64, f64),
        price_range: (f64, f64),
        scale_factor: f32,
        timestamp: u64,
    ) -> Self {
        Self {
            pipeline,
            clip_rect,
            pane_bounds,
            time_range,
            price_range,
            scale_factor,
            timestamp,
        }
    }

    /// Create a render context from a pipeline and pane bounds.
    ///
    /// Uses the pipeline's time/price ranges and scale factor.
    pub fn from_pipeline(pipeline: CoordinatePipeline, pane_bounds: Rect, timestamp: u64) -> Self {
        let clip_rect = pane_bounds;
        let time_range = pipeline.time_range;
        let price_range = pipeline.price_range;
        let scale_factor = pipeline.scale_factor;
        Self::new(
            pipeline,
            clip_rect,
            pane_bounds,
            time_range,
            price_range,
            scale_factor,
            timestamp,
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::coordinates::{CoordinatePipeline, ScreenPoint, WorldPoint};

    fn test_pipeline() -> CoordinatePipeline {
        CoordinatePipeline::new(
            (0.0, 1000.0),
            (50.0, 150.0),
            0.0,
            0.0,
            800.0,
            600.0,
            1.0,
        )
    }

    #[test]
    fn render_context_new() {
        let pipeline = test_pipeline();
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let ctx = RenderContext::new(
            pipeline.clone(),
            bounds,
            bounds,
            (0.0, 1000.0),
            (50.0, 150.0),
            1.0,
            12345,
        );

        assert_eq!(ctx.time_range, (0.0, 1000.0));
        assert_eq!(ctx.price_range, (50.0, 150.0));
        assert_eq!(ctx.scale_factor, 1.0);
        assert_eq!(ctx.timestamp, 12345);
    }

    #[test]
    fn render_context_from_pipeline() {
        let pipeline = test_pipeline();
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let ctx = RenderContext::from_pipeline(pipeline, bounds, 99999);

        assert_eq!(ctx.time_range, (0.0, 1000.0));
        assert_eq!(ctx.price_range, (50.0, 150.0));
        assert_eq!(ctx.scale_factor, 1.0);
        assert_eq!(ctx.timestamp, 99999);
        assert_eq!(ctx.clip_rect, ctx.pane_bounds);
    }

    #[test]
    fn render_context_clone() {
        let pipeline = test_pipeline();
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        let ctx = RenderContext::from_pipeline(pipeline, bounds, 42);
        let cloned = ctx.clone();
        assert_eq!(cloned.timestamp, 42);
        assert_eq!(cloned.time_range, ctx.time_range);
    }
}
