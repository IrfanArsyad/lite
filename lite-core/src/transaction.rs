use crate::Selection;
use ropey::Rope;
use std::borrow::Cow;

/// A single change operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Retain n characters unchanged
    Retain(usize),
    /// Insert text
    Insert(String),
    /// Delete n characters
    Delete(usize),
}

/// A change at a specific position
#[derive(Debug, Clone)]
pub struct Change {
    /// Start position (char index)
    pub start: usize,
    /// End position (char index) - for deletion
    pub end: usize,
    /// Text to insert (empty for pure deletion)
    pub insert: Cow<'static, str>,
}

impl Change {
    /// Create an insertion
    pub fn insert(pos: usize, text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            start: pos,
            end: pos,
            insert: text.into(),
        }
    }

    /// Create a deletion
    pub fn delete(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            insert: Cow::Borrowed(""),
        }
    }

    /// Create a replacement
    pub fn replace(start: usize, end: usize, text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            start,
            end,
            insert: text.into(),
        }
    }
}

/// A set of changes that can be applied atomically
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    /// Original document length (in chars)
    pub doc_len: usize,
    /// List of operations
    pub ops: Vec<Operation>,
}

impl ChangeSet {
    /// Create an empty changeset for a document of given length
    pub fn new(doc_len: usize) -> Self {
        Self {
            doc_len,
            ops: Vec::new(),
        }
    }

    /// Create a changeset from a single change
    pub fn from_change(doc_len: usize, change: &Change) -> Self {
        let mut cs = Self::new(doc_len);

        // Retain up to start
        if change.start > 0 {
            cs.ops.push(Operation::Retain(change.start));
        }

        // Delete if needed
        if change.end > change.start {
            cs.ops.push(Operation::Delete(change.end - change.start));
        }

        // Insert if needed
        if !change.insert.is_empty() {
            cs.ops.push(Operation::Insert(change.insert.to_string()));
        }

        // Retain rest
        if change.end < doc_len {
            cs.ops.push(Operation::Retain(doc_len - change.end));
        }

        cs
    }

    /// Check if the changeset is empty (no actual changes)
    pub fn is_empty(&self) -> bool {
        self.ops.iter().all(|op| matches!(op, Operation::Retain(_)))
    }

    /// Calculate the new document length after applying this changeset
    pub fn new_len(&self) -> usize {
        let mut len = 0;
        for op in &self.ops {
            match op {
                Operation::Retain(n) => len += n,
                Operation::Insert(s) => len += s.chars().count(),
                Operation::Delete(_) => {}
            }
        }
        len
    }

    /// Apply this changeset to a rope
    pub fn apply(&self, rope: &mut Rope) {
        let mut pos = 0;

        for op in &self.ops {
            match op {
                Operation::Retain(n) => {
                    pos += n;
                }
                Operation::Insert(text) => {
                    rope.insert(pos, text);
                    pos += text.chars().count();
                }
                Operation::Delete(n) => {
                    rope.remove(pos..pos + n);
                }
            }
        }
    }

    /// Create the inverse of this changeset (for undo)
    pub fn invert(&self, original: &Rope) -> Self {
        let mut inverted = Self::new(self.new_len());
        let mut pos = 0;

        for op in &self.ops {
            match op {
                Operation::Retain(n) => {
                    inverted.ops.push(Operation::Retain(*n));
                    pos += n;
                }
                Operation::Insert(text) => {
                    // Insert becomes delete
                    inverted.ops.push(Operation::Delete(text.chars().count()));
                }
                Operation::Delete(n) => {
                    // Delete becomes insert (original text)
                    let deleted_text: String = original.slice(pos..pos + n).chars().collect();
                    inverted.ops.push(Operation::Insert(deleted_text));
                    pos += n;
                }
            }
        }

        inverted
    }

    /// Compose two changesets into one
    pub fn compose(&self, other: &ChangeSet) -> Option<ChangeSet> {
        if self.new_len() != other.doc_len {
            return None;
        }

        let mut composed = ChangeSet::new(self.doc_len);
        let mut ops_a = self.ops.iter().peekable();
        let mut ops_b = other.ops.iter().peekable();
        let mut len_a = 0;
        let mut len_b = 0;

        loop {
            let op_a = ops_a.peek();
            let op_b = ops_b.peek();

            match (op_a, op_b) {
                (None, None) => break,
                (Some(Operation::Insert(s)), _) => {
                    composed.ops.push(Operation::Insert(s.clone()));
                    ops_a.next();
                }
                (_, Some(Operation::Delete(n))) => {
                    composed.ops.push(Operation::Delete(*n));
                    ops_b.next();
                }
                (None, Some(op)) => {
                    composed.ops.push((*op).clone());
                    ops_b.next();
                }
                (Some(op), None) => {
                    composed.ops.push((*op).clone());
                    ops_a.next();
                }
                (Some(Operation::Retain(a)), Some(Operation::Retain(b))) => {
                    let min = (*a - len_a).min(*b - len_b);
                    composed.ops.push(Operation::Retain(min));
                    len_a += min;
                    len_b += min;
                    if len_a == *a {
                        ops_a.next();
                        len_a = 0;
                    }
                    if len_b == *b {
                        ops_b.next();
                        len_b = 0;
                    }
                }
                (Some(Operation::Retain(a)), Some(Operation::Insert(s))) => {
                    composed.ops.push(Operation::Insert(s.clone()));
                    ops_b.next();
                }
                (Some(Operation::Delete(n)), Some(Operation::Retain(r))) => {
                    let min = (*n - len_a).min(*r - len_b);
                    composed.ops.push(Operation::Delete(min));
                    len_a += min;
                    len_b += min;
                    if len_a == *n {
                        ops_a.next();
                        len_a = 0;
                    }
                    if len_b == *r {
                        ops_b.next();
                        len_b = 0;
                    }
                }
                (Some(Operation::Delete(n)), Some(Operation::Insert(s))) => {
                    composed.ops.push(Operation::Delete(*n));
                    composed.ops.push(Operation::Insert(s.clone()));
                    ops_a.next();
                    ops_b.next();
                }
                (Some(Operation::Retain(_)), Some(Operation::Delete(_))) => {
                    // Already handled above
                    unreachable!()
                }
                (Some(Operation::Insert(_)), Some(Operation::Retain(_))) => {
                    // Already handled above
                    unreachable!()
                }
                (Some(Operation::Insert(_)), Some(Operation::Insert(_))) => {
                    // Already handled above
                    unreachable!()
                }
            }
        }

        Some(composed)
    }

    /// Map a position through this changeset
    pub fn map_pos(&self, mut pos: usize) -> usize {
        let mut old_pos = 0;
        let mut new_pos = 0;

        for op in &self.ops {
            if old_pos > pos {
                break;
            }

            match op {
                Operation::Retain(n) => {
                    old_pos += n;
                    new_pos += n;
                }
                Operation::Insert(s) => {
                    let len = s.chars().count();
                    if old_pos <= pos {
                        new_pos += len;
                    }
                }
                Operation::Delete(n) => {
                    if old_pos + n <= pos {
                        pos -= n;
                    } else if old_pos < pos {
                        pos = old_pos;
                    }
                    old_pos += n;
                }
            }
        }

        new_pos.min(pos)
    }
}

/// A transaction groups changes with selection and metadata
#[derive(Debug, Clone)]
pub struct Transaction {
    /// The changes to apply
    pub changes: ChangeSet,
    /// The new selection after applying the transaction
    pub selection: Option<Selection>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(changes: ChangeSet) -> Self {
        Self {
            changes,
            selection: None,
        }
    }

    /// Create a transaction from a single change
    pub fn change(doc_len: usize, change: Change) -> Self {
        Self::new(ChangeSet::from_change(doc_len, &change))
    }

    /// Create an insert transaction
    pub fn insert(doc_len: usize, pos: usize, text: impl Into<Cow<'static, str>>) -> Self {
        Self::change(doc_len, Change::insert(pos, text))
    }

    /// Create a delete transaction
    pub fn delete(doc_len: usize, start: usize, end: usize) -> Self {
        Self::change(doc_len, Change::delete(start, end))
    }

    /// Create a replace transaction
    pub fn replace(
        doc_len: usize,
        start: usize,
        end: usize,
        text: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::change(doc_len, Change::replace(start, end, text))
    }

    /// Set the selection for this transaction
    pub fn with_selection(mut self, selection: Selection) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Apply this transaction to a rope
    pub fn apply(&self, rope: &mut Rope) {
        self.changes.apply(rope);
    }

    /// Create the inverse transaction (for undo)
    pub fn invert(&self, original: &Rope, original_selection: &Selection) -> Self {
        Self {
            changes: self.changes.invert(original),
            selection: Some(original_selection.clone()),
        }
    }

    /// Check if the transaction is empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut rope = Rope::from("hello world");
        let tx = Transaction::insert(11, 5, " beautiful");
        tx.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello beautiful world");
    }

    #[test]
    fn test_delete() {
        let mut rope = Rope::from("hello beautiful world");
        let tx = Transaction::delete(21, 5, 15);
        tx.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello world");
    }

    #[test]
    fn test_replace() {
        let mut rope = Rope::from("hello world");
        let tx = Transaction::replace(11, 6, 11, "rust");
        tx.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello rust");
    }

    #[test]
    fn test_invert() {
        let original = Rope::from("hello world");
        let mut rope = original.clone();

        let tx = Transaction::insert(11, 5, " beautiful");
        tx.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello beautiful world");

        let inverse = tx.invert(&original, &Selection::point(0));
        inverse.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello world");
    }

    #[test]
    fn test_changeset_new_len() {
        let cs = ChangeSet::from_change(11, &Change::insert(5, " beautiful"));
        assert_eq!(cs.new_len(), 21);

        let cs = ChangeSet::from_change(21, &Change::delete(5, 15));
        assert_eq!(cs.new_len(), 11);
    }
}
