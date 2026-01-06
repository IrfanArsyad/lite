use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use lite_config::{Key, KeyEvent, Modifier};
use std::time::Duration;
use tokio::sync::mpsc;

/// Editor events
#[derive(Debug)]
pub enum Event {
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(crossterm::event::MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Tick for animations/timeouts
    Tick,
}

/// Event handler that reads terminal events
pub struct EventHandler {
    /// Event sender
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    /// Start the event loop in a background task
    pub fn start(&self) {
        let sender = self.sender.clone();

        tokio::spawn(async move {
            loop {
                // Poll for events with timeout for tick
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        let event = match evt {
                            CrosstermEvent::Key(key) => {
                                Some(Event::Key(convert_key_event(key)))
                            }
                            CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                            CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
                            _ => None,
                        };

                        if let Some(event) = event {
                            if sender.send(event).is_err() {
                                break;
                            }
                        }
                    }
                } else {
                    // Send tick event
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                }
            }
        });
    }

    /// Receive the next event
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert crossterm key event to our KeyEvent
fn convert_key_event(key: crossterm::event::KeyEvent) -> KeyEvent {
    let modifiers = Modifier {
        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
        alt: key.modifiers.contains(KeyModifiers::ALT),
        shift: key.modifiers.contains(KeyModifiers::SHIFT),
    };

    let key = match key.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::F(n) => Key::F(n),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Esc => Key::Escape,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        _ => Key::Char('\0'),
    };

    KeyEvent::new(key, modifiers)
}
