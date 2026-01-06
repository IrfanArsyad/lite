use ratatui::style::{Color, Modifier};
use serde::{Deserialize, Serialize};

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub name: String,
    // UI elements
    pub background: Style,
    pub foreground: Style,
    pub cursor: Style,
    pub selection: Style,
    pub line_number: Style,
    pub line_number_current: Style,
    pub statusline: Style,
    pub statusline_inactive: Style,
    pub tabline: Style,
    pub tabline_active: Style,
    pub popup: Style,
    pub popup_border: Style,

    // Syntax highlighting
    pub keyword: Style,
    pub function: Style,
    pub type_name: Style,
    pub variable: Style,
    pub constant: Style,
    pub string: Style,
    pub number: Style,
    pub comment: Style,
    pub operator: Style,
    pub punctuation: Style,

    // Git diff
    pub diff_add: Style,
    pub diff_delete: Style,
    pub diff_modify: Style,

    // Diagnostics
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    pub hint: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".into(),
            // UI - dark theme inspired by One Dark
            background: Style::new().bg(Color::Rgb(40, 44, 52)),
            foreground: Style::new().fg(Color::Rgb(171, 178, 191)),
            cursor: Style::new().bg(Color::Rgb(97, 175, 239)).fg(Color::Black),
            selection: Style::new().bg(Color::Rgb(62, 68, 81)),
            line_number: Style::new().fg(Color::Rgb(76, 82, 99)),
            line_number_current: Style::new().fg(Color::Rgb(171, 178, 191)),
            statusline: Style::new()
                .bg(Color::Rgb(33, 37, 43))
                .fg(Color::Rgb(171, 178, 191)),
            statusline_inactive: Style::new()
                .bg(Color::Rgb(33, 37, 43))
                .fg(Color::Rgb(76, 82, 99)),
            tabline: Style::new()
                .bg(Color::Rgb(33, 37, 43))
                .fg(Color::Rgb(76, 82, 99)),
            tabline_active: Style::new()
                .bg(Color::Rgb(40, 44, 52))
                .fg(Color::Rgb(171, 178, 191)),
            popup: Style::new()
                .bg(Color::Rgb(33, 37, 43))
                .fg(Color::Rgb(171, 178, 191)),
            popup_border: Style::new().fg(Color::Rgb(76, 82, 99)),

            // Syntax - One Dark colors
            keyword: Style::new().fg(Color::Rgb(198, 120, 221)), // purple
            function: Style::new().fg(Color::Rgb(97, 175, 239)), // blue
            type_name: Style::new().fg(Color::Rgb(229, 192, 123)), // yellow
            variable: Style::new().fg(Color::Rgb(224, 108, 117)), // red
            constant: Style::new().fg(Color::Rgb(209, 154, 102)), // orange
            string: Style::new().fg(Color::Rgb(152, 195, 121)), // green
            number: Style::new().fg(Color::Rgb(209, 154, 102)), // orange
            comment: Style::new().fg(Color::Rgb(92, 99, 112)),  // gray
            operator: Style::new().fg(Color::Rgb(86, 182, 194)), // cyan
            punctuation: Style::new().fg(Color::Rgb(171, 178, 191)),

            // Git
            diff_add: Style::new().fg(Color::Rgb(152, 195, 121)),
            diff_delete: Style::new().fg(Color::Rgb(224, 108, 117)),
            diff_modify: Style::new().fg(Color::Rgb(229, 192, 123)),

            // Diagnostics
            error: Style::new().fg(Color::Rgb(224, 108, 117)),
            warning: Style::new().fg(Color::Rgb(229, 192, 123)),
            info: Style::new().fg(Color::Rgb(97, 175, 239)),
            hint: Style::new().fg(Color::Rgb(92, 99, 112)),
        }
    }
}

/// Style with foreground, background, and modifiers
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Convert to ratatui Style
    pub fn to_ratatui(&self) -> ratatui::style::Style {
        let mut style = ratatui::style::Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }
        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }
        let mut mods = Modifier::empty();
        if self.bold {
            mods |= Modifier::BOLD;
        }
        if self.italic {
            mods |= Modifier::ITALIC;
        }
        if self.underline {
            mods |= Modifier::UNDERLINED;
        }
        style.add_modifier(mods)
    }
}
