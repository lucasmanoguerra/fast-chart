/// Scroll mode for kinetic motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollMode {
    /// Momentum: starts fast, decelerates by friction multiplier per step.
    Momentum,
    /// Inertia: starts fast, decelerates exponentially over time.
    Inertia,
}

/// Snap target for kinetic scroll.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapTarget {
    /// Snap to the nearest bar index.
    BarIndex,
    /// Snap to a specific price level.
    Price(f64),
    /// No snap.
    None,
}

/// Kinetic scroll state for smooth momentum-based scrolling.
///
/// Supports two modes:
/// - `Momentum`: velocity *= friction per frame (backward-compatible).
/// - `Inertia`: exponential decay `v(t) = v0 * e^(-decay_rate * t)`.
///
/// # Examples
///
/// ```
/// use fast_chart_domain::KineticScroll;
///
/// let mut ks = KineticScroll::new(0.95);
/// assert!(!ks.is_active());
///
/// // Start scrolling with initial velocity
/// ks.start(10.0);
/// assert!(ks.is_active());
///
/// // Update decelerates over time
/// let displacement = ks.update();
/// assert!(displacement > 0.0);
/// assert!(ks.velocity() < 10.0); // velocity decreased
/// ```
#[derive(Debug, Clone)]
pub struct KineticScroll {
    /// Current velocity (pixels per frame).
    velocity: f64,
    /// Friction coefficient (0.0 = no friction, 1.0 = instant stop).
    friction: f64,
    /// Minimum velocity threshold to stop.
    threshold: f64,
    /// Whether kinetic scrolling is active.
    active: bool,
    /// Scroll mode (momentum or inertia).
    mode: ScrollMode,
    /// Snap target (if any).
    snap_target: Option<SnapTarget>,
    /// Current time elapsed since start (for velocity decay).
    elapsed: f64,
    /// Velocity decay rate per second (exponential decay).
    decay_rate: f64,
    /// Initial velocity at start (used by inertia mode).
    initial_velocity: f64,
}

impl Default for KineticScroll {
    fn default() -> Self {
        Self {
            velocity: 0.0,
            friction: 0.95,
            threshold: 0.1,
            active: false,
            mode: ScrollMode::Momentum,
            snap_target: None,
            elapsed: 0.0,
            decay_rate: 2.0,
            initial_velocity: 0.0,
        }
    }
}

impl KineticScroll {
    /// Create a new kinetic scroll with given friction.
    pub fn new(friction: f64) -> Self {
        Self {
            velocity: 0.0,
            friction: friction.clamp(0.0, 1.0),
            threshold: 0.1,
            active: false,
            mode: ScrollMode::Momentum,
            snap_target: None,
            elapsed: 0.0,
            decay_rate: 2.0,
            initial_velocity: 0.0,
        }
    }

    /// Create with specific scroll mode.
    pub fn with_mode(mode: ScrollMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// Start a scroll with initial velocity.
    pub fn start(&mut self, initial_velocity: f64) {
        self.velocity = initial_velocity;
        self.initial_velocity = initial_velocity;
        self.active = true;
        self.elapsed = 0.0;
    }

    /// Update velocity for one frame (frame-based). Returns the displacement for this frame.
    ///
    /// Momentum mode: `velocity *= friction` each call.
    /// Inertia mode: velocity decays using elapsed time tracked internally with a
    ///   1-frame delta approximation.
    pub fn update(&mut self) -> f64 {
        if !self.active {
            return 0.0;
        }

        let displacement = match self.mode {
            ScrollMode::Momentum => {
                let d = self.velocity;
                self.velocity *= self.friction;
                d
            }
            ScrollMode::Inertia => {
                // Approximate dt as 1 frame for frame-based usage
                self.elapsed += 1.0 / 60.0;
                let decay = (-self.decay_rate * self.elapsed).exp();
                let current = self.initial_velocity * decay;
                let d = self.velocity;
                self.velocity = current;
                d
            }
        };

        if self.velocity.abs() < self.threshold {
            self.velocity = 0.0;
            self.active = false;
        }

        displacement
    }

    /// Update with time delta (dt in seconds). Returns displacement.
    ///
    /// Momentum mode: displacement = velocity * dt, then velocity *= friction.
    /// Inertia mode: uses `v(t) = v0 * e^(-decay_rate * t)`.
    pub fn update_dt(&mut self, dt: f64) -> f64 {
        if !self.active || dt <= 0.0 {
            return 0.0;
        }

        self.elapsed += dt;

        let displacement = match self.mode {
            ScrollMode::Momentum => {
                let d = self.velocity * dt;
                self.velocity *= self.friction;
                d
            }
            ScrollMode::Inertia => {
                let decay = (-self.decay_rate * self.elapsed).exp();
                let current = self.initial_velocity * decay;
                let d = self.velocity * dt;
                self.velocity = current;
                d
            }
        };

        if self.velocity.abs() < self.threshold {
            self.velocity = 0.0;
            self.active = false;
        }

        displacement
    }

    /// Stop all motion.
    pub fn stop(&mut self) {
        self.velocity = 0.0;
        self.active = false;
    }

    /// Check if kinetic scrolling is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get current velocity.
    pub fn velocity(&self) -> f64 {
        self.velocity
    }

    /// Set friction.
    pub fn set_friction(&mut self, friction: f64) {
        self.friction = friction.clamp(0.0, 1.0);
    }

    /// Set threshold.
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }

    /// Set velocity decay rate (per second). Higher = faster stop.
    pub fn set_decay_rate(&mut self, rate: f64) {
        self.decay_rate = rate;
    }

    /// Set snap target.
    pub fn set_snap_target(&mut self, target: Option<SnapTarget>) {
        self.snap_target = target;
    }

    /// Check if should snap (velocity below threshold and snap_target is set).
    pub fn should_snap(&self) -> bool {
        !self.active && self.snap_target.is_some()
    }

    /// Get the current scroll mode.
    pub fn mode(&self) -> ScrollMode {
        self.mode
    }

    /// Get total elapsed time.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Reset elapsed time.
    pub fn reset_elapsed(&mut self) {
        self.elapsed = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Existing tests (backward-compatible)
    // -----------------------------------------------------------------------

    #[test]
    fn kinetic_new() {
        let k = KineticScroll::new(0.9);
        assert!(!k.is_active());
        assert_eq!(k.velocity(), 0.0);
    }

    #[test]
    fn kinetic_start() {
        let mut k = KineticScroll::new(0.9);
        k.start(10.0);
        assert!(k.is_active());
        assert_eq!(k.velocity(), 10.0);
    }

    #[test]
    fn kinetic_update_decelerates() {
        let mut k = KineticScroll::new(0.9);
        k.start(10.0);

        let d1 = k.update();
        assert_eq!(d1, 10.0);
        assert!((k.velocity() - 9.0).abs() < f64::EPSILON);

        let d2 = k.update();
        assert_eq!(d2, 9.0);
    }

    #[test]
    fn kinetic_stops_at_threshold() {
        let mut k = KineticScroll::new(0.5);
        k.start(1.0);

        let mut frames = 0;
        while k.is_active() && frames < 100 {
            k.update();
            frames += 1;
        }

        assert!(!k.is_active());
        assert!(frames < 100);
    }

    #[test]
    fn kinetic_stop() {
        let mut k = KineticScroll::new(0.9);
        k.start(10.0);
        k.stop();
        assert!(!k.is_active());
        assert_eq!(k.velocity(), 0.0);
    }

    #[test]
    fn kinetic_custom_threshold() {
        let mut k = KineticScroll::new(0.9);
        k.set_threshold(5.0);
        k.start(10.0);

        let mut frames = 0;
        while k.is_active() && frames < 100 {
            k.update();
            frames += 1;
        }

        assert!(!k.is_active());
    }

    // -----------------------------------------------------------------------
    // New tests (Phase 5, PR 5.5)
    // -----------------------------------------------------------------------

    #[test]
    fn momentum_mode_basic() {
        let mut k = KineticScroll::new(0.9);
        k.start(100.0);
        assert_eq!(k.mode(), ScrollMode::Momentum);

        k.update();
        assert!((k.velocity() - 90.0).abs() < f64::EPSILON);

        k.update();
        assert!((k.velocity() - 81.0).abs() < f64::EPSILON);
    }

    #[test]
    fn inertia_mode_basic() {
        let mut k = KineticScroll::with_mode(ScrollMode::Inertia);
        k.set_decay_rate(1.0);
        k.start(100.0);

        // After 1 second: v = 100 * e^(-1) ≈ 36.787
        k.update_dt(1.0);
        let expected = 100.0 * (-1.0_f64).exp();
        assert!((k.velocity() - expected).abs() < 1e-10);
    }

    #[test]
    fn update_dt_momentum() {
        let mut k = KineticScroll::new(0.5);
        k.start(10.0);

        // displacement = v * dt = 10 * 0.1 = 1.0
        let d = k.update_dt(0.1);
        assert!((d - 1.0).abs() < f64::EPSILON);
        // velocity *= 0.5
        assert!((k.velocity() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn update_dt_inertia() {
        let mut k = KineticScroll::with_mode(ScrollMode::Inertia);
        k.set_decay_rate(2.0);
        k.start(50.0);

        // After 0.5s: v = 50 * e^(-2*0.5) = 50 / e
        k.update_dt(0.5);
        let expected = 50.0 / std::f64::consts::E;
        assert!((k.velocity() - expected).abs() < 1e-10);
    }

    #[test]
    fn inertia_faster_decay() {
        let mut fast = KineticScroll::with_mode(ScrollMode::Inertia);
        fast.set_decay_rate(10.0);
        fast.start(100.0);

        let mut slow = KineticScroll::with_mode(ScrollMode::Inertia);
        slow.set_decay_rate(0.5);
        slow.start(100.0);

        fast.update_dt(0.5);
        slow.update_dt(0.5);

        // Higher decay rate should result in lower velocity
        assert!(fast.velocity() < slow.velocity());
        // fast: 100 * e^(-10*0.5) = 100 * e^(-5) ≈ 0.67
        // slow: 100 * e^(-0.5*0.5) = 100 * e^(-0.25) ≈ 77.88
        assert!(fast.velocity() < 1.0);
        assert!(slow.velocity() > 50.0);
    }

    #[test]
    fn snap_target_bar_index() {
        let mut k = KineticScroll::new(0.9);
        k.set_snap_target(Some(SnapTarget::BarIndex));

        // Active => should NOT snap
        k.start(10.0);
        assert!(!k.should_snap());

        // Stop => should snap
        k.stop();
        assert!(k.should_snap());
    }

    #[test]
    fn snap_target_price() {
        let k = KineticScroll::new(0.9);
        let mut with_price = k.clone();
        with_price.set_snap_target(Some(SnapTarget::Price(150.0)));

        // Verify snap target stores price (structural check)
        if let Some(SnapTarget::Price(price)) = with_price.snap_target {
            assert!((price - 150.0).abs() < f64::EPSILON);
        } else {
            panic!("expected SnapTarget::Price(150.0)");
        }
    }

    #[test]
    fn snap_none() {
        let mut k = KineticScroll::new(0.9);
        k.set_snap_target(None);
        k.stop();
        assert!(!k.should_snap());
    }

    #[test]
    fn decay_rate_change() {
        let mut k = KineticScroll::with_mode(ScrollMode::Inertia);
        k.set_decay_rate(1.0);
        k.start(100.0);

        // Run with decay_rate = 1.0
        k.update_dt(0.5);
        let v_after_first = k.velocity();

        // Change decay rate mid-scroll
        k.set_decay_rate(5.0);
        // Re-start to reset elapsed (decay rate change takes effect on next start
        // for clean behavior; mid-scroll rate changes use the new rate going forward)
        k.start(100.0);
        k.update_dt(0.5);

        // Higher rate = lower velocity
        assert!(k.velocity() < v_after_first);
    }

    #[test]
    fn elapsed_tracking() {
        let mut k = KineticScroll::new(0.9);
        assert!((k.elapsed()).abs() < f64::EPSILON);

        k.start(50.0);

        k.update_dt(0.25);
        assert!((k.elapsed() - 0.25).abs() < f64::EPSILON);

        k.update_dt(0.5);
        assert!((k.elapsed() - 0.75).abs() < f64::EPSILON);

        k.reset_elapsed();
        assert!((k.elapsed()).abs() < f64::EPSILON);
    }

    #[test]
    fn with_mode_defaults() {
        let k = KineticScroll::with_mode(ScrollMode::Inertia);
        assert_eq!(k.mode(), ScrollMode::Inertia);
        assert!(!k.is_active());
        assert_eq!(k.velocity(), 0.0);
    }

    #[test]
    fn update_dt_zero_or_negative() {
        let mut k = KineticScroll::new(0.9);
        k.start(10.0);

        let d0 = k.update_dt(0.0);
        assert_eq!(d0, 0.0);

        let dneg = k.update_dt(-1.0);
        assert_eq!(dneg, 0.0);

        // Velocity unchanged
        assert!((k.velocity() - 10.0).abs() < f64::EPSILON);
    }
}
