# Coordinate System

## Coordinate Spaces

```
World (timestamp, price)
  ↓  world_to_screen()
Screen (pixels, origin top-left)
  ↓  screen_to_world()
World (timestamp, price)
```

---

## CoordinatePipeline

Transforms between world (timestamp, price) and screen (pixel) coordinate
spaces with pixel-perfect alignment.

```rust
pub struct CoordinatePipeline {
    pub time_range: (f64, f64),      // visible timestamp range
    pub price_range: (f64, f64),     // visible price range
    pub area_x: f32,                 // pixel area offset
    pub area_y: f32,
    pub area_width: f32,             // pixel area dimensions
    pub area_height: f32,
    pub scale_factor: f32,           // DPI (1.0 = no scaling)
}
```

### Forward Transform (World → Screen)

```rust
let pipeline = CoordinatePipeline::new(
    (0.0, 1000.0),   // time range
    (50.0, 150.0),   // price range
    0.0, 0.0,        // area origin
    800.0, 600.0,    // area size
    1.0,             // scale factor
);

let screen = pipeline.world_to_screen(WorldPoint::new(500.0, 100.0));
// screen.x ≈ 400.0 (center)
// screen.y ≈ 300.0 (center, Y inverted)
```

### Inverse Transform (Screen → World)

```rust
let world = pipeline.screen_to_world(ScreenPoint::new(400.0, 300.0));
// world.timestamp ≈ 500.0
// world.price ≈ 100.0
```

### Roundtrip Stability

```rust
let result = pipeline.roundtrip(original);
// result ≈ original (within pixel-snapping tolerance)
```

---

## Pixel-Perfect Alignment

All transforms snap to pixel centers using `floor(x) + 0.5`:

```rust
// Inside timestamp_to_x() / price_to_y():
let x = min_x + ratio * (max_x - min_x);
(x * scale_factor).floor() / scale_factor + 0.5 * (1.0 - scale_factor) / scale_factor
```

This ensures 1px lines render crisp on both standard and HiDPI displays.

---

## Y-Axis Inversion

Price increases upward on screen but Y increases downward in pixel space.
The pipeline handles this inversion automatically:

- High price → low screen Y (near top)
- Low price → high screen Y (near bottom)

---

## DPI Support

The `scale_factor` field handles HiDPI displays:
- `1.0` — Standard display
- `2.0` — Retina / HiDPI
- `3.0` — Ultra-high DPI

Pixel snapping adapts to the scale factor to maintain crisp rendering.
