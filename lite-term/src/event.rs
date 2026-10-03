use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers};
use lite_config::{Key, KeyEvent, Modifier};
use std::time::Duration;
use tokio::sync::mpsc;

/// How often a `Tick` is sent while the user is idle (drives status timeouts)
const TICK_INTERVAL: Duration = Duration::from_millis(250);

/// Editor events
#[derive(Debug)]
pub enum Event {
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(crossterm::event::MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Text pasted into the terminal (bracketed paste)
    Paste(String),
    /// A message from a background task, shown in the status line
    Notice(String),
    /// Periodic timer while idle
    Tick,
}

/// Event handler that reads terminal events
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    /// Sender for background tasks that want to post events
    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.sender.clone()
    }

    /// Start reading terminal events. Reading blocks, so it runs on its own
    /// OS thread instead of an async task.
    pub fn start(&self) {
        let sender = self.sender.clone();

        std::thread::spawn(move || loop {
            let event = match event::poll(TICK_INTERVAL) {
                Ok(true) => match event::read() {
                    Ok(evt) => convert_event(evt),
                    Err(_) => None,
                },
                Ok(false) => Some(Event::Tick),
                Err(_) => None,
            };

            if let Some(event) = event {
                if sender.send(event).is_err() {
                    break;
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

fn convert_event(evt: CrosstermEvent) -> Option<Event> {
    match evt {
        // Terminals with the kitty protocol also report releases; acting on
        // them would apply every key twice.
        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Release => None,
        CrosstermEvent::Key(key) => convert_key_event(key).map(Event::Key),
        CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
        CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
        CrosstermEvent::Paste(text) => Some(Event::Paste(text)),
        _ => None,
    }
}

/// Convert crossterm key event to our KeyEvent. Keys lite has no use for
/// (media keys, caps lock, ...) yield `None`.
fn convert_key_event(key: crossterm::event::KeyEvent) -> Option<KeyEvent> {
    let mut modifiers = Modifier {
        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
        alt: key.modifiers.contains(KeyModifiers::ALT),
        shift: key.modifiers.contains(KeyModifiers::SHIFT),
    };

    let key = match key.code {
        // Terminals report Ctrl+Shift+S as Ctrl+'S'; the keymap uses
        // lowercase letters plus an explicit Shift modifier.
        KeyCode::Char(c) if (modifiers.ctrl || modifiers.alt) && c.is_ascii_uppercase() => {
            modifiers.shift = true;
            Key::Char(c.to_ascii_lowercase())
        }
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::F(n) => Key::F(n),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => {
            modifiers.shift = true;
            Key::Tab
        }
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
        _ => return None,
    };

    Some(KeyEvent::new(key, modifiers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent as CtKeyEvent, KeyEventState};

    fn press(code: KeyCode, modifiers: KeyModifiers) -> CtKeyEvent {
        CtKeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_unknown_keys_are_dropped() {
        assert_eq!(
            convert_key_event(press(KeyCode::CapsLock, KeyModifiers::NONE)),
            None
        );
    }

    #[test]
    fn test_ctrl_shift_letter_is_normalized() {
        let event = convert_key_event(press(
            KeyCode::Char('S'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));
        assert_eq!(event, Some(KeyEvent::ctrl_shift('s')));
    }

    #[test]
    fn test_key_release_is_ignored() {
        let mut key = press(KeyCode::Char('a'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert!(convert_event(CrosstermEvent::Key(key)).is_none());
    }

    #[test]
    fn test_back_tab_is_shift_tab() {
        let event = convert_key_event(press(KeyCode::BackTab, KeyModifiers::SHIFT));
        assert_eq!(event, Some(KeyEvent::new(Key::Tab, Modifier::SHIFT)));
    }
}
