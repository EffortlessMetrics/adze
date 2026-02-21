// Test helpers for tablegen tests
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

#[cfg(test)]
pub(crate) mod test {
    use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
    use adze_ir::{Grammar, StateId, SymbolId};
    use std::collections::BTreeMap;

    /// Sentinel used throughout the tests for "no goto".
    pub(crate) const INVALID: StateId = StateId(u16::MAX);

    /// Build a minimal but fully-formed ParseTable suitable for unit tests.
    ///
    /// Conventions expected by the project:
    /// - Symbol layout: ERROR(0), terminals `[1..]`, EOF (= token_count + external_token_count), then non-terminals.
    /// - `actions` is indexed by `[state][symbol_index]` and `gotos` by `[state][symbol_index]`.
    pub fn make_minimal_table(
        mut actions: Vec<Vec<Vec<Action>>>,
        mut gotos: Vec<Vec<StateId>>,
        rules: Vec<ParseRule>,
        start_symbol: SymbolId,
        eof_symbol: SymbolId,
        external_token_count: usize,
    ) -> ParseTable {
        // Dimensions
        let state_count = actions.len().max(1);
        let symbol_cols_from_actions = actions.first().map(|r| r.len()).unwrap_or(0);
        let symbol_cols_from_gotos = gotos.first().map(|r| r.len()).unwrap_or(0);
        // Cover the columns referenced by start_symbol and eof_symbol too.
        let min_needed = (start_symbol.0 as usize + 1).max(eof_symbol.0 as usize + 1);
        let symbol_count = symbol_cols_from_actions
            .max(symbol_cols_from_gotos)
            .max(min_needed)
            .max(1);

        // Normalize shapes (pad rows/cols if needed)
        if actions.is_empty() {
            actions = vec![vec![vec![]; symbol_count]];
        } else {
            for row in &mut actions {
                if row.len() < symbol_count {
                    row.resize_with(symbol_count, Vec::new);
                }
            }
        }
        if gotos.len() < state_count {
            gotos.resize_with(state_count, || vec![INVALID; symbol_count]);
        }
        for row in &mut gotos {
            if row.len() < symbol_count {
                row.resize(symbol_count, INVALID);
            }
        }

        // Build symbol maps
        let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
        for i in 0..symbol_count {
            symbol_to_index.insert(SymbolId(i as u16), i);
        }
        let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
        for col in 0..symbol_count {
            // "Is this column used as a goto for any state?"
            if gotos.iter().any(|row| row[col] != INVALID) {
                nonterminal_to_index.insert(SymbolId(col as u16), col);
            }
        }
        nonterminal_to_index
            .entry(start_symbol)
            .or_insert_with(|| start_symbol.0 as usize);

        // Invariants on EOF / token_count
        let eof_idx = eof_symbol.0 as usize;
        debug_assert!(
            eof_idx > 0 && eof_idx < symbol_count,
            "EOF column must be within 1..symbol_count (got {eof_idx} of {symbol_count})"
        );

        // By project convention: EOF index == token_count + external_token_count.
        // (token_count includes EOF; examples set token_count == eof_idx when externals==0)
        let token_count = eof_idx - external_token_count;

        // Minimal lexing configuration (one mode per state)
        let lex_modes = vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            state_count
        ];

        // Build index_to_symbol from symbol_to_index
        let mut index_to_symbol = vec![SymbolId(0); symbol_count];
        for (symbol_id, index) in &symbol_to_index {
            index_to_symbol[*index] = *symbol_id;
        }

        ParseTable {
            // core grids
            action_table: actions,
            goto_table: gotos,
            // grammar rules
            rules,
            // shapes
            state_count,
            symbol_count,
            // symbol bookkeeping
            symbol_to_index,
            index_to_symbol,
            nonterminal_to_index,
            symbol_metadata: vec![], // tests don't need metadata
            // token layout / sentinels
            token_count,
            external_token_count,
            eof_symbol,
            start_symbol,
            // parsing config
            initial_state: StateId(0),
            // lexing config
            lex_modes,
            extras: vec![],
            external_scanner_states: vec![],
            // advanced features (unused in hand tests)
            dynamic_prec_by_rule: vec![],
            rule_assoc_by_rule: vec![],
            alias_sequences: vec![],
            field_names: vec![],
            field_map: BTreeMap::new(),
            // display / provenance (defaults are fine for tests)
            grammar: Grammar::default(),
            // GOTO indexing mode
            goto_indexing: GotoIndexing::NonterminalMap,
        }
    }

    /// Create an *empty* but valid table for tests that don't care about actions/gotos.
    ///
    /// `terms` = number of real terminals (excluding EOF); `nonterms` = number of non-terminals.
    /// Symbol layout produced:
    ///   0: ERROR, 1..=terms: terminals, (terms+externals+1): EOF, the rest: non-terminals.
    pub fn make_empty_table(
        states: usize,
        terms: usize,
        nonterms: usize,
        externals: usize,
    ) -> ParseTable {
        let states = states.max(1);
        let eof_idx = 1 + terms + externals;
        // Ensure at least one nonterminal column so start_symbol is valid.
        let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
        let symbol_count = eof_idx + 1 + nonterms_eff; // +1 for EOF itself

        let actions = vec![vec![vec![]; symbol_count]; states];
        let gotos = vec![vec![INVALID; symbol_count]; states];

        let start_symbol = SymbolId((eof_idx + 1) as u16); // first nonterminal column (now always exists)
        let eof_symbol = SymbolId(eof_idx as u16);

        make_minimal_table(actions, gotos, vec![], start_symbol, eof_symbol, externals)
    }

    #[cfg(test)]
    mod smoke {
        use super::*;
        #[test]
        fn empty_is_constructible() {
            let _ = make_empty_table(2, 1, 0, 0);
        }

        // Handy macro for the simple case.
        #[macro_export]
        macro_rules! empty_table {
            (states: $s:expr, terms: $t:expr, nonterms: $n:expr $(, externals: $e:expr)? ) => {{
                let e = 0 $(+ $e)?;
                $crate::test_helpers::test::make_empty_table($s, $t, $n, e)
            }};
        }
    }
}
