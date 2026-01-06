use crate::{execute_action, Event, EventHandler};
use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lite_config::{Action, Key, KeyEvent, Modifier};
use lite_ui::{Compositor, Component, Context, EditorView, EventResult, StatusLine, TabLine};
use lite_view::Editor;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::io::{self, Stdout};

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

            // Layout: tab line (1), editor (remaining - 1), status line (1)
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
                height: area.height.saturating_sub(2),
            };
            let status_area = Rect {
                x: area.x,
                y: area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            };

            // Render base layers
            TabLine::new().render(frame, tab_area, &ctx);
            EditorView::new().render(frame, editor_area, &ctx);
            StatusLine::new().render(frame, status_area, &ctx);

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

        // First, let compositor handle it
        {
            let mut ctx = Context::new(&mut self.editor);
            let result = self.compositor.handle_key(&key_event, &mut ctx);
            match result {
                EventResult::Consumed => return Ok(()),
                EventResult::Action(action) => {
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
            execute_action(&mut self.editor, &action);
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
