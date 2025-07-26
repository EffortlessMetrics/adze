// Enhanced parser with full reduction support
// This implements a complete LR parser with grammar-aware reductions

use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, RuleId, StateId, SymbolId};
use std::collections::HashMap;

/// Enhanced parser that knows about grammar rules
pub struct ParserV2 {
    /// The grammar being parsed
    #[allow(dead_code)]
    grammar: Grammar,
    /// Parse table for the grammar
    parse_table: ParseTable,
    /// Map from rule ID to rule information
    rule_map: HashMap<RuleId, Rule>,
    /// Stack of parser states
    state_stack: Vec<StateId>,
    /// Stack of parse nodes
    node_stack: Vec<ParseNode>,
    /// Current position in input
    position: usize,
}

/// A node in the parse tree
#[derive(Debug, Clone)]
pub struct ParseNode {
    /// Symbol ID for this node
    pub symbol: SymbolId,
    /// Rule ID if this is a non-terminal
    pub rule_id: Option<RuleId>,
    /// Child nodes
    pub children: Vec<ParseNode>,
    /// Start byte offset
    pub start_byte: usize,
    /// End byte offset
    pub end_byte: usize,
    /// Node text (for terminals)
    pub text: Option<Vec<u8>>,
}

impl ParseNode {
    /// Create a terminal node
    pub fn terminal(symbol: SymbolId, text: Vec<u8>, start: usize, end: usize) -> Self {
        Self {
            symbol,
            rule_id: None,
            children: vec![],
            start_byte: start,
            end_byte: end,
            text: Some(text),
        }
    }
    
    /// Create a non-terminal node
    pub fn non_terminal(
        symbol: SymbolId,
        rule_id: RuleId,
        children: Vec<ParseNode>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            symbol,
            rule_id: Some(rule_id),
            children,
            start_byte: start,
            end_byte: end,
            text: None,
        }
    }
    
    /// Get the symbol name if available
    pub fn symbol_name<'a>(&self, grammar: &'a Grammar) -> Option<&'a str> {
        // Try tokens first
        if let Some(token) = grammar.tokens.get(&self.symbol) {
            return Some(&token.name);
        }
        
        // Then try rules
        if let Some(rules) = grammar.rules.get(&self.symbol) {
            // Use the first rule's lhs symbol name if available
            if let Some(rule) = rules.first() {
                return grammar.tokens.get(&rule.lhs)
                    .map(|t| t.name.as_str());
            }
        }
        
        None
    }
}

/// Token from the lexer
#[derive(Debug, Clone)]
pub struct Token {
    pub symbol: SymbolId,
    pub text: Vec<u8>,
    pub start: usize,
    pub end: usize,
}

/// Parse error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: Vec<SymbolId>,
        found: SymbolId,
        position: usize,
    },
    InvalidState,
    InvalidRule(RuleId),
}

impl ParserV2 {
    /// Create a new parser
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        // Build rule map for quick lookup
        let mut rule_map = HashMap::new();
        let mut rule_counter = 0u16;
        for (_symbol_id, rules) in &grammar.rules {
            for rule in rules {
                // Create a unique rule ID for each rule
                let rule_id = RuleId(rule_counter);
                rule_counter += 1;
                rule_map.insert(rule_id, rule.clone());
            }
        }
        
        Self {
            grammar,
            parse_table,
            rule_map,
            state_stack: vec![StateId(0)], // Start state
            node_stack: Vec::new(),
            position: 0,
        }
    }
    
    /// Parse input tokens
    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<ParseNode, ParseError> {
        self.state_stack.clear();
        self.state_stack.push(StateId(0));
        self.node_stack.clear();
        self.position = 0;
        
        // Add EOF token at the end
        let mut tokens = tokens;
        let last_pos = tokens.last().map(|t| t.end).unwrap_or(0);
        tokens.push(Token {
            symbol: SymbolId(0), // EOF
            text: vec![],
            start: last_pos,
            end: last_pos,
        });
        
        let mut token_index = 0;
        
        loop {
            let current_state = *self.state_stack.last()
                .ok_or(ParseError::InvalidState)?;
            
            let token = &tokens[token_index];
            let action = self.get_action(current_state, token.symbol)?;
            
            match action {
                Action::Shift(next_state) => {
                    // Create terminal node and push
                    let node = ParseNode::terminal(
                        token.symbol,
                        token.text.clone(),
                        token.start,
                        token.end,
                    );
                    self.node_stack.push(node);
                    self.state_stack.push(next_state);
                    token_index += 1;
                }
                
                Action::Reduce(rule_id) => {
                    // Perform reduction
                    self.reduce(rule_id)?;
                    // Don't advance token - we'll check the same token again
                }
                
                Action::Accept => {
                    // Success! Return the root node
                    return self.node_stack.pop()
                        .ok_or(ParseError::InvalidState);
                }
                
                Action::Error => {
                    return Err(ParseError::UnexpectedToken {
                        expected: self.get_expected_symbols(current_state),
                        found: token.symbol,
                        position: token.start,
                    });
                }
                
                Action::Fork(_) => {
                    // TODO: Implement GLR fork handling
                    return Err(ParseError::InvalidState);
                }
            }
        }
    }
    
    /// Perform a reduction
    fn reduce(&mut self, rule_id: RuleId) -> Result<(), ParseError> {
        let rule = self.rule_map.get(&rule_id)
            .ok_or(ParseError::InvalidRule(rule_id))?;
        
        // Pop nodes for each symbol in the rule's RHS
        let rhs_len = rule.rhs.len();
        let mut children = Vec::with_capacity(rhs_len);
        
        // Pop in reverse order to maintain correct child order
        for _ in 0..rhs_len {
            children.push(self.node_stack.pop()
                .ok_or(ParseError::InvalidState)?);
        }
        children.reverse();
        
        // Pop corresponding states
        for _ in 0..rhs_len {
            self.state_stack.pop();
        }
        
        // Get the goto state for the LHS symbol
        let current_state = *self.state_stack.last()
            .ok_or(ParseError::InvalidState)?;
        let goto_state = self.get_goto(current_state, rule.lhs)?;
        
        // Create non-terminal node
        let start_byte = children.first()
            .map(|n| n.start_byte)
            .unwrap_or(self.position);
        let end_byte = children.last()
            .map(|n| n.end_byte)
            .unwrap_or(self.position);
        
        let node = ParseNode::non_terminal(
            rule.lhs,
            rule_id,
            children,
            start_byte,
            end_byte,
        );
        
        // Push new node and state
        self.node_stack.push(node);
        self.state_stack.push(goto_state);
        
        Ok(())
    }
    
    /// Get action for state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action, ParseError> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;
        
        self.parse_table.action_table
            .get(state_idx)
            .and_then(|row| row.get(symbol_idx))
            .cloned()
            .ok_or(ParseError::InvalidState)
    }
    
    /// Get goto state
    fn get_goto(&self, state: StateId, symbol: SymbolId) -> Result<StateId, ParseError> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;
        
        self.parse_table.goto_table
            .get(state_idx)
            .and_then(|row| row.get(symbol_idx))
            .copied()
            .ok_or(ParseError::InvalidState)
    }
    
    /// Get expected symbols for error reporting
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::TokenPattern;
    
    fn create_simple_grammar() -> Grammar {
        // Create a simple arithmetic grammar
        // E -> E + T | T
        // T -> T * F | F  
        // F -> ( E ) | num
        
        let mut grammar = Grammar::new("arithmetic".to_string());
        
        // Add tokens
        grammar.tokens.insert(
            SymbolId(1),
            rust_sitter_ir::Token {
                name: "num".to_string(),
                pattern: TokenPattern::Regex(r"\d+".to_string()),
                fragile: false,
            },
        );
        
        grammar.tokens.insert(
            SymbolId(2),
            rust_sitter_ir::Token {
                name: "+".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );
        
        grammar.tokens.insert(
            SymbolId(3),
            rust_sitter_ir::Token {
                name: "*".to_string(),
                pattern: TokenPattern::String("*".to_string()),
                fragile: false,
            },
        );
        
        // Add rules
        // E -> E + T (symbol 10)
        grammar.rules.insert(
            SymbolId(10),
            Rule {
                lhs: SymbolId(10), // E
                rhs: vec![
                    Symbol::NonTerminal(SymbolId(10)), // E
                    Symbol::Terminal(SymbolId(2)),      // +
                    Symbol::NonTerminal(SymbolId(11)), // T
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: Default::default(),
            },
        );
        
        grammar
    }
    
    #[test]
    fn test_parse_node_creation() {
        let terminal = ParseNode::terminal(
            SymbolId(1),
            b"123".to_vec(),
            0,
            3,
        );
        
        assert_eq!(terminal.symbol, SymbolId(1));
        assert_eq!(terminal.text, Some(b"123".to_vec()));
        assert!(terminal.children.is_empty());
        assert!(terminal.rule_id.is_none());
    }
    
    #[test]
    fn test_non_terminal_node() {
        let child1 = ParseNode::terminal(SymbolId(1), b"1".to_vec(), 0, 1);
        let child2 = ParseNode::terminal(SymbolId(2), b"+".to_vec(), 1, 2);
        let child3 = ParseNode::terminal(SymbolId(1), b"2".to_vec(), 2, 3);
        
        let non_terminal = ParseNode::non_terminal(
            SymbolId(10),
            RuleId(10),
            vec![child1, child2, child3],
            0,
            3,
        );
        
        assert_eq!(non_terminal.symbol, SymbolId(10));
        assert_eq!(non_terminal.rule_id, Some(RuleId(10)));
        assert_eq!(non_terminal.children.len(), 3);
        assert!(non_terminal.text.is_none());
    }
}