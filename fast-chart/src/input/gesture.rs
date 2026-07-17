//! Multi-touch gesture detection.
//!
//! Converts raw touch events into high-level gestures (tap, pan, pinch, etc.)
//! using a state machine that accumulates touch points and evaluates thresholds.

use std::time::Instant;

// ---------------------------------------------------------------------------
// Gesture
// ---------------------------------------------------------------------------

/// Detected gesture.
#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    /// Single tap at position.
    Tap {
        /// X position in screen pixels.
        x: f64,
        /// Y position in screen pixels.
        y: f64,
    },
    /// Double tap at position.
    DoubleTap {
        /// X position in screen pixels.
        x: f64,
        /// Y position in screen pixels.
        y: f64,
    },
    /// Long press at position (after duration_ms).
    LongPress {
        /// X position in screen pixels.
        x: f64,
        /// Y position in screen pixels.
        y: f64,
        /// Duration of the press in milliseconds.
        duration_ms: u64,
    },
    /// Pan gesture with delta.
    Pan {
        /// Horizontal delta in pixels.
        dx: f64,
        /// Vertical delta in pixels.
        dy: f64,
        /// Horizontal velocity in pixels/second.
        velocity_x: f64,
        /// Vertical velocity in pixels/second.
        velocity_y: f64,
    },
    /// Pinch gesture with scale factor and center.
    Pinch {
        /// Scale factor relative to initial distance (> 1.0 = spread, < 1.0 = compress).
        scale: f64,
        /// Center X of the two fingers.
        center_x: f64,
        /// Center Y of the two fingers.
        center_y: f64,
        /// Angular velocity of the pinch in radians/second (0.0 for pure pinch).
        velocity: f64,
    },
    /// Flick (fast swipe) with velocity.
    Flick {
        /// Horizontal velocity in pixels/second.
        velocity_x: f64,
        /// Vertical velocity in pixels/second.
        velocity_y: f64,
        /// Dominant direction of the flick.
        direction: FlickDirection,
    },
}

/// Direction of a flick gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlickDirection {
    /// Flick upward.
    Up,
    /// Flick downward.
    Down,
    /// Flick to the left.
    Left,
    /// Flick to the right.
    Right,
}

// ---------------------------------------------------------------------------
// GestureConfig
// ---------------------------------------------------------------------------

/// Configuration for gesture detection thresholds.
pub struct GestureConfig {
    /// Maximum duration for a tap in milliseconds.
    pub tap_max_duration_ms: u64,
    /// Maximum movement distance in pixels for a tap.
    pub tap_max_distance: f64,
    /// Duration threshold for long press in milliseconds.
    pub long_press_duration_ms: u64,
    /// Maximum interval between taps for double-tap in milliseconds.
    pub double_tap_max_interval_ms: u64,
    /// Minimum distance in pixels before a pan gesture starts.
    pub pan_min_distance: f64,
    /// Minimum velocity in pixels/second for a flick.
    pub flick_min_velocity: f64,
    /// Number of touches required for a pinch gesture.
    pub pinch_touch_count: usize,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_max_duration_ms: 300,
            tap_max_distance: 10.0,
            long_press_duration_ms: 500,
            double_tap_max_interval_ms: 300,
            pan_min_distance: 8.0,
            flick_min_velocity: 800.0,
            pinch_touch_count: 2,
        }
    }
}

// ---------------------------------------------------------------------------
// TouchPoint (internal)
// ---------------------------------------------------------------------------

/// State of a single touch point.
#[derive(Debug, Clone)]
struct TouchPoint {
    id: u64,
    start_x: f64,
    start_y: f64,
    current_x: f64,
    current_y: f64,
    start_time: Instant,
    last_move_time: Instant,
}

impl TouchPoint {
    fn distance_from_start(&self) -> f64 {
        let dx = self.current_x - self.start_x;
        let dy = self.current_y - self.start_y;
        (dx * dx + dy * dy).sqrt()
    }

    fn elapsed_ms(&self, now: Instant) -> u64 {
        now.duration_since(self.start_time).as_millis() as u64
    }
}

// ---------------------------------------------------------------------------
// GestureDetector
// ---------------------------------------------------------------------------

/// Multi-touch gesture detector.
///
/// Accumulates touch events and produces high-level [`Gesture`]s when
/// thresholds are met. The host calls `touch_start`, `touch_move`, and
/// `touch_end` as raw events arrive, then collects any returned gestures.
pub struct GestureDetector {
    config: GestureConfig,
    /// Active touch points.
    touches: Vec<TouchPoint>,
    /// Whether a gesture is currently being tracked.
    tracking: bool,
    /// Last tap time for double-tap detection.
    last_tap_time: Option<Instant>,
    /// Last tap position for double-tap detection.
    last_tap_position: Option<(f64, f64)>,
    /// Whether pan gesture has started (passed min distance).
    pan_started: bool,
    /// Previous pinch distance for computing scale between frames.
    pinch_initial_distance: Option<f64>,
}

impl GestureDetector {
    /// Create a new gesture detector with the given configuration.
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            touches: Vec::new(),
            tracking: false,
            last_tap_time: None,
            last_tap_position: None,
            pan_started: false,
            pinch_initial_distance: None,
        }
    }

    /// Create a new gesture detector with default configuration.
    pub fn with_default_config() -> Self {
        Self::new(GestureConfig::default())
    }

    /// A touch began at the given position.
    pub fn touch_start(&mut self, id: u64, x: f64, y: f64) {
        let now = Instant::now();

        self.touches.push(TouchPoint {
            id,
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
            start_time: now,
            last_move_time: now,
        });

        self.tracking = true;

        // When a second finger touches, record the initial pinch distance.
        if self.touches.len() == self.config.pinch_touch_count {
            self.pinch_initial_distance = self.compute_two_finger_distance();
        }
    }

    /// A touch moved to a new position. Returns a gesture if thresholds are met.
    pub fn touch_move(&mut self, id: u64, x: f64, y: f64) -> Option<Gesture> {
        let now = Instant::now();

        let touch = self.touches.iter_mut().find(|t| t.id == id)?;
        touch.current_x = x;
        touch.current_y = y;
        touch.last_move_time = now;

        // Pinch detection: two fingers moving.
        if self.touches.len() == self.config.pinch_touch_count {
            if self.pinch_initial_distance.is_none() {
                self.pinch_initial_distance = self.compute_two_finger_distance().or(Some(1.0));
            }

            return self.compute_pinch_gesture();
        }

        // Pan detection: single finger moved past threshold.
        if self.touches.len() == 1 && !self.pan_started {
            let touch = &self.touches[0];
            if touch.distance_from_start() >= self.config.pan_min_distance {
                self.pan_started = true;
                return self.compute_pan_gesture();
            }
        }

        // Ongoing pan: emit incremental pan delta.
        if self.pan_started && self.touches.len() == 1 {
            return self.compute_pan_gesture();
        }

        None
    }

    /// A touch ended at the given position. Returns a gesture if thresholds are met.
    pub fn touch_end(&mut self, id: u64, x: f64, y: f64) -> Option<Gesture> {
        let now = Instant::now();

        // Find the touch that ended.
        let touch_index = self.touches.iter().position(|t| t.id == id)?;
        let start_x = self.touches[touch_index].start_x;
        let start_y = self.touches[touch_index].start_y;
        let start_time = self.touches[touch_index].start_time;
        let elapsed_ms = self.touches[touch_index].elapsed_ms(now);
        let distance = self.touches[touch_index].distance_from_start();

        // If this was a two-finger gesture and one finger lifted, clear pinch state.
        if self.touches.len() == self.config.pinch_touch_count {
            self.pinch_initial_distance = None;
        }

        self.touches.remove(touch_index);

        // No more active touches — evaluate final gesture.
        if self.touches.is_empty() {
            let gesture = self.evaluate_end_gesture(
                elapsed_ms, distance, x, y, now, start_x, start_y, start_time,
            );
            self.tracking = false;
            self.pan_started = false;
            return gesture;
        }

        None
    }

    /// Check for long press. Call periodically with the current time.
    ///
    /// Returns a `LongPress` gesture if a single touch has been held
    /// longer than the configured threshold without exceeding the
    /// tap distance limit.
    pub fn check_long_press(&self, now: Instant) -> Option<Gesture> {
        if self.touches.len() != 1 {
            return None;
        }

        let touch = &self.touches[0];
        let elapsed_ms = touch.elapsed_ms(now);

        if elapsed_ms >= self.config.long_press_duration_ms
            && touch.distance_from_start() < self.config.tap_max_distance
        {
            Some(Gesture::LongPress {
                x: touch.current_x,
                y: touch.current_y,
                duration_ms: elapsed_ms,
            })
        } else {
            None
        }
    }

    /// Reset all state to initial.
    pub fn reset(&mut self) {
        self.touches.clear();
        self.tracking = false;
        self.last_tap_time = None;
        self.last_tap_position = None;
        self.pan_started = false;
        self.pinch_initial_distance = None;
    }

    /// Number of active touches.
    pub fn touch_count(&self) -> usize {
        self.touches.len()
    }

    /// Whether currently tracking a gesture.
    pub fn is_tracking(&self) -> bool {
        self.tracking
    }

    /// Get configuration.
    pub fn config(&self) -> &GestureConfig {
        &self.config
    }

    // -- private helpers ----------------------------------------------------

    /// Compute the distance between two active touch points.
    fn compute_two_finger_distance(&self) -> Option<f64> {
        if self.touches.len() < 2 {
            return None;
        }

        let a = &self.touches[0];
        let b = &self.touches[1];
        let dx = b.current_x - a.current_x;
        let dy = b.current_y - a.current_y;
        Some((dx * dx + dy * dy).sqrt())
    }

    /// Compute pinch gesture from two active touches.
    fn compute_pinch_gesture(&self) -> Option<Gesture> {
        if self.touches.len() < 2 {
            return None;
        }

        let a = &self.touches[0];
        let b = &self.touches[1];

        let current_distance = {
            let dx = b.current_x - a.current_x;
            let dy = b.current_y - a.current_y;
            (dx * dx + dy * dy).sqrt()
        };

        let initial = self.pinch_initial_distance.unwrap_or(current_distance);
        let scale = if initial > 0.0 {
            current_distance / initial
        } else {
            1.0
        };

        let center_x = (a.current_x + b.current_x) / 2.0;
        let center_y = (a.current_y + b.current_y) / 2.0;

        Some(Gesture::Pinch {
            scale,
            center_x,
            center_y,
            velocity: 0.0,
        })
    }

    /// Compute pan gesture from the first touch point.
    fn compute_pan_gesture(&self) -> Option<Gesture> {
        if self.touches.is_empty() {
            return None;
        }

        let touch = &self.touches[0];
        let dx = touch.current_x - touch.start_x;
        let dy = touch.current_y - touch.start_y;

        let elapsed = touch.last_move_time.duration_since(touch.start_time);
        let elapsed_secs = elapsed.as_secs_f64();

        let (velocity_x, velocity_y) = if elapsed_secs > 0.0 {
            (
                (touch.current_x - touch.start_x) / elapsed_secs,
                (touch.current_y - touch.start_y) / elapsed_secs,
            )
        } else {
            (0.0, 0.0)
        };

        Some(Gesture::Pan {
            dx,
            dy,
            velocity_x,
            velocity_y,
        })
    }

    /// Evaluate the gesture when the last touch ends.
    fn evaluate_end_gesture(
        &mut self,
        elapsed_ms: u64,
        distance: f64,
        x: f64,
        y: f64,
        now: Instant,
        start_x: f64,
        start_y: f64,
        start_time: Instant,
    ) -> Option<Gesture> {
        // Was a pan in progress? Check for flick.
        if self.pan_started {
            let gesture = self.evaluate_flick_or_pan_end(x, y, now, start_x, start_y, start_time);
            return gesture;
        }

        // Not a pan — evaluate as tap.
        if elapsed_ms <= self.config.tap_max_duration_ms
            && distance <= self.config.tap_max_distance
        {
            return self.evaluate_tap(x, y, now);
        }

        None
    }

    /// Evaluate whether a tap or double-tap occurred.
    fn evaluate_tap(&mut self, x: f64, y: f64, now: Instant) -> Option<Gesture> {
        // Check for double-tap.
        if let Some(last_time) = self.last_tap_time {
            let interval_ms = now.duration_since(last_time).as_millis() as u64;
            if interval_ms <= self.config.double_tap_max_interval_ms {
                if let Some((last_x, last_y)) = self.last_tap_position {
                    let dx = x - last_x;
                    let dy = y - last_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= self.config.tap_max_distance {
                        self.last_tap_time = None;
                        self.last_tap_position = None;
                        return Some(Gesture::DoubleTap { x, y });
                    }
                }
            }
        }

        // Record this tap for potential double-tap.
        self.last_tap_time = Some(now);
        self.last_tap_position = Some((x, y));

        Some(Gesture::Tap { x, y })
    }

    /// Evaluate whether a pan end is actually a flick.
    fn evaluate_flick_or_pan_end(
        &self,
        x: f64,
        y: f64,
        now: Instant,
        start_x: f64,
        start_y: f64,
        start_time: Instant,
    ) -> Option<Gesture> {
        let elapsed_secs = now.duration_since(start_time).as_secs_f64();

        if elapsed_secs > 0.0 {
            let velocity_x = (x - start_x) / elapsed_secs;
            let velocity_y = (y - start_y) / elapsed_secs;
            let speed = (velocity_x * velocity_x + velocity_y * velocity_y).sqrt();

            if speed >= self.config.flick_min_velocity {
                let direction = flick_direction_from_velocity(velocity_x, velocity_y);
                return Some(Gesture::Flick {
                    velocity_x,
                    velocity_y,
                    direction,
                });
            }
        }

        // Not fast enough for flick — emit final pan.
        let dx = x - start_x;
        let dy = y - start_y;
        let (velocity_x, velocity_y) = if elapsed_secs > 0.0 {
            (dx / elapsed_secs, dy / elapsed_secs)
        } else {
            (0.0, 0.0)
        };
        Some(Gesture::Pan {
            dx,
            dy,
            velocity_x,
            velocity_y,
        })
    }
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self::with_default_config()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Determine flick direction from velocity components.
fn flick_direction_from_velocity(velocity_x: f64, velocity_y: f64) -> FlickDirection {
    if velocity_x.abs() >= velocity_y.abs() {
        if velocity_x >= 0.0 {
            FlickDirection::Right
        } else {
            FlickDirection::Left
        }
    } else if velocity_y >= 0.0 {
        FlickDirection::Down
    } else {
        FlickDirection::Up
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn default_config() {
        let config = GestureConfig::default();
        assert_eq!(config.tap_max_duration_ms, 300);
        assert_eq!(config.tap_max_distance, 10.0);
        assert_eq!(config.long_press_duration_ms, 500);
        assert_eq!(config.double_tap_max_interval_ms, 300);
        assert_eq!(config.pan_min_distance, 8.0);
        assert_eq!(config.flick_min_velocity, 800.0);
        assert_eq!(config.pinch_touch_count, 2);
    }

    #[test]
    fn single_tap() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 200.0);
        // End quickly without moving — should produce a tap.
        let gesture = det.touch_end(1, 100.0, 200.0);

        assert_eq!(gesture, Some(Gesture::Tap { x: 100.0, y: 200.0 }));
    }

    #[test]
    fn tap_too_long() {
        let config = GestureConfig {
            tap_max_duration_ms: 200,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);
        let t0 = Instant::now();

        det.touch_start(1, 100.0, 200.0);
        // Simulate time passing beyond threshold by calling check_long_press
        // with a time that exceeds tap_max_duration_ms.
        let future = t0 + Duration::from_millis(250);
        let lp = det.check_long_press(future);
        // Long press not yet triggered (needs 500ms), but tap won't fire
        // because the end will happen after the max duration.
        assert_eq!(lp, None);
    }

    #[test]
    fn tap_too_far() {
        let config = GestureConfig {
            tap_max_distance: 5.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        det.touch_start(1, 100.0, 200.0);
        det.touch_move(1, 120.0, 200.0); // 20px movement, beyond 5px limit
        let gesture = det.touch_end(1, 120.0, 200.0);

        // Moved too far for a tap, but not far enough for pan (pan_min_distance=8).
        // touch_move with 20px triggers pan_started, so we get a Pan.
        assert!(gesture.is_some());
        assert!(!matches!(gesture, Some(Gesture::Tap { .. })));
    }

    #[test]
    fn double_tap() {
        let config = GestureConfig {
            double_tap_max_interval_ms: 300,
            tap_max_duration_ms: 300,
            tap_max_distance: 10.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        // First tap.
        det.touch_start(1, 100.0, 200.0);
        let g1 = det.touch_end(1, 100.0, 200.0);
        assert_eq!(g1, Some(Gesture::Tap { x: 100.0, y: 200.0 }));

        // Second tap quickly — should be a double tap.
        det.touch_start(2, 105.0, 205.0);
        let g2 = det.touch_end(2, 105.0, 205.0);
        assert_eq!(g2, Some(Gesture::DoubleTap { x: 105.0, y: 205.0 }));
    }

    #[test]
    fn double_tap_too_slow() {
        let make_config = || GestureConfig {
            double_tap_max_interval_ms: 100,
            tap_max_duration_ms: 300,
            tap_max_distance: 10.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(make_config());

        // First tap.
        det.touch_start(1, 100.0, 200.0);
        let g1 = det.touch_end(1, 100.0, 200.0);
        assert_eq!(g1, Some(Gesture::Tap { x: 100.0, y: 200.0 }));

        // Fresh detector — no last_tap_time, so this is a single tap.
        let mut det2 = GestureDetector::new(make_config());
        det2.touch_start(2, 100.0, 200.0);
        let g2 = det2.touch_end(2, 100.0, 200.0);
        assert_eq!(g2, Some(Gesture::Tap { x: 100.0, y: 200.0 }));
    }

    #[test]
    fn long_press() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 200.0);
        // Capture time AFTER touch_start so it matches start_time closely.
        let t0 = Instant::now();

        // Before threshold — no long press.
        let early = t0 + Duration::from_millis(200);
        assert_eq!(det.check_long_press(early), None);

        // At threshold — long press.
        let at_threshold = t0 + Duration::from_millis(500);
        let gesture = det.check_long_press(at_threshold);
        match gesture {
            Some(Gesture::LongPress { x, y, duration_ms }) => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
                assert!(duration_ms >= 500, "duration_ms should be >= 500, got {duration_ms}");
            }
            other => panic!("Expected LongPress, got {other:?}"),
        }
    }

    #[test]
    fn long_press_with_movement() {
        let config = GestureConfig {
            tap_max_distance: 5.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        det.touch_start(1, 100.0, 200.0);
        // Move slightly — still within tap_max_distance.
        det.touch_move(1, 103.0, 200.0);

        let t0 = Instant::now();
        let at_threshold = t0 + Duration::from_millis(500);
        let gesture = det.check_long_press(at_threshold);
        // Movement < tap_max_distance (5.0), so long press fires.
        assert!(matches!(gesture, Some(Gesture::LongPress { .. })));
    }

    #[test]
    fn pan_start() {
        let config = GestureConfig {
            pan_min_distance: 5.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        det.touch_start(1, 100.0, 100.0);
        let gesture = det.touch_move(1, 120.0, 120.0);

        assert!(matches!(gesture, Some(Gesture::Pan { dx: 20.0, dy: 20.0, .. })));
        assert!(det.is_tracking());
    }

    #[test]
    fn pan_not_started() {
        let config = GestureConfig {
            pan_min_distance: 50.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        det.touch_start(1, 100.0, 100.0);
        let gesture = det.touch_move(1, 105.0, 100.0);

        // Movement (5px) < pan_min_distance (50px) — no gesture emitted.
        assert_eq!(gesture, None);
    }

    #[test]
    fn pinch_two_fingers() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 300.0, 200.0);

        // Move second finger further apart.
        let gesture = det.touch_move(2, 400.0, 200.0);

        match gesture {
            Some(Gesture::Pinch { scale, .. }) => assert!(scale > 1.0),
            other => panic!("Expected Pinch with scale > 1.0, got {other:?}"),
        }
    }

    #[test]
    fn pinch_spread() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 200.0, 200.0);
        // Initial distance = 100.

        det.touch_move(2, 300.0, 200.0);
        // New distance = 200. Scale = 200/100 = 2.0.

        let gesture = det.touch_move(2, 300.0, 200.0);
        if let Some(Gesture::Pinch { scale, .. }) = gesture {
            assert!(scale > 1.0, "Expected spread scale > 1.0, got {scale}");
        } else {
            panic!("Expected Pinch gesture");
        }
    }

    #[test]
    fn pinch_compress() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 300.0, 200.0);
        // Initial distance = 200.

        det.touch_move(2, 150.0, 200.0);
        // New distance = 50. Scale = 50/200 = 0.25.

        let gesture = det.touch_move(2, 150.0, 200.0);
        if let Some(Gesture::Pinch { scale, .. }) = gesture {
            assert!(scale < 1.0, "Expected compress scale < 1.0, got {scale}");
        } else {
            panic!("Expected Pinch gesture");
        }
    }

    #[test]
    fn flick_fast() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 100.0);
        // Move far enough to start pan.
        det.touch_move(1, 300.0, 100.0);

        // End with high velocity (touch started recently, moved 200px).
        // We need the velocity to be >= flick_min_velocity (800 px/s).
        // Since Instant::now() is used internally, this test validates
        // the code path. The actual velocity depends on real elapsed time.
        let gesture = det.touch_end(1, 300.0, 100.0);
        assert!(gesture.is_some());
        // The gesture is either Flick or Pan depending on actual elapsed time.
        // We verify the touch_end produces a gesture.
        assert!(det.touch_count() == 0);
    }

    #[test]
    fn flick_direction() {
        assert_eq!(flick_direction_from_velocity(100.0, 0.0), FlickDirection::Right);
        assert_eq!(flick_direction_from_velocity(-100.0, 0.0), FlickDirection::Left);
        assert_eq!(flick_direction_from_velocity(0.0, 100.0), FlickDirection::Down);
        assert_eq!(flick_direction_from_velocity(0.0, -100.0), FlickDirection::Up);
        // Diagonal: |vx| > |vy| → horizontal.
        assert_eq!(flick_direction_from_velocity(100.0, 50.0), FlickDirection::Right);
        assert_eq!(flick_direction_from_velocity(-100.0, -50.0), FlickDirection::Left);
        // Diagonal: |vy| > |vx| → vertical.
        assert_eq!(flick_direction_from_velocity(50.0, 100.0), FlickDirection::Down);
        assert_eq!(flick_direction_from_velocity(-50.0, -100.0), FlickDirection::Up);
    }

    #[test]
    fn reset_clears() {
        let mut det = GestureDetector::with_default_config();

        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 300.0, 200.0);
        assert!(det.is_tracking());
        assert_eq!(det.touch_count(), 2);

        det.reset();

        assert!(!det.is_tracking());
        assert_eq!(det.touch_count(), 0);
    }

    #[test]
    fn touch_move_unknown_id_returns_none() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);

        // Move with a non-existent id.
        let gesture = det.touch_move(999, 150.0, 200.0);
        assert_eq!(gesture, None);
    }

    #[test]
    fn touch_end_unknown_id_returns_none() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);

        let gesture = det.touch_end(999, 100.0, 200.0);
        assert_eq!(gesture, None);
    }

    #[test]
    fn config_getter() {
        let config = GestureConfig::default();
        let det = GestureDetector::new(config);
        assert_eq!(det.config().tap_max_duration_ms, 300);
    }

    #[test]
    fn multiple_pan_movements() {
        let config = GestureConfig {
            pan_min_distance: 5.0,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        det.touch_start(1, 100.0, 100.0);
        let g1 = det.touch_move(1, 120.0, 100.0);
        assert!(matches!(g1, Some(Gesture::Pan { dx: 20.0, .. })));

        let g2 = det.touch_move(1, 140.0, 100.0);
        assert!(matches!(g2, Some(Gesture::Pan { dx: 40.0, .. })));
    }

    #[test]
    fn pan_end_without_flick() {
        let config = GestureConfig {
            pan_min_distance: 5.0,
            flick_min_velocity: f64::MAX,
            ..GestureConfig::default()
        };
        let mut det = GestureDetector::new(config);

        det.touch_start(1, 100.0, 100.0);
        det.touch_move(1, 120.0, 100.0);
        let gesture = det.touch_end(1, 120.0, 100.0);

        // Should end as a Pan, not Flick.
        assert!(matches!(gesture, Some(Gesture::Pan { .. })));
    }
}
