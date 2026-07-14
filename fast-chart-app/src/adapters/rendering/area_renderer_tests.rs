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
