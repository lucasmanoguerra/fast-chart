use crate::ports::data_provider::{DataEvent, DataProvider};
use crate::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
use fast_chart_domain::bar::Bar;
use fast_chart_domain::crosshair::Crosshair;
use fast_chart_domain::invalidation::{InvalidationLevel, InvalidationMask};
use fast_chart_domain::kinetic::KineticScroll;
use fast_chart_domain::series::TimeSeries;
use fast_chart_domain::series_type::SeriesType;
use fast_chart_domain::viewport::Viewport;

const CHART_CAPACITY: usize = 100_000;

/// Snapshot of all mutable chart state.
///
/// Owned by [`ChartController`], passed by reference to the renderer on
/// every draw call. Contains the time series, viewport, crosshair,
/// pane heights, series type, and invalidation mask.
pub struct ChartState {
    pub time_series: TimeSeries<Bar, CHART_CAPACITY>,
    pub viewport: Viewport,
    pub crosshair: Crosshair,
    pub pane_heights: Vec<f64>,
    pub series_type: SeriesType,
    pub invalidation: InvalidationMask,
}

impl ChartState {
    fn new() -> Self {
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

/// Central orchestrator that owns the data pipeline and interaction handler.
///
/// `ChartController` sits at the application layer: it polls a [`DataProvider`]
/// for new market events and applies user interactions via an
/// [`InteractionHandler`]. Rendering is handled by the app layer
/// (e.g. `GpuRenderer`) which reads [`ChartState`] directly.
pub struct ChartController {
    data_provider: Box<dyn DataProvider>,
    interaction: Box<dyn InteractionHandler>,
    state: ChartState,
    indicator_overlays: Vec<(String, TimeSeries<f64, CHART_CAPACITY>)>,
    kinetic: KineticScroll,
}

impl ChartController {
    pub fn new(
        data_provider: Box<dyn DataProvider>,
        interaction: Box<dyn InteractionHandler>,
    ) -> Self {
        Self {
            data_provider,
            interaction,
            state: ChartState::new(),
            indicator_overlays: Vec::new(),
            kinetic: KineticScroll::new(0.95),
        }
    }

    /// Poll the data provider for new events, update the time series, and
    /// re-render when new data arrives.
    pub fn tick(&mut self) {
        // 1. Poll data_provider receiver for new events
        if let Some(rx) = self.data_provider.receiver() {
            while let Ok(event) = rx.try_recv() {
                match event {
                    DataEvent::BarClosed(bar) => {
                        self.state.time_series.push(bar);
                        self.state.mark_dirty(InvalidationLevel::Full);
                    }
                    DataEvent::TickUpdate(_tick) => {
                        // Tick updates update the latest bar in-place
                        // For now, mark dirty — actual tick→bar aggregation
                        // lives in the adapter layer.
                        self.state.mark_dirty(InvalidationLevel::Full);
                    }
                    DataEvent::SymbolChanged(_) | DataEvent::TimeframeChanged(_) => {
                        // Symbol/timeframe changes require a full data reload
                        // at the adapter level. Core just marks dirty.
                        self.state.mark_dirty(InvalidationLevel::Full);
                    }
                }
            }
        }

        // 2. Update viewport if dirty (auto-fit on first data)
        if self.state.invalidation.contains(InvalidationLevel::Full)
            && self.state.time_series.len() > 0
        {
            // Ensure viewport shows data range on first load
            if self.state.viewport.time_start == 0 && self.state.viewport.time_end == 3600_000 {
                if let Some(first) = self.state.time_series.get(0) {
                    if let Some(last) = self.state.time_series.latest() {
                        self.state.viewport.time_start = first.timestamp;
                        self.state.viewport.time_end = last.timestamp;
                        self.state.viewport.value_min = f64::MIN;
                        self.state.viewport.value_max = f64::MAX;
                    }
                }
            }
        }

        // NOTE: Rendering is no longer driven from tick().
        // The app layer calls GpuRenderer::render() directly with a
        // reference to ChartState, which consumes the invalidation mask.
        // tick() only manages data and marks state dirty.
    }

    /// Forward an interaction command to the handler and apply the resulting
    /// viewport commands to the chart state.
    pub fn handle_input(&mut self, command: InteractionCommand) {
        // 1. Pass to interaction handler
        let commands = self.interaction.handle_event(command);

        // 2. Process resulting ViewportCommands
        for cmd in commands {
            match cmd {
                ViewportCommand::SetTimeRange { start, end } => {
                    self.state.viewport.time_start = start;
                    self.state.viewport.time_end = end;
                    self.state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::SetValueRange { min, max } => {
                    self.state.viewport.value_min = min;
                    self.state.viewport.value_max = max;
                    self.state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::SetCrosshairPosition {
                    x,
                    y,
                    time,
                    price,
                } => {
                    self.state.crosshair.screen_x = x;
                    self.state.crosshair.screen_y = y;
                    self.state.crosshair.time = time;
                    self.state.crosshair.price = price;
                    self.state.crosshair.active = true;
                    self.state.mark_dirty(InvalidationLevel::Cursor);
                }
                ViewportCommand::DeactivateCrosshair => {
                    self.state.crosshair.deactivate();
                    self.state.mark_dirty(InvalidationLevel::Cursor);
                }
                ViewportCommand::RequestRedraw => {
                    self.state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::ZoomAtCursor { factor, screen_x } => {
                    // Stop kinetic scroll when zooming
                    self.kinetic.stop();
                    self.state.viewport.zoom(factor, screen_x);
                    self.state.mark_dirty(InvalidationLevel::Full);
                }
                ViewportCommand::PanBy { time_delta } => {
                    // Update kinetic scroll velocity from the pan delta
                    self.kinetic.start(time_delta as f64);
                    // Apply the immediate pan
                    self.state.viewport.pan(time_delta);
                    self.state.mark_dirty(InvalidationLevel::Full);
                }
            }
        }
    }

    pub fn state(&self) -> &ChartState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ChartState {
        &mut self.state
    }

    pub fn add_indicator_overlay(&mut self, name: String, data: TimeSeries<f64, CHART_CAPACITY>) {
        self.indicator_overlays.push((name, data));
        self.state.mark_dirty(InvalidationLevel::Full);
    }

    pub fn clear_indicator_overlays(&mut self) {
        self.indicator_overlays.clear();
        self.state.mark_dirty(InvalidationLevel::Full);
    }

    pub fn start_data_provider(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data_provider.start()
    }

    pub fn stop_data_provider(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data_provider.stop()
    }

    /// Update kinetic scroll. Call this each frame.
    /// Returns true if the viewport was displaced (caller should request redraw).
    pub fn update_kinetic(&mut self) -> bool {
        if !self.kinetic.is_active() {
            return false;
        }

        let displacement = self.kinetic.update();
        if displacement.abs() > 0.1 {
            self.state.viewport.pan(displacement as i64);
            self.state.mark_dirty(InvalidationLevel::Full);
            true
        } else {
            self.kinetic.stop();
            false
        }
    }

    /// Stop kinetic scrolling.
    pub fn stop_kinetic(&mut self) {
        self.kinetic.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::data_provider::DataProvider;
    use crate::ports::interaction::{InteractionHandler, InteractionCommand, ViewportCommand};
    use std::cell::RefCell;
    use std::error::Error;
    use std::sync::mpsc;

    // --- Mock Data Provider ---
    struct MockDataProvider {
        rx: Option<mpsc::Receiver<DataEvent>>,
        tx: Option<mpsc::Sender<DataEvent>>,
        started: bool,
    }

    impl MockDataProvider {
        fn new() -> Self {
            let (tx, rx) = mpsc::channel();
            Self {
                rx: Some(rx),
                tx: Some(tx),
                started: false,
            }
        }

        fn sender(&self) -> Option<mpsc::Sender<DataEvent>> {
            self.tx.clone()
        }
    }

    impl DataProvider for MockDataProvider {
        fn start(&mut self) -> Result<(), Box<dyn Error>> {
            self.started = true;
            Ok(())
        }

        fn receiver(&self) -> Option<&mpsc::Receiver<DataEvent>> {
            self.rx.as_ref()
        }

        fn stop(&mut self) -> Result<(), Box<dyn Error>> {
            self.started = false;
            Ok(())
        }

        fn name(&self) -> &str {
            "MockDataProvider"
        }
    }

    // --- Mock Interaction Handler ---
    struct MockInteractionHandler {
        responses: RefCell<Vec<Vec<ViewportCommand>>>,
    }

    impl MockInteractionHandler {
        fn new() -> Self {
            Self {
                responses: RefCell::new(Vec::new()),
            }
        }

        fn push_response(&self, cmds: Vec<ViewportCommand>) {
            self.responses.borrow_mut().push(cmds);
        }
    }

    impl InteractionHandler for MockInteractionHandler {
        fn handle_event(&self, _command: InteractionCommand) -> Vec<ViewportCommand> {
            self.responses.borrow_mut().pop().unwrap_or_default()
        }
    }

    fn make_controller() -> (ChartController, mpsc::Sender<DataEvent>) {
        let mock_provider = MockDataProvider::new();
        let sender = mock_provider.sender().unwrap();
        let provider = Box::new(mock_provider);
        let handler = Box::new(MockInteractionHandler::new());
        (ChartController::new(provider, handler), sender)
    }

    #[test]
    fn tick_processes_bar_events() {
        let (mut ctrl, tx) = make_controller();
        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();

        ctrl.tick();
        assert_eq!(ctrl.state().time_series.len(), 1);
        assert!(ctrl.state().time_series.latest().unwrap().timestamp == 1000);
    }

    #[test]
    fn tick_renders_when_dirty() {
        let (mut ctrl, tx) = make_controller();
        let bar = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 5000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();
        ctrl.tick();
        // After tick, invalidation stays set — the renderer consumes it later.
        assert!(ctrl.state().invalidation.level() > InvalidationLevel::Nothing);
    }

    #[test]
    fn tick_skips_render_when_no_data() {
        let (mut ctrl, _tx) = make_controller();
        ctrl.tick();
        // No data → no render
        assert_eq!(ctrl.state().time_series.len(), 0);
        assert!(ctrl.state().invalidation.is_empty());
    }

    #[test]
    fn tick_processes_multiple_events() {
        let (mut ctrl, tx) = make_controller();
        for i in 0..5 {
            let bar = Bar::new(i, 100.0, 105.0, 99.0, 102.0, 1000).unwrap();
            tx.send(DataEvent::BarClosed(bar)).unwrap();
        }
        ctrl.tick();
        assert_eq!(ctrl.state().time_series.len(), 5);
    }

    #[test]
    fn handle_input_sets_crosshair() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::SetCrosshairPosition {
            x: 400.0,
            y: 250.0,
            time: 5000,
            price: 105.0,
        }]);
        // We need to construct with our handler. Use a simple provider.
        let mock_provider = MockDataProvider::new();
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(provider, handler);

        ctrl.handle_input(InteractionCommand::UpdateCrosshair {
            screen_x: 400.0,
            screen_y: 250.0,
        });

        assert!(ctrl.state().crosshair.active);
        assert_eq!(ctrl.state().crosshair.screen_x, 400.0);
        assert_eq!(ctrl.state().crosshair.time, 5000);
    }

    #[test]
    fn handle_input_deactivates_crosshair() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::DeactivateCrosshair]);
        let mock_provider = MockDataProvider::new();
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(provider, handler);

        // First activate
        ctrl.state.crosshair.active = true;
        ctrl.handle_input(InteractionCommand::DeactivateCrosshair);
        assert!(!ctrl.state().crosshair.active);
    }

    #[test]
    fn handle_input_updates_viewport() {
        let handler = Box::new(MockInteractionHandler::new());
        handler.push_response(vec![ViewportCommand::SetTimeRange {
            start: 5000,
            end: 10000,
        }]);
        let mock_provider = MockDataProvider::new();
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(provider, handler);

        ctrl.handle_input(InteractionCommand::PanBy { time_delta: 5000 });
        assert_eq!(ctrl.state().viewport.time_start, 5000);
        assert_eq!(ctrl.state().viewport.time_end, 10000);
    }

    #[test]
    fn data_provider_start_stop() {
        let (mut ctrl, _tx) = make_controller();
        assert!(ctrl.start_data_provider().is_ok());
        assert!(ctrl.stop_data_provider().is_ok());
    }

    #[test]
    fn add_and_clear_indicator_overlays() {
        let (mut ctrl, _tx) = make_controller();
        let overlay: TimeSeries<f64, CHART_CAPACITY> = TimeSeries::new();
        ctrl.add_indicator_overlay("SMA(14)".to_string(), overlay);
        assert_eq!(ctrl.indicator_overlays.len(), 1);
        ctrl.clear_indicator_overlays();
        assert!(ctrl.indicator_overlays.is_empty());
    }
}
