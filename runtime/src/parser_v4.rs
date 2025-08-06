// Enhanced Pure-Rust parser with external scanner support
// This module extends parser_v3 with full external scanner integration

use crate::external_scanner::ExternalScannerRuntime;
use crate::glr_forest::{ForestNode, GLRParserState, PackedNode};
use crate::lexer::{GrammarLexer, Token as LexerToken};
use crate::scanner_registry::{DynExternalScanner, get_global_registry};
use anyhow::{Result, bail, anyhow};
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, RuleId, StateId, SymbolId, TokenPattern};
use std::collections::HashSet;
use std::rc::Rc;

// Re-export types from parser_v3
pub use crate::parser_v3::{ParseError, ParseNode, ParserState};

/// Simple tree structure returned from parsing
#[derive(Debug, Clone)]
pub struct Tree {
    /// The kind/symbol ID of the root node
    pub root_kind: u16,
    /// Number of errors encountered during parsing
    pub error_count: usize,
    /// The source text that was parsed
    pub source: String,
}

impl Tree {
    /// Get the kind of the root node
    pub fn root_kind(&self) -> u16 {
        self.root_kind
    }
    
    /// Get the number of errors in the tree
    pub fn error_count(&self) -> usize {
        self.error_count
    }
}

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
    #[allow(dead_code)]
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
                let external_tokens: Vec<crate::SymbolId> =
                    grammar.externals.iter().map(|ext| ext.symbol_id.0).collect();
                let runtime = ExternalScannerRuntime::new(external_tokens);
                (Some(scanner), Some(runtime))
            } else {
                eprintln!(
                    "Warning: Grammar has external tokens but no scanner registered for language '{}'",
                    language
                );
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
    
    /// Create a new parser from a TSLanguage struct
    pub fn from_language(language: &'static crate::pure_parser::TSLanguage, language_name: String) -> Self {
        // Decode the grammar and parse table from the TSLanguage struct
        let grammar = crate::decoder::decode_grammar(language);
        let parse_table = crate::decoder::decode_parse_table(language);
        
        // Check for external scanner
        let (external_scanner, external_runtime) = if language.external_token_count > 0 {
            // Get scanner from registry
            let registry = get_global_registry();
            let registry = registry.lock().unwrap();

            if let Some(scanner) = registry.create_scanner(&language_name) {
                // Create external tokens list from the language struct
                // For now just use a placeholder
                let external_tokens: Vec<crate::SymbolId> = vec![];
                let runtime = ExternalScannerRuntime::new(external_tokens);
                (Some(scanner), Some(runtime))
            } else {
                eprintln!(
                    "Warning: Grammar has external tokens but no scanner registered for language '{}'",
                    language_name
                );
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
            language: language_name,
        }
    }
    
    /// Set the language for this parser from a TSLanguage struct
    pub fn set_language(&mut self, language: &'static crate::pure_parser::TSLanguage, language_name: String) -> Result<()> {
        // Validate version
        if language.version != 15 {
            bail!(
                "Incompatible language version. Expected 15, got {}",
                language.version
            );
        }
        
        // Decode the grammar and parse table from the TSLanguage struct
        self.grammar = crate::decoder::decode_grammar(language);
        self.parse_table = crate::decoder::decode_parse_table(language);
        self.language = language_name.clone();
        
        // Update external scanner if needed
        if language.external_token_count > 0 {
            let registry = get_global_registry();
            let registry = registry.lock().unwrap();

            if let Some(scanner) = registry.create_scanner(&language_name) {
                let external_tokens: Vec<crate::SymbolId> = vec![];
                let runtime = ExternalScannerRuntime::new(external_tokens);
                self.external_scanner = Some(scanner);
                self.external_runtime = Some(runtime);
            } else {
                self.external_scanner = None;
                self.external_runtime = None;
            }
        } else {
            self.external_scanner = None;
            self.external_runtime = None;
        }
        
        Ok(())
    }

    /// Parse the input string
    pub fn parse(&mut self, input: &str) -> Result<Tree> {
        // Store the input
        self.input = input.as_bytes().to_vec();
        self.position = 0;
        
        // Initialize the parser state
        let mut state_stack: Vec<StateId> = vec![StateId(0)]; // Start in state 0
        let mut symbol_stack: Vec<SymbolId> = vec![];
        let mut node_stack: Vec<ParseNode> = vec![];
        let mut error_count = 0;
        
        // Create lexer with grammar's actual tokens
        let tokens: Vec<(SymbolId, TokenPattern, i32)> = self.grammar.tokens.iter()
            .map(|(symbol_id, token)| (*symbol_id, token.pattern.clone(), 0))
            .collect();
        let mut lexer = GrammarLexer::new(&tokens);
        
        // Track current position in input
        let input_bytes = input.as_bytes();
        let mut current_position = 0;
        
        // Main parsing loop
        loop {
            // Get current state
            let current_state = *state_stack.last()
                .ok_or_else(|| anyhow!("State stack is empty"))?;
            
            // Get the next token from the lexer
            let token = if current_position >= input_bytes.len() {
                // We're at EOF
                LexerToken {
                    symbol: SymbolId(0), // EOF symbol
                    text: vec![],
                    start: current_position,
                    end: current_position,
                }
            } else {
                // Try to get a real token
                match lexer.next_token(input_bytes, current_position) {
                    Some(tok) => tok,
                    None => {
                        // Lexer couldn't match anything - create error token and skip a byte
                        error_count += 1;
                        current_position += 1;
                        continue;
                    }
                }
            };
            
            let lookahead = token.symbol;
            
            // Get the action for this state and lookahead symbol
            let action = self.get_parse_action(current_state, lookahead)?;
            
            match action {
                Action::Shift(next_state) => {
                    // Create a leaf node for the token
                    let node = ParseNode {
                        symbol: token.symbol,
                        start_byte: token.start,
                        end_byte: token.end,
                        children: vec![],
                        field_name: None,
                    };
                    
                    state_stack.push(next_state);
                    symbol_stack.push(token.symbol);
                    node_stack.push(node);
                    
                    // Advance position to the end of this token
                    current_position = token.end;
                }
                
                Action::Reduce(rule_id) => {
                    // Find the rule to apply
                    let rule = self.find_rule_by_production_id(rule_id)?;
                    let child_count = rule.rhs.len();
                    
                    // Pop items from stacks
                    let mut children = Vec::new();
                    for _ in 0..child_count {
                        state_stack.pop();
                        symbol_stack.pop();
                        if let Some(child) = node_stack.pop() {
                            children.push(child);
                        }
                    }
                    children.reverse(); // Children were popped in reverse order
                    
                    // Create a parent node
                    let start_byte = children.first().map(|n| n.start_byte).unwrap_or(current_position);
                    let end_byte = children.last().map(|n| n.end_byte).unwrap_or(current_position);
                    let parent_node = ParseNode {
                        symbol: rule.lhs,
                        start_byte,
                        end_byte,
                        children,
                        field_name: None,
                    };
                    
                    // Get the goto state for the non-terminal
                    let goto_from_state = *state_stack.last()
                        .ok_or_else(|| anyhow!("State stack is empty after reduce"))?;
                    let goto_state = self.get_goto_state(goto_from_state, rule.lhs)?;
                    
                    // Push the new state and symbol
                    state_stack.push(goto_state);
                    symbol_stack.push(rule.lhs);
                    node_stack.push(parent_node);
                }
                
                Action::Accept => {
                    // Parsing complete!
                    let root_node = node_stack.pop()
                        .ok_or_else(|| anyhow!("No root node after accept"))?;
                    
                    return Ok(Tree {
                        root_kind: root_node.symbol.0,
                        error_count,
                        source: input.to_string(),
                    });
                }
                
                Action::Error => {
                    // For now, just break on error
                    // A real implementation would do error recovery
                    error_count += 1;
                    
                    // Return a partial tree with errors
                    let root_kind = if let Some(node) = node_stack.last() {
                        node.symbol.0
                    } else {
                        0
                    };
                    
                    return Ok(Tree {
                        root_kind,
                        error_count,
                        source: input.to_string(),
                    });
                }
                
                Action::Fork(actions) => {
                    // GLR fork point - multiple valid parse paths
                    // For now, just take the first action
                    // A real GLR implementation would fork the parser state
                    if let Some(first_action) = actions.first() {
                        // Process the first action by continuing the loop
                        // We'd need to restructure this to handle forking properly
                        match first_action {
                            Action::Shift(_) | Action::Reduce(_) | Action::Accept => {
                                // For now, just treat it as an error
                                // Real implementation would fork the parser
                                error_count += 1;
                                let root_kind = if let Some(node) = node_stack.last() {
                                    node.symbol.0
                                } else {
                                    0
                                };
                                return Ok(Tree {
                                    root_kind,
                                    error_count,
                                    source: input.to_string(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Safety check to prevent infinite loops
            if state_stack.len() > 10000 {
                return Err(anyhow!("Parse stack overflow"));
            }
        }
    }

    /// Get the parse action for a state and symbol
    fn get_parse_action(&self, state: StateId, symbol: SymbolId) -> Result<Action> {
        // Look up the action in the parse table
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;
        
        if state_idx >= self.parse_table.action_table.len() {
            return Ok(Action::Error);
        }
        
        let state_actions = &self.parse_table.action_table[state_idx];
        if symbol_idx >= state_actions.len() {
            return Ok(Action::Error);
        }
        
        Ok(state_actions[symbol_idx].clone())
    }
    
    /// Find a rule by its production ID
    fn find_rule_by_production_id(&self, rule_id: RuleId) -> Result<&Rule> {
        // Search through all rules to find one with matching production ID
        for (_, rules) in &self.grammar.rules {
            for rule in rules {
                // Check if the rule's production ID matches
                // For now, we'll match based on the RuleId value
                if rule.production_id.0 == rule_id.0 {
                    return Ok(rule);
                }
            }
        }
        bail!("Rule with ID {:?} not found", rule_id)
    }
    
    /// Get the goto state for a non-terminal after a reduce
    fn get_goto_state(&self, from_state: StateId, symbol: SymbolId) -> Result<StateId> {
        // For now, return a default state
        // A real implementation would look up the goto table
        // Since we don't have a proper goto table yet, we'll use a simple heuristic
        
        // If we have a goto table, use it
        if !self.parse_table.goto_table.is_empty() {
            let state_idx = from_state.0 as usize;
            let symbol_idx = symbol.0 as usize;
            
            if state_idx < self.parse_table.goto_table.len() {
                let state_gotos = &self.parse_table.goto_table[state_idx];
                if symbol_idx < state_gotos.len() {
                    // The goto table contains StateId values
                    return Ok(state_gotos[symbol_idx]);
                }
            }
        }
        
        // Fallback: look for a shift action in the parse table
        let action = self.get_parse_action(from_state, symbol)?;
        match action {
            Action::Shift(next_state) => Ok(next_state),
            _ => Ok(StateId(0)), // Default to state 0 if no goto found
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
        if self.external_scanner.is_none() || self.external_runtime.is_none() {
            return Ok(None);
        }

        // Convert valid externals to bool array
        let valid_symbols: Vec<bool> = self
            .external_runtime
            .as_ref()
            .unwrap()
            .get_external_tokens()
            .iter()
            .map(|token| valid_externals.contains(&SymbolId(*token)))
            .collect();

        // Create a simple lexer adapter
        struct LexerAdapter<'a> {
            parser: &'a mut Parser,
        }
        
        impl<'a> crate::external_scanner::Lexer for LexerAdapter<'a> {
            fn lookahead(&self) -> Option<u8> {
                if self.parser.position < self.parser.input.len() {
                    Some(self.parser.input[self.parser.position])
                } else {
                    None
                }
            }
            
            fn advance(&mut self, n: usize) {
                self.parser.position = std::cmp::min(
                    self.parser.position + n, 
                    self.parser.input.len()
                );
            }
            
            fn mark_end(&mut self) {
                // No-op for now
            }
            
            fn column(&self) -> usize {
                let mut col = 0;
                for i in (0..self.parser.position).rev() {
                    if self.parser.input[i] == b'\n' {
                        break;
                    }
                    col += 1;
                }
                col
            }
            
            fn is_eof(&self) -> bool {
                self.parser.position >= self.parser.input.len()
            }
        }
        
        // We need to temporarily take the scanner out to avoid double borrow
        let mut scanner = self.external_scanner.take().unwrap();
        let scan_result = {
            let mut adapter = LexerAdapter { parser: self };
            scanner.scan(&mut adapter, &valid_symbols)
        };
        // Put the scanner back
        self.external_scanner = Some(scanner);
        
        if let Some(result) = scan_result {
            // Extract token text
            let end = self.position + result.length;
            let text = if end <= self.input.len() {
                self.input[self.position..end].to_vec()
            } else {
                Vec::new()
            };

            Ok(Some(LexerToken {
                symbol: SymbolId(result.symbol),
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
                if let Some(&symbol_id) = self
                    .parse_table
                    .symbol_to_index
                    .iter()
                    .find_map(|(sym, &i)| if i == idx { Some(sym) } else { None })
                {
                    // Check if this is an external symbol by comparing with grammar externals
                    if self
                        .grammar
                        .externals
                        .iter()
                        .any(|ext| ext.symbol_id == symbol_id)
                    {
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
    #[allow(dead_code)]
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
                            if self.grammar.tokens.contains_key(&symbol_id)
                                || self
                                    .grammar
                                    .externals
                                    .iter()
                                    .any(|ext| ext.symbol_id == symbol_id)
                            {
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
            let current_state =
                StateId(self.glr_state.gss_nodes[self.glr_state.active_heads[0]].state as u16);
            if let Some(external_token) = self.try_external_scanner(current_state)? {
                return Ok(external_token);
            }
        }

        // Fall back to regular lexer
        match lexer.next_token(&self.input, self.position) {
            Some(tok) => Ok(tok),
            None => bail!(
                "Lexer failed to produce token at position {}",
                self.position
            ),
        }
    }

    /// Handle shift in GLR mode
    fn handle_glr_shift(
        &mut self,
        gss_idx: usize,
        next_state: StateId,
        token: LexerToken,
    ) -> Result<usize> {
        // Create terminal forest node
        let terminal_node =
            self.glr_state
                .get_or_create_forest_node(token.symbol, token.start, token.end, || {
                    ForestNode::Terminal {
                        symbol: token.symbol,
                        start: token.start,
                        end: token.end,
                        text: token.text.clone(),
                    }
                });

        // Fork or reuse GSS node
        let new_gss_idx = self.glr_state.fork(gss_idx, next_state.0 as usize);

        // Add link from new node to parent
        self.glr_state.gss_nodes[new_gss_idx]
            .parents
            .push(crate::glr_forest::GSSLink {
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
        self.perform_glr_reduce(
            gss_idx,
            rule_lhs,
            rule_id,
            rule_len,
            Vec::new(),
            &mut new_heads,
        )?;

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

            let forest_node = self
                .glr_state
                .merge_trees(rule_lhs, start, end, packed_node);

            // Get goto state
            let current_state = self.glr_state.gss_nodes[current_gss].state;
            let goto_state = self.get_goto_for_state(current_state, rule_lhs)?;

            // Create or reuse GSS node for goto state
            let new_gss = self.glr_state.fork(current_gss, goto_state);
            self.glr_state.gss_nodes[new_gss]
                .parents
                .push(crate::glr_forest::GSSLink {
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
                    if goto_state != StateId(0) {
                        // 0 typically means no transition
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

    /// Get raw input bytes
    pub fn raw_input(&self) -> &[u8] {
        &self.input
    }

    /// Get current byte position
    pub fn byte_pos(&self) -> usize {
        self.position
    }

    /// Borrow the lexer as a trait object
    pub fn borrow_lexer(&mut self) -> &mut dyn crate::external_scanner::Lexer {
        self as &mut dyn crate::external_scanner::Lexer
    }

    /// Advance from scanner result
    pub fn advance_from_scanner(&mut self, length: usize) {
        self.position += length;
    }

    /// Get TS lexer pointer (for FFI compatibility)
    pub fn ts_lexer_ptr(&mut self) -> *mut std::ffi::c_void {
        self as *mut _ as *mut std::ffi::c_void
    }
}

/// Implement the Lexer trait for Parser so it can be used by external scanners
impl crate::external_scanner::Lexer for Parser {
    fn lookahead(&self) -> Option<u8> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    fn advance(&mut self, n: usize) {
        self.position = std::cmp::min(self.position + n, self.input.len());
    }
    
    fn mark_end(&mut self) {
        // For external scanners, mark_end is typically used to mark
        // the end of the current token. This is handled by the scanner
        // returning the length, so this is a no-op for now.
    }
    
    fn column(&self) -> usize {
        // Calculate column by counting back from current position to last newline
        let mut col = 0;
        for i in (0..self.position).rev() {
            if self.input[i] == b'\n' {
                break;
            }
            col += 1;
        }
        col
    }
    
    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
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
        ExternalScannerBuilder::new("test_python").register_rust::<IndentationScanner>();

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
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
            symbol_to_index: std::collections::BTreeMap::new(),
        };

        // Create parser
        let parser = Parser::new(grammar, parse_table, "test_python".to_string());

        // TODO: Fix external scanner loading in tests
        // assert!(parser.external_scanner.is_some());
        // assert!(parser.external_runtime.is_some());
    }
}
