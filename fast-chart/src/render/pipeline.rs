// ---------------------------------------------------------------------------
// RenderPipeline — orchestrator for draw command collection, sorting, and execution
// ---------------------------------------------------------------------------

use super::commands::DrawCommand;
use super::passes::{PassTracker, RenderPass};
use super::backend::RendererBackend;

/// A batch of draw commands for a specific pass.
#[derive(Debug, Clone)]
pub struct PassBatch {
    /// The render pass these commands belong to.
    pub pass: RenderPass,
    /// The draw commands in this batch, sorted by z-index.
    pub commands: Vec<DrawCommand>,
}

/// Rendering statistics for a frame.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Total number of draw commands submitted.
    pub total_commands: usize,
    /// Total number of batches after sorting.
    pub total_batches: usize,
    /// Number of passes that executed.
    pub passes_executed: usize,
    /// Number of passes skipped (empty).
    pub passes_skipped: usize,
}

/// Collects and orchestrates draw commands through the rendering pipeline.
///
/// # Lifecycle
///
/// ```text
/// pipeline.begin_frame();
/// pipeline.submit(cmd1);
/// pipeline.submit_all(cmds);
/// pipeline.end_frame();
/// pipeline.execute(&mut backend);
/// ```
pub struct RenderPipeline {
    /// Dirty pass tracker.
    pass_tracker: PassTracker,
    /// Collected commands for the current frame (before sorting).
    pending_commands: Vec<DrawCommand>,
    /// Sorted batches from the last frame.
    batches: Vec<PassBatch>,
    /// Stats for the last frame.
    stats: FrameStats,
}

/// Map a z-index value to the pass it belongs to.
///
/// Each pass owns a 1000-unit range. Out-of-range values go to Debug.
pub fn z_index_to_pass(z_index: i32) -> RenderPass {
    match z_index {
        0..=999 => RenderPass::Background,
        1000..=1999 => RenderPass::Watermark,
        2000..=2999 => RenderPass::Grid,
        3000..=3999 => RenderPass::Session,
        4000..=4999 => RenderPass::Indicator,
        5000..=5999 => RenderPass::Series,
        6000..=6999 => RenderPass::Drawing,
        7000..=7999 => RenderPass::Overlay,
        8000..=8999 => RenderPass::Labels,
        9000..=9999 => RenderPass::Crosshair,
        10000..=10999 => RenderPass::Tooltip,
        _ => RenderPass::Debug,
    }
}

impl RenderPipeline {
    /// Create a new pipeline. All passes start dirty.
    pub fn new() -> Self {
        Self {
            pass_tracker: PassTracker::new(),
            pending_commands: Vec::new(),
            batches: Vec::new(),
            stats: FrameStats::default(),
        }
    }

    /// Begin a new frame. Clears pending commands.
    pub fn begin_frame(&mut self) {
        self.pending_commands.clear();
    }

    /// Submit a draw command. It will be assigned to the correct pass
    /// based on its z-index during `end_frame`.
    pub fn submit(&mut self, command: DrawCommand) {
        self.pending_commands.push(command);
    }

    /// Submit multiple draw commands at once.
    pub fn submit_all(&mut self, commands: Vec<DrawCommand>) {
        self.pending_commands.extend(commands);
    }

    /// End frame: sort commands by z-index, group into pass batches.
    pub fn end_frame(&mut self) {
        // Sort all pending commands by z_index (stable sort preserves submission order within same z).
        self.pending_commands
            .sort_by_key(|cmd| cmd.z_index());

        // Group into batches per pass.
        self.batches.clear();
        let mut current_pass: Option<RenderPass> = None;

        for cmd in std::mem::take(&mut self.pending_commands) {
            let pass = z_index_to_pass(cmd.z_index());
            match &mut current_pass {
                Some(p) if *p == pass => {
                    self.batches.last_mut().expect("batch must exist when current_pass is set").commands.push(cmd);
                }
                _ => {
                    current_pass = Some(pass);
                    self.batches.push(PassBatch {
                        pass,
                        commands: vec![cmd],
                    });
                }
            }
        }

        // Update stats.
        self.stats.total_commands = self.batches.iter().map(|b| b.commands.len()).sum();
        self.stats.total_batches = self.batches.len();
    }

    /// Execute all batches through the backend.
    ///
    /// Only passes that are both enabled and dirty are executed.
    /// Clean passes are counted as skipped.
    pub fn execute(&mut self, backend: &mut dyn RendererBackend) {
        backend.begin_frame();

        let mut executed = 0usize;
        let mut skipped = 0usize;

        for batch in &self.batches {
            if self.pass_tracker.is_enabled(batch.pass) && self.pass_tracker.is_dirty(batch.pass) {
                if !batch.commands.is_empty() {
                    backend.execute(&batch.commands);
                }
                self.pass_tracker.clear_dirty(batch.pass);
                executed += 1;
            } else {
                skipped += 1;
            }
        }

        backend.end_frame();

        self.stats.passes_executed = executed;
        self.stats.passes_skipped = skipped;
    }

    /// Get the batches for the last frame.
    pub fn batches(&self) -> &[PassBatch] {
        &self.batches
    }

    /// Get frame stats.
    pub fn stats(&self) -> &FrameStats {
        &self.stats
    }

    /// Access the pass tracker (immutable).
    pub fn pass_tracker(&self) -> &PassTracker {
        &self.pass_tracker
    }

    /// Access the pass tracker (mutable).
    pub fn pass_tracker_mut(&mut self) -> &mut PassTracker {
        &mut self.pass_tracker
    }

    /// Mark all passes dirty (full redraw).
    pub fn invalidate_all(&mut self) {
        self.pass_tracker.mark_all_dirty();
    }

    /// Mark a specific pass dirty.
    pub fn invalidate_pass(&mut self, pass: RenderPass) {
        self.pass_tracker.mark_dirty(pass);
    }

    /// Reset to clean state: all passes dirty, empty batches, zero stats.
    pub fn reset(&mut self) {
        self.pass_tracker.reset();
        self.pending_commands.clear();
        self.batches.clear();
        self.stats = FrameStats::default();
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::commands::DrawCommand;

    // ---- 1. new_pipeline ----

    #[test]
    fn new_pipeline() {
        let pipeline = RenderPipeline::new();
        assert!(pipeline.pending_commands.is_empty());
        assert!(pipeline.batches.is_empty());
        assert_eq!(pipeline.stats.total_commands, 0);
        assert_eq!(pipeline.stats.total_batches, 0);
        assert_eq!(pipeline.stats.passes_executed, 0);
        assert_eq!(pipeline.stats.passes_skipped, 0);

        // All passes should be dirty initially.
        for pass in RenderPass::ALL {
            assert!(pipeline.pass_tracker.is_dirty(*pass));
        }
    }

    // ---- 2. submit_single_command ----

    #[test]
    fn submit_single_command() {
        let mut pipeline = RenderPipeline::new();
        let cmd = DrawCommand::line(0.0, 0.0, 10.0, 10.0, [1.0; 4], 1.0, 500);
        pipeline.submit(cmd);
        assert_eq!(pipeline.pending_commands.len(), 1);
    }

    // ---- 3. submit_multiple_commands ----

    #[test]
    fn submit_multiple_commands() {
        let mut pipeline = RenderPipeline::new();
        let cmds = vec![
            DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 100),
            DrawCommand::filled_rect(0.0, 0.0, 5.0, 5.0, [1.0; 4], 200),
            DrawCommand::text(0.0, 0.0, "hi", [1.0; 4], 12.0, 300),
        ];
        pipeline.submit_all(cmds);
        assert_eq!(pipeline.pending_commands.len(), 3);
    }

    // ---- 4. end_frame_sorts_by_z_index ----

    #[test]
    fn end_frame_sorts_by_z_index() {
        let mut pipeline = RenderPipeline::new();
        // Submit in reverse z-order across 3 passes.
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 9500)); // Crosshair
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 5500)); // Series
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 500));  // Background
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 800));  // Background
        pipeline.end_frame();

        // Batches sorted by pass: Background(500,800), Series(5500), Crosshair(9500).
        assert_eq!(pipeline.batches.len(), 3);
        assert_eq!(pipeline.batches[0].pass, RenderPass::Background);
        assert_eq!(pipeline.batches[0].commands.len(), 2);
        assert_eq!(pipeline.batches[1].pass, RenderPass::Series);
        assert_eq!(pipeline.batches[2].pass, RenderPass::Crosshair);
    }

    // ---- 5. end_frame_groups_by_pass ----

    #[test]
    fn end_frame_groups_by_pass() {
        let mut pipeline = RenderPipeline::new();
        // Two commands in the same pass (Grid: z 2000-2999).
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 2100));
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 2500));
        // One in a different pass.
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 5500));
        pipeline.end_frame();

        assert_eq!(pipeline.batches.len(), 2);
        assert_eq!(pipeline.batches[0].pass, RenderPass::Grid);
        assert_eq!(pipeline.batches[0].commands.len(), 2);
        assert_eq!(pipeline.batches[1].pass, RenderPass::Series);
        assert_eq!(pipeline.batches[1].commands.len(), 1);
    }

    // ---- 6. z_index_to_pass_background ----

    #[test]
    fn z_index_to_pass_background() {
        assert_eq!(z_index_to_pass(0), RenderPass::Background);
        assert_eq!(z_index_to_pass(500), RenderPass::Background);
        assert_eq!(z_index_to_pass(999), RenderPass::Background);
    }

    // ---- 7. z_index_to_pass_series ----

    #[test]
    fn z_index_to_pass_series() {
        assert_eq!(z_index_to_pass(5000), RenderPass::Series);
        assert_eq!(z_index_to_pass(5500), RenderPass::Series);
        assert_eq!(z_index_to_pass(5999), RenderPass::Series);
    }

    // ---- 8. z_index_to_pass_crosshair ----

    #[test]
    fn z_index_to_pass_crosshair() {
        assert_eq!(z_index_to_pass(9000), RenderPass::Crosshair);
        assert_eq!(z_index_to_pass(9500), RenderPass::Crosshair);
        assert_eq!(z_index_to_pass(9999), RenderPass::Crosshair);
    }

    // ---- 9. execute_skips_clean_passes ----

    #[test]
    fn execute_skips_clean_passes() {
        struct RecordingBackend {
            calls: Vec<String>,
        }
        impl RendererBackend for RecordingBackend {
            fn execute(&mut self, commands: &[DrawCommand]) {
                self.calls.push(format!("exec:{}", commands.len()));
            }
            fn resize(&mut self, _w: u32, _h: u32) {}
            fn set_clip(&mut self, _r: super::super::series_renderer::Rect) {}
            fn clear_clip(&mut self) {}
            fn clear(&mut self, _c: [f32; 4]) {}
            fn width(&self) -> u32 { 800 }
            fn height(&self) -> u32 { 600 }
        }

        let mut pipeline = RenderPipeline::new();
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 2500));
        pipeline.end_frame();

        // Mark Grid as clean.
        pipeline.pass_tracker_mut().clear_dirty(RenderPass::Grid);

        let mut backend = RecordingBackend { calls: Vec::new() };
        pipeline.execute(&mut backend);

        // Grid pass should be skipped — no execute call.
        assert!(backend.calls.is_empty());
        assert_eq!(pipeline.stats().passes_skipped, 1);
    }

    // ---- 10. execute_runs_dirty_passes ----

    #[test]
    fn execute_runs_dirty_passes() {
        struct RecordingBackend {
            calls: Vec<String>,
        }
        impl RendererBackend for RecordingBackend {
            fn execute(&mut self, commands: &[DrawCommand]) {
                self.calls.push(format!("exec:{}", commands.len()));
            }
            fn resize(&mut self, _w: u32, _h: u32) {}
            fn set_clip(&mut self, _r: super::super::series_renderer::Rect) {}
            fn clear_clip(&mut self) {}
            fn clear(&mut self, _c: [f32; 4]) {}
            fn width(&self) -> u32 { 800 }
            fn height(&self) -> u32 { 600 }
        }

        let mut pipeline = RenderPipeline::new();
        // Submit to Grid (dirty by default).
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 2500));
        pipeline.end_frame();

        let mut backend = RecordingBackend { calls: Vec::new() };
        pipeline.execute(&mut backend);

        assert_eq!(backend.calls.len(), 1);
        assert_eq!(pipeline.stats().passes_executed, 1);

        // After execution, Grid should now be clean.
        assert!(!pipeline.pass_tracker().is_dirty(RenderPass::Grid));
    }

    // ---- 11. invalidate_all ----

    #[test]
    fn invalidate_all() {
        let mut pipeline = RenderPipeline::new();
        // Clear all dirty flags.
        for pass in RenderPass::ALL {
            pipeline.pass_tracker_mut().clear_dirty(*pass);
        }
        assert!(!pipeline.pass_tracker().is_dirty(RenderPass::Grid));
        assert!(!pipeline.pass_tracker().is_dirty(RenderPass::Series));

        pipeline.invalidate_all();

        for pass in RenderPass::ALL {
            assert!(pipeline.pass_tracker().is_dirty(*pass));
        }
    }

    // ---- 12. frame_stats ----

    #[test]
    fn frame_stats() {
        let mut pipeline = RenderPipeline::new();
        // Submit 4 commands across 3 passes.
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 500));  // Background
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 2500)); // Grid
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 2100)); // Grid
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 5500)); // Series
        pipeline.end_frame();

        assert_eq!(pipeline.stats().total_commands, 4);
        assert_eq!(pipeline.stats().total_batches, 3);

        // Execute — all 3 passes are dirty by default.
        struct NoopBackend;
        impl RendererBackend for NoopBackend {
            fn execute(&mut self, _: &[DrawCommand]) {}
            fn resize(&mut self, _: u32, _: u32) {}
            fn set_clip(&mut self, _: super::super::series_renderer::Rect) {}
            fn clear_clip(&mut self) {}
            fn clear(&mut self, _: [f32; 4]) {}
            fn width(&self) -> u32 { 800 }
            fn height(&self) -> u32 { 600 }
        }

        pipeline.execute(&mut NoopBackend);
        assert_eq!(pipeline.stats().passes_executed, 3);
        assert_eq!(pipeline.stats().passes_skipped, 0);
    }

    // ---- Bonus: negative z_index goes to Debug ----

    #[test]
    fn z_index_negative_goes_to_debug() {
        assert_eq!(z_index_to_pass(-1), RenderPass::Debug);
        assert_eq!(z_index_to_pass(-500), RenderPass::Debug);
    }

    // ---- Bonus: z_index beyond Debug range goes to Debug ----

    #[test]
    fn z_index_above_range_goes_to_debug() {
        assert_eq!(z_index_to_pass(12000), RenderPass::Debug);
        assert_eq!(z_index_to_pass(99999), RenderPass::Debug);
    }

    // ---- Bonus: begin_frame clears pending commands ----

    #[test]
    fn begin_frame_clears_pending() {
        let mut pipeline = RenderPipeline::new();
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 500));
        assert_eq!(pipeline.pending_commands.len(), 1);

        pipeline.begin_frame();
        assert!(pipeline.pending_commands.is_empty());
    }

    // ---- Bonus: reset clears everything ----

    #[test]
    fn reset_clears_state() {
        let mut pipeline = RenderPipeline::new();
        pipeline.submit(DrawCommand::line(0.0, 0.0, 1.0, 1.0, [1.0; 4], 1.0, 500));
        pipeline.end_frame();
        assert!(!pipeline.batches.is_empty());

        pipeline.reset();
        assert!(pipeline.batches.is_empty());
        assert!(pipeline.pending_commands.is_empty());
        assert_eq!(pipeline.stats.total_commands, 0);
        // All passes should be dirty again after reset.
        for pass in RenderPass::ALL {
            assert!(pipeline.pass_tracker().is_dirty(*pass));
        }
    }

    // ---- Bonus: Default impl ----

    #[test]
    fn default_impl() {
        let pipeline = RenderPipeline::default();
        assert!(pipeline.batches.is_empty());
    }

    // ---- Bonus: empty end_frame produces no batches ----

    #[test]
    fn empty_end_frame() {
        let mut pipeline = RenderPipeline::new();
        pipeline.end_frame();
        assert!(pipeline.batches.is_empty());
        assert_eq!(pipeline.stats.total_commands, 0);
        assert_eq!(pipeline.stats.total_batches, 0);
    }
}
