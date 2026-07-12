/// Draw order for the single-pass compositing pipeline.
///
/// All layers share one `wgpu::RenderPass`. The order they are drawn determines
/// the visual stacking: grid is lowest (background), line series on top.
///
/// Future layers (candle, indicator overlays, crosshair, HUD text) will be added
/// here following the spec: grid → series → indicators → crosshair → HUD.
#[allow(dead_code)]
pub enum DrawLayer {
    Grid,
    Series,
    // Future:
    // Indicators,
    // Crosshair,
    // Hud,
}
