use fast_chart::render::backend::RendererBackend;
use fast_chart::render::commands::DrawCommand;
use fast_chart::Rect;

pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    width: u32,
    height: u32,
}

impl WgpuBackend {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            surface: None,
            surface_config: None,
            pipeline: None,
            width: 0,
            height: 0,
        }
    }

    pub fn set_surface(
        &mut self,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) {
        self.width = config.width;
        self.height = config.height;
        self.surface_config = Some(config);
        self.surface = Some(surface);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> Option<&wgpu::Surface<'static>> {
        self.surface.as_ref()
    }

    pub fn surface_config(&self) -> Option<&wgpu::SurfaceConfiguration> {
        self.surface_config.as_ref()
    }
}

impl RendererBackend for WgpuBackend {
    fn execute(&mut self, _commands: &[DrawCommand]) {
        log::trace!("WgpuBackend::execute — {} commands (stub)", _commands.len());
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        if let (Some(surface), Some(config)) = (&self.surface, &self.surface_config) {
            let mut new_config = config.clone();
            new_config.width = width;
            new_config.height = height;
            surface.configure(&self.device, &new_config);
            self.surface_config = Some(new_config);
        }
    }

    fn set_clip(&mut self, _rect: Rect) {
        log::trace!("WgpuBackend::set_clip — (stub)");
    }

    fn clear_clip(&mut self) {
        log::trace!("WgpuBackend::clear_clip — (stub)");
    }

    fn clear(&mut self, _color: [f32; 4]) {
        log::trace!("WgpuBackend::clear — (stub)");
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn scale_factor(&self) -> f32 {
        1.0
    }

    fn begin_frame(&mut self) {
        log::trace!("WgpuBackend::begin_frame — (stub)");
    }

    fn end_frame(&mut self) {
        log::trace!("WgpuBackend::end_frame — (stub)");
    }
}
