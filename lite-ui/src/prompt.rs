use crate::{Component, Context, EventResult};
use lite_config::{Action, Key, KeyEvent, Modifier};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};
use unicode_width::UnicodeWidthStr;

/// Type of prompt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptType {
    Command,
    Search,
    SaveAs,
    Open,
    GotoLine,
    /// First step of replace: ask what to look for
    Replace,
    /// Second step of replace: ask for the replacement of the given text
    ReplaceWith(String),
}

/// Input prompt for commands, search, etc.
pub struct Prompt {
    prompt_type: PromptType,
    input: String,
    /// Cursor position as a char index into `input`
    cursor: usize,
}

impl Prompt {
    pub fn new(prompt_type: PromptType) -> Self {
        Self {
            prompt_type,
            input: String::new(),
            cursor: 0,
        }
    }

    pub fn with_initial(mut self, initial: impl Into<String>) -> Self {
        self.input = initial.into();
        self.cursor = self.input.chars().count();
        self
    }

    pub fn prompt_type(&self) -> &PromptType {
        &self.prompt_type
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    fn prefix(&self) -> String {
        match &self.prompt_type {
            PromptType::Command => ":".to_string(),
            PromptType::Search => "Find: ".to_string(),
            PromptType::SaveAs => "Save as: ".to_string(),
            PromptType::Open => "Open: ".to_string(),
            PromptType::GotoLine => "Go to line: ".to_string(),
            PromptType::Replace => "Replace: ".to_string(),
            PromptType::ReplaceWith(find) => format!("Replace \"{find}\" with: "),
        }
    }

    /// Byte offset of the char at `char_idx`
    fn byte_idx(&self, char_idx: usize) -> usize {
        self.input
            .char_indices()
            .nth(char_idx)
            .map_or(self.input.len(), |(idx, _)| idx)
    }

    fn len_chars(&self) -> usize {
        self.input.chars().count()
    }

    fn insert_char(&mut self, c: char) {
        let idx = self.byte_idx(self.cursor);
        self.input.insert(idx, c);
        self.cursor += 1;
    }

    fn insert_str(&mut self, text: &str) {
        // A prompt is single-line, so pasted newlines are dropped.
        for c in text.chars().filter(|c| *c != '\n' && *c != '\r') {
            self.insert_char(c);
        }
    }

    fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let idx = self.byte_idx(self.cursor);
            self.input.remove(idx);
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.len_chars() {
            let idx = self.byte_idx(self.cursor);
            self.input.remove(idx);
        }
    }

    fn submit_action(&self) -> Action {
        let input = self.input.clone();
        match &self.prompt_type {
            PromptType::GotoLine => Action::ExecuteGotoLine(input),
            PromptType::Search => Action::ExecuteSearch(input),
            PromptType::Open => Action::ExecuteOpen(input),
            PromptType::SaveAs => Action::ExecuteSaveAs(input),
            PromptType::Replace => Action::ExecuteReplaceFind(input),
            PromptType::ReplaceWith(find) => Action::ExecuteReplace(find.clone(), input),
            PromptType::Command => Action::Noop,
        }
    }
}

impl Component for Prompt {
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        let theme = &ctx.editor.theme;
        let line = Line::from(vec![
            Span::styled(
                format!(" {}", self.prefix()),
                theme.popup.to_ratatui().patch(theme.info.to_ratatui()),
            ),
            Span::styled(self.input.clone(), theme.popup.to_ratatui()),
        ]);
        // The prompt sits on top of the status line; wipe it first.
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(line).style(theme.popup.to_ratatui()), area);
    }

    fn handle_key(&mut self, event: &KeyEvent, _ctx: &mut Context) -> EventResult {
        match (&event.key, event.modifiers) {
            (Key::Escape, _) => return EventResult::Action(Action::Noop),
            (Key::Enter, Modifier::NONE) => return EventResult::Action(self.submit_action()),
            (Key::Char(c), Modifier::NONE) | (Key::Char(c), Modifier::SHIFT) => {
                self.insert_char(*c)
            }
            (Key::Backspace, Modifier::NONE) => self.delete_char(),
            (Key::Delete, Modifier::NONE) => self.delete_forward(),
            (Key::Left, Modifier::NONE) => self.cursor = self.cursor.saturating_sub(1),
            (Key::Right, Modifier::NONE) => self.cursor = (self.cursor + 1).min(self.len_chars()),
            (Key::Home, Modifier::NONE) => self.cursor = 0,
            (Key::End, Modifier::NONE) => self.cursor = self.len_chars(),
            (Key::Char('u'), Modifier::CTRL) => {
                self.input.clear();
                self.cursor = 0;
            }
            // Swallow everything else so editor shortcuts do not fire mid-prompt.
            _ => {}
        }

        EventResult::Consumed
    }

    fn cursor(&self, area: Rect, _ctx: &Context) -> Option<(u16, u16)> {
        let before_cursor: String = self.input.chars().take(self.cursor).collect();
        let x = 1 + self.prefix().width() + before_cursor.width();
        let x = (x as u16).min(area.width.saturating_sub(1));
        Some((area.x + x, area.y))
    }

    fn is_popup(&self) -> bool {
        true
    }

    fn paste(&mut self, text: &str) -> bool {
        self.insert_str(text);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_ascii_input_does_not_panic() {
        let mut prompt = Prompt::new(PromptType::Search);
        prompt.insert_char('é');
        prompt.insert_char('日');
        prompt.cursor = 1;
        prompt.insert_char('x');
        assert_eq!(prompt.input(), "éx日");
        prompt.delete_char();
        prompt.delete_forward();
        assert_eq!(prompt.input(), "é");
    }

    #[test]
    fn test_replace_flow_actions() {
        let prompt = Prompt::new(PromptType::Replace).with_initial("foo");
        assert_eq!(
            prompt.submit_action(),
            Action::ExecuteReplaceFind("foo".into())
        );

        let prompt = Prompt::new(PromptType::ReplaceWith("foo".into())).with_initial("bar");
        assert_eq!(
            prompt.submit_action(),
            Action::ExecuteReplace("foo".into(), "bar".into())
        );
    }
}
