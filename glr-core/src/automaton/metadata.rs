use crate::{Action, SymbolMetadata};
use adze_ir::*;
use std::collections::BTreeMap;

pub(super) fn build_symbol_metadata(grammar: &Grammar) -> Vec<SymbolMetadata> {
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

pub(super) fn build_external_scanner_states(
    grammar: &Grammar,
    state_count: usize,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    action_table: &[Vec<Vec<Action>>],
) -> Vec<Vec<bool>> {
    let mut external_scanner_states = vec![vec![false; grammar.externals.len()]; state_count];

    for state_idx in 0..state_count {
        for (external_idx, external) in grammar.externals.iter().enumerate() {
            if let Some(&symbol_idx) = symbol_to_index.get(&external.symbol_id)
                && action_table[state_idx][symbol_idx]
                    .iter()
                    .any(|a| matches!(a, Action::Shift(_)))
            {
                external_scanner_states[state_idx][external_idx] = true;
            }
        }
    }

    external_scanner_states
}
