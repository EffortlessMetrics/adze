// External scanner runtime for the pure-Rust Tree-sitter implementation
// This module provides the runtime support for custom lexing logic

use rust_sitter_ir::SymbolId;
use std::collections::HashSet;

/// Result of external scanning
#[derive(Debug, Clone, PartialEq)]
pub struct ScanResult {
    pub symbol: SymbolId,
    pub length: usize,
}

/// External scanner state
#[derive(Debug, Clone)]
pub struct ExternalScannerState {
    /// Current state data (serialized)
    pub data: Vec<u8>,
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

/// Trait for implementing external scanners
pub trait ExternalScanner: Send + Sync {
    /// Create a new instance
    fn new() -> Self where Self: Sized;

    /// Scan for external tokens
    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult>;

    /// Serialize scanner state
    fn serialize(&self, buffer: &mut Vec<u8>);

    /// Deserialize scanner state
    fn deserialize(&mut self, buffer: &[u8]);

    /// Get state size hint
    fn state_size(&self) -> usize {
        16 // Default size
    }
}

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

    /// Execute external scanner
    pub fn scan<S: ExternalScanner>(
        &mut self,
        scanner: &mut S,
        valid_external_tokens: &HashSet<SymbolId>,
        input: &[u8],
        position: usize,
    ) -> Option<(SymbolId, usize)> {
        // Build valid symbols array
        let valid_symbols: Vec<bool> = self.external_tokens
            .iter()
            .map(|token| valid_external_tokens.contains(token))
            .collect();

        // Deserialize scanner state
        scanner.deserialize(&self.state.data);

        // Scan for external tokens
        if let Some(result) = scanner.scan(&valid_symbols, input, position) {
            // Serialize updated state
            self.state.data.clear();
            scanner.serialize(&mut self.state.data);

            return Some((result.symbol, result.length));
        }

        None
    }
}

/// Example external scanner for string literals with escape sequences
pub struct StringScanner {
    /// Whether we're inside a string
    in_string: bool,
    /// The quote character used
    quote_char: Option<u8>,
}

impl ExternalScanner for StringScanner {
    fn new() -> Self {
        StringScanner {
            in_string: false,
            quote_char: None,
        }
    }

    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
        // Check for string start/content/end tokens
        const STRING_START: usize = 0;
        const STRING_CONTENT: usize = 1;
        const STRING_END: usize = 2;

        if position >= input.len() {
            return None;
        }

        let current = input[position];

        if !self.in_string {
            // Look for string start
            if valid_symbols.get(STRING_START) == Some(&true) {
                if current == b'"' || current == b'\'' {
                    self.in_string = true;
                    self.quote_char = Some(current);
                    return Some(ScanResult {
                        symbol: SymbolId(STRING_START as u16),
                        length: 1,
                    });
                }
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
                            symbol: SymbolId(STRING_END as u16),
                            length: 1,
                        });
                    }
                } else if valid_symbols.get(STRING_CONTENT) == Some(&true) {
                    // String content - scan until quote or escape
                    let mut length = 0;
                    let mut i = position;

                    while i < input.len() {
                        let ch = input[i];
                        if ch == quote {
                            break;
                        } else if ch == b'\\' && i + 1 < input.len() {
                            // Skip escape sequence
                            i += 2;
                            length += 2;
                        } else {
                            i += 1;
                            length += 1;
                        }
                    }

                    if length > 0 {
                        return Some(ScanResult {
                            symbol: SymbolId(STRING_CONTENT as u16),
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

impl ExternalScanner for CommentScanner {
    fn new() -> Self {
        CommentScanner { depth: 0 }
    }

    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
        const COMMENT_START: usize = 0;
        const COMMENT_CONTENT: usize = 1;
        const COMMENT_END: usize = 2;

        if position + 1 >= input.len() {
            return None;
        }

        let current = input[position];
        let next = input[position + 1];

        if self.depth == 0 {
            // Look for comment start
            if valid_symbols.get(COMMENT_START) == Some(&true) {
                if current == b'/' && next == b'*' {
                    self.depth = 1;
                    return Some(ScanResult {
                        symbol: SymbolId(COMMENT_START as u16),
                        length: 2,
                    });
                }
            }
        } else {
            // Inside comment
            if current == b'/' && next == b'*' {
                // Nested comment start
                self.depth += 1;
                if valid_symbols.get(COMMENT_CONTENT) == Some(&true) {
                    return Some(ScanResult {
                        symbol: SymbolId(COMMENT_CONTENT as u16),
                        length: 2,
                    });
                }
            } else if current == b'*' && next == b'/' {
                // Comment end
                self.depth -= 1;
                if self.depth == 0 && valid_symbols.get(COMMENT_END) == Some(&true) {
                    return Some(ScanResult {
                        symbol: SymbolId(COMMENT_END as u16),
                        length: 2,
                    });
                } else if valid_symbols.get(COMMENT_CONTENT) == Some(&true) {
                    return Some(ScanResult {
                        symbol: SymbolId(COMMENT_CONTENT as u16),
                        length: 2,
                    });
                }
            } else if valid_symbols.get(COMMENT_CONTENT) == Some(&true) {
                // Regular content
                let mut length = 0;
                let mut i = position;

                while i + 1 < input.len() {
                    if (input[i] == b'/' && input[i + 1] == b'*') ||
                       (input[i] == b'*' && input[i + 1] == b'/') {
                        break;
                    }
                    i += 1;
                    length += 1;
                }

                if length > 0 {
                    return Some(ScanResult {
                        symbol: SymbolId(COMMENT_CONTENT as u16),
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
            self.depth = u32::from_le_bytes([
                buffer[0],
                buffer[1],
                buffer[2],
                buffer[3],
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_scanner() {
        let mut scanner = StringScanner::new();

        // Test string start
        let input = b"\"hello world\"";
        let valid = vec![true, true, true]; // All tokens valid

        let result = scanner.scan(&valid, input, 0);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(0), // STRING_START
            length: 1,
        }));

        // Test string content
        let result = scanner.scan(&valid, input, 1);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(1), // STRING_CONTENT
            length: 11, // "hello world"
        }));

        // Test string end
        let result = scanner.scan(&valid, input, 12);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(2), // STRING_END
            length: 1,
        }));
    }

    #[test]
    fn test_string_scanner_escapes() {
        let mut scanner = StringScanner::new();
        scanner.in_string = true;
        scanner.quote_char = Some(b'"');

        let input = b"hello\\\"world";
        let valid = vec![false, true, false];

        let result = scanner.scan(&valid, input, 0);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(1), // STRING_CONTENT
            length: 12, // includes escape sequence
        }));
    }

    #[test]
    fn test_comment_scanner() {
        let mut scanner = CommentScanner::new();

        // Test comment start
        let input = b"/* hello /* nested */ world */";
        let valid = vec![true, true, true];

        let result = scanner.scan(&valid, input, 0);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(0), // COMMENT_START
            length: 2,
        }));
        assert_eq!(scanner.depth, 1);

        // Test nested comment
        let result = scanner.scan(&valid, input, 9);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(1), // COMMENT_CONTENT
            length: 2,
        }));
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

        assert_eq!(new_scanner.in_string, true);
        assert_eq!(new_scanner.quote_char, Some(b'\''));
    }
}