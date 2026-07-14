use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

pub struct Rsi {
    pub period: usize,
}

impl Indicator<MAX_SERIES_LEN> for Rsi {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < self.period + 1 {
            return result;
        }

        let mut avg_gain = 0.0;
        let mut avg_loss = 0.0;
        for i in 1..=self.period {
            let change = bars[i].close - bars[i - 1].close;
            if change > 0.0 {
                avg_gain += change;
            } else {
                avg_loss -= change;
            }
        }
        avg_gain /= self.period as f64;
        avg_loss /= self.period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        };
        result.push(rsi);

        for i in (self.period + 1)..bars.len() {
            let change = bars[i].close - bars[i - 1].close;
            let (gain, loss) = if change > 0.0 {
                (change, 0.0)
            } else {
                (0.0, -change)
            };
            avg_gain = (avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
            avg_loss = (avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;
            let rsi = if avg_loss == 0.0 {
                100.0
            } else {
                100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
            };
            result.push(rsi);
        }
        result
    }

    fn name(&self) -> &str {
        "RSI"
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
    fn rsi_name() {
        let rsi = Rsi { period: 14 };
        assert_eq!(rsi.name(), "RSI");
    }

    #[test]
    fn rsi_insufficient_data() {
        let bars = make_bars(10);
        let rsi = Rsi { period: 14 };
        let result = rsi.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn rsi_bounds() {
        let bars = make_bars(200);
        let rsi = Rsi { period: 14 };
        let result = rsi.calculate(&bars);
        assert!(result.iter().all(|v| *v >= 0.0 && *v <= 100.0));
    }

    #[test]
    fn rsi_all_gains() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            let p = 100.0 + i as f64;
            bars.push(Bar {
                timestamp: i * 60000,
                open: p,
                high: p + 1.0,
                low: p - 0.5,
                close: p + 1.0,
                volume: 1000,
            });
        }
        let rsi = Rsi { period: 14 };
        let result = rsi.calculate(&bars);
        // All gains, no losses => RSI should be 100.0
        assert_eq!(result.len(), 6);
        assert_eq!(*result.latest().unwrap(), 100.0);
    }

    #[test]
    fn rsi_all_losses() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            let p = 200.0 - i as f64;
            bars.push(Bar {
                timestamp: i * 60000,
                open: p,
                high: p + 0.5,
                low: p - 1.0,
                close: p - 1.0,
                volume: 1000,
            });
        }
        let rsi = Rsi { period: 14 };
        let result = rsi.calculate(&bars);
        // All losses, no gains => RSI should be 0.0
        assert_eq!(*result.latest().unwrap(), 0.0);
    }
}
