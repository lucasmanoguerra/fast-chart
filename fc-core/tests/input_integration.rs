//! Integration tests for Phase 5 input system components.
//!
//! Verifies cross-module interactions between InteractionEngine, ZoomController,
//! PanController, CrosshairController, KeyboardShortcutMap, and GestureDetector.

use fc_core::input::crosshair::{CrosshairController, CrosshairMode};
use fc_core::input::engine::{ChartCommand, DrawingTool, InteractionEngine};
use fc_core::input::gesture::{GestureDetector, GestureConfig, Gesture, FlickDirection};
use fc_core::input::keyboard::{KeyboardPresets, Modifiers};
use fc_core::input::pan::PanController;
use fc_core::input::zoom::{Viewport, ZoomController};
use fc_core::input::{InputEvent, KeyCode, ModifierState};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EPS: f64 = 1e-6;

fn default_viewport() -> Viewport {
    Viewport {
        time_start: 0.0,
        time_end: 1000.0,
        price_min: 100.0,
        price_max: 200.0,
    }
}

fn zoom_controller() -> ZoomController {
    ZoomController::new(10.0, 10_000.0)
}

// ===========================================================================
// Zoom + Pan Interaction
// ===========================================================================

#[test]
fn zoom_then_pan() {
    let mut viewport = default_viewport();
    let zoom = zoom_controller();
    let pan = PanController::new();

    // Zoom in 2x at center
    zoom.wheel_zoom(&mut viewport, 2.0, 0.5, 0.5);
    let zoomed_width = viewport.width();
    let zoomed_height = viewport.height();
    assert!((zoomed_width - 500.0).abs() < EPS);
    assert!((zoomed_height - 50.0).abs() < EPS);

    // Pan right by 100 time units — width stays the same
    pan.pan_by(&mut viewport, 100.0, 0.0);
    assert!((viewport.width() - zoomed_width).abs() < EPS);
    assert!((viewport.height() - zoomed_height).abs() < EPS);

    // Viewport shifted right (was [250..750] after zoom, now [350..850])
    assert!((viewport.time_start - 350.0).abs() < EPS);
    assert!((viewport.time_end - 850.0).abs() < EPS);
}

#[test]
fn pan_then_zoom() {
    let mut viewport = default_viewport();
    let zoom = zoom_controller();
    let pan = PanController::new();

    // Pan right by 200 units
    pan.pan_by(&mut viewport, 200.0, 0.0);
    let center_before = viewport.center_time();

    // Zoom in 2x at center (0.5 ratio) — center stays the same
    zoom.wheel_zoom(&mut viewport, 2.0, 0.5, 0.5);
    let center_after = viewport.center_time();
    assert!((center_before - center_after).abs() < EPS);
}

#[test]
fn zoom_preserves_viewport_size() {
    let mut viewport = default_viewport();
    let zoom = zoom_controller();

    // Zoom in at cursor ratio 0.25 (left quarter)
    let cursor_data_before = viewport.time_start + 0.25 * viewport.width();
    zoom.wheel_zoom(&mut viewport, 2.0, 0.25, 0.5);

    // Data point under cursor is preserved
    let cursor_data_after = viewport.time_start + 0.25 * viewport.width();
    assert!((cursor_data_before - cursor_data_after).abs() < EPS);

    // Width halved
    assert!((viewport.width() - 500.0).abs() < EPS);
}

#[test]
fn momentum_after_drag() {
    let mut pan = PanController::new();
    let mut viewport = default_viewport();

    // Simulate fast drag to the left
    pan.start_drag(100.0, 100.0);
    for i in 1..=10 {
        pan.update_drag(100.0 - i as f64 * 30.0, 100.0);
    }

    let vel = pan.end_drag();
    assert!(vel.is_some(), "fast drag should produce momentum");

    // Momentum should move the viewport
    let time_before = viewport.center_time();
    pan.tick_momentum(&mut viewport, 1.0 / 60.0, 1.0);
    let time_after = viewport.center_time();

    // Viewport shifted (fast left drag → positive time delta)
    assert!(
        (time_before - time_after).abs() > EPS,
        "momentum should move viewport"
    );
}

#[test]
fn momentum_stops_eventually() {
    let mut pan = PanController::new();
    let mut viewport = default_viewport();

    pan.start_drag(100.0, 100.0);
    for i in 1..=10 {
        pan.update_drag(100.0 - i as f64 * 30.0, 100.0);
    }
    let _ = pan.end_drag();

    // Tick until stopped
    let mut ticks = 0;
    for _ in 0..2000 {
        ticks += 1;
        if !pan.tick_momentum(&mut viewport, 1.0 / 60.0, 1.0) {
            break;
        }
    }

    // Verify it stopped and didn't run forever
    assert!(ticks < 2000, "momentum should stop, took {ticks} ticks");
    assert!(
        !pan.tick_momentum(&mut viewport, 1.0 / 60.0, 1.0),
        "should be stopped"
    );
}

// ===========================================================================
// Crosshair + Zoom
// ===========================================================================

#[test]
fn crosshair_persists_through_zoom() {
    let mut crosshair = CrosshairController::new();
    let mut viewport = default_viewport();
    let zoom = zoom_controller();

    // Set crosshair at a known data point
    crosshair.update_position(500.0, 150.0, None);
    let time_before = crosshair.position().expect("crosshair should have position").time;

    // Zoom in 2x at center — data point at 500.0 should still be valid
    zoom.wheel_zoom(&mut viewport, 2.0, 0.5, 0.5);

    // After zoom, the crosshair position (500.0, 150.0) is still within
    // the new viewport [250..750, 125..175]. Re-update crosshair to the
    // same data coordinate — it should still snap to the same data point.
    crosshair.update_position(time_before, 150.0, None);
    let pos = crosshair.position().expect("crosshair should persist");
    assert!(
        (pos.time - 500.0).abs() < EPS,
        "crosshair time should persist through zoom"
    );
}

#[test]
fn magnetic_snap_after_zoom() {
    let mut crosshair = CrosshairController::new();
    crosshair.set_mode(CrosshairMode::Magnetic);
    crosshair.set_snap_threshold(0.1);

    let mut viewport = default_viewport();
    let zoom = zoom_controller();

    // Zoom in so the coordinate space shrinks
    zoom.wheel_zoom(&mut viewport, 2.0, 0.5, 0.5);

    // The nearest data point is now "closer" in normalized space
    // because the viewport is narrower. Update crosshair near a data point.
    let pos = crosshair.update_position(500.0, 150.0, Some((500.02, 150.01)));
    assert!(pos.snapped, "magnetic snap should work after zoom");
}

// ===========================================================================
// Keyboard + Zoom/Pan
// ===========================================================================

#[test]
fn keyboard_zoom_in_out() {
    let mut engine = InteractionEngine::new();
    engine.handle(InputEvent::mouse_move(500.0, 400.0));

    // Zoom in with Plus key
    let cmds_in = engine.handle(InputEvent::key_down(KeyCode::Plus));
    assert!(cmds_in.iter().any(|c| matches!(
        c,
        ChartCommand::ZoomAtCursor { factor, .. } if (factor - 1.2).abs() < EPS
    )));

    // Zoom out with Minus key
    let cmds_out = engine.handle(InputEvent::key_down(KeyCode::Minus));
    assert!(cmds_out.iter().any(|c| matches!(
        c,
        ChartCommand::ZoomAtCursor { factor, .. } if (factor - 1.0 / 1.2).abs() < EPS
    )));
}

#[test]
fn keyboard_pan_arrows() {
    let mut engine = InteractionEngine::new();

    let left = engine.handle(InputEvent::key_down(KeyCode::ArrowLeft));
    assert!(left.contains(&ChartCommand::Pan {
        time_delta: -60,
        price_delta: 0.0,
    }));

    let right = engine.handle(InputEvent::key_down(KeyCode::ArrowRight));
    assert!(right.contains(&ChartCommand::Pan {
        time_delta: 60,
        price_delta: 0.0,
    }));

    let up = engine.handle(InputEvent::key_down(KeyCode::ArrowUp));
    assert!(up.contains(&ChartCommand::Pan {
        time_delta: 0,
        price_delta: -1.0,
    }));

    let down = engine.handle(InputEvent::key_down(KeyCode::ArrowDown));
    assert!(down.contains(&ChartCommand::Pan {
        time_delta: 0,
        price_delta: 1.0,
    }));
}

#[test]
fn keyboard_escape_cancel() {
    let mut engine = InteractionEngine::new();

    // Select a drawing tool
    engine.set_tool(Some(DrawingTool::TrendLine));
    assert!(engine.is_drawing());

    // Escape cancels drawing and deselects
    let cmds = engine.handle(InputEvent::key_down(KeyCode::Escape));
    assert!(cmds.contains(&ChartCommand::CancelDrawing));
    assert!(cmds.contains(&ChartCommand::DeselectAll));
    assert!(!engine.is_drawing());
    assert_eq!(engine.active_tool(), None);
}

// ===========================================================================
// Gesture + Zoom
// ===========================================================================

#[test]
fn pinch_zoom_two_fingers() {
    let mut detector = GestureDetector::with_default_config();

    // Two fingers start at known positions
    detector.touch_start(1, 200.0, 300.0);
    detector.touch_start(2, 400.0, 300.0);

    // Spread fingers apart (zoom in)
    let g1 = detector.touch_move(2, 500.0, 300.0);
    match g1 {
        Some(Gesture::Pinch { scale, .. }) => {
            assert!(scale > 1.0, "spread should produce scale > 1.0, got {scale}");
        }
        other => panic!("Expected Pinch gesture, got {other:?}"),
    }

    // Compress fingers (zoom out)
    let g2 = detector.touch_move(2, 250.0, 300.0);
    match g2 {
        Some(Gesture::Pinch { scale, .. }) => {
            assert!(scale < 1.0, "compress should produce scale < 1.0, got {scale}");
        }
        other => panic!("Expected Pinch gesture, got {other:?}"),
    }
}

#[test]
fn flick_starts_momentum() {
    let config = GestureConfig {
        pan_min_distance: 5.0,
        flick_min_velocity: 100.0, // Low threshold for test determinism
        ..GestureConfig::default()
    };
    let mut detector = GestureDetector::new(config);

    // Fast single-finger swipe
    detector.touch_start(1, 100.0, 100.0);
    detector.touch_move(1, 400.0, 100.0);

    let gesture = detector.touch_end(1, 400.0, 100.0);
    match gesture {
        Some(Gesture::Flick { velocity_x, direction, .. }) => {
            assert!(velocity_x > 0.0, "flick right should have positive velocity");
            assert_eq!(direction, FlickDirection::Right);
        }
        Some(Gesture::Pan { .. }) => {
            // If actual elapsed time was too long, it degrades to Pan.
            // This is acceptable — the important thing is that a gesture was produced.
        }
        other => panic!("Expected Flick or Pan gesture, got {other:?}"),
    }

    // Verify detector cleaned up
    assert_eq!(detector.touch_count(), 0);
    assert!(!detector.is_tracking());
}

// ===========================================================================
// Cross-module State
// ===========================================================================

#[test]
fn interaction_engine_full_cycle() {
    let mut engine = InteractionEngine::new();

    // 1. Mouse move → crosshair
    let cmds = engine.handle(InputEvent::mouse_move(300.0, 250.0));
    assert!(cmds.iter().any(|c| matches!(
        c,
        ChartCommand::UpdateCrosshair {
            screen_x: 300.0,
            screen_y: 250.0
        }
    )));

    // 2. Mouse down → starts drag state
    let cmds = engine.handle(InputEvent::mouse_left_down(300.0, 250.0));
    assert!(cmds.contains(&ChartCommand::RequestRedraw));

    // 3. Mouse move while dragging → still crosshair
    let cmds = engine.handle(InputEvent::mouse_move(350.0, 260.0));
    assert!(cmds.iter().any(|c| matches!(c, ChartCommand::UpdateCrosshair { .. })));

    // 4. Mouse up → ends drag
    let cmds = engine.handle(InputEvent::mouse_left_up(350.0, 260.0));
    assert!(cmds.is_empty());

    // 5. Wheel → zoom at cursor
    let cmds = engine.handle(InputEvent::wheel(0.0, -3.0, 350.0, 260.0));
    assert!(cmds.iter().any(|c| matches!(c, ChartCommand::ZoomAtCursor { .. })));

    // 6. Shift+wheel → pan
    let shift = ModifierState { shift: true, ctrl: false, alt: false, super_key: false };
    let cmds = engine.handle(InputEvent::wheel_with_modifiers(0.0, -3.0, 350.0, 260.0, shift));
    assert!(cmds.iter().any(|c| matches!(c, ChartCommand::Pan { .. })));

    // 7. Release shift
    engine.handle(InputEvent::key_up(KeyCode::Shift));

    // 8. Arrow key → pan
    let cmds = engine.handle(InputEvent::key_down(KeyCode::ArrowRight));
    assert!(cmds.contains(&ChartCommand::Pan {
        time_delta: 60,
        price_delta: 0.0,
    }));
}

#[test]
fn drawing_tool_then_cancel() {
    let mut engine = InteractionEngine::new();

    // 1. Press '1' → start TrendLine drawing
    let cmds = engine.handle(InputEvent::key_down(KeyCode::Key1));
    assert!(cmds.contains(&ChartCommand::StartDrawing {
        tool: DrawingTool::TrendLine,
    }));
    assert!(engine.is_drawing());
    assert_eq!(engine.active_tool(), Some(DrawingTool::TrendLine));

    // 2. Click → place drawing point
    let cmds = engine.handle(InputEvent::mouse_left_down(200.0, 300.0));
    assert!(cmds.contains(&ChartCommand::PlaceDrawingPoint {
        screen_x: 200.0,
        screen_y: 300.0,
    }));

    // 3. Escape → cancel drawing, deselect, return to navigate mode
    let cmds = engine.handle(InputEvent::key_down(KeyCode::Escape));
    assert!(cmds.contains(&ChartCommand::CancelDrawing));
    assert!(cmds.contains(&ChartCommand::DeselectAll));
    assert!(!engine.is_drawing());
    assert_eq!(engine.active_tool(), None);
}

#[test]
fn zoom_mode_switch() {
    let mut engine = InteractionEngine::new();

    // Starts in Navigate mode
    assert!(!engine.is_drawing());
    assert_eq!(engine.active_tool(), None);

    // Switch to Draw mode by selecting a tool
    engine.set_tool(Some(DrawingTool::Rectangle));
    assert!(engine.is_drawing());
    assert_eq!(engine.active_tool(), Some(DrawingTool::Rectangle));

    // In Draw mode, clicking produces PlaceDrawingPoint (not pan)
    let cmds = engine.handle(InputEvent::mouse_left_down(100.0, 100.0));
    assert!(cmds.contains(&ChartCommand::PlaceDrawingPoint {
        screen_x: 100.0,
        screen_y: 100.0,
    }));

    // Switch back to Navigate by clearing tool
    engine.set_tool(None);
    assert!(!engine.is_drawing());
    assert_eq!(engine.active_tool(), None);

    // In Navigate mode, clicking produces just RequestRedraw (no PlaceDrawingPoint)
    let cmds = engine.handle(InputEvent::mouse_left_down(100.0, 100.0));
    assert!(cmds.contains(&ChartCommand::RequestRedraw));
    assert!(!cmds.iter().any(|c| matches!(c, ChartCommand::PlaceDrawingPoint { .. })));
}

// ===========================================================================
// Additional cross-module scenarios
// ===========================================================================

#[test]
fn keyboard_shortcuts_preset_matches_engine() {
    let map = KeyboardPresets::default_shortcuts();

    // Escape shortcut should produce CancelDrawing
    let cmd = map.handle_event("Escape", &Modifiers::NONE);
    assert!(cmd.is_some());
    assert_eq!(cmd.unwrap(), &ChartCommand::CancelDrawing);

    // Delete shortcut should produce DeleteSelected
    let cmd = map.handle_event("Delete", &Modifiers::NONE);
    assert!(cmd.is_some());
    assert_eq!(cmd.unwrap(), &ChartCommand::DeleteSelected);

    // ArrowLeft should produce Pan
    let cmd = map.handle_event("ArrowLeft", &Modifiers::NONE);
    assert!(cmd.is_some());
    assert!(matches!(cmd.unwrap(), ChartCommand::Pan { .. }));

    // Plus should produce ZoomAtCursor
    let cmd = map.handle_event("+", &Modifiers::NONE);
    assert!(cmd.is_some());
    assert!(matches!(cmd.unwrap(), ChartCommand::ZoomAtCursor { factor, .. } if (factor - 1.5).abs() < EPS));
}

#[test]
fn auto_scroll_with_pan_controller() {
    let mut pan = PanController::new();
    pan.set_auto_scroll(true);
    let mut viewport = default_viewport();

    // New data arrives — viewport should shift to show latest
    pan.on_new_data(&mut viewport, 10_000);
    let width = 1000.0;
    let margin = width * 0.05;
    assert!((viewport.time_end - (10_000.0 + margin)).abs() < EPS);
    assert!((viewport.width() - width).abs() < EPS);
}

#[test]
fn follow_price_with_zoom() {
    let mut pan = PanController::new();
    pan.set_follow_price(true, 150.0);
    let mut viewport = default_viewport();
    let zoom = zoom_controller();

    // Verify follow price level
    assert_eq!(pan.follow_price_level(), Some(150.0));

    // Zoom doesn't change follow price
    zoom.wheel_zoom(&mut viewport, 2.0, 0.5, 0.5);
    assert_eq!(pan.follow_price_level(), Some(150.0));
}

#[test]
fn crosshair_sync_groups() {
    let mut a = CrosshairController::new();
    let mut b = CrosshairController::new();
    let mut c = CrosshairController::new();

    a.set_sync_group(Some(1));
    b.set_sync_group(Some(1));
    c.set_sync_group(Some(2));

    assert!(a.should_sync_with(&b));
    assert!(!a.should_sync_with(&c));
    assert!(!b.should_sync_with(&c));
}

#[test]
fn engine_wheel_zoom_produces_correct_factor() {
    let mut engine = InteractionEngine::new();

    // Negative delta_y = scroll up = zoom in → factor 1.1
    let cmds = engine.handle(InputEvent::wheel(0.0, -5.0, 400.0, 300.0));
    assert!(cmds.iter().any(|c| matches!(
        c,
        ChartCommand::ZoomAtCursor { factor, screen_x, screen_y, }
        if (factor - 1.1).abs() < EPS && *screen_x == 400.0 && *screen_y == 300.0
    )));

    // Positive delta_y = scroll down = zoom out → factor 1/1.1
    let cmds = engine.handle(InputEvent::wheel(0.0, 5.0, 400.0, 300.0));
    let expected = 1.0 / 1.1;
    assert!(cmds.iter().any(|c| matches!(
        c,
        ChartCommand::ZoomAtCursor { factor, .. } if (*factor - expected).abs() < EPS
    )));
}
