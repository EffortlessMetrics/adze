// GLR parser implementation with fork/merge support
// This implements Tree-sitter's GLR parsing algorithm with dynamic precedence

use crate::subtree::{Subtree, SubtreeNode};
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
        }
    }
    
    /// Process one token through all active stacks
    pub fn process_token(&mut self, token: SymbolId, text: &str, byte_offset: usize) {
        let mut new_stacks = Vec::new();
        let _stack_merges = HashMap::<(StateId, usize), Vec<usize>>::new();
        
        // Process each active stack - work with a copy of the current stacks
        let current_stacks = std::mem::take(&mut self.stacks);
        self.pending_stacks.clear();
        
        for (stack_idx, stack) in current_stacks.into_iter().enumerate() {
            let state = stack.current_state();
            
            // Look up action in parse table
            if let Some(symbol_idx) = self.table.symbol_to_index.get(&token) {
                let action = self.table.action_table[state.0 as usize][*symbol_idx].clone();
                
                match &action {
                    Action::Shift(new_state) => {
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
                        let mut reduced_stack = stack.clone();
                        self.perform_reduction_on_stack(&mut reduced_stack, *rule_id);
                        new_stacks.push(reduced_stack);
                        // Mark for re-processing after reductions
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
                        // Mark this stack as accepting
                        // In a full implementation, we'd handle this properly
                    }
                    
                    Action::Error => {
                        // Enter error recovery
                        let mut error_stack = stack.clone();
                        error_stack.version.enter_error();
                        new_stacks.push(error_stack);
                    }
                }
            }
        }
        
        // Merge stacks that reach the same state
        self.merge_stacks(&mut new_stacks);
        
        // Update active stacks
        self.stacks = new_stacks;
        self.pending_stacks = (0..self.stacks.len()).collect();
    }
    
    /// Perform a reduction on a specific stack
    fn perform_reduction_on_stack(&mut self, stack: &mut ParseStack, rule_id: RuleId) {
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
                stack.push(goto_state, subtree);
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
        
        self.stacks[best_idx].nodes.last().cloned()
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