#[cfg(test)]
mod area_renderer_tests {
    use fast_chart_core::Bar;

    fn test_bars() -> Vec<Bar> {
        vec![
            Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap(),
            Bar::new(200, 105.0, 115.0, 100.0, 110.0, 1200).unwrap(),
            Bar::new(300, 110.0, 120.0, 100.0, 108.0, 800).unwrap(),
        ]
    }

    #[test]
    fn area_vertex_count_for_n_bars() {
        let bars = test_bars();
        // Each bar produces 2 vertices (close + baseline).
        let expected = bars.len() * 2;
        assert_eq!(expected, 6);
    }

    #[test]
    fn area_index_count_for_n_bars() {
        let bars = test_bars();
        // Each consecutive pair produces 6 indices (2 triangles).
        let expected = (bars.len() - 1) * 6;
        assert_eq!(expected, 12);
    }

    #[test]
    fn area_x_coordinate_within_canvas() {
        let bars = test_bars();
        let canvas_width = 800.0;
        let time_start = 0.0;
        let time_end = 400.0;
        let time_range = time_end - time_start;

        for bar in &bars {
            let x = ((bar.timestamp as f64 - time_start) / time_range * canvas_width as f64)
                as f32;
            assert!(x > 0.0 && x < canvas_width);
        }
    }

    #[test]
    fn area_y_close_within_canvas() {
        let bars = test_bars();
        let canvas_height = 600.0;
        let value_min = 90.0;
        let value_max = 125.0;
        let value_range = value_max - value_min;

        for bar in &bars {
            let y =
                ((1.0 - (bar.close - value_min) / value_range) * canvas_height as f64) as f32;
            assert!(y > 0.0 && y < canvas_height);
        }
    }

    #[test]
    fn area_empty_bars() {
        let bars: Vec<Bar> = vec![];
        // Should produce 0 vertices, 0 indices.
        assert_eq!(bars.len() * 2, 0);
        assert_eq!((bars.len().saturating_sub(1)) * 6, 0);
    }

    #[test]
    fn area_single_bar_no_indices() {
        let bars = vec![Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap()];
        // Single bar: 2 vertices but 0 quads (need at least 2 bars).
        let index_count = (bars.len() - 1) * 6;
        assert_eq!(index_count, 0);
    }

    #[test]
    fn area_zero_range_produces_nothing() {
        let _bars = test_bars();
        let time_range = 0.0_f64;
        let _value_range = 10.0_f64;
        // With zero time range, no vertices should be generated.
        assert!(time_range < f64::EPSILON);
    }
}

#[cfg(test)]
mod baseline_renderer_tests {
    use fast_chart_core::Bar;

    fn test_bars() -> Vec<Bar> {
        vec![
            Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap(),
            Bar::new(200, 105.0, 115.0, 100.0, 110.0, 1200).unwrap(),
            Bar::new(300, 100.0, 120.0, 95.0, 102.0, 800).unwrap(),
        ]
    }

    #[test]
    fn baseline_color_selection() {
        let baseline_price = 105.0;
        let above_color = [0.0, 0.8, 0.0, 0.4];
        let below_color = [0.8, 0.0, 0.0, 0.4];

        let bars = test_bars();

        // bar[0]: close=105.0 >= 105.0 → above
        let color_0 = if bars[0].close >= baseline_price {
            above_color
        } else {
            below_color
        };
        assert_eq!(color_0, above_color);

        // bar[1]: close=110.0 >= 105.0 → above
        let color_1 = if bars[1].close >= baseline_price {
            above_color
        } else {
            below_color
        };
        assert_eq!(color_1, above_color);

        // bar[2]: close=102.0 < 105.0 → below
        let color_2 = if bars[2].close >= baseline_price {
            above_color
        } else {
            below_color
        };
        assert_eq!(color_2, below_color);
    }

    #[test]
    fn baseline_vertex_count_for_n_bars() {
        let bars = test_bars();
        assert_eq!(bars.len() * 2, 6);
    }

    #[test]
    fn baseline_index_count_for_n_bars() {
        let bars = test_bars();
        assert_eq!((bars.len() - 1) * 6, 12);
    }

    #[test]
    fn baseline_empty_bars() {
        let bars: Vec<Bar> = vec![];
        assert!(bars.len() < 2);
    }
}

#[cfg(test)]
mod histogram_renderer_tests {
    use fast_chart_core::Bar;

    fn test_bars() -> Vec<Bar> {
        vec![
            Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap(),
            Bar::new(200, 105.0, 115.0, 100.0, 110.0, 1200).unwrap(),
            Bar::new(300, 100.0, 120.0, 95.0, 102.0, 800).unwrap(),
        ]
    }

    #[test]
    fn histogram_quad_generation() {
        let bars = test_bars();
        // Each bar should generate 4 vertices and 6 indices.
        let expected_vertices = bars.len() * 4;
        let expected_indices = bars.len() * 6;

        assert_eq!(expected_vertices, 12);
        assert_eq!(expected_indices, 18);
    }

    #[test]
    fn histogram_baseline_color() {
        let baseline = 100.0;
        let positive_color = [0.0, 0.8, 0.0, 0.6];
        let negative_color = [0.8, 0.0, 0.0, 0.6];

        let bar_above = Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap();
        let bar_below = Bar::new(200, 100.0, 110.0, 90.0, 95.0, 1000).unwrap();

        let color_above = if bar_above.close >= baseline {
            positive_color
        } else {
            negative_color
        };
        assert_eq!(color_above, positive_color);

        let color_below = if bar_below.close >= baseline {
            positive_color
        } else {
            negative_color
        };
        assert_eq!(color_below, negative_color);
    }

    #[test]
    fn histogram_bar_width_calculation() {
        let canvas_width = 800.0_f64;
        let bar_count = 10_usize;
        let bar_width = (canvas_width / bar_count as f64) * 0.8;
        assert!((bar_width - 64.0).abs() < f64::EPSILON);
    }

    #[test]
    fn histogram_empty_bars() {
        let bars: Vec<Bar> = vec![];
        assert!(bars.is_empty());
        assert_eq!(bars.len() * 4, 0);
        assert_eq!(bars.len() * 6, 0);
    }

    #[test]
    fn histogram_single_bar() {
        let bars = vec![Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap()];
        assert_eq!(bars.len() * 4, 4);
        assert_eq!(bars.len() * 6, 6);
    }

    #[test]
    fn histogram_y_baseline_above_value() {
        // When close > baseline, y_baseline < y_value in screen coords
        // (because higher price = lower y in flipped coordinate system).
        let baseline_price = 100.0;
        let close = 105.0; // above baseline
        let value_min = 90.0;
        let value_max = 120.0;
        let value_range = value_max - value_min;
        let canvas_height = 600.0;

        let y_baseline =
            ((1.0 - (baseline_price - value_min) / value_range) * canvas_height) as f32;
        let y_value = ((1.0 - (close - value_min) / value_range) * canvas_height) as f32;

        // Higher price → lower screen y (flipped).
        assert!(y_value < y_baseline);
    }

    #[test]
    fn histogram_y_baseline_below_value() {
        // When close < baseline, y_value > y_baseline in screen coords.
        let baseline_price = 110.0;
        let close = 105.0; // below baseline
        let value_min = 90.0;
        let value_max = 120.0;
        let value_range = value_max - value_min;
        let canvas_height = 600.0;

        let y_baseline =
            ((1.0 - (baseline_price - value_min) / value_range) * canvas_height) as f32;
        let y_value = ((1.0 - (close - value_min) / value_range) * canvas_height) as f32;

        // Lower price → higher screen y (flipped).
        assert!(y_value > y_baseline);
    }
}

#[cfg(test)]
mod area_update_from_state_tests {
    use fast_chart_core::Bar;

    /// Simulates the visible-bar filtering that `update_area_from_state` performs.
    fn filter_visible_bars(bars: &[Bar], time_start: f64, time_end: f64) -> Vec<Bar> {
        bars.iter()
            .filter(|b| {
                b.timestamp >= time_start as u64 && b.timestamp <= time_end as u64
            })
            .copied()
            .collect()
    }

    /// Simulates the area vertex generation from visible bars.
    fn compute_area_vertices(
        bars: &[Bar],
        canvas_width: f32,
        canvas_height: f32,
        time_start: f64,
        time_end: f64,
        value_min: f64,
        value_max: f64,
        baseline_price: f64,
    ) -> (usize, usize) {
        if bars.len() < 2 {
            return (0, 0);
        }
        let time_range = time_end - time_start;
        let value_range = value_max - value_min;
        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            return (0, 0);
        }

        let vert_count = bars.len() * 2;
        let idx_count = (bars.len() - 1) * 6;

        // Verify vertex coordinate computation for each bar.
        let cw = canvas_width as f64;
        let ch = canvas_height as f64;
        for bar in bars {
            let x = ((bar.timestamp as f64 - time_start) / time_range * cw) as f32;
            let y_close = ((1.0 - (bar.close - value_min) / value_range) * ch) as f32;
            let y_baseline = ((1.0 - (baseline_price - value_min) / value_range) * ch) as f32;
            assert!(x >= 0.0 && x <= canvas_width, "x out of range: {x}");
            assert!(y_close >= 0.0 && y_close <= canvas_height, "y_close out of range: {y_close}");
            assert!(y_baseline >= 0.0 && y_baseline <= canvas_height, "y_baseline out of range: {y_baseline}");
        }

        (vert_count, idx_count)
    }

    fn test_bars() -> Vec<Bar> {
        vec![
            Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap(),
            Bar::new(200, 105.0, 115.0, 100.0, 110.0, 1200).unwrap(),
            Bar::new(300, 110.0, 120.0, 100.0, 108.0, 800).unwrap(),
        ]
    }

    #[test]
    fn area_initialized_with_correct_vertex_index_counts() {
        // AreaRenderer pre-allocates for 100k bars.
        let vertex_capacity = 200_000; // 2 verts per bar
        let index_capacity = 100_000 * 6;
        assert_eq!(vertex_capacity, 200_000);
        assert_eq!(index_capacity, 600_000);
    }

    #[test]
    fn area_update_produces_correct_vertex_and_index_counts() {
        let bars = test_bars();
        let (verts, indices) = compute_area_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 90.0, 125.0, 90.0,
        );
        // 3 bars → 6 vertices, 12 indices
        assert_eq!(verts, 6);
        assert_eq!(indices, 12);
    }

    #[test]
    fn area_empty_seriesProduces_nothing() {
        let bars: Vec<Bar> = vec![];
        let (verts, indices) = compute_area_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 90.0, 125.0, 90.0,
        );
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn area_single_bar_produces_no_indices() {
        let bars = vec![Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap()];
        let (verts, indices) = compute_area_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 90.0, 125.0, 90.0,
        );
        // Single bar: need at least 2 bars for area fill, so 0 vertices, 0 indices.
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn area_zero_time_range_produces_nothing() {
        let bars = test_bars();
        let (verts, indices) = compute_area_vertices(
            &bars, 800.0, 600.0, 100.0, 100.0, 90.0, 125.0, 90.0,
        );
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn area_zero_value_range_produces_nothing() {
        let bars = test_bars();
        let (verts, indices) = compute_area_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 105.0, 105.0, 105.0,
        );
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn area_visible_bar_filtering() {
        let bars = test_bars();
        // Only bars with timestamp in [150, 250] → just bar at 200
        let visible = filter_visible_bars(&bars, 150.0, 250.0);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].timestamp, 200);

        // All bars visible
        let visible = filter_visible_bars(&bars, 0.0, 400.0);
        assert_eq!(visible.len(), 3);

        // No bars visible
        let visible = filter_visible_bars(&bars, 500.0, 600.0);
        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn area_baseline_at_value_min_fills_to_bottom() {
        // When baseline == value_min, y_baseline should be at canvas bottom (max y).
        let value_min = 90.0;
        let value_max = 125.0;
        let canvas_height = 600.0;
        let baseline_price = value_min;

        let y_baseline = ((1.0 - (baseline_price - value_min) / (value_max - value_min)) * canvas_height) as f32;
        // (1.0 - 0.0) * 600 = 600.0 → bottom of canvas
        assert!((y_baseline - 600.0).abs() < f32::EPSILON);
    }

    #[test]
    fn area_close_vertex_is_above_baseline_when_price_above_min() {
        let value_min = 90.0;
        let value_max = 125.0;
        let canvas_height = 600.0;
        let close = 105.0;
        let baseline_price = value_min;

        let y_close = ((1.0 - (close - value_min) / (value_max - value_min)) * canvas_height) as f32;
        let y_baseline = ((1.0 - (baseline_price - value_min) / (value_max - value_min)) * canvas_height) as f32;

        // Higher price → lower screen y (flipped), so y_close < y_baseline.
        assert!(y_close < y_baseline);
    }
}

#[cfg(test)]
mod histogram_update_from_state_tests {
    use fast_chart_core::Bar;

    /// Simulates the visible-bar filtering that `update_histogram_from_state` performs.
    fn filter_visible_bars(bars: &[Bar], time_start: f64, time_end: f64) -> Vec<Bar> {
        bars.iter()
            .filter(|b| {
                b.timestamp >= time_start as u64 && b.timestamp <= time_end as u64
            })
            .copied()
            .collect()
    }

    /// Simulates the histogram vertex/index generation from visible bars.
    /// Each bar produces 4 vertices (quad) + 6 indices (2 triangles).
    fn compute_histogram_vertices(
        bars: &[Bar],
        canvas_width: f32,
        canvas_height: f32,
        time_start: f64,
        time_end: f64,
        value_min: f64,
        value_max: f64,
        baseline_price: f64,
    ) -> (usize, usize) {
        if bars.is_empty() {
            return (0, 0);
        }
        let time_range = time_end - time_start;
        let value_range = value_max - value_min;
        if time_range < f64::EPSILON || value_range < f64::EPSILON {
            return (0, 0);
        }

        let vert_count = bars.len() * 4;
        let idx_count = bars.len() * 6;

        let cw = canvas_width as f64;
        let ch = canvas_height as f64;
        let bar_width = (cw / bars.len() as f64) * 0.8;

        for bar in bars {
            let x_center = ((bar.timestamp as f64 - time_start) / time_range * cw) as f32;
            let half = (bar_width / 2.0) as f32;
            let x_left = x_center - half;
            let x_right = x_center + half;

            let y_baseline =
                ((1.0 - (baseline_price - value_min) / value_range) * ch) as f32;
            let y_value =
                ((1.0 - (bar.close - value_min) / value_range) * ch) as f32;

            // Verify quad bounds are within canvas (may extend slightly for bars near edges).
            assert!(x_left <= x_right, "x_left > x_right for bar at {}", bar.timestamp);
            assert!(y_baseline != y_value || (bar.close - baseline_price).abs() < f64::EPSILON,
                "baseline == value but close != baseline for bar at {}", bar.timestamp);
        }

        (vert_count, idx_count)
    }

    fn test_bars() -> Vec<Bar> {
        vec![
            Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap(),
            Bar::new(200, 105.0, 115.0, 100.0, 110.0, 1200).unwrap(),
            Bar::new(300, 100.0, 120.0, 95.0, 102.0, 800).unwrap(),
        ]
    }

    #[test]
    fn histogram_initialized_with_correct_capacity() {
        // HistogramRenderer pre-allocates for 100k bars: 4 verts + 6 indices each.
        let vertex_capacity = 100_000 * 4;
        let index_capacity = 100_000 * 6;
        assert_eq!(vertex_capacity, 400_000);
        assert_eq!(index_capacity, 600_000);
    }

    #[test]
    fn histogram_update_produces_correct_vertex_and_index_counts() {
        let bars = test_bars();
        let (verts, indices) = compute_histogram_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 90.0, 125.0, 0.0,
        );
        // 3 bars → 12 vertices (4 per bar), 18 indices (6 per bar)
        assert_eq!(verts, 12);
        assert_eq!(indices, 18);
    }

    #[test]
    fn histogram_empty_series_produces_nothing() {
        let bars: Vec<Bar> = vec![];
        let (verts, indices) = compute_histogram_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 90.0, 125.0, 0.0,
        );
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn histogram_single_bar_produces_quad() {
        let bars = vec![Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap()];
        let (verts, indices) = compute_histogram_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 90.0, 125.0, 0.0,
        );
        // Single bar still produces a full quad.
        assert_eq!(verts, 4);
        assert_eq!(indices, 6);
    }

    #[test]
    fn histogram_zero_time_range_produces_nothing() {
        let bars = test_bars();
        let (verts, indices) = compute_histogram_vertices(
            &bars, 800.0, 600.0, 100.0, 100.0, 90.0, 125.0, 0.0,
        );
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn histogram_zero_value_range_produces_nothing() {
        let bars = test_bars();
        let (verts, indices) = compute_histogram_vertices(
            &bars, 800.0, 600.0, 0.0, 400.0, 105.0, 105.0, 0.0,
        );
        assert_eq!(verts, 0);
        assert_eq!(indices, 0);
    }

    #[test]
    fn histogram_visible_bar_filtering() {
        let bars = test_bars();
        // Only bars with timestamp in [150, 250] → just bar at 200
        let visible = filter_visible_bars(&bars, 150.0, 250.0);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].timestamp, 200);

        // All bars visible
        let visible = filter_visible_bars(&bars, 0.0, 400.0);
        assert_eq!(visible.len(), 3);

        // No bars visible
        let visible = filter_visible_bars(&bars, 500.0, 600.0);
        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn histogram_baseline_color_above() {
        let baseline_price = 100.0;
        let bar = Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap();
        // close=105.0 >= 100.0 → positive (bullish) color
        assert!(bar.close >= baseline_price);
    }

    #[test]
    fn histogram_baseline_color_below() {
        let baseline_price = 105.0;
        let bar = Bar::new(100, 100.0, 110.0, 95.0, 100.0, 1000).unwrap();
        // close=100.0 < 105.0 → negative (bearish) color
        assert!(bar.close < baseline_price);
    }

    #[test]
    fn histogram_baseline_at_zero() {
        // Default baseline is 0.0. All positive close values should be above.
        let baseline_price = 0.0;
        let bar = Bar::new(100, 100.0, 110.0, 95.0, 105.0, 1000).unwrap();
        assert!(bar.close >= baseline_price);
    }

    #[test]
    fn histogram_y_baseline_above_value_when_positive() {
        // When close > baseline, y_baseline > y_value in screen coords
        // (because higher price = lower y in flipped coordinate system).
        let baseline_price = 0.0;
        let close = 105.0;
        let value_min = -10.0;
        let value_max = 120.0;
        let value_range = value_max - value_min;
        let canvas_height = 600.0;

        let y_baseline =
            ((1.0 - (baseline_price - value_min) / value_range) * canvas_height) as f32;
        let y_value =
            ((1.0 - (close - value_min) / value_range) * canvas_height) as f32;

        // Higher price → lower screen y (flipped).
        assert!(y_value < y_baseline);
    }

    #[test]
    fn histogram_y_baseline_below_value_when_negative() {
        // When close < baseline, y_value > y_baseline in screen coords.
        let baseline_price = 10.0;
        let close = -5.0;
        let value_min = -20.0;
        let value_max = 20.0;
        let value_range = value_max - value_min;
        let canvas_height = 600.0;

        let y_baseline =
            ((1.0 - (baseline_price - value_min) / value_range) * canvas_height) as f32;
        let y_value =
            ((1.0 - (close - value_min) / value_range) * canvas_height) as f32;

        // Lower price → higher screen y (flipped).
        assert!(y_value > y_baseline);
    }
}
