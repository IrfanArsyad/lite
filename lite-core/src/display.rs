//! Conversion between char offsets and terminal display columns.

use ropey::RopeSlice;
use unicode_width::UnicodeWidthChar;

/// Character shown in place of control characters that have no glyph.
pub const CONTROL_REPLACEMENT: char = '\u{FFFD}';

/// Display width of `c` when it starts at display column `col`.
///
/// Tabs expand to the next tab stop, so their width depends on `col`.
pub fn char_display_width(c: char, col: usize, tab_width: usize) -> usize {
    match c {
        '\t' => {
            let tab_width = tab_width.max(1);
            tab_width - col % tab_width
        }
        '\n' | '\r' => 0,
        c if c.is_control() => 1,
        c => c.width().unwrap_or(0),
    }
}

/// Display column of the char at `char_col` within `line`.
pub fn display_col(line: RopeSlice, char_col: usize, tab_width: usize) -> usize {
    line.chars()
        .take(char_col)
        .fold(0, |col, c| col + char_display_width(c, col, tab_width))
}

/// Char column within `line` whose display span covers `target` display column.
///
/// Columns past the end of the line clamp to the line length (excluding the
/// line ending).
pub fn char_col_at_display(line: RopeSlice, target: usize, tab_width: usize) -> usize {
    let mut col = 0;
    for (idx, c) in line.chars().enumerate() {
        if c == '\n' || c == '\r' {
            return idx;
        }
        let width = char_display_width(c, col, tab_width);
        if col + width > target {
            return idx;
        }
        col += width;
    }
    line.len_chars()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_tab_expands_to_tab_stop() {
        let rope = Rope::from("a\tb");
        assert_eq!(display_col(rope.slice(..), 1, 4), 1);
        assert_eq!(display_col(rope.slice(..), 2, 4), 4);
    }

    #[test]
    fn test_wide_chars() {
        let rope = Rope::from("日本x");
        assert_eq!(display_col(rope.slice(..), 2, 4), 4);
        assert_eq!(char_col_at_display(rope.slice(..), 3, 4), 1);
        assert_eq!(char_col_at_display(rope.slice(..), 4, 4), 2);
    }

    #[test]
    fn test_char_col_clamps_to_line_end() {
        let rope = Rope::from("abc\n");
        assert_eq!(char_col_at_display(rope.line(0), 10, 4), 3);
    }
}
