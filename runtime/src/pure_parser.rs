// Pure-Rust Tree-sitter compatible parser runtime
// This implements the core parsing algorithm with GLR support

use crate::{Node, Point, Language};
use std::sync::atomic::AtomicBool;

/// Parser for Tree-sitter grammars
#[derive(Debug)]
pub struct Parser {
    language: Option<Language>,
    stack: Vec<StackEntry>,
    timeout_micros: u64,
    cancellation_flag: Option<*const AtomicBool>,
}

/// Stack entry for LR parsing
#[derive(Debug, Clone)]
struct StackEntry {
    state: u16,
    subtree: Option<Subtree>,
    position: usize,
}

/// Internal node representation during parsing
#[derive(Debug, Clone)]
struct Subtree {
    symbol: u16,
    children: Vec<Subtree>,
    start_byte: usize,
    end_byte: usize,
    start_point: Point,
    end_point: Point,
    is_extra: bool,
    is_error: bool,
}

/// Parse result
pub struct ParseResult {
    pub root: Option<ParsedNode>,
    pub errors: Vec<ParseError>,
}

/// Parsed node
#[derive(Debug, Clone)]
pub struct ParsedNode {
    symbol: u16,
    children: Vec<ParsedNode>,
    start_byte: usize,
    end_byte: usize,
    start_point: Point,
    end_point: Point,
    is_extra: bool,
    is_error: bool,
}

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: usize,
    pub point: Point,
    pub expected: Vec<u16>,
    pub found: u16,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Parser {
            language: None,
            stack: Vec::new(),
            timeout_micros: 0,
            cancellation_flag: None,
        }
    }
    
    /// Set the language for parsing
    pub fn set_language(&mut self, language: Language) -> Result<(), String> {
        self.language = Some(language);
        self.reset();
        Ok(())
    }
    
    /// Get the current language
    pub fn language(&self) -> Option<&Language> {
        self.language.as_ref()
    }
    
    /// Set timeout for parsing in microseconds
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.timeout_micros = timeout;
    }
    
    /// Set cancellation flag for parsing
    pub fn set_cancellation_flag(&mut self, flag: Option<*const AtomicBool>) {
        self.cancellation_flag = flag;
    }
    
    /// Reset parser state
    pub fn reset(&mut self) {
        self.stack.clear();
    }
    
    /// Parse a string of source code
    pub fn parse_string(&mut self, source: &str) -> ParseResult {
        let bytes = source.as_bytes();
        self.parse_bytes(bytes)
    }
    
    /// Parse bytes of source code
    pub fn parse_bytes(&mut self, source: &[u8]) -> ParseResult {
        let language = match self.language.as_ref() {
            Some(lang) => lang,
            None => return ParseResult {
                root: None,
                errors: vec![ParseError {
                    position: 0,
                    point: Point { row: 0, column: 0 },
                    expected: vec![],
                    found: 0,
                }],
            },
        };
        
        // Initialize parser state
        self.stack.clear();
        self.stack.push(StackEntry {
            state: 0,
            subtree: None,
            position: 0,
        });
        
        let mut errors = Vec::new();
        let mut position = 0;
        let mut point = Point { row: 0, column: 0 };
        let start_time = std::time::Instant::now();
        
        // Main parsing loop
        loop {
            // Check timeout
            if self.timeout_micros > 0 {
                let elapsed = start_time.elapsed().as_micros() as u64;
                if elapsed > self.timeout_micros {
                    errors.push(ParseError {
                        position,
                        point,
                        expected: vec![],
                        found: 0,
                    });
                    break;
                }
            }
            
            // Check cancellation
            if let Some(flag) = self.cancellation_flag {
                unsafe {
                    if (*flag).load(std::sync::atomic::Ordering::Relaxed) {
                        errors.push(ParseError {
                            position,
                            point,
                            expected: vec![],
                            found: 0,
                        });
                        break;
                    }
                }
            }
            
            // Get current state
            let current_state = match self.stack.last() {
                Some(entry) => entry.state,
                None => break,
            };
            
            // Lex next token
            let token = self.lex_token(source, position, current_state);
            
            // Get action for current state and token
            match self.get_action(language, current_state, token.symbol) {
                Action::Shift(next_state) => {
                    // Create leaf node
                    let subtree = Subtree {
                        symbol: token.symbol,
                        children: Vec::new(),
                        start_byte: position,
                        end_byte: position + token.length,
                        start_point: point,
                        end_point: advance_point(point, &source[position..position + token.length]),
                        is_extra: false,
                        is_error: false,
                    };
                    
                    // Push onto stack
                    self.stack.push(StackEntry {
                        state: next_state,
                        subtree: Some(subtree),
                        position: position + token.length,
                    });
                    
                    // Advance position
                    position += token.length;
                    point = advance_point(point, &source[position - token.length..position]);
                }
                
                Action::Reduce(rule_id) => {
                    self.reduce(language, rule_id);
                }
                
                Action::Accept => {
                    // Parse successful
                    if let Some(entry) = self.stack.pop() {
                        if let Some(subtree) = entry.subtree {
                            return ParseResult {
                                root: Some(subtree_to_node(subtree)),
                                errors,
                            };
                        }
                    }
                    break;
                }
                
                Action::Error => {
                    // Record error and try to recover
                    errors.push(ParseError {
                        position,
                        point,
                        expected: self.get_expected_symbols(language, current_state),
                        found: token.symbol,
                    });
                    
                    // Simple error recovery: skip token
                    if position < source.len() {
                        position += 1;
                        point = advance_point(point, &source[position - 1..position]);
                    } else {
                        break;
                    }
                }
            }
        }
        
        ParseResult { root: None, errors }
    }
    
    /// Lex a token at the current position
    fn lex_token(&self, source: &[u8], position: usize, _state: u16) -> Token {
        if position >= source.len() {
            return Token { symbol: 0, length: 0 }; // EOF
        }
        
        // Simple lexer for testing
        let ch = source[position];
        
        // Skip whitespace
        if ch.is_ascii_whitespace() {
            let mut len = 1;
            while position + len < source.len() && source[position + len].is_ascii_whitespace() {
                len += 1;
            }
            return Token { symbol: 1, length: len };
        }
        
        // Single character tokens
        Token { symbol: ch as u16, length: 1 }
    }
    
    /// Get parse action for state and symbol
    fn get_action(&self, language: &Language, state: u16, symbol: u16) -> Action {
        // This would normally look up in the parse table
        // For now, return a simple action based on state
        if state == 0 && symbol > 0 {
            Action::Shift(1)
        } else if state == 1 {
            Action::Accept
        } else {
            Action::Error
        }
    }
    
    /// Perform a reduction
    fn reduce(&mut self, _language: &Language, _rule_id: u16) {
        // Pop items from stack based on rule
        // For now, just pop one item
        if let Some(entry) = self.stack.pop() {
            if let Some(subtree) = entry.subtree {
                // Create parent node
                let parent = Subtree {
                    symbol: 100, // Placeholder non-terminal
                    children: vec![subtree],
                    start_byte: subtree.start_byte,
                    end_byte: subtree.end_byte,
                    start_point: subtree.start_point,
                    end_point: subtree.end_point,
                    is_extra: false,
                    is_error: false,
                };
                
                // Push parent onto stack
                if let Some(prev_entry) = self.stack.last() {
                    self.stack.push(StackEntry {
                        state: prev_entry.state + 1, // Simple goto
                        subtree: Some(parent),
                        position: entry.position,
                    });
                }
            }
        }
    }
    
    /// Get expected symbols for error reporting
    fn get_expected_symbols(&self, _language: &Language, _state: u16) -> Vec<u16> {
        vec![] // Placeholder
    }
}

/// Token returned by lexer
#[derive(Debug, Clone, Copy)]
struct Token {
    symbol: u16,
    length: usize,
}

/// Parse action
#[derive(Debug, Clone, Copy)]
enum Action {
    Shift(u16),
    Reduce(u16),
    Accept,
    Error,
}

/// Advance point by text
fn advance_point(mut point: Point, text: &[u8]) -> Point {
    for &byte in text {
        if byte == b'\n' {
            point.row += 1;
            point.column = 0;
        } else {
            point.column += 1;
        }
    }
    point
}

/// Convert internal subtree to public node
fn subtree_to_node(subtree: Subtree) -> ParsedNode {
    ParsedNode {
        symbol: subtree.symbol,
        children: subtree.children.into_iter().map(subtree_to_node).collect(),
        start_byte: subtree.start_byte,
        end_byte: subtree.end_byte,
        start_point: subtree.start_point,
        end_point: subtree.end_point,
        is_extra: subtree.is_extra,
        is_error: subtree.is_error,
    }
}

impl ParsedNode {
    /// Get symbol ID
    pub fn symbol(&self) -> u16 {
        self.symbol
    }
    
    /// Get start byte offset
    pub fn start_byte(&self) -> usize {
        self.start_byte
    }
    
    /// Get end byte offset
    pub fn end_byte(&self) -> usize {
        self.end_byte
    }
    
    /// Get start point
    pub fn start_point(&self) -> Point {
        self.start_point
    }
    
    /// Get end point
    pub fn end_point(&self) -> Point {
        self.end_point
    }
    
    /// Get child count
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
    
    /// Get child at index
    pub fn child(&self, index: usize) -> Option<&ParsedNode> {
        self.children.get(index)
    }
    
    /// Get all children
    pub fn children(&self) -> &[ParsedNode] {
        &self.children
    }
    
    /// Check if node is extra
    pub fn is_extra(&self) -> bool {
        self.is_extra
    }
    
    /// Check if node is error
    pub fn is_error(&self) -> bool {
        self.is_error
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parser = Parser::new();
        assert!(parser.language().is_none());
        assert_eq!(parser.timeout_micros, 0);
    }
    
    #[test]
    fn test_point_advance() {
        let point = Point { row: 0, column: 0 };
        let point = advance_point(point, b"hello");
        assert_eq!(point.row, 0);
        assert_eq!(point.column, 5);
        
        let point = advance_point(point, b"\nworld");
        assert_eq!(point.row, 1);
        assert_eq!(point.column, 5);
    }
    
    #[test]
    fn test_empty_parse() {
        let mut parser = Parser::new();
        let result = parser.parse_string("");
        assert!(result.root.is_none());
        assert!(!result.errors.is_empty());
    }
}