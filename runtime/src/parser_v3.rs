// Improved Pure-Rust parser execution engine
// This module implements the runtime parsing logic with proper reduction handling

use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, StateId, SymbolId, RuleId, Symbol, TokenPattern};
use crate::lexer::{GrammarLexer, Token as LexerToken};
use anyhow::{Result, bail};
use std::fmt;

// Re-export the lexer Token type for consistency
pub use crate::lexer::Token;

/// Parser state during execution
#[derive(Debug, Clone)]
pub struct ParserState {
    /// Current state in the parse table
    state: StateId,
    /// Symbol that led to this state
    symbol: Option<SymbolId>,
    /// Position in the input
    position: usize,
}

/// A node in the parse tree being constructed
#[derive(Debug, Clone)]
pub struct ParseNode {
    /// Symbol ID for this node
    pub symbol: SymbolId,
    /// Child nodes
    pub children: Vec<ParseNode>,
    /// Start byte offset in the input
    pub start_byte: usize,
    /// End byte offset in the input
    pub end_byte: usize,
    /// Field name if this node is a field
    pub field_name: Option<String>,
}

/// The main parser engine with Grammar awareness
pub struct Parser {
    /// The grammar being used
    grammar: Grammar,
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
    InvalidParse(String),
    /// Parser is in an invalid state
    InvalidState(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, position } => {
                write!(f, "Unexpected token at position {}: found {:?}, expected one of {:?}", 
                    position, found, expected)
            }
            ParseError::InvalidParse(msg) => write!(f, "Invalid parse: {}", msg),
            ParseError::InvalidState(msg) => write!(f, "Invalid parser state: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl Parser {
    /// Create a new parser with the given grammar and parse table
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        Self {
            grammar,
            parse_table,
            state_stack: vec![ParserState {
                state: StateId(0), // Start state
                symbol: None,
                position: 0,
            }],
            node_stack: Vec::new(),
            input: Vec::new(),
            position: 0,
        }
    }
    
    /// Parse the input string
    pub fn parse(&mut self, input: &str) -> Result<ParseNode> {
        self.input = input.as_bytes().to_vec();
        self.position = 0;
        self.state_stack.clear();
        self.state_stack.push(ParserState {
            state: StateId(0),
            symbol: None,
            position: 0,
        });
        self.node_stack.clear();
        
        // Create a lexer from the grammar
        let token_patterns: Vec<(SymbolId, TokenPattern, i32)> = self.grammar.tokens
            .iter()
            .map(|(&id, token)| (id, token.pattern.clone(), 0))
            .collect();
        
        let mut lexer = GrammarLexer::new(&token_patterns);
        
        // Main parse loop
        loop {
            // Get current state
            let current_state = self.state_stack.last()
                .ok_or_else(|| anyhow::anyhow!(ParseError::InvalidState("Empty state stack".to_string())))?
                .state;
            
            // Get next token
            let token = match lexer.next_token(&self.input, self.position) {
                Some(tok) => tok,
                None => bail!("Lexer failed to produce token at position {}", self.position),
            };
            
            // Look up action in parse table
            let action = self.get_action(current_state, token.symbol)?;
            
            match action {
                Action::Shift(next_state) => {
                    self.handle_shift(next_state, token)?;
                }
                
                Action::Reduce(rule_id) => {
                    self.handle_reduce(rule_id)?;
                    // After reduction, don't advance - re-process with the new top state
                    continue;
                }
                
                Action::Accept => {
                    // Parse complete!
                    return self.node_stack.pop()
                        .ok_or_else(|| anyhow::anyhow!(ParseError::InvalidState("No parse tree on accept".to_string())));
                }
                
                Action::Error => {
                    bail!(ParseError::UnexpectedToken {
                        expected: self.get_expected_symbols(current_state),
                        found: token.symbol,
                        position: self.position,
                    });
                }
                
                Action::Fork(actions) => {
                    // For now, take the first non-error action
                    // TODO: Implement proper GLR forking
                    for action in actions {
                        if !matches!(action, Action::Error) {
                            return self.handle_action(action, token);
                        }
                    }
                    bail!("All fork actions were errors");
                }
            }
        }
    }
    
    /// Handle a shift action
    fn handle_shift(&mut self, next_state: StateId, token: LexerToken) -> Result<()> {
        // Push token as a leaf node
        self.node_stack.push(ParseNode {
            symbol: token.symbol,
            children: vec![],
            start_byte: token.start,
            end_byte: token.end,
            field_name: None,
        });
        
        // Push new state
        self.state_stack.push(ParserState {
            state: next_state,
            symbol: Some(token.symbol),
            position: token.end,
        });
        
        // Advance position
        self.position = token.end;
        
        Ok(())
    }
    
    /// Handle a reduce action
    fn handle_reduce(&mut self, rule_id: RuleId) -> Result<()> {
        // Find the rule in the grammar and extract needed data
        let (rule_lhs, rule_rhs_len, rule_fields) = {
            let rule = self.find_rule_by_id(rule_id)?;
            (rule.lhs, rule.rhs.len(), rule.fields.clone())
        };
        
        // Pop states and nodes for the rule length
        let mut children = Vec::with_capacity(rule_rhs_len);
        
        // Collect children in reverse order (they're on stack in reverse)
        for _ in 0..rule_rhs_len {
            self.state_stack.pop()
                .ok_or_else(|| anyhow::anyhow!(ParseError::InvalidState("State stack underflow".to_string())))?;
            
            let child = self.node_stack.pop()
                .ok_or_else(|| anyhow::anyhow!(ParseError::InvalidState("Node stack underflow".to_string())))?;
            
            children.push(child);
        }
        
        // Children were collected in reverse order
        children.reverse();
        
        // Apply field names if any
        for (field_id, position) in rule_fields {
            if position < children.len() {
                if let Some(field_name) = self.grammar.fields.get(&field_id) {
                    children[position].field_name = Some(field_name.clone());
                }
            }
        }
        
        // Get position info from children
        let start_byte = children.first().map(|n| n.start_byte).unwrap_or(self.position);
        let end_byte = children.last().map(|n| n.end_byte).unwrap_or(self.position);
        
        // Create new node for the reduction
        let new_node = ParseNode {
            symbol: rule_lhs,
            children,
            start_byte,
            end_byte,
            field_name: None,
        };
        
        // Push the new node
        self.node_stack.push(new_node);
        
        // Get the state to go to after reduction
        let goto_state = self.get_goto_state(rule_lhs)?;
        
        // Push new state
        self.state_stack.push(ParserState {
            state: goto_state,
            symbol: Some(rule_lhs),
            position: end_byte,
        });
        
        Ok(())
    }
    
    /// Handle any action (used for fork resolution)
    fn handle_action(&mut self, action: Action, token: LexerToken) -> Result<ParseNode> {
        // Save the input to avoid borrowing issues
        let input_str = String::from_utf8_lossy(&self.input).to_string();
        
        match action {
            Action::Shift(state) => {
                self.handle_shift(state, token)?;
                self.parse(&input_str)
            }
            Action::Reduce(rule_id) => {
                self.handle_reduce(rule_id)?;
                self.parse(&input_str)
            }
            _ => bail!("Unexpected action in fork handling"),
        }
    }
    
    /// Find a rule by its ID
    fn find_rule_by_id(&self, rule_id: RuleId) -> Result<&Rule> {
        // Look up the rule by searching through production_ids
        for (rid, &prod_id) in &self.grammar.production_ids {
            if *rid == rule_id {
                // Find the rule with this production ID
                for rule in self.grammar.rules.values() {
                    if rule.production_id == prod_id {
                        return Ok(rule);
                    }
                }
            }
        }
        
        bail!("Rule not found for ID {:?}", rule_id)
    }
    
    /// Get the goto state after a reduction
    fn get_goto_state(&self, symbol: SymbolId) -> Result<StateId> {
        let current_state = self.state_stack.last()
            .ok_or_else(|| anyhow::anyhow!(ParseError::InvalidState("Empty state stack for goto".to_string())))?
            .state;
        
        let state_idx = current_state.0 as usize;
        let symbol_idx = symbol.0 as usize;
        
        if state_idx >= self.parse_table.goto_table.len() {
            bail!("Invalid state index: {}", state_idx);
        }
        
        let state_gotos = &self.parse_table.goto_table[state_idx];
        
        if symbol_idx >= state_gotos.len() {
            bail!("Invalid symbol index for goto: {}", symbol_idx);
        }
        
        Ok(state_gotos[symbol_idx])
    }
    
    /// Get the action for a state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;
        
        if state_idx >= self.parse_table.action_table.len() {
            bail!("Invalid state index: {}", state_idx);
        }
        
        let state_actions = &self.parse_table.action_table[state_idx];
        
        if symbol_idx >= state_actions.len() {
            bail!("Invalid symbol index: {}", symbol_idx);
        }
        
        Ok(state_actions[symbol_idx].clone())
    }
    
    /// Get expected symbols for error reporting
    fn get_expected_symbols(&self, state: StateId) -> Vec<SymbolId> {
        let state_idx = state.0 as usize;
        let mut expected = Vec::new();
        
        if state_idx < self.parse_table.action_table.len() {
            let state_actions = &self.parse_table.action_table[state_idx];
            
            for (symbol_idx, action) in state_actions.iter().enumerate() {
                if !matches!(action, Action::Error) {
                    expected.push(SymbolId(symbol_idx as u16));
                }
            }
        }
        
        expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::*;
    
    fn create_simple_grammar() -> Grammar {
        let mut grammar = Grammar {
            name: "test".to_string(),
            rules: indexmap::IndexMap::new(),
            tokens: indexmap::IndexMap::new(),
            precedences: vec![],
            conflicts: vec![],
            externals: vec![],
            fields: indexmap::IndexMap::new(),
            supertypes: vec![],
            inline_rules: vec![],
            alias_sequences: indexmap::IndexMap::new(),
            production_ids: indexmap::IndexMap::new(),
            max_alias_sequence_length: 0,
        };
        
        // Add tokens
        let num_id = SymbolId(1);
        let plus_id = SymbolId(2);
        let _eof_id = SymbolId(0);
        
        grammar.tokens.insert(num_id, rust_sitter_ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(plus_id, rust_sitter_ir::Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        });
        
        // Add rules: E -> E + E | number
        let expr_id = SymbolId(3);
        
        // Rule 0: E -> number
        let rule0 = Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(num_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        
        // Rule 1: E -> E + E
        let rule1 = Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(plus_id),
                Symbol::NonTerminal(expr_id),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        };
        
        grammar.rules.insert(expr_id, rule0.clone());
        grammar.rules.insert(expr_id, rule1.clone());
        
        grammar.production_ids.insert(RuleId(0), ProductionId(0));
        grammar.production_ids.insert(RuleId(1), ProductionId(1));
        
        grammar
    }
    
    #[test]
    fn test_simple_parse() {
        // This test would require building a parse table
        // For now, we'll just verify the parser compiles
        let grammar = create_simple_grammar();
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
        };
        
        let _parser = Parser::new(grammar, parse_table);
    }
}