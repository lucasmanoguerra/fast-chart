use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

pub struct Macd {
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
}

impl Default for Macd {
    fn default() -> Self {
        Self {
            fast: 12,
            slow: 26,
            signal: 9,
        }
    }
}

pub struct MacdResult {
    pub macd_line: TimeSeries<f64, MAX_SERIES_LEN>,
    pub signal_line: TimeSeries<f64, MAX_SERIES_LEN>,
    pub histogram: TimeSeries<f64, MAX_SERIES_LEN>,
}

impl Macd {
    pub fn calculate_full(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> MacdResult {
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.slow {
            return MacdResult {
                macd_line: TimeSeries::new(),
                signal_line: TimeSeries::new(),
                histogram: TimeSeries::new(),
            };
        }

        // Compute fast and slow EMAs
        let fast_ema = compute_ema(&bars, self.fast);
        let slow_ema = compute_ema(&bars, self.slow);

        // MACD line = fast_ema - slow_ema (aligned to slow_ema's output)
        let offset = self.slow - self.fast;
        let mut macd_line = TimeSeries::new();
        for i in 0..slow_ema.len() {
            if i + offset < fast_ema.len() {
                macd_line.push(fast_ema[i + offset] - slow_ema[i]);
            }
        }

        // Signal line = EMA of macd_line
        let macd_vals: Vec<f64> = macd_line.iter().copied().collect();
        let signal_line = compute_ema_from_values(&macd_vals, self.signal);

        // Histogram = macd_line - signal_line
        let signal_vals: Vec<f64> = signal_line.iter().copied().collect();
        let sig_offset = self.signal - 1;
        let mut histogram = TimeSeries::new();
        for i in 0..macd_vals.len() {
            if i >= sig_offset && (i - sig_offset) < signal_vals.len() {
                histogram.push(macd_vals[i] - signal_vals[i - sig_offset]);
            }
        }

        MacdResult {
            macd_line,
            signal_line,
            histogram,
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for Macd {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        self.calculate_full(series).histogram
    }

    fn name(&self) -> &str {
        "MACD"
    }
}

fn compute_ema(bars: &[&Bar], period: usize) -> Vec<f64> {
    if bars.len() < period {
        return Vec::new();
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let first_sma: f64 = bars[..period].iter().map(|b| b.close).sum::<f64>() / period as f64;
    let mut result = vec![first_sma];
    let mut prev = first_sma;
    for i in period..bars.len() {
        let ema = alpha * bars[i].close + (1.0 - alpha) * prev;
        result.push(ema);
        prev = ema;
    }
    result
}

fn compute_ema_from_values(values: &[f64], period: usize) -> TimeSeries<f64, MAX_SERIES_LEN> {
    let mut result = TimeSeries::new();
    if values.len() < period {
        return result;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let first_sma: f64 = values[..period].iter().sum::<f64>() / period as f64;
    result.push(first_sma);
    let mut prev = first_sma;
    for i in period..values.len() {
        let ema = alpha * values[i] + (1.0 - alpha) * prev;
        result.push(ema);
        prev = ema;
    }
    result
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
    fn macd_name() {
        let macd = Macd::default();
        assert_eq!(macd.name(), "MACD");
    }

    #[test]
    fn macd_insufficient_data() {
        let bars = make_bars(10);
        let macd = Macd::default();
        let result = macd.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn macd_default_params() {
        let macd = Macd::default();
        assert_eq!(macd.fast, 12);
        assert_eq!(macd.slow, 26);
        assert_eq!(macd.signal, 9);
    }

    #[test]
    fn macd_histogram() {
        let bars = make_bars(100);
        let macd = Macd::default();
        let result = macd.calculate(&bars);
        // Histogram should have values
        assert!(result.len() > 0);
    }

    #[test]
    fn macd_full_result_consistency() {
        let bars = make_bars(100);
        let macd = Macd::default();
        let full = macd.calculate_full(&bars);
        let hist = macd.calculate(&bars);
        assert_eq!(full.histogram.len(), hist.len());
    }
}
