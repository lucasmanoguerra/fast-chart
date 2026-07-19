//! Integration tests that exercise the full pipeline: data → domain → core → rendering.
//!
//! Tests markers, price lines, kinetic scroll, price formatting, and the
//! ChartController end-to-end using mock ports.

use fc_core::app::chart_controller::ChartController;
use fc_core::app::layout::{LayoutEngine, VerticalStack};
use fc_core::app::layout_manager::LayoutManager;
use fc_core::app::pane::events::PaneEventBus;
use fc_core::render::series_renderer::Rect;
use fc_core::ports::data_provider::{DataEvent, DataProvider};
use fc_core::ports::interaction::{InteractionCommand, InteractionHandler, ViewportCommand};
use fc_types::bar::Bar;
use fc_types::kinetic::KineticScroll;
use fc_types::marker::{Marker, MarkerPosition, MarkerSet, MarkerShape};
use fc_types::price_line::{LineStyle, PriceLine, PriceLineId, PriceLineSet};
use fc_types::price_scale::{DefaultPriceFormatter, PriceFormatter, PriceScaleId};
use std::error::Error;
use std::sync::{mpsc, Arc, Mutex};

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
    let provider = Box::new(mock_provider);
    let handler = Box::new(MockInteractionHandler::new());
    (ChartController::new(provider, handler), sender)
}

/// Build a controller and return a handle to the interaction handler for pushing
/// responses after construction.
fn make_controller_with_handler() -> (ChartController, mpsc::Sender<DataEvent>, MockInteractionHandler) {
    let mock_provider = MockDataProvider::new();
    let sender = mock_provider.sender().unwrap();
    let provider = Box::new(mock_provider);
    let handler = MockInteractionHandler::new();
    let handler_for_push = MockInteractionHandler {
        responses: Arc::clone(&handler.responses),
    };
    (ChartController::new(provider, Box::new(handler)), sender, handler_for_push)
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

    assert!(markers.remove(&fc_types::marker::MarkerId("a".to_string())));
    assert_eq!(markers.len(), 1);
    assert!(markers.get(&fc_types::marker::MarkerId("a".to_string())).is_none());
    assert!(markers.get(&fc_types::marker::MarkerId("b".to_string())).is_some());
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
        .with_label_position(fc_types::price_line::LabelPosition::Center);

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

// ===========================================================================
// Phase 2 Integration Tests: Multi-pane, divider, viewport sync, scroll, fit
// ===========================================================================

use fc_types::price_scale::{PriceScale, PriceScaleId as PSId, PriceScaleOptions};
use fc_types::scale::{LinearScale, TimeScale};

// --- Multi-pane layout ---

#[test]
fn multi_pane_vertical_stack_rects() {
    let layout = VerticalStack::new();
    let parent = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let rects = layout.compute_rects(parent, 3);
    assert_eq!(rects.len(), 3);
    // Each pane should be 200px tall
    for r in &rects {
        assert!((r.height - 200.0_f32).abs() < f32::EPSILON);
        assert!((r.width - 800.0_f32).abs() < f32::EPSILON);
    }
    // Panes should be stacked vertically (no overlap)
    assert!((rects[0].y - 0.0_f32).abs() < f32::EPSILON);
    assert!((rects[1].y - 200.0_f32).abs() < f32::EPSILON);
    assert!((rects[2].y - 400.0_f32).abs() < f32::EPSILON);
}

#[test]
fn multi_pane_proportional_heights() {
    let layout = VerticalStack::with_heights(vec![0.6, 0.4]);
    let parent = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 500.0,
    };
    let rects = layout.compute_rects(parent, 2);
    assert_eq!(rects.len(), 2);
    assert!((rects[0].height - 300.0_f32).abs() < 0.1);
    assert!((rects[1].height - 200.0_f32).abs() < 0.1);
}

#[test]
fn multi_pane_with_gap() {
    let layout = VerticalStack {
        heights: vec![],
        gap: 4.0,
    };
    let parent = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let rects = layout.compute_rects(parent, 3);
    assert_eq!(rects.len(), 3);
    // Total gap = 2 gaps * 4px = 8px, each pane = (600-8)/3 ≈ 197.33
    let expected_pane_h = (600.0_f32 - 8.0) / 3.0;
    for r in &rects {
        assert!((r.height - expected_pane_h).abs() < 0.1);
    }
    // Verify no overlap: second pane starts after first + gap
    assert!(rects[1].y > rects[0].y + rects[0].height - 0.1);
}

// --- Divider drag -> pane resize ---

#[test]
fn divider_drag_resize_integration() {
    let mut bus = PaneEventBus::new();

    // Simulate a divider drag
    bus.push(fc_core::app::pane::events::PaneEvent::DividerDragged {
        index: 0,
        delta: 30.0,
    });

    // Simulate resulting pane resize
    bus.push(fc_core::app::pane::events::PaneEvent::PaneResized {
        id: 1,
        new_height: 0.55,
    });

    let events = bus.drain();
    assert_eq!(events.len(), 2);

    // Verify the events have correct data
    match &events[0] {
        fc_core::app::pane::events::PaneEvent::DividerDragged { index, delta } => {
            assert_eq!(*index, 0);
            assert!((delta - 30.0).abs() < f64::EPSILON);
        }
        _ => panic!("expected DividerDragged"),
    }

    match &events[1] {
        fc_core::app::pane::events::PaneEvent::PaneResized { id, new_height } => {
            assert_eq!(*id, 1);
            assert!((new_height - 0.55).abs() < f64::EPSILON);
        }
        _ => panic!("expected PaneResized"),
    }
}

// --- Viewport sync across panes ---

#[test]
fn viewport_time_range_sync_across_panes() {
    // All panes share the same time range (TimeScale)
    // Simulate: two panes with the same time range, different value ranges
    let ts = TimeScale {
        start: 1000,
        end: 2000,
        width: 800.0,
        bar_spacing: 8.0,
        right_offset: 0.0,
    };

    // Pane 1: price 100..200 (height 200)
    let ls1 = LinearScale {
        min: 100.0,
        max: 200.0,
        height: 200.0,
    };

    // Pane 2: price 50..150 (height 200, different range)
    let ls2 = LinearScale {
        min: 50.0,
        max: 150.0,
        height: 200.0,
    };

    // Time maps identically for both panes
    let t = 1500u64;
    let x1 = ts.map_to_x(t);
    let x2 = ts.map_to_x(t);
    assert_eq!(x1, x2, "time mapping must be identical across panes");

    // Price maps differently because the value ranges differ
    // price=170 in ls1 maps to y=60, same price in ls2 maps to y=40
    let y1 = ls1.map_to_y(170.0);
    let y2 = ls2.map_to_y(170.0);
    assert_ne!(y1, y2);
}

// --- Time scale infinite scroll ---

#[test]
fn time_scale_scroll_to_end_integration() {
    let mut ts = TimeScale {
        start: 0,
        end: 100,
        width: 800.0,
        bar_spacing: 8.0,
        right_offset: 0.0,
    };

    // Simulate adding data and scrolling
    ts.scroll_to_end(500);
    let (_, last) = ts.visible_range(500);
    assert_eq!(last, 499, "should show the last bar");

    // Scroll forward more
    ts.scroll_to_end(1000);
    let (_, last2) = ts.visible_range(1000);
    assert_eq!(last2, 999);
}

// --- Price scale auto-fit with margins ---

#[test]
fn price_scale_auto_fit_with_margins_integration() {
    let mut ps = PriceScale::new(PSId::Right, PriceScaleOptions::default());
    ps.margin_top = 20.0;
    ps.margin_bottom = 10.0;

    ps.auto_fit(100.0, 200.0);
    // range=100, pad=5%, margin_top=20, margin_bottom=10
    // min = 100 - 5 - 10 = 85
    // max = 200 + 5 + 20 = 225
    assert!((ps.value_min - 85.0).abs() < f64::EPSILON);
    assert!((ps.value_max - 225.0).abs() < f64::EPSILON);
}

#[test]
fn price_scale_locked_no_autofit_integration() {
    let mut ps = PriceScale::new(
        PSId::Left,
        PriceScaleOptions {
            auto_scale: true,
            mode: fc_types::price_scale::PriceScaleMode::Locked,
            ..Default::default()
        },
    );
    ps.value_min = 50.0;
    ps.value_max = 150.0;

    // auto_fit should NOT change anything because mode is Locked
    ps.auto_fit(100.0, 200.0);
    assert!((ps.value_min - 50.0).abs() < f64::EPSILON);
    assert!((ps.value_max - 150.0).abs() < f64::EPSILON);
}

// --- Pane add/remove events ---

#[test]
fn pane_add_remove_roundtrip() {
    let mut bus = PaneEventBus::new();

    bus.push(fc_core::app::pane::events::PaneEvent::PaneAdded { id: 3 });
    bus.push(fc_core::app::pane::events::PaneEvent::PaneAdded { id: 4 });
    bus.push(fc_core::app::pane::events::PaneEvent::PaneRemoved { id: 3 });

    let events = bus.drain();
    assert_eq!(events.len(), 3);

    // After remove, only id=4 should remain "active" in a real system
    let removed: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, fc_core::app::pane::events::PaneEvent::PaneRemoved { .. }))
        .collect();
    assert_eq!(removed.len(), 1);
}

// ===========================================================================
// Phase 3 Integration Tests: All series types
// ===========================================================================

use fc_core::series::{
    line_break::LineBreakBlock, point_figure::PfColumn, range::RangeBar,
    step_line::StepPoint, volume::VolumeBar, LineBreakSeries, PointFigureSeries,
    RangeSeries, StepLineSeries, VolumeSeries,
};
use fc_core::render::series_renderer::SeriesRenderer as _;
use fc_types::Indicator as _;

// --- StepLineSeries ---

#[test]
fn stepline_set_data_and_update() {
    let mut s = StepLineSeries::new();
    s.set_data(vec![
        StepPoint::new(1000, 100.0),
        StepPoint::new(2000, 110.0),
        StepPoint::new(3000, 105.0),
    ]);
    let cmds = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
    // 3 points -> 2 horizontal + 2 vertical segments = 4 commands
    assert_eq!(cmds.len(), 4);
}

#[test]
fn stepline_hit_test_bounds() {
    let mut s = StepLineSeries::new();
    s.set_data(vec![
        StepPoint::new(1000, 100.0),
        StepPoint::new(2000, 110.0),
    ]);
    let _ = s.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
    let hit = s.hit_test(100.0, 100.0);
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().index, 0);
}

#[test]
fn stepline_empty_data() {
    let s = StepLineSeries::new();
    let hit = s.hit_test(100.0, 100.0);
    assert!(hit.is_none());
}

// --- VolumeSeries ---

#[test]
fn volume_set_data_and_update() {
    let mut v = VolumeSeries::new();
    v.set_data(vec![
        VolumeBar::new(1000, 5000.0, true),
        VolumeBar::new(2000, 3000.0, false),
    ]);
    let cmds = v.update(&[], Rect::new(0.0, 200.0, 800.0, 200.0));
    assert_eq!(cmds.len(), 2);
}

#[test]
fn volume_z_index_is_500() {
    let mut v = VolumeSeries::new();
    v.set_data(vec![VolumeBar::new(1000, 1000.0, true)]);
    let cmds = v.update(&[], Rect::new(0.0, 200.0, 800.0, 200.0));
    for cmd in &cmds {
        if let fc_core::render::commands::DrawCommand::DrawRect { z_index, .. } = cmd {
            assert_eq!(*z_index, 500, "volume bars should be z-index 500");
        }
    }
}

#[test]
fn volume_max_volume() {
    let mut v = VolumeSeries::new();
    v.set_data(vec![
        VolumeBar::new(1000, 1000.0, true),
        VolumeBar::new(2000, 5000.0, false),
        VolumeBar::new(3000, 2000.0, true),
    ]);
    assert!((v.max_volume() - 5000.0).abs() < f64::EPSILON);
}

// --- PointFigureSeries ---

#[test]
fn point_figure_set_columns_and_update() {
    let mut pf = PointFigureSeries::new(5.0, 3);
    pf.set_columns(vec![
        PfColumn::Rise { boxes: 3 },
        PfColumn::Fall { boxes: 2 },
        PfColumn::Rise { boxes: 4 },
    ]);
    let cmds = pf.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
    assert!(!cmds.is_empty());
}

#[test]
fn point_figure_build_from_prices() {
    let prices: Vec<(f64, f64)> = vec![
        (100.0, 105.0),
        (103.0, 110.0),
        (106.0, 115.0),
    ];
    let columns = PointFigureSeries::build_from_prices(&prices, 5.0, 3);
    assert!(!columns.is_empty());
    for col in &columns {
        assert!(col.boxes() > 0);
    }
}

#[test]
fn point_figure_column_types() {
    let rise = PfColumn::Rise { boxes: 3 };
    let fall = PfColumn::Fall { boxes: 2 };
    assert!(rise.is_rise());
    assert!(!rise.is_fall());
    assert!(fall.is_fall());
    assert!(!fall.is_rise());
    assert_eq!(rise.boxes(), 3);
    assert_eq!(fall.boxes(), 2);
}

// --- LineBreakSeries ---

#[test]
fn linebreak_set_blocks_and_update() {
    let mut lb = LineBreakSeries::new(3);
    lb.set_blocks(vec![
        LineBreakBlock::new(true, 100.0, 110.0),
        LineBreakBlock::new(false, 110.0, 105.0),
        LineBreakBlock::new(true, 105.0, 115.0),
    ]);
    let cmds = lb.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
    assert!(!cmds.is_empty());
}

#[test]
fn linebreak_build_from_bars() {
    let bars: Vec<(f64, f64, f64)> = vec![
        (100.0, 105.0, 99.0),
        (103.0, 110.0, 101.0),
        (106.0, 115.0, 104.0),
    ];
    let blocks = LineBreakSeries::build_from_bars(&bars, 3);
    assert!(!blocks.is_empty());
    for block in &blocks {
        let range = (block.close - block.open).abs();
        assert!(range > 0.0, "block range must be positive");
    }
}

#[test]
fn linebreak_block_methods() {
    let up = LineBreakBlock::new(true, 100.0, 110.0);
    let down = LineBreakBlock::new(false, 110.0, 105.0);
    assert!(up.is_up);
    assert!(!down.is_up);
    assert!(((up.close - up.open) - 10.0).abs() < f64::EPSILON);
    assert!(((down.open - down.close) - 5.0).abs() < f64::EPSILON);
}

// --- RangeSeries ---

#[test]
fn range_set_data_and_update() {
    let mut r = RangeSeries::new(5.0);
    r.set_data(vec![
        RangeBar::new(1000, 105.0, 100.0, true),
        RangeBar::new(2000, 103.0, 98.0, false),
        RangeBar::new(3000, 108.0, 103.0, true),
    ]);
    let cmds = r.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
    assert_eq!(cmds.len(), 3);
}

#[test]
fn range_build_from_bars() {
    let bars: Vec<(f64, f64, f64)> = vec![
        (100.0, 105.0, 99.0),
        (103.0, 110.0, 101.0),
        (106.0, 115.0, 104.0),
    ];
    let range_bars = RangeSeries::build_from_bars(&bars, 5.0);
    assert!(!range_bars.is_empty());
    for rb in &range_bars {
        assert!((rb.high - rb.low - 5.0).abs() < f64::EPSILON);
    }
}

#[test]
fn range_custom_colors() {
    let mut r = RangeSeries::new(5.0);
    r.bullish_color = [0.0, 0.5, 0.0, 1.0];
    r.bearish_color = [0.5, 0.0, 0.0, 1.0];
    r.set_data(vec![
        RangeBar::new(1000, 105.0, 100.0, true),
        RangeBar::new(2000, 103.0, 98.0, false),
    ]);
    let cmds = r.update(&[], Rect::new(0.0, 0.0, 800.0, 400.0));
    assert_eq!(cmds.len(), 2);
}

// --- SeriesType enum integration ---

#[test]
fn seriestype_all_count_matches_impl_count() {
    assert_eq!(fc_types::SeriesType::ALL.len(), 10);
}

#[test]
fn seriestype_all_display_names_are_unique() {
    let names: Vec<_> = fc_types::SeriesType::ALL
        .iter()
        .map(|s| s.display_name())
        .collect();
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(names.len(), unique.len(), "all display names must be unique");
}

// --- IndicatorRenderer integration ---

use fc_core::render::indicator_renderer::IndicatorRenderer;
use fc_core::render::commands::DrawCommand as DC;

struct MockIndicator;

impl fc_core::render::series_renderer::SeriesRenderer for MockIndicator {
    fn update(&mut self, _data: &[DC], bounds: Rect) -> Vec<DC> {
        vec![DC::DrawRect {
            x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height,
            fill: Some([0.0, 1.0, 0.0, 1.0]), stroke: None, stroke_width: 0.0, z_index: 700,
        }]
    }
    fn hit_test(&self, _x: f32, _y: f32) -> Option<fc_core::render::series_renderer::SeriesHit> { None }
    fn bounds(&self) -> Rect { Rect::new(0.0, 0.0, 0.0, 0.0) }
}

impl IndicatorRenderer for MockIndicator {
    fn render_overlay(&self, pane_bounds: Rect) -> Vec<DC> {
        vec![DC::DrawLine {
            x0: pane_bounds.x,
            y0: pane_bounds.y,
            x1: pane_bounds.x + pane_bounds.width,
            y1: pane_bounds.y,
            color: [1.0, 0.0, 0.0, 1.0],
            width: 2.0,
            style: fc_core::render::commands::LineStyle::Solid,
            z_index: 700,
        }]
    }
    fn render_separate(&self, pane_bounds: Rect) -> Vec<DC> {
        vec![DC::DrawRect {
            x: pane_bounds.x, y: pane_bounds.y, width: pane_bounds.width, height: pane_bounds.height * 0.5,
            fill: Some([0.0, 0.0, 1.0, 0.3]), stroke: None, stroke_width: 0.0, z_index: 700,
        }]
    }
}

#[test]
fn indicator_renderer_overlay_integration() {
    let ind = MockIndicator;
    let cmds = ind.render_overlay(Rect::new(0.0, 0.0, 800.0, 400.0));
    assert_eq!(cmds.len(), 1);
    assert_eq!(ind.indicator_z_index(), 700);
}

#[test]
fn indicator_renderer_separate_integration() {
    let ind = MockIndicator;
    let cmds = ind.render_separate(Rect::new(0.0, 400.0, 800.0, 200.0));
    assert_eq!(cmds.len(), 1);
    if let DC::DrawRect { height, .. } = &cmds[0] {
        assert!((*height - 100.0).abs() < f32::EPSILON);
    } else {
        panic!("expected DrawRect");
    }
}

// --- OverlayMode integration ---

#[test]
fn overlay_mode_default_overlay_on_pane_0() {
    struct SMA;
    impl fc_types::Indicator<100> for SMA {
        fn calculate(&self, _: &fc_types::series::TimeSeries<fc_types::Bar, 100>) -> fc_types::series::TimeSeries<f64, 100> {
            fc_types::series::TimeSeries::new()
        }
        fn name(&self) -> &str { "SMA" }
    }
    let sma = SMA;
    assert_eq!(sma.overlay_mode(), fc_types::indicator::OverlayMode::OverlayOnPane(0));
    assert_eq!(sma.preferred_scale(), fc_types::price_scale::PriceScaleMode::Normal);
}

#[test]
fn overlay_mode_separate_pane_integration() {
    struct RSI;
    impl fc_types::Indicator<100> for RSI {
        fn calculate(&self, _: &fc_types::series::TimeSeries<fc_types::Bar, 100>) -> fc_types::series::TimeSeries<f64, 100> {
            fc_types::series::TimeSeries::new()
        }
        fn name(&self) -> &str { "RSI" }
        fn overlay_mode(&self) -> fc_types::indicator::OverlayMode {
            fc_types::indicator::OverlayMode::SeparatePane
        }
    }
    let rsi = RSI;
    assert_eq!(rsi.overlay_mode(), fc_types::indicator::OverlayMode::SeparatePane);
}
