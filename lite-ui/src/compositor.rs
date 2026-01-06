use lite_config::{Action, KeyEvent};
use lite_view::Editor;
use ratatui::prelude::*;

/// Result of handling an event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    /// Event was consumed
    Consumed,
    /// Event was ignored, try next component
    Ignored,
    /// Event should trigger an action
    Action(Action),
}

/// Context passed to components during render and event handling
pub struct Context<'a> {
    pub editor: &'a mut Editor,
}

impl<'a> Context<'a> {
    pub fn new(editor: &'a mut Editor) -> Self {
        Self { editor }
    }
}

/// Trait for UI components
pub trait Component {
    /// Render the component
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context);

    /// Handle a key event
    fn handle_key(&mut self, _event: &KeyEvent, _ctx: &mut Context) -> EventResult {
        EventResult::Ignored
    }

    /// Get cursor position if this component should show cursor
    fn cursor(&self, _area: Rect, _ctx: &Context) -> Option<(u16, u16)> {
        None
    }

    /// Check if component requires exclusive focus
    fn is_popup(&self) -> bool {
        false
    }
}

/// Manages layered UI components
pub struct Compositor {
    /// Stack of components (bottom to top)
    layers: Vec<Box<dyn Component>>,
}

impl Compositor {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Push a component to the top
    pub fn push(&mut self, component: Box<dyn Component>) {
        self.layers.push(component);
    }

    /// Pop the top component
    pub fn pop(&mut self) -> Option<Box<dyn Component>> {
        self.layers.pop()
    }

    /// Clear all layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Get the number of layers
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Render all components
    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        for component in &self.layers {
            component.render(frame, area, ctx);
        }
    }

    /// Handle key event - goes to top component first
    pub fn handle_key(&mut self, event: &KeyEvent, ctx: &mut Context) -> EventResult {
        // If top component is a popup, only it handles events
        if let Some(top) = self.layers.last_mut() {
            if top.is_popup() {
                return top.handle_key(event, ctx);
            }
        }

        // Otherwise, go from top to bottom until consumed
        for component in self.layers.iter_mut().rev() {
            match component.handle_key(event, ctx) {
                EventResult::Ignored => continue,
                result => return result,
            }
        }

        EventResult::Ignored
    }

    /// Get cursor position from top component that provides one
    pub fn cursor(&self, area: Rect, ctx: &Context) -> Option<(u16, u16)> {
        for component in self.layers.iter().rev() {
            if let Some(pos) = component.cursor(area, ctx) {
                return Some(pos);
            }
        }
        None
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}
