use fast_chart_domain::bar::Bar;
use fast_chart_domain::scale::{LinearScale, TimeScale};
use fast_chart_domain::viewport::Viewport;

pub struct ViewportManager {
    min_zoom: f64,
    max_zoom: f64,
    min_visible_bars: usize,
    max_visible_bars: usize,
}

impl Default for ViewportManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewportManager {
    pub fn new() -> Self {
        Self {
            min_zoom: 0.01,
            max_zoom: 1000.0,
            min_visible_bars: 5,
            max_visible_bars: 10_000,
        }
    }

    /// Minimum zoom level.
    pub fn min_zoom(&self) -> f64 {
        self.min_zoom
    }

    /// Maximum zoom level.
    pub fn max_zoom(&self) -> f64 {
        self.max_zoom
    }

    /// Minimum number of visible bars.
    pub fn min_visible_bars(&self) -> usize {
        self.min_visible_bars
    }

    /// Maximum number of visible bars.
    pub fn max_visible_bars(&self) -> usize {
        self.max_visible_bars
    }

    /// Zoom the viewport around `center_time`, clamping to min/max zoom.
    pub fn apply_zoom(&self, viewport: &mut Viewport, factor: f64, center_time: u64) {
        let new_zoom = (viewport.zoom_level * factor).clamp(self.min_zoom, self.max_zoom);
        let clamped_factor = new_zoom / viewport.zoom_level;
        viewport.zoom(clamped_factor, center_time as f64);
    }

    /// Shift the viewport by `time_delta`, clamping to non-negative time.
    pub fn apply_pan(&self, viewport: &mut Viewport, time_delta: i64) {
        viewport.pan(time_delta);

        // Ensure the viewport doesn't go before time 0
        if viewport.time_start == 0 && time_delta < 0 {
            // Already clamped by saturating_sub in viewport.pan()
        }
    }

    /// Set the viewport to fit all bars.
    pub fn auto_fit(&self, viewport: &mut Viewport, bars: &[Bar]) {
        if bars.is_empty() {
            return;
        }

        let mut min_time = u64::MAX;
        let mut max_time = u64::MIN;
        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;

        for bar in bars {
            min_time = min_time.min(bar.timestamp);
            max_time = max_time.max(bar.timestamp);
            min_price = min_price.min(bar.low);
            max_price = max_price.max(bar.high);
        }

        // Add 5% padding to price range
        let price_range = max_price - min_price;
        let padding = if price_range > 0.0 {
            price_range * 0.05
        } else {
            1.0
        };

        viewport.time_start = min_time;
        viewport.time_end = max_time;
        viewport.value_min = min_price - padding;
        viewport.value_max = max_price + padding;
        viewport.zoom_level = 1.0;
    }

    /// Create a `TimeScale` mapping the viewport's time range to the canvas width.
    pub fn create_time_scale(&self, viewport: &Viewport, canvas_width: f64) -> TimeScale {
        TimeScale {
            start: viewport.time_start,
            end: viewport.time_end,
            width: canvas_width,
        }
    }

    /// Create a `LinearScale` mapping the viewport's value range to the canvas height.
    pub fn create_linear_scale(&self, viewport: &Viewport, canvas_height: f64) -> LinearScale {
        LinearScale {
            min: viewport.value_min,
            max: viewport.value_max,
            height: canvas_height,
        }
    }

    /// Convert a screen x-coordinate to a timestamp.
    pub fn screen_x_to_timestamp(&self, screen_x: f64, canvas_width: f64, viewport: &Viewport) -> f64 {
        if canvas_width < 1.0 {
            return viewport.time_start as f64;
        }
        let ratio = (screen_x / canvas_width).clamp(0.0, 1.0);
        viewport.time_start as f64 + ratio * (viewport.time_end as f64 - viewport.time_start as f64)
    }

    /// Convert a screen y-coordinate to a price value.
    pub fn screen_y_to_price(&self, screen_y: f64, canvas_height: f64, viewport: &Viewport) -> f64 {
        if canvas_height < 1.0 {
            return viewport.value_min;
        }
        let ratio = (screen_y / canvas_height).clamp(0.0, 1.0);
        viewport.value_min + ratio * (viewport.value_max - viewport.value_min)
    }

    /// Convert a timestamp to a screen x-coordinate.
    pub fn timestamp_to_screen_x(&self, timestamp: u64, canvas_width: f64, viewport: &Viewport) -> f64 {
        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        if time_range < f64::EPSILON {
            return 0.0;
        }
        let ratio = (timestamp as f64 - viewport.time_start as f64) / time_range;
        ratio * canvas_width
    }

    /// Convert a price to a screen y-coordinate.
    pub fn price_to_screen_y(&self, price: f64, canvas_height: f64, viewport: &Viewport) -> f64 {
        let value_range = viewport.value_max - viewport.value_min;
        if value_range < f64::EPSILON {
            return 0.0;
        }
        let ratio = (price - viewport.value_min) / value_range;
        (1.0 - ratio) * canvas_height // Y is inverted (0 at top)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_viewport() -> Viewport {
        Viewport {
            time_start: 1000,
            time_end: 2000,
            value_min: 90.0,
            value_max: 110.0,
            zoom_level: 1.0,
        }
    }

    #[test]
    fn new_has_sensible_defaults() {
        let vm = ViewportManager::new();
        assert_eq!(vm.min_zoom(), 0.01);
        assert_eq!(vm.max_zoom(), 1000.0);
        assert_eq!(vm.min_visible_bars(), 5);
        assert_eq!(vm.max_visible_bars(), 10_000);
    }

    #[test]
    fn apply_zoom_in() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_zoom(&mut vp, 2.0, 1500);
        let range = vp.time_end - vp.time_start;
        assert_eq!(range, 500); // halved from 1000
        assert_eq!(vp.zoom_level, 2.0);
    }

    #[test]
    fn apply_zoom_out() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_zoom(&mut vp, 0.5, 1500);
        assert_eq!(vp.zoom_level, 0.5);
    }

    #[test]
    fn apply_zoom_clamps_to_min() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vp.zoom_level = 0.02;
        vm.apply_zoom(&mut vp, 0.01, 1500);
        assert_eq!(vp.zoom_level, vm.min_zoom());
    }

    #[test]
    fn apply_zoom_clamps_to_max() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vp.zoom_level = 500.0;
        vm.apply_zoom(&mut vp, 10.0, 1500);
        assert_eq!(vp.zoom_level, vm.max_zoom());
    }

    #[test]
    fn apply_pan_forward() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_pan(&mut vp, 500);
        assert_eq!(vp.time_start, 1500);
        assert_eq!(vp.time_end, 2500);
    }

    #[test]
    fn apply_pan_backward() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_pan(&mut vp, -500);
        assert_eq!(vp.time_start, 500);
        assert_eq!(vp.time_end, 1500);
    }

    #[test]
    fn apply_pan_saturates_at_zero() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_pan(&mut vp, -5000);
        assert_eq!(vp.time_start, 0);
    }

    #[test]
    fn auto_fit_sets_range() {
        let vm = ViewportManager::new();
        let mut vp = Viewport::default();
        let bars = vec![
            Bar::new(100, 100.0, 105.0, 99.0, 102.0, 100).unwrap(),
            Bar::new(200, 101.0, 108.0, 100.0, 106.0, 200).unwrap(),
            Bar::new(300, 105.0, 110.0, 104.0, 109.0, 300).unwrap(),
        ];
        vm.auto_fit(&mut vp, &bars);

        assert_eq!(vp.time_start, 100);
        assert_eq!(vp.time_end, 300);
        assert_eq!(vp.zoom_level, 1.0);
        // Price range with 5% padding
        assert!(vp.value_min < 99.0);
        assert!(vp.value_max > 110.0);
    }

    #[test]
    fn auto_fit_empty_bars() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.auto_fit(&mut vp, &[]);
        // Viewport unchanged
        assert_eq!(vp.time_start, 1000);
        assert_eq!(vp.time_end, 2000);
    }

    #[test]
    fn auto_fit_single_bar() {
        let vm = ViewportManager::new();
        let mut vp = Viewport::default();
        let bars = vec![Bar::new(500, 100.0, 100.0, 100.0, 100.0, 100).unwrap()];
        vm.auto_fit(&mut vp, &bars);
        assert_eq!(vp.time_start, 500);
        assert_eq!(vp.time_end, 500);
    }

    #[test]
    fn create_time_scale_from_viewport() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ts = vm.create_time_scale(&vp, 800.0);
        assert_eq!(ts.start, 1000);
        assert_eq!(ts.end, 2000);
        assert_eq!(ts.width, 800.0);
    }

    #[test]
    fn create_linear_scale_from_viewport() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ls = vm.create_linear_scale(&vp, 400.0);
        assert_eq!(ls.min, 90.0);
        assert_eq!(ls.max, 110.0);
        assert_eq!(ls.height, 400.0);
    }

    #[test]
    fn zoom_preserves_center() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_zoom(&mut vp, 4.0, 1500);
        let mid = (vp.time_start + vp.time_end) / 2;
        assert_eq!(mid, 1500);
    }

    #[test]
    fn scale_roundtrip_after_auto_fit() {
        let vm = ViewportManager::new();
        let mut vp = Viewport::default();
        let bars = vec![
            Bar::new(0, 50.0, 60.0, 45.0, 55.0, 100).unwrap(),
            Bar::new(1000, 100.0, 110.0, 95.0, 105.0, 200).unwrap(),
        ];
        vm.auto_fit(&mut vp, &bars);
        let ts = vm.create_time_scale(&vp, 800.0);
        let ls = vm.create_linear_scale(&vp, 400.0);

        let x = ts.map_to_x(500);
        let back_time = ts.map_from_x(x);
        assert_eq!(back_time, 500);

        let y = ls.map_to_y(80.0);
        let back_price = ls.map_from_y(y);
        assert!((back_price - 80.0).abs() < 1e-10);
    }

    #[test]
    fn screen_x_to_timestamp_midpoint() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ts = vm.screen_x_to_timestamp(400.0, 800.0, &vp);
        assert_eq!(ts, 1500.0); // midpoint of 1000..2000
    }

    #[test]
    fn screen_x_to_timestamp_clamps_at_zero() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ts = vm.screen_x_to_timestamp(0.0, 800.0, &vp);
        assert_eq!(ts, 1000.0); // time_start
    }

    #[test]
    fn screen_x_to_timestamp_clamps_at_one() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ts = vm.screen_x_to_timestamp(800.0, 800.0, &vp);
        assert_eq!(ts, 2000.0); // time_end
    }

    #[test]
    fn screen_x_to_timestamp_small_canvas() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ts = vm.screen_x_to_timestamp(100.0, 0.0, &vp);
        assert_eq!(ts, 1000.0); // returns time_start for invalid canvas
    }

    #[test]
    fn screen_y_to_price_midpoint() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let price = vm.screen_y_to_price(200.0, 400.0, &vp);
        assert!((price - 100.0).abs() < 1e-10); // midpoint of 90..110
    }

    #[test]
    fn screen_y_to_price_top() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let price = vm.screen_y_to_price(0.0, 400.0, &vp);
        assert_eq!(price, 90.0); // value_min (screen top)
    }

    #[test]
    fn screen_y_to_price_bottom() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let price = vm.screen_y_to_price(400.0, 400.0, &vp);
        assert_eq!(price, 110.0); // value_max (screen bottom)
    }

    #[test]
    fn screen_y_to_price_small_canvas() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let price = vm.screen_y_to_price(100.0, 0.0, &vp);
        assert_eq!(price, 90.0); // returns value_min for invalid canvas
    }

    #[test]
    fn timestamp_to_screen_x_midpoint() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let x = vm.timestamp_to_screen_x(1500, 800.0, &vp);
        assert_eq!(x, 400.0); // midpoint
    }

    #[test]
    fn timestamp_to_screen_x_zero_range() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vp.time_end = vp.time_start; // zero range
        let x = vm.timestamp_to_screen_x(1000, 800.0, &vp);
        assert_eq!(x, 0.0);
    }

    #[test]
    fn price_to_screen_y_midpoint() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let y = vm.price_to_screen_y(100.0, 400.0, &vp);
        assert_eq!(y, 200.0); // midpoint, Y inverted
    }

    #[test]
    fn price_to_screen_y_top() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let y = vm.price_to_screen_y(110.0, 400.0, &vp);
        assert_eq!(y, 0.0); // max price → top of screen
    }

    #[test]
    fn price_to_screen_y_bottom() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let y = vm.price_to_screen_y(90.0, 400.0, &vp);
        assert_eq!(y, 400.0); // min price → bottom of screen
    }

    #[test]
    fn price_to_screen_y_zero_range() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vp.value_max = vp.value_min; // zero range
        let y = vm.price_to_screen_y(100.0, 400.0, &vp);
        assert_eq!(y, 0.0);
    }
}
