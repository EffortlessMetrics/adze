use crate::{
    Action, ActionCell, FirstFollowSets, Grammar, ItemSetCollection, ParseRule, PrecDecision,
    RuleId, StateId, SymbolId, add_action_with_conflict, build_prec_tables, decide_reduce_reduce,
    decide_with_precedence, map_follow_symbol,
};
use std::collections::BTreeMap;

pub(crate) fn build_tables(
    augmented_grammar: &Grammar,
    first_follow: &FirstFollowSets,
    collection: &ItemSetCollection,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    rules: &[ParseRule],
    production_to_rule_id: &BTreeMap<u16, u16>,
    augmented_start: SymbolId,
    eof_symbol: SymbolId,
    internal_token_count: usize,
    external_token_count: usize,
) -> (Vec<Vec<ActionCell>>, Vec<Vec<StateId>>) {
    let state_count = collection.sets.len();
    let symbol_count = symbol_to_index.len();
    let mut action_table = vec![vec![Vec::new(); symbol_count]; state_count];
    let mut goto_table = vec![vec![StateId(0); symbol_count]; state_count];
    let mut conflicts_by_state: BTreeMap<(usize, usize), Vec<Action>> = BTreeMap::new();

    #[cfg(any(feature = "glr_trace", feature = "debug_glr"))]
    trace_collection(collection, augmented_grammar);
    add_shift_actions(
        &mut action_table,
        &mut conflicts_by_state,
        collection,
        augmented_grammar,
        symbol_to_index,
        eof_symbol,
    );
    add_extra_self_loops(&mut action_table, augmented_grammar, symbol_to_index);
    add_reduce_actions(
        &mut action_table,
        &mut conflicts_by_state,
        augmented_grammar,
        first_follow,
        collection,
        symbol_to_index,
        rules,
        production_to_rule_id,
        augmented_start,
        eof_symbol,
    );
    resolve_conflicts(
        &mut action_table,
        conflicts_by_state,
        augmented_grammar,
        symbol_to_index,
        internal_token_count,
        external_token_count,
    );
    fill_goto_table(&mut goto_table, collection, symbol_to_index, eof_symbol);

    (action_table, goto_table)
}

#[cfg(any(feature = "glr_trace", feature = "debug_glr"))]
fn trace_collection(collection: &ItemSetCollection, augmented_grammar: &Grammar) {
    crate::debug_trace!(
        "DEBUG: Collection goto table has {} entries",
        collection.goto_table.len()
    );
    crate::debug_trace!(
        "DEBUG: Augmented grammar has {} tokens",
        augmented_grammar.tokens.len()
    );
    crate::debug_trace!("=== Symbol Classification Debug ===");
    crate::debug_trace!(
        "Tokens in augmented_grammar: {:?}",
        augmented_grammar
            .tokens
            .keys()
            .map(|k| k.0)
            .collect::<Vec<_>>()
    );
    crate::debug_trace!(
        "Externals in augmented_grammar: {:?}",
        augmented_grammar
            .externals
            .iter()
            .map(|e| e.symbol_id.0)
            .collect::<Vec<_>>()
    );
    crate::debug_trace!(
        "Collection goto_table size: {}",
        collection.goto_table.len()
    );

    let state0_gotos: Vec<_> = collection
        .goto_table
        .iter()
        .filter(|((from, _), _)| from.0 == 0)
        .collect();
    crate::debug_trace!("State 0 has {} goto entries", state0_gotos.len());
    for ((_, symbol), to_state) in &state0_gotos {
        crate::debug_trace!("  Symbol {} -> State {}", symbol.0, to_state.0);
    }
}

fn add_shift_actions(
    action_table: &mut Vec<Vec<ActionCell>>,
    conflicts_by_state: &mut BTreeMap<(usize, usize), Vec<Action>>,
    collection: &ItemSetCollection,
    _augmented_grammar: &Grammar,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    eof_symbol: SymbolId,
) {
    for ((from_state, symbol), to_state) in &collection.goto_table {
        let is_terminal = collection
            .symbol_is_terminal
            .get(symbol)
            .copied()
            .unwrap_or(*symbol == eof_symbol);

        #[cfg(any(feature = "glr_trace", feature = "debug_glr"))]
        if from_state.0 == 0 {
            crate::debug_trace!(
                "State 0 goto entry: symbol {} -> state {}, is_terminal={} (in tokens={}, in externals={}, is EOF={})",
                symbol.0,
                to_state.0,
                is_terminal,
                _augmented_grammar.tokens.contains_key(symbol),
                _augmented_grammar
                    .externals
                    .iter()
                    .any(|e| e.symbol_id == *symbol),
                symbol.0 == 0
            );
        }

        if !is_terminal {
            continue;
        }

        if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
            let state_idx = from_state.0 as usize;
            if state_idx < action_table.len() && symbol_idx < action_table[state_idx].len() {
                #[cfg(any(feature = "glr_trace", feature = "debug_glr"))]
                if state_idx == 0 {
                    crate::debug_trace!(
                        "DEBUG: Adding shift action to state 0: symbol {} (idx={}) -> state {}",
                        symbol.0,
                        symbol_idx,
                        to_state.0
                    );
                }
                add_action_with_conflict(
                    action_table,
                    conflicts_by_state,
                    state_idx,
                    symbol_idx,
                    Action::Shift(*to_state),
                );
            } else {
                #[cfg(any(feature = "glr_trace", feature = "debug_glr"))]
                if state_idx == 0 {
                    crate::debug_trace!(
                        "DEBUG: SKIPPING shift for state 0: bounds check failed - state_idx={}, symbol_idx={}, action_table.len={}, inner_len={}",
                        state_idx,
                        symbol_idx,
                        action_table.len(),
                        if state_idx < action_table.len() {
                            action_table[state_idx].len()
                        } else {
                            0
                        }
                    );
                }
            }
        } else {
            #[cfg(any(feature = "glr_trace", feature = "debug_glr"))]
            if from_state.0 == 0 {
                crate::debug_trace!(
                    "DEBUG: Terminal {} not in symbol_to_index for state 0",
                    symbol.0
                );
            }
        }
    }
}

fn add_extra_self_loops(
    action_table: &mut [Vec<ActionCell>],
    augmented_grammar: &Grammar,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
) {
    for (state_idx, row) in action_table.iter_mut().enumerate() {
        for extra_symbol_id in &augmented_grammar.extras {
            if let Some(&symbol_idx) = symbol_to_index.get(extra_symbol_id)
                && row[symbol_idx].is_empty()
            {
                row[symbol_idx].push(Action::Shift(StateId(state_idx as u16)));
            }
        }
    }
}

fn add_reduce_actions(
    action_table: &mut Vec<Vec<ActionCell>>,
    conflicts_by_state: &mut BTreeMap<(usize, usize), Vec<Action>>,
    augmented_grammar: &Grammar,
    first_follow: &FirstFollowSets,
    collection: &ItemSetCollection,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    rules: &[ParseRule],
    production_to_rule_id: &BTreeMap<u16, u16>,
    augmented_start: SymbolId,
    eof_symbol: SymbolId,
) {
    for item_set in &collection.sets {
        let state_idx = item_set.id.0 as usize;

        for item in &item_set.items {
            if !item.is_reduce_item(augmented_grammar) {
                continue;
            }

            if let Some(rule) = augmented_grammar
                .all_rules()
                .find(|r| r.production_id.0 == item.rule_id.0)
                && rule.lhs == augmented_start
            {
                if item.lookahead == eof_symbol
                    && let Some(&eof_idx) = symbol_to_index.get(&eof_symbol)
                {
                    add_action_with_conflict(
                        action_table,
                        conflicts_by_state,
                        state_idx,
                        eof_idx,
                        Action::Accept,
                    );
                }
                continue;
            }

            if let Some(&rule_id) = production_to_rule_id.get(&item.rule_id.0) {
                let rule = &rules[rule_id as usize];
                let lookaheads_to_check =
                    reduce_lookaheads(rule, item.lookahead, first_follow, eof_symbol);
                for lookahead in lookaheads_to_check {
                    if let Some(&lookahead_idx) = symbol_to_index.get(&lookahead) {
                        add_action_with_conflict(
                            action_table,
                            conflicts_by_state,
                            state_idx,
                            lookahead_idx,
                            Action::Reduce(RuleId(rule_id)),
                        );
                    }
                }
            }
        }
    }
}

fn reduce_lookaheads(
    rule: &ParseRule,
    item_lookahead: SymbolId,
    first_follow: &FirstFollowSets,
    eof_symbol: SymbolId,
) -> Vec<SymbolId> {
    if rule.rhs_len != 0 {
        return vec![item_lookahead];
    }

    first_follow
        .follow(rule.lhs)
        .map(|follow_set| {
            follow_set
                .ones()
                .map(|idx| map_follow_symbol(SymbolId(idx as u16), eof_symbol))
                .collect()
        })
        .unwrap_or_else(|| vec![item_lookahead])
}

fn resolve_conflicts(
    action_table: &mut [Vec<ActionCell>],
    conflicts_by_state: BTreeMap<(usize, usize), Vec<Action>>,
    augmented_grammar: &Grammar,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    internal_token_count: usize,
    external_token_count: usize,
) {
    let production_count = augmented_grammar.all_rules().count() as u32;
    let token_count = (internal_token_count + 1) as u32;
    let prec_tables = build_prec_tables(
        augmented_grammar,
        symbol_to_index,
        token_count,
        production_count,
    );
    let first_nonterminal_idx = internal_token_count + external_token_count + 1;

    for ((state_idx, symbol_idx), _actions) in conflicts_by_state {
        debug_assert!(state_idx < action_table.len(), "state_idx out of bounds");
        debug_assert!(
            symbol_idx < action_table[0].len(),
            "symbol_idx out of bounds"
        );

        if symbol_idx >= first_nonterminal_idx {
            continue;
        }

        let cell = &mut action_table[state_idx][symbol_idx];
        if cell.is_empty() {
            continue;
        }
        if cell.iter().any(|a| matches!(a, Action::Accept)) {
            *cell = vec![Action::Accept];
            continue;
        }

        let first_shift = cell.iter().find_map(|a| {
            if let Action::Shift(s) = a {
                Some(*s)
            } else {
                None
            }
        });
        let mut reduces: Vec<u16> = cell
            .iter()
            .filter_map(|a| {
                if let Action::Reduce(pid) = a {
                    Some(pid.0)
                } else {
                    None
                }
            })
            .collect();

        if reduces.len() > 1 {
            let winner = reduces[1..].iter().fold(reduces[0], |acc, &r| {
                decide_reduce_reduce(acc, r, &prec_tables)
            });
            reduces.clear();
            reduces.push(winner);
            cell.retain(|a| {
                matches!(a, Action::Shift(_)) || matches!(a, Action::Reduce(pid) if pid.0 == winner)
            });
        }

        if let (Some(s), Some(r)) = (first_shift, reduces.first().copied()) {
            match decide_with_precedence(symbol_idx, r, &prec_tables) {
                PrecDecision::PreferShift => *cell = vec![Action::Shift(s)],
                PrecDecision::PreferReduce => *cell = vec![Action::Reduce(RuleId(r))],
                PrecDecision::Error | PrecDecision::NoInfo => {}
            }
        }
    }
}

fn fill_goto_table(
    goto_table: &mut [Vec<StateId>],
    collection: &ItemSetCollection,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    eof_symbol: SymbolId,
) {
    for ((from_state, symbol), _to_state) in &collection.goto_table {
        let is_terminal = collection
            .symbol_is_terminal
            .get(symbol)
            .copied()
            .unwrap_or(*symbol == eof_symbol);

        if !is_terminal && let Some(&symbol_idx) = symbol_to_index.get(symbol) {
            let state_idx = from_state.0 as usize;
            if state_idx < goto_table.len() && symbol_idx < goto_table[state_idx].len() {
                // Deliberately validated here; actual assignment happens below for compatibility.
            }
        }
    }

    for ((from_state, symbol), to_state) in &collection.goto_table {
        let from_idx = from_state.0 as usize;
        if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
            goto_table[from_idx][symbol_idx] = *to_state;
        }
    }
}
