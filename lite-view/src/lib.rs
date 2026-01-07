//! Editor state and view management for lite editor

mod document;
mod editor;
mod history;
pub mod syntax;
mod tree;
mod view;

pub use document::{Document, DocumentId, LineEnding};
pub use editor::{Editor, Severity};
pub use history::History;
pub use syntax::{highlighter, Highlight, HighlightSpan, Highlighter};
pub use tree::{Layout, Tree};
pub use view::{View, ViewId};
