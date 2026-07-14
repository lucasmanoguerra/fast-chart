//! Integration tests that exercise the full pipeline: data → domain → core → rendering.
//!
//! Tests markers, price lines, kinetic scroll, price formatting, and the
//! ChartController end-to-end using mock ports.

use fast_chart_core::app::chart_controller::ChartController;
use fast_chart_core::app::layout_manager::LayoutManager;
use fast_chart_core::ports::data_provider::{DataEvent, DataProvider};
use fast_chart_core::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
use fast_chart_core::ports::render::ChartRenderer;
use fast_chart_domain::bar::Bar;
use fast_chart_domain::kinetic::KineticScroll;
use fast_chart_domain::marker::{Marker, MarkerPosition, MarkerSet, MarkerShape};
use fast_chart_domain::price_line::{LineStyle, PriceLine, PriceLineId, PriceLineSet};
use fast_chart_domain::price_scale::{DefaultPriceFormatter, PriceFormatter, PriceScaleId};
use std::error::Error;
use std::sync::{mpsc, Arc, Mutex};

// ---------------------------------------------------------------------------
// Mock Renderer
// ---------------------------------------------------------------------------

struct MockRenderer;

impl MockRenderer {
    fn new() -> Self {
        Self
    }
}

impl ChartRenderer for MockRenderer {
    fn resize(&mut self, _width: u32, _height: u32) {}
}

// ---------------------------------------------------------------------------
// Mock Data Provider
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Mock Interaction Handler (Send + Sync — uses Mutex for shared state)
// ---------------------------------------------------------------------------

struct MockInteractionHandler {
    responses: Arc<Mutex<Vec<Vec<ViewportCommand>>>>,
}

impl MockInteractionHandler {
    fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn push_response(&self, cmds: Vec<ViewportCommand>) {
        self.responses.lock().unwrap().push(cmds);
    }
}

impl InteractionHandler for MockInteractionHandler {
    fn handle_event(&self, _command: InteractionCommand) -> Vec<ViewportCommand> {
        self.responses.lock().unwrap().pop().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a controller with mock ports and return the data sender.
fn make_controller() -> (ChartController, mpsc::Sender<DataEvent>) {
    let mock_provider = MockDataProvider::new();
    let sender = mock_provider.sender().unwrap();
    let renderer = Box::new(MockRenderer::new());
    let provider = Box::new(mock_provider);
    let handler = Box::new(MockInteractionHandler::new());
    (ChartController::new(renderer, provider, handler), sender)
}

/// Build a controller and return a handle to the interaction handler for pushing
/// responses after construction.
fn make_controller_with_handler() -> (ChartController, mpsc::Sender<DataEvent>, MockInteractionHandler) {
    let mock_provider = MockDataProvider::new();
    let sender = mock_provider.sender().unwrap();
    let renderer = Box::new(MockRenderer::new());
    let provider = Box::new(mock_provider);
    let handler = MockInteractionHandler::new();
    let handler_for_push = MockInteractionHandler {
        responses: Arc::clone(&handler.responses),
    };
    (ChartController::new(renderer, provider, Box::new(handler)), sender, handler_for_push)
}

// ---------------------------------------------------------------------------
// Marker Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn markers_added_to_series_and_retrieved() {
    let mut markers = MarkerSet::new();

    markers.add(
        Marker::new("buy", 1000, 105.0)
            .with_position(MarkerPosition::BelowBar)
            .with_shape(MarkerShape::ArrowUp)
            .with_color([0.0, 1.0, 0.0, 1.0]),
    );

    markers.add(
        Marker::new("sell", 2000, 110.0)
            .with_position(MarkerPosition::AboveBar)
            .with_shape(MarkerShape::ArrowDown)
            .with_color([1.0, 0.0, 0.0, 1.0]),
    );

    assert_eq!(markers.len(), 2);

    let in_range = markers.in_range(500, 1500);
    assert_eq!(in_range.len(), 1);
    assert_eq!(in_range[0].id.0, "buy");
}

#[test]
fn markers_filtered_by_scale() {
    let mut markers = MarkerSet::new();

    markers.add(Marker::new("main", 1000, 105.0).with_scale(PriceScaleId::Right));

    markers.add(Marker::new("rsi", 1000, 70.0).with_scale(PriceScaleId::Left));

    let right_markers = markers.for_scale(&PriceScaleId::Right);
    assert_eq!(right_markers.len(), 1);
    assert_eq!(right_markers[0].id.0, "main");

    let left_markers = markers.for_scale(&PriceScaleId::Left);
    assert_eq!(left_markers.len(), 1);
    assert_eq!(left_markers[0].id.0, "rsi");
}

#[test]
fn markers_remove_by_id() {
    let mut markers = MarkerSet::new();
    markers.add(Marker::new("a", 100, 100.0));
    markers.add(Marker::new("b", 200, 110.0));

    assert!(markers.remove(&fast_chart_domain::marker::MarkerId("a".to_string())));
    assert_eq!(markers.len(), 1);
    assert!(markers.get(&fast_chart_domain::marker::MarkerId("a".to_string())).is_none());
    assert!(markers.get(&fast_chart_domain::marker::MarkerId("b".to_string())).is_some());
}

#[test]
fn markers_builder_chaining() {
    let marker = Marker::new("signal", 5000, 200.0)
        .with_position(MarkerPosition::AtPrice)
        .with_shape(MarkerShape::Triangle)
        .with_color([0.0, 0.0, 1.0, 1.0])
        .with_size(12.0)
        .with_label("Signal")
        .with_scale(PriceScaleId::Left);

    assert_eq!(marker.position, MarkerPosition::AtPrice);
    assert_eq!(marker.shape, MarkerShape::Triangle);
    assert_eq!(marker.color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(marker.size, 12.0);
    assert_eq!(marker.label, Some("Signal".to_string()));
    assert_eq!(marker.scale_id, PriceScaleId::Left);
}

// ---------------------------------------------------------------------------
// Price Line Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn price_lines_added_and_retrieved() {
    let mut price_lines = PriceLineSet::new();

    price_lines.add(
        PriceLine::new("support", 100.0)
            .with_color([0.0, 1.0, 0.0, 1.0])
            .with_style(LineStyle::Dashed),
    );

    price_lines.add(
        PriceLine::new("resistance", 120.0)
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_style(LineStyle::Solid),
    );

    assert_eq!(price_lines.len(), 2);

    let support = price_lines.get(&PriceLineId("support".to_string()));
    assert!(support.is_some());
    assert_eq!(support.unwrap().price, 100.0);
    assert_eq!(support.unwrap().style, LineStyle::Dashed);

    let resistance = price_lines.get(&PriceLineId("resistance".to_string()));
    assert!(resistance.is_some());
    assert_eq!(resistance.unwrap().price, 120.0);
}

#[test]
fn price_lines_filtered_by_scale() {
    let mut price_lines = PriceLineSet::new();

    price_lines.add(PriceLine::new("main_line", 100.0).with_scale(PriceScaleId::Right));

    price_lines.add(PriceLine::new("rsi_line", 70.0).with_scale(PriceScaleId::Left));

    let right_lines = price_lines.for_scale(&PriceScaleId::Right);
    assert_eq!(right_lines.len(), 1);
    assert_eq!(right_lines[0].id.0, "main_line");

    let left_lines = price_lines.for_scale(&PriceScaleId::Left);
    assert_eq!(left_lines.len(), 1);
    assert_eq!(left_lines[0].id.0, "rsi_line");
}

#[test]
fn price_lines_remove_by_id() {
    let mut price_lines = PriceLineSet::new();
    price_lines.add(PriceLine::new("a", 100.0));
    price_lines.add(PriceLine::new("b", 200.0));

    assert!(price_lines.remove(&PriceLineId("a".to_string())));
    assert_eq!(price_lines.len(), 1);
    assert!(price_lines.get(&PriceLineId("a".to_string())).is_none());
}

#[test]
fn price_lines_builder_chaining() {
    let line = PriceLine::new("entry", 150.0)
        .with_scale(PriceScaleId::Left)
        .with_color([1.0, 1.0, 0.0, 1.0])
        .with_width(2.5)
        .with_style(LineStyle::Dotted)
        .with_label("Entry")
        .with_label_position(fast_chart_domain::price_line::LabelPosition::Center);

    assert_eq!(line.scale_id, PriceScaleId::Left);
    assert_eq!(line.width, 2.5);
    assert_eq!(line.style, LineStyle::Dotted);
    assert_eq!(line.label, Some("Entry".to_string()));
}

// ---------------------------------------------------------------------------
// Kinetic Scroll Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn kinetic_scroll_start_and_update() {
    let mut kinetic = KineticScroll::new(0.9);

    assert!(!kinetic.is_active());

    kinetic.start(100.0);
    assert!(kinetic.is_active());
    assert_eq!(kinetic.velocity(), 100.0);

    let d1 = kinetic.update();
    assert_eq!(d1, 100.0);
    assert!((kinetic.velocity() - 90.0).abs() < f64::EPSILON);
}

#[test]
fn kinetic_scroll_stops_at_threshold() {
    let mut kinetic = KineticScroll::new(0.5);
    kinetic.start(10.0);

    let mut frames = 0;
    while kinetic.is_active() && frames < 100 {
        kinetic.update();
        frames += 1;
    }

    assert!(!kinetic.is_active());
    assert!(frames < 50);
}

#[test]
fn kinetic_scroll_stop() {
    let mut kinetic = KineticScroll::new(0.9);
    kinetic.start(100.0);
    kinetic.stop();

    assert!(!kinetic.is_active());
    assert_eq!(kinetic.velocity(), 0.0);
}

#[test]
fn kinetic_scroll_custom_friction() {
    // High friction = fast stop
    let mut fast = KineticScroll::new(0.1);
    fast.start(100.0);
    fast.update();
    // velocity = 100 * 0.1 = 10
    assert!((fast.velocity() - 10.0).abs() < f64::EPSILON);

    // Low friction = slow stop
    let mut slow = KineticScroll::new(0.99);
    slow.start(100.0);
    slow.update();
    // velocity = 100 * 0.99 = 99
    assert!((slow.velocity() - 99.0).abs() < f64::EPSILON);
}

#[test]
fn kinetic_scroll_negative_velocity() {
    let mut kinetic = KineticScroll::new(0.9);
    kinetic.start(-50.0);
    assert!(kinetic.is_active());

    let d = kinetic.update();
    assert_eq!(d, -50.0);
    assert!((kinetic.velocity() - (-45.0)).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Price Formatter Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn price_formatter_format_price() {
    let formatter = DefaultPriceFormatter::new(None);

    // Default auto-detect: >= 1.0 → 2 decimals
    assert_eq!(formatter.format(1234.56), "1234.56");
    // < 0.01 → 5 decimals
    assert_eq!(formatter.format(0.005), "0.00500");
    // else → 4 decimals
    assert_eq!(formatter.format(0.5), "0.5000");
}

#[test]
fn price_formatter_format_short() {
    let formatter = DefaultPriceFormatter::new(None);

    // >= 1000 → K suffix
    assert_eq!(formatter.format_short(1234.5), "1.2K");
    // >= 1000 → K suffix (no M suffix in current impl)
    assert_eq!(formatter.format_short(1_500_000.0), "1500.0K");
    // < 1000 → normal format
    assert_eq!(formatter.format_short(50.0), "50.00");
}

#[test]
fn price_formatter_explicit_decimals() {
    let formatter = DefaultPriceFormatter::new(Some(4));
    assert_eq!(formatter.format(105.2), "105.2000");
}

#[test]
fn price_formatter_nan_and_infinity() {
    let formatter = DefaultPriceFormatter::new(None);
    assert_eq!(formatter.format(f64::NAN), "NaN");
    assert_eq!(formatter.format(f64::INFINITY), "∞");
    assert_eq!(formatter.format(f64::NEG_INFINITY), "-∞");
    assert_eq!(formatter.format_short(f64::NAN), "NaN");
}

// ---------------------------------------------------------------------------
// Chart Controller Integration Tests — Kinetic
// ---------------------------------------------------------------------------

#[test]
fn chart_controller_kinetic_update() {
    let (mut ctrl, _tx, handler) = make_controller_with_handler();

    // Push a PanBy response so handle_input actually starts kinetic
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 1000 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 1000 });

    // Update kinetic — should return true because velocity > threshold
    let should_redraw = ctrl.update_kinetic();
    assert!(should_redraw);
}

#[test]
fn chart_controller_kinetic_stop_on_zoom() {
    let (mut ctrl, _tx, handler) = make_controller_with_handler();

    // Start kinetic scroll via PanBy
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 1000 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 1000 });
    // Kinetic should be active now (update_kinetic returns true)
    assert!(ctrl.update_kinetic());

    // Zoom should stop kinetic
    handler.push_response(vec![ViewportCommand::ZoomAtCursor {
        factor: 1.1,
        screen_x: 400.0,
    }]);
    ctrl.handle_input(InteractionCommand::ZoomAtCursor {
        factor: 1.1,
        screen_x: 400.0,
    });

    // After zoom, kinetic should be inactive
    assert!(!ctrl.update_kinetic());
}

#[test]
fn chart_controller_kinetic_decelerates() {
    let (mut ctrl, _tx, handler) = make_controller_with_handler();

    // Start with a moderate velocity
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 50 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 50 });

    let mut frames = 0;
    while ctrl.update_kinetic() {
        frames += 1;
        if frames > 200 {
            break; // safety
        }
    }

    assert!(frames > 0, "kinetic should run for at least one frame");
    assert!(frames < 200, "kinetic should stop within 200 frames");
}

// ---------------------------------------------------------------------------
// Chart Controller Integration Tests — Data Pipeline
// ---------------------------------------------------------------------------

#[test]
fn chart_controller_tick_processes_bars() {
    let (mut ctrl, tx) = make_controller();

    for i in 0..10 {
        let bar = Bar::new(
            i * 1000,
            100.0 + i as f64,
            105.0 + i as f64,
            99.0 + i as f64,
            102.0 + i as f64,
            1000,
        )
        .unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();
    }

    ctrl.tick();
    assert_eq!(ctrl.state().time_series.len(), 10);
}

#[test]
fn chart_controller_tick_updates_viewport_on_first_data() {
    let (mut ctrl, tx) = make_controller();

    // Send bars with specific timestamps
    let bar1 = Bar::new(1000, 100.0, 105.0, 99.0, 102.0, 1000).unwrap();
    let bar2 = Bar::new(5000, 110.0, 115.0, 109.0, 112.0, 1000).unwrap();
    tx.send(DataEvent::BarClosed(bar1)).unwrap();
    tx.send(DataEvent::BarClosed(bar2)).unwrap();

    ctrl.tick();

    // Viewport should have auto-fit to data range
    assert_eq!(ctrl.state().viewport.time_start, 1000);
    assert_eq!(ctrl.state().viewport.time_end, 5000);
}

#[test]
fn chart_controller_pan_and_zoom() {
    let (mut ctrl, tx, handler) = make_controller_with_handler();

    // Load some data first
    for i in 0..5 {
        let bar = Bar::new(i * 1000, 100.0, 105.0, 99.0, 102.0, 1000).unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();
    }
    ctrl.tick();

    let original_start = ctrl.state().viewport.time_start;

    // Pan via handler
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 5000 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 5000 });
    assert_eq!(ctrl.state().viewport.time_start, original_start + 5000);

    // Zoom via handler
    handler.push_response(vec![ViewportCommand::ZoomAtCursor {
        factor: 0.5,
        screen_x: 400.0,
    }]);
    ctrl.handle_input(InteractionCommand::ZoomAtCursor {
        factor: 0.5,
        screen_x: 400.0,
    });
    assert!(ctrl.state().viewport.zoom_level < 1.0);
}

#[test]
fn chart_controller_crosshair_lifecycle() {
    let (mut ctrl, _tx, handler) = make_controller_with_handler();

    // Activate crosshair
    handler.push_response(vec![ViewportCommand::SetCrosshairPosition {
        x: 400.0,
        y: 250.0,
        time: 5000,
        price: 105.0,
    }]);
    ctrl.handle_input(InteractionCommand::UpdateCrosshair {
        screen_x: 400.0,
        screen_y: 250.0,
    });
    assert!(ctrl.state().crosshair.active);
    assert_eq!(ctrl.state().crosshair.screen_x, 400.0);
    assert_eq!(ctrl.state().crosshair.time, 5000);
    assert!((ctrl.state().crosshair.price - 105.0).abs() < f64::EPSILON);

    // Deactivate crosshair
    handler.push_response(vec![ViewportCommand::DeactivateCrosshair]);
    ctrl.handle_input(InteractionCommand::DeactivateCrosshair);
    assert!(!ctrl.state().crosshair.active);
}

// ---------------------------------------------------------------------------
// Layout Manager Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn layout_manager_default_pane_structure() {
    let layout = LayoutManager::new();
    assert_eq!(layout.panes.len(), 2);
    assert!((layout.total_height() - 1.0).abs() < 0.001);
}

#[test]
fn layout_manager_add_remove_pane() {
    let mut layout = LayoutManager::new();
    layout.add_pane(0.2);
    assert_eq!(layout.panes.len(), 3);
    assert!((layout.total_height() - 1.0).abs() < 0.001);

    layout.remove_pane(1);
    assert_eq!(layout.panes.len(), 2);
    assert!((layout.total_height() - 1.0).abs() < 0.001);
}

#[test]
fn layout_manager_sync_time_across_panes() {
    let mut layout = LayoutManager::new();
    layout.sync_time_range(1000, 5000);

    for pane in &layout.panes {
        assert_eq!(pane.viewport.time_start, 1000);
        assert_eq!(pane.viewport.time_end, 5000);
    }
}

#[test]
fn layout_manager_divider_drag() {
    let mut layout = LayoutManager::new();
    let original_top = layout.panes[0].height;

    layout.start_drag(0);
    layout.update_drag(35.0, 700.0); // drag down 5%
    layout.end_drag();

    assert!(layout.panes[0].height > original_top);
    assert!((layout.total_height() - 1.0).abs() < 0.001);
}

#[test]
fn layout_manager_min_height_enforced() {
    let mut layout = LayoutManager::new();
    layout.start_drag(0);
    layout.update_drag(-700.0, 700.0); // try to make top pane tiny
    layout.end_drag();
    assert!(layout.panes[0].height >= layout.min_pane_height() - 0.001);
}

#[test]
fn layout_manager_pane_with_markers_and_lines() {
    let mut layout = LayoutManager::new();

    // Add markers to main pane
    layout.panes[0].markers_mut().add(
        Marker::new("buy", 1000, 105.0).with_position(MarkerPosition::BelowBar),
    );

    // Add price lines to main pane
    layout.panes[0].price_lines_mut().add(
        PriceLine::new("support", 100.0).with_style(LineStyle::Dashed),
    );

    assert_eq!(layout.panes[0].markers().len(), 1);
    assert_eq!(layout.panes[0].price_lines().len(), 1);
}

// ---------------------------------------------------------------------------
// Full Pipeline Integration Test
// ---------------------------------------------------------------------------

#[test]
fn full_pipeline_data_to_render() {
    let (mut ctrl, tx, handler) = make_controller_with_handler();

    // 1. Send bar data
    for i in 0..10 {
        let bar = Bar::new(
            i * 1000,
            100.0 + i as f64,
            105.0 + i as f64,
            99.0 + i as f64,
            102.0 + i as f64,
            1000,
        )
        .unwrap();
        tx.send(DataEvent::BarClosed(bar)).unwrap();
    }

    // 2. Process data
    ctrl.tick();
    assert_eq!(ctrl.state().time_series.len(), 10);

    // 3. Verify viewport auto-fit
    assert_eq!(ctrl.state().viewport.time_start, 0);
    assert_eq!(ctrl.state().viewport.time_end, 9000);

    // 4. Pan should work
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 5000 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 5000 });
    assert_eq!(ctrl.state().viewport.time_start, 5000);

    // 5. Zoom should work
    handler.push_response(vec![ViewportCommand::ZoomAtCursor {
        factor: 0.5,
        screen_x: 400.0,
    }]);
    ctrl.handle_input(InteractionCommand::ZoomAtCursor {
        factor: 0.5,
        screen_x: 400.0,
    });
    assert!(ctrl.state().viewport.zoom_level < 1.0);

    // 6. Kinetic should work
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 100 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 100 });
    assert!(ctrl.update_kinetic());
}

#[test]
fn full_pipeline_domain_sets_on_pane() {
    let mut layout = LayoutManager::new();

    // Add markers
    layout.panes[0].markers_mut().add(
        Marker::new("entry", 1000, 105.0)
            .with_position(MarkerPosition::BelowBar)
            .with_shape(MarkerShape::ArrowUp)
            .with_color([0.0, 1.0, 0.0, 1.0]),
    );

    // Add price lines
    layout.panes[0].price_lines_mut().add(
        PriceLine::new("stop", 98.0)
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_style(LineStyle::Dashed),
    );

    // Verify markers
    let markers = layout.panes[0].markers();
    assert_eq!(markers.len(), 1);
    let in_range = markers.in_range(500, 1500);
    assert_eq!(in_range.len(), 1);
    assert_eq!(in_range[0].position, MarkerPosition::BelowBar);

    // Verify price lines
    let lines = layout.panes[0].price_lines();
    assert_eq!(lines.len(), 1);
    let support = lines.get(&PriceLineId("stop".to_string()));
    assert!(support.is_some());
    assert_eq!(support.unwrap().price, 98.0);

    // Verify formatter
    let formatted = layout.panes[0].formatter().format(105.2);
    assert_eq!(formatted, "105.20");
}

#[test]
fn full_pipeline_kinetic_then_render() {
    let (mut ctrl, _tx, handler) = make_controller_with_handler();

    // Start kinetic scroll with moderate velocity
    handler.push_response(vec![ViewportCommand::PanBy { time_delta: 50 }]);
    ctrl.handle_input(InteractionCommand::PanBy { time_delta: 50 });

    // Run kinetic for several frames
    let mut frame_count = 0;
    while ctrl.update_kinetic() {
        frame_count += 1;
        if frame_count > 200 {
            break;
        }
    }

    assert!(frame_count > 0, "kinetic should produce at least one frame");
    assert!(!ctrl.update_kinetic(), "kinetic should be stopped after deceleration");
}
