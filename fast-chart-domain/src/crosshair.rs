use crate::bar::Bar;
use crate::scale::{LinearScale, TimeScale};

/// Magnet mode for crosshair snapping to OHLC values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnetMode {
    /// No snapping — use interpolated price.
    Off,
    /// Snap to nearest OHLC value.
    OHLC,
    /// Snap to nearest extreme (high/low).
    Extreme,
}

impl Default for MagnetMode {
    fn default() -> Self {
        Self::Off
    }
}

/// Find the nearest bar to a given timestamp in a sorted slice.
pub fn find_nearest_bar(bars: &[Bar], timestamp: u64) -> Option<&Bar> {
    if bars.is_empty() {
        return None;
    }

    // Binary search for nearest timestamp
    match bars.binary_search_by_key(&timestamp, |b| b.timestamp) {
        Ok(idx) => Some(&bars[idx]),
        Err(idx) => {
            if idx == 0 {
                Some(&bars[0])
            } else if idx >= bars.len() {
                Some(&bars[bars.len() - 1])
            } else {
                // Return whichever is closer
                let prev = &bars[idx - 1];
                let next = &bars[idx];
                if timestamp - prev.timestamp < next.timestamp - timestamp {
                    Some(prev)
                } else {
                    Some(next)
                }
            }
        }
    }
}

/// Snap a price to the nearest OHLC value of a bar.
pub fn snap_to_ohlc(price: f64, bar: &Bar, mode: MagnetMode) -> f64 {
    match mode {
        MagnetMode::Off => price,
        MagnetMode::OHLC => {
            let candidates = [bar.open, bar.high, bar.low, bar.close];
            candidates
                .iter()
                .min_by(|a, b| {
                    (**a - price)
                        .abs()
                        .partial_cmp(&(*b - price).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
                .unwrap_or(price)
        }
        MagnetMode::Extreme => {
            let candidates = [bar.high, bar.low];
            candidates
                .iter()
                .min_by(|a, b| {
                    (**a - price)
                        .abs()
                        .partial_cmp(&(*b - price).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
                .unwrap_or(price)
        }
    }
}

/// Crosshair position tracking with optional OHLC magnet snapping.
///
/// # Examples
///
/// ```
/// use fast_chart_domain::Crosshair;
/// use fast_chart_domain::scale::{TimeScale, LinearScale};
///
/// let ts = TimeScale { start: 0, end: 2000, width: 800.0 };
/// let vs = LinearScale { min: 90.0, max: 120.0, height: 300.0 };
///
/// let mut ch = Crosshair::default();
/// assert!(!ch.active);
///
/// // Update position from screen coordinates
/// ch.update(400.0, 150.0, &ts, &vs);
/// assert!(ch.active);
/// assert_eq!(ch.time, 1000);
/// ```
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

    /// Update with magnet snapping.
    pub fn update_with_magnet(
        &mut self,
        x: f64,
        y: f64,
        time_scale: &TimeScale,
        value_scale: &LinearScale,
        bars: &[Bar],
        magnet_mode: MagnetMode,
    ) {
        self.screen_x = x;
        self.screen_y = y;
        self.time = time_scale.map_from_x(x);

        let interpolated_price = value_scale.map_from_y(y);

        if magnet_mode != MagnetMode::Off {
            if let Some(bar) = find_nearest_bar(bars, self.time) {
                self.price = snap_to_ohlc(interpolated_price, bar, magnet_mode);
            } else {
                self.price = interpolated_price;
            }
        } else {
            self.price = interpolated_price;
        }

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

#[cfg(test)]
mod magnet_tests {
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

    fn test_bars() -> Vec<Bar> {
        vec![
            Bar {
                timestamp: 100,
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 105.0,
                volume: 1000,
            },
            Bar {
                timestamp: 200,
                open: 105.0,
                high: 115.0,
                low: 100.0,
                close: 110.0,
                volume: 1200,
            },
            Bar {
                timestamp: 300,
                open: 110.0,
                high: 120.0,
                low: 105.0,
                close: 115.0,
                volume: 800,
            },
        ]
    }

    #[test]
    fn find_nearest_bar_exact() {
        let bars = test_bars();
        let bar = find_nearest_bar(&bars, 200).unwrap();
        assert_eq!(bar.timestamp, 200);
    }

    #[test]
    fn find_nearest_bar_between() {
        let bars = test_bars();
        // 150 is between 100 and 200, closer to 200
        let bar = find_nearest_bar(&bars, 150).unwrap();
        assert_eq!(bar.timestamp, 200);
    }

    #[test]
    fn find_nearest_bar_before_first() {
        let bars = test_bars();
        let bar = find_nearest_bar(&bars, 0).unwrap();
        assert_eq!(bar.timestamp, 100);
    }

    #[test]
    fn find_nearest_bar_after_last() {
        let bars = test_bars();
        let bar = find_nearest_bar(&bars, 999).unwrap();
        assert_eq!(bar.timestamp, 300);
    }

    #[test]
    fn find_nearest_bar_empty() {
        let bars: Vec<Bar> = vec![];
        assert!(find_nearest_bar(&bars, 100).is_none());
    }

    #[test]
    fn snap_to_ohlc_off() {
        let bar = Bar {
            timestamp: 100,
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            volume: 1000,
        };
        assert_eq!(snap_to_ohlc(107.0, &bar, MagnetMode::Off), 107.0);
    }

    #[test]
    fn snap_to_ohlc_mode() {
        let bar = Bar {
            timestamp: 100,
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            volume: 1000,
        };
        // 107 is closest to close (105)
        assert_eq!(snap_to_ohlc(107.0, &bar, MagnetMode::OHLC), 105.0);
        // 109 is closest to high (110)
        assert_eq!(snap_to_ohlc(109.0, &bar, MagnetMode::OHLC), 110.0);
    }

    #[test]
    fn snap_to_extreme_mode() {
        let bar = Bar {
            timestamp: 100,
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            volume: 1000,
        };
        // Only high and low are candidates
        assert_eq!(snap_to_ohlc(107.0, &bar, MagnetMode::Extreme), 110.0);
        assert_eq!(snap_to_ohlc(98.0, &bar, MagnetMode::Extreme), 95.0);
    }

    #[test]
    fn crosshair_update_with_magnet() {
        let mut ch = Crosshair::default();
        let ts = test_time_scale();
        let vs = test_value_scale();
        let bars = test_bars();

        ch.update_with_magnet(400.0, 250.0, &ts, &vs, &bars, MagnetMode::OHLC);
        assert!(ch.active);
        // Price should be snapped to nearest OHLC of the nearest bar
    }
}
