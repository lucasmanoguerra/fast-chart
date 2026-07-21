use fc_render::commands::DrawCommand;
use fc_primitives::Rect;
use fc_render::series_renderer::{SeriesHit, SeriesRenderer};

/// A text label data point in screen coordinates.
#[derive(Debug, Clone)]
pub struct TextItem {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub font_size: f32,
}

/// Text renderer — produces DrawText commands.
pub struct TextRenderer {
    pub color: [f32; 4],
    pub font_size: f32,
    bounds: Rect,
}

impl TextRenderer {
    pub fn new(color: [f32; 4], width: f32) -> Self {
        Self {
            color,
            font_size: width,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Generate draw commands from text items.
    pub fn render(&self, items: &[TextItem], commands: &mut Vec<DrawCommand>) {
        for item in items {
            commands.push(DrawCommand::text(
                item.x,
                item.y,
                &item.text,
                self.color,
                item.font_size,
                900,
            ));
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new([1.0, 1.0, 1.0, 1.0], 12.0)
    }
}

impl SeriesRenderer for TextRenderer {
    fn update(
        &mut self,
        _data: &[DrawCommand],
        bounds: Rect,
    ) -> Vec<DrawCommand> {
        self.bounds = bounds;
        Vec::new()
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<SeriesHit> {
        if self.bounds.contains(x, y) {
            Some(SeriesHit {
                index: 0,
                distance: 0.0,
            })
        } else {
            None
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layer_z_index(&self) -> i32 {
        900
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_renderer() {
        let r = TextRenderer::new([1.0; 4], 14.0);
        assert_eq!(r.color, [1.0; 4]);
        assert_eq!(r.font_size, 14.0);
    }

    #[test]
    fn render_produces_commands() {
        let r = TextRenderer::new([1.0; 4], 12.0);
        let mut cmds = Vec::new();
        let items = vec![
            TextItem { x: 10.0, y: 20.0, text: "BTC".into(), font_size: 12.0 },
            TextItem { x: 50.0, y: 60.0, text: "ETH".into(), font_size: 10.0 },
        ];
        r.render(&items, &mut cmds);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn render_empty_data() {
        let r = TextRenderer::new([1.0; 4], 12.0);
        let mut cmds = Vec::new();
        r.render(&[], &mut cmds);
        assert!(cmds.is_empty());
    }

    #[test]
    fn text_has_correct_content() {
        let r = TextRenderer::new([1.0; 4], 14.0);
        let mut cmds = Vec::new();
        let items = vec![TextItem { x: 10.0, y: 20.0, text: "Hello".into(), font_size: 14.0 }];
        r.render(&items, &mut cmds);
        if let DrawCommand::DrawText { text, x, y, font_size, .. } = &cmds[0] {
            assert_eq!(text, "Hello");
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
            assert_eq!(*font_size, 14.0);
        } else {
            panic!("expected DrawText");
        }
    }
}
