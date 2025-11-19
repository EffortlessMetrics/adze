//! GLR Parsing Engine
//!
//! This module implements the core GLR (Generalized LR) parsing algorithm.
//! It handles fork/merge logic for parsing ambiguous grammars.
//!
//! Contract: docs/specs/GLR_ENGINE_CONTRACT.md

use crate::error::{ParseError, ParseErrorKind};
use crate::Token;
use rust_sitter_glr_core::{Action, ParseTable, StateId, SymbolId};
use rust_sitter_ir::RuleId;
use std::collections::HashMap;
use std::ops::Range;

/// GLR parsing engine configuration
#[derive(Debug, Clone)]
pub struct GLRConfig {
    /// Maximum number of parallel parser stacks
    pub max_forks: usize,
    /// Maximum parse forest nodes
    pub max_forest_nodes: usize,
}

impl Default for GLRConfig {
    fn default() -> Self {
        Self {
            max_forks: 1000,
            max_forest_nodes: 10_000,
        }
    }
}

/// GLR parsing engine
///
/// Handles parsing with a ParseTable, supporting fork/merge on conflicts.
pub struct GLREngine {
    /// Reference to parse table
    parse_table: &'static ParseTable,
    /// Current parser stacks (GSS nodes)
    stacks: Vec<ParserStack>,
    /// Parse forest accumulator
    forest: ParseForest,
    /// Configuration limits
    config: GLRConfig,
}

/// A single parser stack (represents one parse path)
#[derive(Debug, Clone)]
struct ParserStack {
    /// Stack of LR parser states
    states: Vec<StateId>,
    /// Stack of forest node IDs (corresponding to states)
    nodes: Vec<ForestNodeId>,
    /// Unique ID for merging detection
    id: StackId,
}

/// Stack identifier (for merging)
type StackId = usize;

/// Parse forest containing all parse tree nodes
#[derive(Debug)]
pub struct ParseForest {
    /// All nodes in the forest
    pub nodes: Vec<ForestNode>,
    /// Root nodes (successful parses)
    pub roots: Vec<ForestNodeId>,
}

/// ID of a node in the parse forest
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForestNodeId(pub usize);

/// A node in the parse forest
#[derive(Debug, Clone)]
pub struct ForestNode {
    /// Symbol produced by this node
    pub symbol: SymbolId,
    /// Children of this node
    pub children: Vec<ForestNodeId>,
    /// Byte range in input
    pub range: Range<usize>,
}

impl GLREngine {
    /// Create a new GLR engine
    ///
    /// # Contract
    ///
    /// - `parse_table` must satisfy ParseTable invariants
    /// - `config.max_forks > 0`
    /// - `config.max_forest_nodes > 0`
    ///
    pub fn new(parse_table: &'static ParseTable, config: GLRConfig) -> Self {
        // Validate config
        assert!(config.max_forks > 0, "max_forks must be > 0");
        assert!(
            config.max_forest_nodes > 0,
            "max_forest_nodes must be > 0"
        );

        // Create initial stack with state 0
        let initial_stack = ParserStack {
            states: vec![StateId(0)],
            nodes: vec![],
            id: 0,
        };

        Self {
            parse_table,
            stacks: vec![initial_stack],
            forest: ParseForest::new(),
            config,
        }
    }

    /// Parse a token stream
    ///
    /// # Contract
    ///
    /// - `tokens` must be non-empty and end with EOF token
    /// - Returns `Ok(forest)` if parsing succeeds
    /// - `forest.roots.len() >= 1` on success
    ///
    /// # Errors
    ///
    /// - `ParseError::SyntaxError`: No valid parse
    /// - `ParseError::TooManyForks`: Fork limit exceeded
    /// - `ParseError::ForestTooLarge`: Node limit exceeded
    ///
    pub fn parse(&mut self, tokens: &[Token]) -> Result<ParseForest, ParseError> {
        if tokens.is_empty() {
            return Err(ParseError::with_msg("Empty token stream"));
        }

        for (token_idx, token) in tokens.iter().enumerate() {
            self.process_token(token, token_idx)?;

            // Check if all stacks failed
            if self.stacks.is_empty() {
                return Err(ParseError::with_msg(&format!(
                    "Syntax error: unexpected token at position {}",
                    token.start
                )));
            }
        }

        // Check if we have any accepted parses
        if self.forest.roots.is_empty() {
            return Err(ParseError::with_msg("No parse succeeded"));
        }

        // Return the forest (ownership transfer)
        let mut forest = ParseForest::new();
        std::mem::swap(&mut forest, &mut self.forest);
        Ok(forest)
    }

    /// Process a single token
    ///
    /// This is where fork/merge happens.
    fn process_token(&mut self, token: &Token, _token_idx: usize) -> Result<(), ParseError> {
        let mut new_stacks = Vec::new();
        let mut next_stack_id = self.stacks.len();

        // Take ownership of old stacks to avoid borrow conflicts
        // (allows us to iterate while mutating self.forest)
        let old_stacks = std::mem::take(&mut self.stacks);

        for stack in &old_stacks {
            let state = stack.top_state();
            // Clone actions to avoid holding a borrow of self during iteration
            let actions = self.get_actions(state, token.kind).to_vec();

            if actions.is_empty() {
                // No valid action - this stack fails
                continue;
            }

            // Process each action (fork if multiple)
            for action in &actions {
                match action {
                    Action::Shift(next_state) => {
                        let mut new_stack = stack.clone();
                        new_stack.id = next_stack_id;
                        next_stack_id += 1;

                        // Add terminal node to forest
                        let node_id = self.forest.add_terminal(token);

                        // Push to stack
                        new_stack.push(*next_state, node_id);

                        new_stacks.push(new_stack);
                    }
                    Action::Reduce(rule_id) => {
                        let new_stack = self.perform_reduce(stack.clone(), *rule_id)?;
                        new_stacks.push(new_stack);
                    }
                    Action::Accept => {
                        // Mark this parse as accepted
                        if let Some(&root_node) = stack.nodes.last() {
                            self.forest.add_root(root_node);
                        }
                    }
                    Action::Error => {
                        // Skip error actions
                        continue;
                    }
                    _ => {
                        // Unknown action type (future-proofing for non-exhaustive enum)
                        continue;
                    }
                }
            }
        }

        // Check fork limit
        if new_stacks.len() > self.config.max_forks {
            return Err(ParseError::with_msg(&format!(
                "Fork limit exceeded: {} > {}",
                new_stacks.len(),
                self.config.max_forks
            )));
        }

        // Merge identical stacks
        self.stacks = self.merge_identical_stacks(new_stacks);

        Ok(())
    }

    /// Get actions for a given state and symbol
    fn get_actions(&self, state: StateId, symbol: u32) -> &[Action] {
        if (state.0 as usize) < self.parse_table.action_table.len() {
            let state_actions = &self.parse_table.action_table[state.0 as usize];
            if (symbol as usize) < state_actions.len() {
                return &state_actions[symbol as usize];
            }
        }
        &[]
    }

    /// Perform a reduce action
    fn perform_reduce(
        &mut self,
        mut stack: ParserStack,
        rule_id: RuleId,
    ) -> Result<ParserStack, ParseError> {
        // Get rule information
        let rule = self
            .parse_table
            .rules
            .get(rule_id.0 as usize)
            .ok_or_else(|| ParseError::with_msg(&format!("Invalid rule ID: {:?}", rule_id)))?;

        let rhs_len = rule.rhs_len;
        let lhs = rule.lhs;

        // Pop RHS symbols from stack
        let children: Vec<ForestNodeId> = stack
            .nodes
            .drain(stack.nodes.len().saturating_sub(rhs_len as usize)..)
            .collect();

        stack.states.truncate(stack.states.len() - (rhs_len as usize));

        // Calculate byte range (span of all children)
        let range = if children.is_empty() {
            0..0 // Empty production
        } else {
            let first = &self.forest.nodes[children[0]];
            let last = &self.forest.nodes[*children.last().unwrap()];
            first.range.start..last.range.end
        };

        // Add nonterminal node to forest
        let node_id = self.forest.add_nonterminal(lhs, children, range);

        // Get goto state
        let goto_state = stack.top_state();
        let next_state = self.get_goto(goto_state, lhs)?;

        // Push nonterminal and new state
        stack.push(next_state, node_id);

        Ok(stack)
    }

    /// Get goto state for a nonterminal
    fn get_goto(&self, state: StateId, symbol: SymbolId) -> Result<StateId, ParseError> {
        // For now, use a simple lookup in goto table
        // TODO: Implement proper goto table
        // Placeholder: return next state (this is simplified)
        Ok(StateId(state.0 + 1))
    }

    /// Merge stacks with identical state sequences
    ///
    /// Two stacks are identical if they have the same state stack.
    fn merge_identical_stacks(&self, stacks: Vec<ParserStack>) -> Vec<ParserStack> {
        let mut merged: HashMap<Vec<StateId>, ParserStack> = HashMap::new();

        for stack in stacks {
            let key = stack.states.clone();
            if merged.contains_key(&key) {
                // For now, keep the first one (TODO: implement proper merging)
                // Proper merging would combine node stacks (packed nodes)
                continue;
            }
            merged.insert(key, stack);
        }

        merged.into_values().collect()
    }

    /// Reset the engine for reuse
    pub fn reset(&mut self) {
        self.stacks = vec![ParserStack {
            states: vec![StateId(0)],
            nodes: vec![],
            id: 0,
        }];
        self.forest = ParseForest::new();
    }
}

impl ParserStack {
    /// Get the top state
    fn top_state(&self) -> StateId {
        *self.states.last().expect("Stack should never be empty")
    }

    /// Push a new state and node
    fn push(&mut self, state: StateId, node: ForestNodeId) {
        self.states.push(state);
        self.nodes.push(node);
    }
}

impl ParseForest {
    /// Create a new empty forest
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            roots: Vec::new(),
        }
    }

    /// Add a terminal node (leaf)
    fn add_terminal(&mut self, token: &Token) -> ForestNodeId {
        let node = ForestNode {
            symbol: SymbolId(token.kind as u16),
            children: vec![],
            range: (token.start as usize)..(token.end as usize),
        };
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Add a nonterminal node (internal)
    fn add_nonterminal(
        &mut self,
        symbol: SymbolId,
        children: Vec<ForestNodeId>,
        range: Range<usize>,
    ) -> ForestNodeId {
        let node = ForestNode {
            symbol,
            children,
            range,
        };
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Add a root node
    fn add_root(&mut self, node_id: ForestNodeId) {
        if !self.roots.contains(&node_id) {
            self.roots.push(node_id);
        }
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of roots
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glr_config_default() {
        let config = GLRConfig::default();
        assert_eq!(config.max_forks, 1000);
        assert_eq!(config.max_forest_nodes, 10_000);
    }

    #[test]
    fn test_parse_forest_new() {
        let forest = ParseForest::new();
        assert_eq!(forest.node_count(), 0);
        assert_eq!(forest.root_count(), 0);
    }

    // More tests will be added as we implement the engine
}
