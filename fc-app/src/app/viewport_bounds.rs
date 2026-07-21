/// Zoom/pan boundary constraints for the viewport.
///
/// Holds the min/max zoom level and min/max visible bar count. Extracted
/// from [`ViewportManager`](super::viewport_management::ViewportManager)
/// to respect the Single Responsibility Principle — the manager focuses
/// on orchestration while bounds handle clamping logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ViewportBounds {
    pub(crate) min_zoom: f64,
    pub(crate) max_zoom: f64,
    pub(crate) min_visible_bars: usize,
    pub(crate) max_visible_bars: usize,
}

impl Default for ViewportBounds {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewportBounds {
    /// Create bounds with sensible defaults for financial charts.
    pub(crate) fn new() -> Self {
        Self {
            min_zoom: 0.01,
            max_zoom: 1000.0,
            min_visible_bars: 5,
            max_visible_bars: 10_000,
        }
    }

    /// Clamp a zoom value by applying `factor` to `current` and bounding
    /// the result within `[min_zoom, max_zoom]`.
    pub(crate) fn clamp_zoom(&self, current: f64, factor: f64) -> f64 {
        (current * factor).clamp(self.min_zoom, self.max_zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bounds() {
        let b = ViewportBounds::new();
        assert_eq!(b.min_zoom, 0.01);
        assert_eq!(b.max_zoom, 1000.0);
        assert_eq!(b.min_visible_bars, 5);
        assert_eq!(b.max_visible_bars, 10_000);
    }

    #[test]
    fn clamp_zoom_identity_factor() {
        let b = ViewportBounds::new();
        assert_eq!(b.clamp_zoom(1.0, 1.0), 1.0);
    }

    #[test]
    fn clamp_zoom_clamps_to_max() {
        let b = ViewportBounds::new();
        let result = b.clamp_zoom(500.0, 10.0);
        assert_eq!(result, 1000.0);
    }

    #[test]
    fn clamp_zoom_clamps_to_min() {
        let b = ViewportBounds::new();
        let result = b.clamp_zoom(0.02, 0.01);
        assert_eq!(result, 0.01);
    }

    #[test]
    fn clamp_zoom_custom_bounds() {
        let b = ViewportBounds {
            min_zoom: 1.0,
            max_zoom: 10.0,
            min_visible_bars: 10,
            max_visible_bars: 500,
        };
        // Should clamp to min
        assert_eq!(b.clamp_zoom(1.0, 0.5), 1.0);
        // Should clamp to max
        assert_eq!(b.clamp_zoom(8.0, 2.0), 10.0);
        // Within bounds
        assert_eq!(b.clamp_zoom(5.0, 1.0), 5.0);
    }

    #[test]
    fn clamp_zoom_within_bounds() {
        let b = ViewportBounds::new();
        let result = b.clamp_zoom(1.0, 2.0);
        assert_eq!(result, 2.0);
    }
}
