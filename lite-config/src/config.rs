use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub editor: EditorConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
        }
    }
}

/// Editor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    /// Number of spaces for a tab
    pub tab_width: usize,
    /// Use spaces instead of tabs
    pub indent_style: IndentStyle,
    /// Show line numbers
    pub line_numbers: bool,
    /// Show relative line numbers
    pub relative_line_numbers: bool,
    /// Enable mouse support
    pub mouse: bool,
    /// Scrolloff - minimum lines to keep above/below cursor
    pub scrolloff: usize,
    /// Enable auto-save
    pub auto_save: bool,
    /// Auto-save delay in milliseconds
    pub auto_save_delay: u64,
    /// Enable soft wrap
    pub soft_wrap: bool,
    /// Show whitespace characters
    pub show_whitespace: bool,
    /// Cursor blink rate in milliseconds (0 to disable)
    pub cursor_blink: u64,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            indent_style: IndentStyle::Spaces,
            line_numbers: true,
            relative_line_numbers: false,
            mouse: true,
            scrolloff: 5,
            auto_save: false,
            auto_save_delay: 1000,
            soft_wrap: false,
            show_whitespace: false,
            cursor_blink: 530,
        }
    }
}

/// Indentation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
    Tabs,
    Spaces,
}
