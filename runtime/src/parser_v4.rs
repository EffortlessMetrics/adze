// Enhanced Pure-Rust parser with external scanner support
// This module extends parser_v3 with full external scanner integration

use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, StateId, SymbolId, RuleId, TokenPattern};
use crate::lexer::{GrammarLexer, Token as LexerToken};
use crate::external_scanner::ExternalScannerRuntime;
use crate::scanner_registry::{DynExternalScanner, get_global_registry};
use crate::glr_forest::{GLRParserState, ForestNode, PackedNode, forest_to_parse_tree};
use anyhow::{Result, bail};
use std::collections::HashSet;
use std::rc::Rc;

// Re-export types from parser_v3
pub use crate::parser_v3::{ParseNode, ParseError, ParserState};

/// Enhanced parser with external scanner support
pub struct Parser {
    /// The grammar being used
    grammar: Grammar,
    /// Parse table for the grammar
    parse_table: ParseTable,
    /// GLR parser state (replaces simple stacks)
    glr_state: GLRParserState,
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
            glr_state: GLRParserState::new(),
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
        self.glr_state = GLRParserState::new();
        
        // Create a lexer from the grammar
        let token_patterns: Vec<(SymbolId, TokenPattern, i32)> = self.grammar.tokens
            .iter()
            .map(|(&id, token)| (id, token.pattern.clone(), 0))
            .collect();
        
        let mut lexer = GrammarLexer::new(&token_patterns);
        
        // Main GLR parse loop
        loop {
            // Process all active heads
            let mut new_active_heads = Vec::new();
            let mut accepted_forest = None;
            
            // Get next token (same for all heads)
            let token = self.get_next_token(&mut lexer)?;
            
            // Process each active GSS head
            let active_heads = self.glr_state.active_heads.clone();
            for head_idx in active_heads {
                let current_state = self.glr_state.gss_nodes[head_idx].state;
                
                // Look up action in parse table
                let action = self.get_action(StateId(current_state as u16), token.symbol)?;
                
                match action {
                    Action::Shift(next_state) => {
                        let new_head = self.handle_glr_shift(head_idx, next_state, token.clone())?;
                        new_active_heads.push(new_head);
                    }
                    
                    Action::Reduce(rule_id) => {
                        let new_heads = self.handle_glr_reduce(head_idx, rule_id)?;
                        new_active_heads.extend(new_heads);
                    }
                    
                    Action::Accept => {
                        // Found a valid parse!
                        accepted_forest = Some(self.build_final_tree(head_idx)?);
                    }
                    
                    Action::Error => {
                        // This head dies, don't add to new_active_heads
                    }
                    
                    Action::Fork(actions) => {
                        // Handle each forked action
                        for fork_action in actions {
                            match fork_action {
                                Action::Shift(next_state) => {
                                    let new_head = self.handle_glr_shift(head_idx, next_state, token.clone())?;
                                    new_active_heads.push(new_head);
                                }
                                Action::Reduce(rule_id) => {
                                    let new_heads = self.handle_glr_reduce(head_idx, rule_id)?;
                                    new_active_heads.extend(new_heads);
                                }
                                _ => {} // Ignore other actions in fork
                            }
                        }
                    }
                }
            }
            
            // Check if we accepted
            if let Some(forest) = accepted_forest {
                return Ok(forest_to_parse_tree(&forest));
            }
            
            // Check if all heads died
            if new_active_heads.is_empty() {
                bail!(ParseError::UnexpectedToken {
                    expected: vec![],
                    found: token.symbol,
                    position: self.position,
                });
            }
            
            // Update active heads
            self.glr_state.active_heads = new_active_heads;
            
            // Advance position if we shifted
            if matches!(self.get_action(StateId(self.glr_state.gss_nodes[self.glr_state.active_heads[0]].state as u16), token.symbol)?, 
                       Action::Shift(_) | Action::Fork(_)) {
                self.position += token.text.len();
            }
        }
    }
    
    /// Try to scan for external tokens
    fn try_external_scanner(&mut self, current_state: StateId) -> Result<Option<LexerToken>> {
        // Compute valid external tokens for this state first (before mutable borrow)
        let valid_externals = self.compute_valid_externals(current_state)?;
        
        if valid_externals.is_empty() {
            return Ok(None);
        }
        
        // Check if we have external scanner
        let (scanner, runtime) = match (&mut self.external_scanner, &mut self.external_runtime) {
            (Some(s), Some(r)) => (s, r),
            _ => return Ok(None),
        };
        
        // Convert valid externals to bool array
        let valid_symbols: Vec<bool> = runtime.get_external_tokens()
            .iter()
            .map(|token| valid_externals.contains(token))
            .collect();
        
        // Try to scan
        if let Some(result) = scanner.scan(
            &valid_symbols,
            &self.input,
            self.position,
        ) {
            // Extract token text
            let end = self.position + result.length;
            let text = if end <= self.input.len() {
                self.input[self.position..end].to_vec()
            } else {
                Vec::new()
            };
            
            Ok(Some(LexerToken {
                symbol: result.symbol,
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
        let state_idx = state.0 as usize;
        if state_idx < self.parse_table.action_table.len() {
            let state_actions = &self.parse_table.action_table[state_idx];
            // Check each action in the state
            for (idx, _action) in state_actions.iter().enumerate() {
                if let Some(&symbol_id) = self.parse_table.symbol_to_index.iter()
                    .find_map(|(sym, &i)| if i == idx { Some(sym) } else { None }) {
                    // Check if this is an external symbol by comparing with grammar externals
                    if self.grammar.externals.iter().any(|ext| ext.symbol_id == symbol_id) {
                        valid_externals.insert(symbol_id);
                    }
                }
            }
        }
        
        Ok(valid_externals)
    }
    
    
    
    
    
    /// Get action from parse table
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action> {
        let state_idx = state.0 as usize;
        if state_idx < self.parse_table.action_table.len() {
            if let Some(&symbol_idx) = self.parse_table.symbol_to_index.get(&symbol) {
                if symbol_idx < self.parse_table.action_table[state_idx].len() {
                    return Ok(self.parse_table.action_table[state_idx][symbol_idx].clone());
                }
            }
        }
        
        // No action found - this is an error
        Ok(Action::Error)
    }
    
    /// Get expected symbols for error reporting
    fn get_expected_symbols(&self, state: StateId) -> Vec<SymbolId> {
        let state_idx = state.0 as usize;
        let mut expected = Vec::new();
        
        if state_idx < self.parse_table.action_table.len() {
            let state_actions = &self.parse_table.action_table[state_idx];
            
            // Iterate through all symbols to find ones with non-error actions
            for (&symbol_id, &idx) in &self.parse_table.symbol_to_index {
                if idx < state_actions.len() {
                    match &state_actions[idx] {
                        Action::Error => continue,
                        _ => {
                            // Include only terminals and external tokens
                            if self.grammar.tokens.contains_key(&symbol_id) || 
                               self.grammar.externals.iter().any(|ext| ext.symbol_id == symbol_id) {
                                expected.push(symbol_id);
                            }
                        }
                    }
                }
            }
        }
        
        expected
    }
    
    /// Find a rule by its ID
    fn find_rule_by_id(&self, rule_id: RuleId) -> Result<&Rule> {
        // Rules are stored per symbol, need to search all of them
        for (_, rules) in &self.grammar.rules {
            for rule in rules {
                if rule.production_id.0 == rule_id.0 {
                    return Ok(rule);
                }
            }
        }
        bail!("Rule with ID {:?} not found", rule_id)
    }
    
    // GLR-specific methods
    
    /// Get next token (handles external scanner)
    fn get_next_token(&mut self, lexer: &mut GrammarLexer) -> Result<LexerToken> {
        // Try external scanner on first active head
        if !self.glr_state.active_heads.is_empty() {
            let current_state = StateId(self.glr_state.gss_nodes[self.glr_state.active_heads[0]].state as u16);
            if let Some(external_token) = self.try_external_scanner(current_state)? {
                return Ok(external_token);
            }
        }
        
        // Fall back to regular lexer
        match lexer.next_token(&self.input, self.position) {
            Some(tok) => Ok(tok),
            None => bail!("Lexer failed to produce token at position {}", self.position),
        }
    }
    
    /// Handle shift in GLR mode
    fn handle_glr_shift(&mut self, gss_idx: usize, next_state: StateId, token: LexerToken) -> Result<usize> {
        // Create terminal forest node
        let terminal_node = self.glr_state.get_or_create_forest_node(
            token.symbol,
            token.start,
            token.end,
            || ForestNode::Terminal {
                symbol: token.symbol,
                start: token.start,
                end: token.end,
                text: token.text.clone(),
            },
        );
        
        // Fork or reuse GSS node
        let new_gss_idx = self.glr_state.fork(gss_idx, next_state.0 as usize);
        
        // Add link from new node to parent
        self.glr_state.gss_nodes[new_gss_idx].parents.push(crate::glr_forest::GSSLink {
            parent: gss_idx,
            tree_node: terminal_node,
        });
        
        Ok(new_gss_idx)
    }
    
    /// Handle reduce in GLR mode
    fn handle_glr_reduce(&mut self, gss_idx: usize, rule_id: RuleId) -> Result<Vec<usize>> {
        // Clone the rule data we need to avoid borrow checker issues
        let (rule_lhs, rule_len) = {
            let rule = self.find_rule_by_id(rule_id)?;
            (rule.lhs, rule.rhs.len())
        };
        let mut new_heads = Vec::new();
        
        // Perform reduction starting from this GSS node
        self.perform_glr_reduce(gss_idx, rule_lhs, rule_id, rule_len, Vec::new(), &mut new_heads)?;
        
        Ok(new_heads)
    }
    
    /// Recursively perform GLR reduction
    fn perform_glr_reduce(
        &mut self,
        current_gss: usize,
        rule_lhs: SymbolId,
        rule_id: RuleId,
        remaining: usize,
        mut children: Vec<Rc<ForestNode>>,
        new_heads: &mut Vec<usize>,
    ) -> Result<()> {
        if remaining == 0 {
            // Reduction complete - create non-terminal node
            children.reverse(); // Children were collected in reverse order
            
            let start = if children.is_empty() {
                self.position
            } else {
                match children.first().unwrap().as_ref() {
                    ForestNode::Terminal { start, .. } => *start,
                    ForestNode::NonTerminal { start, .. } => *start,
                }
            };
            
            let end = if children.is_empty() {
                self.position
            } else {
                match children.last().unwrap().as_ref() {
                    ForestNode::Terminal { end, .. } => *end,
                    ForestNode::NonTerminal { end, .. } => *end,
                }
            };
            
            let packed_node = PackedNode {
                rule_id,
                children: children.clone(),
            };
            
            let forest_node = self.glr_state.merge_trees(
                rule_lhs,
                start,
                end,
                packed_node,
            );
            
            // Get goto state
            let current_state = self.glr_state.gss_nodes[current_gss].state;
            let goto_state = self.get_goto_for_state(current_state, rule_lhs)?;
            
            // Create or reuse GSS node for goto state
            let new_gss = self.glr_state.fork(current_gss, goto_state);
            self.glr_state.gss_nodes[new_gss].parents.push(crate::glr_forest::GSSLink {
                parent: current_gss,
                tree_node: forest_node,
            });
            
            new_heads.push(new_gss);
        } else {
            // Continue reduction - follow all parent links
            let parents = self.glr_state.gss_nodes[current_gss].parents.clone();
            for link in parents {
                let mut new_children = children.clone();
                new_children.push(link.tree_node.clone());
                self.perform_glr_reduce(
                    link.parent,
                    rule_lhs,
                    rule_id,
                    remaining - 1,
                    new_children,
                    new_heads,
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Get goto state for a given state and symbol
    fn get_goto_for_state(&self, state: usize, symbol: SymbolId) -> Result<usize> {
        if state < self.parse_table.goto_table.len() {
            if let Some(&symbol_idx) = self.parse_table.symbol_to_index.get(&symbol) {
                if symbol_idx < self.parse_table.goto_table[state].len() {
                    let goto_state = self.parse_table.goto_table[state][symbol_idx];
                    if goto_state != StateId(0) { // 0 typically means no transition
                        return Ok(goto_state.0 as usize);
                    }
                }
            }
        }
        bail!("No goto action for symbol {:?} in state {}", symbol, state)
    }
    
    /// Build final tree from accepted GSS node
    fn build_final_tree(&self, gss_idx: usize) -> Result<ForestNode> {
        // Find the path from this node to the start
        let mut current = gss_idx;
        let mut nodes = Vec::new();
        
        while !self.glr_state.gss_nodes[current].parents.is_empty() {
            let link = &self.glr_state.gss_nodes[current].parents[0];
            nodes.push(link.tree_node.clone());
            current = link.parent;
        }
        
        // The last node should be the root of the parse tree
        if let Some(root) = nodes.last() {
            Ok(root.as_ref().clone())
        } else {
            bail!("No parse tree found")
        }
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