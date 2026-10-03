//! Configuration and theming for lite editor

mod config;
mod keymap;
mod theme;

pub use config::{Config, ConfigError, EditorConfig, IndentStyle};
pub use keymap::{Action, Key, KeyEvent, Keymap, Modifier};
pub use theme::{Style, Theme};
