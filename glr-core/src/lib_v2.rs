// Improved parse table building with proper conflict handling
use crate::{
    Action, FirstFollowSets, GLRError, ItemSetCollection, ParseTable, RuleId, StateId, SymbolId,
    SymbolMetadata,
};
use adze_ir::{Grammar, Symbol, TokenPattern};
use std::collections::HashMap;

/// Build LR(1) automaton with proper conflict handling for GLR parsing
pub fn build_lr1_automaton_v2(
    grammar: &Grammar,
    first_follow: &FirstFollowSets,
) -> Result<ParseTable, GLRError> {
    // Build LR(1) item sets
    let collection = ItemSetCollection::build(grammar, first_follow)?;

    // Create symbol to index mapping
    let mut symbol_to_index = HashMap::new();

    // Add terminal symbols
    for (symbol_id, _) in &grammar.tokens {
        symbol_to_index.insert(*symbol_id, symbol_to_index.len());
    }

    // Add non-terminal symbols
    for (symbol_id, _) in &grammar.rule_names {
        symbol_to_index.insert(*symbol_id, symbol_to_index.len());
    }

    // Add external symbols
    for external in &grammar.externals {
        symbol_to_index.insert(external.symbol_id, symbol_to_index.len());
    }

    // Add EOF symbol
    symbol_to_index.insert(SymbolId(0), symbol_to_index.len());

    // Create parse table
    let state_count = collection.sets.len();
    let indexed_symbol_count = symbol_to_index.len();

    let mut action_table = vec![vec![Action::Error; indexed_symbol_count]; state_count];
    let mut goto_table = vec![vec![StateId(0); indexed_symbol_count]; state_count];

    // Track conflicts as we build the table
    let mut conflicts_by_state: HashMap<(usize, usize), Vec<Action>> = HashMap::new();

    // Fill action table with conflict detection
    for item_set in &collection.sets {
        let state_idx = item_set.id.0 as usize;

        for item in &item_set.items {
            if item.is_reduce_item(grammar) {
                // Add reduce action
                if let Some(&lookahead_idx) = symbol_to_index.get(&item.lookahead) {
                    let new_action = Action::Reduce(item.rule_id);
                    add_action_with_conflict(
                        &mut action_table,
                        &mut conflicts_by_state,
                        state_idx,
                        lookahead_idx,
                        new_action,
                    );
                }
            } else if let Some(next_symbol) = item.next_symbol(grammar) {
                let symbol_id = match &next_symbol {
                    Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id,
                };

                if let Some(&symbol_idx) = symbol_to_index.get(symbol_id) {
                    if let Symbol::Terminal(_) = next_symbol {
                        // Add shift action
                        if let Some(&goto_state) =
                            collection.goto_table.get(&(item_set.id, *symbol_id))
                        {
                            let new_action = Action::Shift(goto_state);
                            add_action_with_conflict(
                                &mut action_table,
                                &mut conflicts_by_state,
                                state_idx,
                                symbol_idx,
                                new_action,
                            );
                        }
                    }
                }
            }
        }
    }

    // Convert conflicts to Fork actions
    for ((state_idx, symbol_idx), actions) in conflicts_by_state {
        if actions.len() > 1 {
            action_table[state_idx][symbol_idx] = Action::Fork(actions);
        } else if let Some(action) = actions.into_iter().next() {
            action_table[state_idx][symbol_idx] = action;
        }
    }

    // Fill goto table
    for ((from_state, symbol), to_state) in &collection.goto_table {
        let from_idx = from_state.0 as usize;
        if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
            goto_table[from_idx][symbol_idx] = *to_state;
        }
    }

    // Build symbol metadata
    let mut symbol_metadata = Vec::new();

    // Add terminal symbols
    for (_, token) in &grammar.tokens {
        symbol_metadata.push(SymbolMetadata {
            name: token.name.clone(),
            is_visible: !token.name.starts_with('_'),
            is_named: !matches!(&token.pattern, TokenPattern::String(_)),
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0), // TODO: get proper symbol_id
        });
    }

    // Add non-terminal symbols
    for (symbol_id, name) in &grammar.rule_names {
        let is_supertype_val = grammar.supertypes.contains(symbol_id);
        symbol_metadata.push(SymbolMetadata {
            name: name.clone(),
            is_visible: true,
            is_named: true,
            is_supertype: is_supertype_val,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: *symbol_id,
        });
    }

    // Add external symbols
    for external in &grammar.externals {
        symbol_metadata.push(SymbolMetadata {
            name: external.name.clone(),
            is_visible: true, // TODO: get from external
            is_named: true,   // TODO: get from external
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: external.symbol_id,
        });
    }

    // Add EOF metadata
    symbol_metadata.push(SymbolMetadata {
        name: "_eof".to_string(),
        is_visible: false,
        is_named: false,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(0), // TODO: get proper EOF symbol_id
    });

    Ok(ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count,
        symbol_count: indexed_symbol_count,
        symbol_to_index,
    })
}

/// Add an action to the parse table, tracking conflicts
fn add_action_with_conflict(
    action_table: &mut Vec<Vec<Action>>,
    conflicts_by_state: &mut HashMap<(usize, usize), Vec<Action>>,
    state_idx: usize,
    symbol_idx: usize,
    new_action: Action,
) {
    let current_action = &action_table[state_idx][symbol_idx];

    match current_action {
        Action::Error => {
            // No conflict, just set the action
            action_table[state_idx][symbol_idx] = new_action.clone();
        }
        _ => {
            // Conflict detected! Track it
            let entry = conflicts_by_state
                .entry((state_idx, symbol_idx))
                .or_insert_with(Vec::new);

            // Add the current action if not already tracked
            if entry.is_empty() {
                if let Action::Fork(actions) = current_action {
                    entry.extend(actions.clone());
                } else {
                    entry.push(current_action.clone());
                }
            }

            // Add the new action if not duplicate
            if !entry.iter().any(|a| action_eq(a, &new_action)) {
                entry.push(new_action);
            }
        }
    }
}

/// Check if two actions are equivalent
fn action_eq(a: &Action, b: &Action) -> bool {
    match (a, b) {
        (Action::Shift(s1), Action::Shift(s2)) => s1 == s2,
        (Action::Reduce(r1), Action::Reduce(r2)) => r1 == r2,
        (Action::Accept, Action::Accept) => true,
        (Action::Error, Action::Error) => true,
        (Action::Fork(a1), Action::Fork(a2)) => {
            a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| action_eq(x, y))
        }
        _ => false,
    }
}
