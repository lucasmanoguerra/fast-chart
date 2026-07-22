use fc_primitives::bar::Bar;
use fc_primitives::scale::{LinearScale, TimeScale};
use fc_domain::viewport::Viewport;

use super::viewport_bounds::ViewportBounds;

pub struct ViewportManager {
    bounds: ViewportBounds,
}

impl Default for ViewportManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewportManager {
    pub fn new() -> Self {
        Self {
            bounds: ViewportBounds::new(),
        }
    }

    /// Minimum zoom level.
    pub fn min_zoom(&self) -> f64 {
        self.bounds.min_zoom
    }

    /// Maximum zoom level.
    pub fn max_zoom(&self) -> f64 {
        self.bounds.max_zoom
    }

    /// Minimum number of visible bars.
    pub fn min_visible_bars(&self) -> usize {
        self.bounds.min_visible_bars
    }

    /// Maximum number of visible bars.
    pub fn max_visible_bars(&self) -> usize {
        self.bounds.max_visible_bars
    }

    /// Zoom the viewport around `center_time`, clamping to min/max zoom.
    pub fn apply_zoom(&self, viewport: &mut Viewport, factor: f64, center_time: u64) {
        let new_zoom = self.bounds.clamp_zoom(viewport.zoom_level, factor);
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
            bar_spacing: 8.0,
            right_offset: 0.0,
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

    // Clasificación: determinística — verifica que el constructor produce valores por defecto consistentes
    #[test]
    fn new_has_sensible_defaults() {
        let vm = ViewportManager::new();
        assert_eq!(vm.min_zoom(), 0.01);
        assert_eq!(vm.max_zoom(), 1000.0);
        assert_eq!(vm.min_visible_bars(), 5);
        assert_eq!(vm.max_visible_bars(), 10_000);
    }

    // Clasificación: determinística — verifica apply_zoom_in
    #[test]
    fn apply_zoom_in() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_zoom(&mut vp, 2.0, 1500);
        let range = vp.time_end - vp.time_start;
        assert_eq!(range, 500); // halved from 1000
        assert_eq!(vp.zoom_level, 2.0);
    }

    // Clasificación: determinística — verifica apply_zoom_out
    #[test]
    fn apply_zoom_out() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_zoom(&mut vp, 0.5, 1500);
        assert_eq!(vp.zoom_level, 0.5);
    }

    // Clasificación: determinística — verifica apply_zoom_clamps_to_min
    #[test]
    fn apply_zoom_clamps_to_min() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vp.zoom_level = 0.02;
        vm.apply_zoom(&mut vp, 0.01, 1500);
        assert_eq!(vp.zoom_level, vm.min_zoom());
    }

    // Clasificación: determinística — verifica apply_zoom_clamps_to_max
    #[test]
    fn apply_zoom_clamps_to_max() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vp.zoom_level = 500.0;
        vm.apply_zoom(&mut vp, 10.0, 1500);
        assert_eq!(vp.zoom_level, vm.max_zoom());
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn apply_pan_forward() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_pan(&mut vp, 500);
        assert_eq!(vp.time_start, 1500);
        assert_eq!(vp.time_end, 2500);
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn apply_pan_backward() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_pan(&mut vp, -500);
        assert_eq!(vp.time_start, 500);
        assert_eq!(vp.time_end, 1500);
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn apply_pan_saturates_at_zero() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_pan(&mut vp, -5000);
        assert_eq!(vp.time_start, 0);
    }

    // Clasificación: determinística — verifica auto_fit_sets_range
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

    // Clasificación: determinística — verifica auto_fit_empty_bars
    #[test]
    fn auto_fit_empty_bars() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.auto_fit(&mut vp, &[]);
        // Viewport unchanged
        assert_eq!(vp.time_start, 1000);
        assert_eq!(vp.time_end, 2000);
    }

    // Clasificación: determinística — verifica auto_fit_single_bar
    #[test]
    fn auto_fit_single_bar() {
        let vm = ViewportManager::new();
        let mut vp = Viewport::default();
        let bars = vec![Bar::new(500, 100.0, 100.0, 100.0, 100.0, 100).unwrap()];
        vm.auto_fit(&mut vp, &bars);
        assert_eq!(vp.time_start, 500);
        assert_eq!(vp.time_end, 500);
    }

    // Clasificación: determinística — verifica create_time_scale_from_viewport
    #[test]
    fn create_time_scale_from_viewport() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ts = vm.create_time_scale(&vp, 800.0);
        assert_eq!(ts.start, 1000);
        assert_eq!(ts.end, 2000);
        assert_eq!(ts.width, 800.0);
    }

    // Clasificación: determinística — verifica create_linear_scale_from_viewport
    #[test]
    fn create_linear_scale_from_viewport() {
        let vm = ViewportManager::new();
        let vp = test_viewport();
        let ls = vm.create_linear_scale(&vp, 400.0);
        assert_eq!(ls.min, 90.0);
        assert_eq!(ls.max, 110.0);
        assert_eq!(ls.height, 400.0);
    }

    // Clasificación: determinística — verifica zoom_preserves_center
    #[test]
    fn zoom_preserves_center() {
        let vm = ViewportManager::new();
        let mut vp = test_viewport();
        vm.apply_zoom(&mut vp, 4.0, 1500);
        let mid = (vp.time_start + vp.time_end) / 2;
        assert_eq!(mid, 1500);
    }

    // Clasificación: determinística — verifica scale_roundtrip_after_auto_fit
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

}
