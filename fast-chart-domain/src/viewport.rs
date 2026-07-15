use crate::price_scale::PriceScale;

/// The visible time and value window of a chart pane.
///
/// A `Viewport` tracks the time range (`time_start`..`time_end`), the value
/// range (`value_min`..`value_max`), and the current zoom level. All rendering
/// maps this window to pixel coordinates.
///
/// # Examples
///
/// ```
/// use fast_chart_domain::Viewport;
///
/// let mut vp = Viewport {
///     time_start: 0,
///     time_end: 1000,
///     value_min: 0.0,
///     value_max: 100.0,
///     zoom_level: 1.0,
/// };
///
/// // Zoom in 2x centered at the midpoint
/// vp.zoom(2.0, 500.0);
/// assert_eq!((vp.time_start + vp.time_end) / 2, 500); // center preserved
///
/// // Pan forward
/// vp.pan(100);
/// assert!(vp.time_start > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Viewport {
    pub time_start: u64,
    pub time_end: u64,
    pub value_min: f64,
    pub value_max: f64,
    pub zoom_level: f64,
}

impl Viewport {
    pub fn contains_time(&self, time: u64) -> bool {
        time >= self.time_start && time <= self.time_end
    }

    pub fn zoom(&mut self, factor: f64, center: f64) {
        let time_range = self.time_end as f64 - self.time_start as f64;
        let new_range = time_range / factor;
        let center_ratio = (center - self.time_start as f64) / time_range;

        let new_start = center - new_range * center_ratio;
        let new_end = center + new_range * (1.0 - center_ratio);

        self.time_start = new_start.max(0.0) as u64;
        self.time_end = new_end.max(self.time_start as f64 + 1.0) as u64;
        self.zoom_level *= factor;
    }

    pub fn pan(&mut self, time_delta: i64) {
        if time_delta >= 0 {
            let delta = time_delta as u64;
            self.time_start = self.time_start.saturating_add(delta);
            self.time_end = self.time_end.saturating_add(delta);
        } else {
            let delta = (-time_delta) as u64;
            self.time_start = self.time_start.saturating_sub(delta);
            self.time_end = self.time_end.saturating_sub(delta);
        }
    }

    /// Map a price to a pixel y-coordinate using the given scale.
    ///
    /// Returns the y-coordinate in **screen pixels** from the top of the pane.
    /// Y is flipped: `y = 0` corresponds to `value_max` (top), `y = pane_height`
    /// corresponds to `value_min` (bottom).
    ///
    /// When the scale range is zero, returns `pane_height / 2`.
    pub fn price_to_y(&self, price: f64, scale: &PriceScale, pane_height: f32) -> f32 {
        let range = scale.value_max - scale.value_min;
        if range.abs() < f64::EPSILON {
            return pane_height / 2.0;
        }
        let ratio = (price - scale.value_min) / range;
        let clamped = ratio.clamp(0.0, 1.0);
        pane_height * (1.0 - clamped as f32) // y-flipped: top=0
    }

    /// Map a pixel y-coordinate back to a price using the given scale.
    ///
    /// Inverse of [`price_to_y`].
    pub fn y_to_price(&self, y: f32, scale: &PriceScale, pane_height: f32) -> f64 {
        if pane_height.abs() < f32::EPSILON {
            return (scale.value_min + scale.value_max) / 2.0;
        }
        let ratio = 1.0 - (y / pane_height);
        let clamped = ratio.clamp(0.0, 1.0);
        scale.value_min + clamped as f64 * (scale.value_max - scale.value_min)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            time_start: 0,
            time_end: 3600_000,
            value_min: 0.0,
            value_max: 100.0,
            zoom_level: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_viewport() -> Viewport {
        Viewport {
            time_start: 1000,
            time_end: 2000,
            value_min: 50.0,
            value_max: 150.0,
            zoom_level: 1.0,
        }
    }

    #[test]
    fn contains_time_inside() {
        let vp = test_viewport();
        assert!(vp.contains_time(1500));
    }

    #[test]
    fn contains_time_at_boundary() {
        let vp = test_viewport();
        assert!(vp.contains_time(1000));
        assert!(vp.contains_time(2000));
    }

    #[test]
    fn contains_time_outside() {
        let vp = test_viewport();
        assert!(!vp.contains_time(999));
        assert!(!vp.contains_time(2001));
    }

    #[test]
    fn zoom_in() {
        let mut vp = test_viewport();
        vp.zoom(2.0, 1500.0);
        let range = vp.time_end - vp.time_start;
        assert_eq!(range, 500); // halved from 1000
        assert_eq!(vp.zoom_level, 2.0);
    }

    #[test]
    fn zoom_out() {
        let mut vp = test_viewport();
        vp.zoom(0.5, 1500.0);
        assert_eq!(vp.zoom_level, 0.5);
    }

    #[test]
    fn pan_forward() {
        let mut vp = test_viewport();
        vp.pan(500);
        assert_eq!(vp.time_start, 1500);
        assert_eq!(vp.time_end, 2500);
    }

    #[test]
    fn pan_backward() {
        let mut vp = test_viewport();
        vp.pan(-500);
        assert_eq!(vp.time_start, 500);
        assert_eq!(vp.time_end, 1500);
    }

    #[test]
    fn pan_saturate_at_zero() {
        let mut vp = test_viewport();
        vp.pan(-2000);
        assert_eq!(vp.time_start, 0);
    }

    #[test]
    fn default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.zoom_level, 1.0);
        assert_eq!(vp.time_start, 0);
    }

    // --- price_to_y / y_to_price ---

    use crate::price_scale::{PriceScale, PriceScaleId, PriceScaleOptions};

    fn scale(min: f64, max: f64) -> PriceScale {
        PriceScale {
            id: PriceScaleId::Right,
            options: PriceScaleOptions::default(),
            value_min: min,
            value_max: max,
        }
    }

    #[test]
    fn price_to_y_midpoint() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let y = vp.price_to_y(150.0, &s, 400.0);
        assert!((y - 200.0).abs() < 0.001);
    }

    #[test]
    fn price_to_y_top() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let y = vp.price_to_y(200.0, &s, 400.0);
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn price_to_y_bottom() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let y = vp.price_to_y(100.0, &s, 400.0);
        assert!((y - 400.0).abs() < 0.001);
    }

    #[test]
    fn y_to_price_roundtrip() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let price = 150.0;
        let y = vp.price_to_y(price, &s, 400.0);
        let back = vp.y_to_price(y, &s, 400.0);
        assert!((back - price).abs() < f64::EPSILON);
    }

    #[test]
    fn price_to_y_zero_range() {
        let vp = Viewport::default();
        let s = scale(100.0, 100.0);
        let y = vp.price_to_y(100.0, &s, 400.0);
        assert!((y - 200.0).abs() < 0.001);
    }

    #[test]
    fn price_to_y_clamps_above() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let y = vp.price_to_y(300.0, &s, 400.0);
        // clamped to top
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn price_to_y_clamps_below() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let y = vp.price_to_y(50.0, &s, 400.0);
        // clamped to bottom
        assert!((y - 400.0).abs() < 0.001);
    }

    #[test]
    fn y_to_price_zero_height() {
        let vp = Viewport::default();
        let s = scale(100.0, 200.0);
        let price = vp.y_to_price(0.0, &s, 0.0);
        assert!((price - 150.0).abs() < 0.001);
    }
}
