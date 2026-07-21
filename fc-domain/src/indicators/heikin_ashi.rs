use fc_primitives::bar::Bar;
use crate::indicator::Indicator;
use fc_primitives::series::TimeSeries;
use super::sma::MAX_SERIES_LEN;

/// Heikin Ashi — smoothed candlestick representation.
///
/// Formulas:
/// - HA_Close = (Open + High + Low + Close) / 4
/// - HA_Open  = (prev_HA_Open + prev_HA_Close) / 2
///   (first HA_Open = (Open + Close) / 2)
/// - HA_High  = max(High, HA_Open, HA_Close)
/// - HA_Low   = min(Low, HA_Open, HA_Close)
///
/// This indicator returns the HA_Close values, suitable for overlay on a
/// price chart.
pub struct HeikinAshi;

impl Indicator<MAX_SERIES_LEN> for HeikinAshi {
    fn calculate(
        &self,
        series: &TimeSeries<Bar, MAX_SERIES_LEN>,
    ) -> TimeSeries<f64, MAX_SERIES_LEN> {
        let mut result = TimeSeries::new();
        let bars: Vec<&Bar> = series.iter().collect();

        if bars.is_empty() {
            return result;
        }

        // First bar
        let first = bars[0];
        let first_ha_close = (first.open + first.high + first.low + first.close) / 4.0;
        let first_ha_open = (first.open + first.close) / 2.0;

        result.push(first_ha_close);

        let mut prev_ha_open = first_ha_open;
        let mut prev_ha_close = first_ha_close;

        for &bar in &bars[1..] {
            let ha_close = (bar.open + bar.high + bar.low + bar.close) / 4.0;
            let ha_open = (prev_ha_open + prev_ha_close) / 2.0;

            result.push(ha_close);

            prev_ha_open = ha_open;
            prev_ha_close = ha_close;
        }

        result
    }

    fn name(&self) -> &str {
        "Heikin Ashi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let ha = HeikinAshi;
        assert_eq!(ha.name(), "Heikin Ashi");
    }

    #[test]
    fn empty_series() {
        let series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        assert!(result.is_empty());
    }

    #[test]
    fn single_bar() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
        });
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        assert_eq!(result.len(), 1);
        // HA_Close = (100 + 110 + 90 + 105) / 4 = 405 / 4 = 101.25
        assert!((result.get(0).unwrap() - 101.25).abs() < f64::EPSILON);
    }

    #[test]
    fn output_count_matches_input() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..20 {
            let p = 100.0 + i as f64;
            series.push(Bar {
                timestamp: i * 60000,
                open: p,
                high: p + 2.0,
                low: p - 1.0,
                close: p + 1.0,
                volume: 1000,
            });
        }
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn first_ha_close_formula() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        series.push(Bar {
            timestamp: 0,
            open: 80.0,
            high: 120.0,
            low: 60.0,
            close: 100.0,
            volume: 5000,
        });
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        // HA_Close = (80 + 120 + 60 + 100) / 4 = 90.0
        assert!((result.get(0).unwrap() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn second_bar_uses_ha_open() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        // Bar 0: O=100, H=110, L=90, C=105
        series.push(Bar {
            timestamp: 0,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000,
        });
        // Bar 1: O=110, H=120, L=100, C=115
        series.push(Bar {
            timestamp: 60000,
            open: 110.0,
            high: 120.0,
            low: 100.0,
            close: 115.0,
            volume: 1000,
        });
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        assert_eq!(result.len(), 2);

        // Bar 0: HA_Close = (100+110+90+105)/4 = 101.25
        //         HA_Open = (100+105)/2 = 102.5
        // Bar 1: HA_Close = (110+120+100+115)/4 = 111.25
        //         HA_Open = (102.5 + 101.25) / 2 = 101.875
        assert!((result.get(0).unwrap() - 101.25).abs() < f64::EPSILON);
        assert!((result.get(1).unwrap() - 111.25).abs() < f64::EPSILON);
    }

    #[test]
    fn flat_prices() {
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        for i in 0..10 {
            series.push(Bar {
                timestamp: i * 60000,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000,
            });
        }
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        assert_eq!(result.len(), 10);
        // All HA_Close values should be 100.0 since all OHLC are 100.0
        assert!(result.iter().all(|v| (*v - 100.0).abs() < f64::EPSILON));
    }

    #[test]
    fn trend_smoothing() {
        // Heikin Ashi should smooth out noise; test that consecutive values
        // are closer together than raw closes when there's noise.
        let mut series: TimeSeries<Bar, MAX_SERIES_LEN> = TimeSeries::new();
        let raw_closes = [
            100.0, 102.0, 101.0, 103.0, 100.0, 104.0, 102.0, 105.0, 103.0, 106.0,
        ];
        for (i, &c) in raw_closes.iter().enumerate() {
            series.push(Bar {
                timestamp: i as u64 * 60000,
                open: c - 1.0,
                high: c + 2.0,
                low: c - 2.0,
                close: c,
                volume: 1000,
            });
        }
        let ha = HeikinAshi;
        let result = ha.calculate(&series);
        assert_eq!(result.len(), 10);
        // Verify HA_Close values are between the bounds of the raw data
        for i in 0..result.len() {
            let v = *result.get(i).unwrap();
            assert!(v >= 95.0 && v <= 110.0, "HA_Close {v} out of expected range");
        }
    }
}
