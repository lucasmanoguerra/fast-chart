// ---------------------------------------------------------------------------
// SeriesRenderer — trait for rendering series data into draw commands
// ---------------------------------------------------------------------------

use super::commands::DrawCommand;

// Re-export canonical Rect from fc-primitives
pub use fc_primitives::Rect;

/// A hit-test result for a point on a series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesHit {
    /// The index of the nearest data point.
    pub index: usize,
    /// The distance from the hit point to the nearest data point.
    pub distance: f32,
}

/// A renderer for a specific series type (candlestick, area, line, etc.).
///
/// Each series renderer takes data and a viewport, then produces
/// `DrawCommand`s that any backend can execute. This is the core
/// abstraction that makes the library renderer-agnostic.
///
/// # Object Safety
///
/// This trait is object-safe: you can use `Box<dyn SeriesRenderer>`.
pub trait SeriesRenderer: Send + Sync {
    /// Produce draw commands for the visible portion of the series.
    ///
    /// `bounds` defines the pixel area this series should render into.
    /// `visible_range` is the range of data indices currently visible.
    fn update(
        &mut self,
        data: &[super::commands::DrawCommand], // placeholder — will be Bar/series-specific
        bounds: Rect,
    ) -> Vec<DrawCommand>;

    /// Test if a screen-space point hits a data element in this series.
    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit>;

    /// The bounding rectangle of this series in screen space.
    fn bounds(&self) -> Rect;

    /// The z-index layer this series renders into.
    fn layer_z_index(&self) -> i32 {
        600 // default: Candles layer
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;


    // Clasificación: determinística — verifica series_hit_equality
    #[test]
    fn series_hit_equality() {
        let h1 = SeriesHit {
            index: 5,
            distance: 2.0,
        };
        let h2 = SeriesHit {
            index: 5,
            distance: 2.0,
        };
        assert_eq!(h1, h2);
    }

    // Clasificación: determinística — verifica series_hit_different_index
    #[test]
    fn series_hit_different_index() {
        let h1 = SeriesHit {
            index: 5,
            distance: 2.0,
        };
        let h2 = SeriesHit {
            index: 6,
            distance: 2.0,
        };
        assert_ne!(h1, h2);
    }
}
