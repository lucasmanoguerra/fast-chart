//! Animation engine for smooth transitions.
//!
//! Provides easing functions and value interpolation for animated chart
//! transitions: price ticks, scale changes, zoom, scroll, opacity, etc.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Easing
// ---------------------------------------------------------------------------

/// Easing functions for animation curves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Constant speed from start to end.
    Linear,
    /// Accelerates: slow at start, fast at end.
    EaseIn,
    /// Decelerates: fast at start, slow at end.
    EaseOut,
    /// Accelerates then decelerates.
    EaseInOut,
    /// Spring-based overshoot with configurable stiffness and damping.
    Spring {
        /// Spring stiffness coefficient (higher = snappier).
        stiffness: f64,
        /// Damping ratio (>= 1.0 = critically damped / overdamped).
        damping: f64,
    },
}

// ---------------------------------------------------------------------------
// AnimationTrack
// ---------------------------------------------------------------------------

/// A single animation track: from → to over a duration.
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    /// Start value.
    pub from: f64,
    /// End value.
    pub to: f64,
    /// Total duration in milliseconds.
    pub duration_ms: f64,
    /// Easing function applied to progress.
    pub easing: Easing,
}

// ---------------------------------------------------------------------------
// AnimationState
// ---------------------------------------------------------------------------

/// The current state snapshot of a running animation.
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Elapsed time in milliseconds.
    pub elapsed_ms: f64,
    /// Total duration in milliseconds.
    pub duration_ms: f64,
    /// Whether the animation has completed.
    pub complete: bool,
}

impl AnimationState {
    /// Progress fraction `[0.0, 1.0]`.
    pub fn progress(&self) -> f64 {
        if self.duration_ms <= 0.0 {
            return 1.0;
        }
        (self.elapsed_ms / self.duration_ms).min(1.0)
    }
}

// ---------------------------------------------------------------------------
// AnimatedValue
// ---------------------------------------------------------------------------

/// Animated value that interpolates between two `f64`s over time.
#[derive(Debug, Clone)]
pub struct AnimatedValue {
    /// Start value.
    pub from: f64,
    /// End value.
    pub to: f64,
    /// Duration in milliseconds.
    pub duration_ms: f64,
    /// Easing function.
    pub easing: Easing,
    /// Current elapsed time in ms.
    elapsed_ms: f64,
    /// Whether the animation is complete.
    complete: bool,
}

impl AnimatedValue {
    /// Create a new animated value.
    pub fn new(from: f64, to: f64, duration_ms: f64, easing: Easing) -> Self {
        Self {
            from,
            to,
            duration_ms,
            easing,
            elapsed_ms: 0.0,
            complete: duration_ms <= 0.0,
        }
    }

    /// Advance the animation by `dt_ms` milliseconds.
    pub fn update(&mut self, dt_ms: f64) {
        if self.complete {
            return;
        }
        self.elapsed_ms += dt_ms;
        if self.elapsed_ms >= self.duration_ms {
            self.elapsed_ms = self.duration_ms;
            self.complete = true;
        }
    }

    /// Get the current interpolated value.
    pub fn current(&self) -> f64 {
        let t = self.progress();
        let eased = apply_easing(t, self.easing);
        self.from + (self.to - self.from) * eased
    }

    /// Get the current animation state.
    pub fn state(&self) -> AnimationState {
        AnimationState {
            elapsed_ms: self.elapsed_ms,
            duration_ms: self.duration_ms,
            complete: self.complete,
        }
    }

    /// Whether the animation has completed.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Force complete the animation, snapping to the target value.
    pub fn complete(&mut self) {
        self.elapsed_ms = self.duration_ms;
        self.complete = true;
    }

    /// Reset with a new target value, keeping the current value as the new
    /// start point.
    pub fn retarget(&mut self, new_target: f64) {
        self.from = self.current();
        self.to = new_target;
        self.elapsed_ms = 0.0;
        self.complete = false;
    }

    fn progress(&self) -> f64 {
        if self.duration_ms <= 0.0 {
            return 1.0;
        }
        (self.elapsed_ms / self.duration_ms).min(1.0)
    }
}

// ---------------------------------------------------------------------------
// AnimationEngine
// ---------------------------------------------------------------------------

/// Engine that manages multiple named animated values.
#[derive(Debug, Default)]
pub struct AnimationEngine {
    /// Active animations keyed by name.
    animations: HashMap<String, AnimatedValue>,
}

impl AnimationEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an animation. If one with the same name exists it is replaced.
    pub fn animate(&mut self, name: &str, animation: AnimatedValue) {
        self.animations.insert(name.to_owned(), animation);
    }

    /// Remove a named animation, returning it if it existed.
    pub fn remove(&mut self, name: &str) -> Option<AnimatedValue> {
        self.animations.remove(name)
    }

    /// Get the current interpolated value of a named animation.
    pub fn value(&self, name: &str) -> Option<f64> {
        self.animations.get(name).map(|a| a.current())
    }

    /// Check if a named animation exists and is complete.
    pub fn is_complete(&self, name: &str) -> bool {
        self.animations
            .get(name)
            .map_or(false, |a| a.is_complete())
    }

    /// Advance all animations by `dt_ms` milliseconds.
    pub fn update(&mut self, dt_ms: f64) {
        for anim in self.animations.values_mut() {
            anim.update(dt_ms);
        }
    }

    /// Remove all completed animations.
    pub fn gc(&mut self) {
        self.animations.retain(|_, a| !a.is_complete());
    }

    /// Number of active animations.
    pub fn active_count(&self) -> usize {
        self.animations.len()
    }

    /// Check if any animation is still running (not complete).
    pub fn has_active(&self) -> bool {
        self.animations.values().any(|a| !a.is_complete())
    }
}

// ---------------------------------------------------------------------------
// Easing implementations
// ---------------------------------------------------------------------------

/// Apply an easing function to a progress value `[0.0, 1.0]`.
///
/// Returns the eased output in the same range for well-behaved easings.
/// Spring easing may overshoot.
pub fn apply_easing(t: f64, easing: Easing) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => t * (2.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        Easing::Spring { stiffness, damping } => {
            spring_ease(t, stiffness, damping)
        }
    }
}

/// Critically-damped spring approximation.
///
/// For overdamped (damping >= 1.0) and underdamped (damping < 1.0) cases.
fn spring_ease(t: f64, stiffness: f64, damping: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    let omega = stiffness.sqrt();

    if damping >= 1.0 {
        // Overdamped / critically damped
        1.0 - (1.0 + omega * t) * (-omega * t).exp()
    } else {
        // Underdamped
        let omega_d = omega * (1.0 - damping * damping).sqrt();
        1.0 - (-damping * omega * t * (omega_d * t).cos()
            + (damping * omega / omega_d) * (-damping * omega * t).sin() * (omega_d * t).sin())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Easing tests ---

    #[test]
    fn easing_linear() {
        assert!((apply_easing(0.0, Easing::Linear) - 0.0).abs() < 1e-10);
        assert!((apply_easing(0.5, Easing::Linear) - 0.5).abs() < 1e-10);
        assert!((apply_easing(1.0, Easing::Linear) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn easing_ease_in() {
        let v = apply_easing(0.5, Easing::EaseIn);
        assert!((v - 0.25).abs() < 1e-10, "EaseIn(0.5) = {v}");
    }

    #[test]
    fn easing_ease_out() {
        let v = apply_easing(0.5, Easing::EaseOut);
        assert!((v - 0.75).abs() < 1e-10, "EaseOut(0.5) = {v}");
    }

    #[test]
    fn easing_ease_in_out_first_half() {
        let v = apply_easing(0.25, Easing::EaseInOut);
        assert!((v - 0.125).abs() < 1e-10, "EaseInOut(0.25) = {v}");
    }

    #[test]
    fn easing_ease_in_out_second_half() {
        let v = apply_easing(0.75, Easing::EaseInOut);
        assert!((v - 0.875).abs() < 1e-10, "EaseInOut(0.75) = {v}");
    }

    #[test]
    fn easing_spring_converges() {
        let v = apply_easing(1.0, Easing::Spring { stiffness: 200.0, damping: 12.0 });
        assert!(
            (v - 1.0).abs() < 0.01,
            "Spring(1.0) = {v}, expected ≈ 1.0"
        );
    }

    #[test]
    fn easing_clamps_at_boundaries() {
        let easings = [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
        ];
        for e in easings {
            assert!((apply_easing(0.0, e) - 0.0).abs() < 1e-10);
            assert!((apply_easing(1.0, e) - 1.0).abs() < 1e-10);
        }
    }

    // --- AnimatedValue tests ---

    #[test]
    fn animated_value_new() {
        let a = AnimatedValue::new(10.0, 20.0, 500.0, Easing::Linear);
        assert!((a.from - 10.0).abs() < 1e-10);
        assert!((a.to - 20.0).abs() < 1e-10);
        assert!((a.duration_ms - 500.0).abs() < 1e-10);
        assert_eq!(a.easing, Easing::Linear);
        assert!(!a.is_complete());
    }

    #[test]
    fn animated_value_new_zero_duration_is_complete() {
        let a = AnimatedValue::new(0.0, 1.0, 0.0, Easing::Linear);
        assert!(a.is_complete());
    }

    #[test]
    fn animated_value_update() {
        let mut a = AnimatedValue::new(0.0, 100.0, 1000.0, Easing::Linear);
        a.update(250.0);
        let s = a.state();
        assert!((s.elapsed_ms - 250.0).abs() < 1e-10);
        assert!(!s.complete);
        // Linear: progress = 0.25, value = 25.0
        let v = a.current();
        assert!((v - 25.0).abs() < 1e-10, "current = {v}");
    }

    #[test]
    fn animated_value_complete_at_end() {
        let mut a = AnimatedValue::new(0.0, 10.0, 200.0, Easing::Linear);
        a.update(200.0);
        assert!(a.is_complete());
        let v = a.current();
        assert!((v - 10.0).abs() < 1e-10, "current = {v}");
    }

    #[test]
    fn animated_value_is_complete_flag() {
        let mut a = AnimatedValue::new(0.0, 1.0, 100.0, Easing::EaseIn);
        assert!(!a.is_complete());
        a.update(99.0);
        assert!(!a.is_complete());
        a.update(1.0);
        assert!(a.is_complete());
    }

    #[test]
    fn animated_value_complete_force() {
        let mut a = AnimatedValue::new(0.0, 50.0, 1000.0, Easing::Linear);
        a.update(100.0);
        assert!(!a.is_complete());
        a.complete();
        assert!(a.is_complete());
        assert!((a.current() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn animated_value_retarget() {
        let mut a = AnimatedValue::new(0.0, 100.0, 1000.0, Easing::Linear);
        a.update(500.0);
        // At 50% linear, current ≈ 50.0
        let mid = a.current();
        assert!((mid - 50.0).abs() < 1e-10);

        a.retarget(200.0);
        // Should start from ~50.0 (the current value) toward 200.0
        assert!((a.from - 50.0).abs() < 1e-10);
        assert!((a.to - 200.0).abs() < 1e-10);
        assert!(!a.is_complete());
    }

    #[test]
    fn animated_value_update_past_end_caps() {
        let mut a = AnimatedValue::new(0.0, 10.0, 100.0, Easing::Linear);
        a.update(500.0); // way past end
        assert!(a.is_complete());
        assert!((a.current() - 10.0).abs() < 1e-10);
    }

    // --- AnimationEngine tests ---

    #[test]
    fn animation_engine_add_and_get() {
        let mut engine = AnimationEngine::new();
        let anim = AnimatedValue::new(0.0, 1.0, 500.0, Easing::Linear);
        engine.animate("scroll", anim);

        let v = engine.value("scroll");
        assert!(v.is_some());
        assert!((v.unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn animation_engine_remove() {
        let mut engine = AnimationEngine::new();
        engine.animate("fade", AnimatedValue::new(0.0, 1.0, 200.0, Easing::Linear));
        assert!(engine.remove("fade").is_some());
        assert!(engine.value("fade").is_none());
        assert!(engine.remove("nonexistent").is_none());
    }

    #[test]
    fn animation_engine_update_all() {
        let mut engine = AnimationEngine::new();
        engine.animate("a", AnimatedValue::new(0.0, 10.0, 100.0, Easing::Linear));
        engine.animate("b", AnimatedValue::new(0.0, 20.0, 50.0, Easing::Linear));

        engine.update(50.0);
        let va = engine.value("a").unwrap();
        let vb = engine.value("b").unwrap();
        assert!((va - 5.0).abs() < 1e-10, "a = {va}");
        assert!((vb - 20.0).abs() < 1e-10, "b = {vb}");
        assert!(engine.is_complete("b"));
        assert!(!engine.is_complete("a"));
    }

    #[test]
    fn animation_engine_gc() {
        let mut engine = AnimationEngine::new();
        engine.animate("done", AnimatedValue::new(0.0, 1.0, 50.0, Easing::Linear));
        engine.animate("running", AnimatedValue::new(0.0, 1.0, 500.0, Easing::Linear));

        engine.update(100.0); // "done" completes
        assert_eq!(engine.active_count(), 2);
        engine.gc();
        assert_eq!(engine.active_count(), 1);
        assert!(engine.value("done").is_none());
        assert!(engine.value("running").is_some());
    }

    #[test]
    fn animation_engine_active_count() {
        let mut engine = AnimationEngine::new();
        assert_eq!(engine.active_count(), 0);
        assert!(!engine.has_active());

        engine.animate("a", AnimatedValue::new(0.0, 1.0, 100.0, Easing::Linear));
        engine.animate("b", AnimatedValue::new(0.0, 1.0, 200.0, Easing::Linear));
        assert_eq!(engine.active_count(), 2);
        assert!(engine.has_active());

        engine.update(150.0);
        // "a" completed, "b" still running
        assert_eq!(engine.active_count(), 2); // gc not called yet
        assert!(engine.has_active());

        engine.gc();
        assert_eq!(engine.active_count(), 1);
        assert!(engine.has_active());
    }

    #[test]
    fn animation_engine_replace_same_name() {
        let mut engine = AnimationEngine::new();
        engine.animate("x", AnimatedValue::new(0.0, 10.0, 100.0, Easing::Linear));
        engine.animate("x", AnimatedValue::new(0.0, 20.0, 200.0, Easing::Linear));

        assert_eq!(engine.active_count(), 1);
        assert!(!engine.is_complete("x"));
    }
}
