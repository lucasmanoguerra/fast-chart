//! Runtime configuration hot-reload via filesystem watching.
//!
//! Watches a TOML config file and automatically applies changes to
//! the chart theme (colors, text size, line style, etc.) at runtime.
//!
//! # Example
//!
//! ```no_run
//! use fc_app::config_watcher::ConfigWatcher;
//! use fc_app::theme::{ChartTheme, ThemeHandle};
//!
//! let handle = ThemeHandle::new(ChartTheme::dark());
//!
//! // Watch a config file — changes are applied automatically
//! let _watcher = ConfigWatcher::watch("theme.toml", handle.clone())
//!     .expect("failed to start config watcher");
//!
//! // Now any edit to theme.toml updates the chart theme in real-time
//! ```

use crate::theme::{ChartTheme, ThemeHandle};
use notify::Watcher;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

/// Error type for config watcher operations.
#[derive(Debug)]
pub enum ConfigWatcherError {
    /// Failed to start the filesystem watcher.
    WatcherStart(String),
    /// Failed to read the config file.
    FileRead(std::io::Error),
    /// Failed to parse the config file as TOML.
    Parse(String),
}

impl std::fmt::Display for ConfigWatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WatcherStart(e) => write!(f, "failed to start watcher: {e}"),
            Self::FileRead(e) => write!(f, "failed to read config: {e}"),
            Self::Parse(e) => write!(f, "failed to parse config: {e}"),
        }
    }
}

impl std::error::Error for ConfigWatcherError {}

/// Watches a TOML config file and applies theme changes at runtime.
///
/// Uses `notify` crate's recommended backend for cross-platform
/// filesystem watching. Debounces rapid changes (e.g., editor save
/// sequences) with a simple last-write-wins strategy.
pub struct ConfigWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl ConfigWatcher {
    /// Start watching a config file. When the file changes, the new theme
    /// is parsed and applied to the given `ThemeHandle`.
    ///
    /// The config file must be a valid TOML file with a `[theme]` section.
    /// Colors use hex format: `#RRGGBB` or `#RRGGBBAA`.
    pub fn watch(
        path: impl AsRef<Path>,
        theme_handle: ThemeHandle,
    ) -> Result<Self, ConfigWatcherError> {
        let path = path.as_ref().to_owned();

        // Initial load — fail early if config is invalid
        Self::load_and_apply(&path, &theme_handle)?;

        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| ConfigWatcherError::WatcherStart(e.to_string()))?;

        // Watch the parent directory (editors write to temp + rename)
        let watch_dir = path.parent().unwrap_or(Path::new("."));
        watcher
            .watch(watch_dir.as_ref(), notify::RecursiveMode::NonRecursive)
            .map_err(|e| ConfigWatcherError::WatcherStart(e.to_string()))?;

        // Spawn background thread to process file change events
        let watch_path = path.clone();
        std::thread::spawn(move || {
            Self::event_loop(rx, watch_path, theme_handle);
        });

        Ok(Self { _watcher: watcher })
    }

    /// Event processing loop — runs in a background thread.
    fn event_loop(
        rx: mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>,
        watch_path: PathBuf,
        theme_handle: ThemeHandle,
    ) {
        while let Ok(event_result) = rx.recv() {
            let event = match event_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Only react to modify events
            if !matches!(event.kind, notify::EventKind::Modify(_)) {
                continue;
            }

            let applies_to_us = event.paths.iter().any(|p| {
                p.file_name() == watch_path.file_name()
                    && p.parent() == watch_path.parent()
            });

            if !applies_to_us {
                continue;
            }

            // Debounce: editors may trigger multiple events (write temp, rename)
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Drain pending events
            while let Ok(Ok(_)) = rx.try_recv() {}

            // Apply the latest config
            if let Err(e) = Self::load_and_apply(&watch_path, &theme_handle) {
                log::warn!("config watcher: failed to apply config: {e}");
            } else {
                log::info!("config watcher: applied theme from {}", watch_path.display());
            }
        }
    }

    /// Load the config file and apply it to the theme handle.
    fn load_and_apply(
        path: &Path,
        theme_handle: &ThemeHandle,
    ) -> Result<(), ConfigWatcherError> {
        let content = std::fs::read_to_string(path).map_err(ConfigWatcherError::FileRead)?;
        let new_theme = Self::parse_config(&content)?;
        theme_handle.set(new_theme);
        Ok(())
    }

    /// Parse a TOML config string into a `ChartTheme`.
    pub(crate) fn parse_config(content: &str) -> Result<ChartTheme, ConfigWatcherError> {
        let table: toml::Value =
            toml::from_str(content).map_err(|e| ConfigWatcherError::Parse(e.to_string()))?;

        let mut theme = ChartTheme::dark();

        if let Some(theme_table) = table.get("theme").and_then(|v| v.as_table()) {
            // Color overrides
            macro_rules! try_color {
                ($key:expr, $field:ident) => {
                    if let Some(v) = theme_table.get($key).and_then(|v| v.as_str()) {
                        theme.$field = parse_hex_color(v)
                            .map_err(|e| ConfigWatcherError::Parse(format!("{}: {}", $key, e)))?;
                    }
                };
            }

            try_color!("background", background);
            try_color!("pane_background", pane_background);
            try_color!("grid_line", grid_line);
            try_color!("text_primary", text_primary);
            try_color!("text_secondary", text_secondary);
            try_color!("crosshair_line", crosshair_line);
            try_color!("bullish", bullish);
            try_color!("bearish", bearish);
            try_color!("bullish_fill", bullish_fill);
            try_color!("bearish_fill", bearish_fill);
            try_color!("line_color", line_color);
            try_color!("area_fill", area_fill);
            try_color!("drawing_line", drawing_line);
            try_color!("drawing_fill", drawing_fill);
            try_color!("watermark", watermark);

            // Numeric overrides
            if let Some(v) = theme_table.get("text_font_size").and_then(|v| v.as_float()) {
                theme.text_font_size = v;
            }
        }

        Ok(theme)
    }
}

/// Parse a hex color string like `#RRGGBB` or `#RRGGBBAA` into `Rgba`.
fn parse_hex_color(hex: &str) -> Result<crate::theme::Rgba, String> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            Ok(crate::theme::Rgba::new(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                1.0,
            ))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
            Ok(crate::theme::Rgba::new(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0,
            ))
        }
        _ => Err(format!("invalid hex color length: {}", hex.len())),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_6() {
        use fc_primitives::color::Rgba;
        let c = parse_hex_color("#FF8800").unwrap();
        let expected = Rgba::new(1.0, 0x88 as f64 / 255.0, 0.0, 1.0);
        assert_eq!(c, expected);
    }

    #[test]
    fn parse_hex_8() {
        use fc_primitives::color::Rgba;
        let c = parse_hex_color("#FF880080").unwrap();
        let expected = Rgba::new(1.0, 0x88 as f64 / 255.0, 0.0, 0x80 as f64 / 255.0);
        assert_eq!(c, expected);
    }

    #[test]
    fn parse_hex_no_hash() {
        use fc_primitives::color::Rgba;
        let c = parse_hex_color("00FF00").unwrap();
        let expected = Rgba::new(0.0, 1.0, 0.0, 1.0);
        assert_eq!(c, expected);
    }

    #[test]
    fn parse_hex_invalid() {
        assert!(parse_hex_color("#GGG").is_err());
    }

    #[test]
    fn parse_config_full() {
        use fc_primitives::color::Rgba;
        let s = r##"
[theme]
background = "#111111"
text_primary = "#EEEEEE"
grid_line = "#222222"
crosshair_line = "#333333"
bullish = "#00FF00"
bearish = "#FF0000"
text_font_size = 14.0
"##;
        let theme = ConfigWatcher::parse_config(s).unwrap();
        assert_eq!(theme.background, Rgba::new(0x11 as f64 / 255.0, 0x11 as f64 / 255.0, 0x11 as f64 / 255.0, 1.0));
        assert_eq!(theme.text_primary, Rgba::new(0xEE as f64 / 255.0, 0xEE as f64 / 255.0, 0xEE as f64 / 255.0, 1.0));
        assert_eq!(theme.text_font_size, 14.0);
    }

    #[test]
    fn parse_config_partial() {
        use fc_primitives::color::Rgba;
        let s = r##"
[theme]
background = "#000000"
"##;
        let theme = ConfigWatcher::parse_config(s).unwrap();
        assert_eq!(theme.background, Rgba::new(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn parse_config_empty() {
        let theme = ConfigWatcher::parse_config("").unwrap();
        assert_eq!(theme.text_font_size, 12.0);
    }

    #[test]
    fn parse_config_invalid_toml() {
        let result = ConfigWatcher::parse_config("not valid {{{");
        assert!(result.is_err());
    }

    #[test]
    fn parse_config_invalid_color() {
        let s = r##"
[theme]
background = "not-a-color"
"##;
        let result = ConfigWatcher::parse_config(s);
        assert!(result.is_err());
    }
}
