use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Commodity Channel Index (CCI).
///
/// CCI measures the difference between the current typical price and its
/// historical average, normalized by the mean deviation.
///
/// Formula:
/// - TP (Typical Price) = (High + Low + Close) / 3
/// - SMA(TP, period) = Simple Moving Average of TP over `period`
/// - Mean Deviation = Σ|TP - SMA| / period
/// - CCI = (TP - SMA(TP)) / (0.015 × Mean Deviation)
///
/// Typical values range from -200 to +200.
/// - Above +100 → overbought (potential sell signal)
/// - Below -100 → oversold (potential buy signal)
///
/// Default period: 20
pub struct Cci {
    pub period: usize,
}

impl Cci {
    /// Creates a CCI indicator with the default period of 20.
    pub fn default_period() -> Self {
        Self { period: 20 }
    }
}

impl Indicator<MAX_SERIES_LEN> for Cci {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period {
            return result;
        }

        let mut tp_values: Vec<f64> = Vec::with_capacity(bars.len());
        for &bar in &bars {
            tp_values.push((bar.high + bar.low + bar.close) / 3.0);
        }

        let mut sum_tp: f64 = tp_values[..self.period].iter().sum();
        let mut sma = sum_tp / self.period as f64;

        let mut mean_dev = compute_mean_deviation(&tp_values, 0, self.period, sma);
        let denom = 0.015 * mean_dev;
        let first_cci = if denom.abs() < f64::EPSILON {
            0.0
        } else {
            (tp_values[self.period - 1] - sma) / denom
        };
        result.push(first_cci);

        for i in self.period..bars.len() {
            sum_tp += tp_values[i] - tp_values[i - self.period];
            sma = sum_tp / self.period as f64;
            mean_dev = compute_mean_deviation(&tp_values, i + 1 - self.period, self.period, sma);
            let denom = 0.015 * mean_dev;
            let cci = if denom.abs() < f64::EPSILON {
                0.0
            } else {
                (tp_values[i] - sma) / denom
            };
            result.push(cci);
        }

        result
    }

    fn name(&self) -> &str {
        "CCI"
    }
}

fn compute_mean_deviation(tp_values: &[f64], start: usize, period: usize, sma: f64) -> f64 {
    let sum: f64 = tp_values[start..start + period]
        .iter()
        .map(|&tp| (tp - sma).abs())
        .sum();
    sum / period as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use fc_primitives::bar::Bar;

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
    fn cci_name() {
        let cci = Cci { period: 20 };
        assert_eq!(cci.name(), "CCI");
    }

    #[test]
    fn cci_insufficient_data() {
        let bars = make_bars(5);
        let cci = Cci { period: 20 };
        let result = cci.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn cci_exact_period() {
        let bars = make_bars(20);
        let cci = Cci { period: 20 };
        let result = cci.calculate(&bars);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn cci_basic() {
        let bars = make_bars(50);
        let cci = Cci { period: 20 };
        let result = cci.calculate(&bars);
        assert_eq!(result.len(), 31);
    }

    #[test]
    fn cci_constant_prices() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..25 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let cci = Cci { period: 20 };
        let result = cci.calculate(&bars);
        assert!(!result.is_empty());
        assert!(result.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn cci_single_bar() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000,
        });
        let cci = Cci { period: 20 };
        let result = cci.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn cci_default_period() {
        let cci = Cci::default_period();
        assert_eq!(cci.period, 20);
    }
}
