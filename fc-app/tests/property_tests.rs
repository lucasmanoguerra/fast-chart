//! Property-based tests for fast-chart core types.

use proptest::prelude::*;

proptest! {
    // Clasificación: determinística — verifica viewport_min_le_max
    #[test]
    fn viewport_min_le_max(
        time_start in 0u64..1_000_000,
        time_end in 1_000_001u64..2_000_000,
        value_min in -1000.0f64..0.0,
        value_max in 0.1f64..1000.0,
    ) {
        let vp = fc_domain::Viewport {
            time_start,
            time_end,
            value_min,
            value_max,
            zoom_level: 1.0,
        };
        prop_assert!(vp.time_start < vp.time_end);
        prop_assert!(vp.value_min < vp.value_max);
    }

    // Clasificación: determinística — verifica snap_never_negative
    #[test]
    fn snap_never_negative(x in -1000.0f64..1000.0) {
        use fc_app::render::pixel_perfect::PixelPerfect;
        prop_assert!(x.snap() >= 0.0);
    }

    // Clasificación: determinística — verifica snap_size_non_negative
    #[test]
    fn snap_size_non_negative(x in -100.0f64..100.0) {
        use fc_app::render::pixel_perfect::PixelPerfect;
        prop_assert!(x.snap_size() >= 0.0);
    }

    // Clasificación: determinística — verifica snap_generic_matches_snap
    #[test]
    fn snap_generic_matches_snap(x in 0.0f64..1000.0) {
        use fc_app::render::pixel_perfect::{PixelPerfect, snap_generic};
        let trait_result = x.snap();
        let generic_result = snap_generic(x);
        prop_assert!((trait_result - generic_result).abs() < 1e-10, "snap={} snap_generic={}", trait_result, generic_result);
    }

    // Clasificación: determinística — verifica rgba_fields_in_0_1
    #[test]
    fn rgba_fields_in_0_1(
        r in 0.0f64..=1.0,
        g in 0.0f64..=1.0,
        b in 0.0f64..=1.0,
        a in 0.0f64..=1.0,
    ) {
        let c = fc_app::theme::Rgba::new(r, g, b, a);
        prop_assert!(c.0 >= 0.0 && c.0 <= 1.0);
        prop_assert!(c.1 >= 0.0 && c.1 <= 1.0);
        prop_assert!(c.2 >= 0.0 && c.2 <= 1.0);
        prop_assert!(c.3 >= 0.0 && c.3 <= 1.0);
    }
}
