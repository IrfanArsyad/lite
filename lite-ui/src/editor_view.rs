use crate::{Component, Context, EventResult};
use lite_config::KeyEvent;
use lite_core::RopeExt;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

/// Main editor view component
pub struct EditorView;

impl EditorView {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for EditorView {
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        let view = ctx.editor.current_view();
        let doc = ctx.editor.current_doc();

        // Calculate areas
        let gutter_width = view.gutter_width;
        let text_area = Rect {
            x: area.x + gutter_width,
            y: area.y,
            width: area.width.saturating_sub(gutter_width),
            height: area.height,
        };
        let gutter_area = Rect {
            x: area.x,
            y: area.y,
            width: gutter_width,
            height: area.height,
        };

        // Get visible line range
        let first_line = view.scroll_y;
        let last_line = (first_line + area.height as usize).min(doc.len_lines());

        // Render gutter (line numbers)
        let mut gutter_lines = Vec::new();
        for line_num in first_line..last_line {
            let line_str = format!("{:>width$} ", line_num + 1, width = (gutter_width - 1) as usize);
            gutter_lines.push(Line::from(Span::styled(
                line_str,
                ctx.editor.theme.line_number.to_ratatui(),
            )));
        }
        // Fill remaining space
        for _ in last_line..first_line + area.height as usize {
            gutter_lines.push(Line::from(Span::styled(
                " ".repeat(gutter_width as usize),
                ctx.editor.theme.line_number.to_ratatui(),
            )));
        }

        let gutter_widget = Paragraph::new(gutter_lines)
            .style(ctx.editor.theme.background.to_ratatui());
        frame.render_widget(gutter_widget, gutter_area);

        // Render text content
        let selection = doc.selection(ctx.editor.tree.focus());
        let mut text_lines = Vec::new();

        for line_idx in first_line..last_line {
            if line_idx >= doc.len_lines() {
                // Empty line after EOF
                text_lines.push(Line::from(Span::styled(
                    "~",
                    ctx.editor.theme.comment.to_ratatui(),
                )));
                continue;
            }

            let line = doc.rope.line(line_idx);
            let line_start = doc.rope.line_to_char(line_idx);
            let line_text: String = line.chars().collect();
            let line_text = line_text.trim_end_matches('\n').trim_end_matches('\r');

            // Apply horizontal scroll
            let scroll_x = view.scroll_x;
            let visible_text = if scroll_x < line_text.len() {
                &line_text[scroll_x..]
            } else {
                ""
            };

            // Check for selection highlighting
            let mut spans = Vec::new();
            let mut current_pos = 0;
            let line_chars: Vec<char> = visible_text.chars().collect();

            for (i, ch) in line_chars.iter().enumerate() {
                let char_idx = line_start + scroll_x + i;
                let in_selection = selection
                    .ranges()
                    .iter()
                    .any(|r| char_idx >= r.start() && char_idx < r.end());

                let style = if in_selection {
                    ctx.editor.theme.selection.to_ratatui()
                } else {
                    ctx.editor.theme.foreground.to_ratatui()
                };

                // Convert tabs to spaces
                let display_char = if *ch == '\t' {
                    " ".repeat(ctx.editor.config.editor.tab_width)
                } else {
                    ch.to_string()
                };

                spans.push(Span::styled(display_char, style));
            }

            if spans.is_empty() {
                spans.push(Span::raw(""));
            }

            text_lines.push(Line::from(spans));
        }

        // Fill remaining lines
        for _ in text_lines.len()..area.height as usize {
            text_lines.push(Line::from(Span::styled(
                "~",
                ctx.editor.theme.comment.to_ratatui(),
            )));
        }

        let text_widget = Paragraph::new(text_lines)
            .style(ctx.editor.theme.background.to_ratatui());
        frame.render_widget(text_widget, text_area);
    }

    fn cursor(&self, area: Rect, ctx: &Context) -> Option<(u16, u16)> {
        let view = ctx.editor.current_view();
        let doc = ctx.editor.current_doc();
        let selection = doc.selection(ctx.editor.tree.focus());

        // Get cursor position from primary selection
        let cursor_char = selection.cursor();
        let cursor_pos = doc.rope.char_to_position(cursor_char);

        // Check if cursor is visible
        if cursor_pos.line < view.scroll_y {
            return None;
        }
        if cursor_pos.line >= view.scroll_y + view.height as usize {
            return None;
        }
        if cursor_pos.col < view.scroll_x {
            return None;
        }

        // Calculate screen position
        let screen_y = (cursor_pos.line - view.scroll_y) as u16;
        let screen_x = (cursor_pos.col - view.scroll_x) as u16 + view.gutter_width;

        Some((area.x + screen_x, area.y + screen_y))
    }
}
