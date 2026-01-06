use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key modifier flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Modifier {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifier {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const SHIFT: Self = Self {
        ctrl: false,
        alt: false,
        shift: true,
    };
    pub const CTRL_SHIFT: Self = Self {
        ctrl: true,
        alt: false,
        shift: true,
    };
    pub const CTRL_ALT: Self = Self {
        ctrl: true,
        alt: true,
        shift: false,
    };
    pub const ALT_SHIFT: Self = Self {
        ctrl: false,
        alt: true,
        shift: true,
    };
}

/// Keyboard event representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifier,
}

impl KeyEvent {
    pub fn new(key: Key, modifiers: Modifier) -> Self {
        Self { key, modifiers }
    }

    pub fn char(c: char) -> Self {
        Self::new(Key::Char(c), Modifier::NONE)
    }

    pub fn ctrl(c: char) -> Self {
        Self::new(Key::Char(c), Modifier::CTRL)
    }

    pub fn ctrl_shift(c: char) -> Self {
        Self::new(Key::Char(c), Modifier::CTRL_SHIFT)
    }

    pub fn alt(c: char) -> Self {
        Self::new(Key::Char(c), Modifier::ALT)
    }
}

/// Key codes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
}

/// Editor actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    // File operations
    Save,
    SaveAs,
    Open,
    QuickOpen,
    CloseBuffer,
    CloseWindow,
    Quit,

    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveWordLeft,
    MoveWordRight,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,
    PageUp,
    PageDown,
    GotoLine,
    GotoSymbol,
    JumpBack,
    JumpForward,

    // Editing
    InsertChar(char),
    InsertNewline,
    InsertNewlineBelow,
    InsertNewlineAbove,
    Backspace,
    Delete,
    DeleteLine,
    DuplicateLine,
    MoveLineUp,
    MoveLineDown,
    Indent,
    Unindent,
    ToggleComment,

    // Selection
    SelectAll,
    SelectLine,
    SelectWord,
    SelectNextOccurrence,
    SelectAllOccurrences,
    SplitSelectionLines,
    AddCursorAbove,
    AddCursorBelow,
    ClearSelection,

    // Clipboard
    Copy,
    Cut,
    Paste,

    // Undo/Redo
    Undo,
    Redo,

    // Search
    Find,
    FindNext,
    FindPrevious,
    Replace,
    FindInFiles,
    UseSelectionForFind,

    // Buffer/Tab management
    NextBuffer,
    PreviousBuffer,
    SwitchToBuffer(usize),

    // Splits
    SplitVertical,
    SplitHorizontal,
    FocusNextSplit,
    FocusPreviousSplit,

    // LSP
    Autocomplete,
    GotoDefinition,
    FindReferences,
    RenameSymbol,
    QuickFix,
    SignatureHelp,
    Hover,

    // Code folding
    Fold,
    Unfold,

    // UI
    CommandPalette,
    ToggleFileTree,

    // Misc
    Noop,
}

/// Keymap configuration
#[derive(Debug, Clone)]
pub struct Keymap {
    bindings: HashMap<KeyEvent, Action>,
}

impl Default for Keymap {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // File operations
        bindings.insert(KeyEvent::ctrl('s'), Action::Save);
        bindings.insert(KeyEvent::ctrl_shift('s'), Action::SaveAs);
        bindings.insert(KeyEvent::ctrl('o'), Action::Open);
        bindings.insert(KeyEvent::ctrl('p'), Action::QuickOpen);
        bindings.insert(KeyEvent::ctrl('w'), Action::CloseBuffer);
        bindings.insert(KeyEvent::ctrl_shift('w'), Action::CloseWindow);
        bindings.insert(KeyEvent::ctrl('q'), Action::Quit);

        // Navigation
        bindings.insert(
            KeyEvent::new(Key::Up, Modifier::NONE),
            Action::MoveUp,
        );
        bindings.insert(
            KeyEvent::new(Key::Down, Modifier::NONE),
            Action::MoveDown,
        );
        bindings.insert(
            KeyEvent::new(Key::Left, Modifier::NONE),
            Action::MoveLeft,
        );
        bindings.insert(
            KeyEvent::new(Key::Right, Modifier::NONE),
            Action::MoveRight,
        );
        bindings.insert(
            KeyEvent::new(Key::Left, Modifier::CTRL),
            Action::MoveWordLeft,
        );
        bindings.insert(
            KeyEvent::new(Key::Right, Modifier::CTRL),
            Action::MoveWordRight,
        );
        bindings.insert(
            KeyEvent::new(Key::Home, Modifier::NONE),
            Action::MoveLineStart,
        );
        bindings.insert(
            KeyEvent::new(Key::End, Modifier::NONE),
            Action::MoveLineEnd,
        );
        bindings.insert(
            KeyEvent::new(Key::Home, Modifier::CTRL),
            Action::MoveFileStart,
        );
        bindings.insert(
            KeyEvent::new(Key::End, Modifier::CTRL),
            Action::MoveFileEnd,
        );
        bindings.insert(
            KeyEvent::new(Key::PageUp, Modifier::NONE),
            Action::PageUp,
        );
        bindings.insert(
            KeyEvent::new(Key::PageDown, Modifier::NONE),
            Action::PageDown,
        );
        bindings.insert(KeyEvent::ctrl('g'), Action::GotoLine);
        bindings.insert(KeyEvent::ctrl('r'), Action::GotoSymbol);
        bindings.insert(
            KeyEvent::new(Key::Left, Modifier::ALT),
            Action::JumpBack,
        );
        bindings.insert(
            KeyEvent::new(Key::Right, Modifier::ALT),
            Action::JumpForward,
        );

        // Editing
        bindings.insert(
            KeyEvent::new(Key::Enter, Modifier::NONE),
            Action::InsertNewline,
        );
        bindings.insert(
            KeyEvent::new(Key::Enter, Modifier::CTRL),
            Action::InsertNewlineBelow,
        );
        bindings.insert(
            KeyEvent::new(Key::Enter, Modifier::CTRL_SHIFT),
            Action::InsertNewlineAbove,
        );
        bindings.insert(
            KeyEvent::new(Key::Backspace, Modifier::NONE),
            Action::Backspace,
        );
        bindings.insert(
            KeyEvent::new(Key::Delete, Modifier::NONE),
            Action::Delete,
        );
        bindings.insert(KeyEvent::ctrl_shift('k'), Action::DeleteLine);
        bindings.insert(KeyEvent::ctrl_shift('d'), Action::DuplicateLine);
        bindings.insert(
            KeyEvent::new(Key::Up, Modifier::CTRL_SHIFT),
            Action::MoveLineUp,
        );
        bindings.insert(
            KeyEvent::new(Key::Down, Modifier::CTRL_SHIFT),
            Action::MoveLineDown,
        );
        bindings.insert(
            KeyEvent::new(Key::Tab, Modifier::NONE),
            Action::Indent,
        );
        bindings.insert(
            KeyEvent::new(Key::Tab, Modifier::SHIFT),
            Action::Unindent,
        );
        bindings.insert(KeyEvent::ctrl('/'), Action::ToggleComment);

        // Selection
        bindings.insert(KeyEvent::ctrl('a'), Action::SelectAll);
        bindings.insert(KeyEvent::ctrl('l'), Action::SelectLine);
        bindings.insert(KeyEvent::ctrl('d'), Action::SelectNextOccurrence);
        bindings.insert(KeyEvent::ctrl_shift('a'), Action::SelectAllOccurrences);
        bindings.insert(KeyEvent::ctrl_shift('l'), Action::SplitSelectionLines);
        bindings.insert(
            KeyEvent::new(Key::Up, Modifier::ALT_SHIFT),
            Action::AddCursorAbove,
        );
        bindings.insert(
            KeyEvent::new(Key::Down, Modifier::ALT_SHIFT),
            Action::AddCursorBelow,
        );
        bindings.insert(
            KeyEvent::new(Key::Escape, Modifier::NONE),
            Action::ClearSelection,
        );

        // Clipboard
        bindings.insert(KeyEvent::ctrl('c'), Action::Copy);
        bindings.insert(KeyEvent::ctrl('x'), Action::Cut);
        bindings.insert(KeyEvent::ctrl('v'), Action::Paste);

        // Undo/Redo
        bindings.insert(KeyEvent::ctrl('z'), Action::Undo);
        bindings.insert(KeyEvent::ctrl_shift('z'), Action::Redo);
        bindings.insert(KeyEvent::ctrl('y'), Action::Redo);

        // Search
        bindings.insert(KeyEvent::ctrl('f'), Action::Find);
        bindings.insert(KeyEvent::new(Key::F(3), Modifier::NONE), Action::FindNext);
        bindings.insert(
            KeyEvent::new(Key::F(3), Modifier::SHIFT),
            Action::FindPrevious,
        );
        bindings.insert(KeyEvent::ctrl('h'), Action::Replace);
        bindings.insert(KeyEvent::ctrl_shift('f'), Action::FindInFiles);
        bindings.insert(KeyEvent::ctrl('e'), Action::UseSelectionForFind);

        // Buffer/Tab management
        bindings.insert(
            KeyEvent::new(Key::Tab, Modifier::CTRL),
            Action::NextBuffer,
        );
        bindings.insert(
            KeyEvent::new(Key::Tab, Modifier::CTRL_SHIFT),
            Action::PreviousBuffer,
        );
        for i in 1..=9 {
            bindings.insert(
                KeyEvent::ctrl(char::from_digit(i, 10).unwrap()),
                Action::SwitchToBuffer(i as usize),
            );
        }

        // Splits
        bindings.insert(KeyEvent::ctrl('\\'), Action::SplitVertical);
        bindings.insert(KeyEvent::ctrl_shift('\\'), Action::SplitHorizontal);

        // LSP
        bindings.insert(KeyEvent::ctrl(' '), Action::Autocomplete);
        bindings.insert(
            KeyEvent::new(Key::F(12), Modifier::NONE),
            Action::GotoDefinition,
        );
        bindings.insert(
            KeyEvent::new(Key::F(12), Modifier::SHIFT),
            Action::FindReferences,
        );
        bindings.insert(KeyEvent::new(Key::F(2), Modifier::NONE), Action::RenameSymbol);
        bindings.insert(KeyEvent::ctrl('.'), Action::QuickFix);
        bindings.insert(KeyEvent::ctrl_shift(' '), Action::SignatureHelp);

        // Code folding
        bindings.insert(KeyEvent::ctrl_shift('['), Action::Fold);
        bindings.insert(KeyEvent::ctrl_shift(']'), Action::Unfold);

        // UI
        bindings.insert(KeyEvent::ctrl_shift('p'), Action::CommandPalette);
        bindings.insert(KeyEvent::ctrl('b'), Action::ToggleFileTree);

        Self { bindings }
    }
}

impl Keymap {
    pub fn get(&self, event: &KeyEvent) -> Option<&Action> {
        self.bindings.get(event)
    }

    pub fn insert(&mut self, event: KeyEvent, action: Action) {
        self.bindings.insert(event, action);
    }
}
