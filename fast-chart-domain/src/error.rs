use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ChartError {
    InvalidPriceData(String),
    InsufficientData { required: usize, available: usize },
    OutOfRange { value: f64, min: f64, max: f64 },
}

impl fmt::Display for ChartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartError::InvalidPriceData(msg) => write!(f, "Invalid price data: {msg}"),
            ChartError::InsufficientData {
                required,
                available,
            } => write!(
                f,
                "Insufficient data: required {required}, available {available}"
            ),
            ChartError::OutOfRange { value, min, max } => {
                write!(f, "Value {value} out of range [{min}, {max}]")
            }
        }
    }
}

impl std::error::Error for ChartError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_price_data() {
        let err = ChartError::InvalidPriceData("high < low".into());
        assert_eq!(err.to_string(), "Invalid price data: high < low");
    }

    #[test]
    fn display_insufficient_data() {
        let err = ChartError::InsufficientData {
            required: 20,
            available: 5,
        };
        assert_eq!(
            err.to_string(),
            "Insufficient data: required 20, available 5"
        );
    }

    #[test]
    fn display_out_of_range() {
        let err = ChartError::OutOfRange {
            value: 150.0,
            min: 0.0,
            max: 100.0,
        };
        assert_eq!(
            err.to_string(),
            "Value 150 out of range [0, 100]"
        );
    }

    #[test]
    fn implements_std_error() {
        let err = ChartError::InvalidPriceData("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn debug_format() {
        let err = ChartError::OutOfRange {
            value: 1.0,
            min: 0.0,
            max: 10.0,
        };
        assert!(format!("{err:?}").contains("OutOfRange"));
    }
}
