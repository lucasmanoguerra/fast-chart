/// A simple GPU resource cache with a fixed capacity.
///
/// Tracks resource slots that can be reused across frames to avoid
/// repeated allocation of GPU buffers and textures.
#[derive(Debug)]
pub struct GpuCache {
    capacity: usize,
}

impl GpuCache {
    pub fn new(capacity: usize) -> Self {
        Self { capacity }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for GpuCache {
    fn default() -> Self {
        Self::new(1024)
    }
}
