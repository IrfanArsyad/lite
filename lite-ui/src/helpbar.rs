use crate::{Component, Context};
use lite_config::{Action, KeyEvent, Keymap};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

/// A shortcut shown in the help bar
struct Shortcut {
    action: Action,
    label: &'static str,
    /// Lower numbers stay visible longest when the terminal is narrow
    priority: u8,
}

/// Shortcuts in display order
fn shortcuts() -> [Shortcut; 9] {
    [
        Shortcut {
            action: Action::Save,
            label: "Save",
            priority: 0,
        },
        Shortcut {
            action: Action::Open,
            label: "Open",
            priority: 1,
        },
        Shortcut {
            action: Action::CloseBuffer,
            label: "Close",
            priority: 3,
        },
        Shortcut {
            action: Action::Find,
            label: "Find",
            priority: 1,
        },
        Shortcut {
            action: Action::Replace,
            label: "Replace",
            priority: 4,
        },
        Shortcut {
            action: Action::GotoLine,
            label: "Go to line",
            priority: 3,
        },
        Shortcut {
            action: Action::Undo,
            label: "Undo",
            priority: 2,
        },
        Shortcut {
            action: Action::Redo,
            label: "Redo",
            priority: 5,
        },
        Shortcut {
            action: Action::Quit,
            label: "Quit",
            priority: 0,
        },
    ]
}

/// Space between one shortcut's label and the next key chip
const ITEM_GAP: usize = 3;
/// Blank column before the first chip
const LEFT_MARGIN: usize = 1;

/// Help bar showing keyboard shortcuts at the bottom (nano-style)
pub struct HelpBar;

impl HelpBar {
    pub fn new() -> Self {
        Self
    }

    /// Compact nano-style key name: `^S`, `M-x`, `^S-z`
    fn key_label(key: &KeyEvent) -> String {
        let full = key.to_string();
        full.replace("Ctrl+", "^")
            .replace("Alt+", "M-")
            .replace("Shift+", "S-")
    }

    /// Shortcuts that are bound in `keymap` and fit in `width`, in display order
    fn visible_items(keymap: &Keymap, width: usize) -> Vec<(String, &'static str)> {
        let mut items: Vec<(usize, u8, String, &'static str)> = shortcuts()
            .into_iter()
            .enumerate()
            .filter_map(|(order, s)| {
                keymap
                    .key_for(&s.action)
                    .map(|key| (order, s.priority, Self::key_label(key), s.label))
            })
            .collect();

        // Drop the least important shortcut until the rest fit.
        while Self::total_width(&items) > width && !items.is_empty() {
            let drop_idx = items
                .iter()
                .enumerate()
                .max_by_key(|(idx, item)| (item.1, *idx))
                .map(|(idx, _)| idx)
                .expect("items is not empty");
            items.remove(drop_idx);
        }

        items.sort_by_key(|item| item.0);
        items
            .into_iter()
            .map(|(_, _, key, label)| (key, label))
            .collect()
    }

    fn item_width(key: &str, label: &str) -> usize {
        // " key " chip, a space, then the label
        key.width() + 2 + 1 + label.width()
    }

    fn total_width(items: &[(usize, u8, String, &'static str)]) -> usize {
        let content: usize = items
            .iter()
            .map(|(_, _, key, label)| Self::item_width(key, label))
            .sum();
        LEFT_MARGIN + content + ITEM_GAP * items.len().saturating_sub(1)
    }
}

impl Default for HelpBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HelpBar {
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &Context) {
        let theme = &ctx.editor.theme;
        let bar_style = theme.helpbar.to_ratatui();
        let key_style = theme.helpbar_key.to_ratatui();

        let mut spans = vec![Span::styled(" ".repeat(LEFT_MARGIN), bar_style)];
        let items = Self::visible_items(&ctx.editor.keymap, area.width as usize);
        for (i, (key, label)) in items.into_iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ".repeat(ITEM_GAP), bar_style));
            }
            spans.push(Span::styled(format!(" {key} "), key_style));
            spans.push(Span::styled(format!(" {label}"), bar_style));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)).style(bar_style), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labels_come_from_keymap() {
        let items = HelpBar::visible_items(&Keymap::default(), 200);
        assert_eq!(items.first(), Some(&("^S".to_string(), "Save")));
        assert_eq!(items.len(), shortcuts().len());
    }

    #[test]
    fn test_narrow_terminal_keeps_most_important() {
        let items = HelpBar::visible_items(&Keymap::default(), 30);
        let labels: Vec<_> = items.iter().map(|(_, label)| *label).collect();
        assert_eq!(labels, ["Save", "Quit"]);
    }

    #[test]
    fn test_unbound_action_is_hidden() {
        let mut keymap = Keymap::default();
        keymap.insert(KeyEvent::ctrl('h'), Action::Noop);
        let items = HelpBar::visible_items(&keymap, 200);
        assert!(items.iter().all(|(_, label)| *label != "Replace"));
    }
}
