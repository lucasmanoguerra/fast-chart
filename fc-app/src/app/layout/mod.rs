// ---------------------------------------------------------------------------
// LayoutEngine — trait for computing pane rects from a parent rect
// ---------------------------------------------------------------------------

use fc_primitives::Rect;

/// A layout strategy that computes pane rects from a parent bounding rect.
///
/// Implementations include vertical stacking, horizontal splitting, and grid layouts.
pub trait LayoutEngine {
    /// Compute the rects for each pane given the parent bounding rect.
    ///
    /// Returns one `Rect` per pane, in order. The rects should be non-overlapping
    /// and fit within the parent rect (possibly with gaps for dividers).
    fn compute_rects(&self, parent: Rect, pane_count: usize) -> Vec<Rect>;

    /// The number of panes this layout manages.
    ///
    /// For fixed-count layouts (e.g., GridLayout), this returns the exact count.
    /// For dynamic layouts (e.g., VerticalStack), this returns 0 (caller provides count).
    fn pane_count_hint(&self) -> usize {
        0
    }
}

// ---------------------------------------------------------------------------
// VerticalStack — panes stacked top to bottom (the default)
// ---------------------------------------------------------------------------

/// Vertical stack layout: panes are stacked top to bottom with equal or
/// proportional heights. This is the standard chart layout.
pub struct VerticalStack {
    /// Proportional heights for each pane. If empty, panes are distributed equally.
    pub heights: Vec<f64>,
    /// Gap between panes in pixels (for dividers).
    pub gap: f32,
}

impl VerticalStack {
    /// Create a vertical stack with equal-height panes.
    pub fn new() -> Self {
        Self {
            heights: Vec::new(),
            gap: 0.0,
        }
    }

    /// Create a vertical stack with proportional heights.
    pub fn with_heights(heights: Vec<f64>) -> Self {
        Self {
            heights,
            gap: 0.0,
        }
    }

    /// Set the gap between panes in pixels.
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl Default for VerticalStack {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for VerticalStack {
    fn compute_rects(&self, parent: Rect, pane_count: usize) -> Vec<Rect> {
        if pane_count == 0 {
            return Vec::new();
        }

        let total_gap = self.gap * (pane_count as f32 - 1.0).max(0.0);
        let available_height = parent.height - total_gap;

        let mut rects = Vec::with_capacity(pane_count);
        let mut y = parent.y;

        for i in 0..pane_count {
            let h = if i < self.heights.len() {
                // Use provided proportional height
                (self.heights[i] as f32) * available_height
            } else {
                // Equal distribution
                available_height / pane_count as f32
            };

            rects.push(Rect::new(parent.x, y, parent.width, h));
            y += h + self.gap;
        }

        rects
    }
}

// ---------------------------------------------------------------------------
// HorizontalSplit — panes split left to right
// ---------------------------------------------------------------------------

/// Horizontal split layout: panes are placed side by side, left to right.
/// Useful for comparing multiple charts or indicators side by side.
pub struct HorizontalSplit {
    /// Proportional widths for each pane. If empty, panes are distributed equally.
    pub widths: Vec<f64>,
    /// Gap between panes in pixels.
    pub gap: f32,
}

impl HorizontalSplit {
    /// Create a horizontal split with equal-width panes.
    pub fn new() -> Self {
        Self {
            widths: Vec::new(),
            gap: 0.0,
        }
    }

    /// Create a horizontal split with proportional widths.
    pub fn with_widths(widths: Vec<f64>) -> Self {
        Self {
            widths,
            gap: 0.0,
        }
    }

    /// Set the gap between panes in pixels.
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl Default for HorizontalSplit {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for HorizontalSplit {
    fn compute_rects(&self, parent: Rect, pane_count: usize) -> Vec<Rect> {
        if pane_count == 0 {
            return Vec::new();
        }

        let total_gap = self.gap * (pane_count as f32 - 1.0).max(0.0);
        let available_width = parent.width - total_gap;

        let mut rects = Vec::with_capacity(pane_count);
        let mut x = parent.x;

        for i in 0..pane_count {
            let w = if i < self.widths.len() {
                (self.widths[i] as f32) * available_width
            } else {
                available_width / pane_count as f32
            };

            rects.push(Rect::new(x, parent.y, w, parent.height));
            x += w + self.gap;
        }

        rects
    }
}

// ---------------------------------------------------------------------------
// GridLayout — rows × cols grid of panes
// ---------------------------------------------------------------------------

/// Grid layout: panes arranged in a rows × cols grid.
/// Pane count must equal rows × cols.
pub struct GridLayout {
    pub rows: usize,
    pub cols: usize,
    /// Horizontal gap between columns in pixels.
    pub h_gap: f32,
    /// Vertical gap between rows in pixels.
    pub v_gap: f32,
}

impl GridLayout {
    /// Create a grid layout with the given dimensions.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            h_gap: 0.0,
            v_gap: 0.0,
        }
    }

    /// Set the gaps between cells.
    pub fn with_gaps(mut self, h_gap: f32, v_gap: f32) -> Self {
        self.h_gap = h_gap;
        self.v_gap = v_gap;
        self
    }
}

impl LayoutEngine for GridLayout {
    fn compute_rects(&self, parent: Rect, pane_count: usize) -> Vec<Rect> {
        let expected = self.rows * self.cols;
        if pane_count == 0 || expected == 0 {
            return Vec::new();
        }

        // Only layout up to min(pane_count, expected) panes
        let count = pane_count.min(expected);

        let total_h_gap = self.h_gap * (self.cols as f32 - 1.0).max(0.0);
        let total_v_gap = self.v_gap * (self.rows as f32 - 1.0).max(0.0);

        let cell_width = (parent.width - total_h_gap) / self.cols as f32;
        let cell_height = (parent.height - total_v_gap) / self.rows as f32;

        let mut rects = Vec::with_capacity(count);
        for idx in 0..count {
            let row = idx / self.cols;
            let col = idx % self.cols;

            let x = parent.x + col as f32 * (cell_width + self.h_gap);
            let y = parent.y + row as f32 * (cell_height + self.v_gap);

            rects.push(Rect::new(x, y, cell_width, cell_height));
        }

        rects
    }

    fn pane_count_hint(&self) -> usize {
        self.rows * self.cols
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parent() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    // ---- VerticalStack ----

    #[test]
    fn vertical_stack_equal() {
        let layout = VerticalStack::new();
        let rects = layout.compute_rects(parent(), 3);
        assert_eq!(rects.len(), 3);
        // Each should be ~200px tall
        for r in &rects {
            assert!((r.height - 200.0).abs() < 1.0, "height = {}", r.height);
            assert_eq!(r.width, 800.0);
        }
        // Stacked vertically
        assert!((rects[0].y - 0.0).abs() < 1.0);
        assert!((rects[1].y - 200.0).abs() < 1.0);
        assert!((rects[2].y - 400.0).abs() < 1.0);
    }

    #[test]
    fn vertical_stack_proportional() {
        let layout = VerticalStack::with_heights(vec![0.7, 0.3]);
        let rects = layout.compute_rects(parent(), 2);
        assert_eq!(rects.len(), 2);
        assert!((rects[0].height - 420.0).abs() < 1.0); // 0.7 * 600
        assert!((rects[1].height - 180.0).abs() < 1.0); // 0.3 * 600
    }

    #[test]
    fn vertical_stack_with_gap() {
        let layout = VerticalStack::new().with_gap(10.0);
        let rects = layout.compute_rects(parent(), 2);
        assert_eq!(rects.len(), 2);
        // Total gap = 10px, available = 590px, each = 295px
        assert!((rects[0].height - 295.0).abs() < 1.0);
        assert!((rects[1].height - 295.0).abs() < 1.0);
        // Gap between them
        assert!((rects[1].y - (rects[0].y + rects[0].height + 10.0)).abs() < 1.0);
    }

    #[test]
    fn vertical_stack_empty() {
        let layout = VerticalStack::new();
        let rects = layout.compute_rects(parent(), 0);
        assert!(rects.is_empty());
    }

    // ---- HorizontalSplit ----

    #[test]
    fn horizontal_split_equal() {
        let layout = HorizontalSplit::new();
        let rects = layout.compute_rects(parent(), 2);
        assert_eq!(rects.len(), 2);
        assert!((rects[0].width - 400.0).abs() < 1.0);
        assert!((rects[1].width - 400.0).abs() < 1.0);
        assert_eq!(rects[0].height, 600.0);
        // Side by side
        assert!((rects[0].x - 0.0).abs() < 1.0);
        assert!((rects[1].x - 400.0).abs() < 1.0);
    }

    #[test]
    fn horizontal_split_proportional() {
        let layout = HorizontalSplit::with_widths(vec![0.6, 0.4]);
        let rects = layout.compute_rects(parent(), 2);
        assert!((rects[0].width - 480.0).abs() < 1.0);
        assert!((rects[1].width - 320.0).abs() < 1.0);
    }

    #[test]
    fn horizontal_split_with_gap() {
        let layout = HorizontalSplit::new().with_gap(20.0);
        let rects = layout.compute_rects(parent(), 2);
        // Available = 780px, each = 390px
        assert!((rects[0].width - 390.0).abs() < 1.0);
        assert!((rects[1].width - 390.0).abs() < 1.0);
    }

    #[test]
    fn horizontal_split_empty() {
        let layout = HorizontalSplit::new();
        let rects = layout.compute_rects(parent(), 0);
        assert!(rects.is_empty());
    }

    // ---- GridLayout ----

    #[test]
    fn grid_2x2() {
        let layout = GridLayout::new(2, 2);
        let rects = layout.compute_rects(parent(), 4);
        assert_eq!(rects.len(), 4);
        // Each cell: 400x300
        for r in &rects {
            assert!((r.width - 400.0).abs() < 1.0);
            assert!((r.height - 300.0).abs() < 1.0);
        }
        // Positions
        assert!((rects[0].x - 0.0).abs() < 1.0);
        assert!((rects[0].y - 0.0).abs() < 1.0);
        assert!((rects[1].x - 400.0).abs() < 1.0);
        assert!((rects[1].y - 0.0).abs() < 1.0);
        assert!((rects[2].x - 0.0).abs() < 1.0);
        assert!((rects[2].y - 300.0).abs() < 1.0);
        assert!((rects[3].x - 400.0).abs() < 1.0);
        assert!((rects[3].y - 300.0).abs() < 1.0);
    }

    #[test]
    fn grid_2x2_with_gaps() {
        let layout = GridLayout::new(2, 2).with_gaps(10.0, 10.0);
        let rects = layout.compute_rects(parent(), 4);
        // Available width = 790, cell = 395
        // Available height = 590, cell = 295
        assert!((rects[0].width - 395.0).abs() < 1.0);
        assert!((rects[0].height - 295.0).abs() < 1.0);
        assert!((rects[1].x - 405.0).abs() < 1.0); // 0 + 395 + 10
    }

    #[test]
    fn grid_3x1() {
        let layout = GridLayout::new(3, 1);
        let rects = layout.compute_rects(parent(), 3);
        assert_eq!(rects.len(), 3);
        // Each cell: 800x200
        for r in &rects {
            assert!((r.width - 800.0).abs() < 1.0);
            assert!((r.height - 200.0).abs() < 1.0);
        }
    }

    #[test]
    fn grid_fewer_panes_than_cells() {
        let layout = GridLayout::new(2, 2);
        let rects = layout.compute_rects(parent(), 2);
        assert_eq!(rects.len(), 2);
    }

    #[test]
    fn grid_more_panes_than_cells() {
        let layout = GridLayout::new(2, 2);
        let rects = layout.compute_rects(parent(), 6);
        assert_eq!(rects.len(), 4); // clamped to 2x2
    }

    #[test]
    fn grid_zero() {
        let layout = GridLayout::new(2, 2);
        let rects = layout.compute_rects(parent(), 0);
        assert!(rects.is_empty());
    }

    #[test]
    fn grid_pane_count_hint() {
        let layout = GridLayout::new(3, 4);
        assert_eq!(layout.pane_count_hint(), 12);
    }

    // ---- VerticalStack default ----

    #[test]
    fn vertical_stack_default() {
        let layout = VerticalStack::default();
        let rects = layout.compute_rects(parent(), 2);
        assert_eq!(rects.len(), 2);
    }
}
