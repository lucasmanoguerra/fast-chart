pub mod bar;
pub mod crosshair;
pub mod error;
pub mod indicator;
pub mod scale;
pub mod series;
pub mod series_type;
pub mod tick;
pub mod viewport;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bar::Bar;
    use series::TimeSeries;

    #[test]
    fn bar_roundtrip_in_series() {
        let mut series: TimeSeries<Bar, 100> = TimeSeries::new();
        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        series.push(bar);
        let retrieved = series.latest().unwrap();
        assert_eq!(retrieved.timestamp, 1000);
        assert_eq!(retrieved.open, 100.0);
    }

    #[test]
    fn scale_maps_bar_midpoint() {
        let bar = Bar::new(1, 100.0, 110.0, 100.0, 105.0, 100).unwrap();
        let scale = scale::LinearScale {
            min: 90.0,
            max: 120.0,
            height: 300.0,
        };
        let y = scale.map_to_y(bar.midpoint());
        // midpoint = 105.0, ratio = (105-90)/(120-90) = 0.5, y = 300*(1-0.5) = 150
        assert_eq!(y, 150.0);
    }

    #[test]
    fn crosshair_on_scaled_bar() {
        let ts = scale::TimeScale {
            start: 0,
            end: 2000,
            width: 800.0,
        };
        let vs = scale::LinearScale {
            min: 90.0,
            max: 120.0,
            height: 300.0,
        };
        let mut ch = crosshair::Crosshair::default();
        ch.update(400.0, 150.0, &ts, &vs);
        assert_eq!(ch.time, 1000);
        assert!((ch.price - 105.0).abs() < f64::EPSILON);
    }

    #[test]
    fn viewport_zoom_preserves_center() {
        let mut vp = viewport::Viewport {
            time_start: 0,
            time_end: 1000,
            value_min: 0.0,
            value_max: 100.0,
            zoom_level: 1.0,
        };
        vp.zoom(2.0, 500.0);
        let mid = (vp.time_start + vp.time_end) / 2;
        assert_eq!(mid, 500);
    }

    #[test]
    fn error_display_includes_context() {
        let err = error::ChartError::InvalidPriceData("high < low".into());
        let msg = err.to_string();
        assert!(msg.contains("high < low"));
    }

    #[test]
    fn series_type_default() {
        assert_eq!(series_type::SeriesType::default(), series_type::SeriesType::Candle);
    }
}
