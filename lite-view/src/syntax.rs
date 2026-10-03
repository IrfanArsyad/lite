use once_cell::sync::Lazy;
use ropey::Rope;
use std::collections::HashMap;
use std::ops::Range;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Node, Parser, Point, Query, QueryCursor, Tree};

/// Highlight category for theming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Highlight {
    Keyword,
    Function,
    Type,
    Variable,
    Constant,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Property,
    Parameter,
    Label,
    Namespace,
    Attribute,
}

impl Highlight {
    /// Convert capture name to highlight category
    pub fn from_capture(name: &str) -> Option<Self> {
        Some(match name {
            "keyword" | "keyword.control" | "keyword.function" | "keyword.operator"
            | "keyword.return" | "keyword.import" | "keyword.export" | "keyword.modifier"
            | "keyword.type" | "keyword.storage" | "keyword.coroutine" => Highlight::Keyword,

            "function" | "function.call" | "function.method" | "function.builtin"
            | "function.macro" | "method" | "method.call" => Highlight::Function,

            "type" | "type.builtin" | "type.parameter" | "type.qualifier" | "constructor" => {
                Highlight::Type
            }

            "variable" | "variable.builtin" | "variable.parameter" | "variable.member" => {
                Highlight::Variable
            }

            "constant" | "constant.builtin" | "boolean" => Highlight::Constant,

            "string" | "string.special" | "string.escape" | "character" => Highlight::String,

            "number" | "float" => Highlight::Number,

            "comment" | "comment.line" | "comment.block" | "comment.documentation" => {
                Highlight::Comment
            }

            "operator" => Highlight::Operator,

            "punctuation"
            | "punctuation.bracket"
            | "punctuation.delimiter"
            | "punctuation.special" => Highlight::Punctuation,

            "property" | "field" => Highlight::Property,

            "parameter" => Highlight::Parameter,

            "label" => Highlight::Label,

            "namespace" | "module" => Highlight::Namespace,

            "attribute" | "tag" | "tag.attribute" => Highlight::Attribute,

            _ => return None,
        })
    }
}

/// A highlighted span in the document
#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize, // byte offset
    pub end: usize,   // byte offset
    pub highlight: Highlight,
}

/// Language configuration with parser and queries
struct LanguageConfig {
    language: Language,
    highlight_query: Query,
}

/// Global highlighter instance
static HIGHLIGHTER: Lazy<Highlighter> = Lazy::new(Highlighter::new);

/// Get the global highlighter
pub fn highlighter() -> &'static Highlighter {
    &HIGHLIGHTER
}

/// Language name, grammar and highlight query for every bundled language.
fn bundled_languages() -> Vec<(&'static str, Language, &'static str)> {
    vec![
        (
            "rust",
            tree_sitter_rust::LANGUAGE.into(),
            include_str!("queries/rust.scm"),
        ),
        (
            "python",
            tree_sitter_python::LANGUAGE.into(),
            include_str!("queries/python.scm"),
        ),
        (
            "javascript",
            tree_sitter_javascript::LANGUAGE.into(),
            include_str!("queries/javascript.scm"),
        ),
        (
            "typescript",
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            include_str!("queries/typescript.scm"),
        ),
        (
            "go",
            tree_sitter_go::LANGUAGE.into(),
            include_str!("queries/go.scm"),
        ),
        (
            "c",
            tree_sitter_c::LANGUAGE.into(),
            include_str!("queries/c.scm"),
        ),
        (
            "cpp",
            tree_sitter_cpp::LANGUAGE.into(),
            include_str!("queries/cpp.scm"),
        ),
        (
            "json",
            tree_sitter_json::LANGUAGE.into(),
            include_str!("queries/json.scm"),
        ),
        (
            "bash",
            tree_sitter_bash::LANGUAGE.into(),
            include_str!("queries/bash.scm"),
        ),
        (
            "html",
            tree_sitter_html::LANGUAGE.into(),
            include_str!("queries/html.scm"),
        ),
        (
            "css",
            tree_sitter_css::LANGUAGE.into(),
            include_str!("queries/css.scm"),
        ),
        (
            "markdown",
            tree_sitter_md::LANGUAGE.into(),
            include_str!("queries/markdown.scm"),
        ),
    ]
}

/// Syntax highlighter using tree-sitter
pub struct Highlighter {
    languages: HashMap<&'static str, LanguageConfig>,
    /// Languages whose highlight query failed to compile, with the error.
    query_errors: Vec<(&'static str, String)>,
}

impl Highlighter {
    fn new() -> Self {
        let mut languages = HashMap::new();
        let mut query_errors = Vec::new();

        for (name, language, query_source) in bundled_languages() {
            match Query::new(&language, query_source) {
                Ok(highlight_query) => {
                    languages.insert(
                        name,
                        LanguageConfig {
                            language,
                            highlight_query,
                        },
                    );
                }
                Err(err) => query_errors.push((name, err.to_string())),
            }
        }

        Self {
            languages,
            query_errors,
        }
    }

    /// Check if a language is supported
    pub fn supports(&self, language: &str) -> bool {
        self.languages.contains_key(language)
    }

    /// Languages that were skipped because their highlight query is invalid
    pub fn query_errors(&self) -> &[(&'static str, String)] {
        &self.query_errors
    }

    /// Parse `rope`, reusing `old_tree` for incremental parsing when it has
    /// been updated with `Tree::edit` for every change since it was produced.
    pub fn parse(&self, language: &str, rope: &Rope, old_tree: Option<&Tree>) -> Option<Tree> {
        let config = self.languages.get(language)?;
        let mut parser = Parser::new();
        parser.set_language(&config.language).ok()?;

        parser.parse_with(
            &mut |byte_idx, _| {
                if byte_idx >= rope.len_bytes() {
                    return &[] as &[u8];
                }
                let (chunk, chunk_start, _, _) = rope.chunk_at_byte(byte_idx);
                &chunk.as_bytes()[byte_idx - chunk_start..]
            },
            old_tree,
        )
    }

    /// Highlight spans of `tree` that intersect `byte_range`, sorted by start.
    pub fn highlight_range(
        &self,
        language: &str,
        tree: &Tree,
        rope: &Rope,
        byte_range: Range<usize>,
    ) -> Vec<HighlightSpan> {
        let Some(config) = self.languages.get(language) else {
            return Vec::new();
        };

        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(byte_range);
        let text_provider = |node: Node| {
            rope.byte_slice(node.byte_range())
                .chunks()
                .map(str::as_bytes)
        };

        let capture_names = config.highlight_query.capture_names();
        let mut spans = Vec::new();
        let mut matches = cursor.matches(&config.highlight_query, tree.root_node(), text_provider);

        while let Some(query_match) = matches.next() {
            for capture in query_match.captures {
                if let Some(highlight) =
                    Highlight::from_capture(capture_names[capture.index as usize])
                {
                    spans.push(HighlightSpan {
                        start: capture.node.start_byte(),
                        end: capture.node.end_byte(),
                        highlight,
                    });
                }
            }
        }

        spans.sort_by_key(|s| s.start);
        spans
    }

    /// Parse and highlight a whole source string.
    pub fn highlight(&self, language: &str, source: &str) -> Vec<HighlightSpan> {
        let rope = Rope::from_str(source);
        match self.parse(language, &rope, None) {
            Some(tree) => self.highlight_range(language, &tree, &rope, 0..rope.len_bytes()),
            None => Vec::new(),
        }
    }
}

/// Convert a byte offset into a tree-sitter row/column point.
pub fn byte_to_point(rope: &Rope, byte_idx: usize) -> Point {
    let byte_idx = byte_idx.min(rope.len_bytes());
    let row = rope.byte_to_line(byte_idx);
    Point::new(row, byte_idx - rope.line_to_byte(row))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_bundled_queries_compile() {
        let errors = highlighter().query_errors();
        assert!(errors.is_empty(), "invalid highlight queries: {errors:?}");
    }

    #[test]
    fn test_highlight_range_only_returns_visible_spans() {
        let source = "fn a() {}\nfn b() {}\n";
        let rope = Rope::from_str(source);
        let tree = highlighter().parse("rust", &rope, None).unwrap();
        let second_line = rope.line_to_byte(1)..rope.len_bytes();
        let spans = highlighter().highlight_range("rust", &tree, &rope, second_line.clone());
        assert!(!spans.is_empty());
        assert!(spans.iter().all(|s| s.end > second_line.start));
    }
}
