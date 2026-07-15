use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Williams %R (Williams Percent Range).
///
/// A momentum oscillator that measures overbought and oversold levels,
/// similar to the Stochastic oscillator but inverted.
///
/// Formula:
/// %R = (Highest High - Close) / (Highest High - Lowest Low) × (-100)
///
/// Range: -100 to 0
/// - Above -20 → overbought
/// - Below -80 → oversold
///
/// Default period: 14
pub struct WilliamsR {
    pub period: usize,
}

impl WilliamsR {
    /// Creates a Williams %R indicator with the default period of 14.
    pub fn default_period() -> Self {
        Self { period: 14 }
    }
}

impl Indicator<MAX_SERIES_LEN> for WilliamsR {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period {
            return result;
        }

        // For each bar starting at index `period - 1`, compute %R
        // using the window [i - period + 1 ..= i]
        for i in (self.period - 1)..bars.len() {
            let start = i + 1 - self.period;
            let mut highest_high = f64::NEG_INFINITY;
            let mut lowest_low = f64::INFINITY;

            for &bar in &bars[start..=i] {
                if bar.high > highest_high {
                    highest_high = bar.high;
                }
                if bar.low < lowest_low {
                    lowest_low = bar.low;
                }
            }

            let range = highest_high - lowest_low;
            let williams_r = if range.abs() < f64::EPSILON {
                // When all highs and lows are identical, %R is at 0 (no range)
                0.0
            } else {
                ((highest_high - bars[i].close) / range) * -100.0
            };

            result.push(williams_r);
        }

        result
    }

    fn name(&self) -> &str {
        "Williams %R"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bar::Bar;

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
    fn williams_r_name() {
        let wr = WilliamsR { period: 14 };
        assert_eq!(wr.name(), "Williams %R");
    }

    #[test]
    fn williams_r_insufficient_data() {
        let bars = make_bars(5);
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn williams_r_exact_period() {
        let bars = make_bars(14);
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        // Exactly period bars → 1 output value
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn williams_r_basic() {
        let bars = make_bars(50);
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        // 50 bars, period 14 → 50 - 14 + 1 = 37 outputs
        assert_eq!(result.len(), 37);
    }

    #[test]
    fn williams_r_bounds() {
        let bars = make_bars(100);
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        // All values should be in range [-100, 0]
        assert!(result.iter().all(|v| *v >= -100.0 && *v <= 0.0));
    }

    #[test]
    fn williams_r_overbought() {
        // Close at highest high → %R = 0 (maximum overbought)
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..14 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 110.0, // close = highest high
                volume: 1000,
            });
        }
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        assert_eq!(result.len(), 1);
        // (110 - 110) / (110 - 90) * -100 = 0
        assert_eq!(*result.latest().unwrap(), 0.0);
    }

    #[test]
    fn williams_r_oversold() {
        // Close at lowest low → %R = -100 (maximum oversold)
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..14 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 90.0, // close = lowest low
                volume: 1000,
            });
        }
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        assert_eq!(result.len(), 1);
        // (110 - 90) / (110 - 90) * -100 = -100
        assert_eq!(*result.latest().unwrap(), -100.0);
    }

    #[test]
    fn williams_r_constant_prices() {
        // All bars identical: high=low → range=0 → %R = 0
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        assert!(!result.is_empty());
        assert!(result.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn williams_r_single_bar() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000,
        });
        let wr = WilliamsR { period: 14 };
        let result = wr.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn williams_r_default_period() {
        let wr = WilliamsR::default_period();
        assert_eq!(wr.period, 14);
    }
}
