use lite_core::Transaction;

/// Maximum number of undo states to keep
const MAX_HISTORY_SIZE: usize = 1000;

/// Undo/redo history for a document
#[derive(Debug)]
pub struct History {
    /// Undo stack
    undo_stack: Vec<Transaction>,
    /// Redo stack
    redo_stack: Vec<Transaction>,
}

impl History {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Push a transaction to the undo stack
    pub fn push(&mut self, tx: Transaction) {
        // Clear redo stack on new edit
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push(tx);

        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
    }

    /// Push a transaction to the redo stack (used internally)
    pub fn push_redo(&mut self, tx: Transaction) {
        self.redo_stack.push(tx);
    }

    /// Pop from undo stack
    pub fn undo(&mut self) -> Option<Transaction> {
        self.undo_stack.pop()
    }

    /// Pop from redo stack
    pub fn redo(&mut self) -> Option<Transaction> {
        self.redo_stack.pop()
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

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
