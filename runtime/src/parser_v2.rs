// Enhanced parser with full reduction support
// This implements a complete LR parser with grammar-aware reductions

use rust_sitter_glr_core::{Action, ParseTable, VersionInfo, compare_versions, CompareResult};
use rust_sitter_ir::{Grammar, Rule, RuleId, StateId, SymbolId};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// A parse stack for GLR parsing
#[derive(Debug, Clone)]
struct ParseStack {
    /// Stack of parser states
    state_stack: Vec<StateId>,
    /// Stack of parse nodes
    node_stack: Vec<Arc<ParseNode>>,
    /// Version info for conflict resolution
    version: VersionInfo,
    /// Unique ID for this stack
    id: usize,
}

impl ParseStack {
    fn new(initial_state: StateId, id: usize) -> Self {
        Self {
            state_stack: vec![initial_state],
            node_stack: vec![],
            version: VersionInfo::new(),
            id,
        }
    }
    
    fn current_state(&self) -> StateId {
        *self.state_stack.last().expect("Empty state stack")
    }
}

/// Enhanced parser that knows about grammar rules
pub struct ParserV2 {
    /// The grammar being parsed
    #[allow(dead_code)]
    grammar: Grammar,
    /// Parse table for the grammar
    parse_table: ParseTable,
    /// Map from rule ID to rule information
    rule_map: HashMap<RuleId, Rule>,
    /// Active parse stacks (for GLR)
    stacks: Vec<ParseStack>,
    /// Queue of stacks to process
    pending_stacks: VecDeque<usize>,
    /// Current position in input
    position: usize,
    /// Next stack ID
    next_stack_id: usize,
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
                return grammar.tokens.get(&rule.lhs).map(|t| t.name.as_str());
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

        let initial_stack = ParseStack::new(StateId(0), 0);
        
        Self {
            grammar,
            parse_table,
            rule_map,
            stacks: vec![initial_stack],
            pending_stacks: VecDeque::from([0]),
            position: 0,
            next_stack_id: 1,
        }
    }

    /// Parse input tokens using GLR two-phase algorithm
    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<ParseNode, ParseError> {
        // Reset parser state
        self.stacks.clear();
        self.stacks.push(ParseStack::new(StateId(0), 0));
        self.pending_stacks = VecDeque::from([0]);
        self.position = 0;
        self.next_stack_id = 1;

        // Add EOF token at the end
        let mut tokens = tokens;
        let last_pos = tokens.last().map(|t| t.end).unwrap_or(0);
        tokens.push(Token {
            symbol: SymbolId(0), // EOF
            text: vec![],
            start: last_pos,
            end: last_pos,
        });

        // Process each token
        for token in tokens {
            self.process_token(token)?;
        }

        // Find the accepting stack
        for stack in &self.stacks {
            if let Some(root) = stack.node_stack.last() {
                return Ok((**root).clone());
            }
        }

        Err(ParseError::InvalidState)
    }
    
    /// Process a single token using the two-phase GLR algorithm
    fn process_token(&mut self, token: Token) -> Result<(), ParseError> {
        // Phase 1: Perform all possible reductions
        self.stacks = self.reduce_until_saturated(token.symbol);
        
        // Phase 2: Process token (shift/fork/error)
        self.process_token_phase2(token)
    }
    
    /// Phase 1: Perform all reductions until no more apply
    fn reduce_until_saturated(&mut self, lookahead: SymbolId) -> Vec<ParseStack> {
        let mut stacks = std::mem::take(&mut self.stacks);
        let mut iteration = 0;
        
        loop {
            iteration += 1;
            if iteration > 20 {
                panic!("Too many reduction iterations - possible grammar issue");
            }
            
            let mut any_reduction = false;
            let mut new_stacks = Vec::new();
            
            for stack in stacks {
                let state = stack.current_state();
                let action = self.get_action(state, lookahead).unwrap_or(Action::Error);
                
                match action {
                    Action::Reduce(rule_id) => {
                        any_reduction = true;
                        if let Some(reduced_stack) = self.reduce_stack(stack, rule_id) {
                            new_stacks.push(reduced_stack);
                        }
                    }
                    Action::Fork(ref actions) => {
                        // Check if fork contains any reductions
                        let mut non_reduce_actions = Vec::new();
                        
                        for action in actions {
                            match action {
                                Action::Reduce(rule_id) => {
                                    any_reduction = true;
                                    if let Some(reduced_stack) = self.reduce_stack(stack.clone(), *rule_id) {
                                        new_stacks.push(reduced_stack);
                                    }
                                }
                                _ => non_reduce_actions.push(action.clone()),
                            }
                        }
                        
                        // If there were non-reduce actions, keep the stack for phase 2
                        if !non_reduce_actions.is_empty() {
                            new_stacks.push(stack);
                        }
                    }
                    _ => {
                        // Not a reduction - keep stack for phase 2
                        new_stacks.push(stack);
                    }
                }
            }
            
            // Merge duplicate stacks
            stacks = self.merge_stacks(new_stacks);
            
            if !any_reduction {
                break;
            }
        }
        
        stacks
    }
    
    /// Phase 2: Process token with shift/fork/error actions
    fn process_token_phase2(&mut self, token: Token) -> Result<(), ParseError> {
        let mut new_stacks = Vec::new();
        let mut any_success = false;
        
        for stack in &self.stacks {
            let state = stack.current_state();
            let action = self.get_action(state, token.symbol).unwrap_or(Action::Error);
            
            match action {
                Action::Shift(next_state) => {
                    any_success = true;
                    let mut new_stack = stack.clone();
                    let node = Arc::new(ParseNode::terminal(
                        token.symbol,
                        token.text.clone(),
                        token.start,
                        token.end,
                    ));
                    new_stack.node_stack.push(node);
                    new_stack.state_stack.push(next_state);
                    new_stacks.push(new_stack);
                }
                Action::Accept => {
                    any_success = true;
                    new_stacks.push(stack.clone());
                }
                Action::Fork(actions) => {
                    // Process non-reduce actions from fork
                    for fork_action in &actions {
                        match fork_action {
                            Action::Shift(next_state) => {
                                any_success = true;
                                let mut forked = stack.clone();
                                forked.id = self.next_stack_id;
                                self.next_stack_id += 1;
                                
                                let node = Arc::new(ParseNode::terminal(
                                    token.symbol,
                                    token.text.clone(),
                                    token.start,
                                    token.end,
                                ));
                                forked.node_stack.push(node);
                                forked.state_stack.push(*next_state);
                                new_stacks.push(forked);
                            }
                            Action::Accept => {
                                any_success = true;
                                new_stacks.push(stack.clone());
                            }
                            Action::Reduce(_) => {
                                // Should have been handled in phase 1
                            }
                            _ => {}
                        }
                    }
                }
                Action::Error => {
                    // Stack dies here
                }
                _ => {}
            }
        }
        
        if !any_success && new_stacks.is_empty() {
            return Err(ParseError::UnexpectedToken {
                expected: self.get_expected_symbols(self.stacks[0].current_state()),
                found: token.symbol,
                position: token.start,
            });
        }
        
        self.stacks = self.merge_stacks(new_stacks);
        Ok(())
    }

    /// Perform a reduction on a specific stack
    fn reduce_stack(&mut self, mut stack: ParseStack, rule_id: RuleId) -> Option<ParseStack> {
        let rule = self.rule_map.get(&rule_id)?;
        
        // Pop nodes for each symbol in the rule's RHS
        let rhs_len = rule.rhs.len();
        let mut children = Vec::with_capacity(rhs_len);
        
        // Pop nodes and states
        for _ in 0..rhs_len {
            if let Some(node) = stack.node_stack.pop() {
                children.push(node);
            }
            stack.state_stack.pop();
        }
        children.reverse();
        
        // Get the goto state for the LHS symbol
        let current_state = *stack.state_stack.last()?;
        let goto_state = self.get_goto(current_state, rule.lhs).ok()?;
        
        // Create non-terminal node
        let start_byte = children
            .first()
            .map(|n| n.start_byte)
            .unwrap_or(self.position);
        let end_byte = children.last().map(|n| n.end_byte).unwrap_or(self.position);
        
        let node = Arc::new(ParseNode::non_terminal(
            rule.lhs,
            rule_id,
            children.into_iter().map(|arc| (*arc).clone()).collect(),
            start_byte,
            end_byte,
        ));
        
        // Push new node and state
        stack.node_stack.push(node);
        stack.state_stack.push(goto_state);
        
        // Update version info if rule has dynamic precedence
        if let Some(prec) = rule.precedence {
            match prec {
                rust_sitter_ir::PrecedenceKind::Dynamic(val) => {
                    stack.version.add_dynamic_prec(val as i32);
                }
                rust_sitter_ir::PrecedenceKind::Static(_) => {
                    // Static precedence is handled during conflict resolution
                }
            }
        }
        
        Some(stack)
    }
    
    /// Merge stacks that have reached the same state
    fn merge_stacks(&mut self, stacks: Vec<ParseStack>) -> Vec<ParseStack> {
        let mut merged = Vec::new();
        let mut processed = vec![false; stacks.len()];
        
        for i in 0..stacks.len() {
            if processed[i] {
                continue;
            }
            
            let mut best_stack = stacks[i].clone();
            processed[i] = true;
            
            // Look for other stacks with same state
            for j in (i + 1)..stacks.len() {
                if processed[j] {
                    continue;
                }
                
                let other = &stacks[j];
                
                // Check if stacks can be merged (same state and same parse tree structure)
                if best_stack.current_state() == other.current_state() 
                    && best_stack.node_stack.len() == other.node_stack.len() {
                    
                    // Use version comparison to pick the better stack
                    match compare_versions(&best_stack.version, &other.version) {
                        CompareResult::TakeRight => {
                            best_stack = other.clone();
                        }
                        CompareResult::PreferLeft | CompareResult::PreferRight | CompareResult::Tie => {
                            // Can't decide definitively - keep both
                            continue;
                        }
                        _ => {} // Keep current best
                    }
                    processed[j] = true;
                }
            }
            
            merged.push(best_stack);
        }
        
        merged
    }
    
    /// Handle fork action (for backward compatibility)
    fn handle_fork(&mut self, _actions: Vec<Action>, _token: &Token) -> Result<ParseNode, ParseError> {
        // This method is no longer used with the two-phase algorithm
        // The fork handling is integrated into the main parse loop
        Err(ParseError::InvalidState)
    }


    /// Get action for state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Result<Action, ParseError> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;

        self.parse_table
            .action_table
            .get(state_idx)
            .and_then(|row| row.get(symbol_idx))
            .cloned()
            .ok_or(ParseError::InvalidState)
    }

    /// Get goto state
    fn get_goto(&self, state: StateId, symbol: SymbolId) -> Result<StateId, ParseError> {
        let state_idx = state.0 as usize;
        let symbol_idx = symbol.0 as usize;

        self.parse_table
            .goto_table
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
    use rust_sitter_ir::{ProductionId, Symbol, TokenPattern};

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
        grammar
            .rules
            .entry(SymbolId(10))
            .or_insert_with(Vec::new)
            .push(Rule {
                lhs: SymbolId(10), // E
                rhs: vec![
                    Symbol::NonTerminal(SymbolId(10)), // E
                    Symbol::Terminal(SymbolId(2)),     // +
                    Symbol::NonTerminal(SymbolId(11)), // T
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: Default::default(),
            });

        grammar
    }

    #[test]
    fn test_parse_node_creation() {
        let terminal = ParseNode::terminal(SymbolId(1), b"123".to_vec(), 0, 3);

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

        let non_terminal =
            ParseNode::non_terminal(SymbolId(10), RuleId(10), vec![child1, child2, child3], 0, 3);

        assert_eq!(non_terminal.symbol, SymbolId(10));
        assert_eq!(non_terminal.rule_id, Some(RuleId(10)));
        assert_eq!(non_terminal.children.len(), 3);
        assert!(non_terminal.text.is_none());
    }
}
