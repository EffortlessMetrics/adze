use crate::{Grammar, StateId, SymbolId, ProductionId, Action, ParseTable, FirstFollowSets};
use rustc_hash::FxHashMap;
use fixedbitset::FixedBitSet;

/// Trait for resolving conflicts at runtime
pub trait RuntimeConflictResolver {
    /// Resolve a conflict between multiple actions
    /// Returns Some(action) to take that action, or None to use default fork behavior
    fn resolve(&self, state: StateId, lookahead: SymbolId, actions: &[Action]) -> Option<Action>;
}

pub struct VecWrapperResolver {
    // Cache: state -> optional vec wrapper empty production
    wrapper_states: FxHashMap<StateId, Option<ProductionId>>,
    statement_starters: FixedBitSet,
}

impl VecWrapperResolver {
    pub fn new(grammar: &Grammar, first_follow: &FirstFollowSets) -> Self {
        // Get the maximum symbol ID to size our bitset properly
        let max_symbol_id = grammar.rules.keys()
            .chain(grammar.tokens.keys())
            .map(|id| id.0)
            .max()
            .unwrap_or(0) as usize + 1;
        
        let mut statement_starters = FixedBitSet::with_capacity(max_symbol_id);

        // Find FIRST(Statement) - you already compute this
        if let Some(stmt_id) = grammar.find_symbol_by_name("Statement") {
            if let Some(first_set) = first_follow.first(stmt_id) {
                statement_starters.union_with(first_set);
            }
        }

        // Also check for other common statement starters
        for name in &["ExpressionStatement", "AssignmentStatement", "Primary", "Number"] {
            if let Some(id) = grammar.find_symbol_by_name(name) {
                if let Some(first_set) = first_follow.first(id) {
                    statement_starters.union_with(first_set);
                }
            }
        }

        Self {
            wrapper_states: FxHashMap::default(),
            statement_starters,
        }
    }

    pub fn get_vec_wrapper_action(&mut self,
                                  state: StateId,
                                  table: &ParseTable,
                                  grammar: &Grammar) -> Option<ProductionId> {
        // Check cache first
        if let Some(&cached) = self.wrapper_states.get(&state) {
            return cached;
        }

        // Find vec wrapper empty production in this state
        let mut result = None;
        
        // Look through the action table for reduce actions in this state
        if let Some(state_actions) = table.action_table.get(state.0 as usize) {
            for (_symbol_idx, action) in state_actions.iter().enumerate() {
                match action {
                    Action::Reduce(rule_id) => {
                        // Find the corresponding rule in the grammar
                        if let Some(rule) = grammar.all_rules().find(|r| r.production_id.0 == rule_id.0) {
                            // Check if this is a vec wrapper empty rule
                            if let Some(rule_name) = grammar.rule_names.get(&rule.lhs) {
                                if rule_name.ends_with("_vec_contents") && rule.rhs.is_empty() {
                                    result = Some(ProductionId(rule_id.0));
                                    break;
                                }
                            }
                        }
                    },
                    Action::Fork(actions) => {
                        // Check fork actions too
                        for fork_action in actions {
                            if let Action::Reduce(rule_id) = fork_action {
                                if let Some(rule) = grammar.all_rules().find(|r| r.production_id.0 == rule_id.0) {
                                    if let Some(rule_name) = grammar.rule_names.get(&rule.lhs) {
                                        if rule_name.ends_with("_vec_contents") && rule.rhs.is_empty() {
                                            result = Some(ProductionId(rule_id.0));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        self.wrapper_states.insert(state, result);
        result
    }

    pub fn should_reduce_empty(&self, token: SymbolId) -> bool {
        // Reduce empty if NOT a statement starter
        !self.statement_starters.contains(token.0 as usize)
    }
}

impl RuntimeConflictResolver for VecWrapperResolver {
    fn resolve(&self, state: StateId, lookahead: SymbolId, actions: &[Action]) -> Option<Action> {
        debug_assert!(actions.len() == 2, "VecWrapperResolver expects exactly 2 conflicting actions");
        
        // Look for a reduce action that's a vec_contents empty production
        let mut reduce_action = None;
        let mut shift_action = None;
        
        for action in actions {
            match action {
                Action::Reduce(_) => reduce_action = Some(action.clone()),
                Action::Shift(_) => shift_action = Some(action.clone()),
                _ => {}
            }
        }
        
        // If we have both shift and reduce actions
        if let (Some(reduce), Some(shift)) = (reduce_action, shift_action) {
            // Heuristic: if the lookahead is in FIRST(Statement), choose Shift
            // Otherwise, choose Reduce (empty vec)
            if self.statement_starters.contains(lookahead.0 as usize) {
                Some(shift)
            } else {
                Some(reduce)
            }
        } else {
            None
        }
    }
}