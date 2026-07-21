//! Crosshair state machine.
//!
//! Tracks crosshair mode, position, and snapping behaviour. The controller
//! is a pure state machine — it knows nothing about the data series. The host
//! application resolves screen coordinates to world space and provides the
//! nearest data point when in Magnetic mode.

// ---------------------------------------------------------------------------
// CrosshairMode
// ---------------------------------------------------------------------------

/// Crosshair interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrosshairMode {
    /// Standard crosshair that follows the cursor.
    Normal,
    /// Snaps to the nearest data point within threshold.
    Magnetic,
    /// Crosshair not visible.
    Hidden,
    /// Crosshair position syncs across multiple chart panes.
    Sync,
    /// Crosshair visible across all panes at once.
    Global,
    /// User provides a custom position override.
    Custom,
}

// ---------------------------------------------------------------------------
// CrosshairPosition
// ---------------------------------------------------------------------------

/// Crosshair position in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrosshairPosition {
    /// Time coordinate.
    pub time: f64,
    /// Price coordinate.
    pub price: f64,
    /// Whether this position is snapped to a data point.
    pub snapped: bool,
}

// ---------------------------------------------------------------------------
// CrosshairController
// ---------------------------------------------------------------------------

/// Manages crosshair state for a single chart.
pub struct CrosshairController {
    mode: CrosshairMode,
    position: Option<CrosshairPosition>,
    custom_position: Option<CrosshairPosition>,
    visible: bool,
    sync_group: Option<u32>,
    snap_threshold: f64,
}

impl CrosshairController {
    /// Create a new controller in Normal mode.
    pub fn new() -> Self {
        Self {
            mode: CrosshairMode::Normal,
            position: None,
            custom_position: None,
            visible: true,
            sync_group: None,
            snap_threshold: 0.05,
        }
    }

    /// Get current mode.
    pub fn mode(&self) -> CrosshairMode {
        self.mode
    }

    /// Set mode.
    pub fn set_mode(&mut self, mode: CrosshairMode) {
        self.mode = mode;
        if mode == CrosshairMode::Hidden {
            self.visible = false;
        } else if mode != CrosshairMode::Custom {
            self.visible = true;
        }
    }

    /// Get current position (if any).
    pub fn position(&self) -> Option<&CrosshairPosition> {
        match self.mode {
            CrosshairMode::Custom => self.custom_position.as_ref(),
            _ => self.position.as_ref(),
        }
    }

    /// Update crosshair position from world coordinates.
    ///
    /// In Magnetic mode, if `nearest_data_point` is provided and within
    /// `snap_threshold` (Euclidean distance in normalized 0.0–1.0 space),
    /// the crosshair snaps to that point.
    pub fn update_position(
        &mut self,
        time: f64,
        price: f64,
        nearest_data_point: Option<(f64, f64)>,
    ) -> &CrosshairPosition {
        let snapped = if self.mode == CrosshairMode::Magnetic {
            nearest_data_point
                .map(|(dt, dp)| {
                    let dist = ((dt - time).powi(2) + (dp - price).powi(2)).sqrt();
                    dist < self.snap_threshold
                })
                .unwrap_or(false)
        } else {
            false
        };

        let (time, price) = match (snapped, nearest_data_point) {
            (true, Some(pt)) => pt,
            _ => (time, price),
        };

        self.position = Some(CrosshairPosition { time, price, snapped });
        self.position.as_ref().unwrap()
    }

    /// Set custom position (for Custom mode).
    pub fn set_custom_position(&mut self, time: f64, price: f64) {
        self.custom_position = Some(CrosshairPosition { time, price, snapped: false });
    }

    /// Clear crosshair position.
    pub fn clear(&mut self) {
        self.position = None;
        self.custom_position = None;
    }

    /// Is the crosshair visible?
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Get sync group ID.
    pub fn sync_group(&self) -> Option<u32> {
        self.sync_group
    }

    /// Set sync group.
    pub fn set_sync_group(&mut self, group: Option<u32>) {
        self.sync_group = group;
    }

    /// Set snap threshold (for Magnetic mode).
    pub fn set_snap_threshold(&mut self, threshold: f64) {
        self.snap_threshold = threshold;
    }

    /// Get snap threshold.
    pub fn snap_threshold(&self) -> f64 {
        self.snap_threshold
    }

    /// Check if this crosshair should sync with another.
    pub fn should_sync_with(&self, other: &CrosshairController) -> bool {
        match (self.sync_group, other.sync_group) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

impl Default for CrosshairController {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_normal() {
        let ctrl = CrosshairController::new();
        assert_eq!(ctrl.mode(), CrosshairMode::Normal);
    }

    #[test]
    fn set_mode() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Magnetic);
        assert_eq!(ctrl.mode(), CrosshairMode::Magnetic);
    }

    #[test]
    fn update_position_basic() {
        let mut ctrl = CrosshairController::new();
        ctrl.update_position(100.0, 50.0, None);
        let pos = ctrl.position().expect("position should exist");
        assert_eq!(pos.time, 100.0);
        assert_eq!(pos.price, 50.0);
    }

    #[test]
    fn update_position_returns_ref() {
        let mut ctrl = CrosshairController::new();
        let pos = ctrl.update_position(10.0, 20.0, None);
        assert_eq!(pos.time, 10.0);
        assert_eq!(pos.price, 20.0);
    }

    #[test]
    fn clear_position() {
        let mut ctrl = CrosshairController::new();
        ctrl.update_position(100.0, 50.0, None);
        assert!(ctrl.position().is_some());
        ctrl.clear();
        assert!(ctrl.position().is_none());
    }

    #[test]
    fn magnetic_snaps_to_data() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Magnetic);
        ctrl.set_snap_threshold(0.1);
        let pos = ctrl.update_position(0.5, 0.5, Some((0.52, 0.51)));
        assert!(pos.snapped);
        assert_eq!(pos.time, 0.52);
        assert_eq!(pos.price, 0.51);
    }

    #[test]
    fn magnetic_no_snap_far() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Magnetic);
        ctrl.set_snap_threshold(0.01);
        let pos = ctrl.update_position(0.0, 0.0, Some((0.5, 0.5)));
        assert!(!pos.snapped);
        assert_eq!(pos.time, 0.0);
        assert_eq!(pos.price, 0.0);
    }

    #[test]
    fn magnetic_respects_threshold() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Magnetic);
        ctrl.set_snap_threshold(0.1);
        let pos1 = ctrl.update_position(0.5, 0.5, Some((0.55, 0.55)));
        assert!(pos1.snapped);

        ctrl.set_snap_threshold(0.01);
        let pos2 = ctrl.update_position(0.5, 0.5, Some((0.55, 0.55)));
        assert!(!pos2.snapped);
    }

    #[test]
    fn hidden_mode_not_visible() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Hidden);
        assert!(!ctrl.is_visible());
    }

    #[test]
    fn custom_position_override() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Custom);
        ctrl.set_custom_position(42.0, 99.0);
        let pos = ctrl.position().expect("custom position should exist");
        assert_eq!(pos.time, 42.0);
        assert_eq!(pos.price, 99.0);
    }

    #[test]
    fn sync_group_same() {
        let mut a = CrosshairController::new();
        let mut b = CrosshairController::new();
        a.set_sync_group(Some(1));
        b.set_sync_group(Some(1));
        assert!(a.should_sync_with(&b));
    }

    #[test]
    fn sync_group_different() {
        let mut a = CrosshairController::new();
        let mut b = CrosshairController::new();
        a.set_sync_group(Some(1));
        b.set_sync_group(Some(2));
        assert!(!a.should_sync_with(&b));
    }

    #[test]
    fn sync_group_none() {
        let a = CrosshairController::new();
        let b = CrosshairController::new();
        assert!(!a.should_sync_with(&b));
    }

    #[test]
    fn visibility_toggle() {
        let mut ctrl = CrosshairController::new();
        assert!(ctrl.is_visible());
        ctrl.set_visible(false);
        assert!(!ctrl.is_visible());
        ctrl.set_visible(true);
        assert!(ctrl.is_visible());
    }

    #[test]
    fn position_snapped_flag() {
        let mut ctrl = CrosshairController::new();
        ctrl.set_mode(CrosshairMode::Magnetic);
        ctrl.set_snap_threshold(1.0);
        let pos_snapped = ctrl.update_position(0.0, 0.0, Some((0.1, 0.1)));
        assert!(pos_snapped.snapped);
        let pos_not_snapped = ctrl.update_position(0.0, 0.0, None);
        assert!(!pos_not_snapped.snapped);
    }
}
