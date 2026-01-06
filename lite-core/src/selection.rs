use smallvec::SmallVec;
use std::cmp::Ordering;

/// A range in the text, represented by anchor and head positions.
/// The anchor is the fixed point, head is the moving point (cursor).
/// Positions are char indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// Fixed point of the selection
    pub anchor: usize,
    /// Moving point (cursor position)
    pub head: usize,
}

impl Range {
    /// Create a new range with anchor and head at the same position (cursor)
    pub fn point(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    /// Create a new range from anchor to head
    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    /// Get the start of the range (min of anchor and head)
    pub fn start(&self) -> usize {
        self.anchor.min(self.head)
    }

    /// Get the end of the range (max of anchor and head)
    pub fn end(&self) -> usize {
        self.anchor.max(self.head)
    }

    /// Check if this is a point (no selection)
    pub fn is_point(&self) -> bool {
        self.anchor == self.head
    }

    /// Get the length of the selection
    pub fn len(&self) -> usize {
        self.end() - self.start()
    }

    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the range contains a position
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start() && pos < self.end()
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &Range) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }

    /// Merge with another range if they overlap or touch
    pub fn merge(&self, other: &Range) -> Option<Range> {
        if self.end() >= other.start() && other.end() >= self.start() {
            // They overlap or touch
            Some(Range::new(
                self.start().min(other.start()),
                self.end().max(other.end()),
            ))
        } else {
            None
        }
    }

    /// Move the range by an offset
    pub fn translate(&self, offset: isize) -> Self {
        let translate = |pos: usize| {
            if offset >= 0 {
                pos.saturating_add(offset as usize)
            } else {
                pos.saturating_sub((-offset) as usize)
            }
        };
        Self {
            anchor: translate(self.anchor),
            head: translate(self.head),
        }
    }

    /// Extend the selection to include a position
    pub fn extend_to(&self, pos: usize) -> Self {
        Self {
            anchor: self.anchor,
            head: pos,
        }
    }

    /// Get direction of selection (-1 for backward, 0 for point, 1 for forward)
    pub fn direction(&self) -> i8 {
        match self.head.cmp(&self.anchor) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// Flip the selection (swap anchor and head)
    pub fn flip(&self) -> Self {
        Self {
            anchor: self.head,
            head: self.anchor,
        }
    }

    /// Collapse to cursor position (head)
    pub fn collapse(&self) -> Self {
        Self::point(self.head)
    }
}

impl Default for Range {
    fn default() -> Self {
        Self::point(0)
    }
}

/// A selection consisting of one or more ranges.
/// Supports multiple cursors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// All ranges in the selection (invariant: sorted and non-overlapping)
    ranges: SmallVec<[Range; 1]>,
    /// Index of the primary (active) cursor
    primary_idx: usize,
}

impl Default for Selection {
    fn default() -> Self {
        Self::point(0)
    }
}

impl Selection {
    /// Create a selection with a single cursor at position
    pub fn point(pos: usize) -> Self {
        Self {
            ranges: smallvec::smallvec![Range::point(pos)],
            primary_idx: 0,
        }
    }

    /// Create a selection from a single range
    pub fn single(range: Range) -> Self {
        Self {
            ranges: smallvec::smallvec![range],
            primary_idx: 0,
        }
    }

    /// Create a selection from multiple ranges
    pub fn new(ranges: SmallVec<[Range; 1]>, primary_idx: usize) -> Self {
        let mut sel = Self {
            ranges,
            primary_idx: 0,
        };
        sel.normalize();
        sel.primary_idx = primary_idx.min(sel.ranges.len().saturating_sub(1));
        sel
    }

    /// Get all ranges
    pub fn ranges(&self) -> &[Range] {
        &self.ranges
    }

    /// Get the number of ranges
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Check if selection is empty (should never be true)
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Get the primary range
    pub fn primary(&self) -> &Range {
        &self.ranges[self.primary_idx]
    }

    /// Get the primary cursor position (head of primary range)
    pub fn cursor(&self) -> usize {
        self.primary().head
    }

    /// Get the primary range index
    pub fn primary_idx(&self) -> usize {
        self.primary_idx
    }

    /// Set the primary range index
    pub fn set_primary_idx(&mut self, idx: usize) {
        self.primary_idx = idx.min(self.ranges.len().saturating_sub(1));
    }

    /// Check if any range has a selection (not just cursor)
    pub fn has_selection(&self) -> bool {
        self.ranges.iter().any(|r| !r.is_point())
    }

    /// Transform all ranges with a function
    pub fn transform<F>(&self, f: F) -> Self
    where
        F: Fn(&Range) -> Range,
    {
        let ranges: SmallVec<[Range; 1]> = self.ranges.iter().map(f).collect();
        Self::new(ranges, self.primary_idx)
    }

    /// Add a new cursor at position
    pub fn add_cursor(&mut self, pos: usize) {
        self.ranges.push(Range::point(pos));
        self.normalize();
    }

    /// Add a new range
    pub fn add_range(&mut self, range: Range) {
        self.ranges.push(range);
        self.normalize();
    }

    /// Remove all but the primary cursor
    pub fn into_single(&self) -> Self {
        Self::single(*self.primary())
    }

    /// Collapse all selections to cursors
    pub fn collapse(&self) -> Self {
        self.transform(|r| r.collapse())
    }

    /// Sort ranges and merge overlapping ones
    fn normalize(&mut self) {
        if self.ranges.is_empty() {
            self.ranges.push(Range::point(0));
            self.primary_idx = 0;
            return;
        }

        // Sort by start position
        self.ranges.sort_by_key(|r| (r.start(), r.end()));

        // Merge overlapping ranges
        let mut merged: SmallVec<[Range; 1]> = SmallVec::new();
        let mut primary_range = *self.primary();

        for range in &self.ranges {
            if let Some(last) = merged.last_mut() {
                if let Some(m) = last.merge(range) {
                    *last = m;
                    continue;
                }
            }
            merged.push(*range);
        }

        // Find new primary index
        self.primary_idx = merged
            .iter()
            .position(|r| r.contains(primary_range.head) || *r == primary_range)
            .unwrap_or(0);

        self.ranges = merged;
    }

    /// Replace all ranges
    pub fn replace(&mut self, ranges: SmallVec<[Range; 1]>) {
        self.ranges = ranges;
        self.normalize();
    }
}

impl From<Range> for Selection {
    fn from(range: Range) -> Self {
        Self::single(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_basics() {
        let r = Range::new(5, 10);
        assert_eq!(r.start(), 5);
        assert_eq!(r.end(), 10);
        assert_eq!(r.len(), 5);
        assert!(!r.is_point());
    }

    #[test]
    fn test_range_point() {
        let r = Range::point(5);
        assert_eq!(r.anchor, 5);
        assert_eq!(r.head, 5);
        assert!(r.is_point());
    }

    #[test]
    fn test_range_overlap() {
        let r1 = Range::new(0, 5);
        let r2 = Range::new(3, 8);
        let r3 = Range::new(10, 15);

        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }

    #[test]
    fn test_selection_normalize() {
        let mut sel = Selection::new(
            smallvec::smallvec![
                Range::new(10, 15),
                Range::new(0, 5),
                Range::new(3, 8),
            ],
            0,
        );

        // Should be sorted and merged: [0, 8) and [10, 15)
        assert_eq!(sel.len(), 2);
        assert_eq!(sel.ranges[0], Range::new(0, 8));
        assert_eq!(sel.ranges[1], Range::new(10, 15));
    }

    #[test]
    fn test_selection_add_cursor() {
        let mut sel = Selection::point(0);
        sel.add_cursor(10);
        assert_eq!(sel.len(), 2);
    }
}
