use crate::{Document, DocumentId, Layout, Tree, View, ViewId};
use lite_config::{Config, Keymap, Theme};
use std::collections::HashMap;
use std::path::PathBuf;

/// Message severity for status messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Global editor state
pub struct Editor {
    /// All open documents
    pub documents: HashMap<DocumentId, Document>,
    /// All views
    pub views: HashMap<ViewId, View>,
    /// Layout tree
    pub tree: Tree,
    /// Editor configuration
    pub config: Config,
    /// Current theme
    pub theme: Theme,
    /// Keymap
    pub keymap: Keymap,
    /// Status message
    pub status_msg: Option<(String, Severity)>,
    /// Whether the editor should quit
    pub should_quit: bool,
    /// Command line mode (for :commands)
    pub command_mode: bool,
    /// Command line input
    pub command_input: String,
    /// Search mode
    pub search_mode: bool,
    /// Search query
    pub search_query: String,
    /// Clipboard content
    pub clipboard: String,
}

impl Editor {
    /// Create a new editor instance
    pub fn new() -> Self {
        // Create initial document and view
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
            views,
            tree: Tree::new(view_id),
            config: Config::default(),
            theme: Theme::default(),
            keymap: Keymap::default(),
            status_msg: None,
            should_quit: false,
            command_mode: false,
            command_input: String::new(),
            search_mode: false,
            search_query: String::new(),
            clipboard: String::new(),
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
        self.documents.get_mut(&doc_id).expect("Document must exist")
    }

    /// Open a file
    pub fn open(&mut self, path: impl Into<PathBuf>) -> Result<DocumentId, std::io::Error> {
        let path = path.into();

        // Check if already open
        for (id, doc) in &self.documents {
            if doc.path.as_ref() == Some(&path) {
                // Switch to existing document
                self.switch_to_document(*id);
                return Ok(*id);
            }
        }

        // Open new document
        let doc = Document::open(&path)?;
        let doc_id = doc.id;
        self.documents.insert(doc_id, doc);

        // Create a view for it
        let view = View::new(doc_id);
        let view_id = view.id;
        self.views.insert(view_id, view);

        // Replace current view's document or add to tree
        let current_view = self.current_view_mut();
        let old_doc_id = current_view.doc_id;
        current_view.doc_id = doc_id;

        // Clean up old document if not used elsewhere
        self.cleanup_document(old_doc_id);

        self.set_status(format!("Opened: {}", path.display()), Severity::Info);
        Ok(doc_id)
    }

    /// Create a new empty document
    pub fn new_document(&mut self) -> DocumentId {
        let doc = Document::new();
        let doc_id = doc.id;
        self.documents.insert(doc_id, doc);

        // Update current view to show new document
        let view_id = self.tree.focus();
        if let Some(view) = self.views.get_mut(&view_id) {
            let old_doc_id = view.doc_id;
            view.doc_id = doc_id;
            self.cleanup_document(old_doc_id);
        }

        doc_id
    }

    /// Save the current document
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        let doc = self.current_doc_mut();
        if doc.path.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file name",
            ));
        }
        doc.save()?;
        let name = doc.name().to_string();
        self.set_status(format!("Saved: {}", name), Severity::Info);
        Ok(())
    }

    /// Save the current document with a new path
    pub fn save_as(&mut self, path: impl Into<PathBuf>) -> Result<(), std::io::Error> {
        let path = path.into();
        let doc = self.current_doc_mut();
        doc.save_as(&path)?;
        self.set_status(format!("Saved: {}", path.display()), Severity::Info);
        Ok(())
    }

    /// Switch to a document by ID
    pub fn switch_to_document(&mut self, doc_id: DocumentId) {
        if self.documents.contains_key(&doc_id) {
            let view_id = self.tree.focus();
            if let Some(view) = self.views.get_mut(&view_id) {
                view.doc_id = doc_id;
            }
        }
    }

    /// Split the current view
    pub fn split(&mut self, layout: Layout) {
        let current_doc_id = self.current_view().doc_id;
        let new_view = View::new(current_doc_id);
        let new_view_id = new_view.id;
        self.views.insert(new_view_id, new_view);
        self.tree.split(new_view_id, layout);
    }

    /// Close the current view
    pub fn close_view(&mut self) -> bool {
        let view_id = self.tree.focus();

        if let Some(new_focus) = self.tree.close(view_id) {
            let doc_id = self.views.remove(&view_id).map(|v| v.doc_id);

            // Clean up document if needed
            if let Some(doc_id) = doc_id {
                self.cleanup_document(doc_id);
            }

            self.tree.set_focus(new_focus);
            true
        } else {
            // Last view - check if we should quit
            false
        }
    }

    /// Close the current buffer
    pub fn close_buffer(&mut self) -> bool {
        let doc_id = self.current_view().doc_id;
        let doc = self.documents.get(&doc_id);

        if let Some(doc) = doc {
            if doc.modified {
                self.set_status(
                    "Buffer has unsaved changes. Use :q! to force quit.".into(),
                    Severity::Warning,
                );
                return false;
            }
        }

        // Find another document to switch to
        let other_doc_id = self
            .documents
            .keys()
            .find(|&&id| id != doc_id)
            .copied();

        if let Some(other_id) = other_doc_id {
            self.switch_to_document(other_id);
            self.documents.remove(&doc_id);
            true
        } else {
            // Last document - quit
            self.should_quit = true;
            true
        }
    }

    /// Clean up a document if no views reference it
    fn cleanup_document(&mut self, doc_id: DocumentId) {
        let is_used = self.views.values().any(|v| v.doc_id == doc_id);
        if !is_used {
            self.documents.remove(&doc_id);
        }
    }

    /// Set a status message
    pub fn set_status(&mut self, msg: impl Into<String>, severity: Severity) {
        self.status_msg = Some((msg.into(), severity));
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    /// Get list of open buffers
    pub fn buffer_list(&self) -> Vec<(DocumentId, String)> {
        self.documents
            .iter()
            .map(|(id, doc)| (*id, doc.title()))
            .collect()
    }

    /// Resize the focused view
    pub fn resize(&mut self, width: u16, height: u16) {
        let view = self.current_view_mut();
        view.set_size(width, height.saturating_sub(2)); // Reserve for status/tab lines
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
