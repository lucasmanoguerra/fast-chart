#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesType {
    Candle,
    Bar,
    Line,
    Area,
    Baseline,
    StepLine,
    Volume,
    PointFigure,
    LineBreak,
    Range,
}

impl Default for SeriesType {
    fn default() -> Self {
        Self::Candle
    }
}

impl SeriesType {
    /// All built-in series types.
    pub const ALL: &'static [SeriesType] = &[
        SeriesType::Candle,
        SeriesType::Bar,
        SeriesType::Line,
        SeriesType::Area,
        SeriesType::Baseline,
        SeriesType::StepLine,
        SeriesType::Volume,
        SeriesType::PointFigure,
        SeriesType::LineBreak,
        SeriesType::Range,
    ];

    /// Display name for this series type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Candle => "Candlestick",
            Self::Bar => "Bar",
            Self::Line => "Line",
            Self::Area => "Area",
            Self::Baseline => "Baseline",
            Self::StepLine => "Step Line",
            Self::Volume => "Volume",
            Self::PointFigure => "Point & Figure",
            Self::LineBreak => "Line Break",
            Self::Range => "Range",
        }
    }

    /// Whether this series type is a volume-based indicator.
    pub fn is_volume(&self) -> bool {
        *self == Self::Volume
    }

    /// Whether this series type is a breakout pattern.
    pub fn is_breakout_pattern(&self) -> bool {
        matches!(self, Self::PointFigure | Self::LineBreak | Self::Range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_candle() {
        assert_eq!(SeriesType::default(), SeriesType::Candle);
    }

    #[test]
    fn all_ten_variants() {
        assert_eq!(SeriesType::ALL.len(), 10);
    }

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", SeriesType::Line), "Line");
    }

    #[test]
    fn clone_and_eq() {
        let a = SeriesType::Area;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn display_names() {
        assert_eq!(SeriesType::Candle.display_name(), "Candlestick");
        assert_eq!(SeriesType::StepLine.display_name(), "Step Line");
        assert_eq!(SeriesType::PointFigure.display_name(), "Point & Figure");
        assert_eq!(SeriesType::LineBreak.display_name(), "Line Break");
        assert_eq!(SeriesType::Range.display_name(), "Range");
        assert_eq!(SeriesType::Volume.display_name(), "Volume");
    }

    #[test]
    fn is_volume() {
        assert!(SeriesType::Volume.is_volume());
        assert!(!SeriesType::Candle.is_volume());
        assert!(!SeriesType::Line.is_volume());
    }

    #[test]
    fn is_breakout_pattern() {
        assert!(SeriesType::PointFigure.is_breakout_pattern());
        assert!(SeriesType::LineBreak.is_breakout_pattern());
        assert!(SeriesType::Range.is_breakout_pattern());
        assert!(!SeriesType::Candle.is_breakout_pattern());
        assert!(!SeriesType::Line.is_breakout_pattern());
    }

    #[test]
    fn new_variants_are_distinct() {
        let new_types = [
            SeriesType::StepLine,
            SeriesType::Volume,
            SeriesType::PointFigure,
            SeriesType::LineBreak,
            SeriesType::Range,
        ];
        // Each is distinct
        for (i, a) in new_types.iter().enumerate() {
            for b in &new_types[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
