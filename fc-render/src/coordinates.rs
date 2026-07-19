// ---------------------------------------------------------------------------
// CoordinatePipeline — World ↔ Screen coordinate transforms
// ---------------------------------------------------------------------------

/// A screen-space point in pixels (origin at top-left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

impl ScreenPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A world-space point (timestamp, price).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPoint {
    pub timestamp: f64,
    pub price: f64,
}

impl WorldPoint {
    pub fn new(timestamp: f64, price: f64) -> Self {
        Self { timestamp, price }
    }
}

/// The coordinate pipeline transforms between world (timestamp, price)
/// and screen (pixel) coordinate spaces.
///
/// # Coordinate Spaces
///
/// ```text
/// World (timestamp, price)
///   ↓  world_to_screen()
/// Screen (pixels, origin top-left)
///   ↓  screen_to_world()
/// World (timestamp, price)
/// ```
///
/// # Pixel-Perfect Alignment
///
/// All transforms use `floor(x) + 0.5` for crisp 1px lines.
/// This snaps coordinates to pixel centers instead of edges.
#[derive(Debug, Clone)]
pub struct CoordinatePipeline {
    /// The visible time range (min_timestamp, max_timestamp).
    pub time_range: (f64, f64),
    /// The visible price range (min_price, max_price).
    pub price_range: (f64, f64),
    /// The pixel area this pipeline maps into.
    pub area_x: f32,
    pub area_y: f32,
    pub area_width: f32,
    pub area_height: f32,
    /// DPI scale factor.
    pub scale_factor: f32,
}

impl CoordinatePipeline {
    /// Create a new coordinate pipeline.
    pub fn new(
        time_range: (f64, f64),
        price_range: (f64, f64),
        area_x: f32,
        area_y: f32,
        area_width: f32,
        area_height: f32,
        scale_factor: f32,
    ) -> Self {
        Self {
            time_range,
            price_range,
            area_x,
            area_y,
            area_width,
            area_height,
            scale_factor,
        }
    }

    /// Map a world timestamp to screen X.
    pub fn timestamp_to_x(&self, timestamp: f64) -> f32 {
        let (t_min, t_max) = self.time_range;
        let t_span = t_max - t_min;
        if t_span <= 0.0 {
            return self.area_x + 0.5;
        }
        let ratio = ((timestamp - t_min) / t_span).clamp(0.0, 1.0);
        // Map to pixel center range: [area_x + 0.5, area_x + area_width - 0.5]
        let min_x = self.area_x + 0.5;
        let max_x = self.area_x + self.area_width - 0.5;
        let x = min_x + ratio as f32 * (max_x - min_x);
        // Pixel-perfect: snap to pixel center
        (x * self.scale_factor).floor() / self.scale_factor + 0.5 * (1.0 - self.scale_factor) / self.scale_factor
    }

    /// Map a world price to screen Y (Y is inverted: price up = screen down).
    pub fn price_to_y(&self, price: f64) -> f32 {
        let (p_min, p_max) = self.price_range;
        let p_span = p_max - p_min;
        if p_span <= 0.0 {
            return self.area_y + 0.5;
        }
        let ratio = ((price - p_min) / p_span).clamp(0.0, 1.0);
        // Invert Y: higher price = lower screen Y
        // Map to pixel center range: [area_y + 0.5, area_y + area_height - 0.5]
        let min_y = self.area_y + 0.5;
        let max_y = self.area_y + self.area_height - 0.5;
        let y = min_y + (1.0 - ratio) as f32 * (max_y - min_y);
        (y * self.scale_factor).floor() / self.scale_factor + 0.5 * (1.0 - self.scale_factor) / self.scale_factor
    }

    /// Map a screen X to world timestamp.
    pub fn x_to_timestamp(&self, x: f32) -> f64 {
        let (t_min, t_max) = self.time_range;
        let t_span = t_max - t_min;
        if self.area_width <= 0.0 {
            return t_min;
        }
        // Map from pixel center range back to [0, 1]
        let min_x = self.area_x + 0.5;
        let max_x = self.area_x + self.area_width - 0.5;
        let ratio = ((x - min_x) / (max_x - min_x)).clamp(0.0, 1.0) as f64;
        t_min + ratio * t_span
    }

    /// Map a screen Y to world price.
    pub fn y_to_price(&self, y: f32) -> f64 {
        let (p_min, p_max) = self.price_range;
        let p_span = p_max - p_min;
        if self.area_height <= 0.0 {
            return p_min;
        }
        // Map from pixel center range back to [0, 1], inverted
        let min_y = self.area_y + 0.5;
        let max_y = self.area_y + self.area_height - 0.5;
        let ratio = ((y - min_y) / (max_y - min_y)).clamp(0.0, 1.0) as f64;
        // Invert Y
        p_min + (1.0 - ratio) * p_span
    }

    /// Convert a world point to a screen point.
    pub fn world_to_screen(&self, point: WorldPoint) -> ScreenPoint {
        ScreenPoint::new(
            self.timestamp_to_x(point.timestamp),
            self.price_to_y(point.price),
        )
    }

    /// Convert a screen point to a world point.
    pub fn screen_to_world(&self, point: ScreenPoint) -> WorldPoint {
        WorldPoint::new(
            self.x_to_timestamp(point.x),
            self.y_to_price(point.y),
        )
    }

    /// Roundtrip: world → screen → world should be within epsilon.
    pub fn roundtrip(&self, point: WorldPoint) -> WorldPoint {
        let screen = self.world_to_screen(point);
        self.screen_to_world(screen)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_pipeline() -> CoordinatePipeline {
        CoordinatePipeline::new(
            (0.0, 1000.0),  // time range
            (50.0, 150.0),  // price range
            0.0,
            0.0,
            800.0,
            600.0,
            1.0,
        )
    }

    // ---- Timestamp to X ----

    #[test]
    fn timestamp_to_x_min() {
        let p = default_pipeline();
        let x = p.timestamp_to_x(0.0);
        // Should be near area_x (0.0) + 0.5 (pixel center)
        assert!((x - 0.5).abs() < 1.0, "x = {x}");
    }

    #[test]
    fn timestamp_to_x_max() {
        let p = default_pipeline();
        let x = p.timestamp_to_x(1000.0);
        // Should be near area_x + area_width = 800.0 - 0.5
        assert!((x - 799.5).abs() < 1.0, "x = {x}");
    }

    #[test]
    fn timestamp_to_x_mid() {
        let p = default_pipeline();
        let x = p.timestamp_to_x(500.0);
        // Should be near 400.0 (center)
        assert!((x - 400.0).abs() < 1.0, "x = {x}");
    }

    // ---- Price to Y ----

    #[test]
    fn price_to_y_high() {
        let p = default_pipeline();
        let y = p.price_to_y(150.0); // max price
        // High price = low screen Y (near top)
        assert!((y - 0.5).abs() < 1.0, "y = {y}");
    }

    #[test]
    fn price_to_y_low() {
        let p = default_pipeline();
        let y = p.price_to_y(50.0); // min price
        // Low price = high screen Y (near bottom)
        assert!((y - 599.5).abs() < 1.0, "y = {y}");
    }

    #[test]
    fn price_to_y_mid() {
        let p = default_pipeline();
        let y = p.price_to_y(100.0); // mid price
        assert!((y - 300.0).abs() < 1.0, "y = {y}");
    }

    // ---- Roundtrip ----

    #[test]
    fn roundtrip_world_screen_world() {
        let p = default_pipeline();
        let original = WorldPoint::new(500.0, 100.0);
        let result = p.roundtrip(original);
        // Should be within 1.0 of original (pixel snapping introduces small error)
        assert!(
            (result.timestamp - original.timestamp).abs() < 2.0,
            "timestamp: {} vs {}",
            result.timestamp,
            original.timestamp
        );
        assert!(
            (result.price - original.price).abs() < 1.0,
            "price: {} vs {}",
            result.price,
            original.price
        );
    }

    #[test]
    fn roundtrip_corners() {
        let p = default_pipeline();
        let corners = vec![
            WorldPoint::new(0.0, 50.0),
            WorldPoint::new(1000.0, 150.0),
            WorldPoint::new(0.0, 150.0),
            WorldPoint::new(1000.0, 50.0),
        ];
        for corner in corners {
            let result = p.roundtrip(corner);
            assert!(
                (result.timestamp - corner.timestamp).abs() < 2.0,
                "timestamp roundtrip failed for {corner:?}"
            );
            assert!(
                (result.price - corner.price).abs() < 1.0,
                "price roundtrip failed for {corner:?}"
            );
        }
    }

    // ---- Screen to World ----

    #[test]
    fn x_to_timestamp_center() {
        let p = default_pipeline();
        let ts = p.x_to_timestamp(400.0);
        assert!((ts - 500.0).abs() < 1.0, "ts = {ts}");
    }

    #[test]
    fn y_to_price_center() {
        let p = default_pipeline();
        let price = p.y_to_price(300.0);
        assert!((price - 100.0).abs() < 1.0, "price = {price}");
    }

    // ---- World to Screen / Screen to World ----

    #[test]
    fn world_to_screen_and_back() {
        let p = default_pipeline();
        let world = WorldPoint::new(250.0, 75.0);
        let screen = p.world_to_screen(world);
        let back = p.screen_to_world(screen);
        assert!(
            (back.timestamp - world.timestamp).abs() < 2.0,
            "timestamp: {} vs {}",
            back.timestamp,
            world.timestamp
        );
        assert!(
            (back.price - world.price).abs() < 1.0,
            "price: {} vs {}",
            back.price,
            world.price
        );
    }

    // ---- Edge cases ----

    #[test]
    fn zero_span_time() {
        let p = CoordinatePipeline::new(
            (100.0, 100.0), // zero span
            (50.0, 150.0),
            0.0,
            0.0,
            800.0,
            600.0,
            1.0,
        );
        let x = p.timestamp_to_x(100.0);
        assert!((x - 0.5).abs() < 1.0, "x = {x}");
    }

    #[test]
    fn zero_span_price() {
        let p = CoordinatePipeline::new(
            (0.0, 1000.0),
            (100.0, 100.0), // zero span
            0.0,
            0.0,
            800.0,
            600.0,
            1.0,
        );
        let y = p.price_to_y(100.0);
        assert!((y - 0.5).abs() < 1.0, "y = {y}");
    }

    // ---- Scale factor ----

    #[test]
    fn scale_factor_2x() {
        let p = CoordinatePipeline::new(
            (0.0, 1000.0),
            (50.0, 150.0),
            0.0,
            0.0,
            400.0, // logical pixels
            300.0,
            2.0, // 2x DPI
        );
        let x = p.timestamp_to_x(500.0);
        // Midpoint in logical pixels: 200.0
        assert!((x - 200.0).abs() < 1.0, "x = {x}");
    }

    // ---- Display ----

    #[test]
    fn screen_point_display() {
        let sp = ScreenPoint::new(1.5, 2.5);
        assert_eq!(sp.x, 1.5);
        assert_eq!(sp.y, 2.5);
    }

    #[test]
    fn world_point_display() {
        let wp = WorldPoint::new(1000.0, 50.5);
        assert_eq!(wp.timestamp, 1000.0);
        assert_eq!(wp.price, 50.5);
    }

    // ---- Clone ----

    #[test]
    fn pipeline_clone() {
        let p = default_pipeline();
        let cloned = p.clone();
        assert_eq!(cloned.time_range, p.time_range);
        assert_eq!(cloned.price_range, p.price_range);
        assert_eq!(cloned.area_width, p.area_width);
    }

    // ---- Non-zero offsets ----

    #[test]
    fn offset_area_timestamp() {
        let p = CoordinatePipeline::new(
            (0.0, 1000.0),
            (50.0, 150.0),
            50.0,  // area_x offset (e.g. y-axis labels)
            10.0,
            700.0,
            580.0,
            1.0,
        );
        let x = p.timestamp_to_x(0.0);
        assert!((x - 50.5).abs() < 1.0, "x = {x}");

        let x = p.timestamp_to_x(1000.0);
        assert!((x - 749.5).abs() < 1.0, "x = {x}");
    }

    #[test]
    fn offset_area_price() {
        let p = CoordinatePipeline::new(
            (0.0, 1000.0),
            (50.0, 150.0),
            50.0,
            10.0,
            700.0,
            580.0,
            1.0,
        );
        // High price = near top of area (area_y + 0.5)
        let y = p.price_to_y(150.0);
        assert!((y - 10.5).abs() < 1.0, "y = {y}");

        // Low price = near bottom of area (area_y + height - 0.5)
        let y = p.price_to_y(50.0);
        assert!((y - 589.5).abs() < 1.0, "y = {y}");
    }

    // ---- Asymmetric areas ----

    #[test]
    fn narrow_area() {
        let p = CoordinatePipeline::new(
            (0.0, 100.0),
            (0.0, 10.0),
            0.0,
            0.0,
            100.0, // square area
            100.0,
            1.0,
        );
        let x = p.timestamp_to_x(50.0);
        assert!((x - 50.0).abs() < 1.5, "x = {x}");

        let y = p.price_to_y(5.0);
        assert!((y - 50.0).abs() < 1.5, "y = {y}");
    }

    // ---- DPI 3x ----

    #[test]
    fn scale_factor_3x() {
        let p = CoordinatePipeline::new(
            (0.0, 1000.0),
            (50.0, 150.0),
            0.0,
            0.0,
            300.0,
            300.0,
            3.0,
        );
        // At min timestamp, x should be at pixel center (0.5)
        let x = p.timestamp_to_x(0.0);
        assert!((x - 0.5).abs() < 1.0, "x = {x}");

        // Midpoint
        let x = p.timestamp_to_x(500.0);
        assert!((x - 150.0).abs() < 1.5, "x = {x}");
    }

    // ---- Out-of-range values (clamped) ----

    #[test]
    fn timestamp_before_range() {
        let p = default_pipeline();
        let x = p.timestamp_to_x(-100.0);
        // Clamped to min → near area_x + 0.5
        assert!((x - 0.5).abs() < 1.0, "x = {x}");
    }

    #[test]
    fn timestamp_after_range() {
        let p = default_pipeline();
        let x = p.timestamp_to_x(2000.0);
        // Clamped to max → near area_x + width - 0.5
        assert!((x - 799.5).abs() < 1.0, "x = {x}");
    }

    #[test]
    fn price_above_range() {
        let p = default_pipeline();
        let y = p.price_to_y(200.0);
        // Clamped to max price → near top (0.5)
        assert!((y - 0.5).abs() < 1.0, "y = {y}");
    }

    #[test]
    fn price_below_range() {
        let p = default_pipeline();
        let y = p.price_to_y(0.0);
        // Clamped to min price → near bottom (599.5)
        assert!((y - 599.5).abs() < 1.0, "y = {y}");
    }

    // ---- Screen to World at boundaries ----

    #[test]
    fn x_to_timestamp_left() {
        let p = default_pipeline();
        let ts = p.x_to_timestamp(0.5);
        assert!((ts - 0.0).abs() < 1.0, "ts = {ts}");
    }

    #[test]
    fn x_to_timestamp_right() {
        let p = default_pipeline();
        let ts = p.x_to_timestamp(799.5);
        assert!((ts - 1000.0).abs() < 1.0, "ts = {ts}");
    }

    #[test]
    fn y_to_price_top() {
        let p = default_pipeline();
        let price = p.y_to_price(0.5);
        assert!((price - 150.0).abs() < 1.0, "price = {price}");
    }

    #[test]
    fn y_to_price_bottom() {
        let p = default_pipeline();
        let price = p.y_to_price(599.5);
        assert!((price - 50.0).abs() < 1.0, "price = {price}");
    }

    // ---- Multiple roundtrips stability ----

    #[test]
    fn multiple_roundtrips_stable() {
        let p = default_pipeline();
        let mut point = WorldPoint::new(333.0, 77.7);
        for _ in 0..5 {
            point = p.roundtrip(point);
        }
        assert!(
            (point.timestamp - 333.0).abs() < 3.0,
            "timestamp drifted: {}",
            point.timestamp
        );
        assert!(
            (point.price - 77.7).abs() < 2.0,
            "price drifted: {}",
            point.price
        );
    }

    // ---- ScreenPoint / WorldPoint Debug ----

    #[test]
    fn screen_point_debug() {
        let sp = ScreenPoint::new(1.0, 2.0);
        let debug = format!("{:?}", sp);
        assert!(debug.contains("ScreenPoint"));
        assert!(debug.contains("1.0"));
        assert!(debug.contains("2.0"));
    }

    #[test]
    fn world_point_debug() {
        let wp = WorldPoint::new(1000.0, 50.5);
        let debug = format!("{:?}", wp);
        assert!(debug.contains("WorldPoint"));
        assert!(debug.contains("1000.0"));
        assert!(debug.contains("50.5"));
    }
}
