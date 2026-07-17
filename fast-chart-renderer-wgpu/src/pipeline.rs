pub struct RenderPipeline {
    pipeline: Option<wgpu::RenderPipeline>,
}

impl RenderPipeline {
    pub fn new() -> Self {
        Self { pipeline: None }
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}
