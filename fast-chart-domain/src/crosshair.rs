use crate::scale::{LinearScale, TimeScale};

#[derive(Debug, Clone)]
pub struct Crosshair {
    pub screen_x: f64,
    pub screen_y: f64,
    pub time: u64,
    pub price: f64,
    pub active: bool,
}

impl Default for Crosshair {
    fn default() -> Self {
        Self {
            screen_x: 0.0,
            screen_y: 0.0,
            time: 0,
            price: 0.0,
            active: false,
        }
    }
}

impl Crosshair {
    pub fn update(&mut self, x: f64, y: f64, time_scale: &TimeScale, value_scale: &LinearScale) {
        self.screen_x = x;
        self.screen_y = y;
        self.time = time_scale.map_from_x(x);
        self.price = value_scale.map_from_y(y);
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_time_scale() -> TimeScale {
        TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
        }
    }

    fn test_value_scale() -> LinearScale {
        LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        }
    }

    #[test]
    fn update_sets_position_and_active() {
        let mut ch = Crosshair::default();
        let ts = test_time_scale();
        let vs = test_value_scale();
        ch.update(400.0, 250.0, &ts, &vs);
        assert!(ch.active);
        assert_eq!(ch.screen_x, 400.0);
        assert_eq!(ch.screen_y, 250.0);
        assert_eq!(ch.time, 500);
        assert!((ch.price - 105.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deactivate() {
        let mut ch = Crosshair::default();
        let ts = test_time_scale();
        let vs = test_value_scale();
        ch.update(100.0, 100.0, &ts, &vs);
        assert!(ch.active);
        ch.deactivate();
        assert!(!ch.active);
    }

    #[test]
    fn default_is_inactive() {
        let ch = Crosshair::default();
        assert!(!ch.active);
    }

    #[test]
    fn update_preserves_last_valid_state() {
        let mut ch = Crosshair::default();
        let ts = test_time_scale();
        let vs = test_value_scale();
        ch.update(400.0, 250.0, &ts, &vs);
        let saved_time = ch.time;
        let saved_price = ch.price;
        // Update with new coords
        ch.update(200.0, 100.0, &ts, &vs);
        assert_ne!(ch.time, saved_time);
        assert_ne!(ch.price, saved_price);
    }
}
