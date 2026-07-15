use crate::bar::Bar;
use crate::indicator::Indicator;
use crate::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Parabolic SAR (Stop and Reverse).
///
/// A trend-following indicator that provides potential entry and exit points.
/// The SAR appears as a dot above or below the price, flipping when the
/// trend reverses.
///
/// Formula:
/// SAR = Previous SAR + AF × (EP - Previous SAR)
///
/// - AF (Acceleration Factor): starts at `step` (default 0.02), increases by
///   `step` each time a new EP is established, capped at `max_af` (default 0.20).
/// - EP (Extreme Point): highest high in uptrend, lowest low in downtrend.
/// - Trend reversal occurs when price crosses the SAR level.
///
/// Defaults:
/// - step: 0.02
/// - max_af: 0.20
pub struct ParabolicSar {
    pub step: f64,
    pub max_af: f64,
}

impl ParabolicSar {
    /// Creates a Parabolic SAR with standard defaults (step=0.02, max_af=0.20).
    pub fn default_params() -> Self {
        Self {
            step: 0.02,
            max_af: 0.20,
        }
    }
}

impl Indicator<MAX_SERIES_LEN> for ParabolicSar {
    fn calculate(&self, series: &TimeSeries<Bar, MAX_SERIES_LEN>) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();
        if bars.len() < 2 {
            return result;
        }

        // Initialize: compare first two bars to determine initial trend.
        let mut is_long = bars[1].close > bars[0].close;

        let mut ep: f64;
        let mut sar: f64;
        let mut af = self.step;

        if is_long {
            ep = f64::max(bars[0].high, bars[1].high);
            sar = f64::min(bars[0].low, bars[1].low);
        } else {
            ep = f64::min(bars[0].low, bars[1].low);
            sar = f64::max(bars[0].high, bars[1].high);
        }

        result.push(sar);

        for i in 1..bars.len() {
            let prev_sar = sar;
            let bar = bars[i];

            // Compute raw SAR
            let raw_sar = prev_sar + af * (ep - prev_sar);

            if is_long {
                // Check reversal BEFORE clamping — use raw SAR.
                // Reversal: price (low) drops below SAR
                if bar.low < raw_sar {
                    is_long = false;
                    sar = ep; // SAR flips to previous extreme point (highest high)
                    ep = bar.low;
                    af = self.step;
                } else {
                    // No reversal — clamp SAR to be at or below the lower bound
                    let lower_bound = f64::min(bars[i - 1].low, bar.low);
                    sar = raw_sar.min(lower_bound);
                    if bar.high > ep {
                        ep = bar.high;
                        af = (af + self.step).min(self.max_af);
                    }
                }
            } else {
                // Check reversal BEFORE clamping — use raw SAR.
                // Reversal: price (high) rises above SAR
                if bar.high > raw_sar {
                    is_long = true;
                    sar = ep; // SAR flips to previous extreme point (lowest low)
                    ep = bar.high;
                    af = self.step;
                } else {
                    // No reversal — clamp SAR to be at or above the upper bound
                    let upper_bound = f64::max(bars[i - 1].high, bar.high);
                    sar = raw_sar.max(upper_bound);
                    if bar.low < ep {
                        ep = bar.low;
                        af = (af + self.step).min(self.max_af);
                    }
                }
            }

            result.push(sar);
        }

        result
    }

    fn name(&self) -> &str {
        "Parabolic SAR"
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
    fn parabolic_sar_name() {
        let ps = ParabolicSar::default_params();
        assert_eq!(ps.name(), "Parabolic SAR");
    }

    #[test]
    fn parabolic_sar_insufficient_data() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 100.0,
            volume: 1000,
        });
        let ps = ParabolicSar::default_params();
        let result = ps.calculate(&bars);
        assert!(result.is_empty());
    }

    #[test]
    fn parabolic_sar_two_bars() {
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        bars.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 100.0,
            volume: 1000,
        });
        bars.push(Bar {
            timestamp: 60000,
            open: 102.0,
            high: 107.0,
            low: 101.0,
            close: 105.0,
            volume: 1000,
        });
        let ps = ParabolicSar::default_params();
        let result = ps.calculate(&bars);
        // 2 bars → 2 output values (SAR for each bar)
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parabolic_sar_basic() {
        let bars = make_bars(50);
        let ps = ParabolicSar::default_params();
        let result = ps.calculate(&bars);
        assert_eq!(result.len(), 50);
    }

    #[test]
    fn parabolic_sar_uptrend() {
        // Strong uptrend: each bar higher than the previous
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..30 {
            let base = 100.0 + i as f64 * 2.0;
            bars.push(Bar {
                timestamp: i * 60000,
                open: base,
                high: base + 3.0,
                low: base + 1.0,
                close: base + 2.0,
                volume: 1000,
            });
        }
        let ps = ParabolicSar::default_params();
        let result = ps.calculate(&bars);
        assert_eq!(result.len(), 30);
        // In a pure uptrend, SAR should be below price (below close)
        // After the first couple of bars (initialization), SAR < close
        for i in 2..result.len() {
            let sar = *result.get(i).unwrap();
            let close = bars.get(i).unwrap().close;
            assert!(
                sar < close,
                "SAR ({}) should be below close ({}) in uptrend at index {}",
                sar,
                close,
                i
            );
        }
    }

    #[test]
    fn parabolic_sar_downtrend() {
        // Strong downtrend: each bar lower than the previous
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..30 {
            let base = 150.0 - i as f64 * 2.0;
            bars.push(Bar {
                timestamp: i * 60000,
                open: base,
                high: base + 1.0,
                low: base - 3.0,
                close: base - 2.0,
                volume: 1000,
            });
        }
        let ps = ParabolicSar::default_params();
        let result = ps.calculate(&bars);
        assert_eq!(result.len(), 30);
        // In a pure downtrend, SAR should be above price (above close)
        for i in 2..result.len() {
            let sar = *result.get(i).unwrap();
            let close = bars.get(i).unwrap().close;
            assert!(
                sar > close,
                "SAR ({}) should be above close ({}) in downtrend at index {}",
                sar,
                close,
                i
            );
        }
    }

    #[test]
    fn parabolic_sar_reversal() {
        // Gradual uptrend, then gradual downtrend
        let mut bars: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // 20 bars of steady uptrend (close increases by 2 each bar)
        for i in 0..20 {
            let base = 100.0 + i as f64 * 2.0;
            bars.push(Bar {
                timestamp: i * 60000,
                open: base,
                high: base + 0.5,
                low: base - 0.5,
                close: base + 0.3,
                volume: 1000,
            });
        }
        // 20 bars of steady downtrend (close decreases by 2 each bar)
        for i in 0..20 {
            let base = 139.0 - i as f64 * 2.0;
            bars.push(Bar {
                timestamp: (20 + i) * 60000,
                open: base,
                high: base + 0.5,
                low: base - 0.5,
                close: base - 0.3,
                volume: 1000,
            });
        }
        let ps = ParabolicSar::default_params();
        let result = ps.calculate(&bars);
        assert_eq!(result.len(), 40);
        // In early uptrend, SAR should be below close
        let early_sar = *result.get(5).unwrap();
        let early_close = bars.get(5).unwrap().close;
        assert!(
            early_sar < early_close,
            "In uptrend, SAR ({}) should be below close ({})",
            early_sar,
            early_close
        );
        // After sufficient downtrend bars, SAR should eventually be above close
        let mut found_above = false;
        for i in 25..40 {
            let sar = *result.get(i).unwrap();
            let close = bars.get(i).unwrap().close;
            if sar > close {
                found_above = true;
                break;
            }
        }
        assert!(
            found_above,
            "Expected SAR to be above close in the downtrend phase"
        );
    }

    #[test]
    fn parabolic_sar_default_params() {
        let ps = ParabolicSar::default_params();
        assert_eq!(ps.step, 0.02);
        assert_eq!(ps.max_af, 0.20);
    }

    #[test]
    fn parabolic_sar_custom_params() {
        let ps = ParabolicSar {
            step: 0.01,
            max_af: 0.10,
        };
        let bars = make_bars(50);
        let result = ps.calculate(&bars);
        assert_eq!(result.len(), 50);
    }
}
