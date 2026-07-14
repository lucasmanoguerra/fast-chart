use fast_chart_core::app::chart_controller::ChartState;
use fast_chart_core::app::layout_manager::LayoutManager;
use fast_chart_core::{
    DefaultPriceFormatter, InvalidationLevel, PriceFormatter, Viewport,
};
use winit::window::CursorIcon;

use super::rendering::area_renderer::AreaRenderer;
use super::rendering::bar_renderer::BarRenderer;
use super::rendering::candle_renderer::CandleRenderer;
use super::rendering::crosshair_renderer::CrosshairRenderer;
use super::rendering::grid_renderer::GridRenderer;
use super::rendering::histogram_renderer::HistogramRenderer;
use super::rendering::line_renderer::LineRenderer;
use super::rendering::marker_renderer::MarkerRenderer;
use super::rendering::price_line_renderer::PriceLineRenderer;
use super::rendering::text_renderer::{GpuTextRenderer, PreparedTextArea};
use super::rendering::types::{colors, Uniforms, Vertex};

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
    area_renderer: AreaRenderer,
    histogram_renderer: HistogramRenderer,
    candle_renderer: CandleRenderer,
    bar_renderer: BarRenderer,
    crosshair_renderer: CrosshairRenderer,
    /// Per-pane marker renderers — one per pane, indexed by pane index.
    pane_marker_renderers: Vec<MarkerRenderer>,
    /// Per-pane price line renderers — one per pane, indexed by pane index.
    pane_price_line_renderers: Vec<PriceLineRenderer>,
    text_renderer: GpuTextRenderer,
    // Divider line renderer
    divider_pipeline: wgpu::RenderPipeline,
    divider_vertex_buffer: wgpu::Buffer,
    divider_uniform_buffer: wgpu::Buffer,
    divider_bind_group: wgpu::BindGroup,
    divider_vertex_count: u32,
    // Text buffer indices
    axis_label_pool: Vec<usize>, // pool of text buffer indices for axis labels (12 y + 12 x)
    crosshair_label_buffer: usize,
    // Crosshair state (transient UI — not chart data)
    crosshair_active: bool,
    crosshair_x: f32,
    crosshair_y: f32,
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
        let area_renderer = AreaRenderer::new(&device, surface_format);
        let histogram_renderer = HistogramRenderer::new(&device, surface_format);
        let candle_renderer = CandleRenderer::new(&device, surface_format);
        let bar_renderer = BarRenderer::new(&device, surface_format);
        let crosshair_renderer = CrosshairRenderer::new(&device, surface_format, &shader);

        // Create per-pane marker and price line renderers (initially 2 for default layout).
        let mut pane_marker_renderers = Vec::new();
        let mut pane_price_line_renderers = Vec::new();
        for _ in 0..2 {
            pane_marker_renderers.push(MarkerRenderer::new(&device, surface_format, &shader));
            pane_price_line_renderers.push(PriceLineRenderer::new(&device, surface_format, &shader));
        }

        // --- Create text renderer (glyphon) ---
        let mut text_renderer = GpuTextRenderer::new(&device, &queue, surface_format);
        text_renderer.update_resolution(&queue, size.width, size.height);
        let mut axis_label_pool = Vec::new();
        for _ in 0..24 {
            axis_label_pool.push(text_renderer.create_buffer(12.0));
        }
        let crosshair_label_buffer = text_renderer.create_buffer(12.0);

        // --- Divider line renderer (uses same shader as grid/line) ---
        let divider_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("divider pipeline layout"),
                bind_group_layouts: &[
                    &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("divider uniform bgl"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    }),
                ],
                push_constant_ranges: &[],
            });

        let divider_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("divider pipeline"),
            layout: Some(&divider_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Max 10 dividers = 20 vertices
        let divider_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("divider vertex buffer"),
            size: (20 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let divider_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("divider uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let divider_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("divider bind group"),
            layout: &divider_pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: divider_uniform_buffer.as_entire_binding(),
            }],
        });

        let renderer = Self {
            window,
            surface,
            device,
            queue,
            config,
            size: (size.width, size.height),
            shader,
            grid_renderer,
            line_renderer,
            area_renderer,
            histogram_renderer,
            candle_renderer,
            bar_renderer,
            crosshair_renderer,
            pane_marker_renderers,
            pane_price_line_renderers,
            text_renderer,
            divider_pipeline,
            divider_vertex_buffer,
            divider_uniform_buffer,
            divider_bind_group,
            divider_vertex_count: 0,
            axis_label_pool,
            crosshair_label_buffer,
            crosshair_active: false,
            crosshair_x: 0.0,
            crosshair_y: 0.0,
        };

        // SAFETY: We can borrow renderer mutably here since we just created it.
        let mut renderer = renderer;
        renderer
            .grid_renderer
            .update_grid(&renderer.queue, size.width as f32, size.height as f32);

        Ok(renderer)
    }

    /// GPU-render the chart frame.
    ///
    /// Reads all chart data from `state` (single source of truth) and renders
    /// directly to the surface. The caller is responsible for clearing the
    /// invalidation mask on `state` after this call returns.
    pub fn render(
        &mut self,
        layout: &LayoutManager,
        state: &ChartState,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Read invalidation mask (Copy) — caller clears state.invalidation later.
        let mask = state.invalidation;

        // Update line vertices if data changed (Full level).
        if mask.contains(InvalidationLevel::Full) {
            let w = self.size.0 as f32;
            let h = self.size.1 as f32;
            self.update_line_from_state(w, h, state);
            self.update_area_from_state(w, h, state);
            self.update_histogram_from_state(w, h, state);
        }

        // Update candle vertex/index buffers if data changed (Full level).
        if mask.contains(InvalidationLevel::Full) {
            self.update_candle_data(state);
            self.update_bar_data(state);
        }

        // Always update candle uniform with current viewport (zoom/pan changes viewport).
        self.update_candle_uniforms(&state.viewport);

        // Always update bar uniform with current viewport.
        self.update_bar_uniforms(&state.viewport);

        // Always update divider lines — layout may change without an
        // invalidation mask (e.g., divider drag at the App level). This is cheap.
        self.update_divider_lines(layout);

        // Update text labels (axis + crosshair) if viewport or data changed (Light or Full).
        if mask.contains(InvalidationLevel::Light) || self.crosshair_active {
            let fallback = DefaultPriceFormatter::new(None);
            let formatter: &dyn PriceFormatter = layout.panes.first()
                .map(|p| p.formatter())
                .unwrap_or(&fallback);
            self.update_axis_labels(&state.viewport, formatter);
            self.update_crosshair_labels(&state.viewport, state, formatter);
            self.text_renderer
                .prepare(&self.device, &self.queue, &self.build_text_areas(&state.viewport));
        }

        // Update crosshair lines.
        self.crosshair_renderer.update(
            &self.queue,
            self.crosshair_x,
            self.crosshair_y,
            self.size.0 as f32,
            self.size.1 as f32,
            self.crosshair_active,
        );

        // Sync per-pane renderer count with layout (pane add/remove is rare).
        let pane_count = layout.panes.len();
        while self.pane_marker_renderers.len() < pane_count {
            self.pane_marker_renderers.push(MarkerRenderer::new(
                &self.device,
                self.config.format,
                &self.shader,
            ));
        }
        while self.pane_price_line_renderers.len() < pane_count {
            self.pane_price_line_renderers.push(PriceLineRenderer::new(
                &self.device,
                self.config.format,
                &self.shader,
            ));
        }
        self.pane_marker_renderers.truncate(pane_count);
        self.pane_price_line_renderers.truncate(pane_count);

        // Update per-pane marker and price line renderers (must happen before render pass).
        for (pane_idx, pane) in layout.panes.iter().enumerate() {
            self.pane_marker_renderers[pane_idx].update(
                &self.queue,
                pane.markers().all(),
                &state.viewport,
                self.size.0 as f32,
                self.size.1 as f32,
            );
            self.pane_price_line_renderers[pane_idx].update(
                &self.queue,
                pane.price_lines().all(),
                &state.viewport,
                self.size.0 as f32,
                self.size.1 as f32,
            );
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

            // Layer 2: Candlestick series (world-space, GPU-projected)
            self.candle_renderer.render(&mut render_pass);

            // Layer 2.5: OHLC bar series (world-space, GPU-projected)
            self.bar_renderer.render(&mut render_pass);

            // Layer 3: Line series (NDC-space, CPU-projected)
            self.line_renderer.render(&mut render_pass);

            // Layer 3.5: Area fill (below line, alpha-blended)
            self.area_renderer.render(&mut render_pass);

            // Layer 3.7: Histogram bars (volume/oscillator)
            self.histogram_renderer.render(&mut render_pass);

            // Layer 4: Pane divider lines
            self.render_dividers(&mut render_pass);

            // Layer 5: Crosshair lines
            self.crosshair_renderer.render(&mut render_pass);

            // Layers 6–7: Markers + Price lines (per-pane scissor rects)
            let w = self.size.0;
            let h = self.size.1;
            for pane_idx in 0..layout.panes.len() {
                let y_offset = layout.pane_pixel_offset(pane_idx, h as f64) as u32;
                let y_height = layout.pane_pixel_height(pane_idx, h as f64) as u32;
                render_pass.set_scissor_rect(0, y_offset, w, y_height.max(1));

                self.pane_marker_renderers[pane_idx].render(&mut render_pass);
                self.pane_price_line_renderers[pane_idx].render(&mut render_pass);
            }

            // Reset scissor rect for text layer.
            render_pass.set_scissor_rect(0, 0, w, h);

            // Layer 8: Text labels (axis + crosshair)
            self.text_renderer.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    /// Generate line vertices from ChartState time series and upload to GPU.
    fn update_line_from_state(
        &mut self,
        canvas_width: f32,
        canvas_height: f32,
        state: &ChartState,
    ) {
        use super::rendering::types::Vertex;

        if state.time_series.len() < 2 {
            self.line_renderer
                .update_line_from_empty(&self.queue, canvas_width, canvas_height);
            return;
        }

        let viewport = &state.viewport;
        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        let value_range = viewport.value_max - viewport.value_min;

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

        for bar in state.time_series.iter() {
            if bar.timestamp < viewport.time_start || bar.timestamp > viewport.time_end {
                has_prev = false;
                continue;
            }

            let time_ratio =
                (bar.timestamp as f64 - viewport.time_start as f64) / time_range;
            let x_ndc = (-1.0 + 2.0 * time_ratio) as f32;

            let value_ratio = (bar.close - viewport.value_min) / value_range;
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

    /// Generate area fill vertices from ChartState and upload to GPU.
    ///
    /// The area fills from the close-price line down to `value_min` (bottom
    /// of the visible pane), using the same color as the line with alpha
    /// blending for a translucent fill.
    fn update_area_from_state(
        &mut self,
        canvas_width: f32,
        canvas_height: f32,
        state: &ChartState,
    ) {
        if state.time_series.len() < 2 {
            self.area_renderer.update_area(
                &self.queue,
                &[],
                canvas_width,
                canvas_height,
                0.0,
                1.0,
                0.0,
                1.0,
            );
            return;
        }

        let viewport = &state.viewport;
        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        let value_range = viewport.value_max - viewport.value_min;

        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            self.area_renderer.update_area(
                &self.queue,
                &[],
                canvas_width,
                canvas_height,
                0.0,
                1.0,
                0.0,
                1.0,
            );
            return;
        }

        // Collect visible bars (same filter as line renderer).
        let visible: Vec<_> = state
            .time_series
            .iter()
            .filter(|b| {
                b.timestamp >= viewport.time_start as u64
                    && b.timestamp <= viewport.time_end as u64
            })
            .copied()
            .collect();

        // Baseline = bottom of visible range so area fills to pane bottom.
        self.area_renderer
            .set_baseline(viewport.value_min);

        // Fill color: same hue as line but with transparency.
        self.area_renderer.set_colors(
            [colors::LINE_CLOSE[0], colors::LINE_CLOSE[1], colors::LINE_CLOSE[2], 0.25],
            colors::LINE_CLOSE,
        );

        self.area_renderer.update_area(
            &self.queue,
            &visible,
            canvas_width,
            canvas_height,
            viewport.time_start as f64,
            viewport.time_end as f64,
            viewport.value_min,
            viewport.value_max,
        );
    }

    /// Generate histogram bar vertices from ChartState and upload to GPU.
    ///
    /// Bars extend from baseline (default 0.0) to each bar's close value.
    /// Color is bullish when close >= baseline, bearish otherwise.
    fn update_histogram_from_state(
        &mut self,
        canvas_width: f32,
        canvas_height: f32,
        state: &ChartState,
    ) {
        if state.time_series.is_empty() {
            self.histogram_renderer.update_histogram(
                &self.queue,
                &[],
                canvas_width,
                canvas_height,
                0.0,
                1.0,
                0.0,
                1.0,
            );
            return;
        }

        let viewport = &state.viewport;
        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        let value_range = viewport.value_max - viewport.value_min;

        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            self.histogram_renderer.update_histogram(
                &self.queue,
                &[],
                canvas_width,
                canvas_height,
                0.0,
                1.0,
                0.0,
                1.0,
            );
            return;
        }

        // Collect visible bars (same filter as line/area renderers).
        let visible: Vec<_> = state
            .time_series
            .iter()
            .filter(|b| {
                b.timestamp >= viewport.time_start as u64
                    && b.timestamp <= viewport.time_end as u64
            })
            .copied()
            .collect();

        // Baseline at 0.0 (typical for volume/oscillator histograms).
        self.histogram_renderer.set_baseline(0.0);

        // Bullish/bearish color scheme.
        self.histogram_renderer.set_colors(
            colors::BULLISH,
            colors::BEARISH,
        );

        self.histogram_renderer.update_histogram(
            &self.queue,
            &visible,
            canvas_width,
            canvas_height,
            viewport.time_start as f64,
            viewport.time_end as f64,
            viewport.value_min,
            viewport.value_max,
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
            self.text_renderer
                .update_resolution(&self.queue, width, height);
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn canvas_width(&self) -> f32 {
        self.size.0 as f32
    }

    pub fn canvas_height(&self) -> f32 {
        self.size.1 as f32
    }

    /// Set the window cursor icon (e.g., RowResize when hovering a divider).
    pub fn set_cursor(&self, icon: CursorIcon) {
        self.window.set_cursor(icon);
    }

    /// Regenerate divider line vertices from the layout.
    fn update_divider_lines(&mut self, layout: &LayoutManager) {
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;

        let mut vertices: Vec<Vertex> = Vec::new();
        let divider_color = [0.4, 0.4, 0.45, 0.8]; // subtle gray divider

        for &divider_y_frac in &layout.dividers {
            // Convert normalized y to NDC: y_frac=0 is top (NDC +1), y_frac=1 is bottom (NDC -1)
            let y_ndc = 1.0 - 2.0 * divider_y_frac as f32;
            vertices.push(Vertex::new([-1.0, y_ndc], divider_color));
            vertices.push(Vertex::new([1.0, y_ndc], divider_color));
        }

        self.divider_vertex_count = vertices.len() as u32;
        if self.divider_vertex_count > 0 {
            self.queue.write_buffer(
                &self.divider_vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices),
            );
        }

        let uniforms = Uniforms::new(w, h);
        self.queue
            .write_buffer(&self.divider_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Draw divider lines in the current render pass.
    fn render_dividers<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.divider_vertex_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.divider_pipeline);
        render_pass.set_bind_group(0, &self.divider_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.divider_vertex_buffer.slice(..));
        render_pass.draw(0..self.divider_vertex_count, 0..1);
    }

    pub fn info(&self) -> String {
        format!("{}x{}", self.size.0, self.size.1)
    }

    // --- Crosshair ---

    /// Update the crosshair position in logical pixels. Call from CursorMoved.
    pub fn set_crosshair(&mut self, x: f32, y: f32, active: bool) {
        self.crosshair_x = x;
        self.crosshair_y = y;
        self.crosshair_active = active;
    }

    /// Deactivate the crosshair (e.g., when cursor leaves window).
    pub fn deactivate_crosshair(&mut self) {
        self.crosshair_active = false;
    }

    // --- ChartRenderer trait impl ---

    // --- Private helpers (read from ChartState) ---

    /// Recompute candle vertex/index data from bars and current viewport.
    fn update_candle_data(&mut self, state: &ChartState) {
        if state.time_series.is_empty() {
            return;
        }
        let visible_count = visible_bar_count(state);
        let bar_width = self.compute_bar_width(visible_count, &state.viewport);
        let visible = collect_visible_bars(state);
        self.candle_renderer
            .update_candles(&self.queue, &visible, bar_width);
    }

    /// Upload viewport uniform to the candle renderer's buffer.
    fn update_candle_uniforms(&self, viewport: &Viewport) {
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;
        self.candle_renderer.update_uniforms(
            &self.queue,
            w,
            h,
            viewport.time_start as f32,
            viewport.time_end as f32,
            viewport.value_min as f32,
            viewport.value_max as f32,
        );
    }

    /// Recompute OHLC bar vertex data from bars and current viewport.
    fn update_bar_data(&mut self, state: &ChartState) {
        if state.time_series.is_empty() {
            return;
        }
        let visible_count = visible_bar_count(state);
        let bar_width = self.compute_bar_width(visible_count, &state.viewport);
        let visible = collect_visible_bars(state);
        self.bar_renderer
            .update_bars(&self.queue, &visible, bar_width);
    }

    /// Upload viewport uniform to the bar renderer's buffer.
    fn update_bar_uniforms(&self, viewport: &Viewport) {
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;
        self.bar_renderer.update_uniforms(
            &self.queue,
            w,
            h,
            viewport.time_start as f32,
            viewport.time_end as f32,
            viewport.value_min as f32,
            viewport.value_max as f32,
        );
    }

    /// Candle body width in world units (timestamp space).
    ///
    /// Target: 70 % of the per-bar slot so candles have 30 % gap.
    fn compute_bar_width(&self, visible_count: usize, viewport: &Viewport) -> f32 {
        let count = visible_count.max(1) as f32;
        let time_range = (viewport.time_end - viewport.time_start) as f32;
        (time_range / count) * 0.7
    }

    /// Find the bar closest to the crosshair time.
    fn find_bar_at_crosshair<'a>(
        &self,
        state: &'a ChartState,
    ) -> Option<&'a fast_chart_core::Bar> {
        if !self.crosshair_active {
            return None;
        }
        let crosshair_time = screen_x_to_timestamp(
            &state.viewport,
            self.crosshair_x,
            self.size.0 as f64,
        );
        state
            .time_series
            .iter()
            .min_by_key(|b| (b.timestamp as f64 - crosshair_time).abs() as u64)
            .filter(|b| (b.timestamp as f64 - crosshair_time).abs() < 60_000.0)
    }

    // --- Text labels ---

    /// Update axis labels (price ticks on y-axis, time ticks on x-axis).
    fn update_axis_labels(&mut self, viewport: &Viewport, formatter: &dyn PriceFormatter) {
        // Clear all axis label buffers first
        for &buf_idx in &self.axis_label_pool {
            self.text_renderer.set_text(buf_idx, "");
        }

        let mut label_idx = 0;

        // Y-axis: price labels at grid lines (right edge)
        let h_count = 10;
        let price_step = (viewport.value_max - viewport.value_min) / h_count as f64;
        for i in 0..=h_count {
            let price = viewport.value_min + price_step * i as f64;
            let text = formatter.format(price);
            if label_idx < self.axis_label_pool.len() {
                self.text_renderer
                    .set_text(self.axis_label_pool[label_idx], &text);
                label_idx += 1;
            }
        }

        // X-axis: time labels at vertical grid lines (bottom edge)
        let x_tick_count = 8;
        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        let time_step = time_range / x_tick_count as f64;

        for i in 0..=x_tick_count {
            let timestamp = viewport.time_start as f64 + time_step * i as f64;
            let text = format_timestamp(timestamp as u64);
            if label_idx < self.axis_label_pool.len() {
                self.text_renderer
                    .set_text(self.axis_label_pool[label_idx], &text);
                label_idx += 1;
            }
        }
    }

    /// Update crosshair labels (price on right, time on bottom, OHLC tooltip).
    fn update_crosshair_labels(&mut self, viewport: &Viewport, state: &ChartState, formatter: &dyn PriceFormatter) {
        if !self.crosshair_active {
            self.text_renderer
                .set_text(self.crosshair_label_buffer, "");
            return;
        }

        let mut text = String::new();

        // Price at cursor Y
        let price = screen_y_to_price(viewport, self.crosshair_y, self.size.1 as f64);
        text.push_str(&format!("{}\n", formatter.format_short(price)));

        // Time at cursor X
        let timestamp = screen_x_to_timestamp(
            viewport,
            self.crosshair_x,
            self.size.0 as f64,
        );
        text.push_str(&format!("{}\n", format_timestamp(timestamp as u64)));

        // OHLC tooltip if hovering over a bar
        if let Some(bar) = self.find_bar_at_crosshair(state) {
            text.push_str(&format!(
                "O:{} H:{} L:{} C:{}",
                formatter.format_short(bar.open),
                formatter.format_short(bar.high),
                formatter.format_short(bar.low),
                formatter.format_short(bar.close),
            ));
        }

        self.text_renderer
            .set_text(self.crosshair_label_buffer, &text);
    }

    /// Build the list of text areas to render this frame.
    fn build_text_areas(&self, viewport: &Viewport) -> Vec<PreparedTextArea> {
        let mut areas = Vec::new();
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;
        let axis_color = glyphon::Color::rgba(200, 200, 200, 200);

        let h_count = 10;
        let price_step = (viewport.value_max - viewport.value_min) / h_count as f64;
        let mut label_idx = 0;

        // Y-axis: price labels at each grid line Y position (right edge)
        for i in 0..=h_count {
            if label_idx >= self.axis_label_pool.len() {
                break;
            }
            let price_y = viewport.value_min + price_step * i as f64;
            // Convert price to pixel Y (flipped: max = top = 0px, min = bottom = h px)
            let ratio = (price_y - viewport.value_min)
                / (viewport.value_max - viewport.value_min);
            let pixel_y = h - ratio as f32 * h;

            areas.push(PreparedTextArea {
                buffer_idx: self.axis_label_pool[label_idx],
                left: w - 70.0,
                top: pixel_y - 7.0, // center text vertically on the grid line
                scale: 1.0,
                right: w,
                bottom: h,
                color: axis_color,
            });
            label_idx += 1;
        }

        // X-axis: time labels at each vertical grid line X position (bottom edge)
        let x_tick_count = 8;
        let time_range = viewport.time_end as f64 - viewport.time_start as f64;
        let time_step = time_range / x_tick_count as f64;

        for i in 0..=x_tick_count {
            if label_idx >= self.axis_label_pool.len() {
                break;
            }
            let timestamp = viewport.time_start as f64 + time_step * i as f64;
            // Convert timestamp to pixel X
            let time_ratio = (timestamp - viewport.time_start as f64) / time_range;
            let pixel_x = time_ratio as f32 * w;

            areas.push(PreparedTextArea {
                buffer_idx: self.axis_label_pool[label_idx],
                left: pixel_x - 30.0, // center text horizontally on the grid line
                top: h - 20.0,        // bottom edge, 20px up
                scale: 1.0,
                right: pixel_x + 40.0,
                bottom: h,
                color: axis_color,
            });
            label_idx += 1;
        }

        // Crosshair labels (keep existing single-buffer approach)
        if self.crosshair_active {
            let ch_color = glyphon::Color::rgba(255, 255, 0, 255);
            areas.push(PreparedTextArea {
                buffer_idx: self.crosshair_label_buffer,
                left: (self.crosshair_x + 10.0).min(w - 100.0),
                top: (self.crosshair_y - 30.0).max(0.0),
                scale: 1.0,
                right: w,
                bottom: h,
                color: ch_color,
            });
        }

        areas
    }
}

impl fast_chart_core::ports::render::ChartRenderer for GpuRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        GpuRenderer::resize(self, width, height);
    }
}

// ---------------------------------------------------------------------------
// Free functions — pure computations over ChartState, no &self needed.
// ---------------------------------------------------------------------------

/// Number of bars whose timestamps fall inside the viewport.
fn visible_bar_count(state: &ChartState) -> usize {
    state
        .time_series
        .iter()
        .filter(|b| state.viewport.contains_time(b.timestamp))
        .count()
}

/// Collect bars visible in the current viewport.
fn collect_visible_bars(state: &ChartState) -> Vec<fast_chart_core::Bar> {
    state
        .time_series
        .iter()
        .filter(|b| state.viewport.contains_time(b.timestamp))
        .copied()
        .collect()
}

/// Convert a screen x coordinate to a timestamp in the current viewport.
fn screen_x_to_timestamp(viewport: &Viewport, screen_x: f32, canvas_width: f64) -> f64 {
    if canvas_width < 1.0 {
        return viewport.time_start as f64;
    }
    let ratio = (screen_x as f64 / canvas_width).clamp(0.0, 1.0);
    viewport.time_start as f64
        + ratio * (viewport.time_end as f64 - viewport.time_start as f64)
}

/// Convert a screen y coordinate to a price in the current viewport.
fn screen_y_to_price(viewport: &Viewport, screen_y: f32, canvas_height: f64) -> f64 {
    if canvas_height < 1.0 {
        return viewport.value_min;
    }
    // Y is flipped: screen 0 = top = max price, screen h = bottom = min price
    let ratio = (screen_y as f64 / canvas_height).clamp(0.0, 1.0);
    viewport.value_max - ratio * (viewport.value_max - viewport.value_min)
}

/// Format a timestamp (milliseconds since epoch) to a short time string.
fn format_timestamp(ts: u64) -> String {
    let secs = ts / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    format!(
        "{:02}:{:02}:{:02}",
        hours % 24,
        mins % 60,
        secs % 60
    )
}
