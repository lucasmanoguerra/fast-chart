// ---------------------------------------------------------------------------
// PaneDivider — draggable separator between panes
// ---------------------------------------------------------------------------

use crate::render::series_renderer::Rect;

/// A draggable separator between two adjacent panes.
///
/// Handles hit testing, cursor changes, and resize events.
/// Dividers are horizontal bars that sit between vertically stacked panes.
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
    /// Whether this divider is currently being dragged.
    pub is_dragging: bool,
    /// Minimum allowed position (ensures upper pane has minimum height).
    pub min_position: f64,
    /// Maximum allowed position (ensures lower pane has minimum height).
    pub max_position: f64,
    /// Cursor style when hovering over this divider.
    pub cursor: DividerCursor,
}

/// Cursor styles for divider interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerCursor {
    /// Default: vertical resize cursor (ns-resize).
    ResizeVertical,
    /// While dragging.
    Grabbing,
    /// When hover is not on the divider.
    Default,
}

impl PaneDivider {
    /// Create a new divider between two panes.
    pub fn new(upper_pane_index: usize, position: f64) -> Self {
        Self {
            upper_pane_index,
            position,
            height: 4.0,
            hit_zone_height: 12.0,
            is_dragging: false,
            min_position: 0.05,
            max_position: 0.95,
            cursor: DividerCursor::Default,
        }
    }

    /// Create a divider with custom dimensions.
    pub fn with_dimensions(mut self, height: f32, hit_zone: f32) -> Self {
        self.height = height;
        self.hit_zone_height = hit_zone;
        self
    }

    /// Create a divider with position constraints.
    pub fn with_constraints(mut self, min: f64, max: f64) -> Self {
        self.min_position = min;
        self.max_position = max;
        self
    }

    /// Test if a screen-space y-coordinate hits this divider.
    ///
    /// `divider_y` is the pixel y-position of the divider center.
    /// `screen_y` is the y-coordinate to test.
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

    /// Begin dragging this divider.
    pub fn start_drag(&mut self) {
        self.is_dragging = true;
        self.cursor = DividerCursor::Grabbing;
    }

    /// Update divider position from a screen-space y-coordinate.
    ///
    /// Returns the pixel delta from the last position.
    pub fn update_drag(&mut self, screen_y: f32, canvas_height: f32) -> f32 {
        if !self.is_dragging {
            return 0.0;
        }

        let old_y = self.position as f32 * canvas_height;
        let new_position = (screen_y as f64 / canvas_height as f64).clamp(
            self.min_position,
            self.max_position,
        );

        let delta = new_position as f32 * canvas_height - old_y;
        self.position = new_position;
        delta
    }

    /// End the drag operation.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.cursor = DividerCursor::ResizeVertical;
    }

    /// Update cursor based on hover state.
    pub fn set_hover(&mut self, hovering: bool) {
        if !self.is_dragging {
            self.cursor = if hovering {
                DividerCursor::ResizeVertical
            } else {
                DividerCursor::Default
            };
        }
    }

    /// Set position constraints.
    pub fn set_constraints(&mut self, min: f64, max: f64) {
        self.min_position = min;
        self.max_position = max;
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
        assert!(!d.is_dragging);
        assert_eq!(d.cursor, DividerCursor::Default);
    }

    #[test]
    fn divider_with_dimensions() {
        let d = PaneDivider::new(0, 0.5).with_dimensions(6.0, 16.0);
        assert_eq!(d.height, 6.0);
        assert_eq!(d.hit_zone_height, 16.0);
    }

    #[test]
    fn divider_with_constraints() {
        let d = PaneDivider::new(0, 0.5).with_constraints(0.1, 0.9);
        assert!((d.min_position - 0.1).abs() < f64::EPSILON);
        assert!((d.max_position - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn divider_hit_test_inside() {
        let d = PaneDivider::new(0, 0.5);
        // Divider at 0.5, canvas 600px → divider at y=300
        // Hit zone is 12px, so ±6px around y=300
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
    fn divider_start_drag() {
        let mut d = PaneDivider::new(0, 0.5);
        d.start_drag();
        assert!(d.is_dragging);
        assert_eq!(d.cursor, DividerCursor::Grabbing);
    }

    #[test]
    fn divider_update_drag() {
        let mut d = PaneDivider::new(0, 0.5);
        d.start_drag();
        let delta = d.update_drag(330.0, 600.0);
        // New position = 330/600 = 0.55, old y = 300, new y = 330, delta = 30
        assert!((d.position - 0.55).abs() < 0.01);
        assert!(delta > 25.0);
    }

    #[test]
    fn divider_update_drag_clamped() {
        let mut d = PaneDivider::new(0, 0.5).with_constraints(0.1, 0.9);
        d.start_drag();
        // Try to drag way past max
        d.update_drag(590.0, 600.0);
        assert!((d.position - 0.9).abs() < 0.01);
    }

    #[test]
    fn divider_update_drag_not_dragging() {
        let mut d = PaneDivider::new(0, 0.5);
        let delta = d.update_drag(330.0, 600.0);
        assert!((delta).abs() < f32::EPSILON);
    }

    #[test]
    fn divider_end_drag() {
        let mut d = PaneDivider::new(0, 0.5);
        d.start_drag();
        d.end_drag();
        assert!(!d.is_dragging);
        assert_eq!(d.cursor, DividerCursor::ResizeVertical);
    }

    #[test]
    fn divider_set_hover() {
        let mut d = PaneDivider::new(0, 0.5);
        d.set_hover(true);
        assert_eq!(d.cursor, DividerCursor::ResizeVertical);
        d.set_hover(false);
        assert_eq!(d.cursor, DividerCursor::Default);
    }

    #[test]
    fn divider_hover_ignored_while_dragging() {
        let mut d = PaneDivider::new(0, 0.5);
        d.start_drag();
        d.set_hover(false); // should not change cursor
        assert_eq!(d.cursor, DividerCursor::Grabbing);
    }

    #[test]
    fn divider_set_constraints() {
        let mut d = PaneDivider::new(0, 0.5);
        d.set_constraints(0.2, 0.8);
        assert!((d.min_position - 0.2).abs() < f64::EPSILON);
        assert!((d.max_position - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn divider_clone() {
        let d = PaneDivider::new(1, 0.6);
        let cloned = d.clone();
        assert_eq!(cloned.upper_pane_index, 1);
        assert!((cloned.position - 0.6).abs() < f64::EPSILON);
    }
}
