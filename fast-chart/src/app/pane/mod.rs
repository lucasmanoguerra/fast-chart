pub mod divider;
pub mod events;

use fast_chart_domain::drawing::DrawingSet;
use fast_chart_domain::marker::MarkerSet;
use fast_chart_domain::price_line::PriceLineSet;
use fast_chart_domain::price_scale::{DefaultPriceFormatter, PriceFormatter, PriceScale, PriceScaleId};
use fast_chart_domain::series_type::SeriesType;
use fast_chart_domain::viewport::Viewport;

use crate::render::series_renderer::SeriesRenderer;

/// A single pane in the vertical chart stack.
///
/// Each pane has its own viewport (shared x-axis, independent y-axis), a list of
/// series references, indicator overlays, layer renderers, and drawing tools.
/// Panes are stacked vertically and separated by draggable dividers.
pub struct Pane {
    /// Unique identifier (index in the layout's pane list).
    pub id: usize,
    /// Viewport for this pane. Time range is shared across all panes;
    /// value range is pane-local.
    pub viewport: Viewport,
    /// Proportional height as a fraction of the total canvas (0.0 – 1.0).
    pub height: f64,
    /// Series that belong to this pane.
    series: Vec<SeriesRef>,
    /// Indicators rendered as overlays in this pane.
    indicators: Vec<IndicatorOverlay>,
    /// Whether the pane is currently visible.
    visible: bool,
    /// Price scales owned by this pane (Left, Right, and optional overlays).
    price_scales: Vec<PriceScale>,
    /// The default (primary) price scale used by new series.
    primary_scale_id: PriceScaleId,
    /// Series renderers (layers) for this pane, ordered by z-index.
    layers: Vec<Box<dyn SeriesRenderer>>,
    /// Drawing tools attached to this pane.
    drawings: DrawingSet,
    /// Markers (annotations) attached to this pane.
    markers: MarkerSet,
    /// Horizontal price lines drawn across this pane.
    price_lines: PriceLineSet,
    /// Formatter for price values displayed on this pane.
    formatter: Box<dyn PriceFormatter>,
}

/// Reference to a named series with its rendering type.
#[derive(Debug, Clone)]
pub struct SeriesRef {
    pub name: String,
    pub series_type: SeriesType,
    /// Which price scale this series maps to.
    pub price_scale_id: PriceScaleId,
}

impl Default for SeriesRef {
    fn default() -> Self {
        Self {
            name: String::new(),
            series_type: SeriesType::default(),
            price_scale_id: PriceScaleId::Left,
        }
    }
}

/// An indicator overlay attached to a pane.
#[derive(Debug, Clone)]
pub struct IndicatorOverlay {
    pub name: String,
    pub pane_id: usize,
}

impl Pane {
    /// Create a new pane with the given id and proportional height.
    pub fn new(id: usize, height: f64) -> Self {
        let mut price_scales = Vec::new();
        price_scales.push(PriceScale::new(
            PriceScaleId::Left,
            Default::default(),
        ));
        price_scales.push(PriceScale::new(
            PriceScaleId::Right,
            Default::default(),
        ));

        Self {
            id,
            viewport: Viewport::default(),
            height,
            series: Vec::new(),
            indicators: Vec::new(),
            visible: true,
            price_scales,
            primary_scale_id: PriceScaleId::Left,
            layers: Vec::new(),
            drawings: DrawingSet::new(),
            markers: MarkerSet::new(),
            price_lines: PriceLineSet::new(),
            formatter: Box::new(DefaultPriceFormatter::new(None)),
        }
    }

    /// Add a series reference to this pane.
    pub fn add_series(&mut self, name: String, series_type: SeriesType) {
        self.series.push(SeriesRef {
            name,
            series_type,
            price_scale_id: PriceScaleId::Left,
        });
    }

    /// Add an indicator overlay to this pane.
    pub fn add_indicator(&mut self, name: String) {
        self.indicators.push(IndicatorOverlay {
            name,
            pane_id: self.id,
        });
    }

    // --- Field accessors ---

    /// Get a slice of all series references in this pane.
    pub fn series(&self) -> &[SeriesRef] {
        &self.series
    }

    /// Get a slice of all indicator overlays in this pane.
    pub fn indicators(&self) -> &[IndicatorOverlay] {
        &self.indicators
    }

    /// Whether the pane is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set the pane's visibility.
    pub fn set_visible(&mut self, v: bool) {
        self.visible = v;
    }

    // --- Layers (SeriesRenderers) ---

    /// Add a series renderer (layer) to this pane.
    pub fn add_layer(&mut self, renderer: Box<dyn SeriesRenderer>) {
        self.layers.push(renderer);
    }

    /// Get a slice of all layer renderers in this pane.
    pub fn layers(&self) -> &[Box<dyn SeriesRenderer>] {
        &self.layers
    }

    /// Get a mutable slice of all layer renderers in this pane.
    pub fn layers_mut(&mut self) -> &mut [Box<dyn SeriesRenderer>] {
        &mut self.layers
    }

    /// Remove all layers from this pane.
    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    // --- Drawings ---

    /// Get a reference to the drawing set for this pane.
    pub fn drawings(&self) -> &DrawingSet {
        &self.drawings
    }

    /// Get a mutable reference to the drawing set for this pane.
    pub fn drawings_mut(&mut self) -> &mut DrawingSet {
        &mut self.drawings
    }

    // --- Price scale accessors ---

    /// Get a slice of all price scales owned by this pane.
    pub fn price_scales(&self) -> &[PriceScale] {
        &self.price_scales
    }

    /// Get the primary (default) price scale ID used by new series.
    pub fn primary_scale_id(&self) -> PriceScaleId {
        self.primary_scale_id.clone()
    }

    // --- Price scale accessors ---

    /// Add a price scale to this pane.
    pub fn add_price_scale(&mut self, scale: PriceScale) {
        self.price_scales.push(scale);
    }

    /// Remove all price scales from this pane.
    pub fn clear_price_scales(&mut self) {
        self.price_scales.clear();
    }

    /// Guarantee at least Left + Right price scales exist.
    pub fn ensure_price_scales(&mut self) {
        if self.price_scales.is_empty() {
            self.price_scales.push(PriceScale::new(
                PriceScaleId::Left,
                Default::default(),
            ));
            self.price_scales.push(PriceScale::new(
                PriceScaleId::Right,
                Default::default(),
            ));
        }
    }

    /// Get a price scale by ID.
    pub fn price_scale(&self, id: &PriceScaleId) -> Option<&PriceScale> {
        self.price_scales.iter().find(|s| &s.id == id)
    }

    /// Get a mutable price scale by ID.
    pub fn price_scale_mut(&mut self, id: &PriceScaleId) -> Option<&mut PriceScale> {
        self.price_scales.iter_mut().find(|s| &s.id == id)
    }

    /// Get the primary (default) price scale.
    pub fn primary_scale(&self) -> &PriceScale {
        self.price_scale(&self.primary_scale_id)
            .expect("primary scale always exists after ensure_price_scales()")
    }

    /// Compute the pixel y-offset (from the top of the canvas) for this pane,
    /// given the list of all pane heights (fractions) and the total canvas height.
    pub fn pixel_y_offset(&self, pane_heights: &[f64], canvas_height: f64) -> f64 {
        let mut offset = 0.0;
        for (i, &h) in pane_heights.iter().enumerate() {
            if i >= self.id {
                break;
            }
            offset += h * canvas_height;
        }
        offset
    }

    /// Compute the pixel height for this pane.
    pub fn pixel_height(&self, canvas_height: f64) -> f64 {
        self.height * canvas_height
    }

    // --- Marker accessors ---

    /// Get the marker set for this pane.
    pub fn markers(&self) -> &MarkerSet {
        &self.markers
    }

    /// Get a mutable reference to the marker set for this pane.
    pub fn markers_mut(&mut self) -> &mut MarkerSet {
        &mut self.markers
    }

    // --- Price line accessors ---

    /// Get the price line set for this pane.
    pub fn price_lines(&self) -> &PriceLineSet {
        &self.price_lines
    }

    /// Get a mutable reference to the price line set for this pane.
    pub fn price_lines_mut(&mut self) -> &mut PriceLineSet {
        &mut self.price_lines
    }

    // --- Formatter accessors ---

    /// Get the price formatter for this pane.
    pub fn formatter(&self) -> &dyn PriceFormatter {
        self.formatter.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_creation() {
        let pane = Pane::new(0, 0.7);
        assert_eq!(pane.id, 0);
        assert_eq!(pane.height, 0.7);
        assert!(pane.series().is_empty());
        assert!(pane.indicators().is_empty());
        assert!(pane.is_visible());
        assert!(pane.markers().is_empty());
        assert!(pane.price_lines().is_empty());
    }

    #[test]
    fn pane_default_viewport() {
        let pane = Pane::new(0, 0.7);
        let vp = &pane.viewport;
        assert_eq!(vp.time_start, 0);
        assert_eq!(vp.zoom_level, 1.0);
    }

    #[test]
    fn add_series() {
        let mut pane = Pane::new(0, 0.7);
        pane.add_series("BTC".into(), SeriesType::Candle);
        assert_eq!(pane.series().len(), 1);
        assert_eq!(pane.series()[0].name, "BTC");
        assert_eq!(pane.series()[0].series_type, SeriesType::Candle);
    }

    #[test]
    fn add_multiple_series() {
        let mut pane = Pane::new(0, 0.7);
        pane.add_series("BTC".into(), SeriesType::Candle);
        pane.add_series("ETH".into(), SeriesType::Line);
        assert_eq!(pane.series().len(), 2);
    }

    #[test]
    fn add_indicator() {
        let mut pane = Pane::new(0, 0.7);
        pane.add_indicator("SMA(14)".into());
        assert_eq!(pane.indicators().len(), 1);
        assert_eq!(pane.indicators()[0].name, "SMA(14)");
        assert_eq!(pane.indicators()[0].pane_id, 0);
    }

    #[test]
    fn pixel_height_calculation() {
        let pane = Pane::new(0, 0.7);
        let h = pane.pixel_height(700.0);
        assert!((h - 490.0).abs() < 0.001);
    }

    #[test]
    fn pixel_y_offset_first_pane() {
        let pane = Pane::new(0, 0.7);
        let heights = vec![0.7, 0.3];
        assert!((pane.pixel_y_offset(&heights, 700.0)).abs() < 0.001);
    }

    #[test]
    fn pixel_y_offset_second_pane() {
        let pane = Pane::new(1, 0.3);
        let heights = vec![0.7, 0.3];
        assert!((pane.pixel_y_offset(&heights, 700.0) - 490.0).abs() < 0.001);
    }

    // --- Price scale tests ---

    #[test]
    fn pane_has_left_and_right_scales() {
        let pane = Pane::new(0, 0.7);
        assert_eq!(pane.price_scales().len(), 2);
        assert!(pane.price_scale(&PriceScaleId::Left).is_some());
        assert!(pane.price_scale(&PriceScaleId::Right).is_some());
    }

    #[test]
    fn pane_primary_scale_is_left() {
        let pane = Pane::new(0, 0.7);
        assert_eq!(pane.primary_scale().id, PriceScaleId::Left);
    }

    #[test]
    fn pane_add_overlay_scale() {
        let mut pane = Pane::new(0, 0.7);
        let scale = PriceScale::new(
            PriceScaleId::Overlay("RSI".into()),
            Default::default(),
        );
        pane.add_price_scale(scale);
        assert_eq!(pane.price_scales().len(), 3);
        assert!(pane.price_scale(&PriceScaleId::Overlay("RSI".into())).is_some());
    }

    #[test]
    fn pane_price_scale_mut() {
        let mut pane = Pane::new(0, 0.7);
        {
            let right = pane.price_scale_mut(&PriceScaleId::Right).unwrap();
            right.value_max = 200.0;
        }
        assert!((pane.price_scale(&PriceScaleId::Right).unwrap().value_max - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ensure_price_scales_populates_empty() {
        let mut pane = Pane::new(0, 0.7);
        pane.clear_price_scales();
        pane.ensure_price_scales();
        assert_eq!(pane.price_scales().len(), 2);
    }

    #[test]
    fn series_defaults_to_left_scale() {
        let mut pane = Pane::new(0, 0.7);
        pane.add_series("BTC".into(), SeriesType::Candle);
        assert_eq!(pane.series()[0].price_scale_id, PriceScaleId::Left);
    }

    #[test]
    fn pane_markers_accessor() {
        let pane = Pane::new(0, 0.7);
        assert!(pane.markers().is_empty());
    }

    #[test]
    fn pane_markers_mut_accessor() {
        let mut pane = Pane::new(0, 0.7);
        pane.markers_mut().add(fast_chart_domain::marker::Marker::new(
            "buy", 1000, 105.0,
        ));
        assert_eq!(pane.markers().len(), 1);
    }

    #[test]
    fn pane_price_lines_accessor() {
        let pane = Pane::new(0, 0.7);
        assert!(pane.price_lines().is_empty());
    }

    #[test]
    fn pane_price_lines_mut_accessor() {
        let mut pane = Pane::new(0, 0.7);
        pane.price_lines_mut().add(fast_chart_domain::price_line::PriceLine::new(
            "support", 100.0,
        ));
        assert_eq!(pane.price_lines().len(), 1);
    }

    #[test]
    fn pane_formatter_accessor() {
        let pane = Pane::new(0, 0.7);
        let formatted = pane.formatter().format(105.2);
        assert_eq!(formatted, "105.20");
    }

    // --- New tests for layers and drawings ---

    #[test]
    fn pane_layers_empty_by_default() {
        let pane = Pane::new(0, 0.7);
        assert!(pane.layers().is_empty());
    }

    #[test]
    fn pane_drawings_empty_by_default() {
        let pane = Pane::new(0, 0.7);
        assert!(pane.drawings().all_trend_lines().is_empty());
        assert!(pane.drawings().all_horizontal_lines().is_empty());
    }

    #[test]
    fn pane_clear_layers() {
        let mut pane = Pane::new(0, 0.7);
        pane.clear_layers();
        assert!(pane.layers().is_empty());
    }
}
