use std::fmt;

/// Render pass identifier. Defines the order and grouping of draw commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RenderPass {
    /// Background fill (lowest priority).
    Background = 0,
    /// Watermark text behind content.
    Watermark = 1,
    /// Grid lines (price and time).
    Grid = 2,
    /// Session vertical lines (market open/close).
    Session = 3,
    /// Indicator overlays (RSI, MACD, etc.).
    Indicator = 4,
    /// Main series (candles, lines, etc.).
    Series = 5,
    /// User drawings (lines, fibs, etc.).
    Drawing = 6,
    /// Overlays on top of content.
    Overlay = 7,
    /// Labels and markers.
    Labels = 8,
    /// Crosshair lines.
    Crosshair = 9,
    /// Tooltip on hover.
    Tooltip = 10,
    /// Debug info (highest priority).
    Debug = 11,
}

impl RenderPass {
    /// All passes in execution order.
    pub const ALL: &'static [RenderPass] = &[
        RenderPass::Background,
        RenderPass::Watermark,
        RenderPass::Grid,
        RenderPass::Session,
        RenderPass::Indicator,
        RenderPass::Series,
        RenderPass::Drawing,
        RenderPass::Overlay,
        RenderPass::Labels,
        RenderPass::Crosshair,
        RenderPass::Tooltip,
        RenderPass::Debug,
    ];

    /// Get the z-index range for this pass.
    /// Each pass occupies a 1000-unit z-index range.
    pub fn z_range(&self) -> (i32, i32) {
        let base = (*self as i32) * 1000;
        (base, base + 999)
    }

    /// Get the pass name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Background => "Background",
            Self::Watermark => "Watermark",
            Self::Grid => "Grid",
            Self::Session => "Session",
            Self::Indicator => "Indicator",
            Self::Series => "Series",
            Self::Drawing => "Drawing",
            Self::Overlay => "Overlay",
            Self::Labels => "Labels",
            Self::Crosshair => "Crosshair",
            Self::Tooltip => "Tooltip",
            Self::Debug => "Debug",
        }
    }

    /// Whether this pass can be skipped (for optimization).
    pub fn is_skippable(&self) -> bool {
        matches!(self, Self::Session | Self::Watermark | Self::Debug)
    }
}

impl fmt::Display for RenderPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Tracks which passes need execution.
#[derive(Debug, Clone)]
pub struct PassTracker {
    /// Which passes are enabled.
    enabled: [bool; 12],
    /// Which passes are dirty (need re-execution).
    dirty: [bool; 12],
}

impl PassTracker {
    pub fn new() -> Self {
        Self {
            enabled: [true; 12],
            dirty: [true; 12],
        }
    }

    /// Enable/disable a pass.
    pub fn set_enabled(&mut self, pass: RenderPass, enabled: bool) {
        self.enabled[pass as usize] = enabled;
    }

    /// Check if a pass is enabled.
    pub fn is_enabled(&self, pass: RenderPass) -> bool {
        self.enabled[pass as usize]
    }

    /// Mark a pass as dirty (needs re-execution).
    pub fn mark_dirty(&mut self, pass: RenderPass) {
        self.dirty[pass as usize] = true;
    }

    /// Mark all passes as dirty.
    pub fn mark_all_dirty(&mut self) {
        self.dirty = [true; 12];
    }

    /// Check if a pass is dirty.
    pub fn is_dirty(&self, pass: RenderPass) -> bool {
        self.dirty[pass as usize]
    }

    /// Clear dirty flag after execution.
    pub fn clear_dirty(&mut self, pass: RenderPass) {
        self.dirty[pass as usize] = false;
    }

    /// Get all passes that need execution (enabled AND dirty).
    pub fn passes_to_execute(&self) -> Vec<RenderPass> {
        RenderPass::ALL
            .iter()
            .copied()
            .filter(|&p| self.is_enabled(p) && self.is_dirty(p))
            .collect()
    }

    /// Reset all to initial state (all enabled, all dirty).
    pub fn reset(&mut self) {
        self.enabled = [true; 12];
        self.dirty = [true; 12];
    }
}

impl Default for PassTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_order() {
        for (i, pass) in RenderPass::ALL.iter().enumerate() {
            assert_eq!(*pass as usize, i);
        }
    }

    #[test]
    fn pass_z_range() {
        let bg = RenderPass::Background.z_range();
        assert_eq!(bg, (0, 999));

        let series = RenderPass::Series.z_range();
        assert_eq!(series, (5000, 5999));

        let debug = RenderPass::Debug.z_range();
        assert_eq!(debug, (11000, 11999));
    }

    #[test]
    fn pass_names() {
        assert_eq!(RenderPass::Background.name(), "Background");
        assert_eq!(RenderPass::Grid.name(), "Grid");
        assert_eq!(RenderPass::Series.name(), "Series");
        assert_eq!(RenderPass::Debug.name(), "Debug");
    }

    #[test]
    fn pass_display() {
        assert_eq!(RenderPass::Crosshair.to_string(), "Crosshair");
        assert_eq!(RenderPass::Tooltip.to_string(), "Tooltip");
    }

    #[test]
    fn pass_skippable() {
        assert!(RenderPass::Session.is_skippable());
        assert!(RenderPass::Watermark.is_skippable());
        assert!(RenderPass::Debug.is_skippable());
    }

    #[test]
    fn pass_is_not_skippable() {
        assert!(!RenderPass::Background.is_skippable());
        assert!(!RenderPass::Grid.is_skippable());
        assert!(!RenderPass::Series.is_skippable());
    }

    #[test]
    fn pass_tracker_new() {
        let tracker = PassTracker::new();
        for pass in RenderPass::ALL {
            assert!(tracker.is_enabled(*pass));
            assert!(tracker.is_dirty(*pass));
        }
    }

    #[test]
    fn pass_tracker_disable() {
        let mut tracker = PassTracker::new();
        tracker.set_enabled(RenderPass::Grid, false);
        assert!(!tracker.is_enabled(RenderPass::Grid));
        assert!(tracker.is_enabled(RenderPass::Series));
    }

    #[test]
    fn pass_tracker_mark_dirty() {
        let mut tracker = PassTracker::new();
        tracker.clear_dirty(RenderPass::Indicator);
        assert!(!tracker.is_dirty(RenderPass::Indicator));
        tracker.mark_dirty(RenderPass::Indicator);
        assert!(tracker.is_dirty(RenderPass::Indicator));
    }

    #[test]
    fn pass_tracker_clear_dirty() {
        let mut tracker = PassTracker::new();
        assert!(tracker.is_dirty(RenderPass::Drawing));
        tracker.clear_dirty(RenderPass::Drawing);
        assert!(!tracker.is_dirty(RenderPass::Drawing));
    }

    #[test]
    fn pass_tracker_passes_to_execute() {
        let mut tracker = PassTracker::new();
        tracker.clear_dirty(RenderPass::Grid);
        tracker.clear_dirty(RenderPass::Session);
        tracker.set_enabled(RenderPass::Watermark, false);

        let passes = tracker.passes_to_execute();
        assert!(!passes.contains(&RenderPass::Grid));
        assert!(!passes.contains(&RenderPass::Watermark));
        assert!(!passes.contains(&RenderPass::Session));
        assert!(passes.contains(&RenderPass::Background));
        assert!(passes.contains(&RenderPass::Series));
    }

    #[test]
    fn pass_tracker_mark_all_dirty() {
        let mut tracker = PassTracker::new();
        for pass in RenderPass::ALL {
            tracker.clear_dirty(*pass);
        }
        tracker.mark_all_dirty();
        for pass in RenderPass::ALL {
            assert!(tracker.is_dirty(*pass));
        }
    }
}
