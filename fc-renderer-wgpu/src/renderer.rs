use crate::backend::WgpuBackend;

pub struct WgpuRenderer {
    backend: WgpuBackend,
}

impl WgpuRenderer {
    pub fn new(backend: WgpuBackend) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &WgpuBackend {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut WgpuBackend {
        &mut self.backend
    }
}
