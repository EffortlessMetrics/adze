//! ParseTable serialization v9 comprehensive tests.
//!
//! 80+ tests across 10 categories (8 per category):
//!   sv9_roundtrip_*      — basic roundtrip smoke tests
//!   sv9_field_*          — field-level preservation
//!   sv9_actions_*        — action cell fidelity
//!   sv9_goto_*           — goto table integrity
//!   sv9_rules_*          — rule info preservation
//!   sv9_corrupt_*        — invalid input rejection
//!   sv9_determinism_*    — determinism / idempotence
//!   sv9_size_*           — byte-size properties
//!   sv9_complex_*        — complex grammar shapes
//!   sv9_api_*            — public API contract tests
//!
//! Run with:
//!   cargo test -p adze-glr-core --test serialization_v9 --features serialization -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::{Action, FirstFollowSets, ParseTable, StateId, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_pt(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("first/follow");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn roundtrip(pt: &ParseTable) -> ParseTable {
    let bytes = pt.to_bytes().expect("serialize");
    ParseTable::from_bytes(&bytes).expect("deserialize")
}

// ---------------------------------------------------------------------------
// Grammar factories
// ---------------------------------------------------------------------------

/// S -> a b
fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build()
}

/// S -> a
fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// S -> a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build()
}

/// S -> A, A -> a
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .start("S")
        .build()
}

/// E -> E + E | n (ambiguous)
fn expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build()
}

/// E -> E + T | T, T -> T * F | F, F -> n | ( E )
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("n", r"\d+")
        .token("+", r"\+")
        .token("*", r"\*")
        .token("(", r"\(")
        .token(")", r"\)")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "*", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["n"])
        .rule("F", vec!["(", "E", ")"])
        .start("E")
        .build()
}

/// S -> a | b | c | d | e
fn five_alt_grammar() -> Grammar {
    GrammarBuilder::new("five_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .rule("S", vec!["d"])
        .rule("S", vec!["e"])
        .start("S")
        .build()
}

/// S -> L R, L -> a, R -> b
fn two_nt_grammar() -> Grammar {
    GrammarBuilder::new("two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["L", "R"])
        .rule("L", vec!["a"])
        .rule("R", vec!["b"])
        .start("S")
        .build()
}

/// S -> A B C, A -> a, B -> b, C -> c
fn three_nt_chain_grammar() -> Grammar {
    GrammarBuilder::new("three_nt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B", "C"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("C", vec!["c"])
        .start("S")
        .build()
}

/// S -> S a | a (left-recursive)
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("S", vec!["S", "a"])
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// list -> a list | a (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build()
}

/// S -> a S b | ε (nested)
fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "S", "b"])
        .rule("S", vec![])
        .start("S")
        .build()
}

/// S -> A B, A -> a | ε, B -> b (nullable prefix)
fn nullable_prefix_grammar() -> Grammar {
    GrammarBuilder::new("nullable_prefix")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .rule("B", vec!["b"])
        .start("S")
        .build()
}

/// stmts -> stmt stmts | stmt, stmt -> ID = NUM ;
fn statement_list_grammar() -> Grammar {
    GrammarBuilder::new("stmt_list")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("=", "=")
        .token(";", ";")
        .rule("stmts", vec!["stmt", "stmts"])
        .rule("stmts", vec!["stmt"])
        .rule("stmt", vec!["ID", "=", "NUM", ";"])
        .start("stmts")
        .build()
}

/// S -> a b c d e f (wide RHS)
fn wide_rhs_grammar() -> Grammar {
    GrammarBuilder::new("wide_rhs")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("S", vec!["a", "b", "c", "d", "e", "f"])
        .start("S")
        .build()
}

/// S -> A, A -> B, B -> C, C -> a (deep chain)
fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep_chain")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["a"])
        .start("S")
        .build()
}

/// E -> E + T | T, T -> n — with left-associative precedence
fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["n"])
        .start("E")
        .precedence(1, Associativity::Left, vec!["+"])
        .build()
}

/// S -> A, A -> _inline (with inline rule)
fn inline_grammar() -> Grammar {
    GrammarBuilder::new("inline_test")
        .token("x", "x")
        .rule("S", vec!["Helper"])
        .rule("Helper", vec!["x"])
        .start("S")
        .inline("Helper")
        .build()
}

/// Many rules: S -> R1 | R2 | … | R10, Ri -> ti
fn many_rules_grammar() -> Grammar {
    let mut b = GrammarBuilder::new("many_rules");
    for i in 0..10 {
        let tname = format!("t{i}");
        b = b.token(&tname, &tname);
    }
    for i in 0..10 {
        let rname = format!("R{i}");
        let tname = format!("t{i}");
        b = b.rule(&rname, vec![&tname]);
    }
    // S -> R0 | R1 | … | R9
    for i in 0..10 {
        let rname = format!("R{i}");
        b = b.rule("S", vec![&rname]);
    }
    b.start("S").build()
}

// ===================================================================
// 1. sv9_roundtrip_* — basic roundtrip smoke tests (8 tests)
// ===================================================================

#[test]
fn sv9_roundtrip_simple_grammar() {
    let pt = build_pt(&simple_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

#[test]
fn sv9_roundtrip_single_token() {
    let pt = build_pt(&single_token_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
}

#[test]
fn sv9_roundtrip_chain() {
    let pt = build_pt(&chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

#[test]
fn sv9_roundtrip_two_alt() {
    let pt = build_pt(&two_alt_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
}

#[test]
fn sv9_roundtrip_arithmetic() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

#[test]
fn sv9_roundtrip_nested() {
    let pt = build_pt(&nested_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
}

#[test]
fn sv9_roundtrip_default_table() {
    let pt = ParseTable::default();
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

#[test]
fn sv9_roundtrip_expr_grammar() {
    let pt = build_pt(&expr_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

// ===================================================================
// 2. sv9_field_* — field-level preservation (8 tests)
// ===================================================================

#[test]
fn sv9_field_state_count_preserved() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
}

#[test]
fn sv9_field_symbol_count_preserved() {
    let pt = build_pt(&five_alt_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

#[test]
fn sv9_field_eof_symbol_preserved() {
    let pt = build_pt(&simple_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.eof_symbol, restored.eof_symbol);
}

#[test]
fn sv9_field_start_symbol_preserved() {
    let pt = build_pt(&chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.start_symbol, restored.start_symbol);
}

#[test]
fn sv9_field_initial_state_preserved() {
    let pt = build_pt(&two_nt_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.initial_state, restored.initial_state);
}

#[test]
fn sv9_field_token_count_preserved() {
    let pt = build_pt(&statement_list_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.token_count, restored.token_count);
}

#[test]
fn sv9_field_symbol_maps_preserved() {
    let pt = build_pt(&three_nt_chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.symbol_to_index, restored.symbol_to_index);
    assert_eq!(pt.index_to_symbol, restored.index_to_symbol);
}

#[test]
fn sv9_field_nonterminal_index_preserved() {
    let pt = build_pt(&two_nt_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.nonterminal_to_index, restored.nonterminal_to_index);
    assert_eq!(pt.goto_indexing, restored.goto_indexing);
}

// ===================================================================
// 3. sv9_actions_* — action cell fidelity (8 tests)
// ===================================================================

#[test]
fn sv9_actions_all_cells_match() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_actions_per_state_per_symbol() {
    let g = five_alt_grammar();
    let pt = build_pt(&g);
    let restored = roundtrip(&pt);
    for st in 0..pt.state_count {
        let sid = StateId(st as u16);
        for (&sym, _) in &g.tokens {
            assert_eq!(
                pt.actions(sid, sym),
                restored.actions(sid, sym),
                "action mismatch at state {st}, sym {sym:?}"
            );
        }
    }
}

#[test]
fn sv9_actions_shift_preserved() {
    let pt = build_pt(&simple_grammar());
    let restored = roundtrip(&pt);
    // Verify that initial state has at least one Shift action
    let has_shift = pt
        .action_table
        .iter()
        .flatten()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(has_shift, "original must have shifts");
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_actions_reduce_preserved() {
    let pt = build_pt(&single_token_grammar());
    let restored = roundtrip(&pt);
    let has_reduce = pt
        .action_table
        .iter()
        .flatten()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(has_reduce, "original must have reduces");
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_actions_accept_preserved() {
    let pt = build_pt(&single_token_grammar());
    let restored = roundtrip(&pt);
    let has_accept = pt
        .action_table
        .iter()
        .flatten()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)));
    assert!(has_accept, "original must have accept");
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_actions_multi_action_cells_expr() {
    let pt = build_pt(&expr_grammar());
    let restored = roundtrip(&pt);
    // E -> E + E | n is ambiguous — expect multi-action cells
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_actions_chain_grammar_cells() {
    let pt = build_pt(&chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_actions_stmt_list_cells() {
    let pt = build_pt(&statement_list_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.action_table, restored.action_table);
}

// ===================================================================
// 4. sv9_goto_* — goto table integrity (8 tests)
// ===================================================================

#[test]
fn sv9_goto_table_preserved() {
    let pt = build_pt(&two_nt_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_goto_per_state_per_nt() {
    let pt = build_pt(&three_nt_chain_grammar());
    let restored = roundtrip(&pt);
    for st in 0..pt.state_count {
        let sid = StateId(st as u16);
        for &nt in pt.nonterminal_to_index.keys() {
            assert_eq!(
                pt.goto(sid, nt),
                restored.goto(sid, nt),
                "goto mismatch at state {st}, nt {nt:?}"
            );
        }
    }
}

#[test]
fn sv9_goto_arithmetic() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_goto_chain_has_entries() {
    let pt = build_pt(&chain_grammar());
    let restored = roundtrip(&pt);
    // Chain grammar has at least one goto entry
    assert!(!pt.goto_table.is_empty());
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_goto_deep_chain() {
    let pt = build_pt(&deep_chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_goto_nested_grammar() {
    let pt = build_pt(&nested_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_goto_nullable_prefix() {
    let pt = build_pt(&nullable_prefix_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_goto_left_recursive() {
    let pt = build_pt(&left_recursive_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_table, restored.goto_table);
}

// ===================================================================
// 5. sv9_rules_* — rule info preservation (8 tests)
// ===================================================================

#[test]
fn sv9_rules_count_preserved() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
}

#[test]
fn sv9_rules_lhs_rhs_match() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid), "rule {i} mismatch");
    }
}

#[test]
fn sv9_rules_simple_grammar() {
    let pt = build_pt(&simple_grammar());
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        let (lhs, rhs_len) = pt.rule(rid);
        let (r_lhs, r_rhs_len) = restored.rule(rid);
        assert_eq!(lhs, r_lhs);
        assert_eq!(rhs_len, r_rhs_len);
    }
}

#[test]
fn sv9_rules_expr_grammar() {
    let pt = build_pt(&expr_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn sv9_rules_many_rules() {
    let pt = build_pt(&many_rules_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
}

#[test]
fn sv9_rules_five_alt_all_match() {
    let pt = build_pt(&five_alt_grammar());
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn sv9_rules_wide_rhs() {
    let pt = build_pt(&wide_rhs_grammar());
    let restored = roundtrip(&pt);
    // Wide RHS rule should have rhs_len == 6
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn sv9_rules_stmt_list() {
    let pt = build_pt(&statement_list_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

// ===================================================================
// 6. sv9_corrupt_* — invalid input rejection (8 tests)
// ===================================================================

#[test]
fn sv9_corrupt_empty_bytes() {
    assert!(ParseTable::from_bytes(&[]).is_err());
}

#[test]
fn sv9_corrupt_single_byte() {
    assert!(ParseTable::from_bytes(&[0x42]).is_err());
}

#[test]
fn sv9_corrupt_truncated_halfway() {
    let pt = build_pt(&arithmetic_grammar());
    let bytes = pt.to_bytes().expect("serialize");
    let half = bytes.len() / 2;
    assert!(ParseTable::from_bytes(&bytes[..half]).is_err());
}

#[test]
fn sv9_corrupt_garbage_bytes() {
    let garbage: Vec<u8> = (0u8..200)
        .map(|i| i.wrapping_mul(73).wrapping_add(19))
        .collect();
    assert!(ParseTable::from_bytes(&garbage).is_err());
}

#[test]
fn sv9_corrupt_all_zeros() {
    assert!(ParseTable::from_bytes(&[0u8; 128]).is_err());
}

#[test]
fn sv9_corrupt_all_ones() {
    assert!(ParseTable::from_bytes(&[0xFF; 128]).is_err());
}

#[test]
fn sv9_corrupt_one_byte_short() {
    let pt = build_pt(&chain_grammar());
    let bytes = pt.to_bytes().expect("serialize");
    assert!(ParseTable::from_bytes(&bytes[..bytes.len() - 1]).is_err());
}

#[test]
fn sv9_corrupt_flipped_bits_no_panic() {
    let pt = build_pt(&arithmetic_grammar());
    let mut bytes = pt.to_bytes().expect("serialize");
    if bytes.len() > 10 {
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        bytes[mid + 1] ^= 0xAA;
        bytes[mid + 2] ^= 0x55;
    }
    // Must not panic — result is either Ok or Err
    let _ = ParseTable::from_bytes(&bytes);
}

// ===================================================================
// 7. sv9_determinism_* — determinism / idempotence (8 tests)
// ===================================================================

#[test]
fn sv9_determinism_serialize_twice_same_bytes() {
    let pt = build_pt(&arithmetic_grammar());
    let b1 = pt.to_bytes().expect("s1");
    let b2 = pt.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn sv9_determinism_rebuild_same_grammar_same_bytes() {
    let b1 = build_pt(&simple_grammar()).to_bytes().expect("s1");
    let b2 = build_pt(&simple_grammar()).to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn sv9_determinism_double_roundtrip() {
    let pt = build_pt(&expr_grammar());
    let b1 = pt.to_bytes().expect("s1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn sv9_determinism_triple_roundtrip() {
    let pt = build_pt(&five_alt_grammar());
    let b1 = pt.to_bytes().expect("s1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("s2");
    let t3 = ParseTable::from_bytes(&b2).expect("d2");
    let b3 = t3.to_bytes().expect("s3");
    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

#[test]
fn sv9_determinism_different_grammars_differ() {
    let b1 = build_pt(&single_token_grammar()).to_bytes().expect("s1");
    let b2 = build_pt(&two_alt_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn sv9_determinism_left_vs_right_recursive_differ() {
    let b1 = build_pt(&left_recursive_grammar()).to_bytes().expect("s1");
    let b2 = build_pt(&right_recursive_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn sv9_determinism_chain_vs_deep_chain_differ() {
    let b1 = build_pt(&chain_grammar()).to_bytes().expect("s1");
    let b2 = build_pt(&deep_chain_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn sv9_determinism_nested_vs_chain_differ() {
    let b1 = build_pt(&nested_grammar()).to_bytes().expect("s1");
    let b2 = build_pt(&chain_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

// ===================================================================
// 8. sv9_size_* — byte-size properties (8 tests)
// ===================================================================

#[test]
fn sv9_size_serialized_nonempty() {
    let bytes = build_pt(&simple_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn sv9_size_single_token_nonempty() {
    let bytes = build_pt(&single_token_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn sv9_size_arithmetic_under_100kb() {
    let bytes = build_pt(&arithmetic_grammar()).to_bytes().expect("s");
    assert!(
        bytes.len() < 100_000,
        "serialized size {} exceeds 100KB",
        bytes.len()
    );
}

#[test]
fn sv9_size_more_states_more_bytes() {
    let small = build_pt(&single_token_grammar()).to_bytes().expect("s");
    let large = build_pt(&arithmetic_grammar()).to_bytes().expect("s");
    assert!(
        large.len() > small.len(),
        "arithmetic ({}) should be larger than single_token ({})",
        large.len(),
        small.len()
    );
}

#[test]
fn sv9_size_five_alt_larger_than_two_alt() {
    let two = build_pt(&two_alt_grammar()).to_bytes().expect("s");
    let five = build_pt(&five_alt_grammar()).to_bytes().expect("s");
    assert!(
        five.len() > two.len(),
        "five_alt ({}) should be larger than two_alt ({})",
        five.len(),
        two.len()
    );
}

#[test]
fn sv9_size_default_table_minimal() {
    let bytes = ParseTable::default().to_bytes().expect("s");
    assert!(bytes.len() < 1000, "default table {} bytes", bytes.len());
}

#[test]
fn sv9_size_roundtrip_preserves_length() {
    let pt = build_pt(&expr_grammar());
    let b1 = pt.to_bytes().expect("s1");
    let restored = ParseTable::from_bytes(&b1).expect("d");
    let b2 = restored.to_bytes().expect("s2");
    assert_eq!(b1.len(), b2.len());
}

#[test]
fn sv9_size_stmt_list_reasonable() {
    let bytes = build_pt(&statement_list_grammar()).to_bytes().expect("s");
    assert!(bytes.len() > 10);
    assert!(bytes.len() < 500_000);
}

// ===================================================================
// 9. sv9_complex_* — complex grammar shapes (8 tests)
// ===================================================================

#[test]
fn sv9_complex_precedence_roundtrip() {
    let pt = build_pt(&precedence_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_complex_many_rules_roundtrip() {
    let pt = build_pt(&many_rules_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.rules.len(), restored.rules.len());
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_complex_inline_rules_roundtrip() {
    let pt = build_pt(&inline_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.action_table, restored.action_table);
}

#[test]
fn sv9_complex_nullable_prefix_all_fields() {
    let pt = build_pt(&nullable_prefix_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
    assert_eq!(pt.rules.len(), restored.rules.len());
}

#[test]
fn sv9_complex_wide_rhs_all_fields() {
    let pt = build_pt(&wide_rhs_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_complex_deep_chain_all_fields() {
    let pt = build_pt(&deep_chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn sv9_complex_right_recursive_all_fields() {
    let pt = build_pt(&right_recursive_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn sv9_complex_stmt_list_all_fields() {
    let pt = build_pt(&statement_list_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
    assert_eq!(pt.field_names, restored.field_names);
    assert_eq!(pt.field_map, restored.field_map);
}

// ===================================================================
// 10. sv9_api_* — public API contract tests (8 tests)
// ===================================================================

#[test]
fn sv9_api_eof_method_after_roundtrip() {
    let pt = build_pt(&simple_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.eof(), restored.eof());
}

#[test]
fn sv9_api_start_symbol_method_after_roundtrip() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.start_symbol(), restored.start_symbol());
}

#[test]
fn sv9_api_state_count_nonzero_after_roundtrip() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert!(restored.state_count > 0);
    assert!(restored.symbol_count > 0);
    assert!(!restored.rules.is_empty());
}

#[test]
fn sv9_api_actions_returns_same_slice() {
    let g = simple_grammar();
    let pt = build_pt(&g);
    let restored = roundtrip(&pt);
    for st in 0..pt.state_count {
        let sid = StateId(st as u16);
        for &sym in pt.symbol_to_index.keys() {
            assert_eq!(pt.actions(sid, sym), restored.actions(sid, sym));
        }
    }
}

#[test]
fn sv9_api_goto_returns_same_values() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    for st in 0..pt.state_count {
        let sid = StateId(st as u16);
        for &nt in pt.nonterminal_to_index.keys() {
            assert_eq!(
                pt.goto(sid, nt),
                restored.goto(sid, nt),
                "goto mismatch at state {st}, nt {nt:?}"
            );
        }
    }
}

#[test]
fn sv9_api_rule_returns_same_tuples() {
    let pt = build_pt(&five_alt_grammar());
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn sv9_api_extras_preserved() {
    let pt = build_pt(&right_recursive_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.extras, restored.extras);
}

#[test]
fn sv9_api_lex_modes_preserved() {
    let pt = build_pt(&chain_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.lex_modes.len(), restored.lex_modes.len());
}

// ===================================================================
// Bonus: additional coverage beyond 80 (4 extra tests)
// ===================================================================

#[test]
fn sv9_bonus_large_grammar_performance() {
    // Build a grammar with many alternatives to stress serialization
    let pt = build_pt(&many_rules_grammar());
    let start = std::time::Instant::now();
    let bytes = pt.to_bytes().expect("serialize");
    let _ = ParseTable::from_bytes(&bytes).expect("deserialize");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 5000,
        "roundtrip took too long: {elapsed:?}"
    );
}

#[test]
fn sv9_bonus_dynamic_prec_preserved() {
    let pt = build_pt(&precedence_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.dynamic_prec_by_rule, restored.dynamic_prec_by_rule);
}

#[test]
fn sv9_bonus_rule_assoc_preserved() {
    let pt = build_pt(&precedence_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.rule_assoc_by_rule, restored.rule_assoc_by_rule);
}

#[test]
fn sv9_bonus_alias_sequences_preserved() {
    let pt = build_pt(&arithmetic_grammar());
    let restored = roundtrip(&pt);
    assert_eq!(pt.alias_sequences, restored.alias_sequences);
}
