use fc_primitives::bar::Bar;
use fc_primitives::invalidation::{InvalidationLevel, InvalidationMask};
use fc_primitives::series::TimeSeries;
use fc_primitives::series_type::SeriesType;
use fc_domain::crosshair::Crosshair;
use fc_domain::viewport::Viewport;

const CHART_CAPACITY: usize = 100_000;

/// Snapshot of all mutable chart state.
///
/// Owned by [`ChartController`](super::chart_controller::ChartController),
/// passed by reference to the renderer on every draw call. Contains the
/// time series, viewport, crosshair, pane heights, series type, and
/// invalidation mask.
pub struct ChartState {
    pub time_series: TimeSeries<Bar, CHART_CAPACITY>,
    pub viewport: Viewport,
    pub crosshair: Crosshair,
    pub pane_heights: Vec<f64>,
    pub series_type: SeriesType,
    pub invalidation: InvalidationMask,
}

impl ChartState {
    pub(crate) fn new() -> Self {
        Self {
            time_series: TimeSeries::new(),
            viewport: Viewport::default(),
            crosshair: Crosshair::default(),
            pane_heights: vec![400.0],
            series_type: SeriesType::default(),
            invalidation: InvalidationMask::NONE,
        }
    }

    pub fn mark_dirty(&mut self, level: InvalidationLevel) {
        self.invalidation.merge(InvalidationMask::all_panes(level));
    }

    pub fn mark_pane_dirty(&mut self, level: InvalidationLevel, pane_index: usize) {
        self.invalidation
            .merge(InvalidationMask::single_pane(level, pane_index));
    }

    pub fn consume_invalidation(&mut self) -> InvalidationMask {
        let mask = self.invalidation;
        self.invalidation.clear();
        mask
    }
}
