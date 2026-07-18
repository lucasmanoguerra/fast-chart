use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

pub struct Stochastic {
    pub k_period: usize,
    pub d_period: usize,
}

impl Default for Stochastic {
    fn default() -> Self {
        Self {
            k_period: 14,
            d_period: 3,
        }
    }
}

pub struct StochasticResult {
    pub k: TimeSeries<f64, MAX_SERIES_LEN>,
    pub d: TimeSeries<f64, MAX_SERIES_LEN>,
}

impl Stochastic {
    pub fn calculate_full(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> StochasticResult {
        let bars: Vec<&Bar> = series.iter().collect();
        let mut k_values = Vec::new();

        if bars.len() < self.k_period {
            return StochasticResult {
                k: TimeSeries::new(),
                d: TimeSeries::new(),
            };
        }

        for i in (self.k_period - 1)..bars.len() {
            let window = &bars[(i + 1 - self.k_period)..=i];
            let highest = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
            let lowest = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
            let range = highest - lowest;
            let k = if range == 0.0 {
                50.0
            } else {
                (bars[i].close - lowest) / range * 100.0
            };
            k_values.push(k);
        }

        let mut k_series = TimeSeries::new();
        for &v in &k_values {
            k_series.push(v);
        }

        let mut d_series = TimeSeries::new();
        if k_values.len() >= self.d_period {
            for i in (self.d_period - 1)..k_values.len() {
                let avg: f64 = k_values[(i + 1 - self.d_period)..=i].iter().sum::<f64>()
                    / self.d_period as f64;
                d_series.push(avg);
            }
        }

        StochasticResult {
            k: k_series,
            d: d_series,
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for Stochastic {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        self.calculate_full(series).d
    }

    fn name(&self) -> &str {
        "STOCHASTIC"
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
                high: p + 2.0,
                low: p - 2.0,
                close: p + 0.5,
                volume: 1000,
            });
        }
        s
    }

    #[test]
    fn stochastic_name() {
        let stoch = Stochastic::default();
        assert_eq!(stoch.name(), "STOCHASTIC");
    }

    #[test]
    fn stochastic_insufficient_data() {
        let bars = make_bars(5);
        let stoch = Stochastic::default();
        let result = stoch.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn stochastic_k_bounds() {
        let bars = make_bars(200);
        let stoch = Stochastic::default();
        let full = stoch.calculate_full(&bars);
        // %K should be in [0, 100]
        assert!(full.k.iter().all(|v| *v >= 0.0 && *v <= 100.0));
    }

    #[test]
    fn stochastic_d_bounds() {
        let bars = make_bars(200);
        let stoch = Stochastic::default();
        let full = stoch.calculate_full(&bars);
        // %D should be in [0, 100]
        assert!(full.d.iter().all(|v| *v >= 0.0 && *v <= 100.0));
    }

    #[test]
    fn stochastic_d_length() {
        let bars = make_bars(200);
        let stoch = Stochastic::default();
        let full = stoch.calculate_full(&bars);
        // %D length = %K length - d_period + 1
        assert_eq!(full.d.len(), full.k.len() - stoch.d_period + 1);
    }

    #[test]
    fn stochastic_flat_market() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..30 {
            bars.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let stoch = Stochastic::default();
        let full = stoch.calculate_full(&bars);
        // Flat market => range == 0 => %K defaults to 50.0
        assert!(full.k.iter().all(|v| (*v - 50.0).abs() < 1e-10));
    }
}
