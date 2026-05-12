use crate::{Action, Grammar, SymbolId, SymbolMetadata, TokenPattern};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn symbol_metadata(grammar: &Grammar) -> Vec<SymbolMetadata> {
    let mut symbol_metadata = Vec::new();

    for (symbol_id, token) in &grammar.tokens {
        symbol_metadata.push(SymbolMetadata {
            name: token.name.clone(),
            is_visible: !token.name.starts_with('_'),
            is_named: !matches!(&token.pattern, TokenPattern::String(_)),
            is_supertype: false,
            is_terminal: true,
            is_extra: grammar.extras.contains(symbol_id),
            is_fragile: false,
            symbol_id: *symbol_id,
        });
    }

    for symbol_id in grammar.rules.keys() {
        let is_supertype = grammar.supertypes.contains(symbol_id);
        symbol_metadata.push(SymbolMetadata {
            name: format!("rule_{}", symbol_id.0),
            is_visible: true,
            is_named: true,
            is_supertype,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: *symbol_id,
        });
    }

    for external in &grammar.externals {
        symbol_metadata.push(SymbolMetadata {
            name: external.name.clone(),
            is_visible: !external.name.starts_with('_'),
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: external.symbol_id,
        });
    }

    symbol_metadata
}

pub(crate) fn external_scanner_states(
    action_table: &[Vec<Vec<Action>>],
    grammar: &Grammar,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
) -> Vec<Vec<bool>> {
    let mut states = vec![vec![false; grammar.externals.len()]; action_table.len()];

    for state_idx in 0..action_table.len() {
        for (external_idx, external) in grammar.externals.iter().enumerate() {
            if let Some(&symbol_idx) = symbol_to_index.get(&external.symbol_id)
                && action_table[state_idx][symbol_idx]
                    .iter()
                    .any(|a| matches!(a, Action::Shift(_)))
            {
                states[state_idx][external_idx] = true;
            }
        }
    }

    states
}

pub(crate) fn nonterminal_to_index(
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    nonterminals: &BTreeSet<SymbolId>,
) -> BTreeMap<SymbolId, usize> {
    symbol_to_index
        .iter()
        .filter_map(|(&symbol_id, &idx)| {
            nonterminals
                .contains(&symbol_id)
                .then_some((symbol_id, idx))
        })
        .collect()
}

pub(crate) fn index_to_symbol(symbol_to_index: &BTreeMap<SymbolId, usize>) -> Vec<SymbolId> {
    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in symbol_to_index {
        index_to_symbol[idx] = *sym;
    }
    index_to_symbol
}
