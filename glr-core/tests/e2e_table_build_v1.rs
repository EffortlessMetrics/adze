// End-to-end integration tests for the full parse table generation pipeline.
//
// Pipeline: GrammarBuilder → normalize → FirstFollowSets → build_lr1_automaton
//           → ParseTable → serialize → deserialize → verify
//
// Run with:
//   cargo test -p adze-glr-core --test e2e_table_build_v1 -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::{
    Action, FirstFollowSets, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_table(builder: GrammarBuilder) -> ParseTable {
    let mut grammar = builder.build();
    grammar.normalize();
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(&grammar, &ff).expect("build automaton")
}

fn roundtrip(table: &ParseTable) -> ParseTable {
    let bytes = table.to_bytes().expect("serialize");
    ParseTable::from_bytes(&bytes).expect("deserialize")
}

fn has_accept(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    })
}

fn accept_on_eof(table: &ParseTable) -> bool {
    let Some(&eof_col) = table.symbol_to_index.get(&table.eof_symbol) else {
        return false;
    };
    table.action_table.iter().any(|row| {
        row.get(eof_col)
            .is_some_and(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    })
}

fn has_shift(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    })
}

fn has_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    })
}

/// All Shift targets must be < state_count.
fn shifts_target_valid_states(table: &ParseTable) -> bool {
    table.action_table.iter().all(|row| {
        row.iter().all(|cell| {
            cell.iter().all(|a| match a {
                Action::Shift(s) => (s.0 as usize) < table.state_count,
                Action::Fork(actions) => actions.iter().all(|inner| match inner {
                    Action::Shift(s) => (s.0 as usize) < table.state_count,
                    _ => true,
                }),
                _ => true,
            })
        })
    })
}

// ═════════════════════════════════════════════════════════════════════════════
// 1. Full pipeline produces valid parse table (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_single_token_produces_valid_table() {
    let t = build_table(
        GrammarBuilder::new("p1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(t.state_count >= 2);
    assert!(has_accept(&t));
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_two_token_sequence_valid() {
    let t = build_table(
        GrammarBuilder::new("p2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    assert!(t.state_count >= 3);
    assert!(has_shift(&t));
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_three_token_sequence_valid() {
    let t = build_table(
        GrammarBuilder::new("p3")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    assert!(t.state_count >= 4);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_two_alternatives_valid() {
    let t = build_table(
        GrammarBuilder::new("p4")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert!(has_reduce(&t));
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_nonterminal_delegation_valid() {
    let t = build_table(
        GrammarBuilder::new("p5")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start"),
    );
    assert!(has_accept(&t));
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_left_recursive_valid() {
    let t = build_table(
        GrammarBuilder::new("p6")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list"),
    );
    assert!(t.state_count >= 2);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_right_recursive_valid() {
    let t = build_table(
        GrammarBuilder::new("p7")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["a", "list"])
            .start("list"),
    );
    assert!(t.state_count >= 2);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn pipeline_deeply_nested_valid() {
    let t = build_table(
        GrammarBuilder::new("p8")
            .token("x", "x")
            .rule("d", vec!["x"])
            .rule("c", vec!["d"])
            .rule("b", vec!["c"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert!(t.state_count >= 2);
    sanity_check_tables(&t).expect("sanity");
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. Parse table has Accept action for start + EOF (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn accept_on_eof_single_token() {
    let t = build_table(
        GrammarBuilder::new("aeof1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_sequence() {
    let t = build_table(
        GrammarBuilder::new("aeof2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_alternatives() {
    let t = build_table(
        GrammarBuilder::new("aeof3")
            .token("x", "x")
            .token("y", "y")
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .start("start"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_with_nonterminal() {
    let t = build_table(
        GrammarBuilder::new("aeof4")
            .token("n", "[0-9]+")
            .rule("val", vec!["n"])
            .rule("start", vec!["val"])
            .start("start"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_left_recursive() {
    let t = build_table(
        GrammarBuilder::new("aeof5")
            .token("a", "a")
            .rule("items", vec!["a"])
            .rule("items", vec!["items", "a"])
            .start("items"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_right_recursive() {
    let t = build_table(
        GrammarBuilder::new("aeof6")
            .token("a", "a")
            .rule("items", vec!["a"])
            .rule("items", vec!["a", "items"])
            .start("items"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_nested_chain() {
    let t = build_table(
        GrammarBuilder::new("aeof7")
            .token("x", "x")
            .rule("d", vec!["x"])
            .rule("c", vec!["d"])
            .rule("b", vec!["c"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn accept_on_eof_expr_grammar() {
    let t = build_table(
        GrammarBuilder::new("aeof8")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .start("expr"),
    );
    assert!(accept_on_eof(&t));
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. Full pipeline with serialization roundtrip (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn roundtrip_preserves_state_count() {
    let t = build_table(
        GrammarBuilder::new("rt1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.state_count, restored.state_count);
}

#[test]
fn roundtrip_preserves_symbol_count() {
    let t = build_table(
        GrammarBuilder::new("rt2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.symbol_count, restored.symbol_count);
}

#[test]
fn roundtrip_preserves_eof_symbol() {
    let t = build_table(
        GrammarBuilder::new("rt3")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.eof_symbol, restored.eof_symbol);
}

#[test]
fn roundtrip_preserves_start_symbol() {
    let t = build_table(
        GrammarBuilder::new("rt4")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.start_symbol, restored.start_symbol);
}

#[test]
fn roundtrip_preserves_action_table_shape() {
    let t = build_table(
        GrammarBuilder::new("rt5")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.action_table.len(), restored.action_table.len());
    for (orig_row, rest_row) in t.action_table.iter().zip(restored.action_table.iter()) {
        assert_eq!(orig_row.len(), rest_row.len());
    }
}

#[test]
fn roundtrip_preserves_goto_table_shape() {
    let t = build_table(
        GrammarBuilder::new("rt6")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.goto_table.len(), restored.goto_table.len());
}

#[test]
fn roundtrip_preserves_accept_action() {
    let t = build_table(
        GrammarBuilder::new("rt7")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert!(has_accept(&restored));
    assert!(accept_on_eof(&restored));
}

#[test]
fn roundtrip_preserves_rules() {
    let t = build_table(
        GrammarBuilder::new("rt8")
            .token("a", "a")
            .token("b", "b")
            .rule("stmt", vec!["a", "b"])
            .rule("start", vec!["stmt"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.rules.len(), restored.rules.len());
    for (orig, rest) in t.rules.iter().zip(restored.rules.iter()) {
        assert_eq!(orig.lhs, rest.lhs);
        assert_eq!(orig.rhs_len, rest.rhs_len);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. Parse table actions are consistent (shifts target valid states) (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn shifts_valid_single_token() {
    let t = build_table(
        GrammarBuilder::new("sv1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn shifts_valid_sequence() {
    let t = build_table(
        GrammarBuilder::new("sv2")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn shifts_valid_alternatives() {
    let t = build_table(
        GrammarBuilder::new("sv3")
            .token("x", "x")
            .token("y", "y")
            .token("z", "z")
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .rule("start", vec!["z"])
            .start("start"),
    );
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn shifts_valid_left_recursive() {
    let t = build_table(
        GrammarBuilder::new("sv4")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list"),
    );
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn shifts_valid_nested() {
    let t = build_table(
        GrammarBuilder::new("sv5")
            .token("x", "x")
            .rule("d", vec!["x"])
            .rule("c", vec!["d"])
            .rule("start", vec!["c"])
            .start("start"),
    );
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn shifts_valid_expr_grammar() {
    let t = build_table(
        GrammarBuilder::new("sv6")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr"),
    );
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn shifts_valid_after_roundtrip() {
    let t = build_table(
        GrammarBuilder::new("sv7")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    let restored = roundtrip(&t);
    assert!(shifts_target_valid_states(&restored));
}

#[test]
fn shifts_valid_mixed_terminals_nonterminals() {
    let t = build_table(
        GrammarBuilder::new("sv8")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "b"])
            .start("start"),
    );
    assert!(shifts_target_valid_states(&t));
}

// ═════════════════════════════════════════════════════════════════════════════
// 5. Complex grammars produce expected table structure (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn complex_add_grammar_structure() {
    let t = build_table(
        GrammarBuilder::new("cx1")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .start("expr"),
    );
    assert!(t.state_count >= 4);
    assert!(has_shift(&t));
    assert!(has_reduce(&t));
    assert!(has_accept(&t));
}

#[test]
fn complex_add_mul_grammar_structure() {
    let t = build_table(
        GrammarBuilder::new("cx2")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr"),
    );
    assert!(t.state_count >= 6);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn complex_grammar_with_parens() {
    let t = build_table(
        GrammarBuilder::new("cx3")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .rule("term", vec!["lp", "expr", "rp"])
            .start("expr"),
    );
    assert!(t.state_count >= 6);
    assert!(has_shift(&t));
    assert!(has_accept(&t));
}

#[test]
fn complex_multiple_nonterminals() {
    let t = build_table(
        GrammarBuilder::new("cx4")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("first", vec!["a"])
            .rule("second", vec!["b"])
            .rule("third", vec!["c"])
            .rule("start", vec!["first", "second", "third"])
            .start("start"),
    );
    assert!(t.state_count >= 4);
    assert!(!t.rules.is_empty());
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn complex_action_table_dimensions() {
    let t = build_table(
        GrammarBuilder::new("cx5")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert_eq!(t.action_table.len(), t.state_count);
    for row in &t.action_table {
        assert!(!row.is_empty());
    }
}

#[test]
fn complex_goto_table_dimensions() {
    let t = build_table(
        GrammarBuilder::new("cx6")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start"),
    );
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn complex_symbol_index_consistency() {
    let t = build_table(
        GrammarBuilder::new("cx7")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    for (&sym, &idx) in &t.symbol_to_index {
        assert!(idx < t.index_to_symbol.len());
        assert_eq!(t.index_to_symbol[idx], sym);
    }
}

#[test]
fn complex_eof_in_symbol_index() {
    let t = build_table(
        GrammarBuilder::new("cx8")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "EOF symbol must be in symbol_to_index"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// 6. Pipeline determinism: same grammar → same table (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

fn build_determinism_pair(f: impl Fn() -> GrammarBuilder) -> (ParseTable, ParseTable) {
    (build_table(f()), build_table(f()))
}

#[test]
fn determinism_single_token_state_count() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
    });
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn determinism_single_token_symbol_count() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det2")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
    });
    assert_eq!(t1.symbol_count, t2.symbol_count);
}

#[test]
fn determinism_alternatives_state_count() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det3")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn determinism_sequence_action_table_len() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det4")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
    });
    assert_eq!(t1.action_table.len(), t2.action_table.len());
}

#[test]
fn determinism_recursive_state_count() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det5")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list")
    });
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn determinism_nested_state_count() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det6")
            .token("x", "x")
            .rule("d", vec!["x"])
            .rule("c", vec!["d"])
            .rule("start", vec!["c"])
            .start("start")
    });
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn determinism_expr_grammar_state_count() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det7")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .start("expr")
    });
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.symbol_count, t2.symbol_count);
}

#[test]
fn determinism_serialized_bytes_identical() {
    let (t1, t2) = build_determinism_pair(|| {
        GrammarBuilder::new("det8")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
    });
    let b1 = t1.to_bytes().expect("serialize t1");
    let b2 = t2.to_bytes().expect("serialize t2");
    assert_eq!(b1, b2, "identical grammars must produce identical bytes");
}

// ═════════════════════════════════════════════════════════════════════════════
// 7. Pipeline with different grammar sizes (scaling) (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

fn build_n_token_sequence(n: usize) -> ParseTable {
    let mut b = GrammarBuilder::new("scale");
    let mut syms = Vec::new();
    for i in 0..n {
        let name = format!("t{i}");
        let pat = format!("t{i}");
        b = b.token(&name, &pat);
        syms.push(name);
    }
    let refs: Vec<&str> = syms.iter().map(|s| s.as_str()).collect();
    b = b.rule("start", refs).start("start");
    build_table(b)
}

fn build_n_alternatives(n: usize) -> ParseTable {
    let mut b = GrammarBuilder::new("alt_scale");
    for i in 0..n {
        let name = format!("t{i}");
        let pat = format!("t{i}");
        b = b.token(&name, &pat);
        b = b.rule("start", vec![&name]);
    }
    b = b.start("start");
    build_table(b)
}

#[test]
fn scaling_two_token_sequence() {
    let t = build_n_token_sequence(2);
    assert!(t.state_count >= 3);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn scaling_five_token_sequence() {
    let t = build_n_token_sequence(5);
    assert!(t.state_count >= 6);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn scaling_ten_token_sequence() {
    let t = build_n_token_sequence(10);
    assert!(t.state_count >= 11);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn scaling_longer_sequence_more_states() {
    let t5 = build_n_token_sequence(5);
    let t10 = build_n_token_sequence(10);
    assert!(t10.state_count > t5.state_count);
}

#[test]
fn scaling_two_alternatives() {
    let t = build_n_alternatives(2);
    assert!(t.state_count >= 2);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn scaling_five_alternatives() {
    let t = build_n_alternatives(5);
    assert!(t.state_count >= 2);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn scaling_ten_alternatives() {
    let t = build_n_alternatives(10);
    assert!(t.state_count >= 2);
    assert!(has_accept(&t));
}

#[test]
fn scaling_roundtrip_large_table() {
    let t = build_n_token_sequence(10);
    let restored = roundtrip(&t);
    assert_eq!(t.state_count, restored.state_count);
    assert_eq!(t.symbol_count, restored.symbol_count);
    assert!(has_accept(&restored));
}

// ═════════════════════════════════════════════════════════════════════════════
// 8. Edge cases: single rule, many rules, ambiguous grammars (8 tests)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn edge_single_rule_single_token() {
    let t = build_table(
        GrammarBuilder::new("e1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(t.state_count >= 2);
    assert!(has_accept(&t));
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn edge_many_rules_same_nonterminal() {
    let mut b = GrammarBuilder::new("e2");
    for i in 0..8 {
        let name = format!("t{i}");
        let pat = format!("t{i}");
        b = b.token(&name, &pat);
        b = b.rule("start", vec![&name]);
    }
    b = b.start("start");
    let t = build_table(b);
    assert!(has_accept(&t));
    assert!(has_reduce(&t));
}

#[test]
fn edge_ambiguous_expr_builds() {
    // E → E + E | num  — classic ambiguous grammar
    let t = build_table(
        GrammarBuilder::new("e3")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(t.state_count >= 4);
    assert!(has_accept(&t));
}

#[test]
fn edge_ambiguous_expr_has_eof_accept() {
    let t = build_table(
        GrammarBuilder::new("e4")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(accept_on_eof(&t));
}

#[test]
fn edge_ambiguous_roundtrip() {
    let t = build_table(
        GrammarBuilder::new("e5")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    let restored = roundtrip(&t);
    assert_eq!(t.state_count, restored.state_count);
    assert!(has_accept(&restored));
}

#[test]
fn edge_long_rhs_rule() {
    let t = build_table(
        GrammarBuilder::new("e6")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .token("e", "e")
            .rule("start", vec!["a", "b", "c", "d", "e"])
            .start("start"),
    );
    assert!(t.state_count >= 6);
    sanity_check_tables(&t).expect("sanity");
}

#[test]
fn edge_multiple_nonterminal_chains() {
    let t = build_table(
        GrammarBuilder::new("e7")
            .token("x", "x")
            .token("y", "y")
            .rule("a", vec!["x"])
            .rule("b", vec!["y"])
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    assert!(t.state_count >= 3);
    assert!(has_accept(&t));
    assert!(shifts_target_valid_states(&t));
}

#[test]
fn edge_self_referential_with_base_case() {
    // S → a S | a  — right recursive with shared prefix
    let t = build_table(
        GrammarBuilder::new("e8")
            .token("a", "a")
            .rule("start", vec!["a", "start"])
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(t.state_count >= 2);
    assert!(has_accept(&t));
    assert!(shifts_target_valid_states(&t));
}
