#[derive(Debug, Clone)]
pub struct Viewport {
    pub time_start: u64,
    pub time_end: u64,
    pub value_min: f64,
    pub value_max: f64,
    pub zoom_level: f64,
}

impl Viewport {
    pub fn contains_time(&self, time: u64) -> bool {
        time >= self.time_start && time <= self.time_end
    }

    pub fn zoom(&mut self, factor: f64, center: f64) {
        let time_range = self.time_end as f64 - self.time_start as f64;
        let new_range = time_range / factor;
        let center_ratio = (center - self.time_start as f64) / time_range;

        let new_start = center - new_range * center_ratio;
        let new_end = center + new_range * (1.0 - center_ratio);

        self.time_start = new_start.max(0.0) as u64;
        self.time_end = new_end.max(self.time_start as f64 + 1.0) as u64;
        self.zoom_level *= factor;
    }

    pub fn pan(&mut self, time_delta: i64) {
        if time_delta >= 0 {
            let delta = time_delta as u64;
            self.time_start = self.time_start.saturating_add(delta);
            self.time_end = self.time_end.saturating_add(delta);
        } else {
            let delta = (-time_delta) as u64;
            self.time_start = self.time_start.saturating_sub(delta);
            self.time_end = self.time_end.saturating_sub(delta);
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            time_start: 0,
            time_end: 3600_000,
            value_min: 0.0,
            value_max: 100.0,
            zoom_level: 1.0,
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
            value_min: 50.0,
            value_max: 150.0,
            zoom_level: 1.0,
        }
    }

    #[test]
    fn contains_time_inside() {
        let vp = test_viewport();
        assert!(vp.contains_time(1500));
    }

    #[test]
    fn contains_time_at_boundary() {
        let vp = test_viewport();
        assert!(vp.contains_time(1000));
        assert!(vp.contains_time(2000));
    }

    #[test]
    fn contains_time_outside() {
        let vp = test_viewport();
        assert!(!vp.contains_time(999));
        assert!(!vp.contains_time(2001));
    }

    #[test]
    fn zoom_in() {
        let mut vp = test_viewport();
        vp.zoom(2.0, 1500.0);
        let range = vp.time_end - vp.time_start;
        assert_eq!(range, 500); // halved from 1000
        assert_eq!(vp.zoom_level, 2.0);
    }

    #[test]
    fn zoom_out() {
        let mut vp = test_viewport();
        vp.zoom(0.5, 1500.0);
        assert_eq!(vp.zoom_level, 0.5);
    }

    #[test]
    fn pan_forward() {
        let mut vp = test_viewport();
        vp.pan(500);
        assert_eq!(vp.time_start, 1500);
        assert_eq!(vp.time_end, 2500);
    }

    #[test]
    fn pan_backward() {
        let mut vp = test_viewport();
        vp.pan(-500);
        assert_eq!(vp.time_start, 500);
        assert_eq!(vp.time_end, 1500);
    }

    #[test]
    fn pan_saturate_at_zero() {
        let mut vp = test_viewport();
        vp.pan(-2000);
        assert_eq!(vp.time_start, 0);
    }

    #[test]
    fn default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.zoom_level, 1.0);
        assert_eq!(vp.time_start, 0);
    }
}
