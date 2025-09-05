// Minimal GLR parser without error recovery.
// This implementation supports the new GLR `ActionCell` architecture where
// `action_table[state][symbol]` is `Vec<Action>`.

use rust_sitter_glr_core::{parse_forest::ErrorMeta, parse_forest::ForestAlternative};
use rust_sitter_glr_core::{Action, ParseError, ParseForest, ParseTable};
use rust_sitter_ir::{RuleId, StateId, SymbolId};
use std::collections::HashMap;

/// A simple GLR parser that produces a parse forest. This version does not
/// perform any error recovery or precedence handling – it simply explores all
/// possible parse stacks using the provided parse table.
pub struct GLRParser {
    table: ParseTable,
}

impl GLRParser {
    /// Create a new parser from a parse table
    pub fn new(table: ParseTable) -> Self {
        Self { table }
    }

    /// Parse a sequence of input tokens and produce a parse forest. The input
    /// should be a sequence of `SymbolId` tokens (terminals). An EOF symbol will
    /// be appended automatically.
    pub fn parse(&mut self, tokens: &[SymbolId]) -> Result<ParseForest, ParseError> {
        use rust_sitter_glr_core::ForestNode;

        // Prepare parse forest
        let mut forest = ParseForest {
            roots: vec![],
            nodes: HashMap::new(),
            grammar: self.table.grammar.clone(),
            source: String::new(),
            next_node_id: 0,
        };

        // Prepare input with EOF appended
        let mut input: Vec<SymbolId> = tokens.to_vec();
        input.push(self.table.eof_symbol);

        // Active parse stacks
        let mut stacks = vec![ParseStack::new(self.table.initial_state)];

        for (position, symbol) in input.iter().enumerate() {
            // First, perform all possible reductions until saturation
            let mut reduced = Vec::new();
            for stack in stacks.drain(..) {
                reduced.extend(self.reduce_all(stack, *symbol, &mut forest));
            }

            // Then process shifts/accepts
            let mut new_stacks = Vec::new();
            let mut accepted = false;

            for stack in reduced {
                let actions = self.get_actions(stack.current_state(), *symbol);
                if actions.is_empty() {
                    continue;
                }

                for action in actions {
                    match action {
                        Action::Shift(next_state) => {
                            // Create leaf node for the token
                            let node_id = self.create_leaf(*symbol, position, &mut forest);
                            let mut new_stack = stack.clone();
                            new_stack.states.push(next_state);
                            new_stack.nodes.push(node_id);
                            new_stacks.push(new_stack);
                        }
                        Action::Accept => {
                            if let Some(&root_id) = stack.nodes.last() {
                                if let Some(node) = forest.nodes.get(&root_id).cloned() {
                                    forest.roots.push(node);
                                    accepted = true;
                                }
                            }
                        }
                        Action::Fork(fork_actions) => {
                            for fork_action in fork_actions {
                                match fork_action {
                                    Action::Shift(next_state) => {
                                        let node_id =
                                            self.create_leaf(*symbol, position, &mut forest);
                                        let mut new_stack = stack.clone();
                                        new_stack.states.push(next_state);
                                        new_stack.nodes.push(node_id);
                                        new_stacks.push(new_stack);
                                    }
                                    Action::Accept => {
                                        if let Some(&root_id) = stack.nodes.last() {
                                            if let Some(node) = forest.nodes.get(&root_id).cloned() {
                                                forest.roots.push(node);
                                                accepted = true;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            if new_stacks.is_empty() && !accepted {
                return Err(ParseError::Failed(format!(
                    "Parse error at token position {}",
                    position
                )));
            }

            stacks = new_stacks;
        }

        if forest.roots.is_empty() {
            Err(ParseError::Incomplete)
        } else {
            Ok(forest)
        }
    }

    /// Get actions for a state and symbol (new GLR structure)
    pub fn get_actions(&self, state: StateId, symbol: SymbolId) -> Vec<Action> {
        let state_idx = state.0 as usize;
        let symbol_idx = self
            .table
            .symbol_to_index
            .get(&symbol)
            .copied()
            .unwrap_or(0);

        if state_idx < self.table.action_table.len()
            && symbol_idx < self.table.action_table[0].len()
        {
            self.table.action_table[state_idx][symbol_idx].clone()
        } else {
            vec![]
        }
    }

    /// Perform all reductions for a stack given a lookahead symbol.
    fn reduce_all(
        &self,
        stack: ParseStack,
        lookahead: SymbolId,
        forest: &mut ParseForest,
    ) -> Vec<ParseStack> {
        let actions = self.get_actions(stack.current_state(), lookahead);
        let mut results = Vec::new();
        let mut reduced_any = false;

        for action in actions {
            match action {
                Action::Reduce(rule_id) => {
                    reduced_any = true;
                    if let Some(reduced) = self.apply_reduction(stack.clone(), rule_id, forest) {
                        results.extend(self.reduce_all(reduced, lookahead, forest));
                    }
                }
                Action::Fork(fork_actions) => {
                    for a in fork_actions {
                        if let Action::Reduce(rule_id) = a {
                            reduced_any = true;
                            if let Some(reduced) =
                                self.apply_reduction(stack.clone(), rule_id, forest)
                            {
                                results.extend(self.reduce_all(reduced, lookahead, forest));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !reduced_any {
            results.push(stack);
        }
        results
    }

    /// Apply a single reduction to a stack
    fn apply_reduction(
        &self,
        mut stack: ParseStack,
        rule_id: RuleId,
        forest: &mut ParseForest,
    ) -> Option<ParseStack> {
        let rule = self.table.rules.get(rule_id.0 as usize)?;
        let rhs_len = rule.rhs_len as usize;
        let children = stack.pop(rhs_len);

        // Determine span from children
        let (start, end) = if let (Some(first), Some(last)) = (children.first(), children.last()) {
            let s = forest.nodes.get(first)?.span.0;
            let e = forest.nodes.get(last)?.span.1;
            (s, e)
        } else {
            (0, 0)
        };

        // Goto state after reduction
        let base_state = stack.current_state();
        let goto_state = self.get_goto(base_state, rule.lhs)?;

        // Create nonterminal node
        let node_id = forest.next_node_id;
        forest.next_node_id += 1;
        forest.nodes.insert(
            node_id,
            ForestNode {
                id: node_id,
                symbol: rule.lhs,
                span: (start, end),
                alternatives: vec![ForestAlternative { children: children.clone() }],
                error_meta: ErrorMeta::default(),
            },
        );

        stack.states.push(goto_state);
        stack.nodes.push(node_id);
        Some(stack)
    }

    /// Get the goto state for a nonterminal
    fn get_goto(&self, state: StateId, symbol: SymbolId) -> Option<StateId> {
        let row = self.table.goto_table.get(state.0 as usize)?;
        let col = match self.table.goto_indexing {
            rust_sitter_glr_core::GotoIndexing::NonterminalMap => {
                *self.table.nonterminal_to_index.get(&symbol)?
            }
            rust_sitter_glr_core::GotoIndexing::DirectSymbolId => symbol.0 as usize,
        };
        row.get(col).copied()
    }

    /// Create a leaf node for a token
    fn create_leaf(
        &self,
        symbol: SymbolId,
        position: usize,
        forest: &mut ParseForest,
    ) -> usize {
        let node_id = forest.next_node_id;
        forest.next_node_id += 1;
        forest.nodes.insert(
            node_id,
            rust_sitter_glr_core::ForestNode {
                id: node_id,
                symbol,
                span: (position, position + 1),
                alternatives: vec![ForestAlternative { children: vec![] }],
                error_meta: ErrorMeta::default(),
            },
        );
        node_id
    }
}

/// GLR parse stack used by this parser.
#[derive(Clone)]
pub struct ParseStack {
    states: Vec<StateId>,
    nodes: Vec<usize>,
}

impl ParseStack {
    pub fn new(initial_state: StateId) -> Self {
        Self {
            states: vec![initial_state],
            nodes: vec![],
        }
    }

    pub fn current_state(&self) -> StateId {
        *self.states.last().unwrap_or(&StateId(0))
    }

    fn pop(&mut self, n: usize) -> Vec<usize> {
        self.states.truncate(self.states.len().saturating_sub(n));
        self.nodes.split_off(self.nodes.len().saturating_sub(n))
    }
}
