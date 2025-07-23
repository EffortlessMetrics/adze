// Pure-Rust parser execution engine
// This module implements the runtime parsing logic for the pure-Rust Tree-sitter

use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{StateId, SymbolId};

/// Parser state during execution
#[derive(Debug, Clone)]
pub struct ParserState {
    /// Current state in the parse table
    state: StateId,
    /// Start position in the input
    #[allow(dead_code)]
    start_pos: usize,
    /// End position in the input
    #[allow(dead_code)]
    end_pos: usize,
}

/// A node in the parse tree being constructed
#[derive(Debug, Clone)]
pub struct ParseNode {
    /// Symbol ID for this node
    #[allow(dead_code)]
    symbol: SymbolId,
    /// Child nodes
    #[allow(dead_code)]
    children: Vec<ParseNode>,
    /// Start byte offset in the input
    #[allow(dead_code)]
    start_byte: usize,
    /// End byte offset in the input
    #[allow(dead_code)]
    end_byte: usize,
}

/// The main parser engine
pub struct Parser {
    /// Parse table for the grammar
    parse_table: ParseTable,
    /// Stack of parser states
    state_stack: Vec<ParserState>,
    /// Stack of parse nodes
    node_stack: Vec<ParseNode>,
    /// Input being parsed
    input: Vec<u8>,
    /// Current position in the input
    position: usize,
}

/// Lexer interface for tokenization
pub trait Lexer {
    /// Get the next token from the input
    fn next_token(&mut self, input: &[u8], position: usize) -> Option<Token>;
    
    /// Check if we're at the end of input
    fn is_eof(&self, input: &[u8], position: usize) -> bool {
        position >= input.len()
    }
}

/// A token produced by the lexer
#[derive(Debug, Clone)]
pub struct Token {
    /// Symbol ID for this token
    pub symbol: SymbolId,
    /// Token text
    pub text: Vec<u8>,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

/// Parse errors that can occur
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token encountered
    UnexpectedToken {
        expected: Vec<SymbolId>,
        found: SymbolId,
        position: usize,
    },
    /// No valid parse found
    InvalidParse,
    /// Parser is in an invalid state
    InvalidState,
}

impl Parser {
    /// Create a new parser with the given parse table
    pub fn new(parse_table: ParseTable) -> Self {
        Self {
            parse_table,
            state_stack: vec![ParserState {
                state: StateId(0), // Start state
                start_pos: 0,
                end_pos: 0,
            }],
            node_stack: Vec::new(),
            input: Vec::new(),
            position: 0,
        }
    }
    
    /// Parse the input using the provided lexer
    pub fn parse<L: Lexer>(&mut self, input: &[u8], lexer: &mut L) -> Result<ParseNode, ParseError> {
        self.input = input.to_vec();
        self.position = 0;
        self.state_stack.clear();
        self.state_stack.push(ParserState {
            state: StateId(0),
            start_pos: 0,
            end_pos: 0,
        });
        self.node_stack.clear();
        
        // Main parse loop
        loop {
            // Get current state
            let current_state = self.state_stack.last()
                .ok_or(ParseError::InvalidState)?
                .state;
            
            // Get next token
            let token = if lexer.is_eof(input, self.position) {
                Token {
                    symbol: SymbolId(0), // EOF symbol
                    text: vec![],
                    start: self.position,
                    end: self.position,
                }
            } else {
                lexer.next_token(input, self.position)
                    .ok_or(ParseError::InvalidParse)?
            };
            
            // Look up action in parse table
            let action = self.get_action(current_state, token.symbol)?;
            
            match action {
                Action::Shift(next_state) => {
                    // Push token as a leaf node
                    self.node_stack.push(ParseNode {
                        symbol: token.symbol,
                        children: vec![],
                        start_byte: token.start,
                        end_byte: token.end,
                    });
                    
                    // Push new state
                    self.state_stack.push(ParserState {
                        state: next_state,
                        start_pos: token.start,
                        end_pos: token.end,
                    });
                    
                    // Advance position
                    self.position = token.end;
                }
                
                Action::Reduce(_rule_id) => {
                    // TODO: Implement reduction logic
                    // This requires looking up the rule from the grammar
                    // For now, we'll implement a simplified version
                    
                    // In a real implementation, we would:
                    // 1. Look up the rule to get its length and LHS symbol
                    // 2. Pop that many states and nodes from the stacks
                    // 3. Create a new node with the popped nodes as children
                    // 4. Look up the goto state for the LHS symbol
                    // 5. Push the new node and state
                    
                    return Err(ParseError::InvalidParse);
                }
                
                Action::Accept => {
                    // Parse complete!
                    return self.node_stack.pop()
                        .ok_or(ParseError::InvalidState);
                }
                
                Action::Error => {
                    return Err(ParseError::UnexpectedToken {
                        expected: self.get_expected_symbols(current_state),
                        found: token.symbol,
                        position: self.position,
                    });
                }
                
                Action::Fork(_actions) => {
                    // TODO: Implement GLR fork handling
                    // For now, just take the first action
                    return Err(ParseError::InvalidParse);
                }
            }
        }
    }
    
    /// Get the action for a state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action, ParseError> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;
        
        if state_idx >= self.parse_table.action_table.len() {
            return Err(ParseError::InvalidState);
        }
        
        let row = &self.parse_table.action_table[state_idx];
        if symbol_idx >= row.len() {
            return Err(ParseError::InvalidState);
        }
        
        Ok(row[symbol_idx].clone())
    }
    
    /// Get expected symbols for a state
    fn get_expected_symbols(&self, state: StateId) -> Vec<SymbolId> {
        let state_idx = state.0 as usize;
        let mut expected = Vec::new();
        
        if let Some(row) = self.parse_table.action_table.get(state_idx) {
            for (symbol_idx, action) in row.iter().enumerate() {
                if !matches!(action, Action::Error) {
                    expected.push(SymbolId(symbol_idx as u16));
                }
            }
        }
        
        expected
    }
}

/// Simple lexer implementation for testing
pub struct SimpleLexer {
    /// Token patterns (symbol_id, regex_pattern)
    patterns: Vec<(SymbolId, String)>,
}

impl SimpleLexer {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (SymbolId(1), r"\d+".to_string()),      // Numbers
                (SymbolId(2), r"\+".to_string()),       // Plus
                (SymbolId(3), r"-".to_string()),        // Minus
                (SymbolId(4), r"\*".to_string()),       // Multiply
                (SymbolId(5), r"/".to_string()),        // Divide
                (SymbolId(6), r"\(".to_string()),       // Left paren
                (SymbolId(7), r"\)".to_string()),       // Right paren
                (SymbolId(8), r"\s+".to_string()),      // Whitespace (ignored)
            ],
        }
    }
}

impl Lexer for SimpleLexer {
    fn next_token(&mut self, input: &[u8], position: usize) -> Option<Token> {
        // Skip whitespace
        let mut pos = position;
        while pos < input.len() && input[pos].is_ascii_whitespace() {
            pos += 1;
        }
        
        if pos >= input.len() {
            return None;
        }
        
        // Try to match each pattern
        for (symbol_id, _pattern) in &self.patterns {
            // TODO: Implement actual regex matching
            // For now, just do simple character matching
            match input[pos] {
                b'0'..=b'9' => {
                    // Match number
                    let start = pos;
                    while pos < input.len() && input[pos].is_ascii_digit() {
                        pos += 1;
                    }
                    return Some(Token {
                        symbol: *symbol_id,
                        text: input[start..pos].to_vec(),
                        start,
                        end: pos,
                    });
                }
                b'+' if *symbol_id == SymbolId(2) => {
                    return Some(Token {
                        symbol: *symbol_id,
                        text: vec![b'+'],
                        start: pos,
                        end: pos + 1,
                    });
                }
                b'-' if *symbol_id == SymbolId(3) => {
                    return Some(Token {
                        symbol: *symbol_id,
                        text: vec![b'-'],
                        start: pos,
                        end: pos + 1,
                    });
                }
                b'*' if *symbol_id == SymbolId(4) => {
                    return Some(Token {
                        symbol: *symbol_id,
                        text: vec![b'*'],
                        start: pos,
                        end: pos + 1,
                    });
                }
                _ => continue,
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parse_table = ParseTable {
            state_count: 1,
            symbol_count: 1,
            action_table: vec![vec![Action::Accept]],
            goto_table: vec![vec![StateId(0)]],
            symbol_metadata: vec![],
        };
        
        let parser = Parser::new(parse_table);
        assert_eq!(parser.state_stack.len(), 1);
        assert_eq!(parser.state_stack[0].state, StateId(0));
    }
    
    #[test]
    fn test_lexer_number() {
        let mut lexer = SimpleLexer::new();
        let input = b"123";
        
        let token = lexer.next_token(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(1));
        assert_eq!(token.text, b"123");
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 3);
    }
    
    #[test]
    fn test_lexer_operators() {
        let mut lexer = SimpleLexer::new();
        
        let token = lexer.next_token(b"+", 0).unwrap();
        assert_eq!(token.symbol, SymbolId(2));
        
        let token = lexer.next_token(b"-", 0).unwrap();
        assert_eq!(token.symbol, SymbolId(3));
        
        let token = lexer.next_token(b"*", 0).unwrap();
        assert_eq!(token.symbol, SymbolId(4));
    }
    
    #[test]
    fn test_lexer_skip_whitespace() {
        let mut lexer = SimpleLexer::new();
        let input = b"  123";
        
        let token = lexer.next_token(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(1));
        assert_eq!(token.start, 2);
        assert_eq!(token.text, b"123");
    }
}