//! Terminal application for lite editor

mod application;
mod commands;
mod event;

pub use application::{restore_terminal, Application};
pub use commands::{execute_action, Outcome};
pub use event::{Event, EventHandler};
