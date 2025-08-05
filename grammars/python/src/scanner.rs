// Python indentation scanner for rust-sitter

use rust_sitter::external_scanner::{ExternalScanner, ScanResult};
use rust_sitter_ir::SymbolId;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum TokenType {
    Newline = 0,
    Indent = 1,
    Dedent = 2,
    StringStart = 3,
    StringEnd = 4,
    StringContent = 5,
    Comment = 6,
}

#[derive(Debug, Clone, Default)]
pub struct PythonScanner {
    indent_stack: Vec<u16>,
    inside_string: bool,
    string_delimiter: Option<StringDelimiter>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StringDelimiter {
    SingleQuote,
    DoubleQuote,
    TripleSingleQuote,
    TripleDoubleQuote,
}

impl ExternalScanner for PythonScanner {
    fn new() -> Self {
        PythonScanner::default()
    }
    
    fn scan(&mut self, valid_symbols: &[bool], input: &[u8], position: usize) -> Option<ScanResult> {
        if position >= input.len() {
            return None;
        }
        
        // Handle string scanning
        if self.inside_string && valid_symbols.get(TokenType::StringEnd as usize) == Some(&true) {
            if let Some(delimiter) = self.string_delimiter {
                match delimiter {
                    StringDelimiter::SingleQuote => {
                        if position < input.len() && input[position] == b'\'' {
                            self.inside_string = false;
                            self.string_delimiter = None;
                            return Some(ScanResult {
                                symbol: SymbolId(TokenType::StringEnd as u16),
                                length: 1,
                            });
                        }
                    }
                    StringDelimiter::DoubleQuote => {
                        if position < input.len() && input[position] == b'"' {
                            self.inside_string = false;
                            self.string_delimiter = None;
                            return Some(ScanResult {
                                symbol: SymbolId(TokenType::StringEnd as u16),
                                length: 1,
                            });
                        }
                    }
                    StringDelimiter::TripleSingleQuote => {
                        if self.match_string_at(input, position, b"'''") {
                            self.inside_string = false;
                            self.string_delimiter = None;
                            return Some(ScanResult {
                                symbol: SymbolId(TokenType::StringEnd as u16),
                                length: 3,
                            });
                        }
                    }
                    StringDelimiter::TripleDoubleQuote => {
                        if self.match_string_at(input, position, b"\"\"\"") {
                            self.inside_string = false;
                            self.string_delimiter = None;
                            return Some(ScanResult {
                                symbol: SymbolId(TokenType::StringEnd as u16),
                                length: 3,
                            });
                        }
                    }
                }
            }
            
            // If we're in a string but can't find the end, consume content
            if valid_symbols.get(TokenType::StringContent as usize) == Some(&true) {
                let mut length = 0;
                let mut i = position;
                
                while i < input.len() {
                    // Check if we hit the string delimiter
                    match self.string_delimiter {
                        Some(StringDelimiter::SingleQuote) if input[i] == b'\'' => break,
                        Some(StringDelimiter::DoubleQuote) if input[i] == b'"' => break,
                        Some(StringDelimiter::TripleSingleQuote) 
                            if self.match_string_at(input, i, b"'''") => break,
                        Some(StringDelimiter::TripleDoubleQuote) 
                            if self.match_string_at(input, i, b"\"\"\"") => break,
                        _ => {}
                    }
                    
                    // Handle escape sequences
                    if input[i] == b'\\' && i + 1 < input.len() {
                        i += 2;
                        length += 2;
                    } else {
                        i += 1;
                        length += 1;
                    }
                }
                
                if length > 0 {
                    return Some(ScanResult {
                        symbol: SymbolId(TokenType::StringContent as u16),
                        length,
                    });
                }
            }
            
            return None;
        }
        
        // Check for string start
        if valid_symbols.get(TokenType::StringStart as usize) == Some(&true) {
            // Check for triple quotes first
            if self.match_string_at(input, position, b"'''") {
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::TripleSingleQuote);
                return Some(ScanResult {
                    symbol: SymbolId(TokenType::StringStart as u16),
                    length: 3,
                });
            }
            
            if self.match_string_at(input, position, b"\"\"\"") {
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::TripleDoubleQuote);
                return Some(ScanResult {
                    symbol: SymbolId(TokenType::StringStart as u16),
                    length: 3,
                });
            }
            
            // Check for single quotes
            if position < input.len() && input[position] == b'\'' {
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::SingleQuote);
                return Some(ScanResult {
                    symbol: SymbolId(TokenType::StringStart as u16),
                    length: 1,
                });
            }
            
            if position < input.len() && input[position] == b'"' {
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::DoubleQuote);
                return Some(ScanResult {
                    symbol: SymbolId(TokenType::StringStart as u16),
                    length: 1,
                });
            }
        }
        
        // Handle newlines and indentation
        if valid_symbols.get(TokenType::Newline as usize) == Some(&true) {
            if position < input.len() && input[position] == b'\n' {
                return Some(ScanResult {
                    symbol: SymbolId(TokenType::Newline as u16),
                    length: 1,
                });
            }
        }
        
        // Handle indentation at the beginning of a line (column 0)
        if position == 0 || (position > 0 && input[position - 1] == b'\n') {
            let mut indent_length = 0;
            let mut i = position;
            
            // Count leading whitespace
            while i < input.len() {
                if input[i] == b' ' {
                    indent_length += 1;
                    i += 1;
                } else if input[i] == b'\t' {
                    indent_length += 8; // Tabs count as 8 spaces
                    i += 1;
                } else {
                    break;
                }
            }
            
            // Don't emit indentation tokens for blank lines or comments
            if i < input.len() && input[i] != b'\n' && input[i] != b'#' {
                // Get current indentation level
                let current_indent = self.indent_stack.last().copied().unwrap_or(0);
                
                if valid_symbols.get(TokenType::Indent as usize) == Some(&true) 
                    && indent_length > current_indent {
                    self.indent_stack.push(indent_length);
                    return Some(ScanResult {
                        symbol: SymbolId(TokenType::Indent as u16),
                        length: 0, // Indents don't consume characters
                    });
                }
                
                if valid_symbols.get(TokenType::Dedent as usize) == Some(&true) 
                    && indent_length < current_indent {
                    // Pop all indentation levels greater than the current line's indentation
                    while let Some(&last_indent) = self.indent_stack.last() {
                        if last_indent <= indent_length {
                            break;
                        }
                        self.indent_stack.pop();
                    }
                    
                    return Some(ScanResult {
                        symbol: SymbolId(TokenType::Dedent as u16),
                        length: 0, // Dedents don't consume characters
                    });
                }
            }
        }
        
        None
    }
    
    fn serialize(&self, buffer: &mut Vec<u8>) {
        // Serialize the indent stack
        buffer.extend_from_slice(&(self.indent_stack.len() as u16).to_le_bytes());
        for &indent in &self.indent_stack {
            buffer.extend_from_slice(&indent.to_le_bytes());
        }
        
        // Serialize string state
        buffer.push(if self.inside_string { 1 } else { 0 });
        buffer.push(match self.string_delimiter {
            None => 0,
            Some(StringDelimiter::SingleQuote) => 1,
            Some(StringDelimiter::DoubleQuote) => 2,
            Some(StringDelimiter::TripleSingleQuote) => 3,
            Some(StringDelimiter::TripleDoubleQuote) => 4,
        });
    }
    
    fn deserialize(&mut self, buffer: &[u8]) {
        self.indent_stack.clear();
        
        if buffer.len() < 2 {
            return;
        }
        
        // Deserialize indent stack
        let stack_len = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;
        let mut offset = 2;
        
        for _ in 0..stack_len {
            if offset + 2 > buffer.len() {
                break;
            }
            let indent = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
            self.indent_stack.push(indent);
            offset += 2;
        }
        
        // Deserialize string state
        if offset < buffer.len() {
            self.inside_string = buffer[offset] != 0;
            offset += 1;
        }
        
        if offset < buffer.len() {
            self.string_delimiter = match buffer[offset] {
                1 => Some(StringDelimiter::SingleQuote),
                2 => Some(StringDelimiter::DoubleQuote),
                3 => Some(StringDelimiter::TripleSingleQuote),
                4 => Some(StringDelimiter::TripleDoubleQuote),
                _ => None,
            };
        }
    }
}

impl PythonScanner {
    fn match_string_at(&self, input: &[u8], position: usize, pattern: &[u8]) -> bool {
        if position + pattern.len() > input.len() {
            return false;
        }
        
        for (i, &ch) in pattern.iter().enumerate() {
            if input[position + i] != ch {
                return false;
            }
        }
        
        true
    }
}

// Export the scanner creation function
pub fn create_scanner() -> Box<dyn ExternalScanner> {
    Box::new(PythonScanner::default())
}