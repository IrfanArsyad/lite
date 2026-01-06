use ropey::{Rope, RopeSlice};
use std::cmp::Ordering;

/// A position in a text document represented as line and column.
/// Both are 0-indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    /// Convert a byte offset to a Position
    pub fn from_offset(rope: &Rope, offset: usize) -> Self {
        let offset = offset.min(rope.len_bytes());
        let line = rope.byte_to_line(offset);
        let line_start = rope.line_to_byte(line);
        let col = offset - line_start;
        Self { line, col }
    }

    /// Convert Position to byte offset
    pub fn to_offset(&self, rope: &Rope) -> usize {
        if self.line >= rope.len_lines() {
            return rope.len_bytes();
        }
        let line_start = rope.line_to_byte(self.line);
        let line_len = rope.line(self.line).len_bytes();
        line_start + self.col.min(line_len.saturating_sub(1).max(0))
    }

    /// Convert Position to char offset
    pub fn to_char_offset(&self, rope: &Rope) -> usize {
        if self.line >= rope.len_lines() {
            return rope.len_chars();
        }
        let line_start = rope.line_to_char(self.line);
        let line = rope.line(self.line);
        // Convert byte column to char column
        let col_chars = byte_to_char_col(line, self.col);
        line_start + col_chars
    }

    /// Check if position is valid for the given rope
    pub fn is_valid(&self, rope: &Rope) -> bool {
        if self.line >= rope.len_lines() {
            return false;
        }
        let line = rope.line(self.line);
        self.col <= line.len_bytes()
    }

    /// Clamp position to valid range
    pub fn clamp(&self, rope: &Rope) -> Self {
        let line = self.line.min(rope.len_lines().saturating_sub(1));
        let line_rope = rope.line(line);
        let max_col = line_rope.len_bytes().saturating_sub(if line_ends_with_newline(line_rope) {
            1
        } else {
            0
        });
        Self {
            line,
            col: self.col.min(max_col),
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.line.cmp(&other.line) {
            Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}

/// Convert byte column to char column within a line
fn byte_to_char_col(line: RopeSlice, byte_col: usize) -> usize {
    let byte_col = byte_col.min(line.len_bytes());
    line.byte_slice(..byte_col).len_chars()
}

/// Check if a line ends with a newline character
fn line_ends_with_newline(line: RopeSlice) -> bool {
    line.len_chars() > 0 && {
        let last_char = line.char(line.len_chars() - 1);
        last_char == '\n' || last_char == '\r'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_offset() {
        let rope = Rope::from("hello\nworld\n");
        assert_eq!(Position::from_offset(&rope, 0), Position::new(0, 0));
        assert_eq!(Position::from_offset(&rope, 5), Position::new(0, 5));
        assert_eq!(Position::from_offset(&rope, 6), Position::new(1, 0));
        assert_eq!(Position::from_offset(&rope, 11), Position::new(1, 5));
    }

    #[test]
    fn test_position_to_offset() {
        let rope = Rope::from("hello\nworld\n");
        assert_eq!(Position::new(0, 0).to_offset(&rope), 0);
        assert_eq!(Position::new(0, 5).to_offset(&rope), 5);
        assert_eq!(Position::new(1, 0).to_offset(&rope), 6);
        assert_eq!(Position::new(1, 5).to_offset(&rope), 11);
    }

    #[test]
    fn test_position_ordering() {
        assert!(Position::new(0, 0) < Position::new(0, 1));
        assert!(Position::new(0, 5) < Position::new(1, 0));
        assert!(Position::new(1, 0) > Position::new(0, 100));
    }
}
