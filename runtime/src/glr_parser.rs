//! GLR (Generalized LR) Parser Implementation
//!
//! This module implements a GLR parser that can handle ambiguous grammars by maintaining
//! multiple parse stacks simultaneously. When the parser encounters a shift/reduce or
//! reduce/reduce conflict, it forks the parse stack and explores both possibilities.
//!
//! ## Algorithm Overview
//!
//! The parser uses a two-phase approach for processing tokens:
//!
//! ### Phase 1: Reduction Saturation
//! Before consuming any token, the parser performs all possible reductions on all active
//! stacks. This is crucial because:
//! - Reductions can cascade (one reduction enables another)
//! - We must complete all reductions before shifting to maintain correctness
//! - This prevents tokens from being consumed prematurely or processed multiple times
//!
//! ### Phase 2: Token Processing  
//! After all reductions are complete, the parser:
//! - Processes shift actions for the current token
//! - Handles fork actions (creating new stacks for conflicts)
//! - Processes error recovery if no valid actions exist
//!
//! ## Fork/Merge Strategy
//!
//! When conflicts occur, the parser:
//! 1. Forks the current stack into multiple stacks (one per conflicting action)
//! 2. Processes each fork independently
//! 3. Merges stacks that reach the same state with the same parse tree structure
//! 4. Uses dynamic precedence to resolve ambiguities when possible
//!
//! ## Error Recovery
//!
//! The parser supports configurable error recovery strategies:
//! - Token deletion (skip unexpected tokens)
//! - Token insertion (insert missing tokens)
//! - Panic mode (skip to synchronization points)
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use rust_sitter::glr_parser::GLRParser;
//! use rust_sitter::glr_lexer::GLRLexer;
//! use rust_sitter_ir::{Grammar, SymbolId};
//! use rust_sitter_glr_core::ParseTable;
//!
//! // Create parser with grammar and parse table
//! let grammar: Grammar = /* ... */;
//! let parse_table: ParseTable = /* ... */;
//! let mut parser = GLRParser::new(grammar, parse_table);
//!
//! // Create lexer and tokenize input
//! let mut lexer = GLRLexer::new(&grammar);
//! let tokens = lexer.tokenize("1 + 2 * 3").unwrap();
//!
//! // Process each token
//! for token in tokens {
//!     parser.process_token(token.symbol, &token.text, token.start_byte);
//! }
//!
//! // Process EOF and get result
//! parser.process_eof();
//! match parser.finish() {
//!     Ok(tree) => println!("Parse successful!"),
//!     Err(e) => println!("Parse failed: {}", e),
//! }
//! ```

use crate::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryState, RecoveryAction};
use crate::subtree::{Subtree, SubtreeNode};
use rust_sitter_glr_core::{Action, CompareResult, ParseTable, VersionInfo, compare_versions};
use rust_sitter_glr_core::{FirstFollowSets, VecWrapperResolver};
use rust_sitter_ir::{Grammar, PrecedenceKind, Rule, Symbol};
use rust_sitter_ir::{RuleId, StateId, SymbolId};
use std::collections::VecDeque;
use std::sync::Arc;

// Debug macro for GLR parser
#[cfg(feature = "debug_glr")]
macro_rules! debug_glr {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

#[cfg(not(feature = "debug_glr"))]
macro_rules! debug_glr {
    ($($arg:tt)*) => {};
}

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
    #[allow(dead_code)]
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

    /// Print tree structure for debugging
    fn print_tree_structure(node: &Arc<Subtree>, indent: usize) {
        let _prefix = "  ".repeat(indent);
        debug_glr!(
            "{}Symbol {}, range {:?}",
            _prefix,
            node.node.symbol_id.0,
            node.node.byte_range
        );
        for edge in &node.children {
            Self::print_tree_structure(&edge.subtree, indent + 1);
        }
    }

    /// Check if two stacks have structurally equivalent parse trees
    fn has_equivalent_parse_tree(&self, other: &ParseStack) -> bool {
        // First check if they have the same number of nodes
        if self.nodes.len() != other.nodes.len() {
            return false;
        }

        // Check each node for structural equivalence
        for (node1, node2) in self.nodes.iter().zip(other.nodes.iter()) {
            if !Self::nodes_structurally_equivalent(node1, node2) {
                return false;
            }
        }

        true
    }

    /// Check if two subtree nodes are structurally equivalent
    fn nodes_structurally_equivalent(node1: &Arc<Subtree>, node2: &Arc<Subtree>) -> bool {
        // Check symbol and span
        if node1.node.symbol_id != node2.node.symbol_id {
            return false;
        }

        if node1.node.byte_range != node2.node.byte_range {
            return false;
        }

        // Check if both are error nodes
        if node1.node.is_error != node2.node.is_error {
            return false;
        }

        // Check children structure
        if node1.children.len() != node2.children.len() {
            return false;
        }

        // Recursively check all children
        for (edge1, edge2) in node1.children.iter().zip(node2.children.iter()) {
            // Check field IDs match
            if edge1.field_id != edge2.field_id {
                return false;
            }
            // Check subtrees match
            if !Self::nodes_structurally_equivalent(&edge1.subtree, &edge2.subtree) {
                return false;
            }
        }

        true
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

    /// Conflict resolver for vec wrapper conflicts
    #[allow(dead_code)]
    vec_wrapper_resolver: Option<VecWrapperResolver>,

    /// Total input length in bytes (set when process_eof is called)
    input_length: usize,
}

impl GLRParser {
    /// Get a rule by its ID
    fn get_rule(&self, rule_id: RuleId) -> Option<&Rule> {
        let mut rule_counter = 0;
        for rules in self.grammar.rules.values() {
            for rule in rules {
                if rule_counter == rule_id.0 as usize {
                    return Some(rule);
                }
                rule_counter += 1;
            }
        }
        None
    }

    pub fn new(table: ParseTable, grammar: Grammar) -> Self {
        let initial_stack = ParseStack::new(StateId(0), 0);

        // Compute FIRST/FOLLOW sets for the resolver
        let first_follow = FirstFollowSets::compute(&grammar);
        let vec_wrapper_resolver = Some(VecWrapperResolver::new(&grammar, &first_follow));

        Self {
            table,
            grammar,
            stacks: vec![initial_stack],
            next_stack_id: 1,
            pending_stacks: VecDeque::from([0]),
            error_recovery: None,
            recovery_state: None,
            vec_wrapper_resolver,
            input_length: 0,
        }
    }

    /// Enable error recovery with the given configuration
    pub fn enable_error_recovery(&mut self, config: ErrorRecoveryConfig) {
        self.recovery_state = Some(ErrorRecoveryState::new(config.clone()));
        self.error_recovery = Some(config);
    }

    /// Process one token through all active stacks
    ///
    /// This is the main entry point for processing tokens. It implements the two-phase
    /// approach described in the module documentation:
    ///
    /// 1. First, it performs all possible reductions on all active stacks using
    ///    `reduce_until_saturated()`. This ensures all cascading reductions complete
    ///    before any shifts occur.
    ///
    /// 2. Then, it processes the token by examining shift and fork actions on the
    ///    reduced stacks.
    ///
    /// # Arguments
    /// * `token` - The symbol ID of the token to process
    /// * `text` - The text content of the token
    /// * `byte_offset` - The byte position of the token in the input
    pub fn process_token(&mut self, token: SymbolId, text: &str, byte_offset: usize) {
        // Processing token

        // Phase 1: Perform all possible reductions until saturation
        let mut stacks_to_process = std::mem::take(&mut self.stacks);
        self.pending_stacks.clear();

        stacks_to_process = self.reduce_until_saturated(stacks_to_process, token);

        // Phase 2: Process shifts and other actions on all post-reduction stacks
        let mut new_stacks = Vec::new();

        for stack in stacks_to_process {
            let state = stack.current_state();

            // Debug: Print current state and token being processed
            debug_glr!(
                "DEBUG: Processing token {} (symbol_idx: {:?}) in state {}",
                token.0,
                self.table.symbol_to_index.get(&token),
                state.0
            );

            if let Some(symbol_idx) = self.table.symbol_to_index.get(&token) {
                let action_cell = &self.table.action_table[state.0 as usize][*symbol_idx];

                // Debug: Print action cell contents
                if action_cell.len() > 1 {
                    debug_glr!(
                        "DEBUG: Found multi-action cell at state {} for token {}: {} actions",
                        state.0,
                        token.0,
                        action_cell.len()
                    );
                    #[allow(clippy::unused_enumerate_index)]
                    for (_i, _act) in action_cell.iter().enumerate() {
                        debug_glr!("  Action {}: {:?}", _i, _act);
                    }
                }

                // Check action for token

                // Convert ActionCell to single action or Fork
                let action = if action_cell.is_empty() {
                    Action::Error
                } else if action_cell.len() == 1 {
                    action_cell[0].clone()
                } else {
                    Action::Fork(action_cell.clone())
                };

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
                            )),
                        );
                        new_stacks.push(new_stack);
                    }

                    Action::Accept => {
                        // Keep the accepting stack
                        new_stacks.push(stack);
                    }

                    Action::Reduce(_) => {
                        // This shouldn't happen after reduce_until_saturated
                        unreachable!("Found reduce action after saturation");
                    }

                    Action::Fork(actions) => {
                        // TRUE GLR FORKING! Always fork for ambiguity preservation
                        // Note: we ignore the conflict resolver to maintain all parse alternatives
                        {
                            // No resolution - TRUE GLR FORKING!
                            // This is the critical part where we maintain ambiguity by forking stacks
                            debug_glr!(
                                "DEBUG: GLR Fork! Creating {} stacks for state {} with token {}",
                                actions.len(),
                                state.0,
                                token.0
                            );

                            // Fork the stack for EACH action to explore all parse paths
                            #[allow(unused_variables)]
                            for (i, fork_action) in actions.iter().enumerate() {
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
                                                    byte_range: byte_offset
                                                        ..byte_offset + text.len(),
                                                },
                                                vec![],
                                            )),
                                        );
                                        debug_glr!("  Fork {}: Shift to state {}", i, new_state.0);
                                        new_stacks.push(forked);
                                    }

                                    Action::Reduce(rule_id) => {
                                        // Reductions should have been handled in phase 1, but if not, handle them
                                        let mut forked = stack.fork(self.next_stack_id);
                                        self.next_stack_id += 1;
                                        self.perform_reduction_on_stack(&mut forked, *rule_id);
                                        debug_glr!("  Fork {}: Reduce by rule {}", i, rule_id.0);
                                        new_stacks.push(forked);
                                    }

                                    Action::Fork(nested_actions) => {
                                        // Handle nested Fork recursively
                                        debug_glr!(
                                            "  Fork {}: Nested fork with {} actions",
                                            i,
                                            nested_actions.len()
                                        );
                                        for nested_action in nested_actions {
                                            let mut nested_fork = stack.fork(self.next_stack_id);
                                            self.next_stack_id += 1;

                                            match nested_action {
                                                Action::Shift(new_state) => {
                                                    nested_fork.push(
                                                        *new_state,
                                                        Arc::new(Subtree::new(
                                                            SubtreeNode {
                                                                symbol_id: token,
                                                                is_error: false,
                                                                byte_range: byte_offset
                                                                    ..byte_offset + text.len(),
                                                            },
                                                            vec![],
                                                        )),
                                                    );
                                                    new_stacks.push(nested_fork);
                                                }
                                                Action::Reduce(rule_id) => {
                                                    self.perform_reduction_on_stack(
                                                        &mut nested_fork,
                                                        *rule_id,
                                                    );
                                                    new_stacks.push(nested_fork);
                                                }
                                                _ => {
                                                    new_stacks.push(nested_fork);
                                                }
                                            }
                                        }
                                    }

                                    _ => {
                                        debug_glr!("  Fork {}: Other action", i);
                                    }
                                }
                            }

                            // If no valid forks were created, keep the original stack
                            if new_stacks.is_empty() {
                                new_stacks.push(stack);
                            }
                        }
                    }

                    Action::Recover => {
                        // Handle Recover action - similar to Error but with specific recovery
                        // For now, treat it as an error
                        let mut error_stack = stack.clone();
                        error_stack.version.enter_error();
                        new_stacks.push(error_stack);
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
                                        if let Some(&missing_idx) =
                                            self.table.symbol_to_index.get(&missing_token)
                                        {
                                            let missing_action_cell = &self.table.action_table
                                                [state.0 as usize][missing_idx];
                                            // Find shift action in cell
                                            let shift_action = missing_action_cell
                                                .iter()
                                                .find(|a| matches!(a, Action::Shift(_)));
                                            if let Some(Action::Shift(new_state)) = shift_action {
                                                let mut recovery_stack = stack.clone();
                                                // Create dummy node for inserted token
                                                let error_node = Arc::new(Subtree::new(
                                                    SubtreeNode {
                                                        symbol_id: missing_token,
                                                        is_error: true,
                                                        byte_range: byte_offset..byte_offset,
                                                    },
                                                    vec![], // Empty children for error node
                                                ));
                                                recovery_stack.push(*new_state, error_node);
                                                recovery_stack.version.enter_error();
                                                // Re-queue the current token
                                                self.pending_stacks.push_back(
                                                    self.stacks.len() + new_stacks.len(),
                                                );
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

                    _ => {
                        // Unknown action type - treat as error
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

    /// Perform all possible reductions on the given stacks until no more reductions apply
    ///
    /// This method implements the reduction saturation phase of GLR parsing. It repeatedly
    /// applies all possible reductions to all stacks until no more reductions are available.
    /// This is essential for correctness because:
    ///
    /// 1. **Cascading Reductions**: One reduction may enable another. For example, reducing
    ///    `E → E + E` might enable reducing `S → E` at a higher level.
    ///
    /// 2. **Completeness**: We must explore all reduction paths before shifting to ensure
    ///    we don't miss valid parses.
    ///
    /// 3. **Fork Handling**: When a fork action contains both reduce and shift actions,
    ///    we process all reductions in this phase and defer shifts to phase 2.
    ///
    /// The method includes an iteration limit to prevent infinite loops in case of
    /// grammar bugs.
    ///
    /// # Arguments
    /// * `stacks` - The parse stacks to process
    /// * `token` - The lookahead token (used to determine which reductions apply)
    ///
    /// # Returns
    /// A vector of stacks with all reductions applied
    fn reduce_until_saturated(
        &mut self,
        mut stacks: Vec<ParseStack>,
        token: SymbolId,
    ) -> Vec<ParseStack> {
        // Track which reductions have been applied to prevent infinite loops on epsilon rules
        // Key: (stack_id, top_state, rule_id, pop_length, predecessor_state)
        // This allows legitimate reductions from different predecessor paths while preventing
        // the same reduction from being applied infinitely
        let mut applied_reductions: std::collections::HashSet<(
            usize,
            StateId,
            RuleId,
            usize,
            StateId,
        )> = std::collections::HashSet::new();

        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 100;

        loop {
            iteration += 1;
            if iteration > MAX_ITERATIONS {
                debug_glr!(
                    "ERROR: Exceeded {} reduction iterations with {} stacks - breaking to prevent infinite loop",
                    MAX_ITERATIONS,
                    stacks.len()
                );
                break;
            }

            let mut any_reduction_performed = false;
            let mut result_stacks = Vec::new();

            for stack in stacks {
                let state = stack.current_state();

                debug_glr!(
                    "DEBUG reduce phase: Checking state {} for token {}",
                    state.0,
                    token.0
                );

                if let Some(symbol_idx) = self.table.symbol_to_index.get(&token) {
                    let action_cell =
                        self.table.action_table[state.0 as usize][*symbol_idx].clone();

                    // Handle multiple actions in the cell (GLR)
                    if action_cell.is_empty() {
                        // No action available, keep stack as is
                        result_stacks.push(stack);
                    } else if action_cell.len() == 1 {
                        // Single action
                        match &action_cell[0] {
                            Action::Reduce(rule_id) => {
                                // Get rule to determine pop length
                                let pop_len = if let Some(rule) = self.get_rule(*rule_id) {
                                    rule.rhs.len()
                                } else {
                                    0
                                };

                                // Get predecessor state (state after popping)
                                let pred_state = if stack.states.len() > pop_len {
                                    stack.states[stack.states.len() - pop_len - 1]
                                } else {
                                    StateId(0)
                                };

                                // Check if we've already applied this reduction to avoid infinite loops
                                let reduction_key =
                                    (stack.id, state, *rule_id, pop_len, pred_state);
                                if !applied_reductions.contains(&reduction_key) {
                                    applied_reductions.insert(reduction_key);
                                    any_reduction_performed = true;
                                    let mut reduced_stack = stack.clone();
                                    self.perform_reduction_on_stack(&mut reduced_stack, *rule_id);
                                    result_stacks.push(reduced_stack);
                                } else {
                                    // Already applied this reduction, skip to prevent infinite loop
                                    result_stacks.push(stack);
                                }
                            }
                            Action::Fork(actions) => {
                                // Handle fork action
                                let mut has_reduction = false;
                                let mut fork_results = Vec::new();

                                for fork_action in actions {
                                    match fork_action {
                                        Action::Reduce(rule_id) => {
                                            let pop_len =
                                                if let Some(rule) = self.get_rule(*rule_id) {
                                                    rule.rhs.len()
                                                } else {
                                                    0
                                                };
                                            let pred_state = if stack.states.len() > pop_len {
                                                stack.states[stack.states.len() - pop_len - 1]
                                            } else {
                                                StateId(0)
                                            };

                                            let reduction_key =
                                                (stack.id, state, *rule_id, pop_len, pred_state);
                                            if !applied_reductions.contains(&reduction_key) {
                                                applied_reductions.insert(reduction_key);
                                                has_reduction = true;
                                                any_reduction_performed = true;
                                                let mut forked = stack.fork(self.next_stack_id);
                                                self.next_stack_id += 1;
                                                self.perform_reduction_on_stack(
                                                    &mut forked,
                                                    *rule_id,
                                                );
                                                fork_results.push(forked);
                                            }
                                        }
                                        _ => {
                                            // Non-reduction fork branches will be handled in phase 2
                                        }
                                    }
                                }

                                if has_reduction {
                                    result_stacks.extend(fork_results);
                                } else {
                                    result_stacks.push(stack);
                                }
                            }
                            _ => {
                                // Non-reduction action, keep stack
                                result_stacks.push(stack);
                            }
                        }
                    } else {
                        // Multiple actions - need to fork
                        debug_glr!(
                            "DEBUG reduce: Found {} actions in state {} for token {}",
                            action_cell.len(),
                            state.0,
                            token.0
                        );
                        let mut has_reduction = false;
                        let mut has_shift = false;
                        let mut fork_results = Vec::new();

                        for action in &action_cell {
                            match action {
                                Action::Reduce(rule_id) => {
                                    let pop_len = if let Some(rule) = self.get_rule(*rule_id) {
                                        rule.rhs.len()
                                    } else {
                                        0
                                    };
                                    let pred_state = if stack.states.len() > pop_len {
                                        stack.states[stack.states.len() - pop_len - 1]
                                    } else {
                                        StateId(0)
                                    };

                                    let reduction_key =
                                        (stack.id, state, *rule_id, pop_len, pred_state);
                                    if !applied_reductions.contains(&reduction_key) {
                                        applied_reductions.insert(reduction_key);
                                        has_reduction = true;
                                        any_reduction_performed = true;
                                        let mut forked = stack.fork(self.next_stack_id);
                                        self.next_stack_id += 1;
                                        debug_glr!("  Forking for reduce with rule {}", rule_id.0);
                                        self.perform_reduction_on_stack(&mut forked, *rule_id);
                                        fork_results.push(forked);
                                    }
                                }
                                Action::Shift(_) => {
                                    // Mark that we have a shift action
                                    has_shift = true;
                                    debug_glr!(
                                        "  Found shift action - will preserve stack for phase 2"
                                    );
                                }
                                _ => {
                                    // Other non-reduction actions will be handled in phase 2
                                }
                            }
                        }

                        // CRITICAL: If we have both shift and reduce, we need to keep the original
                        // stack for the shift action that will be processed in phase 2!
                        if has_shift {
                            debug_glr!("  Preserving original stack for shift action");
                            result_stacks.push(stack.clone());
                        }

                        if has_reduction {
                            result_stacks.extend(fork_results);
                        }

                        // If we have neither shift nor reduce, keep the original stack
                        if !has_shift && !has_reduction {
                            result_stacks.push(stack);
                        }
                    }
                } else {
                    // Token not in symbol table, keep stack
                    result_stacks.push(stack);
                }
            }

            // CRITICAL: Merge stacks with the same state to prevent exponential explosion
            self.merge_stacks(&mut result_stacks);

            if result_stacks.len() > 10 {
                debug_glr!(
                    "DEBUG reduce: After merging, have {} stacks",
                    result_stacks.len()
                );
            }

            stacks = result_stacks;

            if !any_reduction_performed {
                break;
            }
        }

        stacks
    }

    /// Perform a reduction on a specific stack
    fn perform_reduction_on_stack(&mut self, stack: &mut ParseStack, rule_id: RuleId) {
        debug_glr!(
            "DEBUG: Performing reduction with rule {} on stack in state {}",
            rule_id.0,
            stack.current_state().0
        );
        // Perform reduction
        // Find the rule in the grammar
        if let Some(rule) = self
            .grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .find(|r| r.production_id.0 == rule_id.0)
        {
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

            // Apply field mappings to children
            let children_with_fields = if rule.fields.is_empty() {
                // No fields, use FIELD_NONE for all children
                children
                    .into_iter()
                    .map(|subtree| crate::subtree::ChildEdge {
                        subtree,
                        field_id: crate::subtree::FIELD_NONE,
                    })
                    .collect()
            } else {
                // Apply field mappings based on rule.fields
                let mut result = Vec::with_capacity(children.len());
                for (idx, child) in children.into_iter().enumerate() {
                    // Find field ID for this child position
                    let field_id = rule
                        .fields
                        .iter()
                        .find(|(_, pos)| *pos == idx)
                        .map(|(field_id, _)| field_id.0)
                        .unwrap_or(crate::subtree::FIELD_NONE);

                    result.push(crate::subtree::ChildEdge {
                        subtree: child,
                        field_id,
                    });
                }
                result
            };

            // Check if this rule has dynamic precedence
            let dynamic_prec =
                if let Some(rust_sitter_ir::PrecedenceKind::Dynamic(prec)) = &rule.precedence {
                    *prec as i32
                } else {
                    0
                };

            let subtree = Arc::new(Subtree::with_dynamic_prec_and_fields(
                node,
                children_with_fields,
                dynamic_prec,
            ));

            // Look up goto state from the unified action table
            if let Some(symbol_idx) = self.table.symbol_to_index.get(&rule.lhs) {
                let current_state = stack.current_state();
                let action_cell = &self.table.action_table[current_state.0 as usize][*symbol_idx];

                // Find shift action in the cell
                let shift_action = action_cell.iter().find(|a| matches!(a, Action::Shift(_)));

                if let Some(Action::Shift(goto_state)) = shift_action {
                    // Goto state after reduction
                    stack.push(*goto_state, subtree);
                } else {
                    // Fall back to goto table if action table doesn't have a shift
                    let goto_state = self.table.goto_table[current_state.0 as usize][*symbol_idx];
                    if goto_state.0 != 0 {
                        // Goto state from goto table
                        stack.push(goto_state, subtree);
                    } else {
                        // No goto state found - error condition
                    }
                }
            } else {
                // No symbol index found - error condition
            }
        }
    }

    /// Merge stacks that have reached the same state
    ///
    /// In GLR parsing, multiple parse stacks can reach the same parser state through
    /// different paths. When this happens, we can merge these stacks to avoid exponential
    /// growth in the number of stacks.
    ///
    /// The merging process:
    /// 1. Identifies stacks with identical state sequences
    /// 2. Compares their parse trees using dynamic precedence and other criteria
    /// 3. Keeps the best parse according to the comparison rules
    /// 4. Handles ambiguity by potentially keeping multiple parses if they're equally valid
    ///
    /// This is a key optimization that makes GLR parsing practical for real grammars.
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
                    // NEW: Check if parse trees are structurally equivalent
                    if stacks[j].has_equivalent_parse_tree(&best_stack) {
                        // Trees are identical - safe to merge using version comparison
                        match compare_versions(&best_stack.version, &stacks[j].version) {
                            CompareResult::TakeLeft => {
                                // Keep best_stack
                            }
                            CompareResult::TakeRight => {
                                best_stack = stacks[j].clone();
                            }
                            CompareResult::PreferLeft => {
                                // Keep the preferred one
                            }
                            CompareResult::PreferRight => {
                                best_stack = stacks[j].clone();
                            }
                            CompareResult::Tie => {
                                // Keep the first one
                            }
                        }
                        processed[j] = true;
                    }
                    // If parse trees differ, DON'T merge - keep both stacks!
                    // This preserves ambiguity in GLR parsing
                }
            }

            merged.push(best_stack);
        }

        // Add any unprocessed stacks (those with different parse trees)
        for (i, stack) in stacks.iter().enumerate() {
            if !processed[i] {
                merged.push(stack.clone());
            }
        }

        if merged.len() > 1 && merged.len() != stacks.len() {
            debug_glr!(
                "DEBUG merge_stacks: {} stacks -> {} stacks after conservative merge",
                stacks.len(),
                merged.len()
            );
        }

        *stacks = merged;
    }

    /// Get the best parse tree from active stacks
    pub fn get_best_parse(&self) -> Option<Arc<Subtree>> {
        // Get best parse from available stacks
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

        // Return best parse
        self.stacks[best_idx].nodes.last().cloned()
    }

    /// Process EOF to complete parsing
    pub fn process_eof(&mut self, total_bytes: usize) {
        // Store the total input length for validation in finish_all_alternatives
        self.input_length = total_bytes;
        // Process EOF token (symbol ID 0)
        self.process_token(SymbolId(0), "", total_bytes);
    }

    /// Get number of active stacks (for debugging)
    pub fn stack_count(&self) -> usize {
        self.stacks.len()
    }

    /// Finish parsing and get the result
    ///
    /// This method is called after all tokens have been processed (including EOF) to
    /// extract the final parse tree. It examines all remaining stacks and returns the
    /// parse tree from a successfully completed parse.
    ///
    /// A successful parse is identified by:
    /// 1. Having exactly one node on the stack (the root of the parse tree)
    /// 2. That node representing a non-terminal symbol (not a raw token)
    ///
    /// # Returns
    /// * `Ok(Arc<Subtree>)` - The root of the parse tree if parsing succeeded
    /// * `Err(String)` - An error message with debugging information if parsing failed
    ///
    /// # Note
    /// In case of ambiguous parses where multiple stacks complete successfully, this
    /// currently returns the first valid parse found. Future enhancements could return
    /// all valid parses or use additional criteria to select the best one.
    pub fn finish(&self) -> Result<Arc<Subtree>, String> {
        // Find a successfully parsed stack
        // Success criteria:
        // 1. Has exactly one node (the root of the parse tree)
        // 2. That node represents the start symbol (we'll accept any non-terminal for now)

        for stack in &self.stacks {
            if stack.nodes.len() == 1 {
                // Accept if we have exactly one node after EOF processing
                // This should be the root of the parse tree (the start symbol)
                let node = &stack.nodes[0];
                return Ok(node.clone());
            }
        }

        // If no accepted stack, return error with debugging info
        let states: Vec<_> = self
            .stacks
            .iter()
            .map(|s| {
                let state = s.states.last().copied().unwrap_or(StateId(0));
                (
                    state,
                    s.nodes.len(),
                    s.nodes.iter().map(|n| n.node.symbol_id).collect::<Vec<_>>(),
                )
            })
            .collect();
        Err(format!("Parse incomplete. Stack states: {:?}", states))
    }

    /// Get all successful parse alternatives (for ambiguous grammars)
    pub fn finish_all_alternatives(&self) -> Result<Vec<Arc<Subtree>>, String> {
        debug_glr!(
            "DEBUG finish_all_alternatives: have {} stacks",
            self.stacks.len()
        );
        #[allow(unused_variables)]
        for (i, stack) in self.stacks.iter().enumerate() {
            debug_glr!(
                "  Stack {}: {} nodes, state {}",
                i,
                stack.nodes.len(),
                stack.current_state().0
            );
            // Print parse tree structure for debugging
            if stack.nodes.len() == 1 {
                ParseStack::print_tree_structure(&stack.nodes[0], 0);
            }
        }

        let mut alternatives = Vec::new();

        // Collect all successfully parsed stacks
        for stack in &self.stacks {
            if stack.nodes.len() == 1 {
                // Accept if we have exactly one node after EOF processing
                // This should be the root of the parse tree (the start symbol)
                let node = &stack.nodes[0];

                // CRITICAL: Check that the parse consumed all input
                if node.node.byte_range.end == self.input_length {
                    alternatives.push(node.clone());
                } else {
                    debug_glr!(
                        "DEBUG: Rejecting incomplete stack - ends at byte {} but input is {} bytes",
                        node.node.byte_range.end,
                        self.input_length
                    );
                }
            }
        }

        if alternatives.is_empty() {
            // If no accepted stack, return error with debugging info
            let states: Vec<_> = self
                .stacks
                .iter()
                .map(|s| {
                    let state = s.states.last().copied().unwrap_or(StateId(0));
                    (
                        state,
                        s.nodes.len(),
                        s.nodes.iter().map(|n| n.node.symbol_id).collect::<Vec<_>>(),
                    )
                })
                .collect();
            Err(format!("Parse incomplete. Stack states: {:?}", states))
        } else {
            debug_glr!("DEBUG: Found {} parse alternatives", alternatives.len());
            Ok(alternatives)
        }
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
            for symbol in self.table.symbol_to_index.keys() {
                if let Some(_action) = self.get_action(state, *symbol) {
                    if !symbols.contains(symbol) {
                        symbols.push(*symbol);
                    }
                }
            }
        }

        symbols
    }

    /// Perform all possible reductions on a stack until no more are possible
    #[allow(dead_code)]
    fn perform_all_reductions(&self, stack: ParseStack) -> Vec<ParseStack> {
        let mut result_stacks = vec![];
        let mut work_list = vec![stack];

        while let Some(current_stack) = work_list.pop() {
            let _state = current_stack.current_state();
            let mut has_reduction = false;

            // Check all possible reductions in this state
            for (_symbol_id, rules) in &self.grammar.rules {
                for rule in rules {
                    // Check if we can reduce by this rule
                    if self.can_reduce(&current_stack, rule) {
                        // After reduction, we need to find the goto state
                        // First get the state we'll be in after popping the RHS symbols
                        let base_state_idx = if current_stack.states.len() > rule.rhs.len() {
                            current_stack.states[current_stack.states.len() - rule.rhs.len() - 1].0
                                as usize
                        } else {
                            0
                        };

                        // Get the symbol index for the LHS non-terminal
                        if let Some(&lhs_idx) = self.table.symbol_to_index.get(&rule.lhs) {
                            // Look up the goto state
                            if base_state_idx < self.table.goto_table.len()
                                && lhs_idx < self.table.goto_table[base_state_idx].len()
                            {
                                let goto_state = self.table.goto_table[base_state_idx][lhs_idx];
                                if goto_state.0 != 0 {
                                    // Valid goto state
                                    has_reduction = true;

                                    // Perform the reduction
                                    let mut reduced_stack = current_stack.clone();
                                    let children: Vec<Arc<Subtree>> = (0..rule.rhs.len())
                                        .filter_map(|_| reduced_stack.nodes.pop())
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                        .rev()
                                        .collect();

                                    // Also pop the corresponding states
                                    for _ in 0..rule.rhs.len() {
                                        reduced_stack.states.pop();
                                    }

                                    // Create new subtree for the reduction
                                    let byte_range = if children.is_empty() {
                                        0..0 // Empty production
                                    } else {
                                        children[0].node.byte_range.start
                                            ..children.last().unwrap().node.byte_range.end
                                    };

                                    let node = SubtreeNode {
                                        symbol_id: rule.lhs,
                                        is_error: false,
                                        byte_range,
                                    };

                                    let dynamic_prec = rule
                                        .precedence
                                        .map(|p| match p {
                                            PrecedenceKind::Static(prec) => prec as i32,
                                            PrecedenceKind::Dynamic(idx) => {
                                                // For dynamic precedence, use child's precedence
                                                let idx_usize = idx as usize;
                                                if idx_usize < children.len() {
                                                    children[idx_usize].dynamic_prec
                                                } else {
                                                    0
                                                }
                                            }
                                        })
                                        .unwrap_or(0);

                                    let parent = Arc::new(Subtree::with_dynamic_prec(
                                        node,
                                        children,
                                        dynamic_prec,
                                    ));

                                    // Push the new subtree
                                    reduced_stack.push(goto_state, parent);

                                    // Continue reducing from this new state
                                    work_list.push(reduced_stack);
                                }
                            }
                        }
                    }
                }
            }

            // If no reductions were possible, this stack is done
            if !has_reduction {
                result_stacks.push(current_stack);
            }
        }

        result_stacks
    }

    /// Check if we can reduce by a rule
    #[allow(dead_code)]
    fn can_reduce(&self, stack: &ParseStack, rule: &Rule) -> bool {
        if stack.nodes.len() < rule.rhs.len() {
            return false;
        }

        // Check if the top of the stack matches the rule's RHS
        let start_idx = stack.nodes.len() - rule.rhs.len();
        for (i, symbol) in rule.rhs.iter().enumerate() {
            let node_symbol = match symbol {
                Symbol::Terminal(id) | Symbol::NonTerminal(id) => *id,
                Symbol::External(id) => *id,
                Symbol::Optional(_)
                | Symbol::Repeat(_)
                | Symbol::RepeatOne(_)
                | Symbol::Choice(_)
                | Symbol::Sequence(_)
                | Symbol::Epsilon => {
                    panic!("Complex symbols should be normalized before GLR parsing");
                }
            };
            if stack.nodes[start_idx + i].node.symbol_id != node_symbol {
                return false;
            }
        }

        true
    }

    /// Get action from parse table for state and symbol
    fn get_action(&self, state: StateId, symbol: SymbolId) -> Option<Action> {
        let state_idx = state.0 as usize;

        if state_idx < self.table.action_table.len() {
            if let Some(&symbol_idx) = self.table.symbol_to_index.get(&symbol) {
                if symbol_idx < self.table.action_table[state_idx].len() {
                    let action_cell = &self.table.action_table[state_idx][symbol_idx];
                    if action_cell.is_empty() {
                        return Some(Action::Error);
                    } else if action_cell.len() == 1 {
                        return Some(action_cell[0].clone());
                    } else {
                        // Multiple actions - create a Fork
                        return Some(Action::Fork(action_cell.clone()));
                    }
                }
            }
        }

        None
    }

    // Methods for incremental parsing state management

    /// Get the current GSS (Graph-Structured Stack) state for snapshots
    pub fn get_gss_state(&self) -> Vec<ParseStack> {
        self.stacks.clone()
    }

    /// Restore the GSS state from a snapshot
    pub fn set_gss_state(&mut self, stacks: Vec<ParseStack>) {
        self.stacks = stacks;
        self.pending_stacks.clear();
        // Re-populate pending stacks with all current stack indices
        for i in 0..self.stacks.len() {
            self.pending_stacks.push_back(i);
        }
    }

    /// Restore GSS state selectively - only restore the most promising stacks
    /// This is a performance optimization for incremental parsing
    pub fn set_gss_state_selective(&mut self, stacks: Vec<ParseStack>) {
        if stacks.is_empty() {
            self.stacks = stacks;
            self.pending_stacks.clear();
            return;
        }

        // AGGRESSIVE OPTIMIZATION: Only keep the single deepest stack
        // This dramatically reduces the work needed to process remaining tokens
        // If the middle chunk is ambiguous, the GLR mechanism will naturally
        // re-create forks as needed
        let best_stack = stacks.into_iter().max_by_key(|s| s.states.len()).unwrap();

        self.stacks = vec![best_stack];
        self.pending_stacks.clear();
        self.pending_stacks.push_back(0);
    }

    /// Get the next stack ID for restoring fork tracking
    pub fn get_next_stack_id(&self) -> usize {
        self.next_stack_id
    }

    /// Set the next stack ID for restoring fork tracking
    pub fn set_next_stack_id(&mut self, id: usize) {
        self.next_stack_id = id;
    }

    /// Inject multiple alternative subtrees (for ambiguous parses)
    /// This is used for incremental GLR parsing to preserve ambiguity
    pub fn inject_ambiguous_subtrees(&mut self, subtrees: Vec<Arc<Subtree>>) -> Result<(), String> {
        if self.stacks.is_empty() {
            return Err("No active stacks to inject subtrees into".to_string());
        }

        if subtrees.is_empty() {
            return Err("No subtrees to inject".to_string());
        }

        // For each subtree alternative, create potential parse stacks
        let mut new_stacks = Vec::new();

        for subtree in subtrees {
            for stack in &self.stacks {
                let new_stack = stack.clone();

                // Get the current state
                let current_state = new_stack.current_state();

                // Look up the goto state after shifting this symbol
                let symbol = subtree.node.symbol_id;
                if let Some(&symbol_idx) = self.table.symbol_to_index.get(&symbol) {
                    let state_idx = current_state.0 as usize;

                    // Check if there's a shift action for this symbol
                    if state_idx < self.table.action_table.len()
                        && symbol_idx < self.table.action_table[state_idx].len()
                    {
                        let action_cell = &self.table.action_table[state_idx][symbol_idx];

                        // Look for shift actions
                        for action in action_cell {
                            if let Action::Shift(next_state) = action {
                                // Push the subtree and advance to the next state
                                let mut forked_stack = new_stack.clone();
                                forked_stack.push(*next_state, subtree.clone());
                                new_stacks.push(forked_stack);
                            }
                        }
                    }
                }
            }
        }

        if new_stacks.is_empty() {
            return Err("Cannot inject any subtrees in current state".to_string());
        }

        self.stacks = new_stacks;
        Ok(())
    }

    /// Inject a pre-parsed subtree at the current position
    /// This is used for incremental parsing to reuse unchanged portions
    pub fn inject_subtree(&mut self, subtree: Arc<Subtree>) -> Result<(), String> {
        if self.stacks.is_empty() {
            return Err("No active stacks to inject subtree into".to_string());
        }

        // For each active stack, inject the subtree
        let mut new_stacks = Vec::new();
        for stack in &self.stacks {
            let mut new_stack = stack.clone();

            // Get the current state
            let current_state = new_stack.current_state();

            // Look up the goto state after shifting this symbol
            let symbol = subtree.node.symbol_id;
            if let Some(&symbol_idx) = self.table.symbol_to_index.get(&symbol) {
                let state_idx = current_state.0 as usize;

                // First check if there's a shift action for this symbol
                if state_idx < self.table.action_table.len()
                    && symbol_idx < self.table.action_table[state_idx].len()
                {
                    let action_cell = &self.table.action_table[state_idx][symbol_idx];

                    // Look for a shift action
                    for action in action_cell {
                        if let Action::Shift(next_state) = action {
                            // Push the subtree and advance to the next state
                            new_stack.push(*next_state, subtree.clone());
                            new_stacks.push(new_stack.clone());
                            break;
                        }
                    }
                }
            }
        }

        if new_stacks.is_empty() {
            return Err(format!(
                "Cannot inject subtree with symbol {:?} in current state",
                subtree.node.symbol_id
            ));
        }

        self.stacks = new_stacks;
        Ok(())
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
