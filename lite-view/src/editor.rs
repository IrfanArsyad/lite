use crate::{Area, Document, DocumentId, Layout, LayoutResult, Tree, View, ViewId};
use lite_config::{Action, Config, Keymap, Theme};
use lite_core::{display_col, RopeExt};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// How long a status message stays visible when no key is pressed
pub const STATUS_TIMEOUT: Duration = Duration::from_secs(5);

/// Message severity for status messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// A message shown in the status line
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub severity: Severity,
    pub shown_at: Instant,
}

/// A destructive action waiting for the user to repeat it as confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingConfirm {
    Quit,
    CloseBuffer(DocumentId),
}

/// Global editor state
pub struct Editor {
    /// All open documents
    pub documents: HashMap<DocumentId, Document>,
    /// Open buffers in tab order
    buffers: Vec<DocumentId>,
    /// All views
    pub views: HashMap<ViewId, View>,
    /// Layout tree
    pub tree: Tree,
    /// Screen area available to views
    area: Area,
    /// Editor configuration
    pub config: Config,
    /// Current theme
    pub theme: Theme,
    /// Keymap
    pub keymap: Keymap,
    /// Status message
    pub status_msg: Option<StatusMessage>,
    /// Whether the editor should quit
    pub should_quit: bool,
    /// Last search query, reused by find next/previous
    pub search_query: String,
    /// Clipboard content
    pub clipboard: String,
    /// Destructive action armed by a first key press
    pending_confirm: Option<PendingConfirm>,
}

impl Editor {
    /// Create a new editor instance with default configuration
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    /// Create a new editor instance
    pub fn with_config(config: Config) -> Self {
        let doc = Document::new();
        let doc_id = doc.id;

        let view = View::new(doc_id);
        let view_id = view.id;

        let mut documents = HashMap::new();
        documents.insert(doc_id, doc);

        let mut views = HashMap::new();
        views.insert(view_id, view);

        Self {
            documents,
            buffers: vec![doc_id],
            views,
            tree: Tree::new(view_id),
            area: Area::default(),
            config,
            theme: Theme::default(),
            keymap: Keymap::default(),
            status_msg: None,
            should_quit: false,
            search_query: String::new(),
            clipboard: String::new(),
            pending_confirm: None,
        }
    }

    /// Get the currently focused view
    pub fn current_view(&self) -> &View {
        let view_id = self.tree.focus();
        self.views.get(&view_id).expect("Focus view must exist")
    }

    /// Get the currently focused view mutably
    pub fn current_view_mut(&mut self) -> &mut View {
        let view_id = self.tree.focus();
        self.views.get_mut(&view_id).expect("Focus view must exist")
    }

    /// Get the currently focused document
    pub fn current_doc(&self) -> &Document {
        let doc_id = self.current_view().doc_id;
        self.documents.get(&doc_id).expect("Document must exist")
    }

    /// Get the currently focused document mutably
    pub fn current_doc_mut(&mut self) -> &mut Document {
        let doc_id = self.current_view().doc_id;
        self.documents
            .get_mut(&doc_id)
            .expect("Document must exist")
    }

    /// Open a file in the focused view. A path that does not exist yet opens
    /// as an empty buffer and is created on first save.
    pub fn open(&mut self, path: impl Into<PathBuf>) -> Result<DocumentId, io::Error> {
        let path = path.into();

        let existing_id = self
            .documents
            .iter()
            .find(|(_, doc)| doc.path.as_ref() == Some(&path))
            .map(|(id, _)| *id);

        if let Some(id) = existing_id {
            self.switch_to_document(id);
            return Ok(id);
        }

        let (doc, message) = match Document::open(&path) {
            Ok(doc) => (doc, format!("Opened: {}", path.display())),
            Err(err) if err.kind() == io::ErrorKind::NotFound => (
                Document::new_file(&path),
                format!("New file: {}", path.display()),
            ),
            Err(err) => return Err(err),
        };

        let doc_id = doc.id;
        self.documents.insert(doc_id, doc);

        let current = self.current_view().doc_id;
        let insert_at = self
            .buffers
            .iter()
            .position(|id| *id == current)
            .map_or(self.buffers.len(), |idx| idx + 1);
        self.buffers.insert(insert_at, doc_id);

        self.switch_to_document(doc_id);
        self.drop_unused_scratch(current);

        self.set_status(message, Severity::Info);
        Ok(doc_id)
    }

    /// Create a new empty document
    pub fn new_document(&mut self) -> DocumentId {
        let doc = Document::new();
        let doc_id = doc.id;
        self.documents.insert(doc_id, doc);
        self.buffers.push(doc_id);
        self.switch_to_document(doc_id);
        doc_id
    }

    /// Save the current document
    pub fn save(&mut self) -> Result<(), io::Error> {
        let doc = self.current_doc_mut();
        doc.save()?;
        let name = doc.name().to_string();
        self.set_status(format!("Saved: {}", name), Severity::Info);
        Ok(())
    }

    /// Save the current document with a new path
    pub fn save_as(&mut self, path: impl Into<PathBuf>) -> Result<(), io::Error> {
        let path = path.into();
        let doc = self.current_doc_mut();
        doc.save_as(&path)?;
        self.set_status(format!("Saved: {}", path.display()), Severity::Info);
        Ok(())
    }

    /// Show a document in the focused view
    pub fn switch_to_document(&mut self, doc_id: DocumentId) {
        if !self.documents.contains_key(&doc_id) {
            return;
        }
        let view_id = self.tree.focus();
        if let Some(view) = self.views.get_mut(&view_id) {
            if view.doc_id != doc_id {
                view.doc_id = doc_id;
                view.scroll_x = 0;
                view.scroll_y = 0;
            }
        }
        self.ensure_cursor_visible();
    }

    /// Split the current view
    pub fn split(&mut self, layout: Layout) {
        let current_doc_id = self.current_view().doc_id;
        let current_view_id = self.tree.focus();
        let new_view = View::new(current_doc_id);
        let new_view_id = new_view.id;
        self.views.insert(new_view_id, new_view);

        // The new view starts where the old one was.
        let doc = self
            .documents
            .get_mut(&current_doc_id)
            .expect("Document must exist");
        let selection = doc.selection(current_view_id);
        doc.set_selection(new_view_id, selection);

        self.tree.split(new_view_id, layout);
        self.relayout();
    }

    /// Close the current view. Returns false when it is the last one.
    pub fn close_view(&mut self) -> bool {
        let view_id = self.tree.focus();

        let Some(new_focus) = self.tree.close(view_id) else {
            self.set_status("Cannot close the last window", Severity::Info);
            return false;
        };

        if let Some(view) = self.views.remove(&view_id) {
            if let Some(doc) = self.documents.get_mut(&view.doc_id) {
                doc.remove_view(view_id);
            }
        }

        self.tree.set_focus(new_focus);
        self.relayout();
        true
    }

    /// Close the current buffer. A buffer with unsaved changes is only closed
    /// when the request is repeated (see [`Editor::confirm`]).
    pub fn close_buffer(&mut self, confirmed: bool) {
        let doc_id = self.current_view().doc_id;

        if !confirmed && self.documents.get(&doc_id).is_some_and(|d| d.is_modified()) {
            let hint = self.repeat_hint(&Action::CloseBuffer);
            self.pending_confirm = Some(PendingConfirm::CloseBuffer(doc_id));
            self.set_status(
                format!("Unsaved changes. {hint} to close without saving."),
                Severity::Warning,
            );
            return;
        }

        let Some(idx) = self.buffers.iter().position(|id| *id == doc_id) else {
            return;
        };
        self.buffers.remove(idx);

        if self.buffers.is_empty() {
            self.should_quit = true;
            return;
        }

        let fallback = self.buffers[idx.min(self.buffers.len() - 1)];
        for view in self.views.values_mut() {
            if view.doc_id == doc_id {
                view.doc_id = fallback;
                view.scroll_x = 0;
                view.scroll_y = 0;
            }
        }
        self.documents.remove(&doc_id);
    }

    /// Quit, asking for confirmation first when buffers have unsaved changes
    pub fn quit(&mut self, confirmed: bool) {
        let unsaved = self.documents.values().filter(|d| d.is_modified()).count();
        if unsaved == 0 || confirmed {
            self.should_quit = true;
            return;
        }

        let hint = self.repeat_hint(&Action::Quit);
        let noun = if unsaved == 1 {
            "buffer has"
        } else {
            "buffers have"
        };
        self.pending_confirm = Some(PendingConfirm::Quit);
        self.set_status(
            format!("{unsaved} {noun} unsaved changes. {hint} to quit without saving."),
            Severity::Warning,
        );
    }

    /// Take the armed confirmation, if any. Any other action disarms it.
    pub fn take_pending_confirm(&mut self) -> Option<PendingConfirm> {
        self.pending_confirm.take()
    }

    /// Whether `action` repeats the armed confirmation
    pub fn confirms(
        pending: Option<PendingConfirm>,
        action: &Action,
        current_doc: DocumentId,
    ) -> bool {
        matches!(
            (pending, action),
            (Some(PendingConfirm::Quit), Action::Quit)
        ) || matches!(
            (pending, action),
            (Some(PendingConfirm::CloseBuffer(id)), Action::CloseBuffer) if id == current_doc
        )
    }

    fn repeat_hint(&self, action: &Action) -> String {
        match self.keymap.key_for(action) {
            Some(key) => format!("Press {key} again"),
            None => "Repeat the command".to_string(),
        }
    }

    /// Remove a scratch buffer that the user has replaced and no view shows
    fn drop_unused_scratch(&mut self, doc_id: DocumentId) {
        let is_scratch = self
            .documents
            .get(&doc_id)
            .is_some_and(Document::is_scratch);
        let is_shown = self.views.values().any(|v| v.doc_id == doc_id);
        if is_scratch && !is_shown {
            self.documents.remove(&doc_id);
            self.buffers.retain(|id| *id != doc_id);
        }
    }

    /// Set a status message
    pub fn set_status(&mut self, msg: impl Into<String>, severity: Severity) {
        self.status_msg = Some(StatusMessage {
            text: msg.into(),
            severity,
            shown_at: Instant::now(),
        });
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    /// Clear the status message once it has been shown long enough.
    /// Returns true when something was cleared.
    pub fn expire_status(&mut self, now: Instant) -> bool {
        let expired = self
            .status_msg
            .as_ref()
            .is_some_and(|msg| now.duration_since(msg.shown_at) >= STATUS_TIMEOUT);
        // A pending confirmation keeps its warning on screen.
        if expired && self.pending_confirm.is_none() {
            self.status_msg = None;
            return true;
        }
        false
    }

    /// Get list of open buffers in tab order
    pub fn buffer_list(&self) -> Vec<(DocumentId, String)> {
        self.buffers
            .iter()
            .filter_map(|id| self.documents.get(id).map(|doc| (*id, doc.title())))
            .collect()
    }

    /// Set the screen area that views are laid out in
    pub fn set_area(&mut self, area: Area) {
        self.area = area;
        self.relayout();
    }

    /// Screen placement of all views
    pub fn layout(&self) -> LayoutResult {
        self.tree.layout(self.area)
    }

    /// Resize every view to its place in the split layout
    fn relayout(&mut self) {
        for (view_id, area) in self.tree.layout(self.area).views {
            if let Some(view) = self.views.get_mut(&view_id) {
                view.set_size(area.width, area.height);
            }
        }
        self.ensure_cursor_visible();
    }

    /// Scroll the focused view so its primary cursor is on screen
    pub fn ensure_cursor_visible(&mut self) {
        let view_id = self.tree.focus();
        let scrolloff = self.config.editor.scrolloff;
        let tab_width = self.config.editor.tab_width;
        let Some(view) = self.views.get(&view_id) else {
            return;
        };
        let Some(doc) = self.documents.get(&view.doc_id) else {
            return;
        };

        let pos = doc.rope.char_to_position(doc.selection(view_id).cursor());
        let col = display_col(doc.rope.line(pos.line), pos.col, tab_width);
        let line_count = doc.len_lines();

        let view = self.views.get_mut(&view_id).expect("view exists");
        view.update_gutter_width(line_count);
        view.ensure_cursor_visible(pos.line, col, scrolloff);
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lite_core::{Selection, Transaction};

    fn temp_file(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lite-editor-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    fn type_text(editor: &mut Editor, text: &str) {
        let view = editor.tree.focus();
        let doc = editor.current_doc_mut();
        let tx = Transaction::insert(doc.len_chars(), 0, text.to_string())
            .with_selection(Selection::point(text.chars().count()));
        doc.apply(&tx, view);
    }

    #[test]
    fn test_open_keeps_modified_scratch_buffer() {
        let mut editor = Editor::new();
        type_text(&mut editor, "draft");
        editor.open(temp_file("a.txt", "a")).unwrap();

        assert_eq!(editor.buffer_list().len(), 2);
        assert_eq!(editor.buffer_list()[0].1, "untitled *");
    }

    #[test]
    fn test_open_replaces_empty_scratch_buffer() {
        let mut editor = Editor::new();
        editor.open(temp_file("b.txt", "b")).unwrap();
        assert_eq!(editor.buffer_list().len(), 1);
    }

    #[test]
    fn test_tabs_keep_open_order() {
        let mut editor = Editor::new();
        let names = ["t1.txt", "t2.txt", "t3.txt", "t4.txt"];
        for name in names {
            editor.open(temp_file(name, name)).unwrap();
        }
        let titles: Vec<_> = editor.buffer_list().into_iter().map(|(_, t)| t).collect();
        assert_eq!(titles, names);
    }

    #[test]
    fn test_open_missing_file_creates_buffer() {
        let mut editor = Editor::new();
        let path = std::env::temp_dir().join("lite-does-not-exist-yet.txt");
        editor.open(&path).unwrap();
        assert_eq!(editor.current_doc().path.as_ref(), Some(&path));
    }

    #[test]
    fn test_close_buffer_shown_in_split_does_not_panic() {
        let mut editor = Editor::new();
        editor.open(temp_file("c.txt", "c")).unwrap();
        editor.open(temp_file("d.txt", "d")).unwrap();
        editor.split(Layout::Vertical);
        editor.close_buffer(false);

        for view in editor.views.values() {
            assert!(editor.documents.contains_key(&view.doc_id));
        }
    }

    #[test]
    fn test_quit_with_unsaved_changes_needs_confirmation() {
        let mut editor = Editor::new();
        type_text(&mut editor, "unsaved");

        editor.quit(false);
        assert!(!editor.should_quit);
        let pending = editor.take_pending_confirm();
        assert_eq!(pending, Some(PendingConfirm::Quit));

        let doc = editor.current_view().doc_id;
        assert!(Editor::confirms(pending, &Action::Quit, doc));
        editor.quit(true);
        assert!(editor.should_quit);
    }
}
