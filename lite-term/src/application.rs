use crate::{execute_action, Event, EventHandler};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lite_config::{Action, Key, KeyEvent, Modifier};
use lite_core::RopeExt;
use lite_ui::{Compositor, Component, Context, EditorView, EventResult, HelpBar, StatusLine, TabLine};
use lite_view::Editor;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::io::{self, Stdout};

use lite_ui::{Prompt, PromptType};

/// Main application struct
pub struct Application {
    /// The editor state
    editor: Editor,
    /// UI compositor
    compositor: Compositor,
    /// Terminal
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Event handler
    events: EventHandler,
}

impl Application {
    /// Create a new application
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create editor
        let mut editor = Editor::new();

        // Get terminal size
        let size = terminal.size()?;
        editor.resize(size.width, size.height);

        // Create compositor with layers
        let compositor = Compositor::new();

        // Create event handler
        let events = EventHandler::new();

        Ok(Self {
            editor,
            compositor,
            terminal,
            events,
        })
    }

    /// Open a file
    pub fn open(&mut self, path: &str) -> Result<()> {
        self.editor.open(path)?;
        Ok(())
    }

    /// Set update notice to show in status bar
    pub fn set_update_notice(&mut self, msg: String) {
        self.editor.set_status(msg, lite_view::Severity::Info);
    }

    /// Run the application main loop
    pub async fn run(&mut self) -> Result<()> {
        // Start event handler
        self.events.start();

        // Main loop
        while !self.editor.should_quit {
            // Render
            self.render()?;

            // Handle events
            if let Some(event) = self.events.next().await {
                self.handle_event(event)?;
            }
        }

        Ok(())
    }

    /// Render the UI
    fn render(&mut self) -> Result<()> {
        let ctx = Context::new(&mut self.editor);

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Layout: tab line (1), editor (remaining - 4), status line (1), help bar (2)
            let tab_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            };
            let editor_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: area.height.saturating_sub(4),
            };
            let status_area = Rect {
                x: area.x,
                y: area.height.saturating_sub(3),
                width: area.width,
                height: 1,
            };
            let help_area = Rect {
                x: area.x,
                y: area.height.saturating_sub(2),
                width: area.width,
                height: 2,
            };

            // Render base layers
            TabLine::new().render(frame, tab_area, &ctx);
            EditorView::new().render(frame, editor_area, &ctx);
            StatusLine::new().render(frame, status_area, &ctx);
            HelpBar::new().render(frame, help_area, &ctx);

            // Render compositor layers (popups, etc.)
            self.compositor.render(frame, area, &ctx);

            // Set cursor position
            if let Some((x, y)) = EditorView::new().cursor(editor_area, &ctx) {
                frame.set_cursor_position((x, y));
            }
        })?;

        Ok(())
    }

    /// Handle an event
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                self.handle_key(key_event)?;
            }
            Event::Resize(width, height) => {
                self.editor.resize(width, height);
            }
            Event::Mouse(_mouse) => {
                // TODO: Mouse handling
            }
            Event::Tick => {
                // Clear old status messages
                // TODO: Add timeout for status messages
            }
        }

        Ok(())
    }

    /// Handle a key event
    fn handle_key(&mut self, key_event: KeyEvent) -> Result<()> {
        // Clear status message on any key
        self.editor.clear_status();

        // First, let compositor handle it (for prompts, etc.)
        {
            let mut ctx = Context::new(&mut self.editor);
            let result = self.compositor.handle_key(&key_event, &mut ctx);
            match result {
                EventResult::Consumed => return Ok(()),
                EventResult::Action(action) => {
                    // Handle prompt submission actions
                    match &action {
                        Action::ExecuteGotoLine(line_str) => {
                            self.compositor.pop(); // Remove the prompt
                            self.handle_goto_line(line_str)?;
                            return Ok(());
                        }
                        Action::ExecuteSearch(search_text) => {
                            self.compositor.pop(); // Remove the prompt
                            self.handle_search(search_text)?;
                            return Ok(());
                        }
                        Action::ExecuteOpen(path) => {
                            self.compositor.pop(); // Remove the prompt
                            self.handle_open_file(path)?;
                            return Ok(());
                        }
                        Action::ExecuteSaveAs(path) => {
                            self.compositor.pop(); // Remove the prompt
                            self.handle_save_as_file(path)?;
                            return Ok(());
                        }
                        Action::Noop => {
                            // Escape was pressed
                            self.compositor.pop();
                            return Ok(());
                        }
                        _ => {}
                    }
                    execute_action(&mut self.editor, &action);
                    return Ok(());
                }
                EventResult::Ignored => {}
            }
        }

        // Handle character input
        if let Key::Char(c) = key_event.key {
            if key_event.modifiers == Modifier::NONE || key_event.modifiers == Modifier::SHIFT {
                execute_action(&mut self.editor, &Action::InsertChar(c));
                return Ok(());
            }
        }

        // Check keymap
        if let Some(action) = self.editor.keymap.get(&key_event).cloned() {
            // Handle actions that require prompts
            match &action {
                Action::GotoLine => {
                    self.compositor.push(Box::new(Prompt::new(PromptType::GotoLine)));
                }
                Action::Find => {
                    self.compositor.push(Box::new(Prompt::new(PromptType::Search)));
                }
                Action::Replace => {
                    // TODO: Implement proper replace with two prompts
                    self.compositor.push(Box::new(Prompt::new(PromptType::Search)));
                }
                Action::Open => {
                    self.compositor.push(Box::new(Prompt::new(PromptType::Open)));
                }
                Action::SaveAs => {
                    self.compositor.push(Box::new(Prompt::new(PromptType::SaveAs)));
                }
                _ => {
                    execute_action(&mut self.editor, &action);
                }
            }
        }

        Ok(())
    }

    /// Handle goto line command
    fn handle_goto_line(&mut self, line_str: &str) -> Result<()> {
        if let Ok(line_num) = line_str.parse::<usize>() {
            if line_num > 0 {
                let view_id = self.editor.tree.focus();
                let doc = self.editor.current_doc_mut();
                let target_line = (line_num - 1).min(doc.len_lines().saturating_sub(1));
                let char_pos = doc.rope.line_to_char(target_line);
                doc.set_selection(view_id, lite_core::Selection::point(char_pos));

                // Ensure cursor is visible
                let pos = doc.rope.char_to_position(char_pos);
                let scrolloff = self.editor.config.editor.scrolloff;
                self.editor
                    .current_view_mut()
                    .ensure_cursor_visible(pos.line, pos.col, scrolloff);
            }
        }
        Ok(())
    }

    /// Handle search command
    fn handle_search(&mut self, search_text: &str) -> Result<()> {
        if !search_text.is_empty() {
            let view_id = self.editor.tree.focus();
            let doc = self.editor.current_doc_mut();
            let text: String = doc.rope.chars().collect();

            if let Some(pos) = text.find(search_text) {
                let end = pos + search_text.len();
                let range = lite_core::Range::new(pos, end);
                doc.set_selection(view_id, lite_core::Selection::single(range));

                // Ensure selection is visible
                let pos = doc.rope.char_to_position(pos);
                let scrolloff = self.editor.config.editor.scrolloff;
                self.editor
                    .current_view_mut()
                    .ensure_cursor_visible(pos.line, pos.col, scrolloff);

                self.editor.set_status("Found", lite_view::Severity::Info);
            } else {
                self.editor.set_status("Not found", lite_view::Severity::Error);
            }
        }
        Ok(())
    }

    /// Handle open file command
    fn handle_open_file(&mut self, path: &str) -> Result<()> {
        if !path.is_empty() {
            if let Err(e) = self.editor.open(path) {
                self.editor.set_status(format!("Error: {}", e), lite_view::Severity::Error);
            }
        }
        Ok(())
    }

    /// Handle save as file command
    fn handle_save_as_file(&mut self, path: &str) -> Result<()> {
        if !path.is_empty() {
            if let Err(e) = self.editor.save_as(path) {
                self.editor.set_status(format!("Error saving: {}", e), lite_view::Severity::Error);
            }
        }
        Ok(())
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
