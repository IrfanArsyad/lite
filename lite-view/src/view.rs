use crate::DocumentId;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique identifier for views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(usize);

impl ViewId {
    pub fn next() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ViewId {
    fn default() -> Self {
        Self::next()
    }
}

/// A view into a document
#[derive(Debug)]
pub struct View {
    /// Unique identifier
    pub id: ViewId,
    /// The document being viewed
    pub doc_id: DocumentId,
    /// Vertical scroll offset (first visible line)
    pub scroll_y: usize,
    /// Horizontal scroll offset (first visible column)
    pub scroll_x: usize,
    /// Viewport width in characters
    pub width: u16,
    /// Viewport height in lines
    pub height: u16,
    /// Gutter width (line numbers, etc.)
    pub gutter_width: u16,
}

impl View {
    /// Create a new view for a document
    pub fn new(doc_id: DocumentId) -> Self {
        Self {
            id: ViewId::next(),
            doc_id,
            scroll_y: 0,
            scroll_x: 0,
            width: 80,
            height: 24,
            gutter_width: 4,
        }
    }

    /// Get the effective editing area width
    pub fn text_width(&self) -> u16 {
        self.width.saturating_sub(self.gutter_width)
    }

    /// Update viewport size
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    /// Ensure cursor is visible, adjusting scroll if needed
    pub fn ensure_cursor_visible(&mut self, cursor_line: usize, cursor_col: usize, scrolloff: usize) {
        let scrolloff = scrolloff.min(self.height as usize / 2);

        // Vertical scrolling
        if cursor_line < self.scroll_y + scrolloff {
            self.scroll_y = cursor_line.saturating_sub(scrolloff);
        }
        let bottom_limit = self.scroll_y + self.height as usize - scrolloff - 1;
        if cursor_line > bottom_limit {
            self.scroll_y = cursor_line + scrolloff + 1 - self.height as usize;
        }

        // Horizontal scrolling
        let text_width = self.text_width() as usize;
        if cursor_col < self.scroll_x {
            self.scroll_x = cursor_col;
        }
        if cursor_col >= self.scroll_x + text_width {
            self.scroll_x = cursor_col - text_width + 1;
        }
    }

    /// Get the range of visible lines
    pub fn visible_lines(&self) -> std::ops::Range<usize> {
        self.scroll_y..self.scroll_y + self.height as usize
    }

    /// Check if a line is visible
    pub fn is_line_visible(&self, line: usize) -> bool {
        line >= self.scroll_y && line < self.scroll_y + self.height as usize
    }

    /// Scroll by a number of lines (positive = down)
    pub fn scroll(&mut self, delta: isize, max_lines: usize) {
        if delta > 0 {
            self.scroll_y = (self.scroll_y + delta as usize).min(max_lines.saturating_sub(1));
        } else {
            self.scroll_y = self.scroll_y.saturating_sub((-delta) as usize);
        }
    }

    /// Scroll to center a line in the viewport
    pub fn center_on_line(&mut self, line: usize) {
        self.scroll_y = line.saturating_sub(self.height as usize / 2);
    }

    /// Update gutter width based on line count
    pub fn update_gutter_width(&mut self, line_count: usize) {
        // Calculate digits needed for line numbers + padding
        let digits = if line_count == 0 {
            1
        } else {
            (line_count as f64).log10().floor() as u16 + 1
        };
        self.gutter_width = digits + 2; // 1 space on each side
    }
}
