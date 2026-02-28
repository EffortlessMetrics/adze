// Property-based tests for table compression determinism and roundtrip correctness.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::*;
use adze_tablegen::compress::{CompressedGotoEntry, TableCompressor};
use adze_tablegen::helpers;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar with `n` terminal alternatives: S -> t0 | t1 | … | t(n-1).
fn grammar_with_n_tokens(n: usize) -> Grammar {
    let mut grammar = Grammar::new("prop".to_string());
    let start = SymbolId((n as u16) + 1); // after all terminals

    for i in 0..n {
        let id = SymbolId((i as u16) + 1); // terminals start at 1
        grammar.tokens.insert(
            id,
            Token {
                name: format!("t{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
        grammar.add_rule(Rule {
            lhs: start,
            rhs: vec![Symbol::Terminal(id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }

    grammar
}

/// End-to-end compression helper: grammar → parse table → compressed tables.
fn compress_grammar(grammar: &Grammar) -> adze_tablegen::Result<adze_tablegen::CompressedTables> {
    let ff = FirstFollowSets::compute(grammar)
        .map_err(|e| adze_tablegen::TableGenError::Compression(format!("{e}")))?;
    let pt = build_lr1_automaton(grammar, &ff)
        .map_err(|e| adze_tablegen::TableGenError::Compression(format!("{e}")))?;
    let token_indices = helpers::collect_token_indices(grammar, &pt);
    let start_can_be_empty = helpers::eof_accepts_or_reduces(&pt);
    TableCompressor::new().compress(&pt, &token_indices, start_can_be_empty)
}

/// Expand a compressed goto table back to flat state-id vectors.
fn expand_goto(compressed: &adze_tablegen::CompressedGotoTable) -> Vec<Vec<u16>> {
    let num_rows = compressed.row_offsets.len().saturating_sub(1);
    let mut rows = Vec::with_capacity(num_rows);

    for r in 0..num_rows {
        let start = compressed.row_offsets[r] as usize;
        let end = compressed.row_offsets[r + 1] as usize;
        let mut row = Vec::new();
        for entry in &compressed.data[start..end] {
            match entry {
                CompressedGotoEntry::Single(s) => row.push(*s),
                CompressedGotoEntry::RunLength { state, count } => {
                    for _ in 0..*count {
                        row.push(*state);
                    }
                }
            }
        }
        rows.push(row);
    }

    rows
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Compressing the same parse table twice must yield identical output.
    #[test]
    fn deterministic_compression(n in 1usize..=6) {
        let grammar = grammar_with_n_tokens(n);

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let pt = build_lr1_automaton(&grammar, &ff).unwrap();
        let token_indices = helpers::collect_token_indices(&grammar, &pt);
        let start_empty = helpers::eof_accepts_or_reduces(&pt);

        let compressor = TableCompressor::new();
        let a = compressor.compress(&pt, &token_indices, start_empty).unwrap();
        let b = compressor.compress(&pt, &token_indices, start_empty).unwrap();

        // Action table must be identical
        prop_assert_eq!(a.action_table.data.len(), b.action_table.data.len());
        prop_assert_eq!(a.action_table.row_offsets, b.action_table.row_offsets);
        prop_assert_eq!(a.action_table.default_actions, b.action_table.default_actions);
        for (ea, eb) in a.action_table.data.iter().zip(b.action_table.data.iter()) {
            prop_assert_eq!(ea.symbol, eb.symbol);
            prop_assert_eq!(&ea.action, &eb.action);
        }

        // Goto table must be identical
        prop_assert_eq!(a.goto_table.row_offsets, b.goto_table.row_offsets);
        prop_assert_eq!(a.goto_table.data.len(), b.goto_table.data.len());
    }

    /// Goto compression roundtrip: expanding the compressed goto table
    /// must recover the original flat representation.
    #[test]
    fn goto_roundtrip(n in 1usize..=6) {
        let grammar = grammar_with_n_tokens(n);
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let pt = build_lr1_automaton(&grammar, &ff).unwrap();
        let token_indices = helpers::collect_token_indices(&grammar, &pt);
        let start_empty = helpers::eof_accepts_or_reduces(&pt);

        let compressed = TableCompressor::new()
            .compress(&pt, &token_indices, start_empty)
            .unwrap();

        // Expand goto and compare against original
        let expanded = expand_goto(&compressed.goto_table);
        prop_assert_eq!(expanded.len(), pt.goto_table.len());

        for (row_idx, (exp_row, orig_row)) in
            expanded.iter().zip(pt.goto_table.iter()).enumerate()
        {
            let orig_u16: Vec<u16> = orig_row.iter().map(|s| s.0).collect();
            prop_assert_eq!(
                exp_row, &orig_u16,
                "goto row {} mismatch after roundtrip", row_idx
            );
        }
    }

    /// Action compression roundtrip: every non-Error action in the original
    /// table must appear in the compressed output for the same row.
    #[test]
    fn action_roundtrip(n in 1usize..=6) {
        let grammar = grammar_with_n_tokens(n);
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let pt = build_lr1_automaton(&grammar, &ff).unwrap();
        let token_indices = helpers::collect_token_indices(&grammar, &pt);
        let start_empty = helpers::eof_accepts_or_reduces(&pt);

        let compressed = TableCompressor::new()
            .compress(&pt, &token_indices, start_empty)
            .unwrap();

        let ct = &compressed.action_table;
        let num_states = ct.row_offsets.len() - 1;
        prop_assert_eq!(num_states, pt.action_table.len());

        for state in 0..num_states {
            let start = ct.row_offsets[state] as usize;
            let end = ct.row_offsets[state + 1] as usize;

            // Collect compressed actions for this state keyed by column
            let mut compressed_actions: std::collections::HashMap<u16, Vec<&Action>> =
                std::collections::HashMap::new();
            for entry in &ct.data[start..end] {
                compressed_actions
                    .entry(entry.symbol)
                    .or_default()
                    .push(&entry.action);
            }

            // Every non-Error original action must be present
            for (col, cell) in pt.action_table[state].iter().enumerate() {
                for action in cell {
                    if *action == Action::Error {
                        continue;
                    }
                    let col_u16 = col as u16;
                    let found = compressed_actions
                        .get(&col_u16)
                        .map_or(false, |v| v.contains(&action));
                    prop_assert!(
                        found,
                        "state {state} col {col}: action {action:?} missing after compression"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Single-row (one-token) table
// ---------------------------------------------------------------------------

#[test]
fn single_token_grammar_compresses() {
    let grammar = grammar_with_n_tokens(1);
    let compressed = compress_grammar(&grammar).expect("single-token grammar must compress");

    // Must have at least one action entry (the shift for the single terminal)
    assert!(
        !compressed.action_table.data.is_empty(),
        "action data must be non-empty for a single-token grammar"
    );
    // Row offsets length == state_count + 1
    assert!(
        compressed.action_table.row_offsets.len() >= 2,
        "must have at least one state"
    );
}

// ---------------------------------------------------------------------------
// Empty action table is rejected (the compressor validates this)
// ---------------------------------------------------------------------------

#[test]
fn empty_table_is_rejected() {
    use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
    use std::collections::BTreeMap;

    // Construct a parse table with an empty action_table.
    let mut sym_map = BTreeMap::new();
    sym_map.insert(SymbolId(0), 0usize); // ERROR
    sym_map.insert(SymbolId(1), 1usize); // a terminal
    sym_map.insert(SymbolId(2), 2usize); // EOF
    sym_map.insert(SymbolId(3), 3usize); // start NT

    let pt = ParseTable {
        action_table: vec![], // empty — should trigger error
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 4,
        symbol_to_index: sym_map,
        index_to_symbol: vec![SymbolId(0), SymbolId(1), SymbolId(2), SymbolId(3)],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(2),
        start_symbol: SymbolId(3),
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    let token_indices = vec![1, 2]; // terminal + EOF
    let result = TableCompressor::new().compress(&pt, &token_indices, false);
    assert!(
        result.is_err(),
        "compressing an empty action table must fail"
    );
}
