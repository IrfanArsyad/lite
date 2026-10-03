use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Errors raised while reading the user's config file
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid config in {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub editor: EditorConfig,
}

impl Config {
    /// Default location: `$XDG_CONFIG_HOME/lite/config.toml` on Linux
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "lite")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Load the config file at the default location. A missing file is not an
    /// error and yields the defaults.
    pub fn load() -> Result<Self, ConfigError> {
        match Self::default_path() {
            Some(path) => Self::load_from(&path),
            None => Ok(Self::default()),
        }
    }

    /// Load the config file at `path`, falling back to defaults when it does not exist
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_path_buf(),
                    source,
                })
            }
        };
        toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_config_keeps_defaults() {
        let config: Config =
            toml::from_str("[editor]\ntab_width = 2\nindent_style = \"tabs\"\n").unwrap();
        assert_eq!(config.editor.tab_width, 2);
        assert_eq!(config.editor.indent_style, IndentStyle::Tabs);
        assert_eq!(config.editor.scrolloff, EditorConfig::default().scrolloff);
    }

    #[test]
    fn test_missing_file_gives_defaults() {
        let config = Config::load_from(Path::new("/nonexistent/lite/config.toml")).unwrap();
        assert_eq!(config.editor.tab_width, EditorConfig::default().tab_width);
    }
}
