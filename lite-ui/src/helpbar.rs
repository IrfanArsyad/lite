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
        let bg_style = Style::default().bg(Color::DarkGray).fg(Color::White);
        let key_style = Style::default().bg(Color::White).fg(Color::Black);

        // Row 1 shortcuts
        let row1: Vec<(&str, &str)> = vec![
            ("^S", "Save"),
            ("^Q", "Quit"),
            ("^F", "Find"),
            ("^G", "GoTo"),
            ("^Z", "Undo"),
        ];

        // Row 2 shortcuts
        let row2: Vec<(&str, &str)> = vec![
            ("^W", "Close"),
            ("^O", "Open"),
            ("^H", "Replace"),
            ("^D", "SelWord"),
            ("^Y", "Redo"),
        ];

        // Calculate spacing
        let width = area.width as usize;
        let items_per_row = 5;
        let item_width = width / items_per_row;

        // Build row 1
        let mut spans1 = Vec::new();
        for (key, desc) in &row1 {
            spans1.push(Span::styled(format!("{}", key), key_style));
            spans1.push(Span::styled(format!(" {:w$}", desc, w = item_width - key.len() - 1), bg_style));
        }

        // Build row 2
        let mut spans2 = Vec::new();
        for (key, desc) in &row2 {
            spans2.push(Span::styled(format!("{}", key), key_style));
            spans2.push(Span::styled(format!(" {:w$}", desc, w = item_width - key.len() - 1), bg_style));
        }

        // Render
        if area.height >= 2 {
            let row1_area = Rect { height: 1, ..area };
            let row2_area = Rect { y: area.y + 1, height: 1, ..area };

            frame.render_widget(
                ratatui::widgets::Paragraph::new(Line::from(spans1)).style(bg_style),
                row1_area,
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(Line::from(spans2)).style(bg_style),
                row2_area,
            );
        } else if area.height == 1 {
            // Single row fallback
            let mut spans = Vec::new();
            for (key, desc) in [("^S", "Save"), ("^Q", "Quit"), ("^F", "Find"), ("^W", "Close")] {
                spans.push(Span::styled(format!("{}", key), key_style));
                spans.push(Span::styled(format!(" {} ", desc), bg_style));
            }
            frame.render_widget(
                ratatui::widgets::Paragraph::new(Line::from(spans)).style(bg_style),
                area,
            );
        }
    }
}
