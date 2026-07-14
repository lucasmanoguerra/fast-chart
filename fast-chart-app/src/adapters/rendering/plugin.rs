//! Plugin system for extensible series rendering and pane primitives.
//!
//! # Architecture Note
//!
//! Plugin traits reference `wgpu` types directly (`Device`, `Queue`,
//! `RenderPass`) because they live in the rendering adapter layer, not
//! the core domain. If a second backend is added, these traits will be
//! refactored behind a `RenderContext` abstraction.
//!
//! This module defines traits that allow third-party code to register
//! custom series renderers and pane overlays without modifying GpuRenderer.

use fast_chart_domain::bar::Bar;
use fast_chart_domain::viewport::Viewport;

/// Unique identifier for a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(pub &'static str);

/// Rendering layer order (lower = rendered first, behind higher layers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderLayer(pub u32);

impl RenderLayer {
    pub const GRID: Self = Self(0);
    pub const DATA_SERIES: Self = Self(100);
    pub const OVERLAY: Self = Self(200);
    pub const CROSSHAIR: Self = Self(300);
    pub const UI: Self = Self(400);
}

/// Errors that can occur during plugin registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    /// A plugin with this ID is already registered.
    DuplicateId(PluginId),
    /// No plugin with this ID was found.
    NotFound(PluginId),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "Plugin with ID '{}' already registered", id.0),
            Self::NotFound(id) => write!(f, "Plugin with ID '{}' not found", id.0),
        }
    }
}

impl std::error::Error for PluginError {}

/// Trait for series renderers (Line, Candle, Area, Histogram, etc.).
///
/// Implement this trait to add a new series type to the chart.
pub trait SeriesRenderer: Send + Sync {
    /// Unique identifier for this renderer type.
    fn plugin_id(&self) -> PluginId;

    /// Rendering layer (lower = behind).
    fn layer(&self) -> RenderLayer {
        RenderLayer::DATA_SERIES
    }

    /// Update vertex buffers with new data.
    ///
    /// Called when `InvalidationLevel::Full` is received.
    fn update_data(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bars: &[Bar],
        viewport: &Viewport,
    );

    /// Issue render commands.
    ///
    /// Called every frame when this layer is visible.
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);

    /// Handle window resize.
    fn resize(&mut self, width: u32, height: u32);
}

/// Trait for pane primitives (horizontal lines, markers, annotations).
///
/// These are lightweight overlays that render on top of data series.
pub trait PanePrimitive: Send + Sync {
    /// Unique identifier for this primitive instance.
    fn plugin_id(&self) -> PluginId;

    /// Rendering layer (default: OVERLAY).
    fn layer(&self) -> RenderLayer {
        RenderLayer::OVERLAY
    }

    /// Update primitive state.
    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &Viewport,
    );

    /// Issue render commands.
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);

    /// Handle window resize.
    fn resize(&mut self, width: u32, height: u32);
}

/// Registry that manages plugins and dispatches rendering.
pub struct PluginRegistry {
    series_renderers: Vec<Box<dyn SeriesRenderer>>,
    pane_primitives: Vec<Box<dyn PanePrimitive>>,
    // Index by layer for sorted dispatch
    series_by_layer: Vec<(RenderLayer, usize)>,
    primitive_by_layer: Vec<(RenderLayer, usize)>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            series_renderers: Vec::new(),
            pane_primitives: Vec::new(),
            series_by_layer: Vec::new(),
            primitive_by_layer: Vec::new(),
        }
    }

    /// Register a series renderer.
    ///
    /// Returns `Err(PluginError::DuplicateId)` if a renderer with the same
    /// plugin ID is already registered.
    pub fn register_series(&mut self, renderer: Box<dyn SeriesRenderer>) -> Result<(), PluginError> {
        let id = renderer.plugin_id();
        if self.series_renderers.iter().any(|r| r.plugin_id() == id) {
            return Err(PluginError::DuplicateId(id));
        }
        let idx = self.series_renderers.len();
        let layer = renderer.layer();
        self.series_renderers.push(renderer);
        self.series_by_layer.push((layer, idx));
        self.series_by_layer.sort_by_key(|(l, _)| *l);
        Ok(())
    }

    /// Register a pane primitive.
    ///
    /// Returns `Err(PluginError::DuplicateId)` if a primitive with the same
    /// plugin ID is already registered.
    pub fn register_primitive(&mut self, primitive: Box<dyn PanePrimitive>) -> Result<(), PluginError> {
        let id = primitive.plugin_id();
        if self.pane_primitives.iter().any(|p| p.plugin_id() == id) {
            return Err(PluginError::DuplicateId(id));
        }
        let idx = self.pane_primitives.len();
        let layer = primitive.layer();
        self.pane_primitives.push(primitive);
        self.primitive_by_layer.push((layer, idx));
        self.primitive_by_layer.sort_by_key(|(l, _)| *l);
        Ok(())
    }

    /// Get a series renderer by plugin ID.
    pub fn get_series(&self, id: &PluginId) -> Option<&dyn SeriesRenderer> {
        self.series_renderers
            .iter()
            .find(|r| r.plugin_id() == *id)
            .map(|r| r.as_ref())
    }

    /// Get a mutable series renderer by plugin ID.
    pub fn get_series_mut(&mut self, id: &PluginId) -> Option<&mut dyn SeriesRenderer> {
        let idx = self
            .series_renderers
            .iter()
            .position(|r| r.plugin_id() == *id)?;
        Some(&mut *self.series_renderers[idx])
    }

    /// Get a pane primitive by plugin ID.
    pub fn get_primitive(&self, id: &PluginId) -> Option<&dyn PanePrimitive> {
        self.pane_primitives
            .iter()
            .find(|p| p.plugin_id() == *id)
            .map(|p| p.as_ref())
    }

    /// Render all series in layer order.
    pub fn render_series<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for &(_, idx) in &self.series_by_layer {
            self.series_renderers[idx].render(render_pass);
        }
    }

    /// Render all primitives in layer order.
    pub fn render_primitives<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for &(_, idx) in &self.primitive_by_layer {
            self.pane_primitives[idx].render(render_pass);
        }
    }

    /// Update all series with new data.
    pub fn update_all_series(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bars: &[Bar],
        viewport: &Viewport,
    ) {
        for renderer in &mut self.series_renderers {
            renderer.update_data(device, queue, bars, viewport);
        }
    }

    /// Update all primitives with viewport changes.
    pub fn update_all_primitives(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &Viewport,
    ) {
        for primitive in &mut self.pane_primitives {
            primitive.update(device, queue, viewport);
        }
    }

    /// Resize all plugins.
    pub fn resize_all(&mut self, width: u32, height: u32) {
        for renderer in &mut self.series_renderers {
            renderer.resize(width, height);
        }
        for primitive in &mut self.pane_primitives {
            primitive.resize(width, height);
        }
    }

    /// Number of registered series renderers.
    pub fn series_count(&self) -> usize {
        self.series_renderers.len()
    }

    /// Number of registered pane primitives.
    pub fn primitive_count(&self) -> usize {
        self.pane_primitives.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock renderer for testing
    struct MockSeriesRenderer {
        id: PluginId,
        layer: RenderLayer,
        updated: bool,
    }

    impl MockSeriesRenderer {
        fn new(id: &'static str, layer: RenderLayer) -> Self {
            Self {
                id: PluginId(id),
                layer,
                updated: false,
            }
        }
    }

    impl SeriesRenderer for MockSeriesRenderer {
        fn plugin_id(&self) -> PluginId {
            self.id.clone()
        }
        fn layer(&self) -> RenderLayer {
            self.layer
        }
        fn update_data(
            &mut self,
            _: &wgpu::Device,
            _: &wgpu::Queue,
            _: &[Bar],
            _: &Viewport,
        ) {
            self.updated = true;
        }
        fn render<'a>(&'a self, _: &mut wgpu::RenderPass<'a>) {}
        fn resize(&mut self, _: u32, _: u32) {}
    }

    struct MockPrimitive {
        id: PluginId,
        layer: RenderLayer,
    }

    impl MockPrimitive {
        fn new(id: &'static str) -> Self {
            Self {
                id: PluginId(id),
                layer: RenderLayer::OVERLAY,
            }
        }

        fn with_layer(id: &'static str, layer: RenderLayer) -> Self {
            Self {
                id: PluginId(id),
                layer,
            }
        }
    }

    impl PanePrimitive for MockPrimitive {
        fn plugin_id(&self) -> PluginId {
            self.id.clone()
        }
        fn layer(&self) -> RenderLayer {
            self.layer
        }
        fn update(&mut self, _: &wgpu::Device, _: &wgpu::Queue, _: &Viewport) {}
        fn render<'a>(&'a self, _: &mut wgpu::RenderPass<'a>) {}
        fn resize(&mut self, _: u32, _: u32) {}
    }

    #[test]
    fn registry_new_is_empty() {
        let reg = PluginRegistry::new();
        assert_eq!(reg.series_count(), 0);
        assert_eq!(reg.primitive_count(), 0);
    }

    #[test]
    fn register_series() {
        let mut reg = PluginRegistry::new();
        reg.register_series(Box::new(MockSeriesRenderer::new(
            "line",
            RenderLayer::DATA_SERIES,
        )))
        .unwrap();
        assert_eq!(reg.series_count(), 1);
        assert!(reg.get_series(&PluginId("line")).is_some());
    }

    #[test]
    fn register_primitive() {
        let mut reg = PluginRegistry::new();
        reg.register_primitive(Box::new(MockPrimitive::new("price-line")))
            .unwrap();
        assert_eq!(reg.primitive_count(), 1);
        assert!(reg.get_primitive(&PluginId("price-line")).is_some());
    }

    #[test]
    fn get_series_not_found() {
        let reg = PluginRegistry::new();
        assert!(reg.get_series(&PluginId("nonexistent")).is_none());
    }

    #[test]
    fn series_layer_ordering() {
        let mut reg = PluginRegistry::new();
        reg.register_series(Box::new(MockSeriesRenderer::new("ui", RenderLayer::UI)))
            .unwrap();
        reg.register_series(Box::new(MockSeriesRenderer::new(
            "data",
            RenderLayer::DATA_SERIES,
        )))
        .unwrap();
        reg.register_series(Box::new(MockSeriesRenderer::new(
            "grid",
            RenderLayer::GRID,
        )))
        .unwrap();

        // Should be sorted: GRID(0) < DATA_SERIES(100) < UI(400)
        assert_eq!(reg.series_by_layer[0].0, RenderLayer::GRID);
        assert_eq!(reg.series_by_layer[1].0, RenderLayer::DATA_SERIES);
        assert_eq!(reg.series_by_layer[2].0, RenderLayer::UI);
    }

    #[test]
    fn primitive_layer_ordering() {
        let mut reg = PluginRegistry::new();
        reg.register_primitive(Box::new(MockPrimitive::new("overlay")))
            .unwrap(); // OVERLAY = 200
        reg.register_primitive(Box::new(MockPrimitive::with_layer(
            "crosshair",
            RenderLayer::CROSSHAIR,
        )))
        .unwrap(); // CROSSHAIR = 300

        assert_eq!(reg.primitive_by_layer[0].0, RenderLayer::OVERLAY);
        assert_eq!(reg.primitive_by_layer[1].0, RenderLayer::CROSSHAIR);
    }

    #[test]
    fn resize_all_calls_all() {
        // Just verify no panic
        let mut reg = PluginRegistry::new();
        reg.register_series(Box::new(MockSeriesRenderer::new(
            "line",
            RenderLayer::DATA_SERIES,
        )))
        .unwrap();
        reg.register_primitive(Box::new(MockPrimitive::new("marker")))
            .unwrap();
        reg.resize_all(800, 600);
    }

    #[test]
    fn get_series_mut_allows_modification() {
        let mut reg = PluginRegistry::new();
        reg.register_series(Box::new(MockSeriesRenderer::new(
            "line",
            RenderLayer::DATA_SERIES,
        )))
        .unwrap();

        // Get mutable reference and modify
        if let Some(renderer) = reg.get_series_mut(&PluginId("line")) {
            // Verify we can call methods on the mutable reference
            renderer.resize(1920, 1080);
        }

        // Verify the renderer is still there
        assert!(reg.get_series(&PluginId("line")).is_some());
    }

    #[test]
    fn default_is_empty() {
        let reg = PluginRegistry::default();
        assert_eq!(reg.series_count(), 0);
        assert_eq!(reg.primitive_count(), 0);
    }

    #[test]
    fn register_series_rejects_duplicate_id() {
        let mut reg = PluginRegistry::new();
        reg.register_series(Box::new(MockSeriesRenderer::new("line", RenderLayer::DATA_SERIES)))
            .unwrap();

        let result =
            reg.register_series(Box::new(MockSeriesRenderer::new("line", RenderLayer::OVERLAY)));
        assert_eq!(
            result,
            Err(PluginError::DuplicateId(PluginId("line")))
        );
        assert_eq!(reg.series_count(), 1);
    }

    #[test]
    fn register_primitive_rejects_duplicate_id() {
        let mut reg = PluginRegistry::new();
        reg.register_primitive(Box::new(MockPrimitive::new("marker")))
            .unwrap();

        let result = reg.register_primitive(Box::new(MockPrimitive::new("marker")));
        assert_eq!(
            result,
            Err(PluginError::DuplicateId(PluginId("marker")))
        );
        assert_eq!(reg.primitive_count(), 1);
    }
}

#[cfg(test)]
mod id_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn plugin_id_equality() {
        assert_eq!(PluginId("a"), PluginId("a"));
        assert_ne!(PluginId("a"), PluginId("b"));
    }

    #[test]
    fn plugin_id_hash() {
        let mut map = HashMap::new();
        map.insert(PluginId("a"), 1);
        map.insert(PluginId("b"), 2);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&PluginId("a")), Some(&1));
    }

    #[test]
    fn render_layer_ordering() {
        assert!(RenderLayer::GRID < RenderLayer::DATA_SERIES);
        assert!(RenderLayer::DATA_SERIES < RenderLayer::OVERLAY);
        assert!(RenderLayer::OVERLAY < RenderLayer::CROSSHAIR);
        assert!(RenderLayer::CROSSHAIR < RenderLayer::UI);
    }

    #[test]
    fn render_layer_constants() {
        assert_eq!(RenderLayer::GRID.0, 0);
        assert_eq!(RenderLayer::DATA_SERIES.0, 100);
        assert_eq!(RenderLayer::OVERLAY.0, 200);
        assert_eq!(RenderLayer::CROSSHAIR.0, 300);
        assert_eq!(RenderLayer::UI.0, 400);
    }
}
