//! Terminal application for lite editor

mod application;
mod commands;
mod event;

pub use application::Application;
pub use commands::execute_action;
pub use event::{Event, EventHandler};
