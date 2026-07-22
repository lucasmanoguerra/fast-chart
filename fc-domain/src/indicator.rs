use fc_primitives::bar::Bar;
use crate::price_scale::PriceScaleMode;
use fc_primitives::series::TimeSeries;

/// Determines whether an indicator renders as an overlay on an existing pane
/// or in its own separate pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMode {
    /// Render as an overlay on the specified pane.
    OverlayOnPane(usize),
    /// Render in a separate (new) pane.
    SeparatePane,
}

/// Trait for technical indicators that compute derived values from price series.
///
/// # Examples
///
/// ```
/// use fc_domain::Indicator;
/// use fc_primitives::Bar;
/// use fc_primitives::series::TimeSeries;
///
/// struct SimpleMovingAverage { period: usize }
///
/// impl Indicator<100> for SimpleMovingAverage {
///     fn calculate(&self, series: &TimeSeries<Bar, 100>) -> TimeSeries<f64, 100> {
///         let mut result = TimeSeries::new();
///         // ... SMA calculation logic
///         result
///     }
///     fn name(&self) -> &str { "SMA" }
/// }
///
/// let sma = SimpleMovingAverage { period: 14 };
/// assert_eq!(sma.name(), "SMA");
/// ```
pub trait Indicator<const N: usize>: Send + Sync {
    fn calculate(&self, series: &TimeSeries<Bar, N>) -> TimeSeries<f64, N>;
    fn name(&self) -> &str;

    /// Whether this indicator overlays an existing pane or gets its own.
    /// Default: overlay on pane 0 (main chart).
    fn overlay_mode(&self) -> OverlayMode {
        OverlayMode::OverlayOnPane(0)
    }

    /// The preferred price scale mode for this indicator.
    /// Default: Normal (linear).
    fn preferred_scale(&self) -> PriceScaleMode {
        PriceScaleMode::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyIndicator;

    impl Indicator<100> for DummyIndicator {
        fn calculate(&self, _series: &TimeSeries<Bar, 100>) -> TimeSeries<f64, 100> {
            TimeSeries::new()
        }

        fn name(&self) -> &str {
            "DummyIndicator"
        }
    }

    struct SeparatePaneIndicator;

    impl Indicator<100> for SeparatePaneIndicator {
        fn calculate(&self, _series: &TimeSeries<Bar, 100>) -> TimeSeries<f64, 100> {
            TimeSeries::new()
        }

        fn name(&self) -> &str {
            "RSI"
        }

        fn overlay_mode(&self) -> OverlayMode {
            OverlayMode::SeparatePane
        }

        fn preferred_scale(&self) -> PriceScaleMode {
            PriceScaleMode::Normal
        }
    }

    struct LogIndicator;

    impl Indicator<100> for LogIndicator {
        fn calculate(&self, _series: &TimeSeries<Bar, 100>) -> TimeSeries<f64, 100> {
            TimeSeries::new()
        }

        fn name(&self) -> &str {
            "LogInd"
        }

        fn preferred_scale(&self) -> PriceScaleMode {
            PriceScaleMode::Logarithmic
        }
    }

    // Clasificación: determinística — verifica indicator_name
    #[test]
    fn indicator_name() {
        let ind = DummyIndicator;
        assert_eq!(ind.name(), "DummyIndicator");
    }

    // Clasificación: determinística — verifica indicator_calculate_returns_empty
    #[test]
    fn indicator_calculate_returns_empty() {
        let ind = DummyIndicator;
        let series: TimeSeries<Bar, 100> = TimeSeries::new();
        let result = ind.calculate(&series);
        assert!(result.is_empty());
    }

    // Clasificación: determinística — verifica sincronización entre crosshairs del mismo grupo
    #[test]
    fn trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DummyIndicator>();
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn default_overlay_mode_is_overlay_on_pane_0() {
        let ind = DummyIndicator;
        assert_eq!(ind.overlay_mode(), OverlayMode::OverlayOnPane(0));
    }

    // Clasificación: determinística — verifica default_preferred_scale_is_normal
    #[test]
    fn default_preferred_scale_is_normal() {
        let ind = DummyIndicator;
        assert_eq!(ind.preferred_scale(), PriceScaleMode::Normal);
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn separate_pane_indicator() {
        let ind = SeparatePaneIndicator;
        assert_eq!(ind.overlay_mode(), OverlayMode::SeparatePane);
        assert_eq!(ind.preferred_scale(), PriceScaleMode::Normal);
    }

    // Clasificación: determinística — verifica log_indicator_preferred_scale
    #[test]
    fn log_indicator_preferred_scale() {
        let ind = LogIndicator;
        assert_eq!(ind.preferred_scale(), PriceScaleMode::Logarithmic);
    }

    // Clasificación: determinística — verifica overlay_mode_debug
    #[test]
    fn overlay_mode_debug() {
        let m = OverlayMode::OverlayOnPane(2);
        let _dbg = format!("{:?}", m);
    }

    // Clasificación: determinística — verifica overlay_mode_clone
    #[test]
    fn overlay_mode_clone() {
        let m = OverlayMode::SeparatePane;
        let m2 = m;
        assert_eq!(m, m2);
    }
}
