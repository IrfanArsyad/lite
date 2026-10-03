use crate::{Component, Context};
use lite_core::{char_display_width, display_col, RopeExt, CONTROL_REPLACEMENT};
use lite_view::{Area, Document, Editor, Highlight, HighlightSpan, Layout, View, ViewId};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Marker drawn on lines past the end of the file
const EOF_MARKER: &str = "~";

/// Main editor view component: renders every split view
pub struct EditorView;

impl EditorView {
    pub fn new() -> Self {
        Self
    }

    /// Get the style for a highlight type from theme
    fn highlight_style(highlight: Highlight, editor: &Editor) -> Style {
        let theme = &editor.theme;
        match highlight {
            Highlight::Keyword => theme.keyword.to_ratatui(),
            Highlight::Function => theme.function.to_ratatui(),
            Highlight::Type => theme.type_name.to_ratatui(),
            Highlight::Variable => theme.variable.to_ratatui(),
            Highlight::Constant => theme.constant.to_ratatui(),
            Highlight::String => theme.string.to_ratatui(),
            Highlight::Number => theme.number.to_ratatui(),
            Highlight::Comment => theme.comment.to_ratatui(),
            Highlight::Operator => theme.operator.to_ratatui(),
            Highlight::Punctuation => theme.punctuation.to_ratatui(),
            Highlight::Property => theme.variable.to_ratatui(),
            Highlight::Parameter => theme.variable.to_ratatui(),
            Highlight::Label => theme.constant.to_ratatui(),
            Highlight::Namespace => theme.type_name.to_ratatui(),
            Highlight::Attribute => theme.keyword.to_ratatui(),
        }
    }

    /// Innermost highlight covering `byte_pos`. When several captures cover
    /// the same node, the earliest query pattern wins (tree-sitter convention);
    /// `min_by_key` keeps the first of equal spans, which is in pattern order.
    fn find_highlight(byte_pos: usize, highlights: &[HighlightSpan]) -> Option<Highlight> {
        highlights
            .iter()
            .take_while(|span| span.start <= byte_pos)
            .filter(|span| byte_pos < span.end)
            .min_by_key(|span| span.end - span.start)
            .map(|span| span.highlight)
    }

    fn render_view(frame: &mut Frame, area: Rect, editor: &Editor, view_id: ViewId) {
        let Some(view) = editor.views.get(&view_id) else {
            return;
        };
        let Some(doc) = editor.documents.get(&view.doc_id) else {
            return;
        };
        let is_focused = editor.tree.focus() == view_id;

        let gutter_width = view.gutter_width.min(area.width);
        let gutter_area = Rect {
            width: gutter_width,
            ..area
        };
        let text_area = Rect {
            x: area.x + gutter_width,
            width: area.width - gutter_width,
            ..area
        };

        let first_line = view.scroll_y;
        let last_line = (first_line + area.height as usize).min(doc.len_lines());
        let selection = doc.selection(view_id);
        let cursor_line = doc
            .rope
            .char_to_line(selection.cursor().min(doc.len_chars()));

        frame.render_widget(
            Paragraph::new(Self::gutter_lines(
                editor,
                first_line,
                last_line,
                area.height,
                gutter_width,
                cursor_line,
            ))
            .style(editor.theme.background.to_ratatui()),
            gutter_area,
        );

        let highlights = doc.highlights(first_line..last_line);
        let mut text_lines: Vec<Line> = (first_line..last_line)
            .map(|line_idx| {
                Self::text_line(
                    editor,
                    doc,
                    view,
                    view_id,
                    is_focused,
                    line_idx,
                    &highlights,
                    text_area.width,
                )
            })
            .collect();
        while text_lines.len() < area.height as usize {
            text_lines.push(Line::from(Span::styled(
                EOF_MARKER,
                editor.theme.comment.to_ratatui(),
            )));
        }

        frame.render_widget(
            Paragraph::new(text_lines).style(editor.theme.background.to_ratatui()),
            text_area,
        );
    }

    fn gutter_lines(
        editor: &Editor,
        first_line: usize,
        last_line: usize,
        height: u16,
        gutter_width: u16,
        cursor_line: usize,
    ) -> Vec<Line<'static>> {
        let number_width = (gutter_width as usize).saturating_sub(2);
        let mut lines: Vec<Line> = (first_line..last_line)
            .map(|line_idx| {
                let style = if line_idx == cursor_line {
                    editor.theme.line_number_current.to_ratatui()
                } else {
                    editor.theme.line_number.to_ratatui()
                };
                Line::from(Span::styled(
                    format!(" {:>number_width$} ", line_idx + 1),
                    style,
                ))
            })
            .collect();
        lines.resize(height as usize, Line::default());
        lines
    }

    #[allow(clippy::too_many_arguments)]
    fn text_line(
        editor: &Editor,
        doc: &Document,
        view: &View,
        view_id: ViewId,
        is_focused: bool,
        line_idx: usize,
        highlights: &[HighlightSpan],
        width: u16,
    ) -> Line<'static> {
        let tab_width = editor.config.editor.tab_width;
        let selection = doc.selection(view_id);
        let primary_idx = selection.primary_idx();
        let line = doc.rope.line(line_idx);
        let line_start_char = doc.rope.line_to_char(line_idx);
        let line_start_byte = doc.rope.line_to_byte(line_idx);
        let view_start = view.scroll_x;
        let view_end = view.scroll_x + width as usize;

        let selection_style = editor.theme.selection.to_ratatui();
        let cursor_style = editor.theme.cursor.to_ratatui();
        let default_style = editor.theme.foreground.to_ratatui();

        let mut builder = SpanBuilder::default();
        let mut col = 0;
        let mut byte_offset = 0;

        for (i, ch) in line.chars().enumerate() {
            if ch == '\n' || ch == '\r' || col >= view_end {
                break;
            }
            let char_idx = line_start_char + i;
            let byte_pos = line_start_byte + byte_offset;
            let ch_width = char_display_width(ch, col, tab_width);
            byte_offset += ch.len_utf8();

            let (start_col, end_col) = (col, col + ch_width);
            col = end_col;
            if end_col <= view_start {
                continue;
            }

            // Secondary cursors (and the primary one in unfocused views) have
            // no terminal cursor, so draw them as a block.
            let is_drawn_cursor = selection
                .ranges()
                .iter()
                .enumerate()
                .any(|(idx, r)| r.head == char_idx && (!is_focused || idx != primary_idx));
            let in_selection = selection.ranges().iter().any(|r| r.contains(char_idx));

            let style = if is_drawn_cursor {
                cursor_style
            } else if in_selection {
                selection_style
            } else if let Some(highlight) = Self::find_highlight(byte_pos, highlights) {
                Self::highlight_style(highlight, editor)
            } else {
                default_style
            };

            // A wide char or tab cut by the left or right edge shows as spaces.
            let visible = end_col.min(view_end) - start_col.max(view_start);
            if ch == '\t' || visible < ch_width {
                builder.push(&" ".repeat(visible), style);
            } else if ch.is_control() {
                builder.push(&CONTROL_REPLACEMENT.to_string(), style);
            } else {
                builder.push(&ch.to_string(), style);
            }
        }

        // Cursors at the end of the line sit on an empty cell.
        let line_end_char = line_start_char + doc.rope.line_len_chars(line_idx);
        let cursor_at_eol = selection
            .ranges()
            .iter()
            .enumerate()
            .any(|(idx, r)| r.head == line_end_char && (!is_focused || idx != primary_idx));
        if cursor_at_eol && col >= view_start && col < view_end {
            builder.push(" ", cursor_style);
        }

        builder.finish()
    }

    fn render_dividers(frame: &mut Frame, editor: &Editor, dividers: &[(Area, Layout)]) {
        let style = editor.theme.divider.to_ratatui();
        for (area, layout) in dividers {
            let rect = to_rect(*area);
            let lines: Vec<Line> = match layout {
                Layout::Vertical => (0..rect.height).map(|_| Line::from("│")).collect(),
                Layout::Horizontal => vec![Line::from("─".repeat(rect.width as usize))],
            };
            frame.render_widget(Paragraph::new(lines).style(style), rect);
        }
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for EditorView {
    fn render(&self, frame: &mut Frame, _area: Rect, ctx: &Context) {
        let editor = &*ctx.editor;
        let layout = editor.layout();
        for (view_id, area) in &layout.views {
            Self::render_view(frame, to_rect(*area), editor, *view_id);
        }
        Self::render_dividers(frame, editor, &layout.dividers);
    }

    fn cursor(&self, _area: Rect, ctx: &Context) -> Option<(u16, u16)> {
        let editor = &*ctx.editor;
        let view_id = editor.tree.focus();
        let view = editor.current_view();
        let doc = editor.current_doc();
        let (_, area) = editor
            .layout()
            .views
            .into_iter()
            .find(|(id, _)| *id == view_id)?;

        let pos = doc.rope.char_to_position(doc.selection(view_id).cursor());
        let col = display_col(
            doc.rope.line(pos.line),
            pos.col,
            editor.config.editor.tab_width,
        );

        if pos.line < view.scroll_y || col < view.scroll_x {
            return None;
        }
        let screen_y = pos.line - view.scroll_y;
        let screen_x = col - view.scroll_x + view.gutter_width as usize;
        if screen_y >= area.height as usize || screen_x >= area.width as usize {
            return None;
        }

        Some((area.x + screen_x as u16, area.y + screen_y as u16))
    }
}

fn to_rect(area: Area) -> Rect {
    Rect::new(area.x, area.y, area.width, area.height)
}

/// Collects characters into as few spans as possible by merging runs that
/// share a style.
#[derive(Default)]
struct SpanBuilder {
    spans: Vec<Span<'static>>,
    current: String,
    style: Style,
}

impl SpanBuilder {
    fn push(&mut self, text: &str, style: Style) {
        if style != self.style && !self.current.is_empty() {
            self.flush();
        }
        self.style = style;
        self.current.push_str(text);
    }

    fn flush(&mut self) {
        let text = std::mem::take(&mut self.current);
        self.spans.push(Span::styled(text, self.style));
    }

    fn finish(mut self) -> Line<'static> {
        if !self.current.is_empty() {
            self.flush();
        }
        Line::from(self.spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_beats_generic_identifier() {
        let spans = lite_view::highlighter().highlight("rust", "fn main() {}");
        let main_start = "fn ".len();
        assert_eq!(
            EditorView::find_highlight(main_start, &spans),
            Some(Highlight::Function)
        );
        assert_eq!(
            EditorView::find_highlight(0, &spans),
            Some(Highlight::Keyword)
        );
    }
}
