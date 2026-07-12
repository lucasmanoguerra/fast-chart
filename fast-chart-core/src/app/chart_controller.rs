use crate::ports::data_provider::{DataEvent, DataProvider};
use crate::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
use crate::ports::render::ChartRenderer;
use fast_chart_domain::bar::Bar;
use fast_chart_domain::crosshair::Crosshair;
use fast_chart_domain::series::TimeSeries;
use fast_chart_domain::series_type::SeriesType;
use fast_chart_domain::viewport::Viewport;

const CHART_CAPACITY: usize = 100_000;

pub struct ChartState {
    pub time_series: TimeSeries<Bar, CHART_CAPACITY>,
    pub viewport: Viewport,
    pub crosshair: Crosshair,
    pub pane_heights: Vec<f64>,
    pub series_type: SeriesType,
    pub needs_redraw: bool,
}

impl ChartState {
    fn new() -> Self {
        Self {
            time_series: TimeSeries::new(),
            viewport: Viewport::default(),
            crosshair: Crosshair::default(),
            pane_heights: vec![400.0],
            series_type: SeriesType::default(),
            needs_redraw: false,
        }
    }
}

pub struct ChartController {
    renderer: Box<dyn ChartRenderer>,
    data_provider: Box<dyn DataProvider>,
    interaction: Box<dyn InteractionHandler>,
    state: ChartState,
    indicator_overlays: Vec<(String, TimeSeries<f64, CHART_CAPACITY>)>,
}

impl ChartController {
    pub fn new(
        renderer: Box<dyn ChartRenderer>,
        data_provider: Box<dyn DataProvider>,
        interaction: Box<dyn InteractionHandler>,
    ) -> Self {
        Self {
            renderer,
            data_provider,
            interaction,
            state: ChartState::new(),
            indicator_overlays: Vec::new(),
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
                        self.state.needs_redraw = true;
                    }
                    DataEvent::TickUpdate(_tick) => {
                        // Tick updates update the latest bar in-place
                        // For now, mark dirty — actual tick→bar aggregation
                        // lives in the adapter layer.
                        self.state.needs_redraw = true;
                    }
                    DataEvent::SymbolChanged(_) | DataEvent::TimeframeChanged(_) => {
                        // Symbol/timeframe changes require a full data reload
                        // at the adapter level. Core just marks dirty.
                        self.state.needs_redraw = true;
                    }
                }
            }
        }

        // 2. Update viewport if dirty (auto-fit on first data)
        if self.state.needs_redraw && self.state.time_series.len() > 0 {
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

        // 3. Call renderer.render() if needs_redraw
        if self.state.needs_redraw {
            let _ = self.renderer.render(&self.state);
            self.state.needs_redraw = false;
        }
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
                    self.state.needs_redraw = true;
                }
                ViewportCommand::SetValueRange { min, max } => {
                    self.state.viewport.value_min = min;
                    self.state.viewport.value_max = max;
                    self.state.needs_redraw = true;
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
                    self.state.needs_redraw = true;
                }
                ViewportCommand::DeactivateCrosshair => {
                    self.state.crosshair.deactivate();
                    self.state.needs_redraw = true;
                }
                ViewportCommand::RequestRedraw => {
                    self.state.needs_redraw = true;
                }
            }
        }
    }

    pub fn state(&self) -> &ChartState {
        &self.state
    }

    pub fn add_indicator_overlay(&mut self, name: String, data: TimeSeries<f64, CHART_CAPACITY>) {
        self.indicator_overlays.push((name, data));
        self.state.needs_redraw = true;
    }

    pub fn clear_indicator_overlays(&mut self) {
        self.indicator_overlays.clear();
        self.state.needs_redraw = true;
    }

    pub fn start_data_provider(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data_provider.start()
    }

    pub fn stop_data_provider(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data_provider.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::data_provider::DataProvider;
    use crate::ports::interaction::{InteractionHandler, InteractionCommand, ViewportCommand};
    use crate::ports::render::ChartRenderer;
    use std::cell::RefCell;
    use std::error::Error;
    use std::sync::mpsc;

    // --- Mock Renderer ---
    struct MockRenderer {
        render_count: usize,
        _last_state: Option<ChartStateSnapshot>,
    }

    #[derive(Clone)]
    struct ChartStateSnapshot {
        _series_len: usize,
        _needs_redraw: bool,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                render_count: 0,
                _last_state: None,
            }
        }
    }

    impl ChartRenderer for MockRenderer {
        fn render(&mut self, state: &ChartState) -> Result<(), Box<dyn Error>> {
            self.render_count += 1;
            self._last_state = Some(ChartStateSnapshot {
                _series_len: state.time_series.len(),
                _needs_redraw: state.needs_redraw,
            });
            Ok(())
        }

        fn resize(&mut self, _width: u32, _height: u32) {}
    }

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
        let renderer = Box::new(MockRenderer::new());
        let provider = Box::new(mock_provider);
        let handler = Box::new(MockInteractionHandler::new());
        (ChartController::new(renderer, provider, handler), sender)
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
        // After tick, needs_redraw should be false (renderer was called)
        assert!(!ctrl.state().needs_redraw);
    }

    #[test]
    fn tick_skips_render_when_no_data() {
        let (mut ctrl, _tx) = make_controller();
        ctrl.tick();
        // No data → no render
        assert_eq!(ctrl.state().time_series.len(), 0);
        assert!(!ctrl.state().needs_redraw);
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
        // We need to construct with our handler. Use a simple renderer/provider.
        let mock_provider = MockDataProvider::new();
        let renderer = Box::new(MockRenderer::new());
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(renderer, provider, handler);

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
        let renderer = Box::new(MockRenderer::new());
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(renderer, provider, handler);

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
        let renderer = Box::new(MockRenderer::new());
        let provider = Box::new(mock_provider);
        let mut ctrl = ChartController::new(renderer, provider, handler);

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
