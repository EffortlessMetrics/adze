//! Comprehensive edge-case tests for `TableCompressor` and related types.
//!
//! Covers: minimal/multi-token/precedence grammars, round-trip compression,
//! AbiLanguageBuilder generation, NodeTypesGenerator output, StaticLanguageGenerator
//! code generation, empty/single/many-state tables, and Debug/Clone traits.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId};
use adze_tablegen::compress::{
    CompressedActionEntry, CompressedActionTable, CompressedGotoEntry, CompressedGotoTable,
    CompressedParseTable, CompressedTables, TableCompressor,
};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};
use std::collections::BTreeMap;

// ============================================================================
// Helpers
// ============================================================================

/// Build grammar + parse table from a GrammarBuilder.
fn build(mut grammar: Grammar) -> (Grammar, adze_glr_core::ParseTable) {
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton failed");
    (grammar, table)
}

/// Sorted, deduped token indices for a parse table.
fn token_indices(pt: &adze_glr_core::ParseTable) -> Vec<usize> {
    let mut v: Vec<usize> = pt.symbol_to_index.values().copied().collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Helper: compress a grammar through the full pipeline.
fn compress_grammar(grammar: Grammar) -> (Grammar, adze_glr_core::ParseTable, CompressedTables) {
    let (g, pt) = build(grammar);
    let ti = token_indices(&pt);
    let sce = adze_tablegen::eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new()
        .compress(&pt, &ti, sce)
        .expect("compression failed");
    (g, pt, ct)
}

/// Minimal grammar: S → x
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

/// Two-token grammar: S → a b
fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

/// Alternatives grammar: S → a | b | c
fn alt_grammar() -> Grammar {
    GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build()
}

/// Chain grammar: S → A, A → B, B → x
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("B", vec!["x"])
        .rule("A", vec!["B"])
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

/// Nullable grammar: S → A, A → a A | ε
fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("start", vec!["A"])
        .rule("A", vec!["a", "A"])
        .rule("A", vec![])
        .start("start")
        .build()
}

/// Four-token grammar: S → a b c d
fn four_token_grammar() -> Grammar {
    GrammarBuilder::new("four_tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build()
}

/// Multi-rule grammar: S → A | B; A → a; B → b
fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .start("start")
        .build()
}

// ============================================================================
// 1. TableCompressor with minimal grammars
// ============================================================================

#[test]
fn tc_minimal_compresses_ok() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    assert!(!ct.action_table.row_offsets.is_empty());
}

#[test]
fn tc_minimal_row_offsets_monotonic() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    for w in ct.action_table.row_offsets.windows(2) {
        assert!(w[1] >= w[0], "non-monotonic row offsets");
    }
}

#[test]
fn tc_minimal_goto_row_offsets_monotonic() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    for w in ct.goto_table.row_offsets.windows(2) {
        assert!(w[1] >= w[0], "non-monotonic goto row offsets");
    }
}

#[test]
fn tc_minimal_has_shift_action() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    assert!(
        ct.action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Shift(_))),
        "expected at least one shift"
    );
}

#[test]
fn tc_minimal_has_reduce_or_accept() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    assert!(
        ct.action_table
            .data
            .iter()
            .any(|e| matches!(e.action, Action::Reduce(_) | Action::Accept)),
        "expected reduce or accept"
    );
}

#[test]
fn tc_minimal_default_actions_all_error() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    for da in &ct.action_table.default_actions {
        assert_eq!(*da, Action::Error, "default optimization is disabled");
    }
}

#[test]
fn tc_minimal_threshold() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    assert_eq!(ct.small_table_threshold, 32768);
}

// ============================================================================
// 2. TableCompressor with multiple tokens
// ============================================================================

#[test]
fn tc_two_token_compresses_ok() {
    let (_, _, ct) = compress_grammar(two_token_grammar());
    assert!(
        ct.action_table.data.len() >= 2,
        "should encode shifts for 2 tokens"
    );
}

#[test]
fn tc_four_token_compresses_ok() {
    let (_, _, ct) = compress_grammar(four_token_grammar());
    assert!(!ct.action_table.data.is_empty());
}

#[test]
fn tc_alt_grammar_compresses_ok() {
    let (_, _, ct) = compress_grammar(alt_grammar());
    // Three alternatives → at least 3 shift entries in state 0
    let shift_count = ct
        .action_table
        .data
        .iter()
        .filter(|e| matches!(e.action, Action::Shift(_)))
        .count();
    assert!(shift_count >= 3, "expected >= 3 shifts, got {shift_count}");
}

#[test]
fn tc_chain_grammar_compresses_ok() {
    let (_, _, _ct) = compress_grammar(chain_grammar());
}

#[test]
fn tc_multi_rule_compresses_ok() {
    let (_, _, ct) = compress_grammar(multi_rule_grammar());
    let has_reduce = ct
        .action_table
        .data
        .iter()
        .any(|e| matches!(e.action, Action::Reduce(_)));
    assert!(has_reduce, "multi-rule grammar must produce reduce actions");
}

// ============================================================================
// 3. TableCompressor with nullable / precedence-like grammars
// ============================================================================

#[test]
fn tc_nullable_start_can_be_empty() {
    let (_g, pt) = build(nullable_grammar());
    let sce = adze_tablegen::eof_accepts_or_reduces(&pt);
    assert!(
        sce,
        "nullable grammar should accept/reduce on EOF at state 0"
    );
    let ti = token_indices(&pt);
    let ct = TableCompressor::new()
        .compress(&pt, &ti, sce)
        .expect("nullable compression");
    assert!(!ct.action_table.row_offsets.is_empty());
}

#[test]
fn tc_nullable_has_accept_action() {
    let (_, _, ct) = compress_grammar(nullable_grammar());
    let has_accept = ct
        .action_table
        .data
        .iter()
        .any(|e| matches!(e.action, Action::Accept));
    assert!(has_accept, "nullable grammar must have accept action");
}

#[test]
fn tc_nullable_row_offsets_len() {
    let (_, pt, ct) = compress_grammar(nullable_grammar());
    assert_eq!(
        ct.action_table.row_offsets.len(),
        pt.state_count + 1,
        "row_offsets should be state_count + 1"
    );
}

// ============================================================================
// 4. Round-trip compression checks
// ============================================================================

#[test]
fn roundtrip_deterministic_minimal() {
    let g = minimal_grammar();
    let (_, pt) = build(g.clone());
    let ti = token_indices(&pt);
    let sce = adze_tablegen::eof_accepts_or_reduces(&pt);
    let c = TableCompressor::new();
    let r1 = c.compress(&pt, &ti, sce).unwrap();
    let r2 = c.compress(&pt, &ti, sce).unwrap();
    assert_eq!(r1.action_table.row_offsets, r2.action_table.row_offsets);
    assert_eq!(r1.goto_table.row_offsets, r2.goto_table.row_offsets);
    assert_eq!(r1.action_table.data.len(), r2.action_table.data.len());
}

#[test]
fn roundtrip_deterministic_multi() {
    let g = multi_rule_grammar();
    let (_, pt) = build(g.clone());
    let ti = token_indices(&pt);
    let sce = adze_tablegen::eof_accepts_or_reduces(&pt);
    let c = TableCompressor::new();
    let r1 = c.compress(&pt, &ti, sce).unwrap();
    let r2 = c.compress(&pt, &ti, sce).unwrap();
    assert_eq!(r1.action_table.data.len(), r2.action_table.data.len());
    assert_eq!(r1.goto_table.data.len(), r2.goto_table.data.len());
}

#[test]
fn roundtrip_action_symbols_match() {
    let (_, _, ct) = compress_grammar(minimal_grammar());
    // Every encoded entry should have a symbol < u16::MAX
    for entry in &ct.action_table.data {
        assert!(entry.symbol < u16::MAX, "unexpected sentinel symbol");
    }
}

#[test]
fn roundtrip_goto_offsets_bound_data() {
    let (_, _, ct) = compress_grammar(two_token_grammar());
    let last = *ct.goto_table.row_offsets.last().unwrap() as usize;
    assert_eq!(
        last,
        ct.goto_table.data.len(),
        "sentinel must equal data len"
    );
}

#[test]
fn roundtrip_action_offsets_bound_data() {
    let (_, _, ct) = compress_grammar(alt_grammar());
    let last = *ct.action_table.row_offsets.last().unwrap() as usize;
    assert_eq!(
        last,
        ct.action_table.data.len(),
        "sentinel must equal data len"
    );
}

// ============================================================================
// 5. AbiLanguageBuilder generation
// ============================================================================

#[test]
fn abi_builder_minimal_generates() {
    let (g, pt) = build(minimal_grammar());
    let builder = AbiLanguageBuilder::new(&g, &pt);
    let ts = builder.generate();
    assert!(!ts.is_empty(), "generated code must not be empty");
}

#[test]
fn abi_builder_two_token_generates() {
    let (g, pt) = build(two_token_grammar());
    let ts = builder_generate(&g, &pt);
    assert!(!ts.is_empty());
}

fn builder_generate(g: &Grammar, pt: &adze_glr_core::ParseTable) -> String {
    AbiLanguageBuilder::new(g, pt).generate().to_string()
}

#[test]
fn abi_builder_alt_generates() {
    let (g, pt) = build(alt_grammar());
    let code = builder_generate(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_chain_generates() {
    let (g, pt) = build(chain_grammar());
    let code = builder_generate(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_nullable_generates() {
    let (g, pt) = build(nullable_grammar());
    let code = builder_generate(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_with_compressed_tables() {
    let (g, pt, ct) = compress_grammar(minimal_grammar());
    let builder = AbiLanguageBuilder::new(&g, &pt).with_compressed_tables(&ct);
    let code = builder.generate();
    assert!(!code.is_empty());
}

// ============================================================================
// 6. NodeTypesGenerator output validation
// ============================================================================

#[test]
fn node_types_minimal_valid_json() {
    let (g, _) = build(minimal_grammar());
    let gen_r = NodeTypesGenerator::new(&g);
    let json_str = gen_r.generate().expect("generate failed");
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_two_token_valid_json() {
    let (g, _) = build(two_token_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_alt_has_entries() {
    let (g, _) = build(alt_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = val.as_array().unwrap();
    assert!(!arr.is_empty(), "alt grammar should have node types");
}

#[test]
fn node_types_chain_has_entries() {
    let (g, _) = build(chain_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(!val.as_array().unwrap().is_empty());
}

#[test]
fn node_types_nullable_valid() {
    let (g, _) = build(nullable_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_entries_have_type_field() {
    let (g, _) = build(multi_rule_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.get("type").is_some(), "every entry must have 'type'");
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let (g, _) = build(multi_rule_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "every entry must have 'named'"
        );
    }
}

// ============================================================================
// 7. StaticLanguageGenerator code generation
// ============================================================================

#[test]
fn static_gen_minimal_code() {
    let (g, pt) = build(minimal_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let code = gen_inst.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn static_gen_minimal_node_types() {
    let (g, pt) = build(minimal_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let nt = gen_inst.generate_node_types();
    assert!(!nt.is_empty());
}

#[test]
fn static_gen_two_token_code() {
    let (g, pt) = build(two_token_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let code = gen_inst.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn static_gen_alt_code() {
    let (g, pt) = build(alt_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let code = gen_inst.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn static_gen_chain_code() {
    let (g, pt) = build(chain_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let code = gen_inst.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn static_gen_nullable_start_can_be_empty() {
    let (g, pt) = build(nullable_grammar());
    let mut gen_inst = StaticLanguageGenerator::new(g, pt);
    gen_inst.set_start_can_be_empty(true);
    let code = gen_inst.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn static_gen_node_types_valid_json() {
    let (g, pt) = build(alt_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let nt = gen_inst.generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&nt).expect("should be valid JSON");
}

// ============================================================================
// 8. Edge cases: empty parse tables, single state, many states
// ============================================================================

#[test]
fn compress_empty_action_table_direct() {
    let c = TableCompressor::new();
    let at = vec![vec![]; 3];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    assert_eq!(res.row_offsets.len(), 4);
    assert!(res.data.is_empty());
}

#[test]
fn compress_single_state_action_table() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![Action::Accept]]];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    assert_eq!(res.row_offsets.len(), 2);
    assert_eq!(res.data.len(), 1);
}

#[test]
fn compress_many_states_action_table() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![Action::Shift(StateId(1))]; 10]; 200];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    assert_eq!(res.row_offsets.len(), 201);
    assert_eq!(res.default_actions.len(), 200);
}

#[test]
fn compress_empty_goto_table_direct() {
    let c = TableCompressor::new();
    let gt: Vec<Vec<StateId>> = vec![vec![]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(res.row_offsets.len(), 2);
    assert!(res.data.is_empty());
}

#[test]
fn compress_single_goto_row() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(5)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(res.data.len(), 1);
}

#[test]
fn compress_goto_long_run() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(7); 50]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(res.data.iter().any(|e| matches!(
        e,
        CompressedGotoEntry::RunLength {
            state: 7,
            count: 50
        }
    )));
}

#[test]
fn compress_goto_two_runs() {
    let c = TableCompressor::new();
    let gt = vec![vec![
        StateId(1),
        StateId(1),
        StateId(1),
        StateId(2),
        StateId(2),
        StateId(2),
    ]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    let rl_count = res
        .data
        .iter()
        .filter(|e| matches!(e, CompressedGotoEntry::RunLength { .. }))
        .count();
    assert_eq!(rl_count, 2, "two distinct runs of length 3");
}

#[test]
fn compress_goto_short_run_uses_singles() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(3), StateId(3)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(
        res.data
            .iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(3)))
    );
    assert_eq!(res.data.len(), 2);
}

#[test]
fn compress_action_error_cells_skipped() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![Action::Error], vec![Action::Shift(StateId(1))]]];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    // Only the shift should be encoded; Error is skipped
    assert_eq!(res.data.len(), 1);
    assert!(matches!(res.data[0].action, Action::Shift(StateId(1))));
}

#[test]
fn compress_action_multi_action_cell() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    assert_eq!(
        res.data.len(),
        2,
        "both actions in same cell should be encoded"
    );
}

// ============================================================================
// 9. Debug/Clone traits on types
// ============================================================================

#[test]
fn compressed_action_entry_debug() {
    let e = CompressedActionEntry::new(1, Action::Accept);
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("Accept"), "Debug should mention action type");
}

#[test]
fn compressed_action_entry_clone() {
    let e = CompressedActionEntry::new(2, Action::Shift(StateId(3)));
    let e2 = e.clone();
    assert_eq!(e.symbol, e2.symbol);
}

#[test]
fn compressed_goto_entry_debug_single() {
    let e = CompressedGotoEntry::Single(42);
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("42"));
}

#[test]
fn compressed_goto_entry_debug_runlength() {
    let e = CompressedGotoEntry::RunLength {
        state: 5,
        count: 10,
    };
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("5"));
    assert!(dbg.contains("10"));
}

#[test]
fn compressed_goto_entry_clone_single() {
    let e = CompressedGotoEntry::Single(99);
    let e2 = e.clone();
    assert!(matches!(e2, CompressedGotoEntry::Single(99)));
}

#[test]
fn compressed_goto_entry_clone_runlength() {
    let e = CompressedGotoEntry::RunLength { state: 8, count: 4 };
    let e2 = e.clone();
    match e2 {
        CompressedGotoEntry::RunLength { state, count } => {
            assert_eq!(state, 8);
            assert_eq!(count, 4);
        }
        _ => panic!("expected RunLength"),
    }
}

#[test]
fn compressed_action_table_debug() {
    let t = CompressedActionTable {
        data: vec![],
        row_offsets: vec![0],
        default_actions: vec![Action::Error],
    };
    let dbg = format!("{:?}", t);
    assert!(!dbg.is_empty());
}

#[test]
fn compressed_action_table_clone() {
    let t = CompressedActionTable {
        data: vec![CompressedActionEntry::new(0, Action::Accept)],
        row_offsets: vec![0, 1],
        default_actions: vec![Action::Error],
    };
    let t2 = t.clone();
    assert_eq!(t2.data.len(), 1);
    assert_eq!(t2.row_offsets, vec![0, 1]);
}

#[test]
fn compressed_goto_table_debug() {
    let t = CompressedGotoTable {
        data: vec![CompressedGotoEntry::Single(1)],
        row_offsets: vec![0, 1],
    };
    let dbg = format!("{:?}", t);
    assert!(dbg.contains("Single"));
}

#[test]
fn compressed_goto_table_clone() {
    let t = CompressedGotoTable {
        data: vec![CompressedGotoEntry::RunLength { state: 2, count: 3 }],
        row_offsets: vec![0, 1],
    };
    let t2 = t.clone();
    assert_eq!(t2.data.len(), 1);
}

#[test]
fn compressed_parse_table_new_for_testing() {
    let t = CompressedParseTable::new_for_testing(10, 20);
    assert_eq!(t.symbol_count(), 10);
    assert_eq!(t.state_count(), 20);
}

#[test]
fn compressed_parse_table_from_parse_table() {
    let (_, pt) = build(minimal_grammar());
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
}

// ============================================================================
// 10. Encoding helpers
// ============================================================================

#[test]
fn encode_action_small_shift_zero() {
    let c = TableCompressor::new();
    assert_eq!(
        c.encode_action_small(&Action::Shift(StateId(0))).unwrap(),
        0
    );
}

#[test]
fn encode_action_small_shift_max_valid() {
    let c = TableCompressor::new();
    let v = c
        .encode_action_small(&Action::Shift(StateId(0x7FFF)))
        .unwrap();
    assert_eq!(v, 0x7FFF);
}

#[test]
fn encode_action_small_shift_overflow() {
    let c = TableCompressor::new();
    assert!(
        c.encode_action_small(&Action::Shift(StateId(0x8000)))
            .is_err()
    );
}

#[test]
fn encode_action_small_reduce_zero() {
    let c = TableCompressor::new();
    // Reduce(0) → 0x8000 | (0 + 1) = 0x8001
    assert_eq!(
        c.encode_action_small(&Action::Reduce(RuleId(0))).unwrap(),
        0x8001
    );
}

#[test]
fn encode_action_small_reduce_overflow() {
    let c = TableCompressor::new();
    assert!(
        c.encode_action_small(&Action::Reduce(RuleId(0x4000)))
            .is_err()
    );
}

#[test]
fn encode_action_small_accept() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn encode_action_small_error() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Error).unwrap(), 0xFFFE);
}

#[test]
fn encode_action_small_recover() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Recover).unwrap(), 0xFFFD);
}

// ============================================================================
// 11. TableCompressor Default trait
// ============================================================================

#[test]
fn table_compressor_default() {
    let c = TableCompressor::default();
    // default() should be identical to new()
    let c2 = TableCompressor::new();
    // Both should work the same on an empty action table
    let at = vec![vec![vec![]; 2]; 1];
    let si = BTreeMap::new();
    let r1 = c.compress_action_table_small(&at, &si).unwrap();
    let r2 = c2.compress_action_table_small(&at, &si).unwrap();
    assert_eq!(r1.row_offsets, r2.row_offsets);
}

// ============================================================================
// 12. Validate method on CompressedTables
// ============================================================================

#[test]
fn validate_on_real_tables() {
    let (_, pt, ct) = compress_grammar(minimal_grammar());
    assert!(ct.validate(&pt).is_ok());
}

#[test]
fn validate_on_empty_compressed_rejects() {
    let ct = CompressedTables {
        action_table: CompressedActionTable {
            data: vec![],
            row_offsets: vec![0],
            default_actions: vec![],
        },
        goto_table: CompressedGotoTable {
            data: vec![],
            row_offsets: vec![0],
        },
        small_table_threshold: 32768,
    };
    let (_, pt) = build(minimal_grammar());
    assert!(ct.validate(&pt).is_err());
}

// ============================================================================
// 13. Full pipeline stress
// ============================================================================

#[test]
fn full_pipeline_four_token() {
    let (_, _, ct) = compress_grammar(four_token_grammar());
    assert!(ct.action_table.data.len() >= 4);
}

#[test]
fn full_pipeline_chain_goto_not_empty() {
    let (_, _, ct) = compress_grammar(chain_grammar());
    assert!(
        !ct.goto_table.data.is_empty(),
        "chain grammar must have goto entries"
    );
}

#[test]
fn full_pipeline_multi_rule_default_actions_count() {
    let (_, pt, ct) = compress_grammar(multi_rule_grammar());
    assert_eq!(ct.action_table.default_actions.len(), pt.state_count);
}

#[test]
fn full_pipeline_alt_state_count() {
    let (_, pt, ct) = compress_grammar(alt_grammar());
    // row_offsets len = state_count + 1
    assert_eq!(ct.action_table.row_offsets.len(), pt.state_count + 1);
    assert_eq!(ct.goto_table.row_offsets.len(), pt.state_count + 1);
}

// ============================================================================
// 14. Additional edge-case combinatorics
// ============================================================================

#[test]
fn goto_all_distinct_states() {
    let c = TableCompressor::new();
    let gt = vec![vec![StateId(0), StateId(1), StateId(2), StateId(3)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    // All distinct → all Single entries
    assert_eq!(res.data.len(), 4);
    assert!(
        res.data
            .iter()
            .all(|e| matches!(e, CompressedGotoEntry::Single(_)))
    );
}

#[test]
fn goto_empty_rows() {
    let c = TableCompressor::new();
    let gt: Vec<Vec<StateId>> = vec![vec![], vec![], vec![]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert_eq!(res.row_offsets.len(), 4);
    assert!(res.data.is_empty());
}

#[test]
fn action_all_accept() {
    let c = TableCompressor::new();
    let at = vec![vec![vec![Action::Accept]; 5]];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    assert_eq!(res.data.len(), 5);
    assert!(res.data.iter().all(|e| matches!(e.action, Action::Accept)));
}

#[test]
fn action_interleaved_error_and_shift() {
    let c = TableCompressor::new();
    let at = vec![vec![
        vec![Action::Error],
        vec![Action::Shift(StateId(1))],
        vec![Action::Error],
        vec![Action::Shift(StateId(2))],
    ]];
    let si = BTreeMap::new();
    let res = c.compress_action_table_small(&at, &si).unwrap();
    // Errors are skipped, so only 2 shifts
    assert_eq!(res.data.len(), 2);
}

#[test]
fn abi_builder_four_token_generates() {
    let (g, pt) = build(four_token_grammar());
    let code = builder_generate(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn static_gen_multi_rule_code() {
    let (g, pt) = build(multi_rule_grammar());
    let gen_inst = StaticLanguageGenerator::new(g, pt);
    let code = gen_inst.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn node_types_four_token_valid() {
    let (g, _) = build(four_token_grammar());
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn compress_goto_run_length_boundary_three() {
    let c = TableCompressor::new();
    // Exactly 3 → threshold for RunLength
    let gt = vec![vec![StateId(9), StateId(9), StateId(9)]];
    let res = c.compress_goto_table_small(&gt).unwrap();
    assert!(
        res.data
            .iter()
            .any(|e| matches!(e, CompressedGotoEntry::RunLength { state: 9, count: 3 }))
    );
}

#[test]
fn eof_accepts_or_reduces_on_non_nullable() {
    let (_, pt) = build(minimal_grammar());
    let sce = adze_tablegen::eof_accepts_or_reduces(&pt);
    assert!(!sce, "S → x is not nullable");
}

#[test]
fn collect_token_indices_includes_eof() {
    let (g, pt) = build(minimal_grammar());
    let ti = adze_tablegen::collect_token_indices(&g, &pt);
    // Must include eof_symbol's index
    let eof_idx = pt.symbol_to_index.get(&pt.eof_symbol).copied();
    assert!(
        eof_idx.is_some_and(|idx| ti.contains(&idx)),
        "token indices must include EOF"
    );
}

#[test]
fn collect_token_indices_sorted_deduped() {
    let (g, pt) = build(alt_grammar());
    let ti = adze_tablegen::collect_token_indices(&g, &pt);
    for w in ti.windows(2) {
        assert!(w[0] < w[1], "must be strictly sorted");
    }
}
