use once_cell::sync::Lazy;
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

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

            "punctuation" | "punctuation.bracket" | "punctuation.delimiter"
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

/// Syntax highlighter using tree-sitter
pub struct Highlighter {
    languages: HashMap<&'static str, LanguageConfig>,
}

impl Highlighter {
    fn new() -> Self {
        let mut languages = HashMap::new();

        // Rust
        if let Ok(config) = Self::create_config(
            tree_sitter_rust::LANGUAGE.into(),
            include_str!("queries/rust.scm"),
        ) {
            languages.insert("rust", config);
        }

        // Python
        if let Ok(config) = Self::create_config(
            tree_sitter_python::LANGUAGE.into(),
            include_str!("queries/python.scm"),
        ) {
            languages.insert("python", config);
        }

        // JavaScript
        if let Ok(config) = Self::create_config(
            tree_sitter_javascript::LANGUAGE.into(),
            include_str!("queries/javascript.scm"),
        ) {
            languages.insert("javascript", config);
        }

        // TypeScript
        if let Ok(config) = Self::create_config(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            include_str!("queries/typescript.scm"),
        ) {
            languages.insert("typescript", config);
        }

        // Go
        if let Ok(config) = Self::create_config(
            tree_sitter_go::LANGUAGE.into(),
            include_str!("queries/go.scm"),
        ) {
            languages.insert("go", config);
        }

        // C
        if let Ok(config) = Self::create_config(
            tree_sitter_c::LANGUAGE.into(),
            include_str!("queries/c.scm"),
        ) {
            languages.insert("c", config);
        }

        // C++
        if let Ok(config) = Self::create_config(
            tree_sitter_cpp::LANGUAGE.into(),
            include_str!("queries/cpp.scm"),
        ) {
            languages.insert("cpp", config);
        }

        // JSON
        if let Ok(config) = Self::create_config(
            tree_sitter_json::LANGUAGE.into(),
            include_str!("queries/json.scm"),
        ) {
            languages.insert("json", config);
        }

        // Bash
        if let Ok(config) = Self::create_config(
            tree_sitter_bash::LANGUAGE.into(),
            include_str!("queries/bash.scm"),
        ) {
            languages.insert("bash", config);
        }

        // HTML
        if let Ok(config) = Self::create_config(
            tree_sitter_html::LANGUAGE.into(),
            include_str!("queries/html.scm"),
        ) {
            languages.insert("html", config);
        }

        // CSS
        if let Ok(config) = Self::create_config(
            tree_sitter_css::LANGUAGE.into(),
            include_str!("queries/css.scm"),
        ) {
            languages.insert("css", config);
        }

        // Markdown
        if let Ok(config) = Self::create_config(
            tree_sitter_md::LANGUAGE.into(),
            include_str!("queries/markdown.scm"),
        ) {
            languages.insert("markdown", config);
        }

        Self { languages }
    }

    fn create_config(language: Language, query_str: &str) -> Result<LanguageConfig, tree_sitter::QueryError> {
        let highlight_query = Query::new(&language, query_str)?;
        Ok(LanguageConfig {
            language,
            highlight_query,
        })
    }

    /// Check if a language is supported
    pub fn supports(&self, language: &str) -> bool {
        self.languages.contains_key(language)
    }

    /// Highlight a document and return spans
    pub fn highlight(&self, language: &str, source: &str) -> Vec<HighlightSpan> {
        let Some(config) = self.languages.get(language) else {
            return Vec::new();
        };

        let mut parser = Parser::new();
        if parser.set_language(&config.language).is_err() {
            return Vec::new();
        }

        let Some(tree) = parser.parse(source, None) else {
            return Vec::new();
        };

        let mut cursor = QueryCursor::new();
        let mut spans = Vec::new();

        let mut matches = cursor.matches(&config.highlight_query, tree.root_node(), source.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let capture_name = &config.highlight_query.capture_names()[capture.index as usize];

                if let Some(highlight) = Highlight::from_capture(capture_name) {
                    let node = capture.node;
                    spans.push(HighlightSpan {
                        start: node.start_byte(),
                        end: node.end_byte(),
                        highlight,
                    });
                }
            }
        }

        // Sort by start position
        spans.sort_by_key(|s| s.start);

        spans
    }

    /// Highlight a specific line range (for incremental rendering)
    pub fn highlight_line(
        &self,
        language: &str,
        source: &str,
        line_start_byte: usize,
        line_end_byte: usize,
    ) -> Vec<HighlightSpan> {
        self.highlight(language, source)
            .into_iter()
            .filter(|span| span.end > line_start_byte && span.start < line_end_byte)
            .collect()
    }
}
