use lite_core::Transaction;
use std::collections::VecDeque;

/// Maximum number of undo states to keep
const MAX_HISTORY_SIZE: usize = 1000;

/// Identifies one version of a document's content.
pub type StateId = u64;

/// One undoable step. Several transactions can be grouped into one step
/// (e.g. a run of typed characters) so a single undo reverts them together.
#[derive(Debug)]
pub struct HistoryEntry {
    /// Transactions to apply, in order, to revert this step
    pub txs: Vec<Transaction>,
    /// Content state the document returns to once `txs` are applied
    pub target_state: StateId,
}

/// Undo/redo history for a document
#[derive(Debug, Default)]
pub struct History {
    undo_stack: VecDeque<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    /// Whether the next mergeable edit may join the newest undo entry
    can_merge: bool,
}

impl History {
    /// Create a new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the inverse of a new edit.
    ///
    /// When `mergeable` is true and the previous edit was mergeable too, the
    /// inverse is folded into the newest entry instead of starting a new one.
    pub fn record(&mut self, inverse: Transaction, target_state: StateId, mergeable: bool) {
        self.redo_stack.clear();

        match self.undo_stack.back_mut() {
            Some(entry) if mergeable && self.can_merge => {
                // Newest change must be reverted first.
                entry.txs.insert(0, inverse);
            }
            _ => {
                self.undo_stack.push_back(HistoryEntry {
                    txs: vec![inverse],
                    target_state,
                });
                if self.undo_stack.len() > MAX_HISTORY_SIZE {
                    self.undo_stack.pop_front();
                }
            }
        }

        self.can_merge = mergeable;
    }

    /// End the current merge group; the next edit starts a new undo step.
    pub fn seal(&mut self) {
        self.can_merge = false;
    }

    /// Pop the newest undo entry
    pub fn pop_undo(&mut self) -> Option<HistoryEntry> {
        self.can_merge = false;
        self.undo_stack.pop_back()
    }

    /// Pop the newest redo entry
    pub fn pop_redo(&mut self) -> Option<HistoryEntry> {
        self.can_merge = false;
        self.redo_stack.pop()
    }

    /// Push an entry that redoes a step that was just undone
    pub fn push_redo(&mut self, entry: HistoryEntry) {
        self.redo_stack.push(entry);
    }

    /// Push an entry that undoes a step that was just redone
    pub fn push_undo(&mut self, entry: HistoryEntry) {
        self.undo_stack.push_back(entry);
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.can_merge = false;
    }

    /// Get the number of undo states
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo states
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}
