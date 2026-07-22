use thiserror::Error;

/// Errors that can occur during chart data processing or rendering.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ChartError {
    #[error("Invalid price data: {0}")]
    InvalidPriceData(String),

    #[error("Insufficient data: required {required}, available {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Value {value} out of range [{min}, {max}]")]
    OutOfRange { value: f64, min: f64, max: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica display_invalid_price_data
    #[test]
    fn display_invalid_price_data() {
        let err = ChartError::InvalidPriceData("high < low".into());
        assert_eq!(err.to_string(), "Invalid price data: high < low");
    }

    // Clasificación: determinística — verifica display_insufficient_data
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

    // Clasificación: determinística — verifica display_out_of_range
    #[test]
    fn display_out_of_range() {
        let err = ChartError::OutOfRange {
            value: 150.0,
            min: 0.0,
            max: 100.0,
        };
        assert_eq!(err.to_string(), "Value 150 out of range [0, 100]");
    }

    // Clasificación: determinística — verifica implements_std_error
    #[test]
    fn implements_std_error() {
        let err = ChartError::InvalidPriceData("test".into());
        let _: &dyn std::error::Error = &err;
    }

    // Clasificación: determinística — verifica debug_format
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
