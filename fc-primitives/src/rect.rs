//! Axis-aligned bounding rectangle in screen space.
//!
//! This is the canonical `Rect` type for the entire workspace. It unifies
//! the former `Rect` (series rendering) and `ScreenRect` (dirty regions)
//! into a single value type at the primitives layer.

/// Axis-aligned bounding rectangle in screen space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle from position and dimensions.
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Full surface rect starting at origin.
    #[must_use]
    pub fn full(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    /// The right edge (x + width).
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// The bottom edge (y + height).
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// The center point.
    #[must_use]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Area of this rect.
    #[must_use]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check if a point is inside this rect.
    #[must_use]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    /// Check if this rect fully contains another rect.
    #[must_use]
    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Check if this rect intersects another.
    #[must_use]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Check if this rect intersects another, treating rects within 1px as adjacent.
    #[must_use]
    pub fn intersects_or_adjacent(&self, other: &Rect) -> bool {
        let epsilon = 1.0;
        self.x - epsilon < other.right()
            && self.right() + epsilon > other.x
            && self.y - epsilon < other.bottom()
            && self.bottom() + epsilon > other.y
    }

    /// Compute the union (bounding box) of two rects.
    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }

    /// Convert to wgpu scissor rect format: (x, y, w, h) in pixels, y-flipped.
    #[must_use]
    pub fn to_scissor(&self, surface_height: f32) -> (u32, u32, u32, u32) {
        let x = self.x.max(0.0) as u32;
        let y_flipped = (surface_height - self.bottom()).max(0.0) as u32;
        let w = self.width.max(0.0) as u32;
        let h = self.height.max(0.0) as u32;
        (x, y_flipped, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica rect_new
    #[test]
    fn rect_new() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 50.0);
    }

    // Clasificación: determinística — verifica rect_full
    #[test]
    fn rect_full() {
        let r = Rect::full(800.0, 600.0);
        assert_eq!(r, Rect::new(0.0, 0.0, 800.0, 600.0));
    }

    // Clasificación: determinística — verifica rect_contains
    #[test]
    fn rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(50.0, 50.0));
        assert!(r.contains(0.0, 0.0));
        assert!(!r.contains(100.0, 100.0));
    }

    // Clasificación: determinística — verifica rect_contains_rect
    #[test]
    fn rect_contains_rect() {
        let outer = Rect::new(0.0, 0.0, 200.0, 200.0);
        let inner = Rect::new(10.0, 10.0, 50.0, 50.0);
        assert!(outer.contains_rect(&inner));
        assert!(!inner.contains_rect(&outer));
    }

    // Clasificación: determinística — verifica rect_intersects
    #[test]
    fn rect_intersects() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));

        let c = Rect::new(200.0, 200.0, 100.0, 100.0);
        assert!(!a.intersects(&c));
    }

    // Clasificación: determinística — verifica rect_intersects_or_adjacent
    #[test]
    fn rect_intersects_or_adjacent() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(100.0, 0.0, 100.0, 100.0);
        assert!(!a.intersects(&b));
        assert!(a.intersects_or_adjacent(&b));
    }

    // Clasificación: determinística — verifica rect_union
    #[test]
    fn rect_union() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        assert_eq!(a.union(&b), Rect::new(0.0, 0.0, 150.0, 150.0));
    }

    // Clasificación: determinística — verifica rect_center
    #[test]
    fn rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 60.0);
        assert_eq!(r.center(), (50.0, 30.0));
    }

    // Clasificación: determinística — verifica rect_to_scissor
    #[test]
    fn rect_to_scissor() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        let (x, y, w, h) = r.to_scissor(600.0);
        assert_eq!(x, 10);
        assert_eq!(y, 530);
        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

    // Clasificación: determinística — verifica rect_area
    #[test]
    fn rect_area() {
        let r = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(r.area(), 5000.0);
    }

    // Clasificación: determinística — verifica rect_right_bottom
    #[test]
    fn rect_right_bottom() {
        let r = Rect::new(10.0, 20.0, 90.0, 80.0);
        assert_eq!(r.right(), 100.0);
        assert_eq!(r.bottom(), 100.0);
    }
}
