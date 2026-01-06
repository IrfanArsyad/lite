use crate::{Component, Context};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Help bar showing keyboard shortcuts at the bottom
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
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        // Define shortcuts to display (2 rows)
        let shortcuts_row1 = vec![
            ("^S", "Save"),
            ("^O", "Open"),
            ("^P", "Quick Open"),
            ("^F", "Find"),
            ("^H", "Replace"),
            ("^G", "Go to Line"),
        ];

        let shortcuts_row2 = vec![
            ("^Q", "Quit"),
            ("^W", "Close"),
            ("^Z", "Undo"),
            ("^Y", "Redo"),
            ("^D", "Select Word"),
            ("^/", "Comment"),
        ];

        let key_style = ctx.editor.theme.statusline.to_ratatui().reversed();
        let desc_style = ctx.editor.theme.statusline.to_ratatui();

        // Build row 1
        let mut spans1 = Vec::new();
        for (key, desc) in &shortcuts_row1 {
            spans1.push(Span::styled(format!(" {} ", key), key_style));
            spans1.push(Span::styled(format!("{} ", desc), desc_style));
        }

        // Build row 2
        let mut spans2 = Vec::new();
        for (key, desc) in &shortcuts_row2 {
            spans2.push(Span::styled(format!(" {} ", key), key_style));
            spans2.push(Span::styled(format!("{} ", desc), desc_style));
        }

        let lines = vec![
            Line::from(spans1),
            Line::from(spans2),
        ];

        let help = Paragraph::new(lines).style(desc_style);
        frame.render_widget(help, area);
    }
}
