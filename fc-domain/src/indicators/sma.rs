use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;

pub const MAX_SERIES_LEN: usize = 100_000;

pub struct Sma {
    pub period: usize,
}

impl Indicator<MAX_SERIES_LEN> for Sma {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period {
            return result;
        }

        let mut sum: f64 = bars[..self.period].iter().map(|b| b.close).sum();
        result.push(sum / self.period as f64);

        for i in self.period..bars.len() {
            sum += bars[i].close - bars[i - self.period].close;
            result.push(sum / self.period as f64);
        }
        result
    }

    fn name(&self) -> &str {
        "SMA"
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

    // Clasificación: determinística — verifica sma_name
    #[test]
    fn sma_name() {
        let sma = Sma { period: 10 };
        assert_eq!(sma.name(), "SMA");
    }

    // Clasificación: determinística — verifica sma_insufficient_data
    #[test]
    fn sma_insufficient_data() {
        let bars = make_bars(5);
        let sma = Sma { period: 10 };
        let result = sma.calculate(&bars);
        assert!(result.is_empty());
    }

    // Clasificación: determinística — verifica sma_basic
    #[test]
    fn sma_basic() {
        let bars = make_bars(50);
        let sma = Sma { period: 10 };
        let result = sma.calculate(&bars);
        assert_eq!(result.len(), 41);
        assert!(result.iter().all(|v| *v > 0.0));
    }

    // Clasificación: determinística — verifica sma_exact_period
    #[test]
    fn sma_exact_period() {
        let bars = make_bars(10);
        let sma = Sma { period: 10 };
        let result = sma.calculate(&bars);
        assert_eq!(result.len(), 1);
    }

    // Clasificación: determinística — verifica sma_single_period_rolling
    #[test]
    fn sma_single_period_rolling() {
        let bars = make_bars(5);
        let sma = Sma { period: 1 };
        let result = sma.calculate(&bars);
        assert_eq!(result.len(), 5);
    }
}
