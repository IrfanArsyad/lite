use crate::{Component, Context};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Tab line showing open buffers
pub struct TabLine;

impl TabLine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TabLine {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for TabLine {
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        let current_doc_id = ctx.editor.current_view().doc_id;
        let buffers = ctx.editor.buffer_list();

        let mut spans = Vec::new();
        let tab_style = ctx.editor.theme.tabline.to_ratatui();
        let tab_active_style = ctx.editor.theme.tabline_active.to_ratatui();

        for (i, (doc_id, title)) in buffers.iter().enumerate() {
            let is_active = *doc_id == current_doc_id;
            let style = if is_active {
                tab_active_style
            } else {
                tab_style
            };

            // Add tab number
            let tab_text = format!(" {}:{} ", i + 1, title);
            spans.push(Span::styled(tab_text, style));

            // Add separator
            if i < buffers.len() - 1 {
                spans.push(Span::styled("│", tab_style));
            }
        }

        // Fill remaining space
        let used_width: usize = spans.iter().map(|s| s.width()).sum();
        if used_width < area.width as usize {
            spans.push(Span::styled(
                " ".repeat(area.width as usize - used_width),
                tab_style,
            ));
        }

        let tabs = Paragraph::new(Line::from(spans));
        frame.render_widget(tabs, area);
    }
}
