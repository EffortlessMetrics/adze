//! GLR (Generalized LR) Parser Implementation
//!
//! This module implements a GLR parser that can handle ambiguous grammars by maintaining
//! multiple parse stacks simultaneously. When the parser encounters a shift/reduce or
//! reduce/reduce conflict, it forks the parse stack and explores both possibilities.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
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
//! ## Configuration Constants
//!
//! ### Safe Deduplication Threshold
//! Only perform pointer-based deduplication when new_stacks.len() exceeds this threshold.
//! This prevents performance overhead for small stack sets while ensuring correctness
//! for larger sets where duplicate stacks could impact performance.
//! Default: 10. Override with env var RUST_SITTER_SAFE_DEDUP_N for testing.
//!
//! ## Example Usage
//!
//! ```ignore
//! use rust_sitter::glr_parser::GLRParser;
//! use rust_sitter::glr_lexer::GLRLexer;
//! use rust_sitter_ir::{Grammar, SymbolId};
//! use rust_sitter_glr_core::ParseTable;
//!
//! // Create parser with grammar and parse table
//! # fn example() {
//! let grammar: Grammar = /* ... */;
//! # let grammar = Grammar::default();
//! let parse_table: ParseTable = /* ... */;
//! # let parse_table = ParseTable::default();
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
//! # }
//! ```

/// Default threshold for pointer-based dedup.
pub const DEFAULT_SAFE_DEDUP_THRESHOLD: usize = 10;

#[inline]
pub fn safe_dedup_threshold() -> usize {
    if let Some(s) = option_env!("RUST_SITTER_SAFE_DEDUP_N") {
        if let Ok(n) = s.parse::<usize>() {
            return n;
        }
    }
    std::env::var("RUST_SITTER_SAFE_DEDUP_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SAFE_DEDUP_THRESHOLD)
}

use crate::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryState, RecoveryAction};
use crate::subtree::{Subtree, SubtreeNode};
use rust_sitter_glr_core::{
    compare_versions, compare_versions_with_symbols, Action, CompareResult, ParseTable, VersionInfo,
};
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

/// Recovery event for tracking what recovery actions were taken
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum RecoveryEvent {
    /// Synthesized a token (zero-width insertion)
    Insert(SymbolId),
    /// Dropped the current lookahead
    Delete(SymbolId),
    /// Popped N symbols from stacks
    Pop(usize),
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

    /// Number of tokens deleted in a row for recovery
    deleted_in_row: usize,

    /// Number of tokens inserted in a row for recovery
    inserted_in_row: usize,

    /// Pending synthetic tokens to process
    pending_synthetic_tokens: VecDeque<SymbolId>,

    /// Telemetry counters for performance monitoring
    #[cfg(feature = "glr_telemetry")]
    telemetry: TelemetryCounters,
}

/// Dummy telemetry type when feature is disabled
#[cfg(not(feature = "glr_telemetry"))]
#[allow(dead_code)]
struct TelemetryCounters;

/// Telemetry counters for GLR performance monitoring
#[cfg(feature = "glr_telemetry")]
#[derive(Debug, Default, Clone)]
struct TelemetryCounters {
    /// Number of reduce operations performed
    reduce_steps: usize,
    /// Number of epsilon reductions
    epsilon_reduces: usize,
    /// Number of shift operations performed
    shift_steps: usize,
    /// Number of times parser forked
    fork_count: usize,
    /// Total stacks before compression
    tops_before_compress: usize,
    /// Total stacks after compression
    tops_after_compress: usize,
    /// Number of ambiguity packs created
    alts_packed: usize,
    /// Maximum active stacks at any point
    max_active_stacks: usize,
    /// Number of accept actions at EOF
    accept_count: usize,
}

#[allow(dead_code)]
impl GLRParser {
    /// Get telemetry summary (only when telemetry feature is enabled)
    #[cfg(feature = "glr_telemetry")]
    pub fn telemetry_summary(&self) -> String {
        format!(
            "GLR Telemetry:\n  Shifts: {}\n  Reduces: {} (epsilon: {})\n  Forks: {}\n  Compression: {}/{} -> {} (packed: {})\n  Max stacks: {}\n  Accepts: {}",
            self.telemetry.shift_steps,
            self.telemetry.reduce_steps,
            self.telemetry.epsilon_reduces,
            self.telemetry.fork_count,
            self.telemetry.tops_before_compress,
            self.telemetry.tops_after_compress,
            self.telemetry.tops_after_compress,
            self.telemetry.alts_packed,
            self.telemetry.max_active_stacks,
            self.telemetry.accept_count
        )
    }

    /// Helper to update telemetry counters (no-op when feature disabled)
    #[cfg(feature = "glr_telemetry")]
    #[inline]
    fn bump_telemetry(&mut self, f: impl FnOnce(&mut TelemetryCounters)) {
        f(&mut self.telemetry);
    }

    #[cfg(not(feature = "glr_telemetry"))]
    #[inline]
    #[allow(dead_code)]
    fn bump_telemetry(&mut self, _f: impl FnOnce(&mut TelemetryCounters)) {
        // No-op when telemetry is disabled
    }

    /// Calculate priority for an action based on precedence
    #[inline]
    fn action_priority(&self, action: &Action) -> i32 {
        use Action::*;

        // Highest: Accept
        if matches!(action, Accept) {
            return 3_000_000;
        }

        // Pull dynamic precedence if this is a reduce
        let mut prec = 0i32;
        if let Reduce(rid) = action {
            // Get dynamic precedence for this rule
            if (rid.0 as usize) < self.table.dynamic_prec_by_rule.len() {
                prec = self.table.dynamic_prec_by_rule[rid.0 as usize] as i32;
            }

            // Get associativity from the rule: +1 left, -1 right, 0 none
            let assoc_bias = if (rid.0 as usize) < self.table.rule_assoc_by_rule.len() {
                self.table.rule_assoc_by_rule[rid.0 as usize] as i32
            } else {
                0
            };

            // Combine precedence and associativity
            prec = prec.saturating_add(assoc_bias);

            // Bump reduces with positive precedence above plain shift
            if prec > 0 {
                return 2_000_000 + prec;
            }
            // Neutral reduce (slightly below shift to prefer shift in S/R conflicts)
            return 1_500_000 + prec;
        }

        // Plain Shift (default TS policy prefers shift over no-prec reduce)
        if matches!(action, Shift(_)) {
            return 2_000_000;
        }

        0 // Error/other
    }

    /// Get a rule by its ID
    #[allow(dead_code)]
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
            deleted_in_row: 0,
            inserted_in_row: 0,
            pending_synthetic_tokens: VecDeque::new(),
            #[cfg(feature = "glr_telemetry")]
            telemetry: TelemetryCounters::default(),
        }
    }

    /// Get the goto state for a nonterminal after a reduction
    #[inline]
    fn goto_next_state(&self, state: StateId, lhs: SymbolId) -> Option<StateId> {
        let row = self.table.goto_table.get(state.0 as usize)?;
        let col = match self.table.goto_indexing {
            rust_sitter_glr_core::GotoIndexing::NonterminalMap => {
                *self.table.nonterminal_to_index.get(&lhs)?
            }
            rust_sitter_glr_core::GotoIndexing::DirectSymbolId => lhs.0 as usize,
        };
        row.get(col).copied().filter(|&s| s.0 != 0)
    }

    /// Get the start symbol from the grammar
    /// This is the LHS of the first production (production_id 0), or the grammar's start symbol
    #[inline]
    pub fn start_symbol_id(&self) -> SymbolId {
        self.grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .find(|r| r.production_id.0 == 0)
            .map(|r| r.lhs)
            .or_else(|| self.grammar.start_symbol())
            .unwrap_or(SymbolId(1)) // Neutral fallback, not EOF(0)
    }

    /// Enable error recovery with the given configuration
    pub fn enable_error_recovery(&mut self, config: ErrorRecoveryConfig) {
        self.recovery_state = Some(ErrorRecoveryState::new(config.clone()));
        self.error_recovery = Some(config);
    }

    /// Builder method to enable error recovery with the given configuration
    pub fn with_error_recovery(mut self, config: ErrorRecoveryConfig) -> Self {
        self.enable_error_recovery(config);
        self
    }

    /// Process a synthetic token (from recovery)
    fn process_synthetic_token(&mut self, token: SymbolId) {
        // Process synthetic token exactly like a real one but with zero width
        let stacks = std::mem::take(&mut self.stacks);
        let stacks = self.reduce_until_saturated(stacks, token, self.input_length);
        self.stacks = stacks;
        self.shift_synthetic_token(token);
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

        // First process any pending synthetic tokens from recovery
        while let Some(synthetic) = self.pending_synthetic_tokens.pop_front() {
            self.process_synthetic_token(synthetic);
        }

        // Phase 1: Perform all possible reductions until saturation
        let mut stacks_to_process = std::mem::take(&mut self.stacks);
        self.pending_stacks.clear();

        stacks_to_process =
            self.reduce_until_saturated(stacks_to_process, token, byte_offset + text.len());

        // In true GLR, we may have both shift and reduce actions in the same cell
        // This is expected behavior for handling ambiguous grammars

        // Check if any stack has an action for the current token
        // If not, try recovery before Phase 2
        if self.error_recovery.is_some()
            && !stacks_to_process.is_empty()
            && !self.any_stack_has_action_in(&stacks_to_process, token)
        {
            // Restore stacks before attempting recovery
            self.stacks = stacks_to_process;
            if let Some(evt) = self.try_recover(token, false) {
                // Recovery performed: it modified stacks and/or input
                debug_glr!("Recovery performed: {:?}", evt);
                // For deletion, we should just skip this token and continue
                if matches!(evt, RecoveryEvent::Delete(_)) {
                    // Mark all stacks as having encountered an error
                    for stack in &mut self.stacks {
                        stack.version.enter_error();
                    }
                    // Token was deleted, caller should advance input
                    return;
                }
                // For insertion or pop, continue processing
                return;
            } else if self.stacks.is_empty() {
                // No stacks left, parsing failed
                debug_glr!("Recovery failed: no stacks remaining");
                return;
            } else {
                // Recovery couldn't help but we still have stacks
                // Create error node and continue
                debug_glr!("Recovery failed but stacks remain, creating error node");
                // Keep stacks but mark as error
                for stack in &mut self.stacks {
                    stack.version.enter_error();
                }
                return;
            }
        }

        // Phase 2: Process shifts and other actions on all post-reduction stacks
        let mut new_stacks = Vec::new();
        let mut accepted_any = false;
        let mut accept_stacks = Vec::new();

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
                let action_cell = self.table.action_table[state.0 as usize][*symbol_idx].clone();

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

                // Process ALL actions in the cell without collapsing
                // This ensures true GLR behavior by exploring all alternatives
                let mut processed_any = false;

                for action in &action_cell {
                    match action {
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
                            processed_any = true;
                        }

                        Action::Accept => {
                            // Collect accepting stacks for aggregation
                            accepted_any = true;
                            accept_stacks.push(stack.clone());
                            processed_any = true;
                        }

                        Action::Reduce(rule_id) => {
                            // Apply the reduction directly
                            let mut reduced_stack = stack.clone();
                            self.perform_reduction_on_stack(
                                &mut reduced_stack,
                                *rule_id,
                                byte_offset + text.len(),
                            );

                            // Re-saturate with the SAME lookahead to reach fixed point
                            // This ensures cascaded reduces and accepts are discovered
                            let closed = self.reduce_until_saturated(
                                vec![reduced_stack],
                                token,
                                byte_offset + text.len(),
                            );
                            new_stacks.extend(closed);
                            processed_any = true;
                        }

                        Action::Fork(actions) => {
                            // TRUE GLR FORKING! Always fork for ambiguity preservation
                            // Note: we ignore the conflict resolver to maintain all parse alternatives
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
                                        self.perform_reduction_on_stack(
                                            &mut forked,
                                            *rule_id,
                                            byte_offset + text.len(),
                                        );
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
                                                        byte_offset + text.len(),
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

                            processed_any = true;
                        }

                        Action::Recover => {
                            // Handle Recover action - similar to Error but with specific recovery
                            // For now, treat it as an error
                            let mut error_stack = stack.clone();
                            error_stack.version.enter_error();
                            new_stacks.push(error_stack);
                            processed_any = true;
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
                                                if let Some(Action::Shift(new_state)) = shift_action
                                                {
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
                                                    byte_range: byte_offset
                                                        ..byte_offset + text.len(),
                                                },
                                                dynamic_prec: 0,
                                                children: vec![],
                                                alternatives: smallvec::SmallVec::new(),
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
                            processed_any = true;
                        }

                        _ => {
                            // Unknown action type - treat as error
                            let mut error_stack = stack.clone();
                            error_stack.version.enter_error();
                            new_stacks.push(error_stack);
                            processed_any = true;
                        }
                    }
                }

                // If no actions were processed, keep the original stack
                if !processed_any {
                    new_stacks.push(stack);
                }
            } else {
                // No symbol in index - keep the stack
                new_stacks.push(stack);
            }
        }

        // Skip merging to preserve all forks for true GLR behavior
        // This allows maintaining multiple parse paths even if they reach the same state
        // self.merge_stacks(&mut new_stacks);

        // If we have accepting stacks, use only those
        // This aggregates all accepts for the current token
        if accepted_any {
            self.stacks = accept_stacks;
            // Don't process further for this token - we've accepted
            self.pending_stacks.clear();
            return;
        }

        // Safe deduplication: remove exact duplicates (same state and same top node pointer)
        // This keeps ambiguities intact while removing inflated stack counts
        // NOTE: Only dedup if we have many stacks to avoid collapsing necessary ambiguity forks
        if new_stacks.len() > safe_dedup_threshold() {
            use std::ptr;
            new_stacks.dedup_by(|a, b| {
                a.current_state() == b.current_state()
                    && a.nodes.last().is_some()
                    && b.nodes.last().is_some()
                    && ptr::eq(
                        a.nodes.last().unwrap().as_ref(),
                        b.nodes.last().unwrap().as_ref(),
                    )
            });
        }

        // Compress stacks with identical tops to prevent explosion
        new_stacks = self.compress_identical_tops(new_stacks);

        // EOF finalization: prefer Accept or start symbol stacks
        if token == self.table.eof_symbol && !new_stacks.is_empty() {
            // EOF processing - prefer stacks that have Accept action or start symbol
            if let Some(&eof_idx) = self.table.symbol_to_index.get(&token) {
                // First, prefer stacks with Accept action
                let (accepted, rest): (Vec<_>, Vec<_>) = new_stacks.into_iter().partition(|st| {
                    let state_idx = st.current_state().0 as usize;
                    self.table.action_table[state_idx][eof_idx]
                        .iter()
                        .any(|a| matches!(a, Action::Accept))
                });

                new_stacks = if !accepted.is_empty() {
                    accepted
                } else {
                    // Otherwise, prefer stacks whose top symbol is the start symbol
                    let start_symbol = self.start_symbol_id();

                    let (start_tops, others): (Vec<_>, Vec<_>) = rest.into_iter().partition(|st| {
                        st.nodes
                            .last()
                            .is_some_and(|n| n.node.symbol_id == start_symbol)
                    });

                    if !start_tops.is_empty() {
                        start_tops
                    } else {
                        others
                    }
                };
            }
        }

        // Update active stacks
        self.stacks = new_stacks;
        self.pending_stacks = (0..self.stacks.len()).collect();
    }

    /// Perform all possible reductions on the given stacks until no more reductions apply
    ///
    /// This method implements the reduction saturation phase of GLR parsing using a
    /// fixed-point worklist algorithm. It repeatedly applies all possible reductions
    /// until no new stack states are reachable for the given lookahead.
    ///
    /// This is essential for correctness because:
    ///
    /// 1. **Cascading Reductions**: One reduction may enable another. After a reduction
    ///    and GOTO, the new state may have additional reductions for the same lookahead.
    ///
    /// 2. **Completeness**: We must reach a fixed point where no new reductions are
    ///    possible before shifting, to ensure we don't miss valid parses.
    ///
    /// 3. **Fork Handling**: When a fork action contains both reduce and shift actions,
    ///    we process all reductions in this phase and defer shifts to phase 2.
    ///
    /// # Arguments
    /// * `stacks` - The parse stacks to process
    /// * `token` - The lookahead token (used to determine which reductions apply)
    ///
    /// # Returns
    /// A vector of stacks with all reductions applied to fixed point
    fn reduce_until_saturated(
        &mut self,
        stacks: Vec<ParseStack>,
        token: SymbolId,
        lookahead_end: usize,
    ) -> Vec<ParseStack> {
        use std::collections::{HashSet, VecDeque};

        // Track which reductions we've already applied at a given position
        // to avoid infinite epsilon loops
        #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
        struct RedStamp {
            state: StateId,
            rule: RuleId,
            start: usize, // start byte position for precise stamping
            end: usize,   // end byte position to prevent epsilon re-fires at same position
        }
        let mut seen_reductions = HashSet::<RedStamp>::new();

        // Track which (stack_id, state_id) tops we've already expanded for this lookahead.
        // This bounds the worklist and prevents infinite exploration.
        let mut seen_tops = HashSet::<(u16, usize)>::new(); // (state, top_ptr)

        // Worklist of stacks to try reduces from
        let mut worklist = VecDeque::new();

        // Result stacks that are fully saturated (no more reductions possible)
        let mut saturated_stacks = Vec::new();

        // Stacks that have shift actions (need to be preserved for phase 2)
        let mut shift_stacks = Vec::new();

        // Initialize worklist with input stacks
        for stack in stacks {
            let state = stack.current_state();
            // For empty stacks, use stack ID as discriminator to avoid collapsing all empty stacks
            let top_ptr = stack
                .nodes
                .last()
                .map(|n| Arc::as_ptr(n) as usize)
                .unwrap_or(stack.id);
            if seen_tops.insert((state.0, top_ptr)) {
                worklist.push_back(stack);
            }
        }

        // Get the column index for the lookahead token
        // For epsilon reductions, we still need to process even if token is not in table
        let symbol_idx = self.table.symbol_to_index.get(&token).copied();

        // Fixed-point iteration: process stacks until no new tops appear
        // Add iteration cap to prevent pathological grammars from hanging
        let mut steps = 0usize;
        const MAX_STEPS: usize = 64; // very conservative; never reached in sane grammars
        while let Some(stack) = worklist.pop_front() {
            if steps >= MAX_STEPS {
                debug_glr!(
                    "  Warning: Reached max epsilon closure steps ({})",
                    MAX_STEPS
                );
                break;
            }
            steps += 1;
            let state = stack.current_state();

            // Get actions for this state and lookahead
            // If symbol_idx is None (e.g. EOF not in table), still check for epsilon reductions
            let action_cell = if let Some(idx) = symbol_idx {
                self.table.action_table[state.0 as usize][idx].clone()
            } else {
                // No specific lookahead - check for epsilon reductions across all columns
                // This handles EOF and other unmapped symbols
                let mut all_reduces = Vec::new();
                for actions in &self.table.action_table[state.0 as usize] {
                    for action in actions {
                        if let Action::Reduce(rid) = action {
                            // Check if this is an epsilon reduction
                            let rhs_len = self.table.rules[rid.0 as usize].rhs_len as usize;
                            if rhs_len == 0 && !all_reduces.contains(action) {
                                all_reduces.push(action.clone());
                            }
                        }
                    }
                }
                all_reduces
            };

            debug_glr!(
                "DEBUG reduce_closure: Processing state {} for token {} ({} actions)",
                state.0,
                token.0,
                action_cell.len()
            );

            // Extract reduce actions from the specific column
            let mut reduces: Vec<(Action, i32)> = action_cell
                .iter()
                .filter_map(|a| match a {
                    Action::Reduce(_rid) => Some((a.clone(), self.action_priority(a))),
                    _ => None,
                })
                .collect();

            // At EOF, if column lacks epsilon reductions, also pull them from entire row
            // This ensures cascading epsilon reductions complete to the start symbol
            let is_eof = token == self.table.eof_symbol;
            if is_eof {
                let has_eps_in_col = reduces.iter().any(|(a, _)| {
                    matches!(a, Action::Reduce(rid) if self.table.rules[rid.0 as usize].rhs_len == 0)
                });

                if !has_eps_in_col {
                    // Include row-wide epsilon reductions, dedup by rule id
                    let mut _added_count = 0;
                    for actions in &self.table.action_table[state.0 as usize] {
                        for a in actions {
                            if let Action::Reduce(rid) = a {
                                if self.table.rules[rid.0 as usize].rhs_len == 0
                                    && !reduces.iter().any(
                                        |(b, _)| matches!(b, Action::Reduce(r2) if r2.0 == rid.0),
                                    )
                                {
                                    reduces.push((a.clone(), self.action_priority(a)));
                                    _added_count += 1;
                                }
                            }
                        }
                    }
                }
            }

            // Sort by priority (highest first)
            reduces.sort_by_key(|(_, prio)| -prio);

            // Check if this stack also has shift actions (needs to be preserved)
            let has_shift = action_cell.iter().any(|a| matches!(a, Action::Shift(_)));
            if has_shift {
                debug_glr!("  Stack has shift action - preserving for phase 2");
                shift_stacks.push(stack.clone());
            }

            // Check for other non-reduce actions
            let has_accept = action_cell.iter().any(|a| matches!(a, Action::Accept));
            if has_accept {
                debug_glr!("  Stack has accept action - preserving");
                saturated_stacks.push(stack.clone());
            }

            if reduces.is_empty() {
                // No reduces available - this stack is saturated
                if !has_shift && !has_accept {
                    saturated_stacks.push(stack);
                }
                continue;
            }

            // Apply each reduce action
            let mut any_reduction_applied = false;
            for (reduce_action, _) in reduces {
                let rule_id = match reduce_action {
                    Action::Reduce(rid) => rid,
                    _ => continue,
                };

                // Guard against repeated application at same position (epsilon loop prevention)
                // Use the parse table to check if this is an epsilon reduction
                let rhs_len = self.table.rules[rule_id.0 as usize].rhs_len as usize;

                // Only stamp epsilon reductions to prevent loops
                if rhs_len == 0 {
                    let (start_byte, end_byte) = if let Some(n) = stack.nodes.last() {
                        (n.node.byte_range.start, n.node.byte_range.end)
                    } else {
                        (lookahead_end, lookahead_end)
                    };
                    let stamp = RedStamp {
                        state: stack.current_state(),
                        rule: rule_id,
                        start: start_byte,
                        end: end_byte,
                    };

                    if !seen_reductions.insert(stamp) {
                        debug_glr!(
                            "  Skipping epsilon re-fire: state {} rule {} at {}..{}",
                            stamp.state.0,
                            rule_id.0,
                            stamp.start,
                            stamp.end
                        );
                        continue;
                    }
                }
                // No stamping for non-epsilon reductions

                debug_glr!("  Applying reduction: rule {}", rule_id.0);

                // Fork the stack for this reduction
                let mut reduced_stack = stack.fork(self.next_stack_id);
                self.next_stack_id += 1;

                // Apply the reduction (this will pop symbols and push via GOTO)
                self.perform_reduction_on_stack(&mut reduced_stack, rule_id, lookahead_end);

                // The new top state after GOTO might have more reduces for the same lookahead
                let new_state = reduced_stack.current_state();

                // Use pointer-based key to match closure-local dedup
                // For empty stacks, use stack ID to avoid collapsing them
                let top_ptr = reduced_stack
                    .nodes
                    .last()
                    .map(|n| Arc::as_ptr(n) as usize)
                    .unwrap_or(reduced_stack.id);
                let key = (new_state.0, top_ptr);

                if seen_tops.insert(key) {
                    // This is a new top we haven't explored - add to worklist for cascading
                    debug_glr!(
                        "  New top reached: state {} - adding to worklist",
                        new_state.0
                    );
                    worklist.push_back(reduced_stack);
                    any_reduction_applied = true;
                } else {
                    // We've already processed this top - it's saturated
                    debug_glr!("  Top already seen: state {} - saturated", new_state.0);
                    saturated_stacks.push(reduced_stack);
                }
            }

            // If no reductions were applied from this stack and it has no shift/accept,
            // then this stack is fully saturated
            if !any_reduction_applied && !has_shift && !has_accept {
                saturated_stacks.push(stack);
            }
        }

        // Combine saturated stacks and shift stacks
        let mut result = saturated_stacks;
        result.extend(shift_stacks);

        // Skip merging to preserve all reduction paths for true GLR behavior
        // self.merge_stacks(&mut result);

        // Closure-local deduplication: drop exact duplicates by (state, top-node pointer)
        // This prevents epsilon chains from blowing up
        use std::ptr;
        // Don't remove stacks with no nodes - they may be valid initial states
        result.sort_by_key(|s| {
            (
                s.current_state().0,
                s.nodes.last().map(|n| Arc::as_ptr(n) as usize).unwrap_or(0),
            )
        });
        result.dedup_by(|a, b| {
            // Only dedup if both stacks have the same state AND the same top node
            a.current_state() == b.current_state()
                && match (a.nodes.last(), b.nodes.last()) {
                    (Some(node_a), Some(node_b)) => ptr::eq(node_a.as_ref(), node_b.as_ref()),
                    (None, None) => true, // Two stacks with no nodes and same state are duplicates
                    _ => false,           // Different node counts means different stacks
                }
        });

        debug_glr!(
            "DEBUG reduce_closure: Fixed point reached with {} stacks",
            result.len()
        );

        result
    }

    // ================================================================================
    // Stack compression to prevent explosion
    // ================================================================================

    /// Compress stacks with identical tops to prevent explosion
    /// This preserves all derivations by packing alternatives at the top
    fn compress_identical_tops(&mut self, mut stacks: Vec<ParseStack>) -> Vec<ParseStack> {
        use std::collections::HashMap;

        // If we have few stacks, no need to compress
        if stacks.len() <= 10 {
            return stacks;
        }

        #[derive(Hash, Eq, PartialEq)]
        struct TopKey {
            state: StateId,
            symbol: SymbolId,
            start: usize,
            end: usize,
        }

        // Map from top key to index in output vector
        let mut keep: HashMap<TopKey, usize> = HashMap::new();
        let mut out: Vec<ParseStack> = Vec::new();
        #[cfg_attr(not(feature = "glr_telemetry"), allow(unused_variables))]
        let mut packed_count = 0usize;

        for mut stack in stacks.drain(..) {
            // Get the top node info, if any
            let key = if let Some(top) = stack.nodes.last() {
                TopKey {
                    state: stack.current_state(),
                    symbol: top.node.symbol_id,
                    start: top.node.byte_range.start,
                    end: top.node.byte_range.end,
                }
            } else {
                // Stack with no nodes - just check state
                TopKey {
                    state: stack.current_state(),
                    symbol: SymbolId(u16::MAX),
                    start: 0,
                    end: 0,
                }
            };

            if let Some(&idx) = keep.get(&key) {
                // We already have a stack with this top - merge ambiguity
                let kept = &mut out[idx];

                // Pop the tops from both stacks
                if let (Some(new_top), Some(kept_top)) = (stack.nodes.pop(), kept.nodes.pop()) {
                    // Merge the two tops, preserving all alternatives
                    let merged_subtree = Arc::try_unwrap(kept_top)
                        .unwrap_or_else(|arc| (*arc).clone())
                        .merge_ambiguous(new_top);

                    // Push the merged top back
                    kept.nodes.push(Arc::new(merged_subtree));
                    packed_count += 1;

                    // Keep the highest dynamic precedence
                    if stack.version.dynamic_prec > kept.version.dynamic_prec {
                        kept.version.dynamic_prec = stack.version.dynamic_prec;
                    }
                }
                // Otherwise stack has no nodes, just drop it
            } else {
                // First time seeing this top
                keep.insert(key, out.len());
                out.push(stack);
            }
        }

        #[cfg_attr(not(feature = "glr_telemetry"), allow(unused_variables))]
        let input_count = stacks.len() + out.len();
        #[cfg(feature = "glr_telemetry")]
        {
            self.bump_telemetry(|t| {
                t.tops_before_compress += input_count;
                t.tops_after_compress += out.len();
                t.alts_packed += packed_count;
            });
        }

        debug_glr!(
            "Compressed {} stacks down to {} unique tops ({} packed)",
            input_count,
            out.len(),
            packed_count
        );

        out
    }

    // ================================================================================
    // GLR-aware error recovery helpers
    // ================================================================================

    /// Check if any active stack has an action for the given token
    #[inline]
    fn any_stack_has_action(&self, lookahead: SymbolId) -> bool {
        let Some(&col) = self.table.symbol_to_index.get(&lookahead) else {
            return false;
        };
        self.stacks.iter().any(|stack| {
            let s = stack.current_state();
            !self.table.action_table[s.0 as usize][col].is_empty()
        })
    }

    fn any_stack_has_action_in(&self, stacks: &[ParseStack], lookahead: SymbolId) -> bool {
        let Some(&col) = self.table.symbol_to_index.get(&lookahead) else {
            return false;
        };
        stacks.iter().any(|stack| {
            let s = stack.current_state();
            !self.table.action_table[s.0 as usize][col].is_empty()
        })
    }

    /// Check if any stack can shift or reduce for the given terminal symbol
    /// Note: This only checks terminals, not nonterminals (which use goto_table)
    #[inline]
    fn can_shift_or_reduce(&self, sym: SymbolId) -> bool {
        // symbol_to_index should only contain terminals
        let Some(&col) = self.table.symbol_to_index.get(&sym) else {
            debug_glr!("  Terminal {:?} not in symbol_to_index map", sym);
            return false;
        };
        let result = self.stacks.iter().any(|stack| {
            let s = stack.current_state();
            if s.0 as usize >= self.table.action_table.len() {
                debug_glr!("  State {} is out of bounds!", s.0);
                return false;
            }
            if col >= self.table.action_table[s.0 as usize].len() {
                debug_glr!("  Column {} is out of bounds for state {}!", col, s.0);
                return false;
            }
            let cell = &self.table.action_table[s.0 as usize][col];
            if !cell.is_empty() {
                debug_glr!(
                    "  State {} has action for symbol {:?}: {:?}",
                    s.0,
                    sym,
                    cell
                );
            } else {
                debug_glr!("  State {} has NO action for symbol {:?}", s.0, sym);
            }
            !cell.is_empty()
        });
        if !result {
            debug_glr!(
                "  No stack has action for symbol {:?} (checked {} stacks)",
                sym,
                self.stacks.len()
            );
        }
        result
    }

    /// Insert a synthetic token with zero width into the input stream
    fn insert_token_zero_width(&mut self, sym: SymbolId) {
        self.pending_synthetic_tokens.push_back(sym);
    }

    /// Perform shifts for a synthetic token across all GLR stacks
    fn shift_synthetic_token(&mut self, sym: SymbolId) {
        let mut new_stacks = Vec::new();

        for stack in self.stacks.drain(..) {
            let state = stack.current_state();
            let mut shifted = false;

            if let Some(&symbol_idx) = self.table.symbol_to_index.get(&sym) {
                let action_cell = &self.table.action_table[state.0 as usize][symbol_idx];

                // Handle shift actions for the synthetic token
                for action in action_cell {
                    if let Action::Shift(new_state) = action {
                        let mut new_stack = stack.clone();
                        // Create synthetic node with zero-width range
                        new_stack.push(
                            *new_state,
                            Arc::new(Subtree::new(
                                SubtreeNode {
                                    symbol_id: sym,
                                    is_error: true, // Mark synthetic tokens as error nodes
                                    byte_range: 0..0, // Zero-width synthetic token
                                },
                                Vec::new(), // No children for synthetic token
                            )),
                        );
                        new_stacks.push(new_stack);
                        shifted = true;
                        break; // Take first shift
                    }
                }
            }

            // If we couldn't shift on this stack, keep the original stack
            // so we don't lose all paths
            if !shifted {
                new_stacks.push(stack);
            }
        }

        self.stacks = new_stacks;
    }

    /// Pop symbols from stacks towards sync tokens (panic-mode recovery)
    fn pop_towards_sync(&mut self, lookahead: SymbolId) -> Option<usize> {
        let config = self.error_recovery.as_ref()?;

        let mut target_set = config.sync_tokens.clone().into_vec();
        target_set.push(lookahead);

        const POP_BOUND: usize = 8;
        let mut max_popped = 0usize;
        let mut progress = false;

        // Try popping from each stack
        let mut modified_stacks = Vec::new();
        for stack in self.stacks.iter() {
            let mut test_stack = stack.clone();
            let mut pops = 0usize;

            while pops < POP_BOUND && test_stack.states.len() > 1 {
                let state = test_stack.current_state();

                // Check if any target token has an action in this state
                let has_action = target_set.iter().any(|&sym| {
                    self.table.symbol_to_index.get(&sym).is_some_and(|&col| {
                        !self.table.action_table[state.0 as usize][col].is_empty()
                    })
                });

                if has_action {
                    progress = true;
                    max_popped = max_popped.max(pops);
                    modified_stacks.push(test_stack);
                    break;
                }

                // Pop one symbol
                if test_stack.states.len() > 1 {
                    test_stack.states.pop();
                    test_stack.nodes.pop();
                    pops += 1;
                } else {
                    break;
                }
            }
        }

        if progress {
            if !modified_stacks.is_empty() {
                self.stacks = modified_stacks;
            }
            Some(max_popped)
        } else {
            None
        }
    }

    /// Main GLR-aware recovery driver
    fn try_recover(&mut self, lookahead: SymbolId, _eof: bool) -> Option<RecoveryEvent> {
        // Check if recovery is configured and we have stacks to recover
        if self.error_recovery.is_none() || self.stacks.is_empty() {
            debug_glr!(
                "try_recover: no recovery (config={:?}, stacks={})",
                self.error_recovery.is_some(),
                self.stacks.len()
            );
            return None;
        }

        // 1) Try synthesizing an insertion if it unlocks progress
        let max_insertions = self.error_recovery.as_ref().unwrap().max_token_insertions;
        if self.inserted_in_row < max_insertions {
            let candidates = self
                .error_recovery
                .as_ref()
                .unwrap()
                .insert_candidates
                .clone();
            debug_glr!(
                "recovery: checking {} insert candidates with {} stacks (eof={})",
                candidates.len(),
                self.stacks.len(),
                _eof
            );
            #[allow(unused_variables)]
            for stack in &self.stacks {
                debug_glr!("  Stack in state {}", stack.current_state().0);
            }
            for tok in candidates {
                debug_glr!("recovery: checking if {:?} would help (eof={})", tok, _eof);
                if self.can_shift_or_reduce(tok) {
                    debug_glr!("recovery: INSERT {:?} would help", tok);
                    self.insert_token_zero_width(tok);
                    self.inserted_in_row += 1;
                    self.deleted_in_row = 0;

                    // Process the synthetic token through closure and shift
                    let stacks = std::mem::take(&mut self.stacks);
                    let stacks = self.reduce_until_saturated(stacks, tok, self.input_length);
                    self.stacks = stacks;
                    self.shift_synthetic_token(tok);

                    return Some(RecoveryEvent::Insert(tok));
                }
            }
        }

        // 2) Try popping towards sync tokens
        if let Some(popped) = self.pop_towards_sync(lookahead) {
            debug_glr!("recovery: POP {} symbols towards sync", popped);
            self.deleted_in_row = 0;
            self.inserted_in_row = 0;

            // After pops, attempt closure again at same lookahead
            let stacks = std::mem::take(&mut self.stacks);
            let stacks = self.reduce_until_saturated(stacks, lookahead, self.input_length);
            self.stacks = stacks;

            if self.any_stack_has_action(lookahead) {
                return Some(RecoveryEvent::Pop(popped));
            }
            // Fall through to try deletion
        }

        // 3) Token deletion: skip current token
        let max_deletions = self.error_recovery.as_ref().unwrap().max_token_deletions;
        if !_eof && self.deleted_in_row < max_deletions {
            debug_glr!("recovery: DELETE {:?}", lookahead);
            self.deleted_in_row += 1;
            self.inserted_in_row = 0;

            // Create error node for the deleted token
            // In real use, advance input cursor here

            return Some(RecoveryEvent::Delete(lookahead));
        }

        None
    }

    /// Perform a reduction on a specific stack
    fn perform_reduction_on_stack(
        &mut self,
        stack: &mut ParseStack,
        rule_id: RuleId,
        lookahead_end: usize,
    ) {
        debug_glr!(
            "DEBUG: Performing reduction with rule {} on stack in state {}",
            rule_id.0,
            stack.current_state().0
        );
        debug_glr!("  Stack has {} nodes before reduction", stack.nodes.len());

        // Perform reduction
        // Find the rule in the grammar
        if let Some(rule) = self
            .grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .find(|r| r.production_id.0 == rule_id.0)
        {
            debug_glr!(
                "  Rule: {:?} -> {:?} ({} symbols)",
                rule.lhs,
                rule.rhs,
                rule.rhs.len()
            );
            let children = stack.pop(rule.rhs.len());
            debug_glr!(
                "  Popped {} children, stack now has {} nodes",
                children.len(),
                stack.nodes.len()
            );

            // Create new subtree for the reduction
            // For epsilon reductions (empty RHS), use lookahead_end as the position
            let byte_range = if children.is_empty() {
                // Epsilon: zero-width span at the current lookahead end
                lookahead_end..lookahead_end
            } else {
                // Normal: span from first child start to last child end
                children[0].node.byte_range.start..children.last().unwrap().node.byte_range.end
            };

            let node = SubtreeNode {
                symbol_id: rule.lhs,
                is_error: false,
                byte_range,
            };

            // Apply field mappings to children
            let children_with_fields = if rule.fields.is_empty() {
                // No fields, use FIELD_NONE for all children
                children
                    .into_iter()
                    .map(crate::subtree::ChildEdge::new_without_field)
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

                    result.push(crate::subtree::ChildEdge::new(child, field_id));
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

            // Look up goto state for the nonterminal after reduction
            let current_state = stack.current_state();
            if let Some(new_state) = self.goto_next_state(current_state, rule.lhs) {
                debug_glr!(
                    "  GOTO: state {} -> state {} for symbol {}",
                    current_state.0,
                    new_state.0,
                    rule.lhs.0
                );
                stack.push(new_state, subtree);
            } else {
                debug_glr!(
                    "  ERROR: No GOTO found for symbol {} from state {}",
                    rule.lhs.0,
                    current_state.0
                );
                // Can't continue with this reduction path
            }
        } else {
            debug_glr!("  ERROR: Rule {} not found in grammar", rule_id.0);
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
    #[allow(dead_code)]
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
        debug_glr!("get_best_parse: {} stacks available", self.stacks.len());

        // Get best parse from available stacks
        if self.stacks.is_empty() {
            return None;
        }

        // Find the best stack according to version comparison
        let mut best_idx = 0;
        for i in 1..self.stacks.len() {
            let left = &self.stacks[best_idx];
            let right = &self.stacks[i];
            let left_sym = left
                .nodes
                .last()
                .map(|n| n.node.symbol_id)
                .unwrap_or(SymbolId(0));
            let right_sym = right
                .nodes
                .last()
                .map(|n| n.node.symbol_id)
                .unwrap_or(SymbolId(0));
            match compare_versions_with_symbols(&left.version, &right.version, left_sym, right_sym)
            {
                CompareResult::TakeRight | CompareResult::PreferRight => {
                    best_idx = i;
                }
                _ => {}
            }
        }

        debug_glr!(
            "Best stack has {} nodes, current state: {}",
            self.stacks[best_idx].nodes.len(),
            self.stacks[best_idx].current_state().0
        );

        // Return best parse
        self.stacks[best_idx].nodes.last().cloned()
    }

    /// Try to reach a state that can accept EOF by repeatedly applying
    /// insertion/pop recovery. Never attempt deletion at EOF.
    fn drive_recovery_until_eof_action(&mut self) {
        if self.error_recovery.is_none() || self.stacks.is_empty() {
            return;
        }
        let eof = self.table.eof_symbol;
        debug_glr!(
            "drive_recovery_until_eof_action: starting with {} stacks",
            self.stacks.len()
        );

        // Hard bound: at most a few iterations (guards infinite loops).
        #[allow(unused_variables)] // Used in debug_glr! macro
        for i in 0..8 {
            // Close first with EOF as lookahead to expose any reduces
            let stacks = std::mem::take(&mut self.stacks);
            let stacks = self.reduce_until_saturated(stacks, eof, self.input_length);
            self.stacks = stacks;

            // Now check if any stack can handle EOF
            if self.any_stack_has_action(eof) {
                debug_glr!(
                    "drive_recovery_until_eof_action: found action for EOF after {} iterations",
                    i
                );
                break;
            }

            debug_glr!(
                "drive_recovery_until_eof_action: iteration {} with {} stacks",
                i,
                self.stacks.len()
            );

            // Try recovery (insert/pop only, never delete at EOF)
            match self.try_recover(eof, /*eof=*/ true) {
                Some(RecoveryEvent::Insert(_tok)) => {
                    debug_glr!("drive_recovery_until_eof_action: inserted {:?}", _tok);
                    // Process pending synthetic tokens
                    while let Some(synthetic) = self.pending_synthetic_tokens.pop_front() {
                        self.process_synthetic_token(synthetic);
                    }
                    continue;
                }
                Some(RecoveryEvent::Pop(_n)) => {
                    debug_glr!("drive_recovery_until_eof_action: popped {} symbols", _n);
                    continue;
                }
                _ => {
                    debug_glr!("drive_recovery_until_eof_action: no recovery possible");
                    break;
                }
            }
        }
    }

    /// Process EOF to complete parsing
    pub fn process_eof(&mut self, total_bytes: usize) {
        // Store the total input length for validation in finish_all_alternatives
        self.input_length = total_bytes;

        // Give recovery a chance to make EOF shiftable/reduceable
        self.drive_recovery_until_eof_action();

        // Process EOF token using the table's EOF symbol
        let eof_symbol = self.table.eof_symbol;
        self.process_token(eof_symbol, "", total_bytes);
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
        // Find the best successfully parsed stack
        // Success criteria:
        // 1. Has exactly one node (the root of the parse tree)
        // 2. That node represents the start symbol (we'll accept any non-terminal for now)
        let mut best: Option<&ParseStack> = None;
        for stack in &self.stacks {
            debug_glr!("finish: stack has {} nodes", stack.nodes.len());
            if stack.nodes.len() == 1 {
                if let Some(current) = best {
                    let current_sym = current.nodes[0].node.symbol_id;
                    let new_sym = stack.nodes[0].node.symbol_id;
                    match compare_versions_with_symbols(
                        &current.version,
                        &stack.version,
                        current_sym,
                        new_sym,
                    ) {
                        CompareResult::TakeRight | CompareResult::PreferRight => {
                            best = Some(stack);
                        }
                        _ => {}
                    }
                } else {
                    best = Some(stack);
                }
            }
        }

        if let Some(best_stack) = best {
            let node = &best_stack.nodes[0];
            // Check if we encountered errors during parsing (e.g., deleted tokens)
            if best_stack.version.in_error && self.error_recovery.is_some() {
                // Wrap in error node to indicate parse had errors
                let error_node = Arc::new(Subtree::new(
                    SubtreeNode {
                        symbol_id: SymbolId(u16::MAX), // Special error symbol
                        is_error: true,
                        byte_range: node.node.byte_range.clone(),
                    },
                    vec![node.clone()],
                ));
                return Ok(error_node);
            }
            return Ok(node.clone());
        }

        // If no stack had a single node, check for partial parse trees with errors
        for stack in &self.stacks {
            if !stack.nodes.is_empty() {
                let has_error =
                    stack.nodes.iter().any(|n| n.node.is_error) || stack.version.in_error;
                if has_error && self.error_recovery.is_some() {
                    let error_node = Arc::new(Subtree::new(
                        SubtreeNode {
                            symbol_id: SymbolId(u16::MAX), // Special error symbol
                            is_error: true,
                            byte_range: 0..self.input_length,
                        },
                        stack.nodes.clone(),
                    ));
                    return Ok(error_node);
                }
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

                        // Get the nonterminal index for the LHS non-terminal
                        if let Some(&lhs_idx) = self.table.nonterminal_to_index.get(&rule.lhs) {
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
                        // Multiple actions - create a Fork with sorted actions
                        let mut sorted_actions = action_cell.clone();
                        sorted_actions.sort_by_key(|a| -self.action_priority(a));
                        return Some(Action::Fork(sorted_actions));
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
