//! UI widgets for lite editor

mod compositor;
mod editor_view;
mod prompt;
mod statusline;
mod tabline;

pub use compositor::{Component, Compositor, Context, EventResult};
pub use editor_view::EditorView;
pub use prompt::Prompt;
pub use statusline::StatusLine;
pub use tabline::TabLine;
