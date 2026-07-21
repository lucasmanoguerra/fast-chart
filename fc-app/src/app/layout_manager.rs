use super::layout::{LayoutEngine, VerticalStack};
use super::pane::Pane;
use super::pane::divider::PaneDivider;
use fc_primitives::Rect;

/// Manages a vertical stack of panes with draggable dividers.
///
/// Default layout: main chart pane (70%) on top, indicator pane (30%) below.
/// Dividers sit between panes and can be dragged to resize.
/// All drag state and interaction logic is owned here — `PaneDivider` is
/// purely a visual/rendering concern.
///
/// Rect computation is delegated to an inner [`LayoutEngine`]. The default
/// engine is [`VerticalStack`] with proportional heights matching the pane
/// height fractions.
pub struct LayoutManager {
    /// Layout engine used for rect computation.
    engine: Box<dyn LayoutEngine>,
    /// Ordered list of panes (top to bottom).
    pub panes: Vec<Pane>,
    /// Dividers between panes, one fewer than panes.
    pub dividers: Vec<PaneDivider>,
    /// Minimum height fraction any pane can occupy (default 0.1 = 10%).
    min_pane_height: f64,
    /// Index of the divider currently being dragged, if any.
    dragging_divider: Option<usize>,
    /// Visual thickness of dividers in pixels.
    divider_height: f32,
    /// Hit zone in pixels for divider interaction.
    divider_hit_zone_px: f32,
}

impl LayoutManager {
    /// Create the default layout: main pane (70%) + indicator pane (30%).
    pub fn new() -> Self {
        let mut panes = Vec::new();
        panes.push(Pane::new(0, 0.7)); // Main chart pane: 70%
        panes.push(Pane::new(1, 0.3)); // Indicator pane: 30%

        Self {
            engine: Box::new(VerticalStack::with_heights(vec![0.7, 0.3])),
            panes,
            dividers: vec![PaneDivider::new(0, 0.7)],
            min_pane_height: 0.1,
            dragging_divider: None,
            divider_height: 4.0,
            divider_hit_zone_px: 12.0,
        }
    }

    // -- Engine accessors ---------------------------------------------------

    /// Immutable access to the current layout engine.
    pub fn engine(&self) -> &dyn LayoutEngine {
        &*self.engine
    }

    /// Mutable access to the current layout engine.
    pub fn engine_mut(&mut self) -> &mut dyn LayoutEngine {
        &mut *self.engine
    }

    /// Swap the layout engine at runtime.
    ///
    /// All subsequent calls to [`compute_rects`] will use the new engine.
    pub fn set_engine(&mut self, engine: Box<dyn LayoutEngine>) {
        self.engine = engine;
    }

    /// Compute pixel rects for all panes using the current layout engine.
    ///
    /// The engine computes proportional rects, then this method adjusts for
    /// divider gaps — panes below each divider are shifted down by the
    /// divider height.
    pub fn compute_rects(&self, canvas: Rect) -> Vec<Rect> {
        let mut rects = self.engine.compute_rects(canvas, self.panes.len());

        // Adjust for divider gaps: shift every pane after a divider
        let gap = self.divider_height;
        for (i, _divider) in self.dividers.iter().enumerate() {
            for rect in rects.iter_mut().skip(i + 1) {
                rect.y += gap;
            }
        }

        rects
    }

    // -- Existing fast-path methods (kept for render-loop performance) ------

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
            let new_pos = self.dividers[idx].position + delta_frac;

            // Clamp: ensure minimum height for adjacent panes
            let min = self.min_pane_height;
            let upper_bound = if idx + 1 < self.panes.len() {
                self.dividers
                    .get(idx + 1)
                    .map(|d| d.position)
                    .unwrap_or(1.0)
                    - min
            } else {
                1.0 - min
            };
            let lower_bound = if idx > 0 {
                self.dividers[idx - 1].position + min
            } else {
                min
            };
            let clamped = new_pos.clamp(lower_bound, upper_bound);

            // Update pane heights based on new divider position
            let prev_divider = if idx > 0 {
                self.dividers[idx - 1].position
            } else {
                0.0
            };
            let next_divider = if idx + 1 < self.dividers.len() {
                self.dividers[idx + 1].position
            } else {
                1.0
            };

            self.panes[idx].height = clamped - prev_divider;
            self.panes[idx + 1].height = next_divider - clamped;

            self.dividers[idx].position = clamped;
        }
    }

    /// End the current drag operation.
    pub fn end_drag(&mut self) {
        self.dragging_divider = None;
    }

    /// Test whether a y-pixel position hits a divider.
    ///
    /// Delegates to `PaneDivider::hit_test()` for each divider.
    /// Returns the divider index if the position is within the hit zone.
    pub fn hit_test_divider(&self, y: f64, canvas_height: f64) -> Option<usize> {
        self.dividers
            .iter()
            .position(|d| d.hit_test(y as f32, canvas_height as f32))
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
        for (i, pane) in self.panes.iter().enumerate().take(self.panes.len().saturating_sub(1)) {
            cumulative += pane.height;
            self.dividers.push(PaneDivider::new(i, cumulative));
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
        assert!((layout.dividers[0].position - 0.7).abs() < 0.001);
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
        // Divider at 0.7, canvas 700px -> divider at y≈490
        assert!(layout.hit_test_divider(490.0, 700.0).is_some());
        assert!(layout.hit_test_divider(492.0, 700.0).is_some()); // within zone
        assert!(layout.hit_test_divider(100.0, 700.0).is_none());
        assert!(layout.hit_test_divider(600.0, 700.0).is_none());
    }

    #[test]
    fn hit_test_returns_correct_index() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.2); // now 3 panes, 2 dividers
        let idx = layout.hit_test_divider(layout.dividers[0].position * 700.0, 700.0);
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
        layout.start_drag(0);
        layout.update_drag(-700.0, 700.0); // huge negative delta
        layout.end_drag();
        assert!(layout.panes[0].height >= layout.min_pane_height() - 0.001);
    }

    #[test]
    fn min_height_enforced_drag_down() {
        let mut layout = LayoutManager::new();
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
        for (i, d) in layout.dividers.iter().enumerate() {
            cum += layout.panes[i].height;
            assert!((d.position - cum).abs() < 0.001);
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

    #[test]
    fn divider_hit_test_delegates_to_pane_divider() {
        let layout = LayoutManager::new();
        // Divider at position 0.7, canvas 700px -> pixel y = 490
        let divider = &layout.dividers[0];
        assert!(divider.hit_test(490.0, 700.0));
        assert!(!divider.hit_test(100.0, 700.0));
    }

    #[test]
    fn divider_position_updates_on_drag() {
        let mut layout = LayoutManager::new();
        let original_position = layout.dividers[0].position;
        layout.start_drag(0);
        layout.update_drag(35.0, 700.0);
        assert!(layout.dividers[0].position > original_position);
        assert!((layout.dividers[0].position - 0.75).abs() < 0.001);
        layout.end_drag();
    }

    // -- Engine integration tests -------------------------------------------

    #[test]
    fn compute_rects_uses_engine() {
        let layout = LayoutManager::new();
        let canvas = Rect::new(0.0, 0.0, 800.0, 700.0);
        let rects = layout.compute_rects(canvas);
        assert_eq!(rects.len(), 2);
        // Pane 0: 0.7 * 700 = 490
        assert!((rects[0].height - 490.0).abs() < 1.0);
        assert_eq!(rects[0].y, 0.0);
        // Pane 1: starts after divider gap (4px)
        assert!((rects[1].height - 210.0).abs() < 1.0);
        assert!((rects[1].y - 494.0).abs() < 1.0);
    }

    #[test]
    fn set_engine_swaps_layout_strategy() {
        let mut layout = LayoutManager::new();
        // Default is VerticalStack with 2 panes (70/30).
        let canvas = Rect::new(0.0, 0.0, 800.0, 600.0);
        let rects_before = layout.compute_rects(canvas);
        assert!((rects_before[0].height - 420.0).abs() < 1.0); // 0.7 * 600

        // Swap to equal VerticalStack
        layout.set_engine(Box::new(VerticalStack::new()));
        let rects_after = layout.compute_rects(canvas);
        assert_eq!(rects_after.len(), 2);
        // Equal distribution: 600/2 = 300 each
        assert!((rects_after[0].height - 300.0).abs() < 1.0);
        assert!((rects_after[1].height - 300.0).abs() < 1.0);
    }

    #[test]
    fn vertical_stack_compute_rects_matches_inline() {
        let layout = LayoutManager::new();
        let canvas_height = 700.0;

        // The engine and the inline methods should agree (ignoring divider gap)
        let engine_rects = layout.engine().compute_rects(
            Rect::new(0.0, 0.0, 0.0, canvas_height as f32),
            layout.panes.len(),
        );
        // Pane 0: both should be 0.7 * 700 = 490
        assert!((engine_rects[0].height as f64 - layout.pane_pixel_height(0, canvas_height)).abs() < 1.0);
        // Pane 1: both should be 0.3 * 700 = 210
        assert!((engine_rects[1].height as f64 - layout.pane_pixel_height(1, canvas_height)).abs() < 1.0);
    }

    #[test]
    fn engine_accessor_returns_immutable_reference() {
        let layout = LayoutManager::new();
        let engine = layout.engine();
        let hint = engine.pane_count_hint();
        // VerticalStack::pane_count_hint() returns 0 (dynamic)
        assert_eq!(hint, 0);
    }

    #[test]
    fn compute_rects_with_three_panes() {
        let mut layout = LayoutManager::new();
        layout.add_pane(0.2);
        let canvas = Rect::new(0.0, 0.0, 800.0, 1000.0);
        let rects = layout.compute_rects(canvas);
        assert_eq!(rects.len(), 3);
        // All rects should have the same width
        for r in &rects {
            assert_eq!(r.width, 800.0);
        }
        // Rects should not overlap (accounting for gap shift)
        for i in 1..rects.len() {
            assert!(rects[i].y >= rects[i - 1].y + rects[i - 1].height);
        }
    }
}
