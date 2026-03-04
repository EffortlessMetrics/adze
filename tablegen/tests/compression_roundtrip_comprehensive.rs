//! Comprehensive roundtrip tests for table compression in adze-tablegen.
//!
//! Covers: compression of real parse tables from grammar pipelines,
//! roundtrip correctness, size reduction, edge cases, and ABI builder integration.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};
use adze_tablegen::compress::{CompressedGotoEntry, TableCompressor};
use adze_tablegen::compression::{
    compress_action_table, compress_goto_table, decompress_action, decompress_goto,
};
use adze_tablegen::{collect_token_indices, eof_accepts_or_reduces};
use std::collections::BTreeMap;

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Build grammar → FIRST/FOLLOW → LR(1) → compress, returning the parse table
/// and compressed tables.
fn pipeline(
    grammar_fn: impl FnOnce() -> adze_ir::Grammar,
) -> (adze_glr_core::ParseTable, adze_tablegen::CompressedTables) {
    let mut grammar = grammar_fn();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton construction failed");
    let token_indices = collect_token_indices(&grammar, &table);
    let start_empty = eof_accepts_or_reduces(&table);
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &token_indices, start_empty)
        .expect("Table compression failed");
    (table, compressed)
}

/// Wrap single actions into GLR action cells for the compression module.
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

// ═══════════════════════════════════════════════════════════════════════════
// 1. Compression of parse tables from simple grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compress_single_token_grammar() {
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build()
    });
    assert!(
        !compressed.action_table.data.is_empty(),
        "action table must have entries"
    );
}

#[test]
fn compress_two_alternative_grammar() {
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn compress_sequence_grammar() {
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn compress_chain_grammar() {
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("x", "x")
            .rule("c", vec!["x"])
            .rule("b", vec!["c"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn compress_left_recursive_grammar() {
    let (pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list")
            .build()
    });
    assert!(
        pt.state_count >= 3,
        "left-recursive grammar needs multiple states"
    );
    assert!(!compressed.action_table.data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Roundtrip: compress then verify action lookups
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn roundtrip_action_table_dedup() {
    let table = single_action_table(vec![
        vec![Action::Shift(StateId(1)), Action::Error, Action::Accept],
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
            assert_eq!(got, expected, "state={state} sym={sym}");
        }
    }
}

#[test]
fn roundtrip_goto_table_sparse() {
    let table = vec![
        vec![None, Some(StateId(3)), None, Some(StateId(5))],
        vec![Some(StateId(1)), None, Some(StateId(2)), None],
        vec![None, None, None, None],
    ];
    let compressed = compress_goto_table(&table);
    for (state, row) in table.iter().enumerate() {
        for (sym, &expected) in row.iter().enumerate() {
            let got = decompress_goto(&compressed, state, sym);
            assert_eq!(got, expected, "GOTO mismatch at state={state} sym={sym}");
        }
    }
}

#[test]
fn roundtrip_small_table_compressor_actions() {
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
    let compressed = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();

    // Verify row_offsets length = n_states + 1
    assert_eq!(compressed.row_offsets.len(), 3);

    // Walk each row and verify entries
    for (state, row) in action_table.iter().enumerate() {
        let start = compressed.row_offsets[state] as usize;
        let end = compressed.row_offsets[state + 1] as usize;
        let entries = &compressed.data[start..end];

        // Count non-error cells in original
        let non_error_count = row.iter().filter(|cell| !cell.is_empty()).count();
        assert_eq!(
            entries.len(),
            non_error_count,
            "state {state}: entry count mismatch"
        );
    }
}

#[test]
fn roundtrip_goto_run_length() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![
        StateId(1),
        StateId(1),
        StateId(1),
        StateId(1),
        StateId(2),
    ]];
    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();

    // Run of 4 should produce RunLength
    let has_rl = compressed
        .data
        .iter()
        .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 1, count: 4 }));
    assert!(has_rl, "run of 4 identical states should produce RunLength");
}

#[test]
fn roundtrip_encode_decode_all_action_types() {
    let compressor = TableCompressor::new();
    let cases: Vec<(Action, u16)> = vec![
        (Action::Shift(StateId(0)), 0),
        (Action::Shift(StateId(100)), 100),
        (Action::Reduce(RuleId(0)), 0x8000 | 1),
        (Action::Reduce(RuleId(42)), 0x8000 | 43),
        (Action::Accept, 0xFFFF),
        (Action::Error, 0xFFFE),
        (Action::Recover, 0xFFFD),
    ];
    for (action, expected_encoding) in cases {
        let encoded = compressor.encode_action_small(&action).unwrap();
        assert_eq!(
            encoded, expected_encoding,
            "encoding mismatch for {:?}",
            action
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Table size reduction verification
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dedup_reduces_identical_rows() {
    let row = vec![
        vec![Action::Shift(StateId(5))],
        vec![Action::Reduce(RuleId(1))],
    ];
    let table = vec![row.clone(), row.clone(), row.clone(), row];
    let compressed = compress_action_table(&table);
    assert_eq!(
        compressed.unique_rows.len(),
        1,
        "4 identical rows → 1 unique"
    );
    assert_eq!(compressed.state_to_row.len(), 4);
}

#[test]
fn sparse_goto_uses_fewer_entries_than_cells() {
    let n_states = 10;
    let n_syms = 10;
    let table: Vec<Vec<Option<StateId>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| {
                    if (s + sym) % 7 == 0 {
                        Some(StateId(1))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();
    let compressed = compress_goto_table(&table);
    let total_cells = n_states * n_syms;
    assert!(
        compressed.entries.len() < total_cells,
        "sparse goto ({} entries) should use fewer than {} total cells",
        compressed.entries.len(),
        total_cells
    );
}

#[test]
fn small_table_compressor_only_stores_non_error() {
    let compressor = TableCompressor::new();
    let n_syms = 20;
    // Only 3 of 20 columns have real actions
    let action_table: Vec<Vec<Vec<Action>>> = vec![
        (0..n_syms)
            .map(|i| {
                if i == 2 || i == 7 || i == 15 {
                    vec![Action::Shift(StateId(i as u16))]
                } else {
                    vec![]
                }
            })
            .collect(),
    ];
    let sym_map = BTreeMap::new();
    let compressed = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    assert_eq!(compressed.data.len(), 3, "only 3 non-error entries stored");
}

#[test]
fn pipeline_compression_produces_fewer_entries_than_cells() {
    let (pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    });
    let total_action_cells: usize = pt.action_table.iter().map(|row| row.len()).sum();
    assert!(
        compressed.action_table.data.len() < total_action_cells,
        "compressed entries ({}) should be < total cells ({})",
        compressed.action_table.data.len(),
        total_action_cells
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_state_action_table() {
    let table = vec![vec![vec![Action::Accept]]];
    let compressed = compress_action_table(&table);
    assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Accept);
}

#[test]
fn single_state_goto_table() {
    let table = vec![vec![Some(StateId(0))]];
    let compressed = compress_goto_table(&table);
    assert_eq!(decompress_goto(&compressed, 0, 0), Some(StateId(0)));
}

#[test]
fn all_error_table() {
    let table = vec![vec![vec![]; 10]; 5];
    let compressed = compress_action_table(&table);
    for state in 0..5 {
        for sym in 0..10 {
            assert_eq!(
                decompress_action(&compressed, state, sym),
                Action::Error,
                "all-error at state={state} sym={sym}"
            );
        }
    }
}

#[test]
fn all_none_goto_table() {
    let table = vec![vec![None; 8]; 4];
    let compressed = compress_goto_table(&table);
    assert!(compressed.entries.is_empty());
    for state in 0..4 {
        for sym in 0..8 {
            assert_eq!(decompress_goto(&compressed, state, sym), None);
        }
    }
}

#[test]
fn multi_action_cell_decompresses_first() {
    let table = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ]]];
    let compressed = compress_action_table(&table);
    assert_eq!(
        decompress_action(&compressed, 0, 0),
        Action::Shift(StateId(1)),
        "GLR multi-action cell returns first action"
    );
}

#[test]
fn large_table_30_states_roundtrip() {
    let n_states = 30;
    let n_syms = 8;
    let table: Vec<Vec<Vec<Action>>> = (0..n_states)
        .map(|s| {
            (0..n_syms)
                .map(|sym| match (s + sym) % 4 {
                    0 => vec![Action::Shift(StateId(((s + sym) % 20) as u16))],
                    1 => vec![Action::Reduce(RuleId(((s * sym) % 10) as u16))],
                    2 => vec![Action::Accept],
                    _ => vec![],
                })
                .collect()
        })
        .collect();
    let compressed = compress_action_table(&table);
    for (state, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            let got = decompress_action(&compressed, state, sym);
            assert_eq!(got, expected, "state={state} sym={sym}");
        }
    }
}

#[test]
fn large_symbol_count_60_columns() {
    let n_syms = 60;
    let table: Vec<Vec<Vec<Action>>> = vec![
        (0..n_syms)
            .map(|sym| {
                if sym % 6 == 0 {
                    vec![Action::Shift(StateId((sym % 25) as u16))]
                } else {
                    vec![]
                }
            })
            .collect(),
    ];
    let compressed = compress_action_table(&table);
    for sym in 0..n_syms {
        let expected = if sym % 6 == 0 {
            Action::Shift(StateId((sym % 25) as u16))
        } else {
            Action::Error
        };
        assert_eq!(
            decompress_action(&compressed, 0, sym),
            expected,
            "sym={sym}"
        );
    }
}

#[test]
fn goto_all_same_state_uses_run_length() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(42); 10]];
    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();
    let has_rl = compressed.data.iter().any(|e| {
        matches!(
            e,
            CompressedGotoEntry::RunLength {
                state: 42,
                count: 10
            }
        )
    });
    assert!(has_rl, "10 identical gotos should produce RunLength");
}

#[test]
fn goto_alternating_states_no_run_length() {
    let compressor = TableCompressor::new();
    let goto_table = vec![vec![StateId(1), StateId(2), StateId(1), StateId(2)]];
    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();
    let all_single = compressed
        .data
        .iter()
        .all(|e| matches!(e, CompressedGotoEntry::Single(_)));
    assert!(
        all_single,
        "alternating states should all be Single entries"
    );
}

#[test]
fn empty_action_table_compresses() {
    let compressor = TableCompressor::new();
    let action_table: Vec<Vec<Vec<Action>>> = vec![vec![]; 3];
    let sym_map = BTreeMap::new();
    let result = compressor.compress_action_table_small(&action_table, &sym_map);
    assert!(result.is_ok());
    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 4); // 3 states + 1
    assert!(compressed.data.is_empty());
}

#[test]
fn empty_goto_table_compresses() {
    let compressor = TableCompressor::new();
    let goto_table: Vec<Vec<StateId>> = vec![];
    let result = compressor.compress_goto_table_small(&goto_table);
    assert!(result.is_ok());
    let compressed = result.unwrap();
    assert_eq!(compressed.row_offsets.len(), 1); // 0 states + 1
    assert!(compressed.data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_deterministic_row_dedup() {
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
    assert_eq!(c1.unique_rows, c2.unique_rows);
    assert_eq!(c1.state_to_row, c2.state_to_row);
}

#[test]
fn compression_deterministic_small_table() {
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
    assert_eq!(c1.row_offsets, c2.row_offsets);
    assert_eq!(c1.default_actions, c2.default_actions);
    assert_eq!(c1.data.len(), c2.data.len());
    for (e1, e2) in c1.data.iter().zip(c2.data.iter()) {
        assert_eq!(e1.symbol, e2.symbol);
        assert_eq!(e1.action, e2.action);
    }
}

#[test]
fn pipeline_compression_deterministic() {
    let build = || {
        let mut grammar = GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        let token_indices = collect_token_indices(&grammar, &table);
        let start_empty = eof_accepts_or_reduces(&table);
        let compressor = TableCompressor::new();
        compressor
            .compress(&table, &token_indices, start_empty)
            .unwrap()
    };
    let c1 = build();
    let c2 = build();
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
    assert_eq!(c1.action_table.row_offsets, c2.action_table.row_offsets);
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
    assert_eq!(c1.goto_table.row_offsets, c2.goto_table.row_offsets);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Row offsets invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn row_offsets_nondecreasing() {
    let compressor = TableCompressor::new();
    let action_table = vec![
        vec![vec![Action::Shift(StateId(0))]; 5],
        vec![vec![]; 5],
        vec![vec![Action::Reduce(RuleId(1))]; 5],
        vec![vec![]; 5],
    ];
    let sym_map = BTreeMap::new();
    let compressed = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    assert_eq!(compressed.row_offsets.len(), 5); // 4 states + 1
    for pair in compressed.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0], "offsets must be non-decreasing");
    }
}

#[test]
fn goto_row_offsets_nondecreasing() {
    let compressor = TableCompressor::new();
    let goto_table = vec![
        vec![StateId(1), StateId(2)],
        vec![StateId(3), StateId(3), StateId(3)],
    ];
    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();
    assert_eq!(compressed.row_offsets.len(), 3); // 2 states + 1
    for pair in compressed.row_offsets.windows(2) {
        assert!(pair[1] >= pair[0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Encode/decode edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn encode_max_valid_shift_state() {
    let compressor = TableCompressor::new();
    // Max valid shift state: 0x7FFF (32767)
    let encoded = compressor
        .encode_action_small(&Action::Shift(StateId(0x7FFF)))
        .unwrap();
    assert_eq!(encoded, 0x7FFF);
}

#[test]
fn encode_max_valid_reduce_rule() {
    let compressor = TableCompressor::new();
    // Max valid reduce rule: 0x3FFF (16383)
    let encoded = compressor
        .encode_action_small(&Action::Reduce(RuleId(0x3FFF)))
        .unwrap();
    assert_eq!(encoded, 0x8000 | (0x3FFF + 1));
}

#[test]
fn encode_shift_too_large_fails() {
    let compressor = TableCompressor::new();
    let result = compressor.encode_action_small(&Action::Shift(StateId(0x8000)));
    assert!(result.is_err(), "shift state >= 0x8000 should fail");
}

#[test]
fn encode_reduce_too_large_fails() {
    let compressor = TableCompressor::new();
    let result = compressor.encode_action_small(&Action::Reduce(RuleId(0x4000)));
    assert!(result.is_err(), "reduce rule >= 0x4000 should fail");
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. ABI builder integration
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn abi_builder_with_compressed_tables() {
    use adze_tablegen::abi_builder::AbiLanguageBuilder;

    let mut grammar = GrammarBuilder::new("roundtrip_abi")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let token_indices = collect_token_indices(&grammar, &table);
    let start_empty = eof_accepts_or_reduces(&table);
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &token_indices, start_empty)
        .unwrap();

    let builder = AbiLanguageBuilder::new(&grammar, &table).with_compressed_tables(&compressed);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(
        code_str.contains("TSLanguage"),
        "generated code must reference TSLanguage"
    );
    assert!(
        code_str.contains("PARSE_TABLE"),
        "generated code must contain PARSE_TABLE"
    );
}

#[test]
fn abi_builder_generates_valid_syntax() {
    use adze_tablegen::abi_builder::AbiLanguageBuilder;

    let mut grammar = GrammarBuilder::new("syncheck")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let token_indices = collect_token_indices(&grammar, &table);
    let start_empty = eof_accepts_or_reduces(&table);
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &token_indices, start_empty)
        .unwrap();

    let builder = AbiLanguageBuilder::new(&grammar, &table).with_compressed_tables(&compressed);
    let code = builder.generate();
    let code_str = code.to_string();

    // Parse with syn to verify it's valid Rust syntax
    let result = syn::parse_str::<syn::File>(&code_str);
    assert!(
        result.is_ok(),
        "generated code must be valid Rust syntax: {:?}",
        result.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Pipeline with more complex grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_nested_rules() {
    let (_pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("atom", vec!["num"])
            .rule("expr", vec!["atom"])
            .rule("expr", vec!["expr", "plus", "atom"])
            .start("expr")
            .build()
    });
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn pipeline_multiple_nonterminals() {
    let (pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("x", vec!["a"])
            .rule("y", vec!["b"])
            .rule("z", vec!["c"])
            .rule("start", vec!["x", "y", "z"])
            .start("start")
            .build()
    });
    // Multiple nonterminals should create goto entries
    assert!(!compressed.goto_table.data.is_empty());
    assert!(pt.state_count >= 4, "need states for each shift + reduces");
}

#[test]
fn pipeline_validates_compressed_tables() {
    let (pt, compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build()
    });
    // validate() currently returns Ok unconditionally, but this tests the API
    let result = compressed.validate(&pt);
    assert!(result.is_ok(), "validation should pass: {:?}", result.err());
}

#[test]
fn pipeline_compressed_parse_table_metadata() {
    use adze_tablegen::CompressedParseTable;

    let (pt, _compressed) = pipeline(|| {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build()
    });
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
    assert_eq!(cpt.state_count(), pt.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Default actions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn default_actions_always_error_when_optimization_disabled() {
    let compressor = TableCompressor::new();
    // All cells are Reduce(0), but default should still be Error
    let action_table = vec![vec![vec![Action::Reduce(RuleId(0))]; 10]; 3];
    let sym_map = BTreeMap::new();
    let compressed = compressor
        .compress_action_table_small(&action_table, &sym_map)
        .unwrap();
    for d in &compressed.default_actions {
        assert_eq!(*d, Action::Error, "default optimization is disabled");
    }
}
