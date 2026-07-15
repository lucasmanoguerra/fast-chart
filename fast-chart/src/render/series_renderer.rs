// ---------------------------------------------------------------------------
// SeriesRenderer — trait for rendering series data into draw commands
// ---------------------------------------------------------------------------

use super::commands::DrawCommand;

/// A hit-test result for a point on a series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesHit {
    /// The index of the nearest data point.
    pub index: usize,
    /// The distance from the hit point to the nearest data point.
    pub distance: f32,
}

/// Axis-aligned bounding rectangle in screen space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point is inside this rectangle.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x
            && px < self.x + self.width
            && py >= self.y
            && py < self.y + self.height
    }

    /// The right edge (x + width).
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// The bottom edge (y + height).
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// The center point.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
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

    // ---- Rect ----

    #[test]
    fn rect_new() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 50.0);
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(50.0, 50.0));
        assert!(r.contains(0.0, 0.0)); // top-left edge
        assert!(!r.contains(100.0, 100.0)); // bottom-right is outside
    }

    #[test]
    fn rect_right() {
        let r = Rect::new(10.0, 0.0, 90.0, 0.0);
        assert_eq!(r.right(), 100.0);
    }

    #[test]
    fn rect_bottom() {
        let r = Rect::new(0.0, 10.0, 0.0, 90.0);
        assert_eq!(r.bottom(), 100.0);
    }

    #[test]
    fn rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 60.0);
        assert_eq!(r.center(), (50.0, 30.0));
    }

    // ---- SeriesHit ----

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
