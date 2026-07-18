# Render Pipeline

## Pass System

The frame is divided into 12 ordered render passes. Each pass is independent
and can be individually enabled, disabled, or marked dirty.

```rust
pub enum RenderPass {
    Background = 0,    // Background fill
    Watermark = 1,     // Watermark text/logo
    Grid = 2,          // Price and time grid lines
    Session = 3,       // Market session vertical lines
    Indicator = 4,     // Indicator overlays (RSI, MACD)
    Series = 5,        // Main series (candles, lines)
    Drawing = 6,       // User drawings (lines, fibs)
    Overlay = 7,       // Content overlays
    Labels = 8,        // Labels and markers
    Crosshair = 9,     // Crosshair lines
    Tooltip = 10,      // Hover tooltip
    Debug = 11,        // Debug info
}
```

Each pass occupies a 1000-unit z-index range:
- Background: 0–999
- Grid: 2000–2999
- Series: 5000–5999
- Crosshair: 9000–9999
- Debug: 11000–11999

### PassTracker

Controls which passes execute each frame:

```rust
let mut tracker = PassTracker::new();
tracker.set_enabled(RenderPass::Debug, false);  // skip debug
tracker.mark_dirty(RenderPass::Grid);           // grid needs redraw
let passes = tracker.passes_to_execute();       // enabled AND dirty
```

---

## DrawLayer System

15 z-index layers ensure correct visual ordering across all content:

| Layer | Z Range | Content |
|-------|---------|---------|
| Background | 0–99 | Background fill |
| Watermark | 100–199 | Watermark/logo |
| Grid | 200–299 | Grid lines |
| PriceScale | 300–399 | Price axis |
| TimeScale | 400–499 | Time axis |
| Indicators | 500–599 | Indicator overlays |
| Candles | 600–699 | Candlestick bars |
| Volume | 700–799 | Volume bars |
| CustomSeries | 800–899 | Custom series |
| Drawings | 900–999 | User drawing tools |
| Crosshair | 1000–1099 | Crosshair |
| Selection | 1100–1199 | Selection highlights |
| FloatingLabels | 1200–1299 | Floating price labels |
| Tooltip | 1300–1399 | Tooltip popups |
| Cursor | 1400–1499 | Mouse cursor |

---

## Dirty Rendering

Not every frame redraws everything. The `DirtyRegionTracker` tracks which
screen regions need re-rendering per pass.

```rust
let mut tracker = DirtyRegionTracker::new(800.0, 600.0);

// Mark specific region dirty
tracker.mark_dirty(RenderPass::Series, ScreenRect::new(0.0, 0.0, 400.0, 600.0));

// Mark entire surface dirty
tracker.mark_full_dirty(RenderPass::Crosshair);

// Check before rendering
if tracker.is_dirty(RenderPass::Grid) {
    // render grid
    tracker.clear(RenderPass::Grid);
}
```

### Auto-merge

Dirty regions that overlap or are adjacent (within 1px) are automatically
merged. If merged regions exceed 50% of the surface, the entire surface
is marked dirty instead.

---

## RenderPipeline

Orchestrates the full frame:

1. Collect dirty regions from `DirtyRegionTracker`
2. For each dirty pass, generate `DrawCommand`s
3. Sort commands by z-index within each pass
4. Batch commands by primitive type
5. Submit batches to `RendererBackend`

### FrameStats

Tracks per-frame metrics: draw command count, pass execution times, GPU
submit count, dirty region count.
