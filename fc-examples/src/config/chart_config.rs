use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub window: WindowConfig,
    pub colors: ColorConfig,
    pub data: DataConfig,
    pub keyboard: KeyboardConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fps_target: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub background: [f32; 4],
    pub grid: [f32; 4],
    pub candle_up: [f32; 4],
    pub candle_down: [f32; 4],
    pub line: [f32; 4],
    pub crosshair: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub source: String,
    pub file_path: Option<String>,
    pub update_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub timeframe_1m: String,
    pub timeframe_5m: String,
    pub timeframe_15m: String,
    pub timeframe_1h: String,
    pub timeframe_1d: String,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 1200,
                height: 800,
                title: "Fast Chart".to_string(),
                fps_target: 60,
            },
            colors: ColorConfig {
                background: [0.06, 0.06, 0.12, 1.0],
                grid: [0.15, 0.15, 0.18, 0.5],
                candle_up: [0.0, 0.8, 0.4, 1.0],
                candle_down: [0.8, 0.2, 0.2, 1.0],
                line: [0.0, 0.8, 0.9, 1.0],
                crosshair: [1.0, 1.0, 1.0, 0.4],
            },
            data: DataConfig {
                source: "simulated".to_string(),
                file_path: None,
                update_interval_ms: 100,
            },
            keyboard: KeyboardConfig {
                timeframe_1m: "1".to_string(),
                timeframe_5m: "5".to_string(),
                timeframe_15m: "15".to_string(),
                timeframe_1h: "6".to_string(),
                timeframe_1d: "D".to_string(),
            },
        }
    }
}

impl ChartConfig {
    /// Load config from a TOML file, falling back to defaults on error.
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                    log::warn!("Failed to parse config: {}, using defaults", e);
                    Self::default()
                }),
                Err(e) => {
                    log::warn!("Failed to read config: {}, using defaults", e);
                    Self::default()
                }
            }
        } else {
            log::info!("No config file found, using defaults");
            Self::default()
        }
    }

    /// Save config to a TOML file.
    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate default config file if it doesn't exist, then load.
    pub fn ensure_config(path: &PathBuf) -> Self {
        if path.exists() {
            Self::load(path)
        } else {
            let config = Self::default();
            if let Err(e) = config.save(path) {
                log::warn!("Failed to create default config: {}", e);
            }
            config
        }
    }
}

/// Watches a config file for changes and returns a reloaded `ChartConfig` on write.
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
    receiver: mpsc::Receiver<Result<notify::Event, notify::Error>>,
    path: PathBuf,
}

impl ConfigWatcher {
    pub fn new(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, receiver) = mpsc::channel();
        let config = notify::Config::default().with_poll_interval(Duration::from_secs(1));
        let mut watcher = RecommendedWatcher::new(tx, config)?;
        if path.exists() {
            watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        }
        Ok(Self {
            _watcher: watcher,
            receiver,
            path: path.clone(),
        })
    }

    /// Non-blocking check: returns `Some(ChartConfig)` if the file was written since last check.
    pub fn check_reload(&self) -> Option<ChartConfig> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => {
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    log::info!("Config file changed, reloading...");
                    Some(ChartConfig::load(&self.path))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
