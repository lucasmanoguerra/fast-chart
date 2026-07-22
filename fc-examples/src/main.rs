use fc_app::app::chart_controller::ChartController;
use fc_app::FrameCounter;
use fc_app::app::layout_manager::LayoutManager;
use fc_app::ports::data_provider::DataProvider;
use fc_app::ports::interaction::InteractionCommand;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorIcon, WindowAttributes, WindowId};

mod adapters;
mod config;
mod examples;

use adapters::data::simulated::SimulatedDataProvider;
use adapters::gpu_renderer::GpuRenderer;
use fc_render::coordinates::CoordinatePipeline;
use adapters::input::handler::WinitInteractionHandler;
use config::{ChartConfig, ConfigWatcher};

struct App {
    /// Shared GPU renderer.
    gpu: Option<Arc<Mutex<GpuRenderer>>>,
    /// Central chart logic: viewport, crosshair, kinetic scroll, data polling.
    chart_controller: Option<ChartController>,
    /// Layout manager for multi-pane vertical stack.
    layout: LayoutManager,
    /// Whether the left mouse button is held for panning.
    is_panning: bool,
    /// Last cursor x during a pan drag (logical pixels) — for delta computation.
    last_pan_x: f64,
    /// Last cursor y during a divider drag (logical pixels).
    last_cursor_y: f64,
    /// Current cursor x (logical pixels) — used for zoom center.
    cursor_x: f64,
    /// Current cursor y (logical pixels) — used for divider hit test.
    cursor_y: f64,
    /// Whether we are currently dragging a pane divider.
    is_dragging_divider: bool,
    /// Application configuration loaded from TOML.
    config: ChartConfig,
    /// Watches the config file for hot-reload.
    config_watcher: Option<ConfigWatcher>,
    /// FPS counter for performance monitoring.
    frame_counter: FrameCounter,
}

impl App {
    fn new() -> Self {
        let config_path = PathBuf::from("chart.toml");
        let config = ChartConfig::ensure_config(&config_path);
        let config_watcher = ConfigWatcher::new(&config_path).ok();

        Self {
            gpu: None,
            chart_controller: None,
            layout: LayoutManager::new(),
            is_panning: false,
            last_pan_x: 0.0,
            last_cursor_y: 0.0,
            cursor_x: 0.0,
            cursor_y: 0.0,
            is_dragging_divider: false,
            config,
            config_watcher,
            frame_counter: FrameCounter::new(),
        }
    }

    /// Build a coordinate pipeline from viewport + canvas dimensions.
    fn viewport_pipeline(
        viewport: &fc_domain::viewport::Viewport,
        canvas_width: f64,
    ) -> CoordinatePipeline {
        CoordinatePipeline::new(
            (viewport.time_start as f64, viewport.time_end as f64),
            (viewport.value_min, viewport.value_max),
            0.0,
            0.0,
            canvas_width as f32,
            0.0,
            1.0,
        )
    }

    /// Sync the layout's time range from the ChartController's viewport.
    fn sync_layout_time_range(&mut self) {
        if let Some(ctrl) = &self.chart_controller {
            let vp = ctrl.state().viewport.clone();
            self.layout.sync_time_range(vp.time_start, vp.time_end);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title(&self.config.window.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.window.width as f64,
                self.config.window.height as f64,
            ))
            .with_min_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let window = event_loop.create_window(attrs).unwrap();
        let gpu_renderer =
            pollster::block_on(GpuRenderer::new(window)).unwrap();
        log::info!("wgpu initialized: {}", gpu_renderer.info());

        let gpu = Arc::new(Mutex::new(gpu_renderer));

        // Create data provider and wire it into ChartController
        let mut provider = SimulatedDataProvider::new("BTC/USDT", 50000.0, 250.0);
        provider.start().unwrap();
        log::info!("Data provider started: {}", provider.name());

        let chart_controller = ChartController::new(
            Box::new(provider),
            Box::new(WinitInteractionHandler::new()),
        );
        self.gpu = Some(gpu);
        self.chart_controller = Some(chart_controller);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // Stop the data provider via ChartController
                if let Some(ctrl) = &mut self.chart_controller {
                    let _ = ctrl.stop_data_provider();
                }
                log::info!("Close requested, shutting down.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Render the frame: pass ChartState (single source of truth) to the
                // GPU renderer, which reads data directly from it.
                if let Some(ctrl) = &mut self.chart_controller {
                    if let Some(gpu) = &self.gpu {
                        if let Ok(mut r) = gpu.lock() {
                            r.render(&self.layout, ctrl.state()).unwrap_or_else(|e| {
                                log::error!("Render error: {:?}", e);
                            });
                        }
                        // Clear invalidation after the renderer has consumed it.
                        ctrl.state_mut().consume_invalidation();
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &self.gpu {
                    if let Ok(mut r) = gpu.lock() {
                        r.resize(size.width, size.height);
                        r.request_redraw();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_x = position.x;
                self.cursor_y = position.y;

                if self.is_dragging_divider {
                    // Pane divider drag — App-level layout logic (not in ChartController)
                    let delta_y = position.y - self.last_cursor_y;
                    self.last_cursor_y = position.y;
                    let canvas_height = self
                        .gpu
                        .as_ref()
                        .and_then(|g| g.lock().ok())
                        .map(|r| r.canvas_height() as f64)
                        .unwrap_or(700.0);
                    self.layout.update_drag(delta_y, canvas_height);
                    if let Some(gpu) = &self.gpu {
                        if let Ok(r) = gpu.lock() {
                            r.request_redraw();
                        }
                    }
                } else if self.is_panning {
                    // Pan: delegate to ChartController
                    if let Some(ctrl) = &mut self.chart_controller {
                        let dx = position.x - self.last_pan_x;
                        self.last_pan_x = position.x;

                        let canvas_width = self
                            .gpu
                            .as_ref()
                            .and_then(|g| g.lock().ok())
                            .map(|r| r.canvas_width() as f64)
                            .unwrap_or(800.0);
                        let vp = &ctrl.state().viewport;
                        let time_range = vp.time_end as f64 - vp.time_start as f64;
                        let time_delta = -(dx / canvas_width * time_range) as i64;

                        ctrl.handle_input(InteractionCommand::PanBy { time_delta });
                    }
                    // Sync layout time range from controller viewport
                    self.sync_layout_time_range();
                    if let Some(gpu) = &self.gpu {
                        if let Ok(r) = gpu.lock() {
                            r.request_redraw();
                        }
                    }
                } else {
                    // Hover: update crosshair on both controller and renderer
                    if let Some(ctrl) = &mut self.chart_controller {
                        ctrl.handle_input(InteractionCommand::UpdateCrosshair {
                            screen_x: position.x,
                            screen_y: position.y,
                        });
                    }
                    if let Some(gpu) = &self.gpu {
                        if let Ok(mut r) = gpu.lock() {
                            r.set_crosshair(
                                position.x as f32,
                                position.y as f32,
                                true,
                            );
                            r.request_redraw();
                        }
                    }

                    // Check if cursor is near a divider
                    let canvas_height = self
                        .gpu
                        .as_ref()
                        .and_then(|g| g.lock().ok())
                        .map(|r| r.canvas_height() as f64)
                        .unwrap_or(700.0);
                    let cursor =
                        if self.layout.hit_test_divider(position.y, canvas_height).is_some() {
                            CursorIcon::RowResize
                        } else {
                            CursorIcon::Default
                        };
                    if let Some(gpu) = &self.gpu {
                        if let Ok(r) = gpu.lock() {
                            r.set_cursor(cursor);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        // Stop kinetic scroll on any mouse press
                        if let Some(ctrl) = &mut self.chart_controller {
                            ctrl.stop_kinetic();
                        }

                        let canvas_height = self
                            .gpu
                            .as_ref()
                            .and_then(|g| g.lock().ok())
                            .map(|r| r.canvas_height() as f64)
                            .unwrap_or(700.0);
                        if let Some(divider_idx) =
                            self.layout.hit_test_divider(self.cursor_y, canvas_height)
                        {
                            // Start dragging divider
                            self.layout.start_drag(divider_idx);
                            self.is_dragging_divider = true;
                            self.last_cursor_y = self.cursor_y;
                            // Deactivate crosshair during divider drag
                            if let Some(ctrl) = &mut self.chart_controller {
                                ctrl.handle_input(InteractionCommand::DeactivateCrosshair);
                            }
                            if let Some(gpu) = &self.gpu {
                                if let Ok(mut r) = gpu.lock() {
                                    r.deactivate_crosshair();
                                    r.request_redraw();
                                }
                            }
                        } else {
                            self.is_panning = true;
                            self.last_pan_x = self.cursor_x;
                            // Deactivate crosshair during pan
                            if let Some(ctrl) = &mut self.chart_controller {
                                ctrl.handle_input(InteractionCommand::DeactivateCrosshair);
                            }
                            if let Some(gpu) = &self.gpu {
                                if let Ok(mut r) = gpu.lock() {
                                    r.deactivate_crosshair();
                                    r.request_redraw();
                                }
                            }
                        }
                    }
                    ElementState::Released => {
                        if self.is_dragging_divider {
                            self.layout.end_drag();
                            self.is_dragging_divider = false;
                        } else {
                            self.is_panning = false;
                        }
                        // Re-activate crosshair at current position
                        if let Some(ctrl) = &mut self.chart_controller {
                            ctrl.handle_input(InteractionCommand::UpdateCrosshair {
                                screen_x: self.cursor_x,
                                screen_y: self.cursor_y,
                            });
                        }
                        if let Some(gpu) = &self.gpu {
                            if let Ok(mut r) = gpu.lock() {
                                r.set_crosshair(
                                    self.cursor_x as f32,
                                    self.cursor_y as f32,
                                    true,
                                );
                                r.request_redraw();
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, dy),
                ..
            } => {
                // Delegate zoom to ChartController
                if let Some(ctrl) = &mut self.chart_controller {
                    let factor = if dy > 0.0 { 1.1 } else { 0.9 };
                    // Convert screen_x to timestamp using controller's viewport
                    let vp = &ctrl.state().viewport;
                    let canvas_width = self
                        .gpu
                        .as_ref()
                        .and_then(|g| g.lock().ok())
                        .map(|r| r.canvas_width() as f64)
                        .unwrap_or(800.0);
                    let pipeline = Self::viewport_pipeline(vp, canvas_width);
                    let timestamp = pipeline.x_to_timestamp(self.cursor_x as f32);
                    // Pass timestamp as screen_x — viewport.zoom expects a timestamp center.
                    // The InteractionCommand parameter name is misleading but functionally correct.
                    ctrl.handle_input(InteractionCommand::ZoomAtCursor {
                        factor,
                        screen_x: timestamp,
                    });
                }
                // Sync layout time range from controller viewport
                self.sync_layout_time_range();
                if let Some(gpu) = &self.gpu {
                    if let Ok(r) = gpu.lock() {
                        r.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed => {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        let key_str = match key_code {
                            KeyCode::Digit1 => "1",
                            KeyCode::Digit5 => "5",
                            KeyCode::Digit6 => "6",
                            KeyCode::KeyD => "d",
                            _ => "",
                        };
                        if !key_str.is_empty() {
                            if let Some(cmd) = WinitInteractionHandler::handle_key(key_str) {
                                log::info!("Keyboard shortcut: {:?}", cmd);
                                if let Some(ctrl) = &mut self.chart_controller {
                                    ctrl.handle_input(cmd);
                                }
                            }
                        }
                    }
                }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Hot-reload config if the file changed.
        if let Some(watcher) = &self.config_watcher {
            if let Some(new_config) = watcher.check_reload() {
                log::info!("Config reloaded successfully");
                self.config = new_config;
            }
        }

        // Tick: poll data, update auto-fit. The renderer no longer syncs state
        // here — it reads ChartState directly in RedrawRequested.
        if let Some(ctrl) = &mut self.chart_controller {
            ctrl.tick();
        }

        // Sync layout time range from controller viewport
        self.sync_layout_time_range();

        // Update kinetic scroll each frame (managed by ChartController)
        if let Some(ctrl) = &mut self.chart_controller {
            let kinetic_moved = ctrl.update_kinetic();
            if kinetic_moved {
                self.sync_layout_time_range();
            }
        }

        if let Some(fps) = self.frame_counter.tick() {
            log::info!("FPS: {:.1}", fps);
        }

        if let Some(gpu) = &self.gpu {
            if let Ok(r) = gpu.lock() {
                r.request_redraw();
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::info!("fast-chart starting...");

    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
