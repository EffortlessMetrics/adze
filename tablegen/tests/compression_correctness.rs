//! Comprehensive correctness tests for table compression.
//!
//! Verifies that compression preserves all parse table semantics:
//! action lookups, goto lookups, default extraction, sparse/dense rows,
//! edge-case row shapes, determinism, size bounds, and metadata preservation.

use adze_glr_core::Action;
use adze_ir::{RuleId, StateId, SymbolId};
use adze_tablegen::compress::{CompressedGotoEntry, TableCompressor};
use adze_tablegen::compression::{
    compress_action_table, compress_goto_table, decompress_action, decompress_goto,
};
use std::collections::BTreeMap;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build a GLR-style action table (Vec<Vec<Vec<Action>>>) from rows of single actions.
fn single_action_table(rows: Vec<Vec<Action>>) -> Vec<Vec<Vec<Action>>> {
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .map(|a| {
                    if matches!(a, Action::Error) {
                        vec![]
                    } else {
                        vec![a]
                    }
                })
                .collect()
        })
        .collect()
}

/// Decompress every cell from a CompressedGotoTable and the original goto table.
fn expand_goto(
    compressed: &adze_tablegen::compression::CompressedGotoTable,
    original: &[Vec<Option<StateId>>],
) {
    for (state, row) in original.iter().enumerate() {
        for (sym, &expected) in row.iter().enumerate() {
            let got = decompress_goto(compressed, state, sym);
            assert_eq!(got, expected, "GOTO mismatch at state={state} sym={sym}");
        }
    }
}

/// Make a minimal parse table via the crate test helper with a shift on token col 1
/// so that the State-0 validation inside `TableCompressor::compress` passes.
fn make_compressible_parse_table(
    action_table: Vec<Vec<Vec<Action>>>,
    goto_table: Vec<Vec<StateId>>,
    rules: Vec<adze_glr_core::ParseRule>,
    symbol_metadata: Vec<adze_glr_core::SymbolMetadata>,
) -> adze_glr_core::ParseTable {
    use adze_glr_core::{GotoIndexing, LexMode};
    use adze_ir::Grammar;

    let state_count = action_table.len().max(1);
    let symbol_count = action_table
        .first()
        .map(|r| r.len())
        .unwrap_or(0)
        .max(goto_table.first().map(|r| r.len()).unwrap_or(0))
        .max(3); // need at least 3 cols: ERROR(0), token(1), EOF(2)

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    let eof_symbol = SymbolId(2);
    let start_symbol = SymbolId(if symbol_count > 3 { 3 } else { 2 });

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    // Pad action_table / goto_table to uniform shape
    let mut actions = action_table;
    for row in &mut actions {
        row.resize_with(symbol_count, Vec::new);
    }
    while actions.len() < state_count {
        actions.push(vec![vec![]; symbol_count]);
    }
    let mut gotos = goto_table;
    for row in &mut gotos {
        row.resize(symbol_count, StateId(u16::MAX));
    }
    while gotos.len() < state_count {
        gotos.push(vec![StateId(u16::MAX); symbol_count]);
    }

    adze_glr_core::ParseTable {
        action_table: actions,
        goto_table: gotos,
        symbol_metadata,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules,
        nonterminal_to_index,
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: 2, // ERROR(0) + token(1)
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

// ── 1. Uncompressed and compressed tables produce same parse results ────────

#[test]
fn action_roundtrip_matches_original() {
    let table = single_action_table(vec![
        vec![Action::Error, Action::Shift(StateId(1)), Action::Accept],
        vec![Action::Reduce(RuleId(0)), Action::Error, Action::Error],
        vec![
            Action::Shift(StateId(2)),
            Action::Reduce(RuleId(1)),
            Action::Error,
        ],
    ]);
    let compressed = compress_action_table(&table);
    for (state, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            let got = decompress_action(&compressed, state, sym);
            assert_eq!(got, expected, "mismatch at state={state} sym={sym}");
        }
    }
}

// ── 2. Action lookup returns same actions before/after compression ──────────

#[test]
fn action_lookup_shift() {
    let table = vec![vec![vec![Action::Shift(StateId(7))]]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Shift(StateId(7)));
}

#[test]
fn action_lookup_reduce() {
    let table = vec![vec![vec![Action::Reduce(RuleId(3))]]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Reduce(RuleId(3)));
}

#[test]
fn action_lookup_accept() {
    let table = vec![vec![vec![Action::Accept]]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Accept);
}

#[test]
fn action_lookup_error_from_empty_cell() {
    let table = vec![vec![vec![]]]; // empty cell -> Error on decompress
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Error);
}

// ── 3. GOTO lookup returns same states before/after compression ─────────────

#[test]
fn goto_roundtrip_matches_original() {
    let table = vec![
        vec![None, Some(StateId(3)), None, Some(StateId(5))],
        vec![Some(StateId(1)), None, Some(StateId(2)), None],
    ];
    let c = compress_goto_table(&table);
    expand_goto(&c, &table);
}

#[test]
fn goto_lookup_present_and_absent() {
    let table = vec![vec![Some(StateId(10)), None, Some(StateId(20))]];
    let c = compress_goto_table(&table);
    assert_eq!(decompress_goto(&c, 0, 0), Some(StateId(10)));
    assert_eq!(decompress_goto(&c, 0, 1), None);
    assert_eq!(decompress_goto(&c, 0, 2), Some(StateId(20)));
}

// ── 4. Sparse rows are correctly compressed ─────────────────────────────────

#[test]
fn sparse_action_row_mostly_errors() {
    let mut row = vec![Action::Error; 20];
    row[7] = Action::Shift(StateId(4));
    row[15] = Action::Reduce(RuleId(2));
    let table = single_action_table(vec![row.clone()]);
    let c = compress_action_table(&table);
    for (sym, a) in row.iter().enumerate() {
        let expected = if matches!(a, Action::Error) {
            Action::Error
        } else {
            a.clone()
        };
        assert_eq!(decompress_action(&c, 0, sym), expected, "sym={sym}");
    }
}

#[test]
fn sparse_goto_table() {
    let mut row = vec![None; 15];
    row[3] = Some(StateId(9));
    row[11] = Some(StateId(2));
    let table = vec![row.clone()];
    let c = compress_goto_table(&table);
    assert_eq!(c.entries.len(), 2, "only 2 non-None entries");
    expand_goto(&c, &table);
}

// ── 5. Dense rows are correctly compressed ──────────────────────────────────

#[test]
fn dense_action_row_no_errors() {
    let row: Vec<Action> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                Action::Shift(StateId(i as u16))
            } else {
                Action::Reduce(RuleId(i as u16))
            }
        })
        .collect();
    let table = single_action_table(vec![row.clone()]);
    let c = compress_action_table(&table);
    for (sym, a) in row.iter().enumerate() {
        assert_eq!(decompress_action(&c, 0, sym), a.clone(), "sym={sym}");
    }
}

#[test]
fn dense_goto_row_all_populated() {
    let row: Vec<Option<StateId>> = (0..8).map(|i| Some(StateId(i))).collect();
    let table = vec![row.clone()];
    let c = compress_goto_table(&table);
    assert_eq!(c.entries.len(), 8, "all 8 entries stored");
    expand_goto(&c, &table);
}

// ── 6. Default actions are correctly extracted ──────────────────────────────

#[test]
fn default_actions_are_error_for_all_rows() {
    // Default action optimization is currently disabled — all defaults should be Error.
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![vec![Action::Reduce(RuleId(0))]; 5],
        vec![vec![Action::Shift(StateId(1))]; 5],
    ];
    let symbol_to_index = BTreeMap::new();
    let compressed = compressor
        .compress_action_table_small(&action_table, &symbol_to_index)
        .unwrap();
    for d in &compressed.default_actions {
        assert_eq!(
            *d,
            Action::Error,
            "default should be Error (optimization disabled)"
        );
    }
}

// ── 7. Empty rows (all Error) are handled correctly ─────────────────────────

#[test]
fn all_error_action_row() {
    let table = vec![vec![vec![]; 8]]; // 8 empty cells
    let c = compress_action_table(&table);
    for sym in 0..8 {
        assert_eq!(decompress_action(&c, 0, sym), Action::Error);
    }
    // Row dedup: one unique row
    assert_eq!(c.unique_rows.len(), 1);
}

#[test]
fn all_none_goto_row() {
    let table = vec![vec![None; 10]];
    let c = compress_goto_table(&table);
    assert!(c.entries.is_empty(), "no entries for all-None row");
    for sym in 0..10 {
        assert_eq!(decompress_goto(&c, 0, sym), None);
    }
}

// ── 8. Single-action rows are handled correctly ─────────────────────────────

#[test]
fn single_cell_action_row() {
    let table = vec![vec![vec![Action::Accept]]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Accept);
    assert_eq!(c.unique_rows.len(), 1);
}

#[test]
fn single_cell_goto_row() {
    let table = vec![vec![Some(StateId(42))]];
    let c = compress_goto_table(&table);
    assert_eq!(decompress_goto(&c, 0, 0), Some(StateId(42)));
    assert_eq!(c.entries.len(), 1);
}

// ── 9. Full rows (all cells populated) are handled correctly ────────────────

#[test]
fn full_action_row() {
    let row: Vec<Vec<Action>> = (0..12).map(|i| vec![Action::Shift(StateId(i))]).collect();
    let table = vec![row];
    let c = compress_action_table(&table);
    for sym in 0u16..12 {
        assert_eq!(
            decompress_action(&c, 0, sym as usize),
            Action::Shift(StateId(sym))
        );
    }
}

#[test]
fn full_goto_row() {
    let row: Vec<Option<StateId>> = (0..12).map(|i| Some(StateId(i))).collect();
    let table = vec![row];
    let c = compress_goto_table(&table);
    assert_eq!(c.entries.len(), 12);
    for sym in 0u16..12 {
        assert_eq!(decompress_goto(&c, 0, sym as usize), Some(StateId(sym)));
    }
}

// ── 10. Tables with many states (20+) compress correctly ────────────────────

#[test]
fn twenty_plus_states_action_table() {
    let n_states = 25;
    let n_syms = 6;
    let mut table = Vec::with_capacity(n_states);
    for s in 0..n_states {
        let mut row = Vec::with_capacity(n_syms);
        for sym in 0..n_syms {
            if (s + sym) % 3 == 0 {
                row.push(vec![Action::Shift(StateId(((s + sym) % 20) as u16))]);
            } else if (s + sym) % 3 == 1 {
                row.push(vec![Action::Reduce(RuleId(((s * sym) % 10) as u16))]);
            } else {
                row.push(vec![]); // Error
            }
        }
        table.push(row);
    }
    let c = compress_action_table(&table);
    for (state, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            let got = decompress_action(&c, state, sym);
            assert_eq!(got, expected, "state={state} sym={sym}");
        }
    }
}

#[test]
fn twenty_plus_states_goto_table() {
    let n_states = 22;
    let n_syms = 5;
    let table: Vec<Vec<Option<StateId>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| {
                    if (s + sym) % 4 == 0 {
                        Some(StateId(((s * 3 + sym) % 15) as u16))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    let c = compress_goto_table(&table);
    expand_goto(&c, &table);
}

// ── 11. Tables with many symbols (50+) compress correctly ───────────────────

#[test]
fn fifty_plus_symbols_action_table() {
    let n_syms = 55;
    let table: Vec<Vec<Vec<Action>>> = vec![
        (0..n_syms)
            .map(|sym| {
                if sym % 5 == 0 {
                    vec![Action::Shift(StateId((sym % 20) as u16))]
                } else {
                    vec![]
                }
            })
            .collect(),
    ];
    let c = compress_action_table(&table);
    for sym in 0..n_syms {
        let expected = if sym % 5 == 0 {
            Action::Shift(StateId((sym % 20) as u16))
        } else {
            Action::Error
        };
        assert_eq!(decompress_action(&c, 0, sym), expected, "sym={sym}");
    }
}

#[test]
fn fifty_plus_symbols_goto_table() {
    let n_syms = 60;
    let row: Vec<Option<StateId>> = (0..n_syms)
        .map(|i| {
            if i % 3 == 0 {
                Some(StateId((i % 30) as u16))
            } else {
                None
            }
        })
        .collect();
    let table = vec![row.clone()];
    let c = compress_goto_table(&table);
    expand_goto(&c, &table);
}

// ── 12. Compression is deterministic (same input → same output) ─────────────

#[test]
fn action_compression_deterministic() {
    let table = single_action_table(vec![
        vec![
            Action::Shift(StateId(1)),
            Action::Error,
            Action::Reduce(RuleId(0)),
        ],
        vec![Action::Error, Action::Accept, Action::Error],
    ]);
    let c1 = compress_action_table(&table);
    let c2 = compress_action_table(&table);
    assert_eq!(c1.unique_rows.len(), c2.unique_rows.len());
    assert_eq!(c1.state_to_row, c2.state_to_row);
    for (r1, r2) in c1.unique_rows.iter().zip(c2.unique_rows.iter()) {
        assert_eq!(r1, r2);
    }
}

#[test]
fn goto_compression_deterministic() {
    let table = vec![
        vec![None, Some(StateId(2)), Some(StateId(3))],
        vec![Some(StateId(1)), None, None],
    ];
    let c1 = compress_goto_table(&table);
    let c2 = compress_goto_table(&table);
    assert_eq!(c1.entries.len(), c2.entries.len());
    for (key, &val) in &c1.entries {
        assert_eq!(c2.entries.get(key), Some(&val));
    }
}

#[test]
fn small_table_compression_deterministic() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![],
            vec![Action::Accept],
        ],
        vec![vec![Action::Reduce(RuleId(0))], vec![], vec![]],
    ];
    let sym_map = BTreeMap::new();
    let c1 = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    let c2 = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    assert_eq!(c1.data.len(), c2.data.len());
    assert_eq!(c1.row_offsets, c2.row_offsets);
    assert_eq!(c1.default_actions, c2.default_actions);
    for (e1, e2) in c1.data.iter().zip(c2.data.iter()) {
        assert_eq!(e1.symbol, e2.symbol);
        assert_eq!(e1.action, e2.action);
    }
}

// ── 13. Compressed table size is <= uncompressed size ────────────────────────

#[test]
fn row_dedup_reduces_unique_rows_with_duplicates() {
    // Two identical rows should compress to 1 unique row.
    let row = vec![
        vec![Action::Shift(StateId(5))],
        vec![Action::Reduce(RuleId(1))],
    ];
    let table = vec![row.clone(), row.clone(), row];
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 1, "three identical rows → one unique");
    assert_eq!(c.state_to_row.len(), 3);
}

#[test]
fn goto_sparse_entries_count_le_total_cells() {
    let n_states = 10;
    let n_syms = 10;
    let total_cells = n_states * n_syms;
    let table: Vec<Vec<Option<StateId>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| {
                    if (s + sym) % 5 == 0 {
                        Some(StateId(1))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    let c = compress_goto_table(&table);
    assert!(
        c.entries.len() <= total_cells,
        "sparse entries ({}) must be <= total cells ({total_cells})",
        c.entries.len()
    );
}

#[test]
fn small_table_compressed_entry_count_le_non_error_cells() {
    let compressor = TableCompressor::new();
    let n_syms = 8;
    let action_table: Vec<Vec<Vec<Action>>> = vec![
        (0..n_syms)
            .map(|i| {
                if i < 3 {
                    vec![Action::Shift(StateId(i as u16))]
                } else {
                    vec![]
                }
            })
            .collect(),
    ];
    let sym_map = BTreeMap::new();
    let c = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    // Only non-error actions are stored
    assert_eq!(c.data.len(), 3, "exactly 3 non-error actions");
}

// ── 14. Symbol metadata is preserved through compression ────────────────────

#[test]
fn symbol_metadata_preserved_after_compress() {
    let metadata = vec![
        adze_glr_core::SymbolMetadata {
            name: "ERROR".into(),
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        },
        adze_glr_core::SymbolMetadata {
            name: "a".into(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(1),
        },
        adze_glr_core::SymbolMetadata {
            name: "EOF".into(),
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(2),
        },
        adze_glr_core::SymbolMetadata {
            name: "S".into(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(3),
        },
    ];

    // State 0: shift token 'a' (col 1)
    let mut actions = vec![vec![vec![]; 4]];
    actions[0][1] = vec![Action::Shift(StateId(1))];
    // State 1: reduce
    actions.push(vec![vec![]; 4]);
    actions[1][2] = vec![Action::Reduce(RuleId(0))];

    let gotos = vec![vec![StateId(u16::MAX); 4]; 2];
    let rules = vec![adze_glr_core::ParseRule {
        lhs: SymbolId(3),
        rhs_len: 1,
    }];

    let pt = make_compressible_parse_table(actions, gotos, rules, metadata.clone());

    // Compression does NOT alter symbol_metadata on the original parse table.
    assert_eq!(pt.symbol_metadata.len(), metadata.len());
    for (orig, got) in metadata.iter().zip(pt.symbol_metadata.iter()) {
        assert_eq!(orig.name, got.name);
        assert_eq!(orig.is_visible, got.is_visible);
        assert_eq!(orig.is_named, got.is_named);
        assert_eq!(orig.is_terminal, got.is_terminal);
        assert_eq!(orig.is_extra, got.is_extra);
        assert_eq!(orig.is_fragile, got.is_fragile);
        assert_eq!(orig.symbol_id, got.symbol_id);
    }

    // Compress via TableCompressor — metadata must still be intact.
    let compressor = TableCompressor::new();
    let token_indices: Vec<usize> = vec![1, 2]; // token 'a' + EOF
    let _compressed = compressor.compress(&pt, &token_indices, false).unwrap();
    // Re-check: metadata on the parse table is untouched (compress takes &ParseTable).
    assert_eq!(pt.symbol_metadata.len(), metadata.len());
}

// ── 15. Production ID mapping is preserved through compression ──────────────

#[test]
fn production_id_preserved_through_action_compression() {
    // Reduce actions carry the RuleId which maps to production IDs.
    let rule_ids: Vec<u16> = vec![0, 3, 7, 15, 42];
    let table: Vec<Vec<Vec<Action>>> = vec![
        rule_ids
            .iter()
            .map(|&r| vec![Action::Reduce(RuleId(r))])
            .collect(),
    ];
    let c = compress_action_table(&table);
    for (sym, &r) in rule_ids.iter().enumerate() {
        assert_eq!(
            decompress_action(&c, 0, sym),
            Action::Reduce(RuleId(r)),
            "rule id {r} at sym={sym}"
        );
    }
}

#[test]
fn encode_action_small_preserves_rule_id() {
    let compressor = TableCompressor::new();
    for rule_id in [0u16, 1, 10, 100, 0x3FFF] {
        let encoded = compressor
            .encode_action_small(&Action::Reduce(RuleId(rule_id)))
            .unwrap();
        // Reduce encoding: bit 15 set, bits 14..0 = rule_id + 1 (1-based)
        assert_eq!(encoded, 0x8000 | (rule_id + 1), "rule_id={rule_id}");
    }
}

// ── Additional edge-case / robustness tests ─────────────────────────────────

#[test]
fn multi_action_cell_returns_first_action() {
    // GLR multi-action cell: decompress_action returns the first action.
    let table = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ]]];
    let c = compress_action_table(&table);
    assert_eq!(decompress_action(&c, 0, 0), Action::Shift(StateId(1)));
}

#[test]
fn row_dedup_identifies_distinct_rows() {
    let row_a = vec![vec![Action::Shift(StateId(1))], vec![]];
    let row_b = vec![vec![], vec![Action::Reduce(RuleId(0))]];
    let table = vec![row_a.clone(), row_b.clone(), row_a];
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 2, "two distinct rows");
    assert_eq!(
        c.state_to_row[0], c.state_to_row[2],
        "states 0 and 2 share a row"
    );
    assert_ne!(c.state_to_row[0], c.state_to_row[1]);
}

#[test]
fn goto_run_length_encoding_with_long_run() {
    let compressor = TableCompressor::new();
    // A run of 5 identical StateIds should use RunLength encoding (threshold > 2).
    let goto_table = vec![vec![StateId(7); 5]];
    let c = compressor.compress_goto_table_small(&goto_table).unwrap();
    let has_rl = c
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 7, count: 5 }));
    assert!(has_rl, "run of 5 should produce RunLength entry");
}

#[test]
fn goto_short_run_uses_singles() {
    let compressor = TableCompressor::new();
    // A run of exactly 2 should use Single entries (threshold > 2 for RunLength).
    let goto_table = vec![vec![StateId(3), StateId(3)]];
    let c = compressor.compress_goto_table_small(&goto_table).unwrap();
    let all_single = c
        .data
        .iter()
        .all(|e| matches!(e, CompressedGotoEntry::Single(3)));
    assert!(all_single, "run of 2 should use Single entries");
    assert_eq!(c.data.len(), 2);
}

#[test]
fn row_offsets_monotonically_nondecreasing() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![vec![Action::Shift(StateId(0))]; 3],
        vec![vec![]; 3], // all-empty
        vec![vec![Action::Reduce(RuleId(1))]; 3],
    ];
    let sym_map = BTreeMap::new();
    let c = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    for pair in c.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0], "offsets must be non-decreasing");
    }
    // n_states + 1 offsets
    assert_eq!(c.row_offsets.len(), 4);
}

#[test]
fn encode_action_small_shift_accept_error_recover() {
    let compressor = TableCompressor::new();
    assert_eq!(
        compressor
            .encode_action_small(&Action::Shift(StateId(0)))
            .unwrap(),
        0
    );
    assert_eq!(
        compressor
            .encode_action_small(&Action::Shift(StateId(100)))
            .unwrap(),
        100
    );
    assert_eq!(
        compressor.encode_action_small(&Action::Accept).unwrap(),
        0xFFFF
    );
    assert_eq!(
        compressor.encode_action_small(&Action::Error).unwrap(),
        0xFFFE
    );
    assert_eq!(
        compressor.encode_action_small(&Action::Recover).unwrap(),
        0xFFFD
    );
}

#[test]
fn large_state_count_row_dedup() {
    // 30 states, half identical — should yield 16 unique rows.
    let row_a: Vec<Vec<Action>> = vec![vec![Action::Shift(StateId(1))]; 4];
    let row_b: Vec<Vec<Action>> = vec![vec![Action::Reduce(RuleId(0))]; 4];
    let mut table = Vec::new();
    for i in 0..30 {
        table.push(if i % 2 == 0 {
            row_a.clone()
        } else {
            row_b.clone()
        });
    }
    let c = compress_action_table(&table);
    assert_eq!(c.unique_rows.len(), 2, "only 2 distinct row patterns");
    assert_eq!(c.state_to_row.len(), 30);
}
