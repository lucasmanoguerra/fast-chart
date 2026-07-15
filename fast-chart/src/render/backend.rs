// ---------------------------------------------------------------------------
// RendererBackend — trait for GPU/software rendering backends
// ---------------------------------------------------------------------------

use super::commands::DrawCommand;
use super::series_renderer::Rect;

/// The trait that a host application must implement to execute draw commands.
///
/// The library produces `Vec<DrawCommand>` and the backend executes them.
/// This separation keeps the library renderer-agnostic: the same chart logic
/// works with wgpu, glow, Skia, or a software rasterizer.
///
/// # Object Safety
///
/// This trait is object-safe: you can use `Box<dyn RendererBackend>`.
pub trait RendererBackend: Send + Sync {
    /// Execute a batch of draw commands.
    ///
    /// Commands are pre-sorted by layer and z-index. The backend
    /// translates them into GPU calls, canvas operations, etc.
    fn execute(&mut self, commands: &[DrawCommand]);

    /// Resize the rendering surface.
    ///
    /// Called when the window or container changes size.
    /// The backend must update its internal surface/texture dimensions.
    fn resize(&mut self, width: u32, height: u32);

    /// Set a clipping rectangle.
    ///
    /// All subsequent draw commands are clipped to this rect until
    /// `clear_clip()` is called. Used for pane-level clipping.
    fn set_clip(&mut self, rect: Rect);

    /// Clear the current clipping rectangle.
    fn clear_clip(&mut self);

    /// Clear the entire surface with a color.
    fn clear(&mut self, color: [f32; 4]);

    /// The current surface width in pixels.
    fn width(&self) -> u32;

    /// The current surface height in pixels.
    fn height(&self) -> u32;

    /// The DPI scale factor (1.0 = no scaling, 2.0 = Retina).
    fn scale_factor(&self) -> f32 {
        1.0
    }

    /// Begin a new frame. Called before any draw commands for this frame.
    fn begin_frame(&mut self) {}

    /// End the current frame. Called after all draw commands for this frame.
    ///
    /// The backend should present/swap buffers here if applicable.
    fn end_frame(&mut self) {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock backend for testing.
    struct MockBackend {
        width: u32,
        height: u32,
        clip: Option<Rect>,
        executed_commands: Vec<Vec<DrawCommand>>,
    }

    impl MockBackend {
        fn new(width: u32, height: u32) -> Self {
            Self {
                width,
                height,
                clip: None,
                executed_commands: Vec::new(),
            }
        }
    }

    impl RendererBackend for MockBackend {
        fn execute(&mut self, commands: &[DrawCommand]) {
            self.executed_commands.push(commands.to_vec());
        }

        fn resize(&mut self, width: u32, height: u32) {
            self.width = width;
            self.height = height;
        }

        fn set_clip(&mut self, rect: Rect) {
            self.clip = Some(rect);
        }

        fn clear_clip(&mut self) {
            self.clip = None;
        }

        fn clear(&mut self, _color: [f32; 4]) {}

        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }
    }

    #[test]
    fn mock_backend_executes_commands() {
        let mut backend = MockBackend::new(800, 600);
        let cmds = vec![DrawCommand::line(
            0.0, 0.0, 100.0, 100.0, [1.0; 4], 1.0, 0,
        )];
        backend.execute(&cmds);
        assert_eq!(backend.executed_commands.len(), 1);
        assert_eq!(backend.executed_commands[0].len(), 1);
    }

    #[test]
    fn mock_backend_resize() {
        let mut backend = MockBackend::new(800, 600);
        backend.resize(1920, 1080);
        assert_eq!(backend.width(), 1920);
        assert_eq!(backend.height(), 1080);
    }

    #[test]
    fn mock_backend_clip() {
        let mut backend = MockBackend::new(800, 600);
        assert!(backend.clip.is_none());

        let rect = Rect::new(100.0, 50.0, 600.0, 400.0);
        backend.set_clip(rect);
        assert_eq!(backend.clip, Some(rect));

        backend.clear_clip();
        assert!(backend.clip.is_none());
    }

    #[test]
    fn mock_backend_default_scale_factor() {
        let backend = MockBackend::new(800, 600);
        assert_eq!(backend.scale_factor(), 1.0);
    }

    #[test]
    fn mock_backend_begin_end_frame() {
        let mut backend = MockBackend::new(800, 600);
        // Should not panic
        backend.begin_frame();
        backend.end_frame();
    }

    #[test]
    fn backend_is_object_safe() {
        // Verify RendererBackend can be used as a trait object
        let backend: Box<dyn RendererBackend> = Box::new(MockBackend::new(800, 600));
        assert_eq!(backend.width(), 800);
    }
}
