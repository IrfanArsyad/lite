use crate::{Component, Context};
use lite_core::RopeExt;
use lite_view::{highlighter, Highlight, HighlightSpan};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Main editor view component
pub struct EditorView;

impl EditorView {
    pub fn new() -> Self {
        Self
    }

    /// Get the style for a highlight type from theme
    fn highlight_style(highlight: Highlight, ctx: &Context) -> ratatui::style::Style {
        match highlight {
            Highlight::Keyword => ctx.editor.theme.keyword.to_ratatui(),
            Highlight::Function => ctx.editor.theme.function.to_ratatui(),
            Highlight::Type => ctx.editor.theme.type_name.to_ratatui(),
            Highlight::Variable => ctx.editor.theme.variable.to_ratatui(),
            Highlight::Constant => ctx.editor.theme.constant.to_ratatui(),
            Highlight::String => ctx.editor.theme.string.to_ratatui(),
            Highlight::Number => ctx.editor.theme.number.to_ratatui(),
            Highlight::Comment => ctx.editor.theme.comment.to_ratatui(),
            Highlight::Operator => ctx.editor.theme.operator.to_ratatui(),
            Highlight::Punctuation => ctx.editor.theme.punctuation.to_ratatui(),
            Highlight::Property => ctx.editor.theme.variable.to_ratatui(),
            Highlight::Parameter => ctx.editor.theme.variable.to_ratatui(),
            Highlight::Label => ctx.editor.theme.constant.to_ratatui(),
            Highlight::Namespace => ctx.editor.theme.type_name.to_ratatui(),
            Highlight::Attribute => ctx.editor.theme.keyword.to_ratatui(),
        }
    }

    /// Find the highlight for a byte position
    fn find_highlight(byte_pos: usize, highlights: &[HighlightSpan]) -> Option<Highlight> {
        // Binary search could be used for optimization, but linear is fine for now
        for span in highlights {
            if byte_pos >= span.start && byte_pos < span.end {
                return Some(span.highlight);
            }
            if span.start > byte_pos {
                break;
            }
        }
        None
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

        // Get syntax highlights
        let source = doc.text();
        let highlights = if let Some(ref lang) = doc.language {
            highlighter().highlight(lang, &source)
        } else {
            Vec::new()
        };

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
            let line_start_char = doc.rope.line_to_char(line_idx);
            let line_start_byte = doc.rope.char_to_byte(line_start_char);
            let line_text: String = line.chars().collect();
            let line_text = line_text.trim_end_matches('\n').trim_end_matches('\r');

            // Apply horizontal scroll
            let scroll_x = view.scroll_x;
            let visible_text = if scroll_x < line_text.len() {
                &line_text[scroll_x..]
            } else {
                ""
            };

            // Build spans with syntax highlighting
            let mut spans = Vec::new();
            let line_chars: Vec<char> = visible_text.chars().collect();

            // Calculate byte offset for scroll_x
            let scroll_byte_offset: usize = line_text.chars().take(scroll_x).map(|c| c.len_utf8()).sum();

            let mut byte_offset = 0;
            for (i, ch) in line_chars.iter().enumerate() {
                let char_idx = line_start_char + scroll_x + i;
                let byte_pos = line_start_byte + scroll_byte_offset + byte_offset;

                let in_selection = selection
                    .ranges()
                    .iter()
                    .any(|r| char_idx >= r.start() && char_idx < r.end());

                // Determine style based on selection and syntax highlighting
                let style = if in_selection {
                    ctx.editor.theme.selection.to_ratatui()
                } else if let Some(highlight) = Self::find_highlight(byte_pos, &highlights) {
                    Self::highlight_style(highlight, ctx)
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
                byte_offset += ch.len_utf8();
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
