// ---------------------------------------------------------------------------
// PaneDivider — visual representation of a separator between panes
// ---------------------------------------------------------------------------

use fc_primitives::Rect;

/// A visual separator between two adjacent panes.
///
/// This is a pure rendering concern — it knows nothing about drag state,
/// cursor management, or position constraints. All interaction logic
/// lives in `LayoutManager`.
#[derive(Debug, Clone)]
pub struct PaneDivider {
    /// Index of the pane above this divider.
    pub upper_pane_index: usize,
    /// Normalized y-position of the divider (0.0–1.0).
    pub position: f64,
    /// Height of the divider in pixels (visual thickness).
    pub height: f32,
    /// Height of the hit zone in pixels (larger than visual for easier grabbing).
    pub hit_zone_height: f32,
}

impl PaneDivider {
    /// Create a new divider between two panes.
    pub fn new(upper_pane_index: usize, position: f64) -> Self {
        Self {
            upper_pane_index,
            position,
            height: 4.0,
            hit_zone_height: 12.0,
        }
    }

    /// Create a divider with custom dimensions.
    pub fn with_dimensions(mut self, height: f32, hit_zone: f32) -> Self {
        self.height = height;
        self.hit_zone_height = hit_zone;
        self
    }

    /// Test if a screen-space y-coordinate hits this divider.
    ///
    /// Pure computation — no state mutation.
    pub fn hit_test(&self, screen_y: f32, canvas_height: f32) -> bool {
        let divider_pixel_y = self.position as f32 * canvas_height;
        let half_hit = self.hit_zone_height / 2.0;
        (divider_pixel_y - screen_y).abs() <= half_hit
    }

    /// Get the screen-space rectangle for this divider.
    pub fn rect(&self, canvas_width: f32, canvas_height: f32) -> Rect {
        let y = self.position as f32 * canvas_height;
        let half_h = self.height / 2.0;
        Rect::new(0.0, y - half_h, canvas_width, self.height)
    }

    /// Get the screen-space hit-test rectangle (larger than visual rect).
    pub fn hit_rect(&self, canvas_width: f32, canvas_height: f32) -> Rect {
        let y = self.position as f32 * canvas_height;
        let half_h = self.hit_zone_height / 2.0;
        Rect::new(0.0, y - half_h, canvas_width, self.hit_zone_height)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_creation() {
        let d = PaneDivider::new(0, 0.7);
        assert_eq!(d.upper_pane_index, 0);
        assert!((d.position - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn divider_with_dimensions() {
        let d = PaneDivider::new(0, 0.5).with_dimensions(6.0, 16.0);
        assert_eq!(d.height, 6.0);
        assert_eq!(d.hit_zone_height, 16.0);
    }

    #[test]
    fn divider_hit_test_inside() {
        let d = PaneDivider::new(0, 0.5);
        // Divider at 0.5, canvas 600px -> divider at y=300
        // Hit zone is 12px, so +/-6px around y=300
        assert!(d.hit_test(300.0, 600.0));
        assert!(d.hit_test(296.0, 600.0));
        assert!(d.hit_test(304.0, 600.0));
    }

    #[test]
    fn divider_hit_test_outside() {
        let d = PaneDivider::new(0, 0.5);
        assert!(!d.hit_test(200.0, 600.0));
        assert!(!d.hit_test(400.0, 600.0));
    }

    #[test]
    fn divider_rect() {
        let d = PaneDivider::new(0, 0.5);
        let rect = d.rect(800.0, 600.0);
        // y = 0.5 * 600 = 300, half_h = 2.0
        assert_eq!(rect.x, 0.0);
        assert!((rect.y - 298.0).abs() < 1.0);
        assert_eq!(rect.width, 800.0);
        assert_eq!(rect.height, 4.0);
    }

    #[test]
    fn divider_hit_rect() {
        let d = PaneDivider::new(0, 0.5);
        let rect = d.hit_rect(800.0, 600.0);
        // y = 300, half_h = 6.0
        assert!((rect.y - 294.0).abs() < 1.0);
        assert_eq!(rect.height, 12.0);
    }

    #[test]
    fn divider_clone() {
        let d = PaneDivider::new(1, 0.6);
        let cloned = d.clone();
        assert_eq!(cloned.upper_pane_index, 1);
        assert!((cloned.position - 0.6).abs() < f64::EPSILON);
    }
}
