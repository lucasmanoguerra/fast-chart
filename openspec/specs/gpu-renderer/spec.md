# GPU Renderer Specification

## Purpose

Define the GPU-accelerated rendering layer for the trading chart. Uses wgpu for surface management and shader pipelines, glyphon for text rendering, and a uniform-based projection system for zoom/pan.

## Requirements

### Requirement: wgpu surface and swapchain

The system MUST create a wgpu surface from a winit window handle. It SHALL manage the swapchain to match the window size, recreating it on resize.

#### Scenario: Surface creation succeeds with winit window

- GIVEN a valid winit window handle and wgpu adapter
- WHEN the renderer initializes the surface
- THEN a valid wgpu surface and swapchain are created

#### Scenario: Swapchain recreation on window resize

- GIVEN an active swapchain with 1920×1080 resolution
- WHEN the window resizes to 1280×720
- THEN the swapchain SHALL be recreated at the new resolution

### Requirement: GPU pipelines

The system MUST provide WGSL pipelines for: line series, candle sticks, bar fills, area fills, and grid lines. Each pipeline SHALL be a separate `RenderPipeline` with its own shader module.

#### Scenario: All pipelines compile

- GIVEN the wgpu device
- WHEN each pipeline shader module is compiled
- THEN all five pipelines (line, candle, bar, area, grid) SHALL compile without errors

#### Scenario: Pipeline selection per series type

- GIVEN a line series and a candle series in the same frame
- WHEN the render pass executes
- THEN each series renders using its respective pipeline

### Requirement: Glyphon text rendering

The system MUST render text using the glyphon library for: axis labels, crosshair tooltip values, and pane titles. Glyphon SHALL be the primary text renderer.

#### Scenario: Axis labels render correctly

- GIVEN a price axis with range 100–110
- WHEN labels at every 2-unit interval are rendered
- THEN text "100", "102", "104", "106", "108", "110" appear at correct y-positions

#### Scenario: Crosshair tooltip text

- GIVEN the crosshair at time 12:00, price 105.50
- WHEN the tooltip is rendered
- THEN formatted text "12:00 | 105.50" appears near the crosshair position

### Requirement: Multi-layer compositing

All render layers SHALL be composited in a single render pass. Draw order MUST be: grid → price series → indicator overlays → crosshair → HUD text.

#### Scenario: Correct draw order

- GIVEN all layers are visible
- WHEN the single render pass executes
- THEN grid draws first, then series, then indicators, then crosshair, then HUD on top

#### Scenario: Layer visibility toggle

- GIVEN the grid layer is toggled off
- WHEN the render pass executes
- THEN grid lines are skipped but all other layers render normally

### Requirement: Projection matrix uniform

Zoom and pan MUST be applied by updating a projection matrix uniform buffer. The vertex buffers SHALL NOT be rebuilt on viewport changes.

#### Scenario: Zoom updates uniform only

- GIVEN a zoom operation on 100k vertices
- WHEN the zoom level changes
- THEN the uniform buffer is updated and vertex buffers are unchanged

#### Scenario: Pan with vertex stability

- GIVEN a pan operation
- WHEN the viewport shifts by 50 pixels
- THEN only the projection uniform changes; vertex geometry is preserved

### Requirement: Double/triple buffering

The renderer SHALL use double or triple buffering for smooth animation. The preferred mode SHALL be triple buffering when the swapchain supports it.

#### Scenario: Triple buffering preferred

- GIVEN a swapchain that supports triple buffering
- WHEN the renderer configures the swapchain
- THEN the present mode SHALL be `Immediate` or `Mailbox` with triple buffering

#### Scenario: Fallback to double buffering

- GIVEN a swapchain that does NOT support triple buffering
- WHEN the renderer configures the swapchain
- THEN it SHALL fall back to `Fifo` (double buffering, vsync)

### Requirement: Frame timing metrics

The renderer SHOULD expose frame timing: frame duration, GPU wait time, and FPS. Metrics SHALL be measured per frame using `std::time::Instant`.

#### Scenario: FPS counter accessible

- GIVEN 60 frames rendered at 16.67ms each
- WHEN the frame timing is queried
- THEN FPS SHALL report approximately 60

#### Scenario: First frame metric

- GIVEN the first frame has just rendered
- WHEN frame duration is queried
- THEN it SHALL report a valid positive duration (≥ 0ms)
