//! Pan system for the chart interaction layer.
//!
//! Provides [`PanController`] which manages all pan modes — drag, momentum,
//! inertia, auto-scroll, and follow-price.  The controller operates on a
//! [`Viewport`] and is completely decoupled from the rendering layer.

use super::zoom::Viewport;

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Delta in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanDelta {
    pub dx: f64,
    pub dy: f64,
}

/// Velocity in pixels per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanVelocity {
    pub vx: f64,
    pub vy: f64,
}

/// Pan mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanMode {
    /// Standard drag pan.
    Drag,
    /// Momentum-based fling after drag release.
    Momentum,
    /// Inertia-based deceleration (configurable friction).
    Inertia,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum velocity (px/s) required to trigger momentum after drag ends.
const MOMENTUM_THRESHOLD: f64 = 100.0;

/// Minimum velocity (px/s) below which momentum is considered stopped.
const STOP_THRESHOLD: f64 = 10.0;

/// Number of recent samples used to compute release velocity.
const VELOCITY_SAMPLES: usize = 5;

// ---------------------------------------------------------------------------
// PanController
// ---------------------------------------------------------------------------

/// Manages pan operations on a [`Viewport`].
pub struct PanController {
    /// Whether panning is currently active (mouse down / drag in progress).
    panning: bool,
    /// Drag start position (screen coords).
    drag_start: Option<(f64, f64)>,
    /// Previous mouse position (for velocity calculation).
    last_position: Option<(f64, f64)>,
    /// Recent frame deltas for velocity averaging (screen pixels).
    recent_deltas: Vec<(f64, f64, f64)>, // (dx, dy, dt)
    /// Current velocity (pixels per second).
    velocity_x: f64,
    velocity_y: f64,
    /// Friction for momentum/inertia (0.0 = stop immediately, 1.0 = no friction).
    friction: f64,
    /// Whether auto-scroll is enabled.
    auto_scroll: bool,
    /// Whether follow-price is enabled.
    follow_price: bool,
    /// The last known price (for follow-price mode).
    last_price: f64,
    /// Current pan mode.
    mode: PanMode,
}

impl PanController {
    /// Create a new controller with default settings.
    pub fn new() -> Self {
        Self {
            panning: false,
            drag_start: None,
            last_position: None,
            recent_deltas: Vec::with_capacity(VELOCITY_SAMPLES),
            velocity_x: 0.0,
            velocity_y: 0.0,
            friction: 0.95,
            auto_scroll: false,
            follow_price: false,
            last_price: 0.0,
            mode: PanMode::Drag,
        }
    }

    /// Start a drag pan.
    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.panning = true;
        self.drag_start = Some((x, y));
        self.last_position = Some((x, y));
        self.recent_deltas.clear();
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
        self.mode = PanMode::Drag;
    }

    /// Update drag position. Returns the delta in screen pixels.
    pub fn update_drag(&mut self, x: f64, y: f64) -> PanDelta {
        if !self.panning {
            return PanDelta { dx: 0.0, dy: 0.0 };
        }

        let (lx, ly) = self.last_position.unwrap_or((x, y));
        let dx = x - lx;
        let dy = y - ly;

        self.last_position = Some((x, y));

        // Accumulate for velocity calculation (dt=1 frame approximated as 1/60).
        if self.recent_deltas.len() >= VELOCITY_SAMPLES {
            self.recent_deltas.remove(0);
        }
        self.recent_deltas.push((dx, dy, 1.0 / 60.0));

        PanDelta { dx, dy }
    }

    /// End drag. If velocity is above threshold, enters momentum mode.
    pub fn end_drag(&mut self) -> Option<PanVelocity> {
        if !self.panning {
            return None;
        }

        self.panning = false;
        self.drag_start = None;
        self.last_position = None;

        let vel = self.compute_velocity();

        if vel.vx.abs() > MOMENTUM_THRESHOLD || vel.vy.abs() > MOMENTUM_THRESHOLD {
            self.velocity_x = vel.vx;
            self.velocity_y = vel.vy;
            self.mode = PanMode::Momentum;
            Some(vel)
        } else {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            self.recent_deltas.clear();
            None
        }
    }

    /// Apply momentum tick. Returns true if still moving.
    pub fn tick_momentum(&mut self, viewport: &mut Viewport, dt: f64, pixels_per_unit: f64) -> bool {
        if self.mode != PanMode::Momentum {
            return false;
        }

        if pixels_per_unit <= 0.0 {
            self.mode = PanMode::Drag;
            return false;
        }

        let speed = (self.velocity_x.powi(2) + self.velocity_y.powi(2)).sqrt();
        if speed < STOP_THRESHOLD {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            self.mode = PanMode::Drag;
            return false;
        }

        // Convert pixel velocity to data-space delta.
        // Negative screen dx → positive time delta (dragging left shows earlier data).
        let time_delta = -self.velocity_x * dt / pixels_per_unit;
        let price_delta = self.velocity_y * dt / pixels_per_unit;

        viewport.time_start += time_delta;
        viewport.time_end += time_delta;
        viewport.price_min -= price_delta;
        viewport.price_max -= price_delta;

        // Apply friction.
        let friction_factor = self.friction.clamp(0.0, 1.0);
        self.velocity_x *= friction_factor;
        self.velocity_y *= friction_factor;

        true
    }

    /// Apply a discrete pan by time/price delta.
    pub fn pan_by(&self, viewport: &mut Viewport, time_delta: f64, price_delta: f64) {
        viewport.time_start += time_delta;
        viewport.time_end += time_delta;
        viewport.price_min -= price_delta;
        viewport.price_max -= price_delta;
    }

    /// Enable/disable auto-scroll.
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }

    /// Check if auto-scroll is active.
    pub fn is_auto_scrolling(&self) -> bool {
        self.auto_scroll
    }

    /// Handle new data arrival — if auto-scrolling, adjust viewport to show latest.
    pub fn on_new_data(&self, viewport: &mut Viewport, latest_timestamp: u64) {
        if !self.auto_scroll {
            return;
        }

        let ts = latest_timestamp as f64;
        let width = viewport.width();

        // Place the latest bar near the right edge with a small margin.
        let margin = width * 0.05;
        viewport.time_end = ts + margin;
        viewport.time_start = viewport.time_end - width;
    }

    /// Enable/disable follow-price.
    pub fn set_follow_price(&mut self, enabled: bool, last_price: f64) {
        self.follow_price = enabled;
        if enabled {
            self.last_price = last_price;
        }
    }

    /// Check if follow-price is active.
    pub fn is_following_price(&self) -> bool {
        self.follow_price
    }

    /// Get the follow-price Y position if active.
    pub fn follow_price_level(&self) -> Option<f64> {
        if self.follow_price {
            Some(self.last_price)
        } else {
            None
        }
    }

    /// Set friction for momentum mode.
    pub fn set_friction(&mut self, friction: f64) {
        self.friction = friction.clamp(0.0, 1.0);
    }

    /// Whether a drag is currently in progress.
    pub fn is_dragging(&self) -> bool {
        self.panning
    }

    // -- private helpers -----------------------------------------------------

    /// Compute weighted average velocity from recent samples.
    fn compute_velocity(&self) -> PanVelocity {
        if self.recent_deltas.is_empty() {
            return PanVelocity { vx: 0.0, vy: 0.0 };
        }

        let mut total_weight = 0.0;
        let mut vx = 0.0;
        let mut vy = 0.0;

        for (i, &(dx, dy, dt)) in self.recent_deltas.iter().enumerate() {
            // More recent samples get higher weight.
            let weight = (i + 1) as f64;
            let inv_dt = if dt > 0.0 { 1.0 / dt } else { 0.0 };
            vx += dx * inv_dt * weight;
            vy += dy * inv_dt * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            vx /= total_weight;
            vy /= total_weight;
        }

        PanVelocity { vx, vy }
    }
}

impl Default for PanController {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    fn default_viewport() -> Viewport {
        Viewport {
            time_start: 0.0,
            time_end: 1000.0,
            price_min: 100.0,
            price_max: 200.0,
        }
    }

    // 1. pan_by_basic — pan by time and price deltas
    #[test]
    fn pan_by_basic() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();

        ctrl.pan_by(&mut vp, 100.0, 10.0);

        assert!((vp.time_start - 100.0).abs() < EPS);
        assert!((vp.time_end - 1100.0).abs() < EPS);
        assert!((vp.price_min - 90.0).abs() < EPS);
        assert!((vp.price_max - 190.0).abs() < EPS);
    }

    // 2. pan_preserves_range — panning doesn't change viewport size
    #[test]
    fn pan_preserves_range() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();

        ctrl.pan_by(&mut vp, 500.0, -20.0);

        assert!((vp.width() - 1000.0).abs() < EPS);
        assert!((vp.height() - 100.0).abs() < EPS);
    }

    // 3. start_drag — sets dragging state
    #[test]
    fn start_drag() {
        let mut ctrl = PanController::new();
        assert!(!ctrl.is_dragging());

        ctrl.start_drag(100.0, 200.0);
        assert!(ctrl.is_dragging());
    }

    // 4. update_drag_returns_delta — delta is correct
    #[test]
    fn update_drag_returns_delta() {
        let mut ctrl = PanController::new();
        ctrl.start_drag(100.0, 100.0);

        let delta = ctrl.update_drag(150.0, 130.0);
        assert!((delta.dx - 50.0).abs() < EPS);
        assert!((delta.dy - 30.0).abs() < EPS);
    }

    // 5. end_drag_with_velocity — enters momentum when fast
    #[test]
    fn end_drag_with_velocity() {
        let mut ctrl = PanController::new();
        ctrl.start_drag(100.0, 100.0);

        // Simulate fast motion — several frames.
        for i in 1..=10 {
            ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0);
        }

        let vel = ctrl.end_drag();
        assert!(vel.is_some());
        let vel = vel.unwrap();
        // Should have significant horizontal velocity.
        assert!(vel.vx.abs() > MOMENTUM_THRESHOLD);
    }

    // 6. end_drag_slow — no momentum when slow
    #[test]
    fn end_drag_slow() {
        let mut ctrl = PanController::new();
        ctrl.start_drag(100.0, 100.0);

        // Tiny movement.
        ctrl.update_drag(100.5, 100.0);

        let vel = ctrl.end_drag();
        assert!(vel.is_none());
        assert!(!ctrl.is_dragging());
    }

    // 7. momentum_decelerates — velocity decreases over time
    #[test]
    fn momentum_decelerates() {
        let mut ctrl = PanController::new();
        let mut vp = default_viewport();

        // Manually enter momentum mode.
        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 {
            ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0);
        }
        let _ = ctrl.end_drag();

        let initial_vx = ctrl.velocity_x;

        // Tick a few times.
        ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0);
        ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0);
        ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0);

        assert!(ctrl.velocity_x.abs() < initial_vx.abs());
    }

    // 8. momentum_stops — eventually stops
    #[test]
    fn momentum_stops() {
        let mut ctrl = PanController::new();
        let mut vp = default_viewport();

        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 {
            ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0);
        }
        let _ = ctrl.end_drag();

        // Tick many times until stopped.
        for _ in 0..1000 {
            if !ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0) {
                break;
            }
        }

        assert!(!ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0));
    }

    // 9. friction_high_stops_fast — high friction = quick stop
    #[test]
    fn friction_high_stops_fast() {
        let mut ctrl = PanController::new();
        ctrl.set_friction(0.5);
        let mut vp = default_viewport();

        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 {
            ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0);
        }
        let _ = ctrl.end_drag();

        let mut ticks = 0;
        for _ in 0..1000 {
            ticks += 1;
            if !ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0) {
                break;
            }
        }

        assert!(ticks < 200, "high friction should stop fast, took {ticks} ticks");
    }

    // 10. friction_low_stops_slow — low friction = slow stop
    #[test]
    fn friction_low_stops_slow() {
        let mut ctrl = PanController::new();
        ctrl.set_friction(0.99);
        let mut vp = default_viewport();

        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 {
            ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0);
        }
        let _ = ctrl.end_drag();

        let mut ticks = 0;
        for _ in 0..10_000 {
            ticks += 1;
            if !ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0) {
                break;
            }
        }

        // Low friction should take longer than high friction.
        assert!(ticks > 200, "low friction should take many ticks, took {ticks}");
    }

    // 11. auto_scroll_on_new_data — viewport shifts to latest
    #[test]
    fn auto_scroll_on_new_data() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();

        // Enable auto-scroll via mutation.
        let mut ctrl = ctrl;
        ctrl.set_auto_scroll(true);

        ctrl.on_new_data(&mut vp, 5000);

        let width = 1000.0;
        let margin = width * 0.05;
        assert!((vp.time_end - (5000.0 + margin)).abs() < EPS);
        assert!((vp.width() - width).abs() < EPS);
    }

    // 12. auto_scroll_disabled_no_shift — viewport unchanged when disabled
    #[test]
    fn auto_scroll_disabled_no_shift() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();
        let original = vp;

        ctrl.on_new_data(&mut vp, 5000);

        assert_eq!(vp.time_start, original.time_start);
        assert_eq!(vp.time_end, original.time_end);
    }

    // 13. follow_price_active — returns price level
    #[test]
    fn follow_price_active() {
        let mut ctrl = PanController::new();
        ctrl.set_follow_price(true, 150.5);

        assert!(ctrl.is_following_price());
        assert_eq!(ctrl.follow_price_level(), Some(150.5));
    }

    // 14. follow_price_inactive — returns None
    #[test]
    fn follow_price_inactive() {
        let ctrl = PanController::new();

        assert!(!ctrl.is_following_price());
        assert_eq!(ctrl.follow_price_level(), None);
    }

    // 15. is_dragging_state — correct during drag lifecycle
    #[test]
    fn is_dragging_state() {
        let mut ctrl = PanController::new();
        assert!(!ctrl.is_dragging());

        ctrl.start_drag(0.0, 0.0);
        assert!(ctrl.is_dragging());

        ctrl.update_drag(10.0, 10.0);
        assert!(ctrl.is_dragging());

        let _ = ctrl.end_drag();
        assert!(!ctrl.is_dragging());
    }
}
