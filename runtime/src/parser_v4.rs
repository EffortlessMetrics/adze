// Enhanced Pure-Rust parser with external scanner support
// This module extends parser_v3 with full external scanner integration

use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, StateId, SymbolId, RuleId, TokenPattern};
use crate::lexer::{GrammarLexer, Token as LexerToken};
use crate::external_scanner::ExternalScannerRuntime;
use crate::scanner_registry::{DynExternalScanner, get_global_registry};
use anyhow::{Result, bail};
use std::collections::HashSet;

// Re-export types from parser_v3
pub use crate::parser_v3::{ParseNode, ParseError, ParserState};

/// Enhanced parser with external scanner support
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
    /// External scanner instance
    external_scanner: Option<Box<dyn DynExternalScanner>>,
    /// External scanner runtime
    external_runtime: Option<ExternalScannerRuntime>,
    /// Language name for scanner registry lookup
    language: String,
}

impl Parser {
    /// Create a new parser with the given grammar and parse table
    pub fn new(grammar: Grammar, parse_table: ParseTable, language: String) -> Self {
        // Check if grammar has external tokens
        let (external_scanner, external_runtime) = if !grammar.externals.is_empty() {
            // Get scanner from registry
            let registry = get_global_registry();
            let registry = registry.lock().unwrap();
            
            if let Some(scanner) = registry.create_scanner(&language) {
                let external_tokens: Vec<SymbolId> = grammar.externals
                    .iter()
                    .map(|ext| ext.symbol_id)
                    .collect();
                let runtime = ExternalScannerRuntime::new(external_tokens);
                (Some(scanner), Some(runtime))
            } else {
                eprintln!("Warning: Grammar has external tokens but no scanner registered for language '{}'", language);
                (None, None)
            }
        } else {
            (None, None)
        };
        
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
            external_scanner,
            external_runtime,
            language,
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
            
            // Try external scanner first if we have one
            let token = if let Some(external_token) = self.try_external_scanner(current_state)? {
                external_token
            } else {
                // Fall back to regular lexer
                match lexer.next_token(&self.input, self.position) {
                    Some(tok) => tok,
                    None => bail!("Lexer failed to produce token at position {}", self.position),
                }
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
    
    /// Try to scan for external tokens
    fn try_external_scanner(&mut self, current_state: StateId) -> Result<Option<LexerToken>> {
        // Check if we have external scanner
        let (scanner, runtime) = match (&mut self.external_scanner, &mut self.external_runtime) {
            (Some(s), Some(r)) => (s, r),
            _ => return Ok(None),
        };
        
        // Compute valid external tokens for this state
        let valid_externals = self.compute_valid_externals(current_state)?;
        
        if valid_externals.is_empty() {
            return Ok(None);
        }
        
        // Try to scan
        if let Some((symbol, length)) = runtime.scan(
            scanner.as_mut(),
            &valid_externals,
            &self.input,
            self.position,
        ) {
            // Extract token text
            let end = self.position + length;
            let text = if end <= self.input.len() {
                self.input[self.position..end].to_vec()
            } else {
                Vec::new()
            };
            
            Ok(Some(LexerToken {
                symbol,
                text,
                start: self.position,
                end,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Compute which external tokens are valid in the given state
    fn compute_valid_externals(&self, state: StateId) -> Result<HashSet<SymbolId>> {
        let mut valid_externals = HashSet::new();
        
        // Get all valid symbols for this state from the parse table
        if let Some(state_actions) = self.parse_table.states.get(&state) {
            for (symbol_id, _) in &state_actions.actions {
                // Check if this is an external symbol by comparing with grammar externals
                if self.grammar.externals.iter().any(|ext| ext.symbol_id == *symbol_id) {
                    valid_externals.insert(*symbol_id);
                }
            }
        }
        
        Ok(valid_externals)
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
        
        self.node_stack.push(new_node);
        
        // Get goto state
        let goto_state = self.get_goto_state()?;
        self.state_stack.push(ParserState {
            state: goto_state,
            symbol: Some(rule_lhs),
            position: end_byte,
        });
        
        Ok(())
    }
    
    /// Get the goto state after a reduction
    fn get_goto_state(&self) -> Result<StateId> {
        let current_state = self.state_stack.last()
            .ok_or_else(|| anyhow::anyhow!("State stack is empty"))?
            .state;
        
        let reduced_symbol = self.node_stack.last()
            .ok_or_else(|| anyhow::anyhow!("Node stack is empty"))?
            .symbol;
        
        // Look up goto action
        if let Some(state_actions) = self.parse_table.states.get(&current_state) {
            if let Some(action) = state_actions.actions.get(&reduced_symbol) {
                if let Action::Shift(goto_state) = action {
                    return Ok(*goto_state);
                }
            }
        }
        
        bail!("No goto action for symbol {:?} in state {:?}", reduced_symbol, current_state)
    }
    
    /// Handle any action (used for fork resolution)
    fn handle_action(&mut self, action: Action, token: LexerToken) -> Result<ParseNode> {
        match action {
            Action::Shift(next_state) => {
                self.handle_shift(next_state, token)?;
                self.parse(&String::from_utf8_lossy(&self.input))
            }
            Action::Reduce(rule_id) => {
                self.handle_reduce(rule_id)?;
                self.parse(&String::from_utf8_lossy(&self.input))
            }
            Action::Accept => {
                self.node_stack.pop()
                    .ok_or_else(|| anyhow::anyhow!("No parse tree on accept"))
            }
            _ => bail!("Cannot handle action: {:?}", action),
        }
    }
    
    /// Get action from parse table
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action> {
        if let Some(state_actions) = self.parse_table.states.get(&state) {
            if let Some(action) = state_actions.actions.get(&symbol) {
                return Ok(action.clone());
            }
        }
        
        // No action found - this is an error
        Ok(Action::Error)
    }
    
    /// Get expected symbols for error reporting
    fn get_expected_symbols(&self, state: StateId) -> Vec<SymbolId> {
        if let Some(state_actions) = self.parse_table.states.get(&state) {
            state_actions.actions.keys()
                .filter(|&&sym| {
                    // Include only terminals and external tokens
                    // Check if it's a token or external
                    self.grammar.tokens.contains_key(&sym) || 
                    self.grammar.externals.iter().any(|ext| ext.symbol_id == sym)
                })
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Find a rule by its ID
    fn find_rule_by_id(&self, rule_id: RuleId) -> Result<&Rule> {
        for rule in &self.grammar.rules {
            if rule.id == rule_id {
                return Ok(rule);
            }
        }
        bail!("Rule with ID {:?} not found", rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner_registry::ExternalScannerBuilder;
    use crate::scanners::IndentationScanner;
    
    #[test]
    fn test_parser_with_external_scanner() {
        // Register an indentation scanner
        ExternalScannerBuilder::new("test_python")
            .register_rust::<IndentationScanner>();
        
        // Create a simple grammar with external tokens
        let mut grammar = Grammar::new("test_python".to_string());
        
        // Add external tokens
        grammar.externals.push(rust_sitter_ir::ExternalToken {
            name: "NEWLINE".to_string(),
            symbol_id: SymbolId(100),
        });
        grammar.externals.push(rust_sitter_ir::ExternalToken {
            name: "INDENT".to_string(),
            symbol_id: SymbolId(101),
        });
        grammar.externals.push(rust_sitter_ir::ExternalToken {
            name: "DEDENT".to_string(),
            symbol_id: SymbolId(102),
        });
        
        // Create a dummy parse table
        let parse_table = ParseTable::new();
        
        // Create parser
        let mut parser = Parser::new(grammar, parse_table, "test_python".to_string());
        
        // Verify external scanner was loaded
        assert!(parser.external_scanner.is_some());
        assert!(parser.external_runtime.is_some());
    }
}