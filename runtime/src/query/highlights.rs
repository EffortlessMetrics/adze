// Syntax highlighting support using queries
use super::{Query, QueryCursor};
use crate::parser_v3::ParseNode;
use std::collections::HashMap;

/// Standard capture names for syntax highlighting
pub mod capture_names {
    pub const COMMENT: &str = "comment";
    pub const STRING: &str = "string";
    pub const NUMBER: &str = "number";
    pub const KEYWORD: &str = "keyword";
    pub const FUNCTION: &str = "function";
    pub const FUNCTION_CALL: &str = "function.call";
    pub const METHOD: &str = "method";
    pub const METHOD_CALL: &str = "method.call";
    pub const VARIABLE: &str = "variable";
    pub const VARIABLE_BUILTIN: &str = "variable.builtin";
    pub const CONSTANT: &str = "constant";
    pub const CONSTANT_BUILTIN: &str = "constant.builtin";
    pub const TYPE: &str = "type";
    pub const TYPE_BUILTIN: &str = "type.builtin";
    pub const PROPERTY: &str = "property";
    pub const OPERATOR: &str = "operator";
    pub const PUNCTUATION_BRACKET: &str = "punctuation.bracket";
    pub const PUNCTUATION_DELIMITER: &str = "punctuation.delimiter";
    pub const PUNCTUATION_SPECIAL: &str = "punctuation.special";
    pub const ATTRIBUTE: &str = "attribute";
    pub const NAMESPACE: &str = "namespace";
    pub const MODULE: &str = "module";
    pub const LABEL: &str = "label";
    pub const TAG: &str = "tag";
    pub const ERROR: &str = "error";
}

/// A highlighted range in the source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    /// Start byte offset
    pub start_byte: usize,
    /// End byte offset
    pub end_byte: usize,
    /// Highlight name (e.g., "keyword", "string", etc.)
    pub highlight: String,
}

/// Highlighter that uses Tree-sitter queries
pub struct Highlighter {
    /// The highlight query
    query: Query,
    /// Mapping from capture indices to highlight names
    highlight_map: HashMap<u32, String>,
}

impl Highlighter {
    /// Create a new highlighter from a query
    pub fn new(query: Query) -> Self {
        let mut highlight_map = HashMap::new();

        // Map capture names to highlight names
        // By convention, captures are named after the highlight they produce
        for (name, &index) in &query.capture_names {
            highlight_map.insert(index, name.clone());
        }

        Highlighter {
            query,
            highlight_map,
        }
    }

    /// Highlight a parse tree
    pub fn highlight(&self, root: &ParseNode) -> Vec<Highlight> {
        let mut highlights = Vec::new();
        let mut cursor = QueryCursor::new();

        // Execute query and collect matches
        let matches = cursor.collect_matches(&self.query, root);

        // Convert matches to highlights
        for query_match in matches {
            for capture in query_match.captures {
                if let Some(highlight_name) = self.highlight_map.get(&capture.index) {
                    highlights.push(Highlight {
                        start_byte: capture.node.start_byte,
                        end_byte: capture.node.end_byte,
                        highlight: highlight_name.clone(),
                    });
                }
            }
        }

        // Sort by start position
        highlights.sort_by_key(|h| (h.start_byte, h.end_byte));

        // Remove overlapping highlights (keep the more specific one)
        self.remove_overlaps(&mut highlights);

        highlights
    }

    /// Remove overlapping highlights, keeping the more specific ones
    fn remove_overlaps(&self, highlights: &mut Vec<Highlight>) {
        if highlights.is_empty() {
            return;
        }

        let mut result = Vec::new();
        let mut current = highlights[0].clone();

        for highlight in highlights.iter().skip(1) {
            if highlight.start_byte >= current.end_byte {
                // No overlap
                result.push(current);
                current = highlight.clone();
            } else if highlight.end_byte <= current.end_byte {
                // highlight is contained within current
                // Keep the more specific (smaller) highlight
                result.push(highlight.clone());

                // Split current if needed
                if highlight.start_byte > current.start_byte {
                    result.push(Highlight {
                        start_byte: current.start_byte,
                        end_byte: highlight.start_byte,
                        highlight: current.highlight.clone(),
                    });
                }
                if highlight.end_byte < current.end_byte {
                    current.start_byte = highlight.end_byte;
                } else {
                    current = highlights[highlights.len() - 1].clone(); // Use a dummy
                }
            } else {
                // Partial overlap - keep the first part of current
                if highlight.start_byte > current.start_byte {
                    result.push(Highlight {
                        start_byte: current.start_byte,
                        end_byte: highlight.start_byte,
                        highlight: current.highlight.clone(),
                    });
                }
                current = highlight.clone();
            }
        }

        if current.start_byte < current.end_byte {
            result.push(current);
        }

        *highlights = result;
    }
}

/// Theme colors for syntax highlighting
#[derive(Debug, Clone)]
pub struct Theme {
    /// Colors for different highlight types
    pub colors: HashMap<String, Color>,
    /// Default color for unhighlighted text
    pub default_color: Color,
    /// Background color
    pub background_color: Color,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl Theme {
    /// Create a default dark theme
    pub fn dark() -> Self {
        let mut colors = HashMap::new();

        // Dark theme colors
        colors.insert(capture_names::COMMENT.to_string(), Color::new(106, 153, 85));
        colors.insert(capture_names::STRING.to_string(), Color::new(206, 145, 120));
        colors.insert(capture_names::NUMBER.to_string(), Color::new(181, 206, 168));
        colors.insert(
            capture_names::KEYWORD.to_string(),
            Color::new(197, 134, 192),
        );
        colors.insert(
            capture_names::FUNCTION.to_string(),
            Color::new(220, 220, 170),
        );
        colors.insert(
            capture_names::VARIABLE.to_string(),
            Color::new(156, 220, 254),
        );
        colors.insert(
            capture_names::CONSTANT.to_string(),
            Color::new(79, 193, 255),
        );
        colors.insert(capture_names::TYPE.to_string(), Color::new(78, 201, 176));
        colors.insert(
            capture_names::OPERATOR.to_string(),
            Color::new(212, 212, 212),
        );
        colors.insert(
            capture_names::PUNCTUATION_BRACKET.to_string(),
            Color::new(212, 212, 212),
        );
        colors.insert(capture_names::ERROR.to_string(), Color::new(244, 71, 71));

        Theme {
            colors,
            default_color: Color::new(212, 212, 212),
            background_color: Color::new(30, 30, 30),
        }
    }

    /// Create a default light theme
    pub fn light() -> Self {
        let mut colors = HashMap::new();

        // Light theme colors
        colors.insert(capture_names::COMMENT.to_string(), Color::new(0, 128, 0));
        colors.insert(capture_names::STRING.to_string(), Color::new(163, 21, 21));
        colors.insert(capture_names::NUMBER.to_string(), Color::new(9, 134, 88));
        colors.insert(capture_names::KEYWORD.to_string(), Color::new(0, 0, 255));
        colors.insert(capture_names::FUNCTION.to_string(), Color::new(121, 94, 38));
        colors.insert(capture_names::VARIABLE.to_string(), Color::new(0, 16, 128));
        colors.insert(
            capture_names::CONSTANT.to_string(),
            Color::new(38, 127, 153),
        );
        colors.insert(capture_names::TYPE.to_string(), Color::new(38, 127, 153));
        colors.insert(capture_names::OPERATOR.to_string(), Color::new(0, 0, 0));
        colors.insert(
            capture_names::PUNCTUATION_BRACKET.to_string(),
            Color::new(0, 0, 0),
        );
        colors.insert(capture_names::ERROR.to_string(), Color::new(255, 0, 0));

        Theme {
            colors,
            default_color: Color::new(0, 0, 0),
            background_color: Color::new(255, 255, 255),
        }
    }

    /// Get color for a highlight type
    pub fn get_color(&self, highlight: &str) -> Color {
        self.colors
            .get(highlight)
            .copied()
            .unwrap_or(self.default_color)
    }
}

/// Example highlight queries for common languages
pub mod queries {
    pub const RUST_HIGHLIGHTS: &str = r#"
; Comments
(line_comment) @comment
(block_comment) @comment

; Strings
(string_literal) @string
(char_literal) @string

; Numbers
(integer_literal) @number
(float_literal) @number

; Keywords
[
  "as" "async" "await" "break" "const" "continue" "crate" "dyn"
  "else" "enum" "extern" "false" "fn" "for" "if" "impl" "in"
  "let" "loop" "match" "mod" "move" "mut" "pub" "ref" "return"
  "self" "Self" "static" "struct" "super" "trait" "true" "type"
  "unsafe" "use" "where" "while"
] @keyword

; Functions
(function_item name: (identifier) @function)
(call_expression function: (identifier) @function.call)

; Types
(type_identifier) @type
(primitive_type) @type.builtin

; Variables
(identifier) @variable

; Operators
[
  "+" "-" "*" "/" "%" "^" "!" "&" "|" "&&" "||"
  "<<" ">>" "==" "!=" "<" "<=" ">" ">="
  "=" "+=" "-=" "*=" "/=" "%=" "^=" "&=" "|="
  "<<=" ">>=" "?" "=>" "->" "::" ".." "..="
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["." "," ":" ";"] @punctuation.delimiter
"#;

    pub const PYTHON_HIGHLIGHTS: &str = r#"
; Comments
(comment) @comment

; Strings
(string) @string

; Numbers
(integer) @number
(float) @number

; Keywords
[
  "and" "as" "assert" "async" "await" "break" "class" "continue"
  "def" "del" "elif" "else" "except" "finally" "for" "from"
  "global" "if" "import" "in" "is" "lambda" "nonlocal" "not"
  "or" "pass" "raise" "return" "try" "while" "with" "yield"
] @keyword

; Functions
(function_definition name: (identifier) @function)
(call function: (identifier) @function.call)

; Constants
(true) @constant.builtin
(false) @constant.builtin
(none) @constant.builtin

; Variables
(identifier) @variable
"#;
}
