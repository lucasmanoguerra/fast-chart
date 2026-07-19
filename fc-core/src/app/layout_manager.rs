use super::pane::Pane;

/// Manages a vertical stack of panes with draggable dividers.
///
/// Default layout: main chart pane (70%) on top, indicator pane (30%) below.
/// Dividers sit between panes and can be dragged to resize.
pub struct LayoutManager {
    /// Ordered list of panes (top to bottom).
    pub panes: Vec<Pane>,
    /// Normalized y-positions of dividers (0.0–1.0), one fewer than panes.
    pub dividers: Vec<f64>,
    /// Minimum height fraction any pane can occupy (default 0.1 = 10%).
    min_pane_height: f64,
    /// Index of the divider currently being dragged, if any.
    dragging_divider: Option<usize>,
    /// Height of the divider hit-zone in pixels (default 6.0).
    divider_hit_zone: f64,
}

impl LayoutManager {
    /// Create the default layout: main pane (70%) + indicator pane (30%).
    pub fn new() -> Self {
        let mut panes = Vec::new();
        panes.push(Pane::new(0, 0.7)); // Main chart pane: 70%
        panes.push(Pane::new(1, 0.3)); // Indicator pane: 30%

        Self {
            panes,
            dividers: vec![0.7], // One divider at 70%
            min_pane_height: 0.1,
            dragging_divider: None,
            divider_hit_zone: 6.0,
        }
    }

    /// Minimum height fraction any pane can occupy.
    pub fn min_pane_height(&self) -> f64 {
        self.min_pane_height
    }

    /// Sum of all pane heights (should always be ≈1.0).
    pub fn total_height(&self) -> f64 {
        self.panes.iter().map(|p| p.height).sum()
    }

    /// Compute the y-offset (in normalized coords) for the top of pane at `index`.
    pub fn pane_y_offset(&self, index: usize) -> f64 {
        self.panes[..index].iter().map(|p| p.height).sum()
    }

    /// Compute the pixel y-offset for pane at `index`.
    pub fn pane_pixel_offset(&self, index: usize, canvas_height: f64) -> f64 {
        self.pane_y_offset(index) * canvas_height
    }

    /// Compute the pixel height for pane at `index`.
    pub fn pane_pixel_height(&self, index: usize, canvas_height: f64) -> f64 {
        self.panes[index].height * canvas_height
    }

    /// Begin dragging a divider.
    pub fn start_drag(&mut self, divider_index: usize) {
        self.dragging_divider = Some(divider_index);
    }

    /// Update the dragged divider position by a pixel delta.
    pub fn update_drag(&mut self, delta_y: f64, canvas_height: f64) {
        if let Some(idx) = self.dragging_divider {
            let delta_frac = delta_y / canvas_height;
            let new_pos = self.dividers[idx] + delta_frac;

            // Clamp: ensure minimum height for adjacent panes
            let min = self.min_pane_height;
            let upper_bound = if idx + 1 < self.panes.len() {
                self.dividers
                    .get(idx + 1)
                    .copied()
                    .unwrap_or(1.0)
                    - min
            } else {
                1.0 - min
            };
            let lower_bound = if idx > 0 {
                self.dividers[idx - 1] + min
            } else {
                min
            };
            let clamped = new_pos.clamp(lower_bound, upper_bound);

            // Update pane heights based on new divider position
            let prev_divider = if idx > 0 {
                self.dividers[idx - 1]
            } else {
                0.0
            };
            let next_divider = if idx + 1 < self.dividers.len() {
                self.dividers[idx + 1]
            } else {
                1.0
            };

            self.panes[idx].height = clamped - prev_divider;
            self.panes[idx + 1].height = next_divider - clamped;

            self.dividers[idx] = clamped;
        }
    }

    /// End the current drag operation.
    pub fn end_drag(&mut self) {
        self.dragging_divider = None;
    }

    /// Test whether a y-pixel position hits a divider.
    ///
    /// Returns the divider index if the position is within `threshold` pixels.
    pub fn hit_test_divider(&self, y: f64, canvas_height: f64) -> Option<usize> {
        let half_zone = self.divider_hit_zone / 2.0;
        self.dividers.iter().position(|&d| {
            let divider_y = d * canvas_height;
            (divider_y - y).abs() <= half_zone
        })
    }

    /// Add a new pane at the bottom with the given height fraction.
    ///
    /// All existing panes are rebalanced so heights sum to 1.0.
    pub fn add_pane(&mut self, height: f64) -> usize {
        let id = self.panes.len();
        self.panes.push(Pane::new(id, height));

        // Rebalance: normalize all heights to sum to 1.0
        let total = self.total_height();
        if total > f64::EPSILON {
            for pane in &mut self.panes {
                pane.height /= total;
            }
        }

        // Rebuild divider positions
        self.rebuild_dividers();
        id
    }

    /// Remove a pane by id.
    ///
    /// Panics if removing the last pane. Rebalances heights afterward.
    pub fn remove_pane(&mut self, id: usize) {
        if self.panes.len() <= 1 {
            return;
        }
        self.panes.retain(|p| p.id != id);

        // Rebalance
        let total = self.total_height();
        if total > f64::EPSILON {
            for pane in &mut self.panes {
                pane.height /= total;
            }
        }

        // Rebuild divider positions and re-index pane ids
        for (i, pane) in self.panes.iter_mut().enumerate() {
            pane.id = i;
        }
        self.rebuild_dividers();
    }

    /// Recompute divider positions from current pane heights.
    fn rebuild_dividers(&mut self) {
        self.dividers.clear();
        let mut cumulative = 0.0;
        for pane in self.panes.iter().take(self.panes.len().saturating_sub(1)) {
            cumulative += pane.height;
            self.dividers.push(cumulative);
        }
    }

    /// Sync all pane viewports' time range to the given range.
    ///
    /// This ensures the x-axis is identical across all panes.
    pub fn sync_time_range(&mut self, time_start: u64, time_end: u64) {
        for pane in &mut self.panes {
            pane.viewport.time_start = time_start;
            pane.viewport.time_end = time_end;
        }
    }

    /// Sync all pane viewports' zoom level.
    pub fn sync_zoom(&mut self, zoom_level: f64) {
        for pane in &mut self.panes {
            pane.viewport.zoom_level = zoom_level;
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout() {
        let layout = LayoutManager::new();
        assert_eq!(layout.panes.len(), 2);
        assert!((layout.total_height() - 1.0).abs() < 0.001);
        assert_eq!(layout.dividers.len(), 1);
        assert!((layout.dividers[0] - 0.7).abs() < 0.001);
    }

    #[test]
    fn default_pane_heights() {
        let layout = LayoutManager::new();
        assert!((layout.panes[0].height - 0.7).abs() < 0.001);
        assert!((layout.panes[1].height - 0.3).abs() < 0.001);
    }

    #[test]
    fn divider_hit_test() {
        let layout = LayoutManager::new();
        // Divider at 0.7, canvas 700px → divider at y≈490
        // Use 490.0 (directly on the divider) and 492.0 (within 6px zone)
        assert!(layout.hit_test_divider(490.0, 700.0).is_some());
        assert!(layout.hit_test_divider(492.0, 700.0).is_some()); // within 6px zone
        assert!(layout.hit_test_divider(100.0, 700.0).is_none());
        assert!(layout.hit_test_divider(600.0, 700.0).is_none());
    }

    #[test]
    fn hit_test_returns_correct_index() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.2); // now 3 panes, 2 dividers
        // Divider 0 at ~0.467, divider 1 at ~0.733 (depends on rebalance)
        let idx = layout.hit_test_divider(layout.dividers[0] * 700.0, 700.0);
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn add_pane_rebalances() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.3);
        assert_eq!(layout.panes.len(), 3);
        assert!((layout.total_height() - 1.0).abs() < 0.001);
        assert_eq!(layout.dividers.len(), 2);
    }

    #[test]
    fn remove_pane_rebalances() {
        let mut layout = LayoutManager::new();
        layout.remove_pane(1);
        assert_eq!(layout.panes.len(), 1);
        assert!((layout.total_height() - 1.0).abs() < 0.001);
        assert!(layout.dividers.is_empty());
    }

    #[test]
    fn remove_pane_reindexes() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.2); // ids: 0, 1, 2
        layout.remove_pane(1); // remove middle
        assert_eq!(layout.panes[0].id, 0);
        assert_eq!(layout.panes[1].id, 1);
    }

    #[test]
    fn cannot_remove_last_pane() {
        let mut layout = LayoutManager::new();
        layout.remove_pane(0);
        assert_eq!(layout.panes.len(), 1); // still 1
    }

    #[test]
    fn min_height_enforced() {
        let mut layout = LayoutManager::new();
        // Try to drag divider way up (make top pane tiny)
        layout.start_drag(0);
        layout.update_drag(-700.0, 700.0); // huge negative delta
        layout.end_drag();
        assert!(layout.panes[0].height >= layout.min_pane_height() - 0.001);
    }

    #[test]
    fn min_height_enforced_drag_down() {
        let mut layout = LayoutManager::new();
        // Try to drag divider way down (make bottom pane tiny)
        layout.start_drag(0);
        layout.update_drag(700.0, 700.0); // huge positive delta
        layout.end_drag();
        assert!(layout.panes[1].height >= layout.min_pane_height() - 0.001);
    }

    #[test]
    fn drag_adjusts_heights() {
        let mut layout = LayoutManager::new();
        let original_top = layout.panes[0].height;
        layout.start_drag(0);
        layout.update_drag(35.0, 700.0); // drag down 5% of canvas
        layout.end_drag();
        // Top pane should have grown
        assert!(layout.panes[0].height > original_top);
        assert!((layout.total_height() - 1.0).abs() < 0.001);
    }

    #[test]
    fn pane_y_offset() {
        let layout = LayoutManager::new();
        assert!((layout.pane_y_offset(0)).abs() < 0.001);
        assert!((layout.pane_y_offset(1) - 0.7).abs() < 0.001);
    }

    #[test]
    fn pane_pixel_dimensions() {
        let layout = LayoutManager::new();
        assert!((layout.pane_pixel_offset(0, 700.0)).abs() < 0.001);
        assert!((layout.pane_pixel_offset(1, 700.0) - 490.0).abs() < 0.001);
        assert!((layout.pane_pixel_height(0, 700.0) - 490.0).abs() < 0.001);
        assert!((layout.pane_pixel_height(1, 700.0) - 210.0).abs() < 0.001);
    }

    #[test]
    fn sync_time_range() {
        let mut layout = LayoutManager::new();
        layout.sync_time_range(1000, 5000);
        for pane in &layout.panes {
            assert_eq!(pane.viewport.time_start, 1000);
            assert_eq!(pane.viewport.time_end, 5000);
        }
    }

    #[test]
    fn sync_zoom() {
        let mut layout = LayoutManager::new();
        layout.sync_zoom(2.5);
        for pane in &layout.panes {
            assert_eq!(pane.viewport.zoom_level, 2.5);
        }
    }

    #[test]
    fn rebuild_dividers_after_add() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.25);
        assert_eq!(layout.dividers.len(), 2);
        // Dividers should match cumulative heights
        let mut cum = 0.0;
        for (i, &d) in layout.dividers.iter().enumerate() {
            cum += layout.panes[i].height;
            assert!((d - cum).abs() < 0.001);
        }
    }

    #[test]
    fn three_pane_layout() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.25);
        assert_eq!(layout.panes.len(), 3);
        assert!((layout.total_height() - 1.0).abs() < 0.001);
        assert_eq!(layout.dividers.len(), 2);
    }
}
