//! External Scanner Tests
//!
//! Tests for external scanner integration including Python-style indentation,
//! nested comments, and stateful scanning.

#![cfg(test)]

use rust_sitter::external_scanner::{ExternalScanner, Lexer as TSLexer};
use rust_sitter::unified_parser::Parser;
use std::sync::Arc;

/// Python-style indentation scanner
#[derive(Debug, Default)]
struct IndentationScanner {
    indent_stack: Vec<u32>,
}

impl ExternalScanner for IndentationScanner {
    fn scan(&mut self, lexer: &mut dyn TSLexer, valid_symbols: &[bool]) -> bool {
        const INDENT: usize = 0;
        const DEDENT: usize = 1;
        const NEWLINE: usize = 2;

        // Skip whitespace except newlines
        while lexer.lookahead() == ' ' || lexer.lookahead() == '\t' {
            lexer.advance(true);
        }

        if lexer.lookahead() == '\n' {
            if valid_symbols[NEWLINE] {
                lexer.advance(false);
                lexer.mark_end();
                return true;
            }

            lexer.advance(true);

            // Count indentation
            let mut indent = 0;
            while lexer.lookahead() == ' ' {
                indent += 1;
                lexer.advance(true);
            }

            let current_indent = *self.indent_stack.last().unwrap_or(&0);

            if indent > current_indent && valid_symbols[INDENT] {
                self.indent_stack.push(indent);
                lexer.mark_end();
                return true;
            }

            if indent < current_indent && valid_symbols[DEDENT] {
                while let Some(&level) = self.indent_stack.last() {
                    if level <= indent {
                        break;
                    }
                    self.indent_stack.pop();
                }
                lexer.mark_end();
                return true;
            }
        }

        false
    }

    fn serialize(&self, buffer: &mut Vec<u8>) -> usize {
        for &indent in &self.indent_stack {
            buffer.extend_from_slice(&indent.to_le_bytes());
        }
        self.indent_stack.len() * 4
    }

    fn deserialize(&mut self, buffer: &[u8]) -> usize {
        self.indent_stack.clear();
        let mut consumed = 0;

        while consumed + 4 <= buffer.len() {
            let bytes = [
                buffer[consumed],
                buffer[consumed + 1],
                buffer[consumed + 2],
                buffer[consumed + 3],
            ];
            self.indent_stack.push(u32::from_le_bytes(bytes));
            consumed += 4;
        }

        consumed
    }
}

/// Nested comment scanner (OCaml-style)
#[derive(Debug, Default)]
struct NestedCommentScanner {
    depth: u32,
}

impl ExternalScanner for NestedCommentScanner {
    fn scan(&mut self, lexer: &mut dyn TSLexer, valid_symbols: &[bool]) -> bool {
        const COMMENT: usize = 0;

        if !valid_symbols[COMMENT] {
            return false;
        }

        // Look for (* to start
        if self.depth == 0 {
            if lexer.lookahead() == '(' {
                lexer.advance(false);
                if lexer.lookahead() == '*' {
                    lexer.advance(false);
                    self.depth = 1;
                }
            }
        }

        // Scan until we find matching *)
        while self.depth > 0 {
            match lexer.lookahead() {
                '(' => {
                    lexer.advance(false);
                    if lexer.lookahead() == '*' {
                        lexer.advance(false);
                        self.depth += 1;
                    }
                }
                '*' => {
                    lexer.advance(false);
                    if lexer.lookahead() == ')' {
                        lexer.advance(false);
                        self.depth -= 1;
                        if self.depth == 0 {
                            lexer.mark_end();
                            return true;
                        }
                    }
                }
                '\0' => return false, // EOF
                _ => lexer.advance(false),
            }
        }

        false
    }

    fn serialize(&self, buffer: &mut Vec<u8>) -> usize {
        buffer.extend_from_slice(&self.depth.to_le_bytes());
        4
    }

    fn deserialize(&mut self, buffer: &[u8]) -> usize {
        if buffer.len() >= 4 {
            let bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
            self.depth = u32::from_le_bytes(bytes);
            4
        } else {
            0
        }
    }
}

#[test]
fn test_python_indentation_scanner() {
    let scanner = Arc::new(IndentationScanner::default());
    let mut parser = Parser::new();
    // TODO: Set language with external scanner
    // parser.set_external_scanner(scanner);

    let source = r#"
def foo():
    x = 1
    if x > 0:
        print("positive")
    else:
        print("non-positive")
    return x

def bar():
    pass
"#;

    let tree = parser
        .parse(source.as_bytes(), None)
        .expect("Failed to parse");

    // Verify INDENT tokens after colons
    // Verify DEDENT tokens at dedentation points
    // TODO: Add assertions once parser integration complete
}

#[test]
fn test_nested_comments() {
    let scanner = Arc::new(NestedCommentScanner::default());
    let mut parser = Parser::new();
    // TODO: Set language with external scanner

    let source = r#"
let x = 42 (* this is a (* nested *) comment *) in x + 1
"#;

    let tree = parser
        .parse(source.as_bytes(), None)
        .expect("Failed to parse");

    // Verify comment is parsed as single token
    // TODO: Add assertions
}

#[test]
fn test_scanner_state_persistence() {
    let mut scanner = IndentationScanner::default();
    scanner.indent_stack = vec![0, 4, 8];

    let mut buffer = Vec::new();
    let serialized_len = scanner.serialize(&mut buffer);
    assert_eq!(serialized_len, 12); // 3 * 4 bytes

    let mut scanner2 = IndentationScanner::default();
    let deserialized_len = scanner2.deserialize(&buffer);
    assert_eq!(deserialized_len, 12);
    assert_eq!(scanner2.indent_stack, vec![0, 4, 8]);
}

#[test]
fn test_lexer_adapter_advance() {
    // TODO: Test TSLexerAdapter implementation
    // - advance moves cursor correctly
    // - tracks row/column
    // - respects skip parameter
}

#[test]
fn test_lexer_adapter_lookahead() {
    // TODO: Test lookahead without advancing
}

#[test]
fn test_lexer_adapter_mark_end() {
    // TODO: Test marking token end position
}

#[test]
fn test_lexer_adapter_get_column() {
    // TODO: Test column tracking
    // - Handles tabs correctly
    // - CRLF line endings
}

#[test]
#[cfg(feature = "miri")]
fn test_scanner_memory_safety() {
    // Run under miri to check for UB
    let scanner = Arc::new(IndentationScanner::default());
    // TODO: Exercise scanner with various inputs
}

/// Test scanner with included ranges (e.g., JavaScript in HTML)
#[test]
#[cfg(feature = "included_ranges")]
fn test_scanner_with_ranges() {
    // TODO: Test is_at_included_range_start
    // See issue #3
}
