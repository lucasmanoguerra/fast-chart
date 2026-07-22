//! Pan system for the chart interaction layer.
//!
//! Provides [`PanController`] which manages all pan modes — drag, momentum,
//! inertia, auto-scroll, and follow-price.  The controller operates on a
//! [`Viewport`] and is completely decoupled from the rendering layer.

use crate::zoom::Viewport;

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
    Drag,
    Momentum,
    Inertia,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MOMENTUM_THRESHOLD: f64 = 100.0;
const STOP_THRESHOLD: f64 = 10.0;
const VELOCITY_SAMPLES: usize = 5;

// ---------------------------------------------------------------------------
// PanController
// ---------------------------------------------------------------------------

/// Manages pan operations on a [`Viewport`].
pub struct PanController {
    panning: bool,
    drag_start: Option<(f64, f64)>,
    last_position: Option<(f64, f64)>,
    recent_deltas: Vec<(f64, f64, f64)>, // (dx, dy, dt)
    velocity_x: f64,
    velocity_y: f64,
    friction: f64,
    auto_scroll: bool,
    follow_price: bool,
    last_price: f64,
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

        let time_delta = -self.velocity_x * dt / pixels_per_unit;
        let price_delta = self.velocity_y * dt / pixels_per_unit;

        viewport.time_start += time_delta;
        viewport.time_end += time_delta;
        viewport.price_min -= price_delta;
        viewport.price_max -= price_delta;

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

    /// Handle new data arrival — if auto-scrolling, adjust viewport.
    pub fn on_new_data(&self, viewport: &mut Viewport, latest_timestamp: u64) {
        if !self.auto_scroll { return; }
        let ts = latest_timestamp as f64;
        let width = viewport.width();
        let margin = width * 0.05;
        viewport.time_end = ts + margin;
        viewport.time_start = viewport.time_end - width;
    }

    /// Enable/disable follow-price.
    pub fn set_follow_price(&mut self, enabled: bool, last_price: f64) {
        self.follow_price = enabled;
        if enabled { self.last_price = last_price; }
    }

    /// Check if follow-price is active.
    pub fn is_following_price(&self) -> bool {
        self.follow_price
    }

    /// Get the follow-price level if active.
    pub fn follow_price_level(&self) -> Option<f64> {
        if self.follow_price { Some(self.last_price) } else { None }
    }

    /// Set friction for momentum mode (0.0–1.0).
    pub fn set_friction(&mut self, friction: f64) {
        self.friction = friction.clamp(0.0, 1.0);
    }

    /// Whether a drag is currently in progress.
    pub fn is_dragging(&self) -> bool {
        self.panning
    }

    // -- private helpers ----------------------------------------------------

    fn compute_velocity(&self) -> PanVelocity {
        if self.recent_deltas.is_empty() {
            return PanVelocity { vx: 0.0, vy: 0.0 };
        }
        let mut total_weight = 0.0;
        let mut vx = 0.0;
        let mut vy = 0.0;
        for (i, &(dx, dy, dt)) in self.recent_deltas.iter().enumerate() {
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
        Viewport { time_start: 0.0, time_end: 1000.0, price_min: 100.0, price_max: 200.0 }
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
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

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pan_preserves_range() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();
        ctrl.pan_by(&mut vp, 500.0, -20.0);
        assert!((vp.width() - 1000.0).abs() < EPS);
        assert!((vp.height() - 100.0).abs() < EPS);
    }

    // Clasificación: determinística — verifica start_drag
    #[test]
    fn start_drag() {
        let mut ctrl = PanController::new();
        assert!(!ctrl.is_dragging());
        ctrl.start_drag(100.0, 200.0);
        assert!(ctrl.is_dragging());
    }

    // Clasificación: determinística — verifica que update() avanza el tiempo y produce valor interpolado
    #[test]
    fn update_drag_returns_delta() {
        let mut ctrl = PanController::new();
        ctrl.start_drag(100.0, 100.0);
        let delta = ctrl.update_drag(150.0, 130.0);
        assert!((delta.dx - 50.0).abs() < EPS);
        assert!((delta.dy - 30.0).abs() < EPS);
    }

    // Clasificación: determinística — verifica end_drag_with_velocity
    #[test]
    fn end_drag_with_velocity() {
        let mut ctrl = PanController::new();
        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 {
            ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0);
        }
        let vel = ctrl.end_drag();
        assert!(vel.is_some());
        assert!(vel.unwrap().vx.abs() > MOMENTUM_THRESHOLD);
    }

    // Clasificación: determinística — verifica end_drag_slow
    #[test]
    fn end_drag_slow() {
        let mut ctrl = PanController::new();
        ctrl.start_drag(100.0, 100.0);
        ctrl.update_drag(100.5, 100.0);
        let vel = ctrl.end_drag();
        assert!(vel.is_none());
        assert!(!ctrl.is_dragging());
    }

    // Clasificación: determinística — verifica momentum_decelerates
    #[test]
    fn momentum_decelerates() {
        let mut ctrl = PanController::new();
        let mut vp = default_viewport();
        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 { ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0); }
        let _ = ctrl.end_drag();
        let initial_vx = ctrl.velocity_x;
        ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0);
        ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0);
        ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0);
        assert!(ctrl.velocity_x.abs() < initial_vx.abs());
    }

    // Clasificación: determinística — verifica momentum_stops
    #[test]
    fn momentum_stops() {
        let mut ctrl = PanController::new();
        let mut vp = default_viewport();
        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 { ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0); }
        let _ = ctrl.end_drag();
        for _ in 0..1000 {
            if !ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0) { break; }
        }
        assert!(!ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0));
    }

    // Clasificación: determinística — verifica friction_high_stops_fast
    #[test]
    fn friction_high_stops_fast() {
        let mut ctrl = PanController::new();
        ctrl.set_friction(0.5);
        let mut vp = default_viewport();
        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 { ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0); }
        let _ = ctrl.end_drag();
        let mut ticks = 0;
        for _ in 0..1000 {
            ticks += 1;
            if !ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0) { break; }
        }
        assert!(ticks < 200, "high friction should stop fast, took {ticks} ticks");
    }

    // Clasificación: determinística — verifica friction_low_stops_slow
    #[test]
    fn friction_low_stops_slow() {
        let mut ctrl = PanController::new();
        ctrl.set_friction(0.99);
        let mut vp = default_viewport();
        ctrl.start_drag(100.0, 100.0);
        for i in 1..=10 { ctrl.update_drag(100.0 + i as f64 * 20.0, 100.0); }
        let _ = ctrl.end_drag();
        let mut ticks = 0;
        for _ in 0..10_000 {
            ticks += 1;
            if !ctrl.tick_momentum(&mut vp, 1.0 / 60.0, 1.0) { break; }
        }
        assert!(ticks > 200, "low friction should take many ticks, took {ticks}");
    }

    // Clasificación: determinística — verifica auto_scroll_on_new_data
    #[test]
    fn auto_scroll_on_new_data() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();
        let mut ctrl = ctrl;
        ctrl.set_auto_scroll(true);
        ctrl.on_new_data(&mut vp, 5000);
        let width = 1000.0;
        let margin = width * 0.05;
        assert!((vp.time_end - (5000.0 + margin)).abs() < EPS);
        assert!((vp.width() - width).abs() < EPS);
    }

    // Clasificación: determinística — verifica auto_scroll_disabled_no_shift
    #[test]
    fn auto_scroll_disabled_no_shift() {
        let ctrl = PanController::new();
        let mut vp = default_viewport();
        let original = vp;
        ctrl.on_new_data(&mut vp, 5000);
        assert_eq!(vp.time_start, original.time_start);
        assert_eq!(vp.time_end, original.time_end);
    }

    // Clasificación: determinística — verifica follow_price_active
    #[test]
    fn follow_price_active() {
        let mut ctrl = PanController::new();
        ctrl.set_follow_price(true, 150.5);
        assert!(ctrl.is_following_price());
        assert_eq!(ctrl.follow_price_level(), Some(150.5));
    }

    // Clasificación: determinística — verifica follow_price_inactive
    #[test]
    fn follow_price_inactive() {
        let ctrl = PanController::new();
        assert!(!ctrl.is_following_price());
        assert_eq!(ctrl.follow_price_level(), None);
    }

    // Clasificación: determinística — verifica is_dragging_state
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
