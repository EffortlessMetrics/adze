//! External scanner runtime for Adze.
//! This module provides the runtime support for custom lexing logic.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// External scanner runtime for Adze
// This module provides the runtime support for custom lexing logic

#[cfg(feature = "external_scanners")]
pub mod adapter;

#[cfg(feature = "external_scanners")]
pub mod lifecycle;

use crate::SymbolId;
use std::collections::HashSet;

/// Result of external scanning
#[derive(Debug, Clone, PartialEq)]
pub struct ScanResult {
    pub symbol: u16,
    pub length: usize,
}

/// External scanner state
#[derive(Debug, Clone)]
pub struct ExternalScannerState {
    /// Current state data (serialized)
    pub data: Vec<u8>,
}

impl Default for ExternalScannerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalScannerState {
    pub fn new() -> Self {
        ExternalScannerState { data: Vec::new() }
    }

    /// Serialize the state
    pub fn serialize(&self) -> &[u8] {
        &self.data
    }

    /// Deserialize from bytes
    pub fn deserialize(data: &[u8]) -> Self {
        ExternalScannerState {
            data: data.to_vec(),
        }
    }
}

/// Trait for external scanner lexing interaction
pub trait Lexer {
    /// Get the next byte at the current position
    fn lookahead(&self) -> Option<u8>;

    /// Advance the lexer by n bytes
    fn advance(&mut self, n: usize);

    /// Mark the end of the current token
    fn mark_end(&mut self);

    /// Get the current column position
    fn column(&self) -> usize;

    /// Check if at end of file
    fn is_eof(&self) -> bool;
}

/// Trait for implementing external scanners (object-safe)
pub trait ExternalScanner: Send + Sync {
    /// Scan for external tokens
    fn scan(&mut self, lexer: &mut dyn Lexer, valid_symbols: &[bool]) -> Option<ScanResult>;

    /// Serialize scanner state
    fn serialize(&self, buffer: &mut Vec<u8>);

    /// Deserialize scanner state
    fn deserialize(&mut self, buffer: &[u8]);
}

/// Type alias for dynamic external scanner
pub type DynExternalScanner = dyn ExternalScanner + Send + Sync;

/// Runtime for executing external scanners
pub struct ExternalScannerRuntime {
    /// Map of external token IDs to their valid symbols
    external_tokens: Vec<SymbolId>,
    /// Scanner state
    state: ExternalScannerState,
}

impl ExternalScannerRuntime {
    pub fn new(external_tokens: Vec<SymbolId>) -> Self {
        ExternalScannerRuntime {
            external_tokens,
            state: ExternalScannerState::new(),
        }
    }

    /// Get the external tokens
    pub fn get_external_tokens(&self) -> &[SymbolId] {
        &self.external_tokens
    }

    /// Reset the scanner state
    ///
    /// This clears any accumulated state and prepares the scanner for a fresh parse
    pub fn reset(&mut self) {
        self.state = ExternalScannerState::new();
    }

    /// Execute external scanner
    pub fn scan(
        &mut self,
        scanner: &mut DynExternalScanner,
        lexer: &mut dyn Lexer,
        valid_external_tokens: &HashSet<SymbolId>,
    ) -> Option<(SymbolId, usize)> {
        // Build valid symbols array
        let valid_symbols: Vec<bool> = self
            .external_tokens
            .iter()
            .map(|token| valid_external_tokens.contains(token))
            .collect();

        // Deserialize scanner state
        scanner.deserialize(&self.state.data);

        // Scan for external tokens
        if let Some(result) = scanner.scan(lexer, &valid_symbols) {
            // Enforce Tree-sitter contract: external scanners may only emit symbols
            // that are valid in the current parser state.
            let emitted_index = usize::from(result.symbol);
            let emitted_by_index =
                emitted_index < valid_symbols.len() && valid_symbols[emitted_index];
            let emitted_by_symbol_id = self
                .external_tokens
                .iter()
                .enumerate()
                .find_map(|(idx, token)| (*token == result.symbol).then_some(idx))
                .is_some_and(|idx| valid_symbols.get(idx) == Some(&true));

            if !emitted_by_index && !emitted_by_symbol_id {
                return None;
            }

            // Serialize updated state
            self.state.data.clear();
            scanner.serialize(&mut self.state.data);

            return Some((result.symbol, result.length));
        }

        None
    }
}

/// Example external scanner for string literals with escape sequences
#[derive(Default)]
pub struct StringScanner {
    /// Whether we're inside a string
    in_string: bool,
    /// The quote character used
    quote_char: Option<u8>,
}

impl StringScanner {
    pub fn new() -> Self {
        StringScanner {
            in_string: false,
            quote_char: None,
        }
    }
}

impl ExternalScanner for StringScanner {
    fn scan(&mut self, lexer: &mut dyn Lexer, valid_symbols: &[bool]) -> Option<ScanResult> {
        // Check for string start/content/end tokens
        const STRING_START: usize = 0;
        const STRING_CONTENT: usize = 1;
        const STRING_END: usize = 2;

        if lexer.is_eof() {
            return None;
        }

        let current = lexer.lookahead()?;

        if !self.in_string {
            // Look for string start
            if valid_symbols.get(STRING_START) == Some(&true)
                && (current == b'"' || current == b'\'')
            {
                self.in_string = true;
                self.quote_char = Some(current);
                return Some(ScanResult {
                    symbol: STRING_START as u16,
                    length: 1,
                });
            }
        } else {
            // Inside string - look for content or end
            if let Some(quote) = self.quote_char {
                if current == quote {
                    // String end
                    if valid_symbols.get(STRING_END) == Some(&true) {
                        self.in_string = false;
                        self.quote_char = None;
                        return Some(ScanResult {
                            symbol: STRING_END as u16,
                            length: 1,
                        });
                    }
                } else if valid_symbols.get(STRING_CONTENT) == Some(&true) {
                    // String content - scan until quote or escape
                    let mut length = 0;

                    while !lexer.is_eof() {
                        if let Some(ch) = lexer.lookahead() {
                            if ch == quote {
                                break;
                            }
                            lexer.advance(1);
                            length += 1;
                            if ch == b'\\' && !lexer.is_eof() {
                                // Skip escape sequence
                                lexer.advance(1);
                                length += 1;
                            }
                        } else {
                            break;
                        }
                    }

                    if length > 0 {
                        return Some(ScanResult {
                            symbol: STRING_CONTENT as u16,
                            length,
                        });
                    }
                }
            }
        }

        None
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.push(if self.in_string { 1 } else { 0 });
        buffer.push(self.quote_char.unwrap_or(0));
    }

    fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.len() >= 2 {
            self.in_string = buffer[0] != 0;
            self.quote_char = if buffer[1] != 0 {
                Some(buffer[1])
            } else {
                None
            };
        }
    }
}

/// External scanner for multi-line comments
pub struct CommentScanner {
    /// Nesting depth for nested comments
    depth: u32,
}

impl Default for CommentScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommentScanner {
    pub fn new() -> Self {
        CommentScanner { depth: 0 }
    }
}

impl ExternalScanner for CommentScanner {
    fn scan(&mut self, lexer: &mut dyn Lexer, valid_symbols: &[bool]) -> Option<ScanResult> {
        const COMMENT_START: usize = 0;
        const COMMENT_CONTENT: usize = 1;
        const COMMENT_END: usize = 2;

        if lexer.is_eof() {
            return None;
        }

        let current = lexer.lookahead()?;
        lexer.advance(1);
        let next = lexer.lookahead().unwrap_or(0);
        // Move back to original position
        lexer.advance(usize::MAX); // This is a hack - we need a better API

        if self.depth == 0 {
            // Look for comment start
            if valid_symbols.get(COMMENT_START) == Some(&true) && current == b'/' && next == b'*' {
                self.depth = 1;
                return Some(ScanResult {
                    symbol: COMMENT_START as u16,
                    length: 2,
                });
            }
        } else {
            // Inside comment
            if current == b'/' && next == b'*' {
                // Nested comment start
                self.depth += 1;
                if valid_symbols.get(COMMENT_CONTENT) == Some(&true) {
                    return Some(ScanResult {
                        symbol: COMMENT_CONTENT as u16,
                        length: 2,
                    });
                }
            } else if current == b'*' && next == b'/' {
                // Comment end
                self.depth -= 1;
                if self.depth == 0 && valid_symbols.get(COMMENT_END) == Some(&true) {
                    return Some(ScanResult {
                        symbol: COMMENT_END as u16,
                        length: 2,
                    });
                } else if valid_symbols.get(COMMENT_CONTENT) == Some(&true) {
                    return Some(ScanResult {
                        symbol: COMMENT_CONTENT as u16,
                        length: 2,
                    });
                }
            } else if valid_symbols.get(COMMENT_CONTENT) == Some(&true) {
                // Regular content
                let mut length = 0;

                while !lexer.is_eof() {
                    let ch = lexer.lookahead().unwrap_or(0);
                    lexer.advance(1);
                    if !lexer.is_eof() {
                        let next_ch = lexer.lookahead().unwrap_or(0);
                        if (ch == b'/' && next_ch == b'*') || (ch == b'*' && next_ch == b'/') {
                            // Move back one position
                            break;
                        }
                    }
                    length += 1;
                }

                if length > 0 {
                    return Some(ScanResult {
                        symbol: COMMENT_CONTENT as u16,
                        length,
                    });
                }
            }
        }

        None
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.depth.to_le_bytes());
    }

    fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.len() >= 4 {
            self.depth = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        }
    }
}

// Tests temporarily disabled during refactoring
// TODO: Re-enable with proper Lexer implementation
#[cfg(test)]
mod tests {
    #![allow(dead_code, unused_imports)]
    use super::*;

    #[test]
    fn test_string_scanner() {
        let mut scanner = StringScanner::new();

        // Test string start
        let input = b"\"hello world\"";
        let valid = vec![true, true, true]; // All tokens valid

        // Create a test lexer
        struct TestLexer<'a> {
            input: &'a [u8],
            position: usize,
        }

        impl<'a> Lexer for TestLexer<'a> {
            fn advance(&mut self, n: usize) {
                self.position = (self.position + n).min(self.input.len());
            }

            fn lookahead(&self) -> Option<u8> {
                if self.position < self.input.len() {
                    Some(self.input[self.position])
                } else {
                    None
                }
            }

            fn mark_end(&mut self) {}

            fn column(&self) -> usize {
                self.position
            }

            fn is_eof(&self) -> bool {
                self.position >= self.input.len()
            }
        }

        let mut lexer = TestLexer { input, position: 0 };
        let result = scanner.scan(&mut lexer, &valid);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: 0, // STRING_START
                length: 1,
            })
        );

        // Test string content
        let mut lexer = TestLexer { input, position: 1 };
        let result = scanner.scan(&mut lexer, &valid);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: 1,  // STRING_CONTENT
                length: 11, // "hello world"
            })
        );

        // Test string end
        let mut lexer = TestLexer {
            input,
            position: 12,
        };
        let result = scanner.scan(&mut lexer, &valid);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: 2, // STRING_END
                length: 1,
            })
        );
    }

    #[test]
    fn test_string_scanner_escapes() {
        let mut scanner = StringScanner::new();
        scanner.in_string = true;
        scanner.quote_char = Some(b'"');

        let input = b"hello\\\"world";
        let valid = vec![false, true, false];

        // Create test lexer
        struct TestLexer {
            input: &'static [u8],
            position: usize,
        }

        impl Lexer for TestLexer {
            fn advance(&mut self, n: usize) {
                self.position = self.position.saturating_add(n);
            }

            fn lookahead(&self) -> Option<u8> {
                if self.position < self.input.len() {
                    Some(self.input[self.position])
                } else {
                    None
                }
            }

            fn mark_end(&mut self) {}

            fn column(&self) -> usize {
                self.position
            }

            fn is_eof(&self) -> bool {
                self.position >= self.input.len()
            }
        }

        let mut lexer = TestLexer { input, position: 0 };
        let result = scanner.scan(&mut lexer, &valid);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: 1,  // STRING_CONTENT
                length: 12, // includes escape sequence
            })
        );
    }

    #[test]
    fn test_comment_scanner() {
        let mut scanner = CommentScanner::new();

        // Test comment start
        let input = b"/* hello /* nested */ world */";
        let valid = vec![true, true, true];

        // Create test lexer
        struct TestLexer {
            input: &'static [u8],
            position: usize,
        }

        impl Lexer for TestLexer {
            fn advance(&mut self, n: usize) {
                self.position = self.position.saturating_add(n);
            }

            fn lookahead(&self) -> Option<u8> {
                if self.position < self.input.len() {
                    Some(self.input[self.position])
                } else {
                    None
                }
            }

            fn mark_end(&mut self) {}

            fn column(&self) -> usize {
                self.position
            }

            fn is_eof(&self) -> bool {
                self.position >= self.input.len()
            }
        }

        let mut lexer = TestLexer { input, position: 0 };
        let result = scanner.scan(&mut lexer, &valid);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: 0, // COMMENT_START
                length: 2,
            })
        );
        assert_eq!(scanner.depth, 1);

        // Test nested comment
        let mut lexer = TestLexer { input, position: 9 };
        let result = scanner.scan(&mut lexer, &valid);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: 1, // COMMENT_CONTENT
                length: 2,
            })
        );
        assert_eq!(scanner.depth, 2);
    }

    #[test]
    fn test_scanner_state_serialization() {
        let mut scanner = StringScanner::new();
        scanner.in_string = true;
        scanner.quote_char = Some(b'\'');

        // Serialize
        let mut buffer = Vec::new();
        scanner.serialize(&mut buffer);

        // Deserialize into new scanner
        let mut new_scanner = StringScanner::new();
        new_scanner.deserialize(&buffer);

        assert!(new_scanner.in_string);
        assert_eq!(new_scanner.quote_char, Some(b'\''));
    }

    #[test]
    fn test_runtime_rejects_emitted_symbol_not_valid_for_state() {
        #[derive(Default)]
        struct InvalidScanner;

        impl ExternalScanner for InvalidScanner {
            fn scan(
                &mut self,
                _lexer: &mut dyn Lexer,
                _valid_symbols: &[bool],
            ) -> Option<ScanResult> {
                // Emit symbol index 1, even if caller only allows index 0.
                Some(ScanResult {
                    symbol: 1,
                    length: 0,
                })
            }

            fn serialize(&self, _buffer: &mut Vec<u8>) {}

            fn deserialize(&mut self, _buffer: &[u8]) {}
        }

        struct EmptyLexer;
        impl Lexer for EmptyLexer {
            fn lookahead(&self) -> Option<u8> {
                None
            }
            fn advance(&mut self, _n: usize) {}
            fn mark_end(&mut self) {}
            fn column(&self) -> usize {
                0
            }
            fn is_eof(&self) -> bool {
                true
            }
        }

        let mut runtime = ExternalScannerRuntime::new(vec![SymbolId(0), SymbolId(1)]);
        let mut scanner = InvalidScanner;
        let mut lexer = EmptyLexer;
        let valid_external_tokens = std::iter::once(SymbolId(0)).collect();

        let scanned = runtime.scan(&mut scanner, &mut lexer, &valid_external_tokens);
        assert!(
            scanned.is_none(),
            "scanner emission must be filtered by valid_symbols",
        );
    }
}
