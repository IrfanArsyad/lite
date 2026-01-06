use ropey::RopeSlice;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
use unicode_width::UnicodeWidthStr;

/// Get the display width of a grapheme cluster
pub fn grapheme_width(grapheme: &str) -> usize {
    if grapheme == "\t" {
        // Tab width is handled separately
        1
    } else if grapheme.chars().any(|c| c.is_control()) {
        // Control characters
        0
    } else {
        UnicodeWidthStr::width(grapheme)
    }
}

/// Iterator over grapheme clusters in a RopeSlice
pub struct RopeGraphemes<'a> {
    text: RopeSlice<'a>,
    chunks: ropey::iter::Chunks<'a>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
}

impl<'a> RopeGraphemes<'a> {
    pub fn new(text: RopeSlice<'a>) -> Self {
        let mut chunks = text.chunks();
        let first_chunk = chunks.next().unwrap_or("");
        Self {
            text,
            chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: 0,
            cursor: GraphemeCursor::new(0, text.len_bytes(), true),
        }
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.cursor.cur_cursor();
        if start >= self.text.len_bytes() {
            return None;
        }

        loop {
            match self
                .cursor
                .next_boundary(self.cur_chunk, self.cur_chunk_start)
            {
                Ok(None) => return None,
                Ok(Some(end)) => {
                    // We found the boundary
                    let start_in_chunk = start - self.cur_chunk_start;
                    let end_in_chunk = end - self.cur_chunk_start;

                    if end_in_chunk <= self.cur_chunk.len() {
                        return Some(&self.cur_chunk[start_in_chunk..end_in_chunk]);
                    } else {
                        // Grapheme spans multiple chunks - need to collect
                        // For simplicity, we'll use the rope's byte_slice
                        let grapheme_bytes = self.text.byte_slice(start..end);
                        // This is a bit inefficient but handles edge cases
                        if let Some(cow) = grapheme_bytes.chunks().next() {
                            return Some(cow);
                        }
                        return None;
                    }
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    // Need more chunks
                    self.cur_chunk_start += self.cur_chunk.len();
                    self.cur_chunk = self.chunks.next().unwrap_or("");
                }
                Err(GraphemeIncomplete::PreContext(n)) => {
                    let ctx_chunk = self
                        .text
                        .byte_slice(..n)
                        .chunks()
                        .last()
                        .unwrap_or("");
                    self.cursor.provide_context(ctx_chunk, n - ctx_chunk.len());
                }
                Err(_) => return None,
            }
        }
    }
}

/// Get the nth next grapheme boundary from a byte position
pub fn nth_next_grapheme(text: RopeSlice, byte_pos: usize, n: usize) -> usize {
    let mut pos = byte_pos;
    for _ in 0..n {
        pos = next_grapheme_boundary(text, pos);
    }
    pos
}

/// Get the nth previous grapheme boundary from a byte position
pub fn nth_prev_grapheme(text: RopeSlice, byte_pos: usize, n: usize) -> usize {
    let mut pos = byte_pos;
    for _ in 0..n {
        pos = prev_grapheme_boundary(text, pos);
    }
    pos
}

/// Get the next grapheme boundary
fn next_grapheme_boundary(text: RopeSlice, byte_pos: usize) -> usize {
    if byte_pos >= text.len_bytes() {
        return text.len_bytes();
    }

    let mut cursor = GraphemeCursor::new(byte_pos, text.len_bytes(), true);
    let mut chunks = text.chunks_at_byte(byte_pos);
    let mut cur_chunk = chunks.0.next().unwrap_or("");
    let mut cur_chunk_start = byte_pos - (byte_pos - text.byte_to_char(byte_pos));

    // Adjust chunk start
    if let Some((chunk, start, _, _)) = text.chunks().enumerate().find_map(|(i, c)| {
        let start: usize = text.chunks().take(i).map(|s| s.len()).sum();
        if start <= byte_pos && byte_pos < start + c.len() {
            Some((c, start, i, c.len()))
        } else {
            None
        }
    }) {
        cur_chunk = chunk;
        cur_chunk_start = start;
    }

    loop {
        match cursor.next_boundary(cur_chunk, cur_chunk_start) {
            Ok(Some(pos)) => return pos,
            Ok(None) => return text.len_bytes(),
            Err(GraphemeIncomplete::NextChunk) => {
                cur_chunk_start += cur_chunk.len();
                cur_chunk = chunks.0.next().unwrap_or("");
            }
            Err(GraphemeIncomplete::PreContext(n)) => {
                let ctx = text.byte_slice(..n);
                let ctx_chunk = ctx.chunks().last().unwrap_or("");
                cursor.provide_context(ctx_chunk, n - ctx_chunk.len());
            }
            Err(_) => return text.len_bytes(),
        }
    }
}

/// Get the previous grapheme boundary
fn prev_grapheme_boundary(text: RopeSlice, byte_pos: usize) -> usize {
    if byte_pos == 0 {
        return 0;
    }

    let byte_pos = byte_pos.min(text.len_bytes());
    let mut cursor = GraphemeCursor::new(byte_pos, text.len_bytes(), true);

    // Find the chunk containing byte_pos
    let mut cur_chunk = "";
    let mut cur_chunk_start = 0;
    let mut offset = 0;

    for chunk in text.chunks() {
        if offset + chunk.len() >= byte_pos {
            cur_chunk = chunk;
            cur_chunk_start = offset;
            break;
        }
        offset += chunk.len();
    }

    loop {
        match cursor.prev_boundary(cur_chunk, cur_chunk_start) {
            Ok(Some(pos)) => return pos,
            Ok(None) => return 0,
            Err(GraphemeIncomplete::PrevChunk) => {
                // Need previous chunk
                let mut offset = 0;
                let mut prev_chunk = "";
                let mut prev_start = 0;

                for chunk in text.chunks() {
                    if offset + chunk.len() >= cur_chunk_start {
                        break;
                    }
                    prev_chunk = chunk;
                    prev_start = offset;
                    offset += chunk.len();
                }

                cur_chunk = prev_chunk;
                cur_chunk_start = prev_start;
            }
            Err(GraphemeIncomplete::PreContext(n)) => {
                let ctx = text.byte_slice(..n);
                let ctx_chunk = ctx.chunks().last().unwrap_or("");
                cursor.provide_context(ctx_chunk, n - ctx_chunk.len());
            }
            Err(_) => return 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_grapheme_width() {
        assert_eq!(grapheme_width("a"), 1);
        assert_eq!(grapheme_width("中"), 2);
        assert_eq!(grapheme_width("\t"), 1);
    }

    #[test]
    fn test_grapheme_iterator() {
        let rope = Rope::from("hello");
        let graphemes: Vec<_> = RopeGraphemes::new(rope.slice(..)).collect();
        assert_eq!(graphemes, vec!["h", "e", "l", "l", "o"]);
    }
}
