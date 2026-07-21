//! Pure domain primitives for the fast-chart trading library.
//!
//! Zero external dependencies (optional serde). Contains fundamental
//! building blocks: OHLCV bars, ticks, time series, scales, colors,
//! viewport, invalidation, localization, and kinetic scrolling.

pub mod bar;
pub mod color;
pub mod error;
pub mod invalidation;
pub mod kinetic;
pub mod line_style;
pub mod localization;
pub mod rect;
pub mod scale;
pub mod series;
pub mod series_type;
pub mod tick;

// Re-exports
pub use bar::Bar;
pub use color::Rgba;
pub use error::ChartError;
pub use invalidation::{InvalidationLevel, InvalidationMask, PaneBitmask};
pub use kinetic::KineticScroll;
pub use line_style::LineStyle;
pub use localization::{EnglishLocalizer, Localizer, SpanishLocalizer};
pub use rect::Rect;
pub use scale::{LinearScale, TimeScale};
pub use series::TimeSeries;
pub use series_type::SeriesType;
pub use tick::Tick;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_roundtrip() {
        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        assert_eq!(bar.timestamp, 1000);
        assert_eq!(bar.close, 102.0);
    }

    #[test]
    fn series_push_and_latest() {
        let mut series: TimeSeries<Bar, 100> = TimeSeries::new();
        let bar = Bar::new(1, 10.0, 12.0, 9.0, 11.0, 100).unwrap();
        series.push(bar);
        let latest = series.latest().unwrap();
        assert_eq!(latest.open, 10.0);
    }
}
