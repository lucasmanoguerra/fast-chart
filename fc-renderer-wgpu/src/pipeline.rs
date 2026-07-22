/// Wrapper around a wgpu render pipeline.
///
/// Holds the compiled shader and pipeline configuration used to
/// issue draw commands against the GPU.
#[derive(Debug)]
pub struct RenderPipeline {
    #[allow(dead_code)]
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
