/// Localization trait for formatting text in different locales.
pub trait Localizer: Send + Sync {
    /// Format a timestamp as a date-like label (placeholder — not a real calendar date).
    fn format_timestamp(&self, timestamp: u64) -> String;

    /// Format a timestamp as a time-like label (placeholder — not a real clock time).
    fn format_time_label(&self, timestamp: u64) -> String;

    /// Get the locale identifier (e.g., "en-US", "es-AR").
    fn locale_id(&self) -> &str;
}

/// Default English localizer.
pub struct EnglishLocalizer;

impl Localizer for EnglishLocalizer {
    fn format_timestamp(&self, timestamp: u64) -> String {
        format!("T+{}", timestamp)
    }

    fn format_time_label(&self, timestamp: u64) -> String {
        format!("T+{}", timestamp)
    }

    fn locale_id(&self) -> &str {
        "en-US"
    }
}

/// Spanish (Argentina) localizer.
pub struct SpanishLocalizer;

impl Localizer for SpanishLocalizer {
    fn format_timestamp(&self, timestamp: u64) -> String {
        format!("T+{}", timestamp)
    }

    fn format_time_label(&self, timestamp: u64) -> String {
        format!("T+{}", timestamp)
    }

    fn locale_id(&self) -> &str {
        "es-AR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Clasificación: determinística — verifica english_format_timestamp
    #[test]
    fn english_format_timestamp() {
        let loc = EnglishLocalizer;
        assert_eq!(loc.format_timestamp(1000), "T+1000");
    }

    // Clasificación: determinística — verifica english_format_time_label
    #[test]
    fn english_format_time_label() {
        let loc = EnglishLocalizer;
        assert_eq!(loc.format_time_label(42), "T+42");
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn spanish_format_timestamp() {
        let loc = SpanishLocalizer;
        assert_eq!(loc.format_timestamp(500), "T+500");
    }

    // Clasificación: determinística — verifica detección de gesto pan (arrastre con un dedo)
    #[test]
    fn spanish_format_time_label() {
        let loc = SpanishLocalizer;
        assert_eq!(loc.format_time_label(0), "T+0");
    }

    // Clasificación: determinística — verifica locale_ids
    #[test]
    fn locale_ids() {
        assert_eq!(EnglishLocalizer.locale_id(), "en-US");
        assert_eq!(SpanishLocalizer.locale_id(), "es-AR");
    }
}
