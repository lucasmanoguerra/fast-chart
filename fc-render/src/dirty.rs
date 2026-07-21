use super::passes::RenderPass;
use fc_primitives::Rect;

/// Type alias for backward compatibility. The canonical type is [`Rect`] in fc-primitives.
pub type ScreenRect = Rect;

/// Dirty region: a pass + the screen rect that needs re-rendering.
#[derive(Debug, Clone)]
pub struct DirtyRegion {
    pub pass: RenderPass,
    pub rect: ScreenRect,
}

/// Tracks dirty regions per pass for selective partial redraw.
pub struct DirtyRegionTracker {
    regions: Vec<DirtyRegion>,
    surface_width: f32,
    surface_height: f32,
}

impl DirtyRegionTracker {
    pub fn new(surface_width: f32, surface_height: f32) -> Self {
        Self {
            regions: Vec::new(),
            surface_width,
            surface_height,
        }
    }

    /// Mark a specific rect as dirty for a pass.
    pub fn mark_dirty(&mut self, pass: RenderPass, rect: ScreenRect) {
        let merged = self.merge_with_existing(pass, rect);
        self.regions.push(DirtyRegion {
            pass,
            rect: merged,
        });
    }

    /// Mark the entire surface as dirty for a pass.
    pub fn mark_full_dirty(&mut self, pass: RenderPass) {
        self.mark_dirty(pass, ScreenRect::full(self.surface_width, self.surface_height));
    }

    /// Mark all passes as fully dirty.
    pub fn mark_all_dirty(&mut self) {
        let full = ScreenRect::full(self.surface_width, self.surface_height);
        for &pass in RenderPass::ALL {
            self.mark_dirty(pass, full);
        }
    }

    /// Get all dirty regions for a pass.
    pub fn dirty_regions(&self, pass: RenderPass) -> Vec<&DirtyRegion> {
        self.regions.iter().filter(|r| r.pass == pass).collect()
    }

    /// Check if a pass has any dirty regions.
    pub fn is_dirty(&self, pass: RenderPass) -> bool {
        self.regions.iter().any(|r| r.pass == pass)
    }

    /// Get the merged bounding rect for a pass (union of all dirty rects).
    pub fn merged_rect(&self, pass: RenderPass) -> Option<ScreenRect> {
        let rects: Vec<&ScreenRect> = self.regions.iter().filter(|r| r.pass == pass).map(|r| &r.rect).collect();
        if rects.is_empty() {
            return None;
        }

        let mut result = *rects[0];
        for &rect in &rects[1..] {
            result = result.union(rect);
        }

        let surface_area = self.surface_width * self.surface_height;
        if surface_area > 0.0 && result.area() > surface_area * 0.5 {
            return Some(ScreenRect::full(self.surface_width, self.surface_height));
        }

        Some(result)
    }

    /// Check if a specific rect needs re-rendering for a pass.
    pub fn needs_redraw(&self, pass: RenderPass, rect: &ScreenRect) -> bool {
        self.regions
            .iter()
            .filter(|r| r.pass == pass)
            .any(|r| r.rect.intersects(rect))
    }

    /// Clear dirty regions for a pass (after rendering).
    pub fn clear(&mut self, pass: RenderPass) {
        self.regions.retain(|r| r.pass != pass);
    }

    /// Clear all dirty regions.
    pub fn clear_all(&mut self) {
        self.regions.clear();
    }

    /// Update surface dimensions.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    /// Get total number of dirty regions (all passes combined).
    pub fn dirty_count(&self) -> usize {
        self.regions.len()
    }

    /// Try to merge a new rect with an existing dirty region for the same pass.
    /// If merging produces a rect > 50% of surface, return full surface instead.
    fn merge_with_existing(&self, pass: RenderPass, rect: ScreenRect) -> ScreenRect {
        let existing: Vec<&ScreenRect> = self
            .regions
            .iter()
            .filter(|r| r.pass == pass)
            .map(|r| &r.rect)
            .collect();

        if existing.is_empty() {
            return rect;
        }

        let mut merged = rect;
        for &other in &existing {
            if merged.intersects_or_adjacent(other) {
                merged = merged.union(other);
            }
        }

        let surface_area = self.surface_width * self.surface_height;
        if surface_area > 0.0 && merged.area() > surface_area * 0.5 {
            ScreenRect::full(self.surface_width, self.surface_height)
        } else {
            merged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_tracker_new() {
        let tracker = DirtyRegionTracker::new(800.0, 600.0);
        assert_eq!(tracker.dirty_count(), 0);
        assert!(!tracker.is_dirty(RenderPass::Grid));
    }

    #[test]
    fn mark_dirty_single() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        let rect = ScreenRect::new(0.0, 0.0, 100.0, 100.0);
        tracker.mark_dirty(RenderPass::Grid, rect);
        assert!(tracker.is_dirty(RenderPass::Grid));
        assert_eq!(tracker.dirty_regions(RenderPass::Grid).len(), 1);
    }

    #[test]
    fn mark_dirty_multiple() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        let a = ScreenRect::new(0.0, 0.0, 50.0, 50.0);
        let b = ScreenRect::new(200.0, 200.0, 50.0, 50.0);
        tracker.mark_dirty(RenderPass::Series, a);
        tracker.mark_dirty(RenderPass::Series, b);
        assert_eq!(tracker.dirty_regions(RenderPass::Series).len(), 2);
    }

    #[test]
    fn mark_full_dirty() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        tracker.mark_full_dirty(RenderPass::Crosshair);
        assert!(tracker.is_dirty(RenderPass::Crosshair));
        let regions = tracker.dirty_regions(RenderPass::Crosshair);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].rect, ScreenRect::full(800.0, 600.0));
    }

    #[test]
    fn merged_rect_single() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        let rect = ScreenRect::new(10.0, 20.0, 100.0, 50.0);
        tracker.mark_dirty(RenderPass::Grid, rect);
        let merged = tracker.merged_rect(RenderPass::Grid);
        assert_eq!(merged, Some(rect));
    }

    #[test]
    fn merged_rect_multiple() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        let a = ScreenRect::new(0.0, 0.0, 50.0, 50.0);
        let b = ScreenRect::new(200.0, 200.0, 50.0, 50.0);
        tracker.mark_dirty(RenderPass::Series, a);
        tracker.mark_dirty(RenderPass::Series, b);
        let merged = tracker.merged_rect(RenderPass::Series).unwrap();
        assert_eq!(merged, ScreenRect::new(0.0, 0.0, 250.0, 250.0));
    }

    #[test]
    fn needs_redraw() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        tracker.mark_dirty(RenderPass::Grid, ScreenRect::new(0.0, 0.0, 100.0, 100.0));
        assert!(tracker.needs_redraw(RenderPass::Grid, &ScreenRect::new(50.0, 50.0, 10.0, 10.0)));
        assert!(!tracker.needs_redraw(RenderPass::Grid, &ScreenRect::new(200.0, 200.0, 10.0, 10.0)));
        assert!(!tracker.needs_redraw(RenderPass::Series, &ScreenRect::new(50.0, 50.0, 10.0, 10.0)));
    }

    #[test]
    fn clear_pass() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        tracker.mark_dirty(RenderPass::Grid, ScreenRect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(RenderPass::Series, ScreenRect::new(0.0, 0.0, 50.0, 50.0));
        tracker.clear(RenderPass::Grid);
        assert!(!tracker.is_dirty(RenderPass::Grid));
        assert!(tracker.is_dirty(RenderPass::Series));
    }

    #[test]
    fn clear_all() {
        let mut tracker = DirtyRegionTracker::new(800.0, 600.0);
        tracker.mark_dirty(RenderPass::Grid, ScreenRect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(RenderPass::Series, ScreenRect::new(0.0, 0.0, 50.0, 50.0));
        tracker.clear_all();
        assert_eq!(tracker.dirty_count(), 0);
    }
}
