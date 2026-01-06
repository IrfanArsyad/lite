use crate::history::History;
use lite_core::{Range, Rope, Selection, Transaction};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique identifier for documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(usize);

impl DocumentId {
    pub fn next() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::next()
    }
}

/// A document in the editor
#[derive(Debug)]
pub struct Document {
    /// Unique identifier
    pub id: DocumentId,
    /// The text content
    pub rope: Rope,
    /// Path to the file (if saved)
    pub path: Option<PathBuf>,
    /// Whether the document has been modified
    pub modified: bool,
    /// Selection per view
    selections: HashMap<crate::ViewId, Selection>,
    /// Undo/redo history
    pub history: History,
    /// Line ending style
    pub line_ending: LineEnding,
    /// File encoding (currently only UTF-8)
    pub encoding: &'static str,
    /// Language identifier (for syntax highlighting)
    pub language: Option<String>,
    /// Last saved version (for tracking modifications)
    last_saved_version: usize,
    /// Current version counter
    version: usize,
}

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    #[default]
    LF,
    CRLF,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
        }
    }

    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            LineEnding::CRLF
        } else {
            LineEnding::LF
        }
    }
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            id: DocumentId::next(),
            rope: Rope::new(),
            path: None,
            modified: false,
            selections: HashMap::new(),
            history: History::new(),
            line_ending: LineEnding::LF,
            encoding: "utf-8",
            language: None,
            last_saved_version: 0,
            version: 0,
        }
    }

    /// Create a document from text
    pub fn from_text(text: impl AsRef<str>) -> Self {
        let text = text.as_ref();
        let line_ending = LineEnding::detect(text);
        Self {
            id: DocumentId::next(),
            rope: Rope::from(text),
            path: None,
            modified: false,
            selections: HashMap::new(),
            history: History::new(),
            line_ending,
            encoding: "utf-8",
            language: None,
            last_saved_version: 0,
            version: 0,
        }
    }

    /// Open a document from file
    pub fn open(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        let text = std::fs::read_to_string(&path)?;
        let line_ending = LineEnding::detect(&text);
        let language = detect_language(&path);

        Ok(Self {
            id: DocumentId::next(),
            rope: Rope::from(text),
            path: Some(path),
            modified: false,
            selections: HashMap::new(),
            history: History::new(),
            line_ending,
            encoding: "utf-8",
            language,
            last_saved_version: 0,
            version: 0,
        })
    }

    /// Save the document to its path
    pub fn save(&mut self) -> std::io::Result<()> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No path set"))?;

        let text = self.text();
        std::fs::write(path, text)?;

        self.modified = false;
        self.last_saved_version = self.version;
        Ok(())
    }

    /// Save the document to a new path
    pub fn save_as(&mut self, path: impl Into<PathBuf>) -> std::io::Result<()> {
        self.path = Some(path.into());
        self.language = self.path.as_ref().and_then(|p| detect_language(p));
        self.save()
    }

    /// Get the full text content
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get the file name (or "untitled")
    pub fn name(&self) -> &str {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
    }

    /// Get the display title
    pub fn title(&self) -> String {
        let name = self.name();
        if self.modified {
            format!("{} *", name)
        } else {
            name.to_string()
        }
    }

    /// Get or create selection for a view
    pub fn selection(&self, view_id: crate::ViewId) -> Selection {
        self.selections
            .get(&view_id)
            .cloned()
            .unwrap_or_else(|| Selection::point(0))
    }

    /// Get mutable selection for a view
    pub fn selection_mut(&mut self, view_id: crate::ViewId) -> &mut Selection {
        self.selections
            .entry(view_id)
            .or_insert_with(|| Selection::point(0))
    }

    /// Set selection for a view
    pub fn set_selection(&mut self, view_id: crate::ViewId, selection: Selection) {
        self.selections.insert(view_id, selection);
    }

    /// Apply a transaction to the document
    pub fn apply(&mut self, tx: &Transaction, view_id: crate::ViewId) -> bool {
        if tx.is_empty() {
            return false;
        }

        // Store current state for undo
        let old_selection = self.selection(view_id);
        let inverse = tx.invert(&self.rope, &old_selection);

        // Apply changes
        tx.apply(&mut self.rope);

        // Update selection if provided
        if let Some(ref sel) = tx.selection {
            self.set_selection(view_id, sel.clone());
        } else {
            // Map existing selection through the changes
            let sel = &self.selection(view_id);
            let new_sel = sel.transform(|range| {
                let anchor = tx.changes.map_pos(range.anchor);
                let head = tx.changes.map_pos(range.head);
                Range::new(anchor, head)
            });
            self.set_selection(view_id, new_sel);
        }

        // Push to history
        self.history.push(inverse);

        self.version += 1;
        self.modified = self.version != self.last_saved_version;

        true
    }

    /// Undo the last change
    pub fn undo(&mut self, view_id: crate::ViewId) -> bool {
        if let Some(tx) = self.history.undo() {
            // Get inverse before applying
            let old_sel = self.selection(view_id);
            let inverse = tx.invert(&self.rope, &old_sel);

            // Apply undo
            tx.apply(&mut self.rope);

            // Restore selection
            if let Some(ref sel) = tx.selection {
                self.set_selection(view_id, sel.clone());
            }

            // Push to redo
            self.history.push_redo(inverse);

            self.version += 1;
            self.modified = self.version != self.last_saved_version;
            true
        } else {
            false
        }
    }

    /// Redo the last undone change
    pub fn redo(&mut self, view_id: crate::ViewId) -> bool {
        if let Some(tx) = self.history.redo() {
            // Get inverse before applying
            let old_sel = self.selection(view_id);
            let inverse = tx.invert(&self.rope, &old_sel);

            // Apply redo
            tx.apply(&mut self.rope);

            // Restore selection
            if let Some(ref sel) = tx.selection {
                self.set_selection(view_id, sel.clone());
            }

            // Push back to undo
            self.history.push(inverse);

            self.version += 1;
            self.modified = self.version != self.last_saved_version;
            true
        } else {
            false
        }
    }

    /// Get line count
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get char count
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Check if document is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// Remove view's selection when view is closed
    pub fn remove_view(&mut self, view_id: crate::ViewId) {
        self.selections.remove(&view_id);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect language from file extension
fn detect_language(path: &PathBuf) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    let lang = match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "jsx" => "javascript",
        "tsx" => "typescript",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "clj" | "cljs" => "clojure",
        "lua" => "lua",
        "sh" | "bash" | "zsh" => "bash",
        "fish" => "fish",
        "ps1" => "powershell",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "less" => "less",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "md" | "markdown" => "markdown",
        "tex" => "latex",
        "vim" => "vim",
        "dockerfile" | "Dockerfile" => "dockerfile",
        "makefile" | "Makefile" => "makefile",
        _ => return None,
    };
    Some(lang.to_string())
}
