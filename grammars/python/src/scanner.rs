// Python indentation scanner for rust-sitter

use rust_sitter::external_scanner::{ExternalScanner, Lexer, ScanResult};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum TokenType {
    Newline = 0,
    Indent = 1,
    Dedent = 2,
    StringStart = 3,
    StringEnd = 4,
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
    fn scan(&mut self, lexer: &mut Lexer, valid_symbols: &[bool]) -> ScanResult {
        // Skip any horizontal whitespace at the start
        let mut indent_length = 0;
        let mut found_space_indent = false;
        let mut found_tab_indent = false;
        
        // Handle string scanning
        if self.inside_string && valid_symbols[TokenType::StringEnd as usize] {
            if let Some(delimiter) = self.string_delimiter {
                match delimiter {
                    StringDelimiter::SingleQuote => {
                        if lexer.lookahead() == '\'' {
                            lexer.advance(false);
                            self.inside_string = false;
                            self.string_delimiter = None;
                            lexer.result_symbol(TokenType::StringEnd as usize);
                            return ScanResult::Found;
                        }
                    }
                    StringDelimiter::DoubleQuote => {
                        if lexer.lookahead() == '"' {
                            lexer.advance(false);
                            self.inside_string = false;
                            self.string_delimiter = None;
                            lexer.result_symbol(TokenType::StringEnd as usize);
                            return ScanResult::Found;
                        }
                    }
                    StringDelimiter::TripleSingleQuote => {
                        if self.match_string(lexer, "'''") {
                            self.inside_string = false;
                            self.string_delimiter = None;
                            lexer.result_symbol(TokenType::StringEnd as usize);
                            return ScanResult::Found;
                        }
                    }
                    StringDelimiter::TripleDoubleQuote => {
                        if self.match_string(lexer, r#"""""#) {
                            self.inside_string = false;
                            self.string_delimiter = None;
                            lexer.result_symbol(TokenType::StringEnd as usize);
                            return ScanResult::Found;
                        }
                    }
                }
            }
            return ScanResult::NotFound;
        }
        
        // Check for string start
        if valid_symbols[TokenType::StringStart as usize] {
            // Check for triple quotes first
            if self.match_string(lexer, "'''") {
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::TripleSingleQuote);
                lexer.result_symbol(TokenType::StringStart as usize);
                return ScanResult::Found;
            }
            
            if self.match_string(lexer, r#"""""#) {
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::TripleDoubleQuote);
                lexer.result_symbol(TokenType::StringStart as usize);
                return ScanResult::Found;
            }
            
            // Check for single quotes
            if lexer.lookahead() == '\'' {
                lexer.advance(false);
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::SingleQuote);
                lexer.result_symbol(TokenType::StringStart as usize);
                return ScanResult::Found;
            }
            
            if lexer.lookahead() == '"' {
                lexer.advance(false);
                self.inside_string = true;
                self.string_delimiter = Some(StringDelimiter::DoubleQuote);
                lexer.result_symbol(TokenType::StringStart as usize);
                return ScanResult::Found;
            }
        }
        
        // Handle newlines and indentation
        if valid_symbols[TokenType::Newline as usize] {
            if lexer.lookahead() == '\n' {
                lexer.advance(false);
                lexer.mark_end();
                
                // Skip blank lines and comments
                loop {
                    // Count indentation
                    indent_length = 0;
                    while !lexer.eof() {
                        if lexer.lookahead() == ' ' {
                            found_space_indent = true;
                            indent_length += 1;
                        } else if lexer.lookahead() == '\t' {
                            found_tab_indent = true;
                            indent_length += 8; // Tabs count as 8 spaces
                        } else {
                            break;
                        }
                        lexer.advance(true);
                    }
                    
                    // Check if it's a blank line or comment
                    if lexer.lookahead() == '\n' {
                        lexer.advance(false);
                        continue;
                    } else if lexer.lookahead() == '#' {
                        // Skip comment line
                        while !lexer.eof() && lexer.lookahead() != '\n' {
                            lexer.advance(true);
                        }
                        if lexer.lookahead() == '\n' {
                            lexer.advance(false);
                            continue;
                        }
                    }
                    
                    break;
                }
                
                lexer.result_symbol(TokenType::Newline as usize);
                return ScanResult::Found;
            }
        }
        
        // Handle indentation at the beginning of a line
        if lexer.get_column() == 0 {
            // Count leading whitespace
            while !lexer.eof() {
                if lexer.lookahead() == ' ' {
                    found_space_indent = true;
                    indent_length += 1;
                    lexer.advance(true);
                } else if lexer.lookahead() == '\t' {
                    found_tab_indent = true;
                    indent_length += 8; // Tabs count as 8 spaces
                    lexer.advance(true);
                } else {
                    break;
                }
            }
            
            // Don't emit indentation tokens for blank lines or comments
            if lexer.lookahead() == '\n' || lexer.lookahead() == '#' {
                return ScanResult::NotFound;
            }
            
            // Get current indentation level
            let current_indent = if self.indent_stack.is_empty() {
                0
            } else {
                *self.indent_stack.last().unwrap()
            };
            
            if valid_symbols[TokenType::Indent as usize] && indent_length > current_indent {
                self.indent_stack.push(indent_length);
                lexer.result_symbol(TokenType::Indent as usize);
                return ScanResult::Found;
            }
            
            if valid_symbols[TokenType::Dedent as usize] && indent_length < current_indent {
                // Pop all indentation levels greater than the current line's indentation
                while !self.indent_stack.is_empty() {
                    let last_indent = *self.indent_stack.last().unwrap();
                    if last_indent <= indent_length {
                        break;
                    }
                    self.indent_stack.pop();
                }
                
                lexer.result_symbol(TokenType::Dedent as usize);
                return ScanResult::Found;
            }
        }
        
        ScanResult::NotFound
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
    
    fn deserialize(buffer: &[u8]) -> Self where Self: Sized {
        let mut scanner = PythonScanner::default();
        
        if buffer.len() < 2 {
            return scanner;
        }
        
        // Deserialize indent stack
        let stack_len = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;
        let mut offset = 2;
        
        for _ in 0..stack_len {
            if offset + 2 > buffer.len() {
                break;
            }
            let indent = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
            scanner.indent_stack.push(indent);
            offset += 2;
        }
        
        // Deserialize string state
        if offset < buffer.len() {
            scanner.inside_string = buffer[offset] != 0;
            offset += 1;
        }
        
        if offset < buffer.len() {
            scanner.string_delimiter = match buffer[offset] {
                1 => Some(StringDelimiter::SingleQuote),
                2 => Some(StringDelimiter::DoubleQuote),
                3 => Some(StringDelimiter::TripleSingleQuote),
                4 => Some(StringDelimiter::TripleDoubleQuote),
                _ => None,
            };
        }
        
        scanner
    }
}

impl PythonScanner {
    fn match_string(&self, lexer: &mut Lexer, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 {
                lexer.advance(false);
            }
            if lexer.lookahead() != ch {
                return false;
            }
        }
        if !chars.is_empty() {
            lexer.advance(false);
        }
        true
    }
}

// Export the scanner
pub fn create_scanner() -> Box<dyn ExternalScanner> {
    Box::new(PythonScanner::default())
}