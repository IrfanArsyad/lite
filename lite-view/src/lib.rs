//! Editor state and view management for lite editor

mod document;
mod editor;
mod history;
pub mod syntax;
mod tree;
mod view;

pub use document::{Document, DocumentId, LineEnding};
pub use editor::{Editor, PendingConfirm, Severity, StatusMessage, STATUS_TIMEOUT};
pub use history::{History, HistoryEntry, StateId};
pub use syntax::{highlighter, Highlight, HighlightSpan, Highlighter};
pub use tree::{Area, Layout, LayoutResult, Tree};
pub use view::{View, ViewId};
