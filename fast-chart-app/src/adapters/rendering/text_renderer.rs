use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextAtlas, TextArea, TextBounds, TextRenderer as GlyphonTextRenderer, Viewport as GlyphonViewport,
};

/// Describes a text area to render after buffer setup.
pub struct PreparedTextArea {
    pub buffer_idx: usize,
    pub left: f32,
    pub top: f32,
    pub scale: f32,
    pub right: f32,
    pub bottom: f32,
    pub color: Color,
}

/// GPU-accelerated text renderer wrapping glyphon (cosmic-text + swash).
///
/// Manages font loading, glyph caching, and text rendering into an existing wgpu render pass.
pub struct GpuTextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonTextRenderer,
    viewport: GlyphonViewport,
    buffers: Vec<Buffer>,
}

impl GpuTextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let cache = Cache::new(device);
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = GlyphonTextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );
        let viewport = GlyphonViewport::new(device, &cache);

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            viewport,
            buffers: Vec::new(),
        }
    }

    /// Create a new text buffer with the given font size. Returns its index.
    pub fn create_buffer(&mut self, font_size: f32) -> usize {
        let buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 1.4),
        );
        let idx = self.buffers.len();
        self.buffers.push(buffer);
        idx
    }

    /// Set text content for a buffer.
    pub fn set_text(&mut self, buffer_idx: usize, text: &str) {
        let buffer = &mut self.buffers[buffer_idx];
        buffer.set_size(&mut self.font_system, Some(f32::INFINITY), Some(f32::INFINITY));
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
        );
    }

    /// Update the screen resolution (call on resize).
    pub fn update_resolution(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.viewport
            .update(queue, Resolution { width, height });
    }

    /// Prepare all buffers for rendering. Only buffers referenced by `areas` are prepared.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        areas: &[PreparedTextArea],
    ) {
        if areas.is_empty() {
            return;
        }

        // Build TextArea references. Each borrows from self.buffers (immutable).
        // We need to pass &mut self.font_system etc. to prepare(), which are different fields.
        // Rust's NLL allows split borrows on struct fields.
        let text_areas: Vec<TextArea<'_>> = areas
            .iter()
            .map(|ta| TextArea {
                buffer: &self.buffers[ta.buffer_idx],
                left: ta.left,
                top: ta.top,
                scale: ta.scale,
                bounds: TextBounds {
                    left: ta.left as i32,
                    top: ta.top as i32,
                    right: ta.right as i32,
                    bottom: ta.bottom as i32,
                },
                default_color: ta.color,
                custom_glyphs: &[],
            })
            .collect();

        let _ = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        );
    }

    /// Render all prepared text into the active render pass.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let _ = self
            .renderer
            .render(&self.atlas, &self.viewport, render_pass);
    }
}
