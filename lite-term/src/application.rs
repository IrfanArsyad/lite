use crate::commands::{self, Outcome};
use crate::{execute_action, Event, EventHandler};
use anyhow::Result;
use crossterm::{
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        KeyboardEnhancementFlags, MouseButton, MouseEventKind, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use lite_config::{Action, Config, Key, KeyEvent, Modifier};
use lite_ui::{
    Component, Compositor, Context, EditorView, EventResult, HelpBar, Prompt, StatusLine, TabLine,
};
use lite_view::{Area, Editor};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

/// Rows used by the tab line above the editor
const TAB_LINE_HEIGHT: u16 = 1;
/// Rows used by the status line and help bar below the editor
const BOTTOM_BARS_HEIGHT: u16 = 2;

/// Screen regions of the main window
struct ScreenLayout {
    tabs: Rect,
    editor: Rect,
    status: Rect,
    help: Rect,
}

impl ScreenLayout {
    fn new(area: Rect) -> Self {
        let editor_height = area
            .height
            .saturating_sub(TAB_LINE_HEIGHT + BOTTOM_BARS_HEIGHT);
        let row = |offset: u16, height: u16| Rect {
            x: area.x,
            y: (area.y + offset).min(area.bottom().saturating_sub(1)),
            width: area.width,
            height: height.min(area.height),
        };
        Self {
            tabs: row(0, TAB_LINE_HEIGHT),
            editor: row(TAB_LINE_HEIGHT, editor_height),
            status: row(TAB_LINE_HEIGHT + editor_height, 1),
            help: row(TAB_LINE_HEIGHT + editor_height + 1, 1),
        }
    }
}

/// Whether the kitty keyboard protocol was switched on and must be switched off
static KEYBOARD_ENHANCED: AtomicBool = AtomicBool::new(false);

/// Put the terminal back the way the shell expects it. Safe to call more
/// than once; also used by the panic hook.
pub fn restore_terminal() {
    if KEYBOARD_ENHANCED.swap(false, Ordering::SeqCst) {
        let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
    }
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen,
        crossterm::cursor::Show
    );
}

/// Main application struct
pub struct Application {
    /// The editor state
    editor: Editor,
    /// UI compositor (prompts)
    compositor: Compositor,
    /// Terminal
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Event handler
    events: EventHandler,
    /// Whether the screen must be redrawn
    needs_redraw: bool,
}

impl Application {
    /// Create a new application
    pub fn new(config: Config) -> Result<Self> {
        let mouse = config.editor.mouse;

        enable_raw_mode()?;
        // From here on, any failure must hand the terminal back in a usable state.
        let setup = || -> Result<Terminal<CrosstermBackend<Stdout>>> {
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
            // Terminals that speak the kitty keyboard protocol can then tell
            // Ctrl+Shift+S from Ctrl+S, Ctrl+Tab from Tab, and so on.
            if matches!(supports_keyboard_enhancement(), Ok(true)) {
                execute!(
                    stdout,
                    PushKeyboardEnhancementFlags(
                        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    )
                )?;
                KEYBOARD_ENHANCED.store(true, Ordering::SeqCst);
            }
            if mouse {
                execute!(stdout, EnableMouseCapture)?;
            }
            Ok(Terminal::new(CrosstermBackend::new(stdout))?)
        };
        let terminal = setup().inspect_err(|_| restore_terminal())?;

        let mut editor = Editor::with_config(config);
        let size = terminal.size()?;
        editor.set_area(to_area(
            ScreenLayout::new(Rect::new(0, 0, size.width, size.height)).editor,
        ));

        Ok(Self {
            editor,
            compositor: Compositor::new(),
            terminal,
            events: EventHandler::new(),
            needs_redraw: true,
        })
    }

    /// Open a file
    pub fn open(&mut self, path: &str) -> Result<()> {
        self.editor.open(path)?;
        Ok(())
    }

    /// Show a message in the status bar
    pub fn set_notice(&mut self, msg: String) {
        self.editor.set_status(msg, lite_view::Severity::Info);
    }

    /// Sender that background tasks can use to post events
    pub fn event_sender(&self) -> UnboundedSender<Event> {
        self.events.sender()
    }

    /// Run the application main loop
    pub async fn run(&mut self) -> Result<()> {
        self.events.start();

        while !self.editor.should_quit {
            if self.needs_redraw {
                self.render()?;
                self.needs_redraw = false;
            }

            match self.events.next().await {
                Some(event) => self.handle_event(event),
                None => break,
            }
        }

        Ok(())
    }

    /// Render the UI
    fn render(&mut self) -> Result<()> {
        let ctx = Context::new(&mut self.editor);
        let compositor = &self.compositor;

        self.terminal.draw(|frame| {
            let layout = ScreenLayout::new(frame.area());

            TabLine::new().render(frame, layout.tabs, &ctx);
            EditorView::new().render(frame, layout.editor, &ctx);
            StatusLine::new().render(frame, layout.status, &ctx);
            HelpBar::new().render(frame, layout.help, &ctx);

            // Prompts take over the status line.
            compositor.render(frame, layout.status, &ctx);

            let cursor = if compositor.is_empty() {
                EditorView::new().cursor(layout.editor, &ctx)
            } else {
                compositor.cursor(layout.status, &ctx)
            };
            if let Some(position) = cursor {
                frame.set_cursor_position(position);
            }
        })?;

        Ok(())
    }

    /// Handle an event
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Resize(width, height) => {
                let layout = ScreenLayout::new(Rect::new(0, 0, width, height));
                self.editor.set_area(to_area(layout.editor));
            }
            Event::Mouse(mouse) => {
                if !self.compositor.is_empty() {
                    return;
                }
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        commands::click(&mut self.editor, mouse.column, mouse.row);
                    }
                    MouseEventKind::ScrollDown => {
                        commands::scroll(&mut self.editor, mouse.column, mouse.row, true)
                    }
                    MouseEventKind::ScrollUp => {
                        commands::scroll(&mut self.editor, mouse.column, mouse.row, false)
                    }
                    _ => return,
                }
            }
            Event::Paste(text) => {
                if !self.compositor.paste(&text) {
                    commands::paste_text(&mut self.editor, &text);
                    self.editor.ensure_cursor_visible();
                }
            }
            Event::Notice(msg) => self.set_notice(msg),
            Event::Tick => {
                if !self.editor.expire_status(Instant::now()) {
                    return;
                }
            }
        }
        self.needs_redraw = true;
    }

    /// Handle a key event
    fn handle_key(&mut self, key_event: KeyEvent) {
        self.editor.clear_status();

        if !self.compositor.is_empty() {
            let mut ctx = Context::new(&mut self.editor);
            match self.compositor.handle_key(&key_event, &mut ctx) {
                EventResult::Action(action) => {
                    self.compositor.pop();
                    if action != Action::Noop {
                        self.run_action(&action);
                    }
                }
                EventResult::Consumed | EventResult::Ignored => {}
            }
            return;
        }

        let action = match key_event.key {
            Key::Char(c)
                if key_event.modifiers == Modifier::NONE
                    || key_event.modifiers == Modifier::SHIFT =>
            {
                Some(Action::InsertChar(c))
            }
            _ => self.editor.keymap.get(&key_event).cloned(),
        };
        if let Some(action) = action {
            self.run_action(&action);
        }
    }

    /// Execute an action and open the prompt it asks for, if any
    fn run_action(&mut self, action: &Action) {
        if let Outcome::OpenPrompt(prompt_type, initial) = execute_action(&mut self.editor, action)
        {
            self.compositor
                .push(Box::new(Prompt::new(prompt_type).with_initial(initial)));
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn to_area(rect: Rect) -> Area {
    Area::new(rect.x, rect.y, rect.width, rect.height)
}
