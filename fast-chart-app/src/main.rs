use fast_chart_core::ports::data_provider::{DataEvent, DataProvider};
use fast_chart_domain::bar::Bar;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowAttributes, WindowId};

mod adapters;

use adapters::data::simulated::SimulatedDataProvider;

struct App {
    renderer: Option<adapters::gpu_renderer::GpuRenderer>,
    data_provider: Option<SimulatedDataProvider>,
}

impl App {
    fn new() -> Self {
        Self {
            renderer: None,
            data_provider: None,
        }
    }

    fn poll_data(&mut self) {
        if let Some(provider) = &self.data_provider {
            if let Some(rx) = provider.receiver() {
                let mut new_bars: Vec<Bar> = Vec::new();
                while let Ok(event) = rx.try_recv() {
                    match event {
                        DataEvent::BarClosed(bar) => {
                            log::debug!(
                                "Bar received: {} @ {:.2}",
                                bar.timestamp,
                                bar.close,
                            );
                            new_bars.push(bar);
                        }
                        _ => {}
                    }
                }
                if !new_bars.is_empty() {
                    if let Some(renderer) = &mut self.renderer {
                        renderer.push_bars(new_bars);
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("FastChart")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let window = event_loop.create_window(attrs).unwrap();
        let renderer =
            pollster::block_on(adapters::gpu_renderer::GpuRenderer::new(window)).unwrap();
        log::info!("wgpu initialized: {}", renderer.info());

        self.renderer = Some(renderer);

        // Start simulated data provider — generates 1-min OHLC bars every 500ms
        let mut provider = SimulatedDataProvider::new("BTC/USDT", 50000.0, 250.0);
        provider.start().unwrap();
        log::info!("Data provider started: {}", provider.name());
        self.data_provider = Some(provider);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(mut provider) = self.data_provider.take() {
                    let _ = provider.stop();
                }
                log::info!("Close requested, shutting down.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render().unwrap_or_else(|e| {
                        log::error!("Render error: {}", e);
                    });
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    renderer.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.poll_data();
        if let Some(renderer) = &self.renderer {
            renderer.request_redraw();
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
