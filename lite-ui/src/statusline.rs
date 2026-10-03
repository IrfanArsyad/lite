use crate::{Component, Context};
use lite_core::RopeExt;
use lite_view::{LineEnding, Severity};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

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
        let editor = &*ctx.editor;
        let theme = &editor.theme;
        let doc = editor.current_doc();
        let selection = doc.selection(editor.tree.focus());
        let base_style = theme.statusline.to_ratatui();

        let (left_text, left_style) = match &editor.status_msg {
            Some(msg) => {
                let fg = match msg.severity {
                    Severity::Info => theme.info,
                    Severity::Warning => theme.warning,
                    Severity::Error => theme.error,
                };
                (format!(" {}", msg.text), base_style.patch(fg.to_ratatui()))
            }
            None => {
                let modified = if doc.is_modified() { " [+]" } else { "" };
                (format!(" {}{}", doc.name(), modified), base_style)
            }
        };

        let pos = doc.rope.char_to_position(selection.cursor());
        let cursors = if selection.len() > 1 {
            format!("{} cursors  ", selection.len())
        } else {
            String::new()
        };
        let line_ending = match doc.line_ending {
            LineEnding::LF => "LF",
            LineEnding::CRLF => "CRLF",
        };
        let right_text = format!(
            "{}{}:{}  {}  {}  {} ",
            cursors,
            pos.line + 1,
            pos.col + 1,
            doc.language.as_deref().unwrap_or("text"),
            doc.encoding,
            line_ending
        );

        let width = area.width as usize;
        let right_width = right_text.width();
        // The message wins over the right-hand info when space runs out.
        let (left_text, right_text) = if left_text.width() + right_width + 1 > width {
            (truncate_to_width(&left_text, width), String::new())
        } else {
            (left_text, right_text)
        };
        let padding = width.saturating_sub(left_text.width() + right_text.width());

        let line = Line::from(vec![
            Span::styled(left_text, left_style),
            Span::styled(" ".repeat(padding), base_style),
            Span::styled(right_text, base_style),
        ]);
        frame.render_widget(Paragraph::new(line).style(base_style), area);
    }
}

/// Cut `text` so it takes at most `max_width` terminal columns
pub(crate) fn truncate_to_width(text: &str, max_width: usize) -> String {
    let mut width = 0;
    text.chars()
        .take_while(|c| {
            width += unicode_width::UnicodeWidthChar::width(*c).unwrap_or(0);
            width <= max_width
        })
        .collect()
}
