use crate::history::{History, HistoryEntry, StateId};
use crate::syntax::{byte_to_point, highlighter, HighlightSpan};
use lite_core::{ChangeSet, Operation, Range, Rope, Selection, Transaction};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use tree_sitter::{InputEdit, Tree};

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
    /// Syntax tree, kept in sync with `rope` through incremental edits
    syntax_tree: Option<Tree>,
    /// Content state currently shown
    state: StateId,
    /// Content state that matches the file on disk
    saved_state: StateId,
    /// Next unused state id
    next_state: StateId,
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
        Self::from_text("")
    }

    /// Create a document from text
    pub fn from_text(text: impl AsRef<str>) -> Self {
        let text = text.as_ref();
        Self {
            id: DocumentId::next(),
            rope: Rope::from(text),
            path: None,
            selections: HashMap::new(),
            history: History::new(),
            line_ending: LineEnding::detect(text),
            encoding: "utf-8",
            language: None,
            syntax_tree: None,
            state: 0,
            saved_state: 0,
            next_state: 1,
        }
    }

    /// Open a document from file
    pub fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let bytes = fs::read(&path)?;
        let text = String::from_utf8(bytes).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "file is not valid UTF-8 (binary files are not supported)",
            )
        })?;

        let mut doc = Self::from_text(&text);
        doc.set_path(path);
        Ok(doc)
    }

    /// Create an empty document for a file that does not exist yet
    pub fn new_file(path: impl Into<PathBuf>) -> Self {
        let mut doc = Self::new();
        doc.set_path(path.into());
        doc
    }

    /// Set the file path and refresh language detection
    fn set_path(&mut self, path: PathBuf) {
        self.language = detect_language(&path);
        self.path = Some(path);
        self.syntax_tree = None;
        self.reparse();
    }

    /// Save the document to its path
    pub fn save(&mut self) -> io::Result<()> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No file name"))?;

        write_atomically(&path, &self.rope)?;

        self.saved_state = self.state;
        self.history.seal();
        Ok(())
    }

    /// Save the document to a new path
    pub fn save_as(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        write_atomically(&path, &self.rope)?;
        self.set_path(path);
        self.saved_state = self.state;
        self.history.seal();
        Ok(())
    }

    /// Whether the content differs from the last save
    pub fn is_modified(&self) -> bool {
        self.state != self.saved_state
    }

    /// An untitled, unmodified, empty buffer that can be replaced freely
    pub fn is_scratch(&self) -> bool {
        self.path.is_none() && !self.is_modified() && self.is_empty()
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
        if self.is_modified() {
            format!("{} *", name)
        } else {
            name.to_string()
        }
    }

    /// Get the selection for a view, clamped to the document length
    pub fn selection(&self, view_id: crate::ViewId) -> Selection {
        let len = self.rope.len_chars();
        match self.selections.get(&view_id) {
            Some(sel) if sel.ranges().iter().all(|r| r.end() <= len) => sel.clone(),
            Some(sel) => sel.transform(|r| Range::new(r.anchor.min(len), r.head.min(len))),
            None => Selection::point(0),
        }
    }

    /// Set selection for a view. Moving the cursor ends the current undo group.
    pub fn set_selection(&mut self, view_id: crate::ViewId, selection: Selection) {
        self.history.seal();
        self.selections.insert(view_id, selection);
    }

    /// Apply a transaction as its own undo step
    pub fn apply(&mut self, tx: &Transaction, view_id: crate::ViewId) -> bool {
        self.apply_tx(tx, view_id, false)
    }

    /// Apply a transaction that may join the previous undo step (typing)
    pub fn apply_mergeable(&mut self, tx: &Transaction, view_id: crate::ViewId) -> bool {
        self.apply_tx(tx, view_id, true)
    }

    fn apply_tx(&mut self, tx: &Transaction, view_id: crate::ViewId, mergeable: bool) -> bool {
        if tx.is_empty() {
            return false;
        }

        let inverse = tx.invert(&self.rope, &self.selection(view_id));
        self.apply_changes(&tx.changes, tx.selection.as_ref().map(|sel| (view_id, sel)));

        let previous_state = self.state;
        self.state = self.next_state;
        self.next_state += 1;
        self.history.record(inverse, previous_state, mergeable);

        true
    }

    /// Undo the last change
    pub fn undo(&mut self, view_id: crate::ViewId) -> bool {
        match self.history.pop_undo() {
            Some(entry) => {
                let redo = self.replay(entry, view_id);
                self.history.push_redo(redo);
                true
            }
            None => false,
        }
    }

    /// Redo the last undone change
    pub fn redo(&mut self, view_id: crate::ViewId) -> bool {
        match self.history.pop_redo() {
            Some(entry) => {
                let undo = self.replay(entry, view_id);
                self.history.push_undo(undo);
                true
            }
            None => false,
        }
    }

    /// Apply a history entry and return the entry that reverses it
    fn replay(&mut self, entry: HistoryEntry, view_id: crate::ViewId) -> HistoryEntry {
        let mut reverse_txs = Vec::with_capacity(entry.txs.len());

        for tx in &entry.txs {
            reverse_txs.push(tx.invert(&self.rope, &self.selection(view_id)));
            self.apply_changes(&tx.changes, tx.selection.as_ref().map(|sel| (view_id, sel)));
        }
        reverse_txs.reverse();

        let reverse = HistoryEntry {
            txs: reverse_txs,
            target_state: self.state,
        };
        self.state = entry.target_state;
        reverse
    }

    /// Mutate the rope, keep the syntax tree in sync, and move every view's
    /// selection along with the text. `explicit` overrides one view's selection.
    fn apply_changes(
        &mut self,
        changes: &ChangeSet,
        explicit: Option<(crate::ViewId, &Selection)>,
    ) {
        let mut pos = 0;

        for op in &changes.ops {
            match op {
                Operation::Retain(n) => pos += n,
                Operation::Delete(n) => {
                    let start_byte = self.rope.char_to_byte(pos);
                    let old_end_byte = self.rope.char_to_byte(pos + n);
                    let start_position = byte_to_point(&self.rope, start_byte);
                    let old_end_position = byte_to_point(&self.rope, old_end_byte);

                    self.rope.remove(pos..pos + n);

                    if let Some(tree) = &mut self.syntax_tree {
                        tree.edit(&InputEdit {
                            start_byte,
                            old_end_byte,
                            new_end_byte: start_byte,
                            start_position,
                            old_end_position,
                            new_end_position: start_position,
                        });
                    }
                }
                Operation::Insert(text) => {
                    let start_byte = self.rope.char_to_byte(pos);
                    let start_position = byte_to_point(&self.rope, start_byte);

                    self.rope.insert(pos, text);
                    pos += text.chars().count();

                    if let Some(tree) = &mut self.syntax_tree {
                        let new_end_byte = start_byte + text.len();
                        tree.edit(&InputEdit {
                            start_byte,
                            old_end_byte: start_byte,
                            new_end_byte,
                            start_position,
                            old_end_position: start_position,
                            new_end_position: byte_to_point(&self.rope, new_end_byte),
                        });
                    }
                }
            }
        }

        for (id, sel) in self.selections.iter_mut() {
            *sel =
                sel.transform(|r| Range::new(changes.map_pos(r.anchor), changes.map_pos(r.head)));
            if let Some((view_id, explicit_sel)) = explicit {
                if *id == view_id {
                    *sel = explicit_sel.clone();
                }
            }
        }
        if let Some((view_id, explicit_sel)) = explicit {
            self.selections
                .entry(view_id)
                .or_insert_with(|| explicit_sel.clone());
        }

        self.reparse();
    }

    /// Re-run the parser, reusing the edited tree when there is one
    fn reparse(&mut self) {
        self.syntax_tree = match &self.language {
            Some(lang) => highlighter().parse(lang, &self.rope, self.syntax_tree.as_ref()),
            None => None,
        };
    }

    /// Highlight spans for the given line range
    pub fn highlights(&self, lines: std::ops::Range<usize>) -> Vec<HighlightSpan> {
        let (Some(lang), Some(tree)) = (&self.language, &self.syntax_tree) else {
            return Vec::new();
        };
        let start_line = lines.start.min(self.rope.len_lines());
        let end_line = lines.end.min(self.rope.len_lines());
        let start_byte = self.rope.line_to_byte(start_line);
        let end_byte = self.rope.line_to_byte(end_line).max(start_byte);
        highlighter().highlight_range(
            lang,
            tree,
            &self.rope,
            start_byte..end_byte.max(start_byte + 1),
        )
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

/// Write `rope` to `path` without ever leaving a half-written file behind:
/// the content goes to a temp file in the same directory, which is then
/// renamed over the target. Symlinks are followed so the link itself survives.
fn write_atomically(path: &Path, rope: &Rope) -> io::Result<()> {
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let dir = match target.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid file name"))?;
    let tmp_path = dir.join(format!(".{}.lite-{}.tmp", file_name, std::process::id()));

    let result = (|| {
        let mut file = fs::File::create(&tmp_path)?;
        for chunk in rope.chunks() {
            file.write_all(chunk.as_bytes())?;
        }
        file.sync_all()?;
        if let Ok(meta) = fs::metadata(&target) {
            fs::set_permissions(&tmp_path, meta.permissions())?;
        }
        fs::rename(&tmp_path, &target)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

/// Detect language from the file name or extension
fn detect_language(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    match file_name {
        "Dockerfile" => return Some("dockerfile".to_string()),
        "Makefile" | "makefile" | "GNUmakefile" => return Some("makefile".to_string()),
        _ => {}
    }

    let ext = path.extension()?.to_str()?;
    let lang = match ext {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
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
        _ => return None,
    };
    Some(lang.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ViewId;

    fn insert(doc: &mut Document, view: ViewId, pos: usize, text: &str) {
        let tx = Transaction::insert(doc.len_chars(), pos, text.to_string())
            .with_selection(Selection::point(pos + text.chars().count()));
        doc.apply_mergeable(&tx, view);
    }

    #[test]
    fn test_typing_is_one_undo_step() {
        let mut doc = Document::new();
        let view = ViewId::next();
        insert(&mut doc, view, 0, "a");
        insert(&mut doc, view, 1, "b");
        insert(&mut doc, view, 2, "c");
        assert_eq!(doc.text(), "abc");

        assert!(doc.undo(view));
        assert_eq!(doc.text(), "");
        assert!(doc.redo(view));
        assert_eq!(doc.text(), "abc");
    }

    #[test]
    fn test_cursor_move_splits_undo_group() {
        let mut doc = Document::new();
        let view = ViewId::next();
        insert(&mut doc, view, 0, "a");
        doc.set_selection(view, Selection::point(1));
        insert(&mut doc, view, 1, "b");

        doc.undo(view);
        assert_eq!(doc.text(), "a");
    }

    #[test]
    fn test_undo_back_to_saved_state_is_not_modified() {
        let dir = std::env::temp_dir().join(format!("lite-doc-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("saved.txt");

        let mut doc = Document::new_file(&path);
        let view = ViewId::next();
        insert(&mut doc, view, 0, "x");
        doc.save().unwrap();
        assert!(!doc.is_modified());

        doc.set_selection(view, Selection::point(1));
        insert(&mut doc, view, 1, "y");
        assert!(doc.is_modified());

        doc.undo(view);
        assert!(!doc.is_modified());
        doc.redo(view);
        assert!(doc.is_modified());

        assert_eq!(fs::read_to_string(&path).unwrap(), "x");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_other_views_follow_edits() {
        let mut doc = Document::from_text("hello world");
        let editing = ViewId::next();
        let other = ViewId::next();
        doc.set_selection(other, Selection::point(11));

        let tx = Transaction::delete(doc.len_chars(), 0, 6).with_selection(Selection::point(0));
        doc.apply(&tx, editing);

        assert_eq!(doc.selection(other).cursor(), 5);
    }

    #[test]
    fn test_open_rejects_non_utf8() {
        let path = std::env::temp_dir().join(format!("lite-binary-{}", std::process::id()));
        fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        let err = Document::open(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_syntax_tree_stays_in_sync() {
        let mut doc = Document::from_text("fn main() {}\n");
        doc.language = Some("rust".to_string());
        doc.reparse();
        let view = ViewId::next();
        insert(&mut doc, view, 0, "// ");
        let fresh = highlighter().parse("rust", &doc.rope, None).unwrap();
        assert_eq!(
            doc.syntax_tree.as_ref().unwrap().root_node().to_sexp(),
            fresh.root_node().to_sexp()
        );
    }
}
