use lite_config::{Action, IndentStyle};
use lite_core::{
    char_col_at_display, display_col, Change, Range, Rope, RopeExt, Selection, Transaction,
};
use lite_ui::PromptType;
use lite_view::{Document, Editor, Layout, Severity, ViewId};

/// Lines scrolled per mouse wheel notch
const MOUSE_SCROLL_LINES: usize = 3;

/// What the application has to do after an action ran
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    /// Show a prompt, pre-filled with the given text
    OpenPrompt(PromptType, String),
}

/// Execute an action on the editor
pub fn execute_action(editor: &mut Editor, action: &Action) -> Outcome {
    let pending = editor.take_pending_confirm();
    let confirmed = Editor::confirms(pending, action, editor.current_view().doc_id);
    let outcome = run_action(editor, action, confirmed);
    editor.ensure_cursor_visible();
    outcome
}

fn run_action(editor: &mut Editor, action: &Action, confirmed: bool) -> Outcome {
    match action {
        // File operations
        Action::Save => {
            if editor.current_doc().path.is_none() {
                return Outcome::OpenPrompt(PromptType::SaveAs, String::new());
            }
            if let Err(e) = editor.save() {
                editor.set_status(format!("Error saving: {}", e), Severity::Error);
            }
        }
        Action::SaveAs => {
            let current = editor
                .current_doc()
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            return Outcome::OpenPrompt(PromptType::SaveAs, current);
        }
        Action::Open => return Outcome::OpenPrompt(PromptType::Open, String::new()),
        Action::CloseBuffer => editor.close_buffer(confirmed),
        Action::CloseWindow => {
            editor.close_view();
        }
        Action::Quit => editor.quit(confirmed),

        // Navigation
        Action::MoveUp => move_vertically(editor, VerticalDirection::Up, 1),
        Action::MoveDown => move_vertically(editor, VerticalDirection::Down, 1),
        Action::MoveLeft => move_horizontally(editor, HorizontalDirection::Left),
        Action::MoveRight => move_horizontally(editor, HorizontalDirection::Right),
        Action::MoveWordLeft => move_word(editor, HorizontalDirection::Left),
        Action::MoveWordRight => move_word(editor, HorizontalDirection::Right),
        Action::MoveLineStart => move_line_start(editor),
        Action::MoveLineEnd => move_line_end(editor),
        Action::MoveFileStart => set_cursor(editor, 0),
        Action::MoveFileEnd => {
            let end = editor.current_doc().len_chars();
            set_cursor(editor, end);
        }
        Action::PageUp => page_move(editor, VerticalDirection::Up),
        Action::PageDown => page_move(editor, VerticalDirection::Down),
        Action::GotoLine => return Outcome::OpenPrompt(PromptType::GotoLine, String::new()),

        // Editing
        Action::InsertChar(c) => insert_char(editor, *c),
        Action::InsertNewline => {
            let line_ending = editor.current_doc().line_ending.as_str();
            insert_text(editor, line_ending, false);
        }
        Action::InsertNewlineBelow => insert_newline_below(editor),
        Action::InsertNewlineAbove => insert_newline_above(editor),
        Action::Backspace => delete_backward(editor),
        Action::Delete => delete_forward(editor),
        Action::DeleteLine => delete_lines(editor),
        Action::DuplicateLine => duplicate_line(editor),
        Action::MoveLineUp => move_line(editor, VerticalDirection::Up),
        Action::MoveLineDown => move_line(editor, VerticalDirection::Down),
        Action::Indent => indent(editor),
        Action::Unindent => unindent(editor),
        Action::ToggleComment => toggle_comment(editor),

        // Selection
        Action::SelectAll => {
            let (view_id, doc) = focused(editor);
            let end = doc.len_chars();
            doc.set_selection(view_id, Selection::single(Range::new(0, end)));
        }
        Action::SelectLine => select_line(editor),
        Action::SelectWord => select_word(editor),
        Action::SelectNextOccurrence => select_next_occurrence(editor),
        Action::SelectAllOccurrences => select_all_occurrences(editor),
        Action::SplitSelectionLines => split_selection_lines(editor),
        Action::AddCursorAbove => add_cursor(editor, VerticalDirection::Up),
        Action::AddCursorBelow => add_cursor(editor, VerticalDirection::Down),
        Action::ClearSelection => {
            let (view_id, doc) = focused(editor);
            let selection = doc.selection(view_id);
            doc.set_selection(view_id, selection.into_single().collapse());
        }

        // Clipboard
        Action::Copy => copy(editor),
        Action::Cut => cut(editor),
        Action::Paste => {
            let text = editor.clipboard.clone();
            paste_text(editor, &text);
        }

        // Undo/Redo
        Action::Undo => {
            let (view_id, doc) = focused(editor);
            if !doc.undo(view_id) {
                editor.set_status("Nothing to undo", Severity::Info);
            }
        }
        Action::Redo => {
            let (view_id, doc) = focused(editor);
            if !doc.redo(view_id) {
                editor.set_status("Nothing to redo", Severity::Info);
            }
        }

        // Search
        Action::Find => {
            let initial = selected_text(editor).unwrap_or_else(|| editor.search_query.clone());
            return Outcome::OpenPrompt(PromptType::Search, initial);
        }
        Action::FindNext => find_again(editor, SearchDirection::Forward),
        Action::FindPrevious => find_again(editor, SearchDirection::Backward),
        Action::UseSelectionForFind => match selected_text(editor) {
            Some(text) => {
                editor.set_status(format!("Find: {text}"), Severity::Info);
                editor.search_query = text;
            }
            None => editor.set_status("Select some text first", Severity::Info),
        },
        Action::Replace => {
            let initial = selected_text(editor).unwrap_or_else(|| editor.search_query.clone());
            return Outcome::OpenPrompt(PromptType::Replace, initial);
        }

        // Prompt results
        Action::ExecuteGotoLine(input) => goto_line(editor, input),
        Action::ExecuteSearch(query) => {
            if !query.is_empty() {
                editor.search_query = query.clone();
                search(editor, SearchDirection::Forward, true);
            }
        }
        Action::ExecuteOpen(path) => {
            if !path.is_empty() {
                if let Err(e) = editor.open(path.as_str()) {
                    editor.set_status(format!("Error opening {path}: {e}"), Severity::Error);
                }
            }
        }
        Action::ExecuteSaveAs(path) => {
            if !path.is_empty() {
                if let Err(e) = editor.save_as(path.as_str()) {
                    editor.set_status(format!("Error saving: {e}"), Severity::Error);
                }
            }
        }
        Action::ExecuteReplaceFind(find) => {
            if !find.is_empty() {
                return Outcome::OpenPrompt(PromptType::ReplaceWith(find.clone()), String::new());
            }
        }
        Action::ExecuteReplace(find, replacement) => replace_all(editor, find, replacement),

        // Buffer/Tab management
        Action::NextBuffer => cycle_buffer(editor, 1),
        Action::PreviousBuffer => cycle_buffer(editor, -1),
        Action::SwitchToBuffer(n) => {
            let buffers = editor.buffer_list();
            if *n > 0 && *n <= buffers.len() {
                editor.switch_to_document(buffers[n - 1].0);
            }
        }

        // Splits
        Action::SplitVertical => editor.split(Layout::Vertical),
        Action::SplitHorizontal => editor.split(Layout::Horizontal),
        Action::FocusNextSplit => editor.tree.focus_next(),
        Action::FocusPreviousSplit => editor.tree.focus_prev(),

        // Bound in the keymap but not built yet: tell the user instead of
        // silently ignoring the key.
        Action::QuickOpen
        | Action::GotoSymbol
        | Action::JumpBack
        | Action::JumpForward
        | Action::FindInFiles
        | Action::Autocomplete
        | Action::GotoDefinition
        | Action::FindReferences
        | Action::RenameSymbol
        | Action::QuickFix
        | Action::SignatureHelp
        | Action::Hover
        | Action::Fold
        | Action::Unfold
        | Action::CommandPalette
        | Action::ToggleFileTree => {
            editor.set_status(format!("{action:?} is not available yet"), Severity::Info);
        }

        Action::Noop => {}
    }

    Outcome::Done
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VerticalDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SearchDirection {
    Forward,
    Backward,
}

/// Focused view and its document
fn focused(editor: &mut Editor) -> (ViewId, &mut Document) {
    let view_id = editor.tree.focus();
    (view_id, editor.current_doc_mut())
}

fn set_cursor(editor: &mut Editor, pos: usize) {
    let (view_id, doc) = focused(editor);
    doc.set_selection(view_id, Selection::point(pos));
}

/// Move every cursor with `f`, collapsing selections
fn move_cursors(editor: &mut Editor, f: impl Fn(&Rope, &Range) -> usize) {
    let (view_id, doc) = focused(editor);
    let selection = doc.selection(view_id);
    let rope = &doc.rope;
    let moved = selection.transform(|range| Range::point(f(rope, range)));
    doc.set_selection(view_id, moved);
}

/// Char index on `line` at the same display column as `from`
fn char_on_line(rope: &Rope, from: usize, line: usize, tab_width: usize) -> usize {
    let pos = rope.char_to_position(from);
    let col = display_col(rope.line(pos.line), pos.col, tab_width);
    rope.line_to_char(line) + char_col_at_display(rope.line(line), col, tab_width)
}

fn move_vertically(editor: &mut Editor, direction: VerticalDirection, count: usize) {
    let tab_width = editor.config.editor.tab_width;
    move_cursors(editor, |rope, range| {
        let line = rope.char_to_line(range.head);
        let last_line = rope.len_lines().saturating_sub(1);
        let target = match direction {
            VerticalDirection::Up => line.saturating_sub(count),
            VerticalDirection::Down => (line + count).min(last_line),
        };
        char_on_line(rope, range.head, target, tab_width)
    });
}

fn move_horizontally(editor: &mut Editor, direction: HorizontalDirection) {
    move_cursors(editor, |rope, range| match (direction, range.is_point()) {
        // With a selection, the first press only collapses it.
        (HorizontalDirection::Left, false) => range.start(),
        (HorizontalDirection::Right, false) => range.end(),
        (HorizontalDirection::Left, true) => step_back(rope, range.head),
        (HorizontalDirection::Right, true) => step_forward(rope, range.head),
    });
}

/// Position one character back, treating `\r\n` as a single step
fn step_back(rope: &Rope, pos: usize) -> usize {
    if pos >= 2 && rope.char(pos - 1) == '\n' && rope.char(pos - 2) == '\r' {
        pos - 2
    } else {
        pos.saturating_sub(1)
    }
}

/// Position one character forward, treating `\r\n` as a single step
fn step_forward(rope: &Rope, pos: usize) -> usize {
    let len = rope.len_chars();
    if pos + 1 < len && rope.char(pos) == '\r' && rope.char(pos + 1) == '\n' {
        pos + 2
    } else {
        (pos + 1).min(len)
    }
}

fn move_word(editor: &mut Editor, direction: HorizontalDirection) {
    move_cursors(editor, |rope, range| {
        let mut pos = range.head;
        let len = rope.len_chars();
        match direction {
            HorizontalDirection::Left => {
                while pos > 0 && !rope.is_word_char(pos - 1) {
                    pos -= 1;
                }
                while pos > 0 && rope.is_word_char(pos - 1) {
                    pos -= 1;
                }
            }
            HorizontalDirection::Right => {
                while pos < len && rope.is_word_char(pos) {
                    pos += 1;
                }
                while pos < len && !rope.is_word_char(pos) {
                    pos += 1;
                }
            }
        }
        pos
    });
}

fn move_line_start(editor: &mut Editor) {
    move_cursors(editor, |rope, range| {
        rope.line_to_char(rope.char_to_line(range.head))
    });
}

fn move_line_end(editor: &mut Editor) {
    move_cursors(editor, |rope, range| {
        let line = rope.char_to_line(range.head);
        rope.line_to_char(line) + rope.line_len_chars(line)
    });
}

fn page_move(editor: &mut Editor, direction: VerticalDirection) {
    let height = editor.current_view().height as usize;
    move_vertically(editor, direction, height.saturating_sub(2).max(1));
}

/// One replacement produced for one cursor
struct Edit {
    start: usize,
    end: usize,
    text: String,
    /// Where the cursor lands, in chars from `start` in the new text
    cursor: usize,
}

impl Edit {
    fn delete(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            text: String::new(),
            cursor: 0,
        }
    }

    fn replace(start: usize, end: usize, text: String) -> Self {
        let cursor = text.chars().count();
        Self {
            start,
            end,
            text,
            cursor,
        }
    }
}

/// Run `f` for every cursor and apply all resulting edits as one transaction.
/// Cursors that produce no edit keep their place.
fn edit_each_cursor(
    editor: &mut Editor,
    mergeable: bool,
    mut f: impl FnMut(usize, &Rope, &Range) -> Option<Edit>,
) {
    let (view_id, doc) = focused(editor);
    let selection = doc.selection(view_id);

    let mut changes = Vec::new();
    let mut new_ranges = Vec::with_capacity(selection.len());
    let mut delta: isize = 0;
    let mut last_end = 0;

    for (idx, range) in selection.ranges().iter().enumerate() {
        let shift = |pos: usize| pos.saturating_add_signed(delta);
        match f(idx, &doc.rope, range) {
            // Edits from neighbouring cursors may touch; the first one wins.
            Some(edit) if changes.is_empty() || edit.start >= last_end => {
                new_ranges.push(Range::point(shift(edit.start) + edit.cursor));
                delta += edit.text.chars().count() as isize - (edit.end - edit.start) as isize;
                last_end = edit.end;
                changes.push(Change::replace(edit.start, edit.end, edit.text));
            }
            _ => new_ranges.push(Range::point(shift(range.head))),
        }
    }

    if changes.is_empty() {
        return;
    }

    let tx = Transaction::changes(doc.len_chars(), &changes)
        .with_selection(Selection::from_ranges(new_ranges, selection.primary_idx()));
    if mergeable {
        doc.apply_mergeable(&tx, view_id);
    } else {
        doc.apply(&tx, view_id);
    }
}

/// Replace every selection with `text`
fn insert_text(editor: &mut Editor, text: &str, mergeable: bool) {
    edit_each_cursor(editor, mergeable, |_, _, range| {
        Some(Edit::replace(range.start(), range.end(), text.to_string()))
    });
}

fn insert_char(editor: &mut Editor, c: char) {
    let text = if c == '\t' && editor.config.editor.indent_style == IndentStyle::Spaces {
        " ".repeat(editor.config.editor.tab_width)
    } else {
        c.to_string()
    };
    insert_text(editor, &text, true);
}

fn insert_newline_below(editor: &mut Editor) {
    let line_ending = editor.current_doc().line_ending.as_str();
    edit_each_cursor(editor, false, |_, rope, range| {
        let line = rope.char_to_line(range.head);
        let line_end = rope.line_to_char(line) + rope.line_len_chars(line);
        Some(Edit::replace(line_end, line_end, line_ending.to_string()))
    });
}

fn insert_newline_above(editor: &mut Editor) {
    let line_ending = editor.current_doc().line_ending.as_str();
    edit_each_cursor(editor, false, |_, rope, range| {
        let line_start = rope.line_to_char(rope.char_to_line(range.head));
        Some(Edit {
            start: line_start,
            end: line_start,
            text: line_ending.to_string(),
            cursor: 0,
        })
    });
}

fn delete_backward(editor: &mut Editor) {
    edit_each_cursor(editor, false, |_, rope, range| {
        if !range.is_point() {
            Some(Edit::delete(range.start(), range.end()))
        } else if range.head > 0 {
            Some(Edit::delete(step_back(rope, range.head), range.head))
        } else {
            None
        }
    });
}

fn delete_forward(editor: &mut Editor) {
    edit_each_cursor(editor, false, |_, rope, range| {
        if !range.is_point() {
            Some(Edit::delete(range.start(), range.end()))
        } else if range.head < rope.len_chars() {
            Some(Edit::delete(range.head, step_forward(rope, range.head)))
        } else {
            None
        }
    });
}

/// Line numbers touched by any range, sorted and without duplicates. A range
/// that ends at the very start of a line does not include that line.
fn selected_lines(rope: &Rope, selection: &Selection) -> Vec<usize> {
    let mut lines = Vec::new();
    for range in selection.ranges() {
        let first = rope.char_to_line(range.start());
        let mut last = rope.char_to_line(range.end());
        if last > first && rope.line_to_char(last) == range.end() {
            last -= 1;
        }
        lines.extend(first..=last);
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// Apply line-based changes, letting every cursor follow the text
fn apply_line_changes(editor: &mut Editor, changes: Vec<Change>) {
    if changes.is_empty() {
        return;
    }
    let (view_id, doc) = focused(editor);
    let tx = Transaction::changes(doc.len_chars(), &changes);
    doc.apply(&tx, view_id);
}

fn delete_lines(editor: &mut Editor) {
    let (view_id, doc) = focused(editor);
    let rope = &doc.rope;
    let lines = selected_lines(rope, &doc.selection(view_id));

    // Merge runs of adjacent lines into blocks.
    let mut blocks: Vec<(usize, usize)> = Vec::new();
    for line in lines {
        match blocks.last_mut() {
            Some((_, last)) if *last + 1 == line => *last = line,
            _ => blocks.push((line, line)),
        }
    }

    let mut changes = Vec::new();
    for (first, last) in blocks {
        let mut start = rope.line_to_char(first);
        let end = if last + 1 < rope.len_lines() {
            rope.line_to_char(last + 1)
        } else {
            rope.len_chars()
        };
        // Deleting the final line also removes the line break before it.
        if end == rope.len_chars() && start > 0 {
            start = step_back(rope, start);
        }
        if start < end {
            changes.push(Change::delete(start, end));
        }
    }
    apply_line_changes(editor, changes);
}

fn duplicate_line(editor: &mut Editor) {
    let line_ending = editor.current_doc().line_ending.as_str();
    let (view_id, doc) = focused(editor);
    let cursor = doc.selection(view_id).cursor();
    let rope = &doc.rope;

    let line = rope.char_to_line(cursor);
    let line_start = rope.line_to_char(line);
    let content_end = line_start + rope.line_len_chars(line);
    let content: String = rope.slice(line_start..content_end).chars().collect();
    let next_line_start = if line + 1 < rope.len_lines() {
        rope.line_to_char(line + 1)
    } else {
        rope.len_chars()
    };
    let ending: String = rope.slice(content_end..next_line_start).chars().collect();

    let (insert_at, text) = if ending.is_empty() {
        (content_end, format!("{line_ending}{content}"))
    } else {
        (next_line_start, format!("{content}{ending}"))
    };
    let ending_len = if ending.is_empty() {
        line_ending.len()
    } else {
        ending.chars().count()
    };
    let new_cursor = cursor + content.chars().count() + ending_len;

    let tx = Transaction::insert(doc.len_chars(), insert_at, text)
        .with_selection(Selection::point(new_cursor));
    doc.apply(&tx, view_id);
}

fn move_line(editor: &mut Editor, direction: VerticalDirection) {
    let (view_id, doc) = focused(editor);
    let cursor = doc.selection(view_id).cursor();
    let rope = &doc.rope;
    let line = rope.char_to_line(cursor);
    let col = cursor - rope.line_to_char(line);

    let (first, second) = match direction {
        VerticalDirection::Up if line > 0 => (line - 1, line),
        VerticalDirection::Down if line + 1 < rope.len_lines() => (line, line + 1),
        _ => return,
    };
    let first_start = rope.line_to_char(first);
    let first_end = first_start + rope.line_len_chars(first);
    let second_start = rope.line_to_char(second);
    let second_end = second_start + rope.line_len_chars(second);
    // The empty line after a trailing newline is not a real line to swap with.
    if second_start == rope.len_chars() {
        return;
    }

    let first_text: String = rope.slice(first_start..first_end).chars().collect();
    let between: String = rope.slice(first_end..second_start).chars().collect();
    let second_text: String = rope.slice(second_start..second_end).chars().collect();

    let new_cursor = match direction {
        VerticalDirection::Up => first_start + col,
        VerticalDirection::Down => {
            first_start + second_text.chars().count() + between.chars().count() + col
        }
    };

    let tx = Transaction::replace(
        doc.len_chars(),
        first_start,
        second_end,
        format!("{second_text}{between}{first_text}"),
    )
    .with_selection(Selection::point(new_cursor));
    doc.apply(&tx, view_id);
}

fn indent_unit(editor: &Editor) -> String {
    match editor.config.editor.indent_style {
        IndentStyle::Spaces => " ".repeat(editor.config.editor.tab_width),
        IndentStyle::Tabs => "\t".to_string(),
    }
}

fn indent(editor: &mut Editor) {
    let unit = indent_unit(editor);
    let (view_id, doc) = focused(editor);
    let selection = doc.selection(view_id);

    // Without a selection Tab just types an indent at the cursor.
    if !selection.has_selection() {
        insert_text(editor, &unit, true);
        return;
    }

    let rope = &doc.rope;
    let lines = selected_lines(rope, &selection);
    let skip_blank = lines.len() > 1;
    let changes = lines
        .into_iter()
        .filter(|line| !skip_blank || rope.line_len_chars(*line) > 0)
        .map(|line| Change::insert(rope.line_to_char(line), unit.clone()))
        .collect();
    apply_line_changes(editor, changes);
}

fn unindent(editor: &mut Editor) {
    let tab_width = editor.config.editor.tab_width;
    let (view_id, doc) = focused(editor);
    let rope = &doc.rope;
    let changes = selected_lines(rope, &doc.selection(view_id))
        .into_iter()
        .filter_map(|line| {
            let start = rope.line_to_char(line);
            let mut chars = rope.line(line).chars();
            let remove = if rope.line(line).chars().next() == Some('\t') {
                1
            } else {
                chars
                    .by_ref()
                    .take(tab_width)
                    .take_while(|c| *c == ' ')
                    .count()
            };
            (remove > 0).then(|| Change::delete(start, start + remove))
        })
        .collect();
    apply_line_changes(editor, changes);
}

/// Comment markers for a language: (open, close). `close` is empty for line comments.
fn comment_tokens(language: Option<&str>) -> Option<(&'static str, &'static str)> {
    Some(match language? {
        "rust" | "javascript" | "typescript" | "go" | "c" | "cpp" | "java" | "swift" | "kotlin"
        | "scala" | "php" | "scss" | "less" => ("//", ""),
        "python" | "bash" | "fish" | "ruby" | "yaml" | "toml" | "elixir" | "makefile"
        | "dockerfile" | "powershell" => ("#", ""),
        "lua" | "sql" | "haskell" => ("--", ""),
        "latex" | "erlang" => ("%", ""),
        "clojure" => (";", ""),
        "vim" => ("\"", ""),
        "ocaml" => ("(*", "*)"),
        "css" => ("/*", "*/"),
        "html" | "xml" | "markdown" => ("<!--", "-->"),
        _ => return None,
    })
}

fn toggle_comment(editor: &mut Editor) {
    let language = editor.current_doc().language.clone();
    let Some((open, close)) = comment_tokens(language.as_deref()) else {
        editor.set_status(
            "Comments are not supported for this file type",
            Severity::Info,
        );
        return;
    };

    let (view_id, doc) = focused(editor);
    let rope = &doc.rope;

    // (line start, indent width, content)
    let lines: Vec<(usize, usize, String)> = selected_lines(rope, &doc.selection(view_id))
        .into_iter()
        .map(|line| {
            let start = rope.line_to_char(line);
            let content: String = rope
                .slice(start..start + rope.line_len_chars(line))
                .chars()
                .collect();
            let indent = content.chars().take_while(|c| c.is_whitespace()).count();
            (start, indent, content)
        })
        .collect();
    let non_blank: Vec<_> = lines
        .iter()
        .filter(|(_, indent, c)| *indent < c.chars().count())
        .collect();
    let targets = if non_blank.is_empty() {
        lines.iter().collect()
    } else {
        non_blank
    };

    let is_commented = |content: &str| {
        let trimmed = content.trim();
        trimmed.starts_with(open) && trimmed.ends_with(close)
    };
    let uncomment = targets.iter().all(|(_, _, content)| is_commented(content));

    // Each line's body (after indentation, before trailing spaces) is
    // replaced as a whole.
    let changes = targets
        .into_iter()
        .map(|(start, indent, content)| {
            let body: String = content
                .chars()
                .skip(*indent)
                .collect::<String>()
                .trim_end()
                .to_string();
            let body_start = start + indent;
            let body_end = body_start + body.chars().count();

            let new_body = if uncomment {
                let inner = &body[open.len()..body.len() - close.len()];
                let inner = inner.strip_prefix(' ').unwrap_or(inner);
                let inner = if close.is_empty() {
                    inner
                } else {
                    inner.strip_suffix(' ').unwrap_or(inner)
                };
                inner.to_string()
            } else if close.is_empty() {
                format!("{open} {body}")
            } else {
                format!("{open} {body} {close}")
            };
            Change::replace(body_start, body_end, new_body)
        })
        .collect();
    apply_line_changes(editor, changes);
}

fn select_line(editor: &mut Editor) {
    let (view_id, doc) = focused(editor);
    let selection = doc.selection(view_id);
    let rope = &doc.rope;
    let lines = selection.transform(|range| {
        let first = rope.char_to_line(range.start());
        let last = rope.char_to_line(range.end());
        let end = if last + 1 < rope.len_lines() {
            rope.line_to_char(last + 1)
        } else {
            rope.len_chars()
        };
        Range::new(rope.line_to_char(first), end)
    });
    doc.set_selection(view_id, lines);
}

fn select_word(editor: &mut Editor) {
    let (view_id, doc) = focused(editor);
    let selection = doc.selection(view_id);
    let rope = &doc.rope;
    let words = selection.transform(|range| {
        let (start, end) = rope.word_at(range.head);
        Range::new(start, end)
    });
    doc.set_selection(view_id, words);
}

/// Text of the primary selection, if it is not empty
fn selected_text(editor: &Editor) -> Option<String> {
    let doc = editor.current_doc();
    let primary = *doc.selection(editor.tree.focus()).primary();
    (!primary.is_point()).then(|| doc.rope.slice(primary.start()..primary.end()).to_string())
}

/// Char indices of every match of `needle`
fn find_all(rope: &Rope, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    let text = rope.to_string();
    text.match_indices(needle)
        .map(|(byte_idx, _)| rope.byte_to_char(byte_idx))
        .collect()
}

/// Find `needle` starting at char `from`, wrapping around the document.
/// Returns the char index of the match and whether the search wrapped.
fn find_from(
    rope: &Rope,
    needle: &str,
    from: usize,
    direction: SearchDirection,
) -> Option<(usize, bool)> {
    if needle.is_empty() {
        return None;
    }
    let text = rope.to_string();
    let from_byte = rope.char_to_byte(from.min(rope.len_chars()));

    let (found, wrapped) = match direction {
        SearchDirection::Forward => match text[from_byte..].find(needle) {
            Some(idx) => (Some(from_byte + idx), false),
            None => (text.find(needle), true),
        },
        SearchDirection::Backward => match text[..from_byte].rfind(needle) {
            Some(idx) => (Some(idx), false),
            None => (text.rfind(needle), true),
        },
    };
    found.map(|byte_idx| (rope.byte_to_char(byte_idx), wrapped))
}

/// Select the next or previous match of the search query
fn search(editor: &mut Editor, direction: SearchDirection, include_cursor: bool) {
    let query = editor.search_query.clone();
    let (view_id, doc) = focused(editor);
    let primary = *doc.selection(view_id).primary();
    let from = match direction {
        SearchDirection::Forward if include_cursor => primary.start(),
        SearchDirection::Forward => primary.end(),
        SearchDirection::Backward => primary.start(),
    };

    match find_from(&doc.rope, &query, from, direction) {
        Some((start, wrapped)) => {
            let end = start + query.chars().count();
            doc.set_selection(view_id, Selection::single(Range::new(start, end)));
            if wrapped {
                editor.set_status("Search wrapped around", Severity::Info);
            }
        }
        None => editor.set_status(format!("Not found: {query}"), Severity::Error),
    }
}

fn find_again(editor: &mut Editor, direction: SearchDirection) {
    if editor.search_query.is_empty() {
        editor.set_status("Nothing to search for yet", Severity::Info);
        return;
    }
    search(editor, direction, false);
}

fn replace_all(editor: &mut Editor, find: &str, replacement: &str) {
    let (view_id, doc) = focused(editor);
    let len = find.chars().count();
    let changes: Vec<Change> = find_all(&doc.rope, find)
        .into_iter()
        .map(|start| Change::replace(start, start + len, replacement.to_string()))
        .collect();
    let count = changes.len();

    if count == 0 {
        editor.set_status(format!("Not found: {find}"), Severity::Error);
        return;
    }
    let tx = Transaction::changes(doc.len_chars(), &changes);
    doc.apply(&tx, view_id);
    editor.search_query = find.to_string();
    let noun = if count == 1 {
        "occurrence"
    } else {
        "occurrences"
    };
    editor.set_status(format!("Replaced {count} {noun}"), Severity::Info);
}

/// Text to search for: the primary selection, or the word under the cursor
fn occurrence_needle(rope: &Rope, primary: Range) -> Option<(usize, usize)> {
    let (start, end) = if primary.is_point() {
        rope.word_at(primary.head)
    } else {
        (primary.start(), primary.end())
    };
    (start < end).then_some((start, end))
}

fn select_next_occurrence(editor: &mut Editor) {
    let (view_id, doc) = focused(editor);
    let mut selection = doc.selection(view_id);
    let primary = *selection.primary();
    let Some((start, end)) = occurrence_needle(&doc.rope, primary) else {
        return;
    };

    // First press only selects the word under the cursor.
    if primary.is_point() {
        doc.set_selection(view_id, Selection::single(Range::new(start, end)));
        return;
    }

    let needle: String = doc.rope.slice(start..end).chars().collect();
    let last_end = selection
        .ranges()
        .iter()
        .map(Range::end)
        .max()
        .unwrap_or(end);
    let Some((found, _)) = find_from(&doc.rope, &needle, last_end, SearchDirection::Forward) else {
        return;
    };
    let range = Range::new(found, found + needle.chars().count());
    if selection.ranges().contains(&range) {
        editor.set_status("No more occurrences", Severity::Info);
        return;
    }

    selection.add_range(range);
    if let Some(idx) = selection.ranges().iter().position(|r| *r == range) {
        selection.set_primary_idx(idx);
    }
    doc.set_selection(view_id, selection);
}

fn select_all_occurrences(editor: &mut Editor) {
    let (view_id, doc) = focused(editor);
    let primary = *doc.selection(view_id).primary();
    let Some((start, end)) = occurrence_needle(&doc.rope, primary) else {
        return;
    };
    let needle: String = doc.rope.slice(start..end).chars().collect();
    let len = needle.chars().count();
    let ranges: Vec<Range> = find_all(&doc.rope, &needle)
        .into_iter()
        .map(|pos| Range::new(pos, pos + len))
        .collect();
    let primary_idx = ranges.iter().position(|r| r.start() == start).unwrap_or(0);
    let count = ranges.len();
    doc.set_selection(view_id, Selection::from_ranges(ranges, primary_idx));
    editor.set_status(format!("{count} cursors"), Severity::Info);
}

fn split_selection_lines(editor: &mut Editor) {
    let (view_id, doc) = focused(editor);
    let selection = doc.selection(view_id);
    let rope = &doc.rope;

    let mut ranges = Vec::new();
    for range in selection.ranges() {
        let first = rope.char_to_line(range.start());
        let last = rope.char_to_line(range.end());
        if first == last {
            ranges.push(*range);
            continue;
        }
        for line in first..=last {
            let line_start = rope.line_to_char(line);
            let start = range.start().max(line_start);
            let end = range.end().min(line_start + rope.line_len_chars(line));
            if start < end || line == first {
                ranges.push(Range::new(start, end.max(start)));
            }
        }
    }
    doc.set_selection(view_id, Selection::from_ranges(ranges, 0));
}

fn add_cursor(editor: &mut Editor, direction: VerticalDirection) {
    let tab_width = editor.config.editor.tab_width;
    let (view_id, doc) = focused(editor);
    let mut selection = doc.selection(view_id);
    let rope = &doc.rope;

    // Grow from the outermost cursor in the chosen direction.
    let edge = match direction {
        VerticalDirection::Up => selection.ranges().first(),
        VerticalDirection::Down => selection.ranges().last(),
    }
    .copied()
    .unwrap_or_default();
    let line = rope.char_to_line(edge.head);
    let target = match direction {
        VerticalDirection::Up if line > 0 => line - 1,
        VerticalDirection::Down if line + 1 < rope.len_lines() => line + 1,
        _ => return,
    };

    let pos = char_on_line(rope, edge.head, target, tab_width);
    selection.add_cursor(pos);
    doc.set_selection(view_id, selection);
}

fn copy(editor: &mut Editor) {
    let doc = editor.current_doc();
    let selection = doc.selection(editor.tree.focus());
    let line_ending = doc.line_ending.as_str();

    let text = if selection.has_selection() {
        selection
            .ranges()
            .iter()
            .filter(|r| !r.is_point())
            .map(|r| doc.rope.slice(r.start()..r.end()).to_string())
            .collect::<Vec<_>>()
            .join(line_ending)
    } else {
        // No selection: copy the whole line, like Sublime.
        let line = doc.rope.char_to_line(selection.cursor());
        doc.rope.line(line).to_string()
    };

    editor.clipboard = text;
    editor.set_status("Copied", Severity::Info);
}

fn cut(editor: &mut Editor) {
    copy(editor);
    let has_selection = editor
        .current_doc()
        .selection(editor.tree.focus())
        .has_selection();
    if has_selection {
        edit_each_cursor(editor, false, |_, _, range| {
            (!range.is_point()).then(|| Edit::delete(range.start(), range.end()))
        });
    } else {
        delete_lines(editor);
    }
    editor.set_status("Cut", Severity::Info);
}

/// Insert text from the clipboard or a terminal paste. With one clipboard line
/// per cursor, each cursor gets its own line.
pub fn paste_text(editor: &mut Editor, text: &str) {
    if text.is_empty() {
        return;
    }
    let line_ending = editor.current_doc().line_ending.as_str();
    let normalized = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', line_ending);

    let cursor_count = editor.current_doc().selection(editor.tree.focus()).len();
    let parts: Vec<&str> = normalized.split(line_ending).collect();
    if cursor_count > 1 && parts.len() == cursor_count {
        edit_each_cursor(editor, false, |idx, _, range| {
            Some(Edit::replace(
                range.start(),
                range.end(),
                parts[idx].to_string(),
            ))
        });
    } else {
        insert_text(editor, &normalized, false);
    }
}

fn goto_line(editor: &mut Editor, input: &str) {
    let mut parts = input.trim().splitn(2, ':');
    let line = parts.next().and_then(|s| s.trim().parse::<usize>().ok());
    let col = parts
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(1);

    let Some(line) = line.filter(|l| *l > 0) else {
        editor.set_status(format!("Not a line number: {input}"), Severity::Error);
        return;
    };

    let (view_id, doc) = focused(editor);
    let rope = &doc.rope;
    let target = (line - 1).min(rope.len_lines().saturating_sub(1));
    let pos = rope.line_to_char(target) + (col - 1).min(rope.line_len_chars(target));
    doc.set_selection(view_id, Selection::point(pos));
}

fn cycle_buffer(editor: &mut Editor, step: isize) {
    let buffers = editor.buffer_list();
    let current = editor.current_view().doc_id;
    if let Some(idx) = buffers.iter().position(|(id, _)| *id == current) {
        let next = (idx as isize + step).rem_euclid(buffers.len() as isize) as usize;
        editor.switch_to_document(buffers[next].0);
    }
}

/// Focus the view under a mouse click and place the cursor there
pub fn click(editor: &mut Editor, column: u16, row: u16) {
    let tab_width = editor.config.editor.tab_width;
    let Some((view_id, area)) = view_at(editor, column, row) else {
        return;
    };
    editor.tree.set_focus(view_id);

    let view = &editor.views[&view_id];
    let line = view.scroll_y + (row - area.y) as usize;
    let text_col = (column - area.x).saturating_sub(view.gutter_width) as usize + view.scroll_x;

    let doc = editor.current_doc_mut();
    let line = line.min(doc.len_lines().saturating_sub(1));
    let pos =
        doc.rope.line_to_char(line) + char_col_at_display(doc.rope.line(line), text_col, tab_width);
    doc.set_selection(view_id, Selection::point(pos));
}

/// Scroll the view under the mouse pointer without moving its cursor
pub fn scroll(editor: &mut Editor, column: u16, row: u16, down: bool) {
    let Some((view_id, _)) = view_at(editor, column, row) else {
        return;
    };
    let doc_id = editor.views[&view_id].doc_id;
    let line_count = editor.documents[&doc_id].len_lines();
    let delta = if down {
        MOUSE_SCROLL_LINES as isize
    } else {
        -(MOUSE_SCROLL_LINES as isize)
    };
    if let Some(view) = editor.views.get_mut(&view_id) {
        view.scroll(delta, line_count);
    }
}

fn view_at(editor: &Editor, column: u16, row: u16) -> Option<(ViewId, lite_view::Area)> {
    editor.layout().views.into_iter().find(|(_, a)| {
        column >= a.x && column < a.x + a.width && row >= a.y && row < a.y + a.height
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_with(text: &str) -> Editor {
        let mut editor = Editor::new();
        editor.set_area(lite_view::Area::new(0, 0, 80, 20));
        let view_id = editor.tree.focus();
        let doc = editor.current_doc_mut();
        let tx = Transaction::insert(0, 0, text.to_string());
        doc.apply(&tx, view_id);
        doc.set_selection(view_id, Selection::point(0));
        editor
    }

    fn text(editor: &Editor) -> String {
        editor.current_doc().text()
    }

    fn select(editor: &mut Editor, ranges: Vec<Range>) {
        let (view_id, doc) = focused(editor);
        doc.set_selection(view_id, Selection::from_ranges(ranges, 0));
    }

    fn cursors(editor: &Editor) -> Vec<usize> {
        let doc = editor.current_doc();
        doc.selection(editor.tree.focus())
            .ranges()
            .iter()
            .map(|r| r.head)
            .collect()
    }

    #[test]
    fn test_typing_reaches_every_cursor() {
        let mut editor = editor_with("ab\ncd\n");
        select(&mut editor, vec![Range::point(0), Range::point(3)]);
        execute_action(&mut editor, &Action::InsertChar('x'));
        assert_eq!(text(&editor), "xab\nxcd\n");
        assert_eq!(cursors(&editor), vec![1, 5]);
    }

    #[test]
    fn test_backspace_reaches_every_cursor() {
        let mut editor = editor_with("ab\ncd\n");
        select(&mut editor, vec![Range::point(1), Range::point(4)]);
        execute_action(&mut editor, &Action::Backspace);
        assert_eq!(text(&editor), "b\nd\n");
    }

    #[test]
    fn test_paste_non_ascii_places_cursor_after_text() {
        let mut editor = editor_with("");
        editor.clipboard = "héllo 日本".to_string();
        execute_action(&mut editor, &Action::Paste);
        assert_eq!(cursors(&editor), vec![8]);
    }

    #[test]
    fn test_search_with_non_ascii_text() {
        let mut editor = editor_with("ééé foo bar foo");
        execute_action(&mut editor, &Action::ExecuteSearch("foo".into()));
        let doc = editor.current_doc();
        let primary = *doc.selection(editor.tree.focus()).primary();
        assert_eq!((primary.start(), primary.end()), (4, 7));

        execute_action(&mut editor, &Action::FindNext);
        let primary = *editor
            .current_doc()
            .selection(editor.tree.focus())
            .primary();
        assert_eq!(primary.start(), 12);
    }

    #[test]
    fn test_select_next_occurrence_with_non_ascii_text() {
        let mut editor = editor_with("日本 foo 日本 foo");
        select(&mut editor, vec![Range::point(4)]);
        execute_action(&mut editor, &Action::SelectNextOccurrence);
        execute_action(&mut editor, &Action::SelectNextOccurrence);
        let doc = editor.current_doc();
        let ranges: Vec<_> = doc.selection(editor.tree.focus()).ranges().to_vec();
        assert_eq!(ranges, vec![Range::new(3, 6), Range::new(10, 13)]);

        execute_action(&mut editor, &Action::InsertChar('X'));
        assert_eq!(text(&editor), "日本 X 日本 X");
    }

    #[test]
    fn test_replace_all() {
        let mut editor = editor_with("a.b.c");
        execute_action(
            &mut editor,
            &Action::ExecuteReplace(".".into(), "::".into()),
        );
        assert_eq!(text(&editor), "a::b::c");
    }

    #[test]
    fn test_duplicate_last_line_without_newline() {
        let mut editor = editor_with("one\ntwo");
        select(&mut editor, vec![Range::point(5)]);
        execute_action(&mut editor, &Action::DuplicateLine);
        assert_eq!(text(&editor), "one\ntwo\ntwo");
    }

    #[test]
    fn test_move_line_down_onto_last_line_without_newline() {
        let mut editor = editor_with("a\nb");
        execute_action(&mut editor, &Action::MoveLineDown);
        assert_eq!(text(&editor), "b\na");
        assert_eq!(cursors(&editor), vec![2]);
    }

    #[test]
    fn test_toggle_comment_uses_language() {
        let mut editor = editor_with("x = 1\n");
        editor.current_doc_mut().language = Some("python".into());
        execute_action(&mut editor, &Action::ToggleComment);
        assert_eq!(text(&editor), "# x = 1\n");
        execute_action(&mut editor, &Action::ToggleComment);
        assert_eq!(text(&editor), "x = 1\n");
    }

    #[test]
    fn test_toggle_block_comment() {
        let mut editor = editor_with("  <p>hi</p>\n");
        editor.current_doc_mut().language = Some("html".into());
        execute_action(&mut editor, &Action::ToggleComment);
        assert_eq!(text(&editor), "  <!-- <p>hi</p> -->\n");
        execute_action(&mut editor, &Action::ToggleComment);
        assert_eq!(text(&editor), "  <p>hi</p>\n");
    }

    #[test]
    fn test_indent_selected_lines() {
        let mut editor = editor_with("a\nb\n");
        select(&mut editor, vec![Range::new(0, 3)]);
        execute_action(&mut editor, &Action::Indent);
        assert_eq!(text(&editor), "    a\n    b\n");
        execute_action(&mut editor, &Action::Unindent);
        assert_eq!(text(&editor), "a\nb\n");
    }

    #[test]
    fn test_crlf_backspace_removes_whole_line_break() {
        let mut editor = editor_with("a\r\nb");
        select(&mut editor, vec![Range::point(3)]);
        execute_action(&mut editor, &Action::Backspace);
        assert_eq!(text(&editor), "ab");
    }

    #[test]
    fn test_delete_last_line_removes_line_break() {
        let mut editor = editor_with("a\nb");
        select(&mut editor, vec![Range::point(2)]);
        execute_action(&mut editor, &Action::DeleteLine);
        assert_eq!(text(&editor), "a");
    }

    #[test]
    fn test_save_untitled_opens_save_as_prompt() {
        let mut editor = editor_with("x");
        let outcome = execute_action(&mut editor, &Action::Save);
        assert_eq!(
            outcome,
            Outcome::OpenPrompt(PromptType::SaveAs, String::new())
        );
    }

    #[test]
    fn test_paste_one_line_per_cursor() {
        let mut editor = editor_with("a\nb\n");
        select(&mut editor, vec![Range::point(1), Range::point(3)]);
        paste_text(&mut editor, "1\n2");
        assert_eq!(text(&editor), "a1\nb2\n");
    }
}
