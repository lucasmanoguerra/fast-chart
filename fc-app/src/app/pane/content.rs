use fc_domain::drawing::DrawingSet;
use fc_domain::marker::MarkerSet;
use fc_domain::price_line::PriceLineSet;
use fc_domain::price_scale::{DefaultPriceFormatter, PriceFormatter};

use super::IndicatorOverlay;
use super::SeriesRef;

/// Data-related contents of a pane.
///
/// Groups series references, indicators, drawings, markers, price lines,
/// and the price formatter. Extracted from [`Pane`](super::Pane) to
/// separate data ownership from layout/geometry concerns (SRP).
pub(crate) struct PaneContent {
    series: Vec<SeriesRef>,
    indicators: Vec<IndicatorOverlay>,
    drawings: DrawingSet,
    markers: MarkerSet,
    price_lines: PriceLineSet,
    formatter: Box<dyn PriceFormatter>,
}

impl PaneContent {
    /// Create an empty content instance with a default price formatter.
    pub(crate) fn new() -> Self {
        Self {
            series: Vec::new(),
            indicators: Vec::new(),
            drawings: DrawingSet::new(),
            markers: MarkerSet::new(),
            price_lines: PriceLineSet::new(),
            formatter: Box::new(DefaultPriceFormatter::new(None)),
        }
    }

    // --- Series ---

    /// Get a slice of all series references.
    pub(crate) fn series(&self) -> &[SeriesRef] {
        &self.series
    }

    /// Add a series reference.
    pub(crate) fn push_series(&mut self, series: SeriesRef) {
        self.series.push(series);
    }

    // --- Indicators ---

    /// Get a slice of all indicator overlays.
    pub(crate) fn indicators(&self) -> &[IndicatorOverlay] {
        &self.indicators
    }

    /// Add an indicator overlay.
    pub(crate) fn push_indicator(&mut self, indicator: IndicatorOverlay) {
        self.indicators.push(indicator);
    }

    // --- Drawings ---

    /// Get a reference to the drawing set.
    pub(crate) fn drawings(&self) -> &DrawingSet {
        &self.drawings
    }

    /// Get a mutable reference to the drawing set.
    pub(crate) fn drawings_mut(&mut self) -> &mut DrawingSet {
        &mut self.drawings
    }

    // --- Markers ---

    /// Get the marker set.
    pub(crate) fn markers(&self) -> &MarkerSet {
        &self.markers
    }

    /// Get a mutable reference to the marker set.
    pub(crate) fn markers_mut(&mut self) -> &mut MarkerSet {
        &mut self.markers
    }

    // --- Price lines ---

    /// Get the price line set.
    pub(crate) fn price_lines(&self) -> &PriceLineSet {
        &self.price_lines
    }

    /// Get a mutable reference to the price line set.
    pub(crate) fn price_lines_mut(&mut self) -> &mut PriceLineSet {
        &mut self.price_lines
    }

    // --- Formatter ---

    /// Get the price formatter.
    pub(crate) fn formatter(&self) -> &dyn PriceFormatter {
        self.formatter.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::pane::{Pane, SeriesRef};
    use fc_domain::price_scale::PriceScaleId;
    use fc_primitives::series_type::SeriesType;

    #[test]
    fn pane_content_default_is_empty() {
        let content = PaneContent::new();
        assert!(content.series().is_empty());
        assert!(content.indicators().is_empty());
        assert!(content.markers().is_empty());
        assert!(content.price_lines().is_empty());
    }

    #[test]
    fn pane_content_push_series() {
        let mut content = PaneContent::new();
        content.push_series(SeriesRef {
            name: "BTC".into(),
            series_type: SeriesType::Candle,
            price_scale_id: PriceScaleId::Left,
        });
        assert_eq!(content.series().len(), 1);
        assert_eq!(content.series()[0].name, "BTC");
    }

    #[test]
    fn pane_content_push_indicator() {
        let mut content = PaneContent::new();
        content.push_indicator(IndicatorOverlay {
            name: "SMA(14)".into(),
            pane_id: 0,
        });
        assert_eq!(content.indicators().len(), 1);
        assert_eq!(content.indicators()[0].name, "SMA(14)");
    }

    #[test]
    fn pane_content_formatter_produces_output() {
        let content = PaneContent::new();
        let formatted = content.formatter().format(105.2);
        assert_eq!(formatted, "105.20");
    }

    #[test]
    fn pane_delegates_to_content() {
        let mut pane = Pane::new(0, 0.7);
        pane.add_series("ETH".into(), SeriesType::Line);
        assert_eq!(pane.series().len(), 1);
        assert_eq!(pane.series()[0].name, "ETH");
    }

    #[test]
    fn pane_markers_delegates_to_content() {
        let mut pane = Pane::new(0, 0.7);
        pane.markers_mut().add(fc_domain::marker::Marker::new(
            "buy", 1000, 105.0,
        ));
        assert_eq!(pane.markers().len(), 1);
    }

    #[test]
    fn pane_price_lines_delegates_to_content() {
        let mut pane = Pane::new(0, 0.7);
        pane.price_lines_mut().add(fc_domain::price_line::PriceLine::new(
            "support", 100.0,
        ));
        assert_eq!(pane.price_lines().len(), 1);
    }
}
