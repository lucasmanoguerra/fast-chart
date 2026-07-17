#[derive(Debug, Clone)]
pub struct ScissorRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ScissorRect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Default for ScissorRect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}
