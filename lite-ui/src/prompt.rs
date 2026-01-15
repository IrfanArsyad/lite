use crate::{Component, Context, EventResult};
use lite_config::{Action, Key, KeyEvent, Modifier};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Type of prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptType {
    Command,
    Search,
    SaveAs,
    Open,
    GotoLine,
}

/// Input prompt for commands, search, etc.
pub struct Prompt {
    prompt_type: PromptType,
    input: String,
    cursor: usize,
    submitted: bool,
}

impl Prompt {
    pub fn new(prompt_type: PromptType) -> Self {
        Self {
            prompt_type,
            input: String::new(),
            cursor: 0,
            submitted: false,
        }
    }

    pub fn with_initial(mut self, initial: impl Into<String>) -> Self {
        self.input = initial.into();
        self.cursor = self.input.len();
        self
    }

    pub fn prompt_type(&self) -> PromptType {
        self.prompt_type
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    fn prefix(&self) -> &str {
        match self.prompt_type {
            PromptType::Command => ":",
            PromptType::Search => "/",
            PromptType::SaveAs => "Save as: ",
            PromptType::Open => "Open: ",
            PromptType::GotoLine => "Goto line: ",
        }
    }

    fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    fn move_start(&mut self) {
        self.cursor = 0;
    }

    fn move_end(&mut self) {
        self.cursor = self.input.len();
    }

    fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }
}

impl Component for Prompt {
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        let style = ctx.editor.theme.popup.to_ratatui();
        let prefix = self.prefix();
        let text = format!("{}{}", prefix, self.input);
        let prompt = Paragraph::new(text).style(style);
        frame.render_widget(prompt, area);
    }

    fn handle_key(&mut self, event: &KeyEvent, _ctx: &mut Context) -> EventResult {
        match (&event.key, event.modifiers) {
            // Cancel
            (Key::Escape, _) => {
                return EventResult::Action(Action::Noop);
            }

            // Submit
            (Key::Enter, Modifier::NONE) => {
                self.submitted = true;
                let action = match self.prompt_type {
                    PromptType::GotoLine => Action::ExecuteGotoLine(self.input.clone()),
                    PromptType::Search => Action::ExecuteSearch(self.input.clone()),
                    PromptType::Open => Action::ExecuteOpen(self.input.clone()),
                    PromptType::SaveAs => Action::ExecuteSaveAs(self.input.clone()),
                    _ => Action::Noop,
                };
                return EventResult::Action(action);
            }

            // Character input
            (Key::Char(c), Modifier::NONE) | (Key::Char(c), Modifier::SHIFT) => {
                self.insert_char(*c);
            }

            // Backspace
            (Key::Backspace, Modifier::NONE) => {
                self.delete_char();
            }

            // Delete
            (Key::Delete, Modifier::NONE) => {
                self.delete_forward();
            }

            // Navigation
            (Key::Left, Modifier::NONE) => {
                self.move_left();
            }
            (Key::Right, Modifier::NONE) => {
                self.move_right();
            }
            (Key::Home, Modifier::NONE) => {
                self.move_start();
            }
            (Key::End, Modifier::NONE) => {
                self.move_end();
            }

            // Clear input
            (Key::Char('u'), Modifier::CTRL) => {
                self.clear();
            }

            _ => return EventResult::Ignored,
        }

        EventResult::Consumed
    }

    fn cursor(&self, area: Rect, _ctx: &Context) -> Option<(u16, u16)> {
        let prefix_len = self.prefix().len();
        let cursor_x = area.x + prefix_len as u16 + self.cursor as u16;
        Some((cursor_x, area.y))
    }

    fn is_popup(&self) -> bool {
        true
    }
}
