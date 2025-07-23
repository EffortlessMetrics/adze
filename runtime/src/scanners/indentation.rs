// Indentation scanner for Python-like languages
// Handles INDENT, DEDENT, and NEWLINE tokens

use crate::external_scanner::{ExternalScanner, ScanResult};
use rust_sitter_ir::SymbolId;

/// Token indices for indentation scanner
pub const NEWLINE: usize = 0;
pub const INDENT: usize = 1;
pub const DEDENT: usize = 2;

/// Indentation scanner for Python-like languages
pub struct IndentationScanner {
    /// Stack of indentation levels (column numbers)
    indent_stack: Vec<usize>,
    /// Whether we're at the beginning of a line
    at_line_start: bool,
    /// Pending dedent count
    pending_dedents: usize,
}

impl ExternalScanner for IndentationScanner {
    fn new() -> Self {
        IndentationScanner {
            indent_stack: vec![0], // Start with column 0
            at_line_start: true,
            pending_dedents: 0,
        }
    }
    
    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
        // If we have pending dedents, emit them first
        if self.pending_dedents > 0 && valid_symbols.get(DEDENT) == Some(&true) {
            self.pending_dedents -= 1;
            return Some(ScanResult {
                symbol: SymbolId(DEDENT as u16),
                length: 0,
            });
        }
        
        // Skip any whitespace that's not at line start
        if !self.at_line_start {
            // Look for newline
            if position < input.len() && input[position] == b'\n' {
                if valid_symbols.get(NEWLINE) == Some(&true) {
                    self.at_line_start = true;
                    return Some(ScanResult {
                        symbol: SymbolId(NEWLINE as u16),
                        length: 1,
                    });
                }
            }
            return None;
        }
        
        // We're at line start - count indentation
        let mut indent_length = 0;
        let mut column = 0;
        let mut i = position;
        
        while i < input.len() {
            match input[i] {
                b' ' => {
                    indent_length += 1;
                    column += 1;
                    i += 1;
                }
                b'\t' => {
                    indent_length += 1;
                    column = (column / 8 + 1) * 8; // Tab to next multiple of 8
                    i += 1;
                }
                b'\n' => {
                    // Empty line - skip it
                    return Some(ScanResult {
                        symbol: SymbolId(NEWLINE as u16),
                        length: i - position + 1,
                    });
                }
                b'#' => {
                    // Comment line - skip to end
                    while i < input.len() && input[i] != b'\n' {
                        i += 1;
                    }
                    if i < input.len() {
                        return Some(ScanResult {
                            symbol: SymbolId(NEWLINE as u16),
                            length: i - position + 1,
                        });
                    }
                    return None;
                }
                _ => {
                    // Non-whitespace character - process indentation
                    break;
                }
            }
        }
        
        // Check if we're at EOF after whitespace
        if i >= input.len() {
            return None;
        }
        
        self.at_line_start = false;
        let current_indent = *self.indent_stack.last().unwrap();
        
        if column > current_indent {
            // Indent
            if valid_symbols.get(INDENT) == Some(&true) {
                self.indent_stack.push(column);
                return Some(ScanResult {
                    symbol: SymbolId(INDENT as u16),
                    length: indent_length,
                });
            }
        } else if column < current_indent {
            // Dedent - might be multiple levels
            let mut dedent_count = 0;
            
            while let Some(&level) = self.indent_stack.last() {
                if level <= column {
                    break;
                }
                self.indent_stack.pop();
                dedent_count += 1;
            }
            
            // Verify we found a matching indent level
            if self.indent_stack.last() != Some(&column) {
                // Invalid dedent - this would be a parse error
                return None;
            }
            
            if dedent_count > 0 && valid_symbols.get(DEDENT) == Some(&true) {
                // Emit first dedent, store rest as pending
                self.pending_dedents = dedent_count - 1;
                return Some(ScanResult {
                    symbol: SymbolId(DEDENT as u16),
                    length: indent_length,
                });
            }
        }
        
        // Same indentation level - consume the whitespace
        if indent_length > 0 {
            return Some(ScanResult {
                symbol: SymbolId(NEWLINE as u16),
                length: 0, // Don't consume - let parser handle content
            });
        }
        
        None
    }
    
    fn serialize(&self, buffer: &mut Vec<u8>) {
        // Serialize the indent stack
        buffer.extend_from_slice(&(self.indent_stack.len() as u32).to_le_bytes());
        for &level in &self.indent_stack {
            buffer.extend_from_slice(&(level as u32).to_le_bytes());
        }
        
        // Serialize other state
        buffer.push(if self.at_line_start { 1 } else { 0 });
        buffer.extend_from_slice(&(self.pending_dedents as u32).to_le_bytes());
    }
    
    fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.len() < 4 {
            return;
        }
        
        let mut offset = 0;
        
        // Read indent stack length
        let stack_len = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as usize;
        offset += 4;
        
        // Read indent stack
        self.indent_stack.clear();
        for _ in 0..stack_len {
            if offset + 4 > buffer.len() {
                break;
            }
            let level = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as usize;
            self.indent_stack.push(level);
            offset += 4;
        }
        
        // Read other state
        if offset < buffer.len() {
            self.at_line_start = buffer[offset] != 0;
            offset += 1;
        }
        
        if offset + 4 <= buffer.len() {
            self.pending_dedents = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_indentation() {
        let mut scanner = IndentationScanner::new();
        
        // Test input with indentation
        let input = b"def foo():\n    print('hello')\n    print('world')\n";
        let valid = vec![true, true, true]; // All tokens valid
        
        // First line - no indent
        let result = scanner.scan(&valid, input, 0);
        assert!(result.is_none() || result.unwrap().symbol == SymbolId(NEWLINE as u16));
        
        // After newline, should get indent
        scanner.at_line_start = true;
        let result = scanner.scan(&valid, input, 11); // After "def foo():\n"
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(INDENT as u16),
            length: 4,
        }));
    }
    
    #[test]
    fn test_dedent() {
        let mut scanner = IndentationScanner::new();
        scanner.indent_stack = vec![0, 4]; // Already indented
        scanner.at_line_start = true;
        
        let input = b"return\n";
        let valid = vec![true, true, true];
        
        // At column 0, should dedent
        let result = scanner.scan(&valid, input, 0);
        assert_eq!(result, Some(ScanResult {
            symbol: SymbolId(DEDENT as u16),
            length: 0,
        }));
    }
    
    #[test]
    fn test_serialization() {
        let mut scanner = IndentationScanner::new();
        scanner.indent_stack = vec![0, 4, 8];
        scanner.at_line_start = false;
        scanner.pending_dedents = 2;
        
        // Serialize
        let mut buffer = Vec::new();
        scanner.serialize(&mut buffer);
        
        // Deserialize into new scanner
        let mut new_scanner = IndentationScanner::new();
        new_scanner.deserialize(&buffer);
        
        assert_eq!(new_scanner.indent_stack, vec![0, 4, 8]);
        assert_eq!(new_scanner.at_line_start, false);
        assert_eq!(new_scanner.pending_dedents, 2);
    }
}