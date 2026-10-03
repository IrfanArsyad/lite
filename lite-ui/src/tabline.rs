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
        let tab_style = ctx.editor.theme.tabline.to_ratatui();
        let tab_active_style = ctx.editor.theme.tabline_active.to_ratatui();

        let mut tabs: Vec<(Span, bool)> = Vec::new();
        for (i, (doc_id, title)) in buffers.iter().enumerate() {
            let is_active = *doc_id == current_doc_id;
            let style = if is_active {
                tab_active_style
            } else {
                tab_style
            };
            tabs.push((
                Span::styled(format!(" {}:{} ", i + 1, title), style),
                is_active,
            ));
            if i + 1 < buffers.len() {
                tabs.push((Span::styled("│", tab_style), false));
            }
        }

        // When the tabs overflow, drop tabs from the left until the active one fits.
        let width = area.width as usize;
        let active_end = tabs
            .iter()
            .position(|(_, active)| *active)
            .map(|idx| tabs[..=idx].iter().map(|(s, _)| s.width()).sum::<usize>())
            .unwrap_or(0);
        let mut skipped = 0;
        let mut skip = 0;
        while active_end - skipped > width && skip < tabs.len() {
            skipped += tabs[skip].0.width();
            skip += 1;
        }

        let mut spans: Vec<Span> = tabs.into_iter().skip(skip).map(|(s, _)| s).collect();
        let used_width: usize = spans.iter().map(|s| s.width()).sum();
        if used_width < width {
            spans.push(Span::styled(" ".repeat(width - used_width), tab_style));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}
