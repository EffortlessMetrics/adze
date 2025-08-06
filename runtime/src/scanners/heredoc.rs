// Heredoc scanner for shell-like languages
use crate::external_scanner::{ExternalScanner, ScanResult, Lexer};

/// Scanner for heredoc strings in shell-like languages
#[derive(Debug, Clone, Default)]
pub struct HeredocScanner {
    delimiter: Vec<u8>,
    in_heredoc: bool,
}

impl HeredocScanner {
    pub fn new() -> Self {
        HeredocScanner {
            delimiter: Vec::new(),
            in_heredoc: false,
        }
    }
}

impl ExternalScanner for HeredocScanner {
    fn scan(
        &mut self,
        lexer: &mut dyn Lexer,
        valid_symbols: &[bool],
    ) -> Option<ScanResult> {
        const HEREDOC_START: usize = 0;
        const HEREDOC_BODY: usize = 1;
        const HEREDOC_END: usize = 2;
        
        if lexer.is_eof() {
            return None;
        }
        
        if !self.in_heredoc {
            // Look for heredoc start (<<DELIMITER)
            if valid_symbols.get(HEREDOC_START) == Some(&true) {
                // Check for <<
                if lexer.lookahead() == Some(b'<') {
                    lexer.advance(1);
                    if lexer.lookahead() == Some(b'<') {
                        lexer.advance(1);
                        
                        // Skip optional whitespace
                        while lexer.lookahead() == Some(b' ') || lexer.lookahead() == Some(b'\t') {
                            lexer.advance(1);
                        }
                        
                        // Read delimiter
                        self.delimiter.clear();
                        while !lexer.is_eof() {
                            if let Some(ch) = lexer.lookahead() {
                                if ch == b'\n' || ch == b' ' || ch == b'\t' {
                                    break;
                                }
                                self.delimiter.push(ch);
                                lexer.advance(1);
                            } else {
                                break;
                            }
                        }
                        
                        if !self.delimiter.is_empty() {
                            self.in_heredoc = true;
                            lexer.mark_end();
                            return Some(ScanResult {
                                symbol: HEREDOC_START as u16,
                                length: 2 + self.delimiter.len(),
                            });
                        }
                    }
                }
            }
        } else {
            // Inside heredoc - look for delimiter or body
            if valid_symbols.get(HEREDOC_END) == Some(&true) {
                // Check if current line starts with delimiter
                if lexer.column() == 0 {
                    let mut matches = true;
                    let mut temp_pos = 0;
                    
                    for &expected in &self.delimiter {
                        if lexer.lookahead() != Some(expected) {
                            matches = false;
                            break;
                        }
                        lexer.advance(1);
                        temp_pos += 1;
                    }
                    
                    if matches && (lexer.lookahead() == Some(b'\n') || lexer.is_eof()) {
                        self.in_heredoc = false;
                        self.delimiter.clear();
                        lexer.mark_end();
                        return Some(ScanResult {
                            symbol: HEREDOC_END as u16,
                            length: temp_pos,
                        });
                    }
                    
                    // Rewind if not a match
                    // Note: This is simplified - proper implementation would need better lookahead
                }
            }
            
            if valid_symbols.get(HEREDOC_BODY) == Some(&true) {
                // Consume heredoc body until end of line
                let mut length = 0;
                while !lexer.is_eof() && lexer.lookahead() != Some(b'\n') {
                    lexer.advance(1);
                    length += 1;
                }
                
                if lexer.lookahead() == Some(b'\n') {
                    lexer.advance(1);
                    length += 1;
                }
                
                if length > 0 {
                    lexer.mark_end();
                    return Some(ScanResult {
                        symbol: HEREDOC_BODY as u16,
                        length,
                    });
                }
            }
        }
        
        None
    }
    
    fn serialize(&self, buffer: &mut Vec<u8>) {
        // Serialize delimiter length and content
        buffer.extend_from_slice(&(self.delimiter.len() as u16).to_le_bytes());
        buffer.extend_from_slice(&self.delimiter);
        
        // Serialize state
        buffer.push(if self.in_heredoc { 1 } else { 0 });
    }
    
    fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.len() < 2 {
            return;
        }
        
        // Deserialize delimiter
        let delimiter_len = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;
        self.delimiter.clear();
        
        let offset = 2;
        if offset + delimiter_len <= buffer.len() {
            self.delimiter.extend_from_slice(&buffer[offset..offset + delimiter_len]);
        }
        
        // Deserialize state
        let state_offset = offset + delimiter_len;
        if state_offset < buffer.len() {
            self.in_heredoc = buffer[state_offset] != 0;
        }
    }
}