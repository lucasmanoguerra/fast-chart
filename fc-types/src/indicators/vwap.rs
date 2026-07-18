use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Volume Weighted Average Price.
///
/// VWAP = Σ(TypicalPrice × Volume) / Σ(Volume)
///
/// Where TypicalPrice = (High + Low + Close) / 3.
/// The calculation resets when a gap larger than 24 hours is detected
/// between consecutive bars (indicating a new trading session).
pub struct Vwap;

impl Indicator<MAX_SERIES_LEN> for Vwap {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.is_empty() {
            return result;
        }

        let mut cumulative_tp_vol: f64 = 0.0;
        let mut cumulative_vol: f64 = 0.0;
        let mut prev_timestamp: u64 = bars[0].timestamp;

        for bar in &bars {
            // Reset on session boundary (gap > 24 hours = 86_400 seconds)
            if bar.timestamp.saturating_sub(prev_timestamp) > 86_400 {
                cumulative_tp_vol = 0.0;
                cumulative_vol = 0.0;
            }

            let typical_price = (bar.high + bar.low + bar.close) / 3.0;
            cumulative_tp_vol += typical_price * bar.volume as f64;
            cumulative_vol += bar.volume as f64;

            let vwap = if cumulative_vol > 0.0 {
                cumulative_tp_vol / cumulative_vol
            } else {
                typical_price
            };

            result.push(vwap);
            prev_timestamp = bar.timestamp;
        }

        result
    }

    fn name(&self) -> &str {
        "VWAP"
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
    fn vwap_name() {
        let vwap = Vwap;
        assert_eq!(vwap.name(), "VWAP");
    }

    #[test]
    fn vwap_empty_series() {
        let bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn vwap_single_bar() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
        });
        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        assert_eq!(result.len(), 1);
        // Typical price = (110 + 90 + 105) / 3 = 101.666...
        let expected_tp = (110.0 + 90.0 + 105.0) / 3.0;
        assert!((result.get(0).unwrap() - expected_tp).abs() < 1e-10);
    }

    #[test]
    fn vwap_basic() {
        let bars = make_bars(50);
        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        assert_eq!(result.len(), 50);
        assert!(result.iter().all(|v| *v > 0.0));
    }

    #[test]
    fn vwap_exact_period() {
        // VWAP produces one output per input bar
        let bars = make_bars(10);
        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn vwap_zero_volume() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // First bar with volume
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
        });
        // Second bar with zero volume
        bars.push(Bar {
            timestamp: 60000,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 0,
        });
        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        assert_eq!(result.len(), 2);
        // Both should be valid (second falls back to typical price)
        assert!(result.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn vwap_session_reset() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // Day 1: two bars
        bars.push(Bar {
            timestamp: 1000,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
        });
        bars.push(Bar {
            timestamp: 2000,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
        });
        // Day 2: gap > 24h (86400 seconds), new session
        bars.push(Bar {
            timestamp: 100_000,
            open: 200.0,
            high: 210.0,
            low: 190.0,
            close: 205.0,
            volume: 1000,
        });

        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        assert_eq!(result.len(), 3);

        // Bar 1: tp = (110+90+105)/3 = 101.666..., vol=1000
        let tp1 = (110.0 + 90.0 + 105.0) / 3.0;
        assert!((result.get(0).unwrap() - tp1).abs() < 1e-10);

        // Bar 2: cumulative, same typical price => same VWAP
        assert!((result.get(1).unwrap() - tp1).abs() < 1e-10);

        // Bar 3: reset, new VWAP based on only this bar
        let tp3 = (210.0 + 190.0 + 205.0) / 3.0;
        assert!((result.get(2).unwrap() - tp3).abs() < 1e-10);
    }

    #[test]
    fn vwap_cumulative_weighted() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // Bar 1: tp=100, vol=100
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 100,
        });
        // Bar 2: tp=200, vol=100
        bars.push(Bar {
            timestamp: 60000,
            open: 200.0,
            high: 200.0,
            low: 200.0,
            close: 200.0,
            volume: 100,
        });
        let vwap = Vwap;
        let result = vwap.calculate(&bars);
        // Bar 1: 100*100 / 100 = 100
        assert!((result.get(0).unwrap() - 100.0).abs() < 1e-10);
        // Bar 2: (100*100 + 200*100) / (100+100) = 150
        assert!((result.get(1).unwrap() - 150.0).abs() < 1e-10);
    }
}
