use crate::{Component, Context};
use lite_core::RopeExt;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Status line at the bottom of the editor
pub struct StatusLine;

impl StatusLine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StatusLine {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for StatusLine {
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        let doc = ctx.editor.current_doc();
        let _view = ctx.editor.current_view();
        let selection = doc.selection(ctx.editor.tree.focus());

        // Left side: mode, filename, modified
        let modified_indicator = if doc.modified { " [+]" } else { "" };
        let filename = doc.name();

        // Calculate cursor position
        let cursor_char = selection.cursor();
        let cursor_pos = doc.rope.char_to_position(cursor_char);
        let line = cursor_pos.line + 1;
        let col = cursor_pos.col + 1;

        // Right side: position, language, encoding
        let language = doc.language.as_deref().unwrap_or("text");
        let encoding = doc.encoding;
        let line_ending = match doc.line_ending {
            lite_view::LineEnding::LF => "LF",
            lite_view::LineEnding::CRLF => "CRLF",
        };

        let position_info = format!("{}:{}", line, col);
        let right_info = format!(" {} | {} | {} ", language, encoding, line_ending);

        // Check for status message
        let (left_text, _left_style) = if let Some((msg, severity)) = &ctx.editor.status_msg {
            let style = match severity {
                lite_view::Severity::Info => ctx.editor.theme.info.to_ratatui(),
                lite_view::Severity::Warning => ctx.editor.theme.warning.to_ratatui(),
                lite_view::Severity::Error => ctx.editor.theme.error.to_ratatui(),
            };
            (msg.clone(), style)
        } else {
            (
                format!(" {}{}", filename, modified_indicator),
                ctx.editor.theme.statusline.to_ratatui(),
            )
        };

        // Build the status line
        let status_style = ctx.editor.theme.statusline.to_ratatui();

        // Calculate padding
        let left_len = left_text.len();
        let right_len = position_info.len() + right_info.len();
        let padding = area.width as usize - left_len.min(area.width as usize) - right_len.min(area.width as usize - left_len);

        let status_text = format!(
            "{}{}{}{}",
            left_text,
            " ".repeat(padding.max(1)),
            position_info,
            right_info
        );

        let status = Paragraph::new(status_text).style(status_style);
        frame.render_widget(status, area);
    }
}
