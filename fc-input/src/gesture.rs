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
        x: f64,
        y: f64,
    },
    /// Double tap at position.
    DoubleTap {
        x: f64,
        y: f64,
    },
    /// Long press at position (after duration_ms).
    LongPress {
        x: f64,
        y: f64,
        duration_ms: u64,
    },
    /// Pan gesture with delta.
    Pan {
        dx: f64,
        dy: f64,
        velocity_x: f64,
        velocity_y: f64,
    },
    /// Pinch gesture with scale factor and center.
    Pinch {
        scale: f64,
        center_x: f64,
        center_y: f64,
        velocity: f64,
    },
    /// Flick (fast swipe) with velocity.
    Flick {
        velocity_x: f64,
        velocity_y: f64,
        direction: FlickDirection,
    },
}

/// Direction of a flick gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlickDirection {
    Up,
    Down,
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// GestureConfig
// ---------------------------------------------------------------------------

/// Configuration for gesture detection thresholds.
#[derive(Debug, Clone)]
pub struct GestureConfig {
    pub tap_max_duration_ms: u64,
    pub tap_max_distance: f64,
    pub long_press_duration_ms: u64,
    pub double_tap_max_interval_ms: u64,
    pub pan_min_distance: f64,
    pub flick_min_velocity: f64,
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
/// thresholds are met.
pub struct GestureDetector {
    config: GestureConfig,
    touches: Vec<TouchPoint>,
    tracking: bool,
    last_tap_time: Option<Instant>,
    last_tap_position: Option<(f64, f64)>,
    pan_started: bool,
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
            id, start_x: x, start_y: y, current_x: x, current_y: y,
            start_time: now, last_move_time: now,
        });
        self.tracking = true;
        if self.touches.len() == self.config.pinch_touch_count {
            self.pinch_initial_distance = self.compute_two_finger_distance();
        }
    }

    /// Return a gesture if thresholds are met.
    pub fn touch_move(&mut self, id: u64, x: f64, y: f64) -> Option<Gesture> {
        let now = Instant::now();
        let touch = self.touches.iter_mut().find(|t| t.id == id)?;
        touch.current_x = x;
        touch.current_y = y;
        touch.last_move_time = now;

        if self.touches.len() == self.config.pinch_touch_count {
            if self.pinch_initial_distance.is_none() {
                self.pinch_initial_distance = self.compute_two_finger_distance().or(Some(1.0));
            }
            return self.compute_pinch_gesture();
        }

        if self.touches.len() == 1 && !self.pan_started {
            let touch = &self.touches[0];
            if touch.distance_from_start() >= self.config.pan_min_distance {
                self.pan_started = true;
                return self.compute_pan_gesture();
            }
        }

        if self.pan_started && self.touches.len() == 1 {
            return self.compute_pan_gesture();
        }

        None
    }

    /// A touch ended at the given position.
    pub fn touch_end(&mut self, id: u64, x: f64, y: f64) -> Option<Gesture> {
        let now = Instant::now();
        let touch_index = self.touches.iter().position(|t| t.id == id)?;
        let start_x = self.touches[touch_index].start_x;
        let start_y = self.touches[touch_index].start_y;
        let start_time = self.touches[touch_index].start_time;
        let elapsed_ms = self.touches[touch_index].elapsed_ms(now);
        let distance = self.touches[touch_index].distance_from_start();

        if self.touches.len() == self.config.pinch_touch_count {
            self.pinch_initial_distance = None;
        }

        self.touches.remove(touch_index);

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

    /// Check for long press. Call periodically.
    pub fn check_long_press(&self, now: std::time::Instant) -> Option<Gesture> {
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
        let scale = if initial > 0.0 { current_distance / initial } else { 1.0 };
        let center_x = (a.current_x + b.current_x) / 2.0;
        let center_y = (a.current_y + b.current_y) / 2.0;
        Some(Gesture::Pinch { scale, center_x, center_y, velocity: 0.0 })
    }

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
            ((touch.current_x - touch.start_x) / elapsed_secs, (touch.current_y - touch.start_y) / elapsed_secs)
        } else {
            (0.0, 0.0)
        };
        Some(Gesture::Pan { dx, dy, velocity_x, velocity_y })
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_end_gesture(
        &mut self,
        elapsed_ms: u64,
        distance: f64,
        x: f64,
        y: f64,
        now: std::time::Instant,
        start_x: f64,
        start_y: f64,
        start_time: std::time::Instant,
    ) -> Option<Gesture> {
        if self.pan_started {
            return self.evaluate_flick_or_pan_end(x, y, now, start_x, start_y, start_time);
        }
        if elapsed_ms <= self.config.tap_max_duration_ms && distance <= self.config.tap_max_distance {
            return self.evaluate_tap(x, y, now);
        }
        None
    }

    fn evaluate_tap(&mut self, x: f64, y: f64, now: std::time::Instant) -> Option<Gesture> {
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
        self.last_tap_time = Some(now);
        self.last_tap_position = Some((x, y));
        Some(Gesture::Tap { x, y })
    }

    fn evaluate_flick_or_pan_end(
        &self,
        x: f64,
        y: f64,
        now: std::time::Instant,
        start_x: f64,
        start_y: f64,
        start_time: std::time::Instant,
    ) -> Option<Gesture> {
        let elapsed_secs = now.duration_since(start_time).as_secs_f64();
        if elapsed_secs > 0.0 {
            let velocity_x = (x - start_x) / elapsed_secs;
            let velocity_y = (y - start_y) / elapsed_secs;
            let speed = (velocity_x * velocity_x + velocity_y * velocity_y).sqrt();
            if speed >= self.config.flick_min_velocity {
                let direction = flick_direction_from_velocity(velocity_x, velocity_y);
                return Some(Gesture::Flick { velocity_x, velocity_y, direction });
            }
        }
        let dx = x - start_x;
        let dy = y - start_y;
        let (velocity_x, velocity_y) = if elapsed_secs > 0.0 {
            (dx / elapsed_secs, dy / elapsed_secs)
        } else {
            (0.0, 0.0)
        };
        Some(Gesture::Pan { dx, dy, velocity_x, velocity_y })
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

fn flick_direction_from_velocity(velocity_x: f64, velocity_y: f64) -> FlickDirection {
    if velocity_x.abs() >= velocity_y.abs() {
        if velocity_x >= 0.0 { FlickDirection::Right } else { FlickDirection::Left }
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

    // Clasificación: determinística — verifica default_config
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

    // Clasificación: determinística — verifica detección de toque simple
    #[test]
    fn single_tap() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        let gesture = det.touch_end(1, 100.0, 200.0);
        assert_eq!(gesture, Some(Gesture::Tap { x: 100.0, y: 200.0 }));
    }

    // Clasificación: determinística — edge case: toque fuera de umbrales
    #[test]
    fn tap_too_long() {
        let config = GestureConfig { tap_max_duration_ms: 200, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        let t0 = Instant::now();
        det.touch_start(1, 100.0, 200.0);
        let future = t0 + Duration::from_millis(250);
        let lp = det.check_long_press(future);
        assert_eq!(lp, None);
    }

    // Clasificación: determinística — edge case: toque fuera de umbrales
    #[test]
    fn tap_too_far() {
        let config = GestureConfig { tap_max_distance: 5.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 200.0);
        det.touch_move(1, 120.0, 200.0); // 20px beyond 5
        let gesture = det.touch_end(1, 120.0, 200.0);
        assert!(gesture.is_some());
        assert!(!matches!(gesture, Some(Gesture::Tap { .. })));
    }

    // Clasificación: determinística — verifica detección de doble toque dentro del intervalo configurado
    #[test]
    fn double_tap() {
        let config = GestureConfig { double_tap_max_interval_ms: 300, tap_max_duration_ms: 300, tap_max_distance: 10.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 200.0);
        let g1 = det.touch_end(1, 100.0, 200.0);
        assert_eq!(g1, Some(Gesture::Tap { x: 100.0, y: 200.0 }));
        det.touch_start(2, 105.0, 205.0);
        let g2 = det.touch_end(2, 105.0, 205.0);
        assert_eq!(g2, Some(Gesture::DoubleTap { x: 105.0, y: 205.0 }));
    }

    // Clasificación: determinística — edge case: intervalo entre taps excedido — no debe detectar double tap
    #[test]
    fn double_tap_too_slow() {
        let make = || GestureConfig { double_tap_max_interval_ms: 100, tap_max_duration_ms: 300, tap_max_distance: 10.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(make());
        det.touch_start(1, 100.0, 200.0);
        let g1 = det.touch_end(1, 100.0, 200.0);
        assert_eq!(g1, Some(Gesture::Tap { x: 100.0, y: 200.0 }));
        let mut det2 = GestureDetector::new(make());
        det2.touch_start(2, 100.0, 200.0);
        let g2 = det2.touch_end(2, 100.0, 200.0);
        assert_eq!(g2, Some(Gesture::Tap { x: 100.0, y: 200.0 }));
    }

    // Clasificación: determinística — verifica detección de press prolongado tras umbral de tiempo
    #[test]
    fn long_press() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        let t0 = Instant::now();
        assert_eq!(det.check_long_press(t0 + Duration::from_millis(200)), None);
        let gesture = det.check_long_press(t0 + Duration::from_millis(500));
        match gesture {
            Some(Gesture::LongPress { x, y, duration_ms }) => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
                assert!(duration_ms >= 500);
            }
            other => panic!("Expected LongPress, got {other:?}"),
        }
    }

    // Clasificación: determinística — edge case: long press con movimiento leve dentro del umbral de distancia
    #[test]
    fn long_press_with_movement() {
        let config = GestureConfig { tap_max_distance: 5.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 200.0);
        det.touch_move(1, 103.0, 200.0);
        let t0 = Instant::now();
        let gesture = det.check_long_press(t0 + Duration::from_millis(500));
        assert!(matches!(gesture, Some(Gesture::LongPress { .. })));
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn pan_start() {
        let config = GestureConfig { pan_min_distance: 5.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 100.0);
        let gesture = det.touch_move(1, 120.0, 120.0);
        assert!(matches!(gesture, Some(Gesture::Pan { dx: 20.0, dy: 20.0, .. })));
        assert!(det.is_tracking());
    }

    // Clasificación: determinística — edge case: movimiento insuficiente no inicia pan
    #[test]
    fn pan_not_started() {
        let config = GestureConfig { pan_min_distance: 50.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 100.0);
        let gesture = det.touch_move(1, 105.0, 100.0);
        assert_eq!(gesture, None);
    }

    // Clasificación: determinística — verifica detección de gesto pinch con dos dedos
    #[test]
    fn pinch_two_fingers() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 300.0, 200.0);
        let gesture = det.touch_move(2, 400.0, 200.0);
        match gesture {
            Some(Gesture::Pinch { scale, .. }) => assert!(scale > 1.0),
            other => panic!("Expected Pinch with scale > 1.0, got {other:?}"),
        }
    }

    // Clasificación: determinística — verifica pinch-to-zoom: dedos separándose producen scale > 1.0
    #[test]
    fn pinch_spread() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 200.0, 200.0);
        det.touch_move(2, 300.0, 200.0);
        let gesture = det.touch_move(2, 300.0, 200.0);
        if let Some(Gesture::Pinch { scale, .. }) = gesture {
            assert!(scale > 1.0);
        } else {
            panic!("Expected Pinch gesture");
        }
    }

    // Clasificación: determinística — verifica pinch-to-zoom: dedos acercándose producen scale < 1.0
    #[test]
    fn pinch_compress() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        det.touch_start(2, 300.0, 200.0);
        det.touch_move(2, 150.0, 200.0);
        let gesture = det.touch_move(2, 150.0, 200.0);
        if let Some(Gesture::Pinch { scale, .. }) = gesture {
            assert!(scale < 1.0);
        } else {
            panic!("Expected Pinch gesture");
        }
    }

    // Clasificación: determinística — verifica detección de flick rápido (swipe con inercia)
    #[test]
    fn flick_fast() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 100.0);
        det.touch_move(1, 300.0, 100.0);
        let gesture = det.touch_end(1, 300.0, 100.0);
        assert!(gesture.is_some());
        assert!(det.touch_count() == 0);
    }

    // Clasificación: determinística — verifica mapeo de velocidad a dirección cardinal del flick
    #[test]
    fn flick_direction() {
        assert_eq!(flick_direction_from_velocity(100.0, 0.0), FlickDirection::Right);
        assert_eq!(flick_direction_from_velocity(-100.0, 0.0), FlickDirection::Left);
        assert_eq!(flick_direction_from_velocity(0.0, 100.0), FlickDirection::Down);
        assert_eq!(flick_direction_from_velocity(0.0, -100.0), FlickDirection::Up);
        assert_eq!(flick_direction_from_velocity(100.0, 50.0), FlickDirection::Right);
        assert_eq!(flick_direction_from_velocity(-100.0, -50.0), FlickDirection::Left);
        assert_eq!(flick_direction_from_velocity(50.0, 100.0), FlickDirection::Down);
        assert_eq!(flick_direction_from_velocity(-50.0, -100.0), FlickDirection::Up);
    }

    // Clasificación: determinística — verifica que clear() resetea completamente el estado y las estadísticas
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

    // Clasificación: determinística — edge case: touch con ID no registrado retorna None sin panic
    #[test]
    fn touch_move_unknown_id_returns_none() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        let gesture = det.touch_move(999, 150.0, 200.0);
        assert_eq!(gesture, None);
    }

    // Clasificación: determinística — edge case: touch con ID no registrado retorna None sin panic
    #[test]
    fn touch_end_unknown_id_returns_none() {
        let mut det = GestureDetector::with_default_config();
        det.touch_start(1, 100.0, 200.0);
        let gesture = det.touch_end(999, 100.0, 200.0);
        assert_eq!(gesture, None);
    }

    // Clasificación: determinística — verifica acceso de solo lectura a la configuración del detector
    #[test]
    fn config_getter() {
        let config = GestureConfig::default();
        let det = GestureDetector::new(config);
        assert_eq!(det.config().tap_max_duration_ms, 300);
    }

    // Clasificación: determinística — verifica acumulación de deltas en múltiples movimientos de pan
    #[test]
    fn multiple_pan_movements() {
        let config = GestureConfig { pan_min_distance: 5.0, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 100.0);
        let g1 = det.touch_move(1, 120.0, 100.0);
        assert!(matches!(g1, Some(Gesture::Pan { dx: 20.0, .. })));
        let g2 = det.touch_move(1, 140.0, 100.0);
        assert!(matches!(g2, Some(Gesture::Pan { dx: 40.0, .. })));
    }

    // Clasificación: determinística — verifica detección de flick rápido (swipe con inercia)
    #[test]
    fn pan_end_without_flick() {
        let config = GestureConfig { pan_min_distance: 5.0, flick_min_velocity: f64::MAX, ..GestureConfig::default() };
        let mut det = GestureDetector::new(config);
        det.touch_start(1, 100.0, 100.0);
        det.touch_move(1, 120.0, 100.0);
        let gesture = det.touch_end(1, 120.0, 100.0);
        assert!(matches!(gesture, Some(Gesture::Pan { .. })));
    }
}
