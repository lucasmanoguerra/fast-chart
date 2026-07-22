use std::time::{Duration, Instant};

/// Lightweight frame-time counter — logs FPS once per second.
pub struct FrameCounter {
    frame_count: u32,
    last_time: Instant,
    fps: f64,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_time: Instant::now(),
            fps: 0.0,
        }
    }

    /// Call once per frame. Returns current FPS when updated, None otherwise.
    pub fn tick(&mut self) -> Option<f64> {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time);
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_time = now;
            Some(self.fps)
        } else {
            None
        }
    }

    /// Get the last computed FPS value.
    pub fn fps(&self) -> f64 {
        self.fps
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica frame_counter_new
    #[test]
    fn frame_counter_new() {
        let fc = FrameCounter::new();
        assert_eq!(fc.fps(), 0.0);
    }

    // Clasificación: determinística — verifica frame_counter_tick_returns_none_within_second
    #[test]
    fn frame_counter_tick_returns_none_within_second() {
        let mut fc = FrameCounter::new();
        // First tick won't return Some because elapsed < 1s
        assert!(fc.tick().is_none());
    }
}
