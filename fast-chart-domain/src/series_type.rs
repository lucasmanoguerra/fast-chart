#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesType {
    Candle,
    Bar,
    Line,
    Area,
    Baseline,
}

impl Default for SeriesType {
    fn default() -> Self {
        Self::Candle
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
    fn all_five_variants() {
        let variants = [
            SeriesType::Candle,
            SeriesType::Bar,
            SeriesType::Line,
            SeriesType::Area,
            SeriesType::Baseline,
        ];
        assert_eq!(variants.len(), 5);
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
}
