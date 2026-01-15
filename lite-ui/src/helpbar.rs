use crate::{Component, Context};
use ratatui::prelude::*;

/// Help bar showing keyboard shortcuts at the bottom (nano-style)
pub struct HelpBar;

impl HelpBar {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelpBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HelpBar {
    fn render(&self, frame: &mut Frame, area: Rect, _ctx: &Context) {
        let bg_style = Style::default().bg(Color::Rgb(64, 64, 64)).fg(Color::Rgb(220, 220, 220));
        let key_style = Style::default().fg(Color::Rgb(255, 255, 255)).bold();

        // All shortcuts organized by category
        let shortcuts: Vec<(&str, &str)> = vec![
            ("^S", "Save"),
            ("^O", "Open"),
            ("^W", "Close"),
            ("^Q", "Quit"),
            ("^F", "Find"),
            ("^H", "Replace"),
            ("^G", "GoTo"),
            ("^Z", "Undo"),
            ("^Y", "Redo"),
            ("^D", "SelNext"),
        ];

        // Calculate spacing for better distribution
        let width = area.width as usize;
        let items_per_row = if area.height >= 2 { 5 } else { 4 };

        // Render based on available height
        if area.height >= 2 {
            // Two rows - show all shortcuts
            let row1_items: Vec<(&str, &str)> = shortcuts.iter().take(items_per_row).copied().collect();
            let row2_items: Vec<(&str, &str)> = shortcuts.iter().skip(items_per_row).copied().collect();

            let row1_area = Rect { height: 1, ..area };
            let row2_area = Rect { y: area.y + 1, height: 1, ..area };

            let row1_line = Self::build_line(&row1_items, width, bg_style, key_style);
            let row2_line = Self::build_line(&row2_items, width, bg_style, key_style);

            frame.render_widget(
                ratatui::widgets::Paragraph::new(row1_line).style(bg_style),
                row1_area,
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(row2_line).style(bg_style),
                row2_area,
            );
        } else if area.height == 1 {
            // Single row - show most important shortcuts
            let items: Vec<(&str, &str)> = shortcuts.iter().take(4).copied().collect();
            let line = Self::build_line(&items, width, bg_style, key_style);
            frame.render_widget(
                ratatui::widgets::Paragraph::new(line).style(bg_style),
                area,
            );
        }
    }
}

impl HelpBar {
    /// Build a line of shortcuts with consistent spacing
    fn build_line(items: &[(&str, &str)], width: usize, bg_style: Style, key_style: Style) -> Line<'static> {
        let mut spans = Vec::new();
        let item_width = width / items.len();

        for (i, (key, desc)) in items.iter().enumerate() {
            // Add key with highlighting
            spans.push(Span::styled(format!("{}", key), key_style));

            // Add description with padding
            let desc_str = format!(" {}", desc);
            let remaining = if i < items.len() - 1 {
                item_width.saturating_sub(key.len() + desc_str.len())
            } else {
                0 // Last item doesn't need padding
            };

            spans.push(Span::styled(desc_str, bg_style));

            if remaining > 0 && i < items.len() - 1 {
                spans.push(Span::styled(" ".repeat(remaining), bg_style));
            }
        }

        Line::from(spans)
    }
}
