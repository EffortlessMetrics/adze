// GLR parser implementation with fork/merge support
// This implements Tree-sitter's GLR parsing algorithm with dynamic precedence

use crate::subtree::{Subtree, SubtreeNode};
use crate::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryState, RecoveryAction};
use rust_sitter_glr_core::{
    Action, ParseTable, StateId, SymbolId, RuleId,
    VersionInfo, CompareResult, compare_versions,
};
use rust_sitter_ir::Grammar;
use std::collections::{VecDeque, HashMap};
use std::sync::Arc;

/// A parse stack version (fork) in GLR parsing
#[derive(Debug, Clone)]
pub struct ParseStack {
    /// Stack of states
    states: Vec<StateId>,
    
    /// Stack of subtrees
    nodes: Vec<Arc<Subtree>>,
    
    /// Version tracking info for conflict resolution
    version: VersionInfo,
    
    /// Unique ID for this fork
    id: usize,
}

impl ParseStack {
    fn new(initial_state: StateId, id: usize) -> Self {
        Self {
            states: vec![initial_state],
            nodes: vec![],
            version: VersionInfo::new(),
            id,
        }
    }
    
    /// Get the current state
    fn current_state(&self) -> StateId {
        *self.states.last().expect("Empty state stack")
    }
    
    /// Push a new state and node
    fn push(&mut self, state: StateId, node: Arc<Subtree>) {
        // Update version info with dynamic precedence
        self.version.add_dynamic_prec(node.dynamic_prec);
        
        self.states.push(state);
        self.nodes.push(node);
    }
    
    /// Pop n states and nodes for a reduction
    fn pop(&mut self, n: usize) -> Vec<Arc<Subtree>> {
        self.states.truncate(self.states.len() - n);
        self.nodes.split_off(self.nodes.len() - n)
    }
    
    /// Clone this stack for forking
    fn fork(&self, new_id: usize) -> Self {
        Self {
            states: self.states.clone(),
            nodes: self.nodes.clone(),
            version: self.version.clone(),
            id: new_id,
        }
    }
}

/// GLR parser engine
pub struct GLRParser {
    /// Parse table
    table: ParseTable,
    
    /// Grammar for reductions
    grammar: Grammar,
    
    /// Active parse stacks
    stacks: Vec<ParseStack>,
    
    /// Next stack ID
    next_stack_id: usize,
    
    /// Stacks to process in the next step
    pending_stacks: VecDeque<usize>,
    
    /// Error recovery configuration
    error_recovery: Option<ErrorRecoveryConfig>,
    
    /// Error recovery state
    recovery_state: Option<ErrorRecoveryState>,
}

impl GLRParser {
    pub fn new(table: ParseTable, grammar: Grammar) -> Self {
        let initial_stack = ParseStack::new(StateId(0), 0);
        
        Self {
            table,
            grammar,
            stacks: vec![initial_stack],
            next_stack_id: 1,
            pending_stacks: VecDeque::from([0]),
            error_recovery: None,
            recovery_state: None,
        }
    }
    
    /// Enable error recovery with the given configuration
    pub fn enable_error_recovery(&mut self, config: ErrorRecoveryConfig) {
        self.recovery_state = Some(ErrorRecoveryState::new(config.clone()));
        self.error_recovery = Some(config);
    }
    
    /// Process one token through all active stacks
    pub fn process_token(&mut self, token: SymbolId, text: &str, byte_offset: usize) {
        // println!("Processing token: {} '{}' at offset {}", token.0, text, byte_offset);
        let mut new_stacks = Vec::new();
        let _stack_merges = HashMap::<(StateId, usize), Vec<usize>>::new();
        
        // Process each active stack - work with a copy of the current stacks
        let current_stacks = std::mem::take(&mut self.stacks);
        self.pending_stacks.clear();
        
        for (_stack_idx, stack) in current_stacks.into_iter().enumerate() {
            let state = stack.current_state();
            // println!("  Stack {} in state {}", stack.id, state.0);
            
            // Look up action in parse table
            if let Some(symbol_idx) = self.table.symbol_to_index.get(&token) {
                // println!("    Token {} maps to symbol index {}", token.0, symbol_idx);
                let action = self.table.action_table[state.0 as usize][*symbol_idx].clone();
                
                // If action is Error, check if we have any reductions available
                if matches!(action, Action::Error) {
                    // println!("    No action for token {} in state {}", token.0, state.0);
                    // Check all possible actions in this state
                    // println!("    Available actions in state {}:", state.0);
                    for (sym_id, sym_idx) in &self.table.symbol_to_index {
                        let act = &self.table.action_table[state.0 as usize][*sym_idx];
                        if !matches!(act, Action::Error) {
                            // println!("      For symbol {} (idx {}): {:?}", sym_id.0, sym_idx, act);
                        }
                    }
                }
                
                match &action {
                    Action::Shift(new_state) => {
                        // println!("    Action: Shift to state {}", new_state.0);
                        let mut new_stack = stack.clone();
                        new_stack.push(
                            *new_state,
                            Arc::new(Subtree::new(
                                SubtreeNode {
                                    symbol_id: token,
                                    is_error: false,
                                    byte_range: byte_offset..byte_offset + text.len(),
                                },
                                vec![],
                            ))
                        );
                        new_stacks.push(new_stack);
                    }
                    
                    Action::Reduce(rule_id) => {
                        // println!("    Action: Reduce rule {}", rule_id.0);
                        let mut reduced_stack = stack.clone();
                        self.perform_reduction_on_stack(&mut reduced_stack, *rule_id);
                        
                        // After reduction, we need to re-process the current token
                        // with the new state
                        let new_state = reduced_stack.current_state();
                        // println!("    After reduction, now in state {}", new_state.0);
                        
                        // Check what action to take with the current token in the new state
                        if let Some(symbol_idx) = self.table.symbol_to_index.get(&token) {
                            let new_action = &self.table.action_table[new_state.0 as usize][*symbol_idx];
                            // println!("    New action for token after reduction: {:?}", new_action);
                            
                            match new_action {
                                Action::Shift(shift_state) => {
                                    // println!("    Shifting to state {} after reduction", shift_state.0);
                                    reduced_stack.push(
                                        *shift_state,
                                        Arc::new(Subtree::new(
                                            SubtreeNode {
                                                symbol_id: token,
                                                is_error: false,
                                                byte_range: byte_offset..byte_offset + text.len(),
                                            },
                                            vec![],
                                        ))
                                    );
                                }
                                _ => {
                                    // If it's another reduce or error, just add the stack for further processing
                                    // println!("    Action after reduction is {:?}, will process later", new_action);
                                }
                            }
                        }
                        
                        new_stacks.push(reduced_stack);
                    }
                    
                    Action::Fork(actions) => {
                        // Handle GLR fork - create multiple stacks
                        for (_i, fork_action) in actions.iter().enumerate() {
                            match fork_action {
                                Action::Shift(new_state) => {
                                    let mut forked = stack.fork(self.next_stack_id);
                                    self.next_stack_id += 1;
                                    
                                    forked.push(
                                        *new_state,
                                        Arc::new(Subtree::new(
                                            SubtreeNode {
                                                symbol_id: token,
                                                is_error: false,
                                                byte_range: byte_offset..byte_offset + text.len(),
                                            },
                                            vec![],
                                        ))
                                    );
                                    new_stacks.push(forked);
                                }
                                
                                Action::Reduce(rule_id) => {
                                    let mut forked = stack.fork(self.next_stack_id);
                                    self.next_stack_id += 1;
                                    self.perform_reduction_on_stack(&mut forked, *rule_id);
                                    new_stacks.push(forked);
                                    // Mark for re-processing
                                }
                                
                                _ => {}
                            }
                        }
                    }
                    
                    Action::Accept => {
                        // println!("    Action: Accept");
                        // This shouldn't happen anymore since we removed Accept from parse table
                        // Keep the stack as an accepting stack
                        new_stacks.push(stack);
                    }
                    
                    Action::Error => {
                        // println!("    Action: Error");
                        
                        // Try error recovery if enabled
                        if let Some(recovery_state) = &mut self.recovery_state {
                            if let Some(recovery_action) = recovery_state.suggest_recovery(
                                state,
                                token,
                                &self.table,
                                &self.grammar,
                            ) {
                                match recovery_action {
                                    RecoveryAction::InsertToken(missing_token) => {
                                        // Try to shift the missing token
                                        if let Some(&missing_idx) = self.table.symbol_to_index.get(&missing_token) {
                                            let missing_action = &self.table.action_table[state.0 as usize][missing_idx];
                                            if let Action::Shift(new_state) = missing_action {
                                                let mut recovery_stack = stack.clone();
                                                // Create dummy node for inserted token
                                                let error_node = Arc::new(Subtree {
                                                    node: SubtreeNode {
                                                        symbol_id: missing_token,
                                                        is_error: true,
                                                        byte_range: byte_offset..byte_offset,
                                                    },
                                                    dynamic_prec: 0,
                                                    children: vec![],
                                                });
                                                recovery_stack.push(*new_state, error_node);
                                                recovery_stack.version.enter_error();
                                                // Re-queue the current token
                                                self.pending_stacks.push_back(self.stacks.len() + new_stacks.len());
                                                new_stacks.push(recovery_stack);
                                                continue;
                                            }
                                        }
                                    }
                                    RecoveryAction::DeleteToken => {
                                        // Skip this token and continue with the same stack
                                        new_stacks.push(stack.clone());
                                        continue;
                                    }
                                    RecoveryAction::CreateErrorNode(_) => {
                                        // Create an error node containing the unexpected token
                                        let error_node = Arc::new(Subtree {
                                            node: SubtreeNode {
                                                symbol_id: token,
                                                is_error: true,
                                                byte_range: byte_offset..byte_offset + text.len(),
                                            },
                                            dynamic_prec: 0,
                                            children: vec![],
                                        });
                                        let mut error_stack = stack.clone();
                                        // Just add the error node without changing state
                                        error_stack.nodes.push(error_node);
                                        error_stack.version.enter_error();
                                        new_stacks.push(error_stack);
                                        continue;
                                    }
                                    _ => {} // Other recovery actions not implemented yet
                                }
                            }
                        }
                        
                        // Default error handling - mark stack as errored
                        let mut error_stack = stack.clone();
                        error_stack.version.enter_error();
                        new_stacks.push(error_stack);
                    }
                }
            }
        }
        
        // After processing all stacks, check if any need further reductions
        let mut needs_reprocessing = true;
        let mut iterations = 0;
        while needs_reprocessing && iterations < 20 {
            needs_reprocessing = false;
            iterations += 1;
            
            let mut additional_stacks = Vec::new();
            for stack in &new_stacks {
                let state = stack.current_state();
                
                // Check if this state has any reduce actions for the current token
                if let Some(symbol_idx) = self.table.symbol_to_index.get(&token) {
                    let action = &self.table.action_table[state.0 as usize][*symbol_idx];
                    
                    if let Action::Reduce(rule_id) = action {
                        // Need to perform another reduction
                        let mut reduced_stack = stack.clone();
                        self.perform_reduction_on_stack(&mut reduced_stack, *rule_id);
                        additional_stacks.push(reduced_stack);
                        needs_reprocessing = true;
                    }
                }
            }
            
            new_stacks.extend(additional_stacks);
        }
        
        // Merge stacks that reach the same state
        self.merge_stacks(&mut new_stacks);
        
        // Update active stacks
        // println!("  After processing: {} stacks", new_stacks.len());
        self.stacks = new_stacks;
        self.pending_stacks = (0..self.stacks.len()).collect();
    }
    
    /// Perform a reduction on a specific stack
    fn perform_reduction_on_stack(&mut self, stack: &mut ParseStack, rule_id: RuleId) {
        // println!("  Performing reduction of rule {}", rule_id.0);
        // Find the rule in the grammar
        if let Some(rule) = self.grammar.rules.values().find(|r| r.production_id.0 == rule_id.0) {
            let children = stack.pop(rule.rhs.len());
            
            // Create new subtree for the reduction
            let node = SubtreeNode {
                symbol_id: rule.lhs,
                is_error: false,
                byte_range: if children.is_empty() {
                    0..0
                } else {
                    children[0].node.byte_range.start..children.last().unwrap().node.byte_range.end
                },
            };
            
            // Check if this rule has dynamic precedence
            let dynamic_prec = if let Some(rust_sitter_ir::PrecedenceKind::Dynamic(prec)) = &rule.precedence {
                *prec as i32
            } else {
                0
            };
            
            let subtree = Arc::new(Subtree::with_dynamic_prec(node, children, dynamic_prec));
            
            // Look up goto state
            if let Some(symbol_idx) = self.table.symbol_to_index.get(&rule.lhs) {
                let goto_state = self.table.goto_table[stack.current_state().0 as usize][*symbol_idx];
                // println!("  After reducing, goto state {} for symbol {}", goto_state.0, rule.lhs.0);
                stack.push(goto_state, subtree);
            } else {
                // println!("  WARNING: No symbol index for LHS {}", rule.lhs.0);
            }
        }
    }
    
    /// Merge stacks that have reached the same state
    fn merge_stacks(&mut self, stacks: &mut Vec<ParseStack>) {
        let mut merged = Vec::new();
        let mut processed = vec![false; stacks.len()];
        
        for i in 0..stacks.len() {
            if processed[i] {
                continue;
            }
            
            let mut best_stack = stacks[i].clone();
            processed[i] = true;
            
            // Find all stacks with the same state and node count
            for j in (i + 1)..stacks.len() {
                if processed[j] {
                    continue;
                }
                
                if stacks[j].states == best_stack.states {
                    // Same parse state - compare versions
                    match compare_versions(&best_stack.version, &stacks[j].version) {
                        CompareResult::TakeLeft => {
                            // Keep best_stack
                        }
                        CompareResult::TakeRight => {
                            best_stack = stacks[j].clone();
                        }
                        CompareResult::PreferLeft => {
                            // In full GLR, we might keep both
                            // For now, keep the preferred one
                        }
                        CompareResult::PreferRight => {
                            best_stack = stacks[j].clone();
                        }
                        CompareResult::Tie => {
                            // Use symbol comparison or keep both
                            // For now, keep the first one
                        }
                    }
                    processed[j] = true;
                }
            }
            
            merged.push(best_stack);
        }
        
        *stacks = merged;
    }
    
    /// Get the best parse tree from active stacks
    pub fn get_best_parse(&self) -> Option<Arc<Subtree>> {
        // println!("Getting best parse from {} stacks", self.stacks.len());
        if self.stacks.is_empty() {
            return None;
        }
        
        // Find the best stack according to version comparison
        let mut best_idx = 0;
        for i in 1..self.stacks.len() {
            match compare_versions(&self.stacks[best_idx].version, &self.stacks[i].version) {
                CompareResult::TakeRight | CompareResult::PreferRight => {
                    best_idx = i;
                }
                _ => {}
            }
        }
        
        // println!("Best stack {} has {} nodes", best_idx, self.stacks[best_idx].nodes.len());
        self.stacks[best_idx].nodes.last().cloned()
    }
    
    /// Process EOF to complete parsing
    pub fn process_eof(&mut self) {
        // println!("Processing EOF");
        // Process EOF token (symbol ID 0)
        self.process_token(SymbolId(0), "", 0);
    }
    
    /// Get number of active stacks (for debugging)
    pub fn stack_count(&self) -> usize {
        self.stacks.len()
    }
    
    /// Finish parsing and get the result
    pub fn finish(&self) -> Result<Arc<Subtree>, String> {
        // Find a successfully parsed stack
        // Success criteria:
        // 1. Has exactly one node (the root of the parse tree)
        // 2. That node represents the start symbol (we'll accept any non-terminal for now)
        
        for stack in &self.stacks {
            if stack.nodes.len() == 1 {
                // Check if the single node is a non-terminal (not a raw token)
                let node = &stack.nodes[0];
                // In our grammar, non-terminals have IDs >= 10
                if node.node.symbol_id.0 >= 10 {
                    return Ok(node.clone());
                }
            }
        }
        
        // If no accepted stack, return error with debugging info
        let states: Vec<_> = self.stacks.iter()
            .map(|s| {
                let state = s.states.last().copied().unwrap_or(StateId(0));
                (state, s.nodes.len(), s.nodes.iter().map(|n| n.node.symbol_id).collect::<Vec<_>>())
            })
            .collect();
        Err(format!("Parse incomplete. Stack states: {:?}", states))
    }

    /// Reset parser state for reuse
    pub fn reset(&mut self) {
        self.stacks.clear();
        let initial_stack = ParseStack::new(StateId(0), self.next_stack_id);
        self.next_stack_id += 1;
        self.stacks.push(initial_stack);
        self.pending_stacks.clear();
        self.pending_stacks.push_back(0);
    }

    /// Get expected symbols at current parse state
    pub fn expected_symbols(&self) -> Vec<SymbolId> {
        let mut symbols = Vec::new();
        
        for stack in &self.stacks {
            let state = stack.current_state();
            
            // Check all possible actions from this state
            for (symbol, _symbol_idx) in &self.table.symbol_to_index {
                if let Some(_action) = self.get_action(state, *symbol) {
                    if !symbols.contains(symbol) {
                        symbols.push(*symbol);
                    }
                }
            }
        }
        
        symbols
    }

    /// Inject a pre-parsed subtree into the parser
    pub fn inject_subtree(&mut self, subtree: Arc<Subtree>) {
        // For each active stack, try to process this subtree
        let mut new_stacks = Vec::new();
        
        for stack in &self.stacks {
            let state = stack.current_state();
            
            // Check if we can shift this subtree's symbol
            if let Some(action) = self.get_action(state, subtree.node.symbol_id) {
                match action {
                    Action::Shift(next_state) => {
                        let mut new_stack = stack.clone();
                        new_stack.push(next_state, subtree.clone());
                        new_stacks.push(new_stack);
                    }
                    _ => {
                        // For reduce/accept actions, keep the original stack
                        new_stacks.push(stack.clone());
                    }
                }
            }
        }
        
        self.stacks = new_stacks;
    }
    
    /// Get action from parse table for state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Option<Action> {
        let state_idx = state.0 as usize;
        
        if state_idx < self.table.action_table.len() {
            if let Some(&symbol_idx) = self.table.symbol_to_index.get(&symbol) {
                if symbol_idx < self.table.action_table[state_idx].len() {
                    return Some(self.table.action_table[state_idx][symbol_idx].clone());
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_stack_creation() {
        let stack = ParseStack::new(StateId(0), 0);
        assert_eq!(stack.current_state(), StateId(0));
        assert_eq!(stack.nodes.len(), 0);
        assert_eq!(stack.version.dynamic_prec, 0);
    }
    
    #[test]
    fn test_parse_stack_fork() {
        let mut stack = ParseStack::new(StateId(0), 0);
        
        // Add a node
        let node = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(1),
                is_error: false,
                byte_range: 0..5,
            },
            vec![],
        ));
        stack.push(StateId(1), node);
        
        // Fork the stack
        let forked = stack.fork(1);
        assert_eq!(forked.states, stack.states);
        assert_eq!(forked.nodes.len(), stack.nodes.len());
        assert_ne!(forked.id, stack.id);
    }
    
    #[test]
    fn test_dynamic_precedence_accumulation() {
        let mut stack = ParseStack::new(StateId(0), 0);
        
        // Add nodes with dynamic precedence
        let node1 = Arc::new(Subtree::with_dynamic_prec(
            SubtreeNode {
                symbol_id: SymbolId(1),
                is_error: false,
                byte_range: 0..5,
            },
            vec![],
            3,
        ));
        stack.push(StateId(1), node1);
        assert_eq!(stack.version.dynamic_prec, 3);
        
        let node2 = Arc::new(Subtree::with_dynamic_prec(
            SubtreeNode {
                symbol_id: SymbolId(2),
                is_error: false,
                byte_range: 5..10,
            },
            vec![],
            2,
        ));
        stack.push(StateId(2), node2);
        assert_eq!(stack.version.dynamic_prec, 5); // 3 + 2
    }
}