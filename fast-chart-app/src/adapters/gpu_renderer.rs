use fast_chart_domain::bar::Bar;
use fast_chart_domain::viewport::Viewport;

use super::rendering::grid_renderer::GridRenderer;
use super::rendering::line_renderer::LineRenderer;
use super::rendering::types::colors;

pub struct GpuRenderer {
    window: winit::window::Window,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    // Rendering sub-renderers
    /// Kept alive — pipelines reference this module. Sub-renderers receive it at init.
    #[allow(dead_code)]
    shader: wgpu::ShaderModule,
    grid_renderer: GridRenderer,
    line_renderer: LineRenderer,
    // Chart state (owned by renderer for simplicity; will move to ChartController later)
    bars: Vec<Bar>,
    viewport: Viewport,
    needs_line_update: bool,
}

impl GpuRenderer {
    pub async fn new(window: winit::window::Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // SAFETY: The window is stored in this struct and outlives the surface.
        // wgpu only extracts raw window/display handles — it does not hold a
        // reference to the Window. Surface<'static> is sound as long as the
        // Window (which owns the native window) is dropped after the surface.
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window)?)
        }?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable GPU adapter found")?;

        log::info!("GPU adapter: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("fast-chart device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // --- Load WGSL shader ---
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line/grid shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("rendering/shaders/line.wgsl").into(),
            ),
        });

        // --- Create sub-renderers ---
        let grid_renderer = GridRenderer::new(&device, surface_format, &shader);
        let line_renderer = LineRenderer::new(&device, surface_format, &shader);

        let mut renderer = Self {
            window,
            surface,
            device,
            queue,
            config,
            size: (size.width, size.height),
            shader,
            grid_renderer,
            line_renderer,
            bars: Vec::new(),
            viewport: Viewport::default(),
            needs_line_update: false,
        };

        // Initial grid generation.
        renderer
            .grid_renderer
            .update_grid(&renderer.queue, size.width as f32, size.height as f32);

        Ok(renderer)
    }

    /// Accept new bars from the data provider and mark the line layer dirty.
    pub fn push_bars(&mut self, new_bars: Vec<Bar>) {
        if new_bars.is_empty() {
            return;
        }
        self.bars.extend(new_bars);
        self.needs_line_update = true;

        // Auto-fit viewport on first data arrival.
        if self.viewport.time_start == 0 && self.viewport.time_end == 3600_000 {
            self.auto_fit_viewport();
        }
    }

    /// Fit the viewport to the loaded data range with some padding.
    fn auto_fit_viewport(&mut self) {
        if self.bars.is_empty() {
            return;
        }

        let first = &self.bars[0];
        let last = self.bars.last().unwrap();

        let time_pad = ((last.timestamp - first.timestamp) / 20).max(1);
        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;
        for bar in &self.bars {
            if bar.low < min_price {
                min_price = bar.low;
            }
            if bar.high > max_price {
                max_price = bar.high;
            }
        }
        let price_pad = ((max_price - min_price) / 10.0).max(0.01);

        self.viewport.time_start = first.timestamp.saturating_sub(time_pad);
        self.viewport.time_end = last.timestamp.saturating_add(time_pad);
        self.viewport.value_min = (min_price - price_pad).max(0.0);
        self.viewport.value_max = max_price + price_pad;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Update line vertices if data changed.
        if self.needs_line_update {
            let w = self.size.0 as f32;
            let h = self.size.1 as f32;
            // Create a temporary TimeSeries-like view from the Vec<Bar>.
            // We use a small trick: since TimeSeries is generic over N,
            // and we own a Vec<Bar>, we pass the viewport and let the
            // line_renderer handle empty series gracefully.
            // For now, we generate NDC vertices directly from the Vec.
            self.update_line_from_vec(w, h);
            self.needs_line_update = false;
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("chart render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(colors::BACKGROUND),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // Layer 1: Grid (low alpha, behind data)
            self.grid_renderer.render(&mut render_pass);

            // Layer 2: Line series (full alpha, on top)
            self.line_renderer.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    /// Generate line vertices from the internal Vec<Bar> and upload to GPU.
    ///
    /// This is a temporary bridge until the ChartController owns the data
    /// and passes it directly to the renderer.
    fn update_line_from_vec(&mut self, canvas_width: f32, canvas_height: f32) {
        use super::rendering::types::Vertex;

        if self.bars.len() < 2 {
            // Upload empty buffer — line_renderer handles this gracefully.
            self.line_renderer
                .update_line_from_empty(&self.queue, canvas_width, canvas_height);
            return;
        }

        let time_range = self.viewport.time_end as f64 - self.viewport.time_start as f64;
        let value_range = self.viewport.value_max - self.viewport.value_min;

        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            self.line_renderer
                .update_line_from_empty(&self.queue, canvas_width, canvas_height);
            return;
        }

        let line_color = colors::LINE_CLOSE;
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut prev_x: f32 = 0.0;
        let mut prev_y: f32 = 0.0;
        let mut has_prev = false;

        for bar in &self.bars {
            if bar.timestamp < self.viewport.time_start
                || bar.timestamp > self.viewport.time_end
            {
                has_prev = false;
                continue;
            }

            let time_ratio =
                (bar.timestamp as f64 - self.viewport.time_start as f64) / time_range;
            let x_ndc = (-1.0 + 2.0 * time_ratio) as f32;

            let value_ratio = (bar.close - self.viewport.value_min) / value_range;
            let y_ndc = (-1.0 + 2.0 * value_ratio) as f32;

            if has_prev {
                vertices.push(Vertex::new([prev_x, prev_y], line_color));
                vertices.push(Vertex::new([x_ndc, y_ndc], line_color));
            }

            prev_x = x_ndc;
            prev_y = y_ndc;
            has_prev = true;
        }

        self.line_renderer.upload_vertices(
            &self.queue,
            &vertices,
            canvas_width,
            canvas_height,
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            // Regenerate grid for new dimensions.
            self.grid_renderer
                .update_grid(&self.queue, width as f32, height as f32);
            self.needs_line_update = true;
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn info(&self) -> String {
        format!("{}x{} ({} bars)", self.size.0, self.size.1, self.bars.len())
    }
}
