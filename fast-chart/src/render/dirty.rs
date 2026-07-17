use super::passes::RenderPass;

/// A rectangular region in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ScreenRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Full surface rect.
    pub fn full(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    /// Right edge (x + width).
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Bottom edge (y + height).
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Area of this rect.
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check if this rect intersects another.
    pub fn intersects(&self, other: &ScreenRect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Check if this rect intersects another, treating rects within 1px as adjacent.
    pub fn intersects_or_adjacent(&self, other: &ScreenRect) -> bool {
        let epsilon = 1.0;
        self.x - epsilon < other.right()
            && self.right() + epsilon > other.x
            && self.y - epsilon < other.bottom()
            && self.bottom() + epsilon > other.y
    }

    /// Compute the union (bounding box) of two rects.
    pub fn union(&self, other: &ScreenRect) -> ScreenRect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        ScreenRect::new(x, y, right - x, bottom - y)
    }

    /// Check if a point is inside this rect.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Check if this rect fully contains another rect.
    pub fn contains_rect(&self, other: &ScreenRect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Convert to wgpu scissor rect format: (x, y, w, h) in pixels, y-flipped.
    pub fn to_scissor(&self, surface_height: f32) -> (u32, u32, u32, u32) {
        let x = self.x.max(0.0) as u32;
        let y_flipped = (surface_height - self.bottom()).max(0.0) as u32;
        let w = self.width.max(0.0) as u32;
        let h = self.height.max(0.0) as u32;
        (x, y_flipped, w, h)
    }
}

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
    fn screen_rect_new() {
        let r = ScreenRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 50.0);
    }

    #[test]
    fn screen_rect_full() {
        let r = ScreenRect::full(800.0, 600.0);
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 0.0);
        assert_eq!(r.width, 800.0);
        assert_eq!(r.height, 600.0);
    }

    #[test]
    fn screen_rect_intersects_true() {
        let a = ScreenRect::new(0.0, 0.0, 100.0, 100.0);
        let b = ScreenRect::new(50.0, 50.0, 100.0, 100.0);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn screen_rect_intersects_false() {
        let a = ScreenRect::new(0.0, 0.0, 100.0, 100.0);
        let b = ScreenRect::new(200.0, 200.0, 100.0, 100.0);
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn screen_rect_union() {
        let a = ScreenRect::new(0.0, 0.0, 100.0, 100.0);
        let b = ScreenRect::new(50.0, 50.0, 100.0, 100.0);
        let u = a.union(&b);
        assert_eq!(u, ScreenRect::new(0.0, 0.0, 150.0, 150.0));
    }

    #[test]
    fn screen_rect_contains_point() {
        let r = ScreenRect::new(10.0, 10.0, 80.0, 80.0);
        assert!(r.contains(50.0, 50.0));
        assert!(!r.contains(5.0, 5.0));
        assert!(!r.contains(100.0, 100.0));
    }

    #[test]
    fn screen_rect_contains_rect() {
        let outer = ScreenRect::new(0.0, 0.0, 200.0, 200.0);
        let inner = ScreenRect::new(10.0, 10.0, 50.0, 50.0);
        assert!(outer.contains_rect(&inner));
        assert!(!inner.contains_rect(&outer));
    }

    #[test]
    fn screen_rect_to_scissor() {
        let r = ScreenRect::new(10.0, 20.0, 100.0, 50.0);
        let (x, y, w, h) = r.to_scissor(600.0);
        assert_eq!(x, 10);
        assert_eq!(y, 530);
        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

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
