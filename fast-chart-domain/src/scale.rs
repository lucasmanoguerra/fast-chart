#[derive(Debug, Clone)]
pub struct LinearScale {
    pub min: f64,
    pub max: f64,
    pub height: f64,
}

impl LinearScale {
    pub fn map_to_y(&self, value: f64) -> f64 {
        if (self.max - self.min).abs() < f64::EPSILON {
            return self.height / 2.0;
        }
        let ratio = (value - self.min) / (self.max - self.min);
        self.height * (1.0 - ratio)
    }

    pub fn map_from_y(&self, y: f64) -> f64 {
        if self.height.abs() < f64::EPSILON {
            return (self.min + self.max) / 2.0;
        }
        let ratio = 1.0 - (y / self.height);
        self.min + ratio * (self.max - self.min)
    }
}

#[derive(Debug, Clone)]
pub struct TimeScale {
    pub start: u64,
    pub end: u64,
    pub width: f64,
}

impl TimeScale {
    pub fn map_to_x(&self, time: u64) -> f64 {
        let range = self.end as f64 - self.start as f64;
        if range < f64::EPSILON {
            return self.width / 2.0;
        }
        let ratio = (time as f64 - self.start as f64) / range;
        self.width * ratio
    }

    pub fn map_from_x(&self, x: f64) -> u64 {
        let range = self.end as f64 - self.start as f64;
        if self.width < f64::EPSILON {
            return (self.start + self.end) / 2;
        }
        let ratio = x / self.width;
        (self.start as f64 + ratio * range) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_midpoint() {
        let scale = LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(105.0), 250.0);
    }

    #[test]
    fn linear_top() {
        let scale = LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(110.0), 0.0);
    }

    #[test]
    fn linear_bottom() {
        let scale = LinearScale {
            min: 100.0,
            max: 110.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(100.0), 500.0);
    }

    #[test]
    fn linear_roundtrip() {
        let scale = LinearScale {
            min: 50.0,
            max: 200.0,
            height: 600.0,
        };
        let value = 125.0;
        let y = scale.map_to_y(value);
        let back = scale.map_from_y(y);
        assert!((back - value).abs() < f64::EPSILON);
    }

    #[test]
    fn linear_equal_min_max() {
        let scale = LinearScale {
            min: 100.0,
            max: 100.0,
            height: 500.0,
        };
        assert_eq!(scale.map_to_y(100.0), 250.0);
    }

    #[test]
    fn time_midpoint() {
        let scale = TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
        };
        assert_eq!(scale.map_to_x(500), 400.0);
    }

    #[test]
    fn time_start() {
        let scale = TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
        };
        assert_eq!(scale.map_to_x(0), 0.0);
    }

    #[test]
    fn time_end() {
        let scale = TimeScale {
            start: 0,
            end: 1000,
            width: 800.0,
        };
        assert_eq!(scale.map_to_x(1000), 800.0);
    }

    #[test]
    fn time_roundtrip() {
        let scale = TimeScale {
            start: 1000,
            end: 2000,
            width: 600.0,
        };
        let time = 1500u64;
        let x = scale.map_to_x(time);
        let back = scale.map_from_x(x);
        assert_eq!(back, time);
    }

    #[test]
    fn time_equal_start_end() {
        let scale = TimeScale {
            start: 1000,
            end: 1000,
            width: 800.0,
        };
        assert_eq!(scale.map_to_x(1000), 400.0);
    }
}
