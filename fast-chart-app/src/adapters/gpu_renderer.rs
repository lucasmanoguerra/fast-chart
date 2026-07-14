use fast_chart_core::app::layout_manager::LayoutManager;
use fast_chart_domain::bar::Bar;
use fast_chart_domain::invalidation::{InvalidationLevel, InvalidationMask};
use fast_chart_domain::marker::MarkerSet;
use fast_chart_domain::price_line::PriceLineSet;
use fast_chart_domain::price_scale::{DefaultPriceFormatter, PriceFormatter, PriceScale};
use fast_chart_domain::viewport::Viewport;
use winit::window::CursorIcon;

use super::rendering::candle_renderer::CandleRenderer;
use super::rendering::crosshair_renderer::CrosshairRenderer;
use super::rendering::grid_renderer::GridRenderer;
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
    candle_renderer: CandleRenderer,
    crosshair_renderer: CrosshairRenderer,
    marker_renderer: MarkerRenderer,
    price_line_renderer: PriceLineRenderer,
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
    // Crosshair state
    crosshair_active: bool,
    crosshair_x: f32,
    crosshair_y: f32,
    // Chart state (owned by renderer for simplicity; will move to ChartController later)
    bars: Vec<Bar>,
    viewport: Viewport,
    invalidation: InvalidationMask,
    // Formatter for price axis labels
    formatter: Box<dyn PriceFormatter>,
    // Markers and price lines for future rendering
    markers: MarkerSet,
    price_lines: PriceLineSet,
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
        let candle_renderer = CandleRenderer::new(&device, surface_format);
        let crosshair_renderer = CrosshairRenderer::new(&device, surface_format, &shader);
        let marker_renderer = MarkerRenderer::new(&device, surface_format, &shader);
        let price_line_renderer = PriceLineRenderer::new(&device, surface_format, &shader);

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
            candle_renderer,
            crosshair_renderer,
            marker_renderer,
            price_line_renderer,
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
            bars: Vec::new(),
            viewport: Viewport::default(),
            invalidation: {
                let mut m = InvalidationMask::single_pane(InvalidationLevel::Full, 0);
                m.merge(InvalidationMask::single_pane(InvalidationLevel::Light, 0));
                m
            },
            formatter: Box::new(DefaultPriceFormatter::new(None)),
            markers: MarkerSet::new(),
            price_lines: PriceLineSet::new(),
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
        self.invalidation
            .merge(InvalidationMask::all_panes(InvalidationLevel::Full));

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

    pub fn render(&mut self, layout: &LayoutManager) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mask = self.invalidation;
        self.invalidation.clear();

        // Update line vertices if data changed (Full level).
        if mask.contains(InvalidationLevel::Full) {
            let w = self.size.0 as f32;
            let h = self.size.1 as f32;
            self.update_line_from_vec(w, h);
        }

        // Update candle vertex/index buffers if data changed (Full level).
        if mask.contains(InvalidationLevel::Full) {
            self.update_candle_data();
        }

        // Always update candle uniform with current viewport (zoom/pan changes viewport).
        self.update_candle_uniforms();

        // Update divider lines if layout changed (Full level).
        if mask.contains(InvalidationLevel::Full) {
            self.update_divider_lines(layout);
        }

        // Update text labels (axis + crosshair) if viewport or data changed (Light or Full).
        if mask.contains(InvalidationLevel::Light) || self.crosshair_active {
            self.update_axis_labels();
            self.update_crosshair_labels();
            self.text_renderer
                .prepare(&self.device, &self.queue, &self.build_text_areas());
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

        // Update markers for the current viewport.
        self.marker_renderer.update(
            &self.queue,
            self.markers.all(),
            &self.viewport,
            self.size.0 as f32,
            self.size.1 as f32,
        );

        // Update price lines for the current viewport.
        self.price_line_renderer.update(
            &self.queue,
            self.price_lines.all(),
            &self.viewport,
            self.size.0 as f32,
            self.size.1 as f32,
        );

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

            // Layer 3: Line series (NDC-space, CPU-projected)
            self.line_renderer.render(&mut render_pass);

            // Layer 4: Pane divider lines
            self.render_dividers(&mut render_pass);

            // Layer 5: Crosshair lines
            self.crosshair_renderer.render(&mut render_pass);

            // Layer 6: Markers (above data, below text)
            self.marker_renderer.render(&mut render_pass);

            // Layer 7: Price lines (above markers, below text)
            self.price_line_renderer.render(&mut render_pass);

            // Layer 8: Text labels (axis + crosshair)
            self.text_renderer.render(&mut render_pass);
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
            self.text_renderer
                .update_resolution(&self.queue, width, height);
            self.invalidation
                .merge(InvalidationMask::all_panes(InvalidationLevel::Full));
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

    /// Get the price formatter used by this renderer.
    pub fn formatter(&self) -> &dyn PriceFormatter {
        self.formatter.as_ref()
    }

    /// Get the marker set for this renderer.
    pub fn markers(&self) -> &MarkerSet {
        &self.markers
    }

    /// Get a mutable reference to the marker set for this renderer.
    pub fn markers_mut(&mut self) -> &mut MarkerSet {
        &mut self.markers
    }

    /// Get the price line set for this renderer.
    pub fn price_lines(&self) -> &PriceLineSet {
        &self.price_lines
    }

    /// Get a mutable reference to the price line set for this renderer.
    pub fn price_lines_mut(&mut self) -> &mut PriceLineSet {
        &mut self.price_lines
    }

    /// Set the window cursor icon (e.g., RowResize when hovering a divider).
    pub fn set_cursor(&self, icon: CursorIcon) {
        self.window.set_cursor(icon);
    }

    /// Sync the renderer's internal state with a new layout configuration.
    ///
    /// Marks dividers and line data as needing re-render.
    pub fn sync_layout(&mut self, _layout: &LayoutManager) {
        self.invalidation
            .merge(InvalidationMask::all_panes(InvalidationLevel::Full));
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

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Signal that the viewport changed (zoom/pan) — triggers candle vertex re-upload.
    pub fn mark_candles_dirty(&mut self) {
        self.invalidation
            .merge(InvalidationMask::all_panes(InvalidationLevel::Light));
    }

    // --- ChartRenderer trait impl ---

    /// Sync state from `ChartState` into the renderer's internal fields.
    ///
    /// Called by `ChartController::tick()` through the `ChartRenderer` trait.
    /// After this call the renderer's viewport, bars, and invalidation mask
    /// are up-to-date. The actual GPU render still happens via
    /// [`render_frame`](Self::render_frame) which is triggered by
    /// `RedrawRequested`.
    fn sync_state_from_chart(&mut self, state: &fast_chart_core::app::chart_controller::ChartState) {
        // Sync viewport
        self.viewport = state.viewport.clone();

        // Sync bars from time_series (full replace — avoids incremental diffing)
        self.bars.clear();
        for bar in state.time_series.iter() {
            self.bars.push(*bar);
        }

        // Mark as dirty so the next render_frame picks up the changes
        self.invalidation
            .merge(InvalidationMask::all_panes(InvalidationLevel::Full));
    }
}

impl fast_chart_core::ports::render::ChartRenderer for GpuRenderer {
    fn render(
        &mut self,
        state: &fast_chart_core::app::chart_controller::ChartState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.sync_state_from_chart(state);
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        // Delegate to the inherent method (same logic, avoids ambiguity).
        GpuRenderer::resize(self, width, height);
    }
}

impl GpuRenderer {
    /// Recompute candle vertex/index data from bars and current viewport.
    fn update_candle_data(&mut self) {
        if self.bars.is_empty() {
            return;
        }
        let visible_bars = self.visible_bar_count();
        let bar_width = self.compute_bar_width(visible_bars);
        let visible = self.visible_bars();
        self.candle_renderer
            .update_candles(&self.queue, &visible, bar_width);
    }

    /// Upload viewport uniform to the candle renderer's buffer.
    fn update_candle_uniforms(&self) {
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;
        self.candle_renderer.update_uniforms(
            &self.queue,
            w,
            h,
            self.viewport.time_start as f32,
            self.viewport.time_end as f32,
            self.viewport.value_min as f32,
            self.viewport.value_max as f32,
        );
    }

    /// Number of bars whose timestamps fall inside the current viewport.
    fn visible_bar_count(&self) -> usize {
        self.bars
            .iter()
            .filter(|b| self.viewport.contains_time(b.timestamp))
            .count()
    }

    /// Collect bars visible in the current viewport.
    fn visible_bars(&self) -> Vec<Bar> {
        self.bars
            .iter()
            .filter(|b| self.viewport.contains_time(b.timestamp))
            .copied()
            .collect()
    }

    /// Candle body width in world units (timestamp space).
    ///
    /// Target: 70 % of the per-bar slot so candles have 30 % gap.
    fn compute_bar_width(&self, visible_count: usize) -> f32 {
        let count = visible_count.max(1) as f32;
        let time_range = (self.viewport.time_end - self.viewport.time_start) as f32;
        (time_range / count) * 0.7
    }

    pub fn info(&self) -> String {
        format!("{}x{} ({} bars)", self.size.0, self.size.1, self.bars.len())
    }

    // --- Crosshair ---

    /// Update the crosshair position in logical pixels. Call from CursorMoved.
    pub fn set_crosshair(&mut self, x: f32, y: f32, active: bool) {
        self.crosshair_x = x;
        self.crosshair_y = y;
        self.crosshair_active = active;
        self.invalidation
            .merge(InvalidationMask::all_panes(InvalidationLevel::Cursor));
    }

    /// Deactivate the crosshair (e.g., when cursor leaves window).
    pub fn deactivate_crosshair(&mut self) {
        self.crosshair_active = false;
        self.invalidation
            .merge(InvalidationMask::all_panes(InvalidationLevel::Cursor));
    }

    /// Find the bar closest to the crosshair time.
    fn find_bar_at_crosshair(&self) -> Option<&Bar> {
        if !self.crosshair_active {
            return None;
        }
        let crosshair_time = self.screen_x_to_timestamp(self.crosshair_x);
        self.bars
            .iter()
            .min_by_key(|b| {
                (b.timestamp as f64 - crosshair_time).abs() as u64
            })
            .filter(|b| {
                (b.timestamp as f64 - crosshair_time).abs() < 60_000.0
            })
    }

    /// Convert a screen x coordinate to a timestamp in the current viewport.
    fn screen_x_to_timestamp(&self, screen_x: f32) -> f64 {
        let w = self.size.0 as f64;
        if w < 1.0 {
            return self.viewport.time_start as f64;
        }
        let ratio = (screen_x as f64 / w).clamp(0.0, 1.0);
        self.viewport.time_start as f64
            + ratio * (self.viewport.time_end as f64 - self.viewport.time_start as f64)
    }

    /// Convert a screen y coordinate to a price in the current viewport.
    fn screen_y_to_price(&self, screen_y: f32) -> f64 {
        let h = self.size.1 as f64;
        if h < 1.0 {
            return self.viewport.value_min;
        }
        // Y is flipped: screen 0 = top = max price, screen h = bottom = min price
        let ratio = (screen_y as f64 / h).clamp(0.0, 1.0);
        self.viewport.value_max - ratio * (self.viewport.value_max - self.viewport.value_min)
    }

    /// Convert a screen y coordinate to a price using the given price scale.
    fn screen_y_to_price_with_scale(&self, screen_y: f32, scale: &PriceScale) -> f64 {
        self.viewport.y_to_price(screen_y, scale, self.size.1 as f32)
    }

    /// Convert a price to screen y using the given price scale.
    fn price_to_screen_y_with_scale(&self, price: f64, scale: &PriceScale) -> f32 {
        self.viewport.price_to_y(price, scale, self.size.1 as f32)
    }

    // --- Text labels ---

    /// Update axis labels (price ticks on y-axis, time ticks on x-axis).
    fn update_axis_labels(&mut self) {
        // Clear all axis label buffers first
        for &buf_idx in &self.axis_label_pool {
            self.text_renderer.set_text(buf_idx, "");
        }

        let mut label_idx = 0;

        // Y-axis: price labels at grid lines (right edge)
        let h_count = 10;
        let price_step = (self.viewport.value_max - self.viewport.value_min) / h_count as f64;
        for i in 0..=h_count {
            let price = self.viewport.value_min + price_step * i as f64;
            let text = self.formatter.format(price);
            if label_idx < self.axis_label_pool.len() {
                self.text_renderer
                    .set_text(self.axis_label_pool[label_idx], &text);
                label_idx += 1;
            }
        }

        // X-axis: time labels at vertical grid lines (bottom edge)
        let x_tick_count = 8;
        let time_range = self.viewport.time_end as f64 - self.viewport.time_start as f64;
        let time_step = time_range / x_tick_count as f64;

        for i in 0..=x_tick_count {
            let timestamp = self.viewport.time_start as f64 + time_step * i as f64;
            let text = format_timestamp(timestamp as u64);
            if label_idx < self.axis_label_pool.len() {
                self.text_renderer
                    .set_text(self.axis_label_pool[label_idx], &text);
                label_idx += 1;
            }
        }
    }

    /// Update crosshair labels (price on right, time on bottom, OHLC tooltip).
    fn update_crosshair_labels(&mut self) {
        if !self.crosshair_active {
            self.text_renderer
                .set_text(self.crosshair_label_buffer, "");
            return;
        }

        let mut text = String::new();

        // Price at cursor Y
        let price = self.screen_y_to_price(self.crosshair_y);
        text.push_str(&format!("{}\n", self.formatter.format_short(price)));

        // Time at cursor X
        let timestamp = self.screen_x_to_timestamp(self.crosshair_x);
        text.push_str(&format!("{}\n", format_timestamp(timestamp as u64)));

        // OHLC tooltip if hovering over a bar
        if let Some(bar) = self.find_bar_at_crosshair() {
            text.push_str(&format!(
                "O:{} H:{} L:{} C:{}",
                self.formatter.format_short(bar.open),
                self.formatter.format_short(bar.high),
                self.formatter.format_short(bar.low),
                self.formatter.format_short(bar.close),
            ));
        }

        self.text_renderer
            .set_text(self.crosshair_label_buffer, &text);
    }

    /// Build the list of text areas to render this frame.
    fn build_text_areas(&self) -> Vec<PreparedTextArea> {
        let mut areas = Vec::new();
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;
        let axis_color = glyphon::Color::rgba(200, 200, 200, 200);

        let h_count = 10;
        let price_step = (self.viewport.value_max - self.viewport.value_min) / h_count as f64;
        let mut label_idx = 0;

        // Y-axis: price labels at each grid line Y position (right edge)
        for i in 0..=h_count {
            if label_idx >= self.axis_label_pool.len() {
                break;
            }
            let price_y = self.viewport.value_min + price_step * i as f64;
            // Convert price to pixel Y (flipped: max = top = 0px, min = bottom = h px)
            let ratio = (price_y - self.viewport.value_min)
                / (self.viewport.value_max - self.viewport.value_min);
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
        let time_range = self.viewport.time_end as f64 - self.viewport.time_start as f64;
        let time_step = time_range / x_tick_count as f64;

        for i in 0..=x_tick_count {
            if label_idx >= self.axis_label_pool.len() {
                break;
            }
            let timestamp = self.viewport.time_start as f64 + time_step * i as f64;
            // Convert timestamp to pixel X
            let time_ratio = (timestamp - self.viewport.time_start as f64) / time_range;
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
