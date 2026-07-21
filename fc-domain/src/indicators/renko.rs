use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Renko chart indicator.
///
/// Renko charts plot price movement only, ignoring time. A new brick is
/// added when the price moves by at least `brick_size` from the previous
/// brick's level.
///
/// **Brick size**:
/// - If `brick_size > 0.0`, it is used directly as the fixed brick size.
/// - If `brick_size == 0.0`, the ATR (period `atr_period`) is used instead.
///
/// The output is a series of brick closing prices (one per brick).
///
/// Defaults: `brick_size: 5.0`, `atr_period: 14`.
pub struct Renko {
    /// Fixed brick size in price units. Set to `0.0` to use ATR-based sizing.
    pub brick_size: f64,
    /// ATR period used when `brick_size` is `0.0`.
    pub atr_period: usize,
}

impl Default for Renko {
    fn default() -> Self {
        Self {
            brick_size: 5.0,
            atr_period: 14,
        }
    }
}

impl Renko {
    /// Compute ATR using Wilder's smoothing (same logic as the `Atr` indicator).
    ///
    /// Returns the latest ATR value for the given series, or `None` when
    /// there are insufficient bars.
    fn compute_atr(bars: &[&Bar], period: usize) -> Option<f64> {
        if bars.len() < period + 1 {
            return None;
        }

        let mut tr_values: Vec<f64> = Vec::with_capacity(bars.len() - 1);
        for i in 1..bars.len() {
            let hl = bars[i].high - bars[i].low;
            let hpc = (bars[i].high - bars[i - 1].close).abs();
            let lpc = (bars[i].low - bars[i - 1].close).abs();
            tr_values.push(f64::max(hl, f64::max(hpc, lpc)));
        }

        let initial_atr: f64 =
            tr_values[..period].iter().sum::<f64>() / period as f64;

        let mut prev_atr = initial_atr;
        for &tr in &tr_values[period..] {
            let atr = (prev_atr * (period as f64 - 1.0) + tr) / period as f64;
            prev_atr = atr;
        }
        Some(prev_atr)
    }
}

impl Indicator<MAX_SERIES_LEN> for Renko {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();

        if bars.is_empty() {
            return result;
        }

        // Determine brick size
        let effective_brick = if self.brick_size > 0.0 {
            self.brick_size
        } else {
            match Self::compute_atr(&bars, self.atr_period) {
                Some(atr) if atr > 0.0 => atr,
                _ => return result,
            }
        };

        // Seed with the first bar's close as the initial brick price
        let mut last_brick_price = bars[0].close;
        result.push(last_brick_price);

        for &bar in &bars[1..] {
            // Use the close price to determine brick changes
            let close = bar.close;
            // Add bullish bricks if price moved up enough
            while close - last_brick_price >= effective_brick {
                last_brick_price += effective_brick;
                result.push(last_brick_price);
            }

            // Add bearish bricks if price moved down enough
            while last_brick_price - close >= effective_brick {
                last_brick_price -= effective_brick;
                result.push(last_brick_price);
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Renko"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bars(count: usize) -> TimeSeries<Bar, MAX_SERIES_LEN> {
        let mut s = TimeSeries::new();
        for i in 0..count {
            let p = 100.0 + (i as f64 * 0.1).sin() * 10.0;
            s.push(Bar {
                timestamp: i as u64 * 60000,
                open: p,
                high: p + 1.0,
                low: p - 1.0,
                close: p + 0.5,
                volume: 1000,
            });
        }
        s
    }

    #[test]
    fn renko_name() {
        let renko = Renko::default();
        assert_eq!(renko.name(), "Renko");
    }

    #[test]
    fn renko_empty_series() {
        let series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        let renko = Renko::default();
        let result = renko.calculate(&series);
        assert!(result.is_empty());
    }

    #[test]
    fn renko_single_bar() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 105.0,
            low: 99.0,
            close: 102.0,
            volume: 1000,
        });
        let renko = Renko::default();
        let result = renko.calculate(&series);
        // Single bar → only the initial brick price
        assert_eq!(result.len(), 1);
        assert!((result.get(0).unwrap() - 102.0).abs() < f64::EPSILON);
    }

    #[test]
    fn renko_basic() {
        let bars = make_bars(50);
        let renko = Renko { brick_size: 2.0, atr_period: 14 };
        let result = renko.calculate(&bars);
        // With varying prices there should be multiple bricks
        assert!(result.len() >= 1);
    }

    #[test]
    fn renko_exact_period() {
        // 15 bars → enough for ATR (period + 1 = 15)
        let bars = make_bars(15);
        let renko = Renko { brick_size: 0.0, atr_period: 14 };
        let result = renko.calculate(&bars);
        // Should produce bricks (at least the initial one)
        assert!(result.len() >= 1);
    }

    #[test]
    fn renko_insufficient_data_for_atr() {
        // 10 bars < period + 1 = 15, ATR mode returns empty
        let bars = make_bars(10);
        let renko = Renko { brick_size: 0.0, atr_period: 14 };
        let result = renko.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn renko_fixed_brick_size() {
        // Manually construct bars that produce clear brick movements
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // Start at 100
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        // Move to 102 → should add brick at 101 and 102
        series.push(Bar { timestamp: 1, open: 100.0, high: 102.5, low: 99.5, close: 102.0, volume: 1000 });
        // Move to 104 → should add brick at 103 and 104
        series.push(Bar { timestamp: 2, open: 102.0, high: 104.5, low: 101.5, close: 104.0, volume: 1000 });

        let renko = Renko { brick_size: 1.0, atr_period: 14 };
        let result = renko.calculate(&series);

        // Initial (100) + 2 bricks (101, 102) + 2 bricks (103, 104) = 5
        assert_eq!(result.len(), 5);
        let expected = [100.0, 101.0, 102.0, 103.0, 104.0];
        for (i, &exp) in expected.iter().enumerate() {
            let val = result.get(i).unwrap();
            assert!((val - exp).abs() < 1e-10, "brick {i}: expected {exp}, got {val}");
        }
    }

    #[test]
    fn renko_bearish_bricks() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        series.push(Bar { timestamp: 1, open: 100.0, high: 100.5, low: 97.0, close: 97.0, volume: 1000 });
        series.push(Bar { timestamp: 2, open: 97.0, high: 97.5, low: 94.0, close: 94.0, volume: 1000 });

        let renko = Renko { brick_size: 1.0, atr_period: 14 };
        let result = renko.calculate(&series);

        // Initial (100) + 3 bearish (99, 98, 97) + 3 bearish (96, 95, 94) = 7
        assert_eq!(result.len(), 7);
        let expected = [100.0, 99.0, 98.0, 97.0, 96.0, 95.0, 94.0];
        for (i, &exp) in expected.iter().enumerate() {
            let val = result.get(i).unwrap();
            assert!((val - exp).abs() < 1e-10, "brick {i}: expected {exp}, got {val}");
        }
    }

    #[test]
    fn renko_flat_prices_no_new_bricks() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            series.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.5,
                low: 99.5,
                close: 100.0,
                volume: 1000,
            });
        }
        let renko = Renko { brick_size: 2.0, atr_period: 14 };
        let result = renko.calculate(&series);
        // Flat prices → only the initial brick
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn renko_large_move() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        // Move from 100 to 110 in one bar — should produce 10 bricks
        series.push(Bar { timestamp: 1, open: 100.0, high: 110.5, low: 99.5, close: 110.0, volume: 1000 });

        let renko = Renko { brick_size: 1.0, atr_period: 14 };
        let result = renko.calculate(&series);
        // Initial + 10 bricks = 11
        assert_eq!(result.len(), 11);
    }

    #[test]
    fn renko_direction_change() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar { timestamp: 0, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000 });
        // Up 3 bricks
        series.push(Bar { timestamp: 1, open: 100.0, high: 103.5, low: 99.5, close: 103.0, volume: 1000 });
        // Down 6 bricks — reversal
        series.push(Bar { timestamp: 2, open: 103.0, high: 103.5, low: 96.0, close: 97.0, volume: 1000 });

        let renko = Renko { brick_size: 1.0, atr_period: 14 };
        let result = renko.calculate(&series);

        // Initial (100) + 3 up (101, 102, 103) + 6 down (102, 101, 100, 99, 98, 97) = 10
        assert_eq!(result.len(), 10);
        // Last brick should be at 97
        assert!((result.latest().unwrap() - 97.0).abs() < 1e-10);
    }

    #[test]
    fn renko_atr_mode() {
        let bars = make_bars(50);
        let renko = Renko { brick_size: 0.0, atr_period: 14 };
        let result = renko.calculate(&bars);
        // Should produce bricks from ATR-based sizing
        assert!(result.len() >= 1);
    }

    #[test]
    fn renko_atr_mode_insufficient_bars() {
        let bars = make_bars(5);
        let renko = Renko { brick_size: 0.0, atr_period: 14 };
        let result = renko.calculate(&bars);
        assert!(result.is_empty());
    }
}
