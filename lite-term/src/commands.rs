use lite_config::Action;
use lite_core::{Range, RopeExt, Selection, Transaction};
use lite_view::{Editor, Layout, Severity};

/// Execute an action on the editor
pub fn execute_action(editor: &mut Editor, action: &Action) {
    match action {
        // File operations
        Action::Save => {
            if let Err(e) = editor.save() {
                editor.set_status(format!("Error saving: {}", e), Severity::Error);
            }
        }
        Action::SaveAs => {
            // This should open a prompt - handled by application
        }
        Action::Open => {
            // This should open a prompt - handled by application
        }
        Action::QuickOpen => {
            // This should open a file picker - handled by application
        }
        Action::CloseBuffer => {
            editor.close_buffer();
        }
        Action::CloseWindow => {
            editor.close_view();
        }
        Action::Quit => {
            // Just quit - user can save with Ctrl+S first if needed
            editor.should_quit = true;
        }

        // Navigation
        Action::MoveUp => move_cursor(editor, Direction::Up, 1),
        Action::MoveDown => move_cursor(editor, Direction::Down, 1),
        Action::MoveLeft => move_cursor(editor, Direction::Left, 1),
        Action::MoveRight => move_cursor(editor, Direction::Right, 1),
        Action::MoveWordLeft => move_word(editor, Direction::Left),
        Action::MoveWordRight => move_word(editor, Direction::Right),
        Action::MoveLineStart => move_line_start(editor),
        Action::MoveLineEnd => move_line_end(editor),
        Action::MoveFileStart => move_file_start(editor),
        Action::MoveFileEnd => move_file_end(editor),
        Action::PageUp => page_move(editor, Direction::Up),
        Action::PageDown => page_move(editor, Direction::Down),
        Action::GotoLine => {
            // Handled by application - opens prompt
        }
        Action::GotoSymbol => {
            // TODO: LSP integration
        }
        Action::JumpBack | Action::JumpForward => {
            // TODO: Jump list
        }

        // Editing
        Action::InsertChar(c) => insert_char(editor, *c),
        Action::InsertNewline => insert_newline(editor),
        Action::InsertNewlineBelow => insert_newline_below(editor),
        Action::InsertNewlineAbove => insert_newline_above(editor),
        Action::Backspace => delete_backward(editor),
        Action::Delete => delete_forward(editor),
        Action::DeleteLine => delete_line(editor),
        Action::DuplicateLine => duplicate_line(editor),
        Action::MoveLineUp => move_line(editor, Direction::Up),
        Action::MoveLineDown => move_line(editor, Direction::Down),
        Action::Indent => indent(editor),
        Action::Unindent => unindent(editor),
        Action::ToggleComment => toggle_comment(editor),

        // Selection
        Action::SelectAll => select_all(editor),
        Action::SelectLine => select_line(editor),
        Action::SelectWord => select_word(editor),
        Action::SelectNextOccurrence => select_next_occurrence(editor),
        Action::SelectAllOccurrences => {
            // TODO
        }
        Action::SplitSelectionLines => {
            // TODO
        }
        Action::AddCursorAbove => add_cursor(editor, Direction::Up),
        Action::AddCursorBelow => add_cursor(editor, Direction::Down),
        Action::ClearSelection => clear_selection(editor),

        // Clipboard
        Action::Copy => copy(editor),
        Action::Cut => cut(editor),
        Action::Paste => paste(editor),

        // Undo/Redo
        Action::Undo => undo(editor),
        Action::Redo => redo(editor),

        // Search - handled by application
        Action::Find
        | Action::FindNext
        | Action::FindPrevious
        | Action::Replace
        | Action::FindInFiles
        | Action::UseSelectionForFind => {}

        // Buffer/Tab management
        Action::NextBuffer => {
            let buffers = editor.buffer_list();
            let current = editor.current_view().doc_id;
            if let Some((idx, _)) = buffers.iter().enumerate().find(|(_, (id, _))| *id == current) {
                let next_idx = (idx + 1) % buffers.len();
                editor.switch_to_document(buffers[next_idx].0);
            }
        }
        Action::PreviousBuffer => {
            let buffers = editor.buffer_list();
            let current = editor.current_view().doc_id;
            if let Some((idx, _)) = buffers.iter().enumerate().find(|(_, (id, _))| *id == current) {
                let prev_idx = if idx == 0 { buffers.len() - 1 } else { idx - 1 };
                editor.switch_to_document(buffers[prev_idx].0);
            }
        }
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

        // LSP - handled elsewhere
        Action::Autocomplete
        | Action::GotoDefinition
        | Action::FindReferences
        | Action::RenameSymbol
        | Action::QuickFix
        | Action::SignatureHelp
        | Action::Hover => {}

        // Code folding - TODO
        Action::Fold | Action::Unfold => {}

        // UI - handled by application
        Action::CommandPalette | Action::ToggleFileTree => {}

        // Prompt results - handled by application
        Action::ExecuteGotoLine(_) | Action::ExecuteSearch(_) | Action::ExecuteOpen(_) | Action::ExecuteSaveAs(_) => {}

        Action::Noop => {}
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn move_cursor(editor: &mut Editor, direction: Direction, count: usize) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);

    let new_selection = selection.transform(|range| {
        let pos = doc.rope.char_to_position(range.head);
        let new_pos = match direction {
            Direction::Up => lite_core::Position::new(pos.line.saturating_sub(count), pos.col),
            Direction::Down => lite_core::Position::new(
                (pos.line + count).min(doc.len_lines().saturating_sub(1)),
                pos.col,
            ),
            Direction::Left => {
                let new_char = range.head.saturating_sub(count);
                return Range::point(new_char);
            }
            Direction::Right => {
                let new_char = (range.head + count).min(doc.len_chars());
                return Range::point(new_char);
            }
        };

        // Clamp column to line length
        let line_len = doc.rope.line_len_chars(new_pos.line);
        let clamped_pos = lite_core::Position::new(new_pos.line, new_pos.col.min(line_len));
        let new_char = doc.rope.position_to_char(clamped_pos);
        Range::point(new_char)
    });

    doc.set_selection(view_id, new_selection);

    // Ensure cursor visibility
    let cursor_pos = doc.rope.char_to_position(doc.selection(view_id).cursor());
    let scrolloff = editor.config.editor.scrolloff;
    editor
        .current_view_mut()
        .ensure_cursor_visible(cursor_pos.line, cursor_pos.col, scrolloff);
}

fn move_word(editor: &mut Editor, direction: Direction) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);

    let new_selection = selection.transform(|range| {
        let mut pos = range.head;
        let len = doc.len_chars();

        match direction {
            Direction::Left => {
                // Skip whitespace
                while pos > 0 && !doc.rope.is_word_char(pos.saturating_sub(1)) {
                    pos -= 1;
                }
                // Move through word
                while pos > 0 && doc.rope.is_word_char(pos.saturating_sub(1)) {
                    pos -= 1;
                }
            }
            Direction::Right => {
                // Move through word
                while pos < len && doc.rope.is_word_char(pos) {
                    pos += 1;
                }
                // Skip whitespace
                while pos < len && !doc.rope.is_word_char(pos) {
                    pos += 1;
                }
            }
            _ => {}
        }

        Range::point(pos)
    });

    doc.set_selection(view_id, new_selection);
}

fn move_line_start(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);

    let new_selection = selection.transform(|range| {
        let line = doc.rope.char_to_line(range.head);
        let line_start = doc.rope.line_to_char(line);
        Range::point(line_start)
    });

    doc.set_selection(view_id, new_selection);
}

fn move_line_end(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);

    let new_selection = selection.transform(|range| {
        let line = doc.rope.char_to_line(range.head);
        let line_end = doc.rope.line_to_char(line) + doc.rope.line_len_chars(line);
        Range::point(line_end)
    });

    doc.set_selection(view_id, new_selection);
}

fn move_file_start(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    doc.set_selection(view_id, Selection::point(0));
}

fn move_file_end(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let end = doc.len_chars();
    doc.set_selection(view_id, Selection::point(end));
}

fn page_move(editor: &mut Editor, direction: Direction) {
    let height = editor.current_view().height as usize;
    move_cursor(editor, direction, height.saturating_sub(2));
}

fn insert_char(editor: &mut Editor, c: char) {
    let view_id = editor.tree.focus();
    let indent_style = editor.config.editor.indent_style;
    let tab_width = editor.config.editor.tab_width;

    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let text = if c == '\t' && indent_style == lite_config::IndentStyle::Spaces {
        " ".repeat(tab_width)
    } else {
        c.to_string()
    };

    let tx =
        Transaction::insert(doc.len_chars(), cursor, text.clone()).with_selection(Selection::point(
            cursor + text.chars().count(),
        ));

    doc.apply(&tx, view_id);
}

fn insert_newline(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line_ending = doc.line_ending.as_str();
    let tx = Transaction::insert(doc.len_chars(), cursor, line_ending)
        .with_selection(Selection::point(cursor + line_ending.len()));

    doc.apply(&tx, view_id);
}

fn insert_newline_below(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_end = doc.rope.line_to_char(line) + doc.rope.line_len_chars(line);
    let line_ending = doc.line_ending.as_str();

    let tx = Transaction::insert(doc.len_chars(), line_end, line_ending)
        .with_selection(Selection::point(line_end + line_ending.len()));

    doc.apply(&tx, view_id);
}

fn insert_newline_above(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);
    let line_ending = doc.line_ending.as_str();

    let tx = Transaction::insert(doc.len_chars(), line_start, line_ending)
        .with_selection(Selection::point(line_start));

    doc.apply(&tx, view_id);
}

fn delete_backward(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);

    if selection.has_selection() {
        // Delete selection
        let range = selection.primary();
        let tx = Transaction::delete(doc.len_chars(), range.start(), range.end())
            .with_selection(Selection::point(range.start()));
        doc.apply(&tx, view_id);
    } else {
        // Delete one char backward
        let cursor = selection.cursor();
        if cursor > 0 {
            let tx = Transaction::delete(doc.len_chars(), cursor - 1, cursor)
                .with_selection(Selection::point(cursor - 1));
            doc.apply(&tx, view_id);
        }
    }
}

fn delete_forward(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);

    if selection.has_selection() {
        // Delete selection
        let range = selection.primary();
        let tx = Transaction::delete(doc.len_chars(), range.start(), range.end())
            .with_selection(Selection::point(range.start()));
        doc.apply(&tx, view_id);
    } else {
        // Delete one char forward
        let cursor = selection.cursor();
        if cursor < doc.len_chars() {
            let tx = Transaction::delete(doc.len_chars(), cursor, cursor + 1)
                .with_selection(Selection::point(cursor));
            doc.apply(&tx, view_id);
        }
    }
}

fn delete_line(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);
    let line_end = if line + 1 < doc.len_lines() {
        doc.rope.line_to_char(line + 1)
    } else {
        doc.len_chars()
    };

    if line_start < line_end {
        let tx = Transaction::delete(doc.len_chars(), line_start, line_end)
            .with_selection(Selection::point(line_start));
        doc.apply(&tx, view_id);
    }
}

fn duplicate_line(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);
    let line_text: String = doc.rope.line(line).chars().collect();

    // Insert at end of line
    let insert_pos = line_start + doc.rope.line_len_chars(line);
    let new_text = if line_text.ends_with('\n') {
        line_text
    } else {
        format!("{}\n", line_text.trim_end_matches('\n'))
    };

    let tx = Transaction::insert(doc.len_chars(), insert_pos, new_text);
    doc.apply(&tx, view_id);
}

fn move_line(editor: &mut Editor, direction: Direction) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);

    match direction {
        Direction::Up if line > 0 => {
            // Swap with line above
            let this_start = doc.rope.line_to_char(line);
            let this_end = doc.rope.line_to_char(line + 1);
            let prev_start = doc.rope.line_to_char(line - 1);

            let this_text: String = doc.rope.slice(this_start..this_end).chars().collect();
            let prev_text: String = doc.rope.slice(prev_start..this_start).chars().collect();

            let new_text = format!("{}{}", this_text, prev_text);

            let tx = Transaction::replace(doc.len_chars(), prev_start, this_end, new_text);
            doc.apply(&tx, view_id);

            // Move cursor up
            move_cursor(editor, Direction::Up, 1);
        }
        Direction::Down if line + 1 < doc.len_lines() => {
            // Swap with line below
            let this_start = doc.rope.line_to_char(line);
            let next_end = if line + 2 < doc.len_lines() {
                doc.rope.line_to_char(line + 2)
            } else {
                doc.len_chars()
            };
            let next_start = doc.rope.line_to_char(line + 1);

            let this_text: String = doc.rope.slice(this_start..next_start).chars().collect();
            let next_text: String = doc.rope.slice(next_start..next_end).chars().collect();

            let new_text = format!("{}{}", next_text, this_text);

            let tx = Transaction::replace(doc.len_chars(), this_start, next_end, new_text);
            doc.apply(&tx, view_id);

            // Move cursor down
            move_cursor(editor, Direction::Down, 1);
        }
        _ => {}
    }
}

fn indent(editor: &mut Editor) {
    let indent_str = if editor.config.editor.indent_style == lite_config::IndentStyle::Spaces {
        " ".repeat(editor.config.editor.tab_width)
    } else {
        "\t".to_string()
    };

    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);

    let tx = Transaction::insert(doc.len_chars(), line_start, indent_str);
    doc.apply(&tx, view_id);
}

fn unindent(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let tab_width = editor.config.editor.tab_width;

    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);
    let line_text: String = doc.rope.line(line).chars().collect();

    let remove_count = if line_text.starts_with('\t') {
        1
    } else {
        let spaces: usize = line_text.chars().take_while(|c| *c == ' ').count();
        spaces.min(tab_width)
    };

    if remove_count > 0 {
        let tx = Transaction::delete(doc.len_chars(), line_start, line_start + remove_count);
        doc.apply(&tx, view_id);
    }
}

fn toggle_comment(editor: &mut Editor) {
    // Simple // comment for now
    let comment_prefix = "// ";

    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);
    let line_text: String = doc.rope.line(line).chars().collect();
    let trimmed = line_text.trim_start();

    if trimmed.starts_with(comment_prefix) {
        // Remove comment
        let whitespace_len = line_text.len() - trimmed.len();
        let tx = Transaction::delete(
            doc.len_chars(),
            line_start + whitespace_len,
            line_start + whitespace_len + comment_prefix.len(),
        );
        doc.apply(&tx, view_id);
    } else {
        // Add comment
        let whitespace_len = line_text.len() - trimmed.len();
        let tx = Transaction::insert(
            doc.len_chars(),
            line_start + whitespace_len,
            comment_prefix,
        );
        doc.apply(&tx, view_id);
    }
}

fn select_all(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let end = doc.len_chars();
    doc.set_selection(view_id, Selection::single(Range::new(0, end)));
}

fn select_line(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let line = doc.rope.char_to_line(cursor);
    let line_start = doc.rope.line_to_char(line);
    let line_end = doc.rope.line_to_char(line) + doc.rope.line_len_chars(line);

    doc.set_selection(view_id, Selection::single(Range::new(line_start, line_end)));
}

fn select_word(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let cursor = selection.cursor();

    let (start, end) = doc.rope.word_at(cursor);
    doc.set_selection(view_id, Selection::single(Range::new(start, end)));
}

fn select_next_occurrence(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let mut selection = doc.selection(view_id);

    // Get the word under cursor or current selection
    let primary = selection.primary();
    let search_text: String = if primary.is_point() {
        let (start, end) = doc.rope.word_at(primary.head);
        doc.rope.slice(start..end).chars().collect()
    } else {
        doc.rope.slice(primary.start()..primary.end()).chars().collect()
    };

    if search_text.is_empty() {
        return;
    }

    // Find next occurrence after the primary selection
    let text: String = doc.rope.chars().collect();
    let search_start = primary.end();

    if let Some(pos) = text[search_start..].find(&search_text) {
        let abs_pos = search_start + pos;
        selection.add_range(Range::new(abs_pos, abs_pos + search_text.len()));
        doc.set_selection(view_id, selection);
    }
}

fn add_cursor(editor: &mut Editor, direction: Direction) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let mut selection = doc.selection(view_id);
    let primary = selection.primary();
    let pos = doc.rope.char_to_position(primary.head);

    let new_line = match direction {
        Direction::Up => pos.line.saturating_sub(1),
        Direction::Down => (pos.line + 1).min(doc.len_lines().saturating_sub(1)),
        _ => return,
    };

    if new_line != pos.line {
        let line_len = doc.rope.line_len_chars(new_line);
        let new_col = pos.col.min(line_len);
        let new_pos = lite_core::Position::new(new_line, new_col);
        let new_char = doc.rope.position_to_char(new_pos);
        selection.add_cursor(new_char);
        doc.set_selection(view_id, selection);
    }
}

fn clear_selection(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    doc.set_selection(view_id, selection.into_single().collapse());
}

fn copy(editor: &mut Editor) {
    let doc = editor.current_doc();
    let view_id = editor.tree.focus();
    let selection = doc.selection(view_id);
    let primary = selection.primary();

    if primary.is_point() {
        // Copy whole line
        let line = doc.rope.char_to_line(primary.head);
        let text: String = doc.rope.line(line).chars().collect();
        editor.clipboard = text;
    } else {
        let text: String = doc
            .rope
            .slice(primary.start()..primary.end())
            .chars()
            .collect();
        editor.clipboard = text;
    }

    editor.set_status("Copied", Severity::Info);
}

fn cut(editor: &mut Editor) {
    copy(editor);

    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let primary = selection.primary();

    if primary.is_point() {
        delete_line(editor);
    } else {
        let tx = Transaction::delete(doc.len_chars(), primary.start(), primary.end())
            .with_selection(Selection::point(primary.start()));
        doc.apply(&tx, view_id);
    }
}

fn paste(editor: &mut Editor) {
    if editor.clipboard.is_empty() {
        return;
    }

    let view_id = editor.tree.focus();
    let text = editor.clipboard.clone();
    let doc = editor.current_doc_mut();
    let selection = doc.selection(view_id);
    let primary = selection.primary();

    let (start, end) = if primary.is_point() {
        (primary.head, primary.head)
    } else {
        (primary.start(), primary.end())
    };

    let tx = Transaction::replace(doc.len_chars(), start, end, text.clone())
        .with_selection(Selection::point(start + text.len()));
    doc.apply(&tx, view_id);
}

fn undo(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    if !doc.undo(view_id) {
        editor.set_status("Nothing to undo", Severity::Info);
    }
}

fn redo(editor: &mut Editor) {
    let view_id = editor.tree.focus();
    let doc = editor.current_doc_mut();
    if !doc.redo(view_id) {
        editor.set_status("Nothing to redo", Severity::Info);
    }
}
