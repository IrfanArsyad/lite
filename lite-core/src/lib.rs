//! Core text primitives for lite editor
//!
//! This crate provides the fundamental building blocks for text editing:
//! - `Rope`: Efficient text storage using rope data structure
//! - `Selection`: Multi-cursor selections
//! - `Transaction`: Atomic text operations with undo support
//! - `Position`: Line/column position utilities

mod display;
mod grapheme;
mod position;
mod rope_ext;
mod selection;
mod transaction;

pub use display::{char_col_at_display, char_display_width, display_col, CONTROL_REPLACEMENT};
pub use grapheme::{grapheme_width, nth_next_grapheme, nth_prev_grapheme, RopeGraphemes};
pub use position::Position;
pub use ropey::{Rope, RopeSlice};
pub use rope_ext::RopeExt;
pub use selection::{Range, Selection};
pub use transaction::{Change, ChangeSet, Operation, Transaction};
