use crate::Position;
use ropey::{Rope, RopeSlice};

/// Extension trait for Rope with utility methods
pub trait RopeExt {
    /// Get the line at the given index, or None if out of bounds
    fn get_line(&self, line_idx: usize) -> Option<RopeSlice<'_>>;

    /// Get line count (excluding trailing empty line if file ends with newline)
    fn len_lines_display(&self) -> usize;

    /// Get the byte offset at the start of a line
    fn line_to_byte_start(&self, line_idx: usize) -> usize;

    /// Get the byte offset at the end of a line (before newline)
    fn line_to_byte_end(&self, line_idx: usize) -> usize;

    /// Check if the rope ends with a newline
    fn ends_with_newline(&self) -> bool;

    /// Get the length of a line in bytes (excluding newline)
    fn line_len_bytes(&self, line_idx: usize) -> usize;

    /// Get the length of a line in chars (excluding newline)
    fn line_len_chars(&self, line_idx: usize) -> usize;

    /// Convert a char index to Position
    fn char_to_position(&self, char_idx: usize) -> Position;

    /// Convert Position to char index
    fn position_to_char(&self, pos: Position) -> usize;

    /// Find word boundaries around a position
    fn word_at(&self, char_idx: usize) -> (usize, usize);

    /// Check if char at index is a word character
    fn is_word_char(&self, char_idx: usize) -> bool;
}

impl RopeExt for Rope {
    fn get_line(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        if line_idx < self.len_lines() {
            Some(self.line(line_idx))
        } else {
            None
        }
    }

    fn len_lines_display(&self) -> usize {
        let lines = self.len_lines();
        if lines > 0 && self.ends_with_newline() {
            lines
        } else {
            lines.max(1)
        }
    }

    fn line_to_byte_start(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            self.len_bytes()
        } else {
            self.line_to_byte(line_idx)
        }
    }

    fn line_to_byte_end(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            self.len_bytes()
        } else {
            let line = self.line(line_idx);
            let start = self.line_to_byte(line_idx);
            start + line_len_without_newline(line)
        }
    }

    fn ends_with_newline(&self) -> bool {
        self.len_chars() > 0 && {
            let last = self.char(self.len_chars() - 1);
            last == '\n' || last == '\r'
        }
    }

    fn line_len_bytes(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            0
        } else {
            line_len_without_newline(self.line(line_idx))
        }
    }

    fn line_len_chars(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            0
        } else {
            line_len_chars_without_newline(self.line(line_idx))
        }
    }

    fn char_to_position(&self, char_idx: usize) -> Position {
        let char_idx = char_idx.min(self.len_chars());
        let line = self.char_to_line(char_idx);
        let line_start = self.line_to_char(line);
        let col = char_idx - line_start;
        Position { line, col }
    }

    fn position_to_char(&self, pos: Position) -> usize {
        if pos.line >= self.len_lines() {
            return self.len_chars();
        }
        let line_start = self.line_to_char(pos.line);
        let line_len = self.line_len_chars(pos.line);
        line_start + pos.col.min(line_len)
    }

    fn word_at(&self, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(self.len_chars().saturating_sub(1));

        // Find start of word
        let mut start = char_idx;
        while start > 0 && self.is_word_char(start - 1) {
            start -= 1;
        }

        // Find end of word
        let mut end = char_idx;
        while end < self.len_chars() && self.is_word_char(end) {
            end += 1;
        }

        (start, end)
    }

    fn is_word_char(&self, char_idx: usize) -> bool {
        if char_idx >= self.len_chars() {
            return false;
        }
        let c = self.char(char_idx);
        c.is_alphanumeric() || c == '_'
    }
}

/// Get line length in bytes without trailing newline
fn line_len_without_newline(line: RopeSlice) -> usize {
    let len = line.len_bytes();
    if len == 0 {
        return 0;
    }

    let mut end = len;
    if end > 0 {
        let last_byte = line.byte(end - 1);
        if last_byte == b'\n' {
            end -= 1;
        }
        if end > 0 {
            let prev_byte = line.byte(end - 1);
            if prev_byte == b'\r' {
                end -= 1;
            }
        }
    }
    end
}

/// Get line length in chars without trailing newline
fn line_len_chars_without_newline(line: RopeSlice) -> usize {
    let len = line.len_chars();
    if len == 0 {
        return 0;
    }

    let mut end = len;
    if end > 0 {
        let last_char = line.char(end - 1);
        if last_char == '\n' {
            end -= 1;
        }
        if end > 0 {
            let prev_char = line.char(end - 1);
            if prev_char == '\r' {
                end -= 1;
            }
        }
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_len_without_newline() {
        let rope = Rope::from("hello\nworld\n");
        assert_eq!(rope.line_len_bytes(0), 5);
        assert_eq!(rope.line_len_bytes(1), 5);
    }

    #[test]
    fn test_word_at() {
        let rope = Rope::from("hello world");
        assert_eq!(rope.word_at(2), (0, 5));
        assert_eq!(rope.word_at(7), (6, 11));
    }

    #[test]
    fn test_position_conversion() {
        let rope = Rope::from("hello\nworld");
        assert_eq!(rope.char_to_position(0), Position::new(0, 0));
        assert_eq!(rope.char_to_position(5), Position::new(0, 5));
        assert_eq!(rope.char_to_position(6), Position::new(1, 0));
        assert_eq!(rope.position_to_char(Position::new(1, 0)), 6);
    }
}
