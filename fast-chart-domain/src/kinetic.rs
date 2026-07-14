/// Kinetic scroll state for smooth momentum-based scrolling.
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
}

impl Default for KineticScroll {
    fn default() -> Self {
        Self {
            velocity: 0.0,
            friction: 0.95,
            threshold: 0.1,
            active: false,
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
        }
    }

    /// Start a scroll with initial velocity.
    pub fn start(&mut self, initial_velocity: f64) {
        self.velocity = initial_velocity;
        self.active = true;
    }

    /// Update velocity for one frame. Returns the displacement for this frame.
    pub fn update(&mut self) -> f64 {
        if !self.active {
            return 0.0;
        }

        let displacement = self.velocity;
        self.velocity *= self.friction;

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
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // Run until stopped
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

        // Should stop when velocity < 5.0
        let mut frames = 0;
        while k.is_active() && frames < 100 {
            k.update();
            frames += 1;
        }

        assert!(!k.is_active());
    }
}
