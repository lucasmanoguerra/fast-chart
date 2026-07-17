//! Zoom system for the chart interaction layer.
//!
//! Provides [`ZoomController`] which applies zoom operations to a [`Viewport`].
//! All zoom modes (wheel, pinch, box, axis-locked, animated, programmatic)
//! are implemented here, keeping the interaction engine focused on event
//! translation.

// ---------------------------------------------------------------------------
// Viewport
// ---------------------------------------------------------------------------

/// Viewport state that zoom operates on.
///
/// All coordinates are in data space (timestamps as `f64`, prices as `f64`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Left edge of the visible time range.
    pub time_start: f64,
    /// Right edge of the visible time range.
    pub time_end: f64,
    /// Bottom edge of the visible price range.
    pub price_min: f64,
    /// Top edge of the visible price range.
    pub price_max: f64,
}

impl Viewport {
    pub fn width(&self) -> f64 {
        self.time_end - self.time_start
    }

    pub fn height(&self) -> f64 {
        self.price_max - self.price_min
    }

    pub fn center_time(&self) -> f64 {
        (self.time_start + self.time_end) / 2.0
    }

    pub fn center_price(&self) -> f64 {
        (self.price_min + self.price_max) / 2.0
    }
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Zoom mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomMode {
    /// Standard wheel zoom at cursor position.
    Wheel,
    /// Pinch zoom centered between two fingers.
    Pinch,
    /// Zoom into a drag-selected box.
    Box,
    /// Zoom only the X (time) axis.
    AxisX,
    /// Zoom only the Y (price) axis.
    AxisY,
}

/// Axis lock for axis-locked zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisLock {
    /// Lock to the time (X) axis.
    Time,
    /// Lock to the price (Y) axis.
    Price,
}

/// A screen-space rectangle used for box zoom.
///
/// Coordinates are ratios (0.0–1.0) within the chart area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomRect {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

// ---------------------------------------------------------------------------
// Easing
// ---------------------------------------------------------------------------

/// Ease-in-out cubic for smooth animation.
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// ---------------------------------------------------------------------------
// ZoomController
// ---------------------------------------------------------------------------

/// Manages zoom operations on a [`Viewport`].
pub struct ZoomController {
    /// Minimum time range the viewport can display (in data units).
    min_time_range: f64,
    /// Maximum time range the viewport can display (in data units).
    max_time_range: f64,
    /// Animation target viewport (if animated zoom is active).
    animation_target: Option<Viewport>,
    /// Source viewport when animation started.
    animation_source: Option<Viewport>,
    /// Animation progress (0.0 = start, 1.0 = done).
    animation_progress: f32,
    /// Animation duration in seconds.
    animation_duration: f32,
}

impl ZoomController {
    /// Create a new controller with time range constraints.
    pub fn new(min_time_range: f64, max_time_range: f64) -> Self {
        Self {
            min_time_range,
            max_time_range,
            animation_target: None,
            animation_source: None,
            animation_progress: 1.0,
            animation_duration: 0.2,
        }
    }

    /// Apply a wheel zoom at cursor position.
    ///
    /// `factor > 1.0` zooms in, `factor < 1.0` zooms out.
    /// `cursor_x_ratio` is the cursor's horizontal position as 0.0–1.0.
    /// `cursor_y_ratio` is the cursor's vertical position as 0.0–1.0.
    pub fn wheel_zoom(
        &self,
        viewport: &mut Viewport,
        factor: f64,
        cursor_x_ratio: f64,
        cursor_y_ratio: f64,
    ) {
        let new_time_width = viewport.width() / factor;
        let new_price_height = viewport.height() / factor;

        let cursor_x_ratio = cursor_x_ratio.clamp(0.0, 1.0);
        let cursor_y_ratio = cursor_y_ratio.clamp(0.0, 1.0);

        viewport.time_start = viewport.time_start
            + cursor_x_ratio * (viewport.width() - new_time_width);
        viewport.time_end = viewport.time_start + new_time_width;

        viewport.price_max = viewport.price_max
            - cursor_y_ratio * (viewport.height() - new_price_height);
        viewport.price_min = viewport.price_max - new_price_height;

        self.clamp(viewport);
    }

    /// Apply pinch zoom at center.
    pub fn pinch_zoom(
        &self,
        viewport: &mut Viewport,
        scale: f64,
        center_x_ratio: f64,
        center_y_ratio: f64,
    ) {
        self.wheel_zoom(viewport, scale, center_x_ratio, center_y_ratio);
    }

    /// Apply box zoom: set viewport to the selected rectangle.
    ///
    /// `rect` coordinates are screen ratios (0.0–1.0).
    pub fn box_zoom(&self, viewport: &mut Viewport, rect: ZoomRect) {
        let x_min = rect.x1.min(rect.x2).clamp(0.0, 1.0);
        let x_max = rect.x1.max(rect.x2).clamp(0.0, 1.0);
        let y_min = rect.y1.min(rect.y2).clamp(0.0, 1.0);
        let y_max = rect.y1.max(rect.y2).clamp(0.0, 1.0);

        let full_width = viewport.width();
        let full_height = viewport.height();

        let new_time_start = viewport.time_start + x_min * full_width;
        let new_time_end = viewport.time_start + x_max * full_width;
        let new_price_max = viewport.price_max - y_min * full_height;
        let new_price_min = viewport.price_max - y_max * full_height;

        viewport.time_start = new_time_start;
        viewport.time_end = new_time_end;
        viewport.price_min = new_price_min;
        viewport.price_max = new_price_max;

        self.clamp(viewport);
    }

    /// Apply axis-locked zoom.
    ///
    /// Only modifies the specified axis; the other stays unchanged.
    pub fn axis_zoom(&self, viewport: &mut Viewport, factor: f64, axis: AxisLock, cursor_ratio: f64) {
        let cursor_ratio = cursor_ratio.clamp(0.0, 1.0);

        match axis {
            AxisLock::Time => {
                let new_width = viewport.width() / factor;
                viewport.time_start += cursor_ratio * (viewport.width() - new_width);
                viewport.time_end = viewport.time_start + new_width;
            }
            AxisLock::Price => {
                let new_height = viewport.height() / factor;
                viewport.price_max -= cursor_ratio * (viewport.height() - new_height);
                viewport.price_min = viewport.price_max - new_height;
            }
        }

        self.clamp(viewport);
    }

    /// Programmatic zoom: set exact viewport (clamped to valid ranges).
    pub fn programmatic_zoom(&self, viewport: &mut Viewport, target: Viewport) {
        viewport.time_start = target.time_start;
        viewport.time_end = target.time_end;
        viewport.price_min = target.price_min;
        viewport.price_max = target.price_max;
        self.clamp(viewport);
    }

    /// Start an animated zoom toward a target viewport.
    pub fn start_animated_zoom(&mut self, target: Viewport) {
        self.animation_source = None; // will be set on next tick
        self.animation_target = Some(target);
        self.animation_progress = 0.0;
    }

    /// Tick the animation. Returns `true` if animation is still in progress.
    pub fn tick_animation(&mut self, viewport: &mut Viewport, dt: f32) -> bool {
        let target = match self.animation_target {
            Some(t) => t,
            None => return false,
        };

        if self.animation_source.is_none() {
            self.animation_source = Some(*viewport);
        }

        if self.animation_progress >= 1.0 {
            *viewport = target;
            self.animation_target = None;
            self.animation_source = None;
            return false;
        }

        let source = self.animation_source.unwrap_or(*viewport);

        self.animation_progress += dt / self.animation_duration;
        if self.animation_progress > 1.0 {
            self.animation_progress = 1.0;
        }

        let t = ease_in_out_cubic(self.animation_progress) as f64;

        viewport.time_start = source.time_start + t * (target.time_start - source.time_start);
        viewport.time_end = source.time_end + t * (target.time_end - source.time_end);
        viewport.price_min = source.price_min + t * (target.price_min - source.price_min);
        viewport.price_max = source.price_max + t * (target.price_max - source.price_max);

        self.clamp(viewport);

        self.animation_progress < 1.0
    }

    /// Clamp viewport to valid time and price ranges.
    pub fn clamp(&self, viewport: &mut Viewport) {
        let width = viewport.time_end - viewport.time_start;
        if width < self.min_time_range {
            let center = viewport.center_time();
            viewport.time_start = center - self.min_time_range / 2.0;
            viewport.time_end = center + self.min_time_range / 2.0;
        } else if width > self.max_time_range {
            let center = viewport.center_time();
            viewport.time_start = center - self.max_time_range / 2.0;
            viewport.time_end = center + self.max_time_range / 2.0;
        }

        if viewport.time_start >= viewport.time_end {
            viewport.time_end = viewport.time_start + self.min_time_range;
        }

        if viewport.price_min >= viewport.price_max {
            viewport.price_max = viewport.price_min + 1.0;
        }
    }

    /// Get current zoom level (1.0 = default, >1.0 = zoomed in).
    pub fn zoom_level(&self, viewport: &Viewport, full_time_range: f64) -> f64 {
        if full_time_range <= 0.0 {
            return 1.0;
        }
        full_time_range / viewport.width()
    }
}

impl Default for ZoomController {
    fn default() -> Self {
        Self::new(60.0, 86_400.0 * 365.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;
    const EPS_F32: f32 = 1e-5;

    fn default_viewport() -> Viewport {
        Viewport {
            time_start: 0.0,
            time_end: 1000.0,
            price_min: 100.0,
            price_max: 200.0,
        }
    }

    fn controller() -> ZoomController {
        ZoomController::new(100.0, 10_000.0)
    }

    // --- Viewport ---

    #[test]
    fn viewport_width_height() {
        let vp = default_viewport();
        assert!((vp.width() - 1000.0).abs() < EPS);
        assert!((vp.height() - 100.0).abs() < EPS);
        assert!((vp.center_time() - 500.0).abs() < EPS);
        assert!((vp.center_price() - 150.0).abs() < EPS);
    }

    // --- Wheel zoom ---

    #[test]
    fn wheel_zoom_in() {
        let ctrl = controller();
        let mut vp = default_viewport();
        // Zoom in 2x at center (0.5).
        ctrl.wheel_zoom(&mut vp, 2.0, 0.5, 0.5);

        assert!((vp.width() - 500.0).abs() < EPS);
        assert!((vp.height() - 50.0).abs() < EPS);
        // Cursor position preserved: data point at 50% stays at 50%.
        assert!((vp.center_time() - 500.0).abs() < EPS);
        assert!((vp.center_price() - 150.0).abs() < EPS);
    }

    #[test]
    fn wheel_zoom_out() {
        let ctrl = controller();
        let mut vp = default_viewport();
        // Zoom out 2x at center.
        ctrl.wheel_zoom(&mut vp, 0.5, 0.5, 0.5);

        assert!((vp.width() - 2000.0).abs() < EPS);
        assert!((vp.height() - 200.0).abs() < EPS);
    }

    #[test]
    fn wheel_zoom_at_left_edge() {
        let ctrl = controller();
        let mut vp = default_viewport();
        ctrl.wheel_zoom(&mut vp, 2.0, 0.0, 0.5);

        assert!((vp.width() - 500.0).abs() < EPS);
        // At left edge, time_start should stay near original.
        assert!((vp.time_start - 0.0).abs() < EPS);
    }

    #[test]
    fn wheel_zoom_at_right_edge() {
        let ctrl = controller();
        let mut vp = default_viewport();
        ctrl.wheel_zoom(&mut vp, 2.0, 1.0, 0.5);

        assert!((vp.width() - 500.0).abs() < EPS);
        // At right edge, time_end should stay near original.
        assert!((vp.time_end - 1000.0).abs() < EPS);
    }

    // --- Pinch zoom ---

    #[test]
    fn pinch_zoom_in() {
        let ctrl = controller();
        let mut vp = default_viewport();
        ctrl.pinch_zoom(&mut vp, 2.0, 0.5, 0.5);

        assert!((vp.width() - 500.0).abs() < EPS);
        assert!((vp.height() - 50.0).abs() < EPS);
    }

    #[test]
    fn pinch_zoom_out() {
        let ctrl = controller();
        let mut vp = default_viewport();
        ctrl.pinch_zoom(&mut vp, 0.5, 0.5, 0.5);

        assert!((vp.width() - 2000.0).abs() < EPS);
        assert!((vp.height() - 200.0).abs() < EPS);
    }

    // --- Box zoom ---

    #[test]
    fn box_zoom() {
        let ctrl = controller();
        let mut vp = default_viewport();
        let rect = ZoomRect {
            x1: 0.2,
            y1: 0.25,
            x2: 0.8,
            y2: 0.75,
        };
        ctrl.box_zoom(&mut vp, rect);

        // time range should be 0.6 of original (1000 * 0.6 = 600).
        assert!((vp.width() - 600.0).abs() < EPS);
        // price range should be 0.5 of original (100 * 0.5 = 50).
        assert!((vp.height() - 50.0).abs() < EPS);
    }

    // --- Axis zoom ---

    #[test]
    fn axis_zoom_time() {
        let ctrl = controller();
        let mut vp = default_viewport();
        ctrl.axis_zoom(&mut vp, 2.0, AxisLock::Time, 0.5);

        assert!((vp.width() - 500.0).abs() < EPS);
        // Price axis unchanged.
        assert!((vp.height() - 100.0).abs() < EPS);
    }

    #[test]
    fn axis_zoom_price() {
        let ctrl = controller();
        let mut vp = default_viewport();
        ctrl.axis_zoom(&mut vp, 2.0, AxisLock::Price, 0.5);

        assert!((vp.height() - 50.0).abs() < EPS);
        // Time axis unchanged.
        assert!((vp.width() - 1000.0).abs() < EPS);
    }

    // --- Programmatic zoom ---

    #[test]
    fn programmatic_zoom() {
        let ctrl = controller();
        let mut vp = default_viewport();
        let target = Viewport {
            time_start: 500.0,
            time_end: 600.0,
            price_min: 120.0,
            price_max: 180.0,
        };
        ctrl.programmatic_zoom(&mut vp, target);

        assert!((vp.time_start - 500.0).abs() < EPS);
        assert!((vp.time_end - 600.0).abs() < EPS);
        assert!((vp.price_min - 120.0).abs() < EPS);
        assert!((vp.price_max - 180.0).abs() < EPS);
    }

    // --- Clamping ---

    #[test]
    fn zoom_clamp_min() {
        let ctrl = ZoomController::new(200.0, 10_000.0);
        let mut vp = default_viewport();
        // Try to zoom in way past min (width=1000, factor=10 → width=100 < 200).
        ctrl.wheel_zoom(&mut vp, 10.0, 0.5, 0.5);

        assert!(vp.width() >= 200.0 - EPS);
    }

    #[test]
    fn zoom_clamp_max() {
        let ctrl = ZoomController::new(100.0, 500.0);
        let mut vp = Viewport {
            time_start: 0.0,
            time_end: 300.0,
            price_min: 100.0,
            price_max: 200.0,
        };
        // Zoom out past max (width=300, factor=0.5 → width=600 > 500).
        ctrl.wheel_zoom(&mut vp, 0.5, 0.5, 0.5);

        assert!(vp.width() <= 500.0 + EPS);
    }

    // --- Animated zoom ---

    #[test]
    fn animated_zoom_completes() {
        let mut ctrl = controller();
        let mut vp = default_viewport();
        let target = Viewport {
            time_start: 100.0,
            time_end: 400.0,
            price_min: 120.0,
            price_max: 180.0,
        };
        ctrl.start_animated_zoom(target);

        // Simulate enough ticks to complete the 200ms animation.
        let mut still_running = true;
        for _ in 0..100 {
            still_running = ctrl.tick_animation(&mut vp, 0.01);
            if !still_running {
                break;
            }
        }

        assert!(!still_running);
        assert!((vp.time_start - 100.0).abs() < EPS);
        assert!((vp.time_end - 400.0).abs() < EPS);
        assert!((vp.price_min - 120.0).abs() < EPS);
        assert!((vp.price_max - 180.0).abs() < EPS);
    }

    // --- Zoom level ---

    #[test]
    fn zoom_level_calculation() {
        let ctrl = controller();
        let vp = default_viewport();
        let level = ctrl.zoom_level(&vp, 2000.0);

        // 2000 / 1000 = 2.0 (viewing half the data → 2x zoom).
        assert!((level - 2.0).abs() < EPS);
    }

    // --- Edge cases ---

    #[test]
    fn wheel_zoom_clamps_price_when_too_narrow() {
        let ctrl = controller();
        let mut vp = Viewport {
            time_start: 0.0,
            time_end: 1000.0,
            price_min: 150.0,
            price_max: 150.0001, // essentially zero height
        };
        ctrl.wheel_zoom(&mut vp, 10.0, 0.5, 0.5);

        assert!(vp.price_max > vp.price_min);
    }

    #[test]
    fn box_zoom_inverted_rect() {
        let ctrl = controller();
        let mut vp = default_viewport();
        let rect = ZoomRect {
            x1: 0.8,
            y1: 0.75,
            x2: 0.2,
            y2: 0.25,
        };
        ctrl.box_zoom(&mut vp, rect);

        assert!((vp.width() - 600.0).abs() < EPS);
        assert!((vp.height() - 50.0).abs() < EPS);
    }

    #[test]
    fn programmatic_zoom_clamps_too_small() {
        let ctrl = ZoomController::new(200.0, 10_000.0);
        let mut vp = default_viewport();
        let target = Viewport {
            time_start: 500.0,
            time_end: 550.0, // 50 < min_time_range of 200
            price_min: 100.0,
            price_max: 200.0,
        };
        ctrl.programmatic_zoom(&mut vp, target);

        assert!(vp.width() >= 200.0 - EPS);
    }
}
