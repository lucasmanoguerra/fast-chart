/// A scissor rectangle in screen coordinates (pixels, top-left origin).
///
/// All coordinates use `u32` pixel values. The y-axis points downward
/// (top-left origin), consistent with screen layout conventions.
/// Conversion to wgpu's bottom-left origin is handled by [`to_wgpu`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ScissorRect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Full surface rect covering the entire viewport.
    pub fn full(width: u32, height: u32) -> Self {
        Self::new(0, 0, width, height)
    }

    /// Right edge (exclusive): `x + width`.
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    /// Bottom edge (exclusive): `y + height`.
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    /// Intersect with another rect. Returns `None` if there is no overlap.
    ///
    /// The intersection is the largest rectangle contained in both rects,
    /// computed as `max(left) / max(top) / min(right) / min(bottom)`.
    pub fn intersect(&self, other: &ScissorRect) -> Option<ScissorRect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        let width = right.saturating_sub(left);
        let height = bottom.saturating_sub(top);

        if width == 0 || height == 0 {
            None
        } else {
            Some(ScissorRect::new(left, top, width, height))
        }
    }

    /// Check if this rect contains the point at `(x, y)`.
    ///
    /// The point must be strictly inside the rect bounds (not on the
    /// exclusive right/bottom edge).
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Convert to wgpu scissor rect format: `(x, y, width, height)`.
    ///
    /// The y-coordinate is flipped from screen space (top-left origin)
    /// to wgpu's bottom-left origin using the provided `surface_height`.
    pub fn to_wgpu(&self, surface_height: u32) -> (u32, u32, u32, u32) {
        let y_flipped = surface_height.saturating_sub(self.bottom());
        (self.x, y_flipped, self.width, self.height)
    }
}

/// Manages a stack of scissor rects for multi-pane rendering.
///
/// Each pane pushes its scissor rect before rendering and pops it after.
/// Nested pushes are automatically intersected with the current effective
/// rect, ensuring that inner panes cannot draw outside their parent's
/// clipping region.
///
/// # Multi-pane example
///
/// ```text
/// ScissorManager starts with no active rect (full surface).
///
/// Push(pane1: 0, 0, 1920, 400)     → current = (0, 0, 1920, 400)
///   Render candle pane...
/// Pop()                              → current = None
///
/// Push(pane2: 0, 400, 1920, 300)   → current = (0, 400, 1920, 300)
///   Render volume pane...
/// Pop()                              → current = None
///
/// Push(pane3: 0, 700, 1920, 380)   → current = (0, 700, 1920, 380)
///   Render RSI pane...
/// Pop()                              → current = None
/// ```
#[derive(Debug)]
pub struct ScissorManager {
    stack: Vec<ScissorRect>,
    current: Option<ScissorRect>,
    surface_width: u32,
    surface_height: u32,
}

impl ScissorManager {
    pub fn new(surface_width: u32, surface_height: u32) -> Self {
        Self {
            stack: Vec::new(),
            current: None,
            surface_width,
            surface_height,
        }
    }

    /// Push a scissor rect onto the stack.
    ///
    /// If the stack is non-empty, the new rect is intersected with the
    /// current effective rect. If the stack is empty, the pushed rect
    /// becomes the current rect directly.
    pub fn push(&mut self, rect: ScissorRect) {
        let effective = match self.current {
            Some(current) => current.intersect(&rect),
            None => Some(rect),
        };
        self.stack.push(rect);
        self.current = effective;
    }

    /// Pop the top scissor rect.
    ///
    /// Restores the previous effective rect from the stack. Returns the
    /// popped rect, or `None` if the stack was empty.
    pub fn pop(&mut self) -> Option<ScissorRect> {
        let popped = self.stack.pop()?;

        self.current = if self.stack.is_empty() {
            None
        } else {
            // Recompute the intersection of all remaining rects.
            let mut acc = self.stack[0];
            for rect in &self.stack[1..] {
                match acc.intersect(rect) {
                    Some(intersection) => acc = intersection,
                    None => return Some(popped),
                }
            }
            Some(acc)
        };

        Some(popped)
    }

    /// Get the current effective scissor rect.
    ///
    /// Returns `None` when no scissor is active (i.e. the full surface
    /// is the effective clipping region).
    pub fn current(&self) -> Option<&ScissorRect> {
        self.current.as_ref()
    }

    /// Get the current effective scissor in wgpu format `(x, y, w, h)`.
    ///
    /// When no scissor is active, returns the full surface rect.
    pub fn current_wgpu(&self) -> (u32, u32, u32, u32) {
        match &self.current {
            Some(rect) => rect.to_wgpu(self.surface_height),
            None => (0, 0, self.surface_width, self.surface_height),
        }
    }

    /// Reset the manager: clear the stack and set the current rect to
    /// the full surface.
    pub fn reset(&mut self) {
        self.stack.clear();
        self.current = None;
    }

    /// Update surface dimensions (call on window resize).
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    /// Current stack depth.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ScissorRect tests ────────────────────────────────────────────

    #[test]
    fn scissor_rect_new() {
        let r = ScissorRect::new(10, 20, 300, 400);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 300);
        assert_eq!(r.height, 400);
    }

    #[test]
    fn scissor_rect_full() {
        let r = ScissorRect::full(1920, 1080);
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
        assert_eq!(r.width, 1920);
        assert_eq!(r.height, 1080);
    }

    #[test]
    fn scissor_rect_intersect_overlapping() {
        let a = ScissorRect::new(0, 0, 100, 100);
        let b = ScissorRect::new(50, 50, 100, 100);
        let result = a.intersect(&b).expect("should intersect");
        assert_eq!(result, ScissorRect::new(50, 50, 50, 50));
    }

    #[test]
    fn scissor_rect_intersect_non_overlapping() {
        let a = ScissorRect::new(0, 0, 100, 100);
        let b = ScissorRect::new(200, 200, 100, 100);
        assert!(a.intersect(&b).is_none());
    }

    #[test]
    fn scissor_rect_intersect_edge_touching() {
        let a = ScissorRect::new(0, 0, 100, 100);
        let b = ScissorRect::new(100, 0, 100, 100);
        assert!(a.intersect(&b).is_none());
    }

    #[test]
    fn scissor_rect_intersect_contained() {
        let outer = ScissorRect::new(0, 0, 200, 200);
        let inner = ScissorRect::new(10, 10, 50, 50);
        let result = outer.intersect(&inner).expect("should intersect");
        assert_eq!(result, inner);
    }

    #[test]
    fn scissor_rect_contains_inside() {
        let r = ScissorRect::new(10, 10, 80, 80);
        assert!(r.contains(10, 10));
        assert!(r.contains(50, 50));
        assert!(r.contains(89, 89));
    }

    #[test]
    fn scissor_rect_contains_outside() {
        let r = ScissorRect::new(10, 10, 80, 80);
        assert!(!r.contains(0, 0));
        assert!(!r.contains(90, 50));
        assert!(!r.contains(50, 90));
    }

    #[test]
    fn scissor_rect_to_wgpu_flips_y() {
        let r = ScissorRect::new(10, 100, 200, 300);
        let (x, y, w, h) = r.to_wgpu(1080);
        assert_eq!(x, 10);
        assert_eq!(y, 680); // 1080 - (100 + 300)
        assert_eq!(w, 200);
        assert_eq!(h, 300);
    }

    // ── ScissorManager tests ─────────────────────────────────────────

    #[test]
    fn scissor_manager_new() {
        let mgr = ScissorManager::new(1920, 1080);
        assert!(mgr.current().is_none());
        assert_eq!(mgr.depth(), 0);
        assert_eq!(mgr.current_wgpu(), (0, 0, 1920, 1080));
    }

    #[test]
    fn push_single() {
        let mut mgr = ScissorManager::new(1920, 1080);
        let rect = ScissorRect::new(0, 0, 1920, 400);
        mgr.push(rect);
        assert_eq!(mgr.current(), Some(&rect));
        assert_eq!(mgr.depth(), 1);
    }

    #[test]
    fn push_multiple_intersects() {
        let mut mgr = ScissorManager::new(1920, 1080);
        mgr.push(ScissorRect::new(0, 0, 1920, 1080));
        mgr.push(ScissorRect::new(0, 100, 1920, 800));
        assert_eq!(
            mgr.current(),
            Some(&ScissorRect::new(0, 100, 1920, 800))
        );
        assert_eq!(mgr.depth(), 2);
    }

    #[test]
    fn push_multiple_no_overlap() {
        let mut mgr = ScissorManager::new(1920, 1080);
        mgr.push(ScissorRect::new(0, 0, 1920, 400));
        // This push has no overlap with the current scissor.
        mgr.push(ScissorRect::new(0, 500, 1920, 300));
        assert!(mgr.current().is_none());
    }

    #[test]
    fn pop_restores() {
        let mut mgr = ScissorManager::new(1920, 1080);
        let pane1 = ScissorRect::new(0, 0, 1920, 400);
        mgr.push(pane1);
        let pane2 = ScissorRect::new(0, 400, 1920, 300);
        mgr.push(pane2);

        let popped = mgr.pop().expect("should pop pane2");
        assert_eq!(popped, pane2);
        assert_eq!(mgr.current(), Some(&pane1));
        assert_eq!(mgr.depth(), 1);
    }

    #[test]
    fn pop_empty_stack() {
        let mut mgr = ScissorManager::new(1920, 1080);
        assert!(mgr.pop().is_none());
        assert!(mgr.current().is_none());
    }

    #[test]
    fn reset() {
        let mut mgr = ScissorManager::new(1920, 1080);
        mgr.push(ScissorRect::new(0, 0, 1920, 400));
        mgr.push(ScissorRect::new(0, 400, 1920, 300));
        mgr.reset();
        assert!(mgr.current().is_none());
        assert_eq!(mgr.depth(), 0);
    }

    #[test]
    fn resize() {
        let mut mgr = ScissorManager::new(1920, 1080);
        assert_eq!(mgr.current_wgpu(), (0, 0, 1920, 1080));
        mgr.resize(2560, 1440);
        assert_eq!(mgr.current_wgpu(), (0, 0, 2560, 1440));
    }

    #[test]
    fn current_wgpu_with_active_scissor() {
        let mut mgr = ScissorManager::new(1920, 1080);
        mgr.push(ScissorRect::new(0, 400, 1920, 300));
        let (x, y, w, h) = mgr.current_wgpu();
        assert_eq!(x, 0);
        assert_eq!(y, 380); // 1080 - (400 + 300)
        assert_eq!(w, 1920);
        assert_eq!(h, 300);
    }

    #[test]
    fn three_panes_sequential() {
        let mut mgr = ScissorManager::new(1920, 1080);

        // Pane 1: candles
        let p1 = ScissorRect::new(0, 0, 1920, 400);
        mgr.push(p1);
        assert_eq!(mgr.current(), Some(&p1));
        mgr.pop();
        assert!(mgr.current().is_none());

        // Pane 2: volume
        let p2 = ScissorRect::new(0, 400, 1920, 300);
        mgr.push(p2);
        assert_eq!(mgr.current(), Some(&p2));
        mgr.pop();
        assert!(mgr.current().is_none());

        // Pane 3: RSI
        let p3 = ScissorRect::new(0, 700, 1920, 380);
        mgr.push(p3);
        assert_eq!(mgr.current(), Some(&p3));
        mgr.pop();
        assert!(mgr.current().is_none());
    }
}
