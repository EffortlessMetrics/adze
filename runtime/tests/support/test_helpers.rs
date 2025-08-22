/// Common test helper utilities for rust-sitter tests
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

/// Creates a minimal ParseTable for testing purposes.
/// This helper provides all required fields with default values that satisfy
/// the struct shape requirements. Tests can override specific fields as needed.
pub fn make_test_table(grammar: Grammar) -> ParseTable {
    ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Normalizes JSON strings for comparison by removing whitespace variations.
/// Useful for golden test assertions where formatting may differ.
pub fn normalize_json(s: &str) -> String {
    s.split_whitespace()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_json() {
        let input = r#"{
            "key":   "value",
            "number": 42
        }"#;
        let expected = r#"{ "key": "value", "number": 42 }"#;
        assert_eq!(normalize_json(input), normalize_json(expected));
    }
}
