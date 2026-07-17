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
