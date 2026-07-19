// ---------------------------------------------------------------------------
// IndicatorRenderer — trait for rendering indicators with draw commands
// ---------------------------------------------------------------------------

use crate::commands::DrawCommand;
use crate::series_renderer::{Rect, SeriesHit, SeriesRenderer};

/// A trait for rendering indicators, extending SeriesRenderer with
/// overlay and separate-pane rendering modes.
///
/// Indicators can render themselves in two modes:
/// 1. **Overlay**: rendered on top of an existing pane (e.g., SMA on price chart)
/// 2. **Separate pane**: rendered in their own dedicated pane (e.g., RSI, MACD)
pub trait IndicatorRenderer: SeriesRenderer + Send + Sync {
    /// Render this indicator as an overlay on the given pane.
    ///
    /// The `pane_bounds` defines the pixel area of the pane this indicator
    /// overlays. Commands are clipped to this area.
    fn render_overlay(&self, pane_bounds: Rect) -> Vec<DrawCommand>;

    /// Render this indicator in a separate pane.
    ///
    /// The `pane_bounds` defines the full pixel area allocated to this
    /// indicator's dedicated pane.
    fn render_separate(&self, pane_bounds: Rect) -> Vec<DrawCommand>;

    /// The z-index layer for this indicator's rendering.
    /// Overlays typically use a higher z-index than the base series.
    fn indicator_z_index(&self) -> i32 {
        700 // above candle layer (600)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple test indicator for verifying the trait.
    struct TestIndicator {
        color: [f32; 4],
    }

    impl SeriesRenderer for TestIndicator {
        fn update(&mut self, _data: &[DrawCommand], bounds: Rect) -> Vec<DrawCommand> {
            vec![DrawCommand::DrawRect {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                fill: Some(self.color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 700,
            }]
        }

        fn hit_test(&self, _x: f32, _y: f32) -> Option<SeriesHit> {
            None
        }

        fn bounds(&self) -> Rect {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        }
    }

    impl IndicatorRenderer for TestIndicator {
        fn render_overlay(&self, pane_bounds: Rect) -> Vec<DrawCommand> {
            vec![DrawCommand::DrawRect {
                x: pane_bounds.x,
                y: pane_bounds.y,
                width: pane_bounds.width,
                height: pane_bounds.height,
                fill: Some(self.color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 700,
            }]
        }

        fn render_separate(&self, pane_bounds: Rect) -> Vec<DrawCommand> {
            vec![DrawCommand::DrawRect {
                x: pane_bounds.x,
                y: pane_bounds.y,
                width: pane_bounds.width,
                height: pane_bounds.height * 0.5,
                fill: Some(self.color),
                stroke: None,
                stroke_width: 0.0,
                z_index: 700,
            }]
        }
    }

    #[test]
    fn indicator_renderer_overlay() {
        let ind = TestIndicator { color: [0.0, 1.0, 0.0, 1.0] };
        let bounds = Rect::new(0.0, 0.0, 800.0, 400.0);
        let cmds = ind.render_overlay(bounds);
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn indicator_renderer_separate() {
        let ind = TestIndicator { color: [1.0, 0.0, 0.0, 1.0] };
        let bounds = Rect::new(0.0, 400.0, 800.0, 200.0);
        let cmds = ind.render_separate(bounds);
        assert_eq!(cmds.len(), 1);

        // Separate pane should use half height
        if let DrawCommand::DrawRect { height, .. } = &cmds[0] {
            assert!((*height - 100.0).abs() < f32::EPSILON);
        } else {
            panic!("expected DrawRect");
        }
    }

    #[test]
    fn indicator_renderer_z_index() {
        let ind = TestIndicator { color: [1.0, 1.0, 1.0, 1.0] };
        assert_eq!(ind.indicator_z_index(), 700);
    }

    #[test]
    fn trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestIndicator>();
    }
}
