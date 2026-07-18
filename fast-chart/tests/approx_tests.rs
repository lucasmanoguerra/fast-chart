//! Approximate floating-point tests using the `approx` crate.

use approx::assert_relative_eq;

#[test]
fn pixel_perfect_snap_approx() {
    use fast_chart::render::pixel_perfect::PixelPerfect;
    assert_relative_eq!(3.2_f64.snap(), 3.5);
    assert_relative_eq!(7.8_f64.snap(), 7.5);
    assert_relative_eq!(0.0_f64.snap(), 0.5);
}

#[test]
fn pixel_perfect_rect_approx() {
    use fast_chart::render::pixel_perfect::pixel_perfect_rect;
    let (x, y, w, h) = pixel_perfect_rect(3.2, 5.7, 10.3, 20.9);
    assert_relative_eq!(x, 3.0);
    assert_relative_eq!(y, 5.0);
    assert_relative_eq!(w, 11.0);
    assert_relative_eq!(h, 22.0);
}

#[test]
fn snap_generic_f64() {
    use fast_chart::render::pixel_perfect::snap_generic;
    assert_relative_eq!(snap_generic(3.2_f64), 3.5);
    assert_relative_eq!(snap_generic(0.0_f64), 0.5);
}

#[test]
fn snap_generic_f32() {
    use fast_chart::render::pixel_perfect::snap_generic;
    assert_relative_eq!(snap_generic(3.2_f32), 3.5);
    assert_relative_eq!(snap_generic(0.0_f32), 0.5);
}
