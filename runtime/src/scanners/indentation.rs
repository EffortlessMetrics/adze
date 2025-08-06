// Indentation-based scanning for languages like Python
use crate::external_scanner::{ExternalScanner, ScanResult, Lexer};

/// Scanner for tracking indentation levels
#[derive(Debug, Clone, Default)]
pub struct IndentationScanner {
    indent_stack: Vec<usize>,
    at_line_start: bool,
    pending_dedents: usize,
}

impl IndentationScanner {
    pub fn new() -> Self {
        IndentationScanner {
            indent_stack: vec![0], // Start with column 0
            at_line_start: true,
            pending_dedents: 0,
        }
    }
}

impl ExternalScanner for IndentationScanner {
    fn scan(
        &mut self,
        lexer: &mut dyn Lexer,
        valid_symbols: &[bool],
    ) -> Option<ScanResult> {
        const NEWLINE: usize = 0;
        const INDENT: usize = 1;
        const DEDENT: usize = 2;
        
        // If we have pending dedents, emit them
        if self.pending_dedents > 0 && valid_symbols.get(DEDENT) == Some(&true) {
            self.pending_dedents -= 1;
            return Some(ScanResult {
                symbol: DEDENT as u16,
                length: 0,
            });
        }
        
        if lexer.is_eof() {
            return None;
        }
        
        // Check for newline
        if valid_symbols.get(NEWLINE) == Some(&true) && lexer.lookahead() == Some(b'\n') {
            self.at_line_start = true;
            lexer.advance(1);
            lexer.mark_end();
            return Some(ScanResult {
                symbol: NEWLINE as u16,
                length: 1,
            });
        }
        
        // Handle indentation at start of line
        if self.at_line_start {
            let mut indent_count = 0;
            
            // Count leading whitespace
            while !lexer.is_eof() {
                match lexer.lookahead() {
                    Some(b' ') => {
                        indent_count += 1;
                        lexer.advance(1);
                    }
                    Some(b'\t') => {
                        indent_count += 8; // Tabs count as 8 spaces
                        lexer.advance(1);
                    }
                    _ => break,
                }
            }
            
            // Skip blank lines and comment lines
            if !lexer.is_eof() {
                let next = lexer.lookahead();
                if next != Some(b'\n') && next != Some(b'#') {
                    self.at_line_start = false;
                    let current_indent = *self.indent_stack.last().unwrap();
                    
                    if indent_count > current_indent {
                        // Indent
                        if valid_symbols.get(INDENT) == Some(&true) {
                            self.indent_stack.push(indent_count);
                            lexer.mark_end();
                            return Some(ScanResult {
                                symbol: INDENT as u16,
                                length: 0,
                            });
                        }
                    } else if indent_count < current_indent {
                        // Dedent(s)
                        if valid_symbols.get(DEDENT) == Some(&true) {
                            // Count how many dedents are needed
                            let mut dedent_count = 0;
                            let mut temp_stack = self.indent_stack.clone();
                            
                            while let Some(&last) = temp_stack.last() {
                                if last <= indent_count {
                                    break;
                                }
                                temp_stack.pop();
                                dedent_count += 1;
                            }
                            
                            if dedent_count > 0 {
                                // Apply the dedents
                                for _ in 0..dedent_count {
                                    self.indent_stack.pop();
                                }
                                self.pending_dedents = dedent_count - 1;
                                lexer.mark_end();
                                return Some(ScanResult {
                                    symbol: DEDENT as u16,
                                    length: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
    
    fn serialize(&self, buffer: &mut Vec<u8>) {
        // Serialize indent stack
        buffer.extend_from_slice(&(self.indent_stack.len() as u16).to_le_bytes());
        for &indent in &self.indent_stack {
            buffer.extend_from_slice(&(indent as u16).to_le_bytes());
        }
        
        // Serialize flags
        buffer.push(if self.at_line_start { 1 } else { 0 });
        buffer.extend_from_slice(&(self.pending_dedents as u16).to_le_bytes());
    }
    
    fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.len() < 2 {
            return;
        }
        
        self.indent_stack.clear();
        
        // Deserialize indent stack
        let stack_len = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;
        let mut offset = 2;
        
        for _ in 0..stack_len {
            if offset + 2 > buffer.len() {
                break;
            }
            let indent = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as usize;
            self.indent_stack.push(indent);
            offset += 2;
        }
        
        // Deserialize flags
        if offset < buffer.len() {
            self.at_line_start = buffer[offset] != 0;
            offset += 1;
        }
        
        if offset + 2 <= buffer.len() {
            self.pending_dedents = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as usize;
        }
    }
}