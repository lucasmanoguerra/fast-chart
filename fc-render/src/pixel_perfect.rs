//! Pixel-perfect rendering utilities.
//!
//! Ensures lines, rectangles, and other primitives align to pixel centers
//! to avoid anti-aliasing blur on screen-aligned geometry.

use crate::coordinates::ScreenPoint;
use num_traits::Float;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Snap a floating-point coordinate to the nearest pixel centre.
///
/// On standard screens a pixel is at `(floor(x) + 0.5, floor(y) + 0.5)`, which
/// keeps 1 px wide lines crisp.  For Retina / high-DPI displays the logical
/// coordinate may already be scaled, so the same rule applies in **logical**
/// (device-independent) pixels.
///
/// # Examples
///
/// ```
/// use fc_render::pixel_perfect::PixelPerfect;
///
/// // Snap a value to the nearest pixel centre
/// let snapped = 3.2_f64.snap();
/// assert_eq!(snapped, 3.5);
///
/// // Sizes are rounded to whole pixels, not offset
/// let size = 4.7_f64.snap_size();
/// assert_eq!(size, 5.0);
///
/// // Floor and ceil for bounding box edges
/// assert_eq!(4.2_f64.floor_pixel(), 4.0);
/// assert_eq!(4.2_f64.ceil_pixel(), 5.0);
/// ```
pub trait PixelPerfect: Sized {
    /// Snap this value to the nearest pixel centre along one axis.
    fn snap(self) -> Self;

    /// Snap a size (width / height) to the nearest whole pixel.
    ///
    /// Sizes are **not** offset by 0.5 — only positions are.
    fn snap_size(self) -> Self;

    /// Round down to the nearest pixel boundary (useful for bounding boxes).
    fn floor_pixel(self) -> Self;

    /// Round up to the nearest pixel boundary.
    fn ceil_pixel(self) -> Self;
}

// ---------------------------------------------------------------------------
// f32 / f64 implementations
// ---------------------------------------------------------------------------

impl PixelPerfect for f32 {
    #[inline]
    fn snap(self) -> Self {
        (self.floor() + 0.5).max(0.0)
    }

    #[inline]
    fn snap_size(self) -> Self {
        self.round().max(0.0)
    }

    #[inline]
    fn floor_pixel(self) -> Self {
        self.floor().max(0.0)
    }

    #[inline]
    fn ceil_pixel(self) -> Self {
        self.ceil().max(0.0)
    }
}

impl PixelPerfect for f64 {
    #[inline]
    fn snap(self) -> Self {
        (self.floor() + 0.5).max(0.0)
    }

    #[inline]
    fn snap_size(self) -> Self {
        self.round().max(0.0)
    }

    #[inline]
    fn floor_pixel(self) -> Self {
        self.floor().max(0.0)
    }

    #[inline]
    fn ceil_pixel(self) -> Self {
        self.ceil().max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Generic numeric snap (num-traits)
// ---------------------------------------------------------------------------

/// Generic snap-to-pixel-centre for any float type via `num_traits::Float`.
///
/// Works with f32, f64, or any custom float type implementing `num_traits::Float`.
#[inline]
pub fn snap_generic<T: Float>(value: T) -> T {
    value.floor() + T::from(0.5).expect("0.5 is always representable in any Float type")
}

// ---------------------------------------------------------------------------
// Point / size helpers
// ---------------------------------------------------------------------------

/// Snap a `ScreenPoint` so both coordinates land on pixel centres.
#[inline]
pub fn snap_point(p: ScreenPoint) -> ScreenPoint {
    ScreenPoint {
        x: p.x.snap(),
        y: p.y.snap(),
    }
}

/// Return a `(x, y, width, height)` rectangle whose edges are pixel-aligned.
///
/// Positions are snapped to pixel centres; sizes are rounded to whole pixels.
/// The result is guaranteed to cover at least the same visual area (outward
/// snapping: floor for origin, ceil for bottom-right corner).
///
/// # Examples
///
/// ```
/// use fc_render::pixel_perfect::pixel_perfect_rect;
///
/// let (x, y, w, h) = pixel_perfect_rect(3.2, 5.7, 10.3, 20.9);
/// assert_eq!(x, 3.0);
/// assert_eq!(y, 5.0);
/// assert_eq!(w, 11.0);
/// assert_eq!(h, 22.0);
/// ```
#[inline]
pub fn pixel_perfect_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> (f64, f64, f64, f64) {
    let x1 = x.floor_pixel();
    let y1 = y.floor_pixel();
    let x2 = (x + width).ceil_pixel();
    let y2 = (y + height).ceil_pixel();
    (x1, y1, (x2 - x1).snap_size(), (y2 - y1).snap_size())
}

/// Ensure a line stays at least 1 px wide after snapping.
///
/// Both the start and the end of the line are snapped to pixel centres, and
/// a guard prevents the two from collapsing into a single point.
///
/// # Examples
///
/// ```
/// use fc_render::pixel_perfect::snap_line;
///
/// let (a, b) = snap_line(10.2, 50.7);
/// assert_eq!(a, 10.5);
/// assert_eq!(b, 50.5);
///
/// // Very short lines are kept to at least 1px
/// let (c, d) = snap_line(10.2, 10.3);
/// assert!(d - c >= 1.0);
/// ```
#[inline]
pub fn snap_line(start: f64, end: f64) -> (f64, f64) {
    let a = start.snap();
    let b = end.snap();
    if (b - a).abs() < 0.5 {
        (a, a + 1.0) // prevent collapsing
    } else {
        (a, b)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- f32 ---------------------------------------------------------------

    #[test]
    fn f32_snap_midpoint() {
        assert_eq!(3.2_f32.snap(), 3.5);
    }

    #[test]
    fn f32_snap_whole() {
        assert_eq!(4.0_f32.snap(), 4.5);
    }

    #[test]
    fn f32_snap_negative() {
        // negative values clamp to 0.0
        assert_eq!((-1.0_f32).snap(), 0.0);
    }

    #[test]
    fn f32_snap_size_rounds() {
        assert_eq!(3.7_f32.snap_size(), 4.0);
        assert_eq!(3.2_f32.snap_size(), 3.0);
    }

    #[test]
    fn f32_floor_ceil() {
        assert_eq!(3.7_f32.floor_pixel(), 3.0);
        assert_eq!(3.2_f32.ceil_pixel(), 4.0);
    }

    // --- f64 ---------------------------------------------------------------

    #[test]
    fn f64_snap_midpoint() {
        assert_eq!(3.2_f64.snap(), 3.5);
    }

    #[test]
    fn f64_snap_whole() {
        assert_eq!(4.0_f64.snap(), 4.5);
    }

    #[test]
    fn f64_snap_size_rounds() {
        assert_eq!(3.7_f64.snap_size(), 4.0);
        assert_eq!(3.2_f64.snap_size(), 3.0);
    }

    // --- helpers -----------------------------------------------------------

    #[test]
    fn pixel_perfect_rect_outward_snapping() {
        let (x, y, w, h) = pixel_perfect_rect(3.2, 5.7, 10.3, 20.9);
        assert_eq!(x, 3.0);
        assert_eq!(y, 5.0);
        // width covers [3.0 .. 14.0)  → 14.0 - 3.0 = 11.0
        assert_eq!(w, 11.0);
        // height covers [5.0 .. 27.0)  → 27.0 - 5.0 = 22.0
        assert_eq!(h, 22.0);
    }

    #[test]
    fn snap_line_normal() {
        let (a, b) = snap_line(10.2, 50.7);
        assert_eq!(a, 10.5);
        assert_eq!(b, 50.5); // 50.7.snap() → floor(50.7) + 0.5 = 50.5
    }

    #[test]
    fn snap_line_prevents_collapse() {
        // Without the guard these would both snap to 10.5 → zero width.
        let (a, b) = snap_line(10.2, 10.3);
        assert_eq!(a, 10.5);
        assert_eq!(b, 11.5);
    }

    #[test]
    fn snap_line_exact_one_pixel() {
        let (a, b) = snap_line(10.2, 11.3);
        assert_eq!(a, 10.5);
        assert_eq!(b, 11.5);
    }

    #[test]
    fn snap_point_centres_both_axes() {
        let p = snap_point(ScreenPoint { x: 3.2, y: 5.7 });
        assert_eq!(p.x, 3.5);
        assert_eq!(p.y, 5.5); // floor(5.7) + 0.5 = 5.5
    }
}
