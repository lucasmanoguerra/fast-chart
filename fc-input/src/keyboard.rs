//! Configurable keyboard shortcut system.
//!
//! [`KeyboardShortcutMap`] maps key + modifier combinations to
//! [`ChartCommand`]s. [`KeyboardPresets`] provides ready-made profiles
//! (default, trading, minimal).

use crate::engine::ChartCommand;

// ---------------------------------------------------------------------------
// Modifiers (keyboard-specific)
// ---------------------------------------------------------------------------

/// Modifier keys state for shortcut matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Modifiers {
    pub const NONE: Self = Self { shift: false, ctrl: false, alt: false, meta: false };

    pub const fn new(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        Self { shift, ctrl, alt, meta }
    }
}

// ---------------------------------------------------------------------------
// KeyboardShortcut
// ---------------------------------------------------------------------------

/// A keyboard shortcut: key + modifiers -> command.
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    /// Key name (e.g., `"Escape"`, `"Delete"`, `"1"`, `"ArrowLeft"`).
    pub key: String,
    /// Required modifiers.
    pub modifiers: Modifiers,
    /// Command to emit when this shortcut is triggered.
    pub command: ChartCommand,
    /// Description for documentation/UI.
    pub description: String,
}

// ---------------------------------------------------------------------------
// KeyboardShortcutMap
// ---------------------------------------------------------------------------

/// Maps keyboard shortcuts to chart commands.
pub struct KeyboardShortcutMap {
    shortcuts: Vec<KeyboardShortcut>,
}

impl KeyboardShortcutMap {
    pub fn new() -> Self {
        Self { shortcuts: Vec::new() }
    }

    /// Register a shortcut. Duplicate key+modifiers allowed; first match wins.
    pub fn register(&mut self, shortcut: KeyboardShortcut) {
        self.shortcuts.push(shortcut);
    }

    /// Remove all shortcuts.
    pub fn clear(&mut self) {
        self.shortcuts.clear();
    }

    /// Handle a keyboard event. Returns the first matching shortcut's command.
    pub fn handle_event(&self, key: &str, modifiers: &Modifiers) -> Option<&ChartCommand> {
        self.find(key, modifiers).map(|s| &s.command)
    }

    /// Get all registered shortcuts.
    pub fn shortcuts(&self) -> &[KeyboardShortcut] {
        &self.shortcuts
    }

    /// Find the first shortcut matching `key` + `modifiers`.
    pub fn find(&self, key: &str, modifiers: &Modifiers) -> Option<&KeyboardShortcut> {
        self.shortcuts.iter().find(|s| s.key == key && s.modifiers == *modifiers)
    }

    /// Remove the first shortcut matching `key` + `modifiers`.
    pub fn remove(&mut self, key: &str, modifiers: &Modifiers) -> bool {
        if let Some(pos) = self.shortcuts.iter().position(|s| s.key == key && s.modifiers == *modifiers) {
            self.shortcuts.remove(pos);
            true
        } else {
            false
        }
    }

    /// Number of registered shortcuts.
    pub fn len(&self) -> usize {
        self.shortcuts.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.shortcuts.is_empty()
    }
}

impl Default for KeyboardShortcutMap {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// KeyboardPresets
// ---------------------------------------------------------------------------

/// Preset shortcut profiles.
pub struct KeyboardPresets;

impl KeyboardPresets {
    /// Default shortcuts: Escape=cancel/deselect, Delete/Backspace=delete,
    /// +/-=zoom, arrows=pan, 1-9=drawing tools.
    pub fn default_shortcuts() -> KeyboardShortcutMap {
        let mut map = KeyboardShortcutMap::new();
        let none = Modifiers::NONE;

        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: none,
            command: ChartCommand::CancelDrawing,
            description: "Cancel drawing and deselect all".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: none,
            command: ChartCommand::DeselectAll,
            description: "Deselect all".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "Delete".to_string(), modifiers: none,
            command: ChartCommand::DeleteSelected,
            description: "Delete selected drawing".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "Backspace".to_string(), modifiers: none,
            command: ChartCommand::DeleteSelected,
            description: "Delete selected drawing".to_string(),
        });

        // Zoom
        map.register(KeyboardShortcut {
            key: "+".to_string(), modifiers: none,
            command: ChartCommand::ZoomAtCursor { factor: 1.5, screen_x: 0.0, screen_y: 0.0 },
            description: "Zoom in at cursor".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "=".to_string(), modifiers: none,
            command: ChartCommand::ZoomAtCursor { factor: 1.5, screen_x: 0.0, screen_y: 0.0 },
            description: "Zoom in at cursor".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "-".to_string(), modifiers: none,
            command: ChartCommand::ZoomAtCursor { factor: 0.67, screen_x: 0.0, screen_y: 0.0 },
            description: "Zoom out at cursor".to_string(),
        });

        // Pan
        map.register(KeyboardShortcut {
            key: "ArrowLeft".to_string(), modifiers: none,
            command: ChartCommand::Pan { time_delta: -5, price_delta: 0.0 },
            description: "Pan left".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "ArrowRight".to_string(), modifiers: none,
            command: ChartCommand::Pan { time_delta: 5, price_delta: 0.0 },
            description: "Pan right".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "ArrowUp".to_string(), modifiers: none,
            command: ChartCommand::Pan { time_delta: 0, price_delta: 5.0 },
            description: "Pan up".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "ArrowDown".to_string(), modifiers: none,
            command: ChartCommand::Pan { time_delta: 0, price_delta: -5.0 },
            description: "Pan down".to_string(),
        });

        // Drawing tools 1-9
        let tools = [
            "TrendLine", "Arrow", "Ray", "Segment", "Rectangle", "Ellipse",
            "HorizontalLine", "VerticalLine", "FibonacciRetracement",
        ];
        for (i, tool_name) in tools.iter().enumerate() {
            let digit = char::from_digit((i + 1) as u32, 10).expect("index 1..=9 is always a digit");
            let tool = match *tool_name {
                "TrendLine" => crate::engine::DrawingTool::TrendLine,
                "Arrow" => crate::engine::DrawingTool::Arrow,
                "Ray" => crate::engine::DrawingTool::Ray,
                "Segment" => crate::engine::DrawingTool::Segment,
                "Rectangle" => crate::engine::DrawingTool::Rectangle,
                "Ellipse" => crate::engine::DrawingTool::Ellipse,
                "HorizontalLine" => crate::engine::DrawingTool::HorizontalLine,
                "VerticalLine" => crate::engine::DrawingTool::VerticalLine,
                "FibonacciRetracement" => crate::engine::DrawingTool::FibonacciRetracement,
                _ => unreachable!(),
            };
            map.register(KeyboardShortcut {
                key: digit.to_string(), modifiers: none,
                command: ChartCommand::StartDrawing { tool },
                description: format!("Start drawing {tool_name}"),
            });
        }

        map
    }

    /// Trading-focused shortcuts.
    pub fn trading_shortcuts() -> KeyboardShortcutMap {
        let mut map = Self::default_shortcuts();
        let none = Modifiers::NONE;

        map.register(KeyboardShortcut {
            key: "h".to_string(), modifiers: none,
            command: ChartCommand::RequestRedraw,
            description: "Toggle crosshair mode".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "m".to_string(), modifiers: none,
            command: ChartCommand::RequestRedraw,
            description: "Toggle magnetic crosshair".to_string(),
        });
        map.register(KeyboardShortcut {
            key: " ".to_string(), modifiers: none,
            command: ChartCommand::RequestRedraw,
            description: "Toggle auto-scroll".to_string(),
        });

        map
    }

    /// Minimal shortcuts: only Escape and Delete.
    pub fn minimal_shortcuts() -> KeyboardShortcutMap {
        let mut map = KeyboardShortcutMap::new();
        let none = Modifiers::NONE;

        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: none,
            command: ChartCommand::CancelDrawing,
            description: "Cancel drawing".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: none,
            command: ChartCommand::DeselectAll,
            description: "Deselect all".to_string(),
        });
        map.register(KeyboardShortcut {
            key: "Delete".to_string(), modifiers: none,
            command: ChartCommand::DeleteSelected,
            description: "Delete selected drawing".to_string(),
        });
        map
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd_delete() -> ChartCommand { ChartCommand::DeleteSelected }
    fn cmd_cancel() -> ChartCommand { ChartCommand::CancelDrawing }
    fn cmd_deselect() -> ChartCommand { ChartCommand::DeselectAll }

    #[test]
    fn empty_map() {
        let map = KeyboardShortcutMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn register_shortcut() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "Cancel".to_string(),
        });
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
    }

    #[test]
    fn handle_event_match() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "Cancel".to_string(),
        });
        assert_eq!(map.handle_event("Escape", &Modifiers::NONE), Some(&cmd_cancel()));
    }

    #[test]
    fn handle_event_no_match() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut {
            key: "Escape".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "".to_string(),
        });
        assert!(map.handle_event("Delete", &Modifiers::NONE).is_none());
    }

    #[test]
    fn handle_event_with_modifier() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut {
            key: "z".to_string(), modifiers: Modifiers::new(true, true, false, false), command: cmd_cancel(), description: "Shift+Ctrl+Z".to_string(),
        });
        assert_eq!(map.handle_event("z", &Modifiers::new(true, true, false, false)), Some(&cmd_cancel()));
    }

    #[test]
    fn handle_event_wrong_modifier() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut {
            key: "z".to_string(), modifiers: Modifiers::new(true, false, false, false), command: cmd_cancel(), description: "Shift+Z".to_string(),
        });
        assert!(map.handle_event("z", &Modifiers::NONE).is_none());
    }

    #[test]
    fn find_shortcut() {
        let mut map = KeyboardShortcutMap::new();
        let shortcut = KeyboardShortcut {
            key: "Delete".to_string(), modifiers: Modifiers::NONE, command: cmd_delete(), description: "Delete".to_string(),
        };
        map.register(shortcut);
        let found = map.find("Delete", &Modifiers::NONE);
        assert!(found.is_some());
        assert_eq!(found.unwrap().key, "Delete");
    }

    #[test]
    fn remove_shortcut() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut {
            key: "Delete".to_string(), modifiers: Modifiers::NONE, command: cmd_delete(), description: "Delete".to_string(),
        });
        assert!(map.remove("Delete", &Modifiers::NONE));
        assert!(map.is_empty());
    }

    #[test]
    fn clear_shortcuts() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut { key: "a".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "".to_string() });
        map.register(KeyboardShortcut { key: "b".to_string(), modifiers: Modifiers::NONE, command: cmd_delete(), description: "".to_string() });
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn len_and_empty() {
        let mut map = KeyboardShortcutMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.register(KeyboardShortcut { key: "a".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "".to_string() });
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        map.register(KeyboardShortcut { key: "b".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "".to_string() });
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn default_preset_has_basics() {
        let map = KeyboardPresets::default_shortcuts();
        assert!(map.find("Escape", &Modifiers::NONE).is_some());
        assert!(map.find("Delete", &Modifiers::NONE).is_some());
        assert!(map.find("ArrowLeft", &Modifiers::NONE).is_some());
        assert!(map.find("ArrowRight", &Modifiers::NONE).is_some());
        assert!(map.find("ArrowUp", &Modifiers::NONE).is_some());
        assert!(map.find("ArrowDown", &Modifiers::NONE).is_some());
        assert!(map.find("+", &Modifiers::NONE).is_some());
        assert!(map.find("-", &Modifiers::NONE).is_some());
        assert!(map.find("1", &Modifiers::NONE).is_some());
    }

    #[test]
    fn trading_preset_has_more() {
        let default = KeyboardPresets::default_shortcuts();
        let trading = KeyboardPresets::trading_shortcuts();
        assert!(trading.len() > default.len());
        assert!(trading.find("h", &Modifiers::NONE).is_some());
        assert!(trading.find("m", &Modifiers::NONE).is_some());
        assert!(trading.find(" ", &Modifiers::NONE).is_some());
    }

    #[test]
    fn minimal_preset_has_few() {
        let map = KeyboardPresets::minimal_shortcuts();
        let keys: Vec<&str> = map.shortcuts().iter().map(|s| s.key.as_str()).collect();
        let unique: std::collections::HashSet<&str> = keys.iter().copied().collect();
        assert_eq!(unique.len(), 2);
        assert!(map.find("Escape", &Modifiers::NONE).is_some());
        assert!(map.find("Delete", &Modifiers::NONE).is_some());
    }

    #[test]
    fn duplicate_key_register() {
        let mut map = KeyboardShortcutMap::new();
        map.register(KeyboardShortcut { key: "Escape".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(), description: "Cancel".to_string() });
        map.register(KeyboardShortcut { key: "Escape".to_string(), modifiers: Modifiers::NONE, command: cmd_deselect(), description: "Deselect".to_string() });
        assert_eq!(map.len(), 2);
        assert_eq!(map.handle_event("Escape", &Modifiers::NONE), Some(&cmd_cancel()));
    }

    #[test]
    fn description_field() {
        let shortcut = KeyboardShortcut {
            key: "Escape".to_string(), modifiers: Modifiers::NONE, command: cmd_cancel(),
            description: "Cancel the current drawing".to_string(),
        };
        assert_eq!(shortcut.description, "Cancel the current drawing");
    }
}
