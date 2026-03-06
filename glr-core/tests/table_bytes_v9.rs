//! ParseTable byte serialization comprehensive tests (v9).
//!
//! 80+ tests for `to_bytes()` / `from_bytes()` round-trip correctness.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test table_bytes_v9 --features serialization -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::{Action, FirstFollowSets, ParseTable, StateId, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn roundtrip(pt: &ParseTable) -> ParseTable {
    let bytes = pt.to_bytes().expect("serialize");
    ParseTable::from_bytes(&bytes).expect("deserialize")
}

// ---------------------------------------------------------------------------
// Grammar factories — each name prefixed tb_v9_
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_minimal")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn pair_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_pair")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build()
}

fn alt_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_chain")
        .token("x", "x")
        .rule("s", vec!["m"])
        .rule("m", vec!["x"])
        .start("s")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_arith")
        .token("n", r"\d+")
        .token("+", r"\+")
        .token("*", r"\*")
        .token("(", r"\(")
        .token(")", r"\)")
        .rule("e", vec!["e", "+", "t"])
        .rule("e", vec!["t"])
        .rule("t", vec!["t", "*", "f"])
        .rule("t", vec!["f"])
        .rule("f", vec!["n"])
        .rule("f", vec!["(", "e", ")"])
        .start("e")
        .build()
}

fn prec_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_prec")
        .token("n", r"\d+")
        .token("+", r"\+")
        .token("*", r"\*")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build()
}

fn multi_alt_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_multi_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["e"])
        .start("s")
        .build()
}

fn left_rec_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_left_rec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn right_rec_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_right_rec")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn nested_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_nested")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "s", "b"])
        .rule("s", vec![])
        .start("s")
        .build()
}

fn nullable_prefix_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_nullable_pre")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["opt", "b"])
        .rule("opt", vec!["a"])
        .rule("opt", vec![])
        .start("s")
        .build()
}

fn wide_rhs_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_wide_rhs")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("s", vec!["a", "b", "c", "d", "e", "f"])
        .start("s")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_deep_chain")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("s")
        .build()
}

fn stmt_list_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_stmt_list")
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

fn two_nt_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["l", "r"])
        .rule("l", vec!["a"])
        .rule("r", vec!["b"])
        .start("s")
        .build()
}

fn ambig_grammar() -> Grammar {
    GrammarBuilder::new("tb_v9_ambig")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("e", vec!["e", "+", "e"])
        .rule("e", vec!["n"])
        .start("e")
        .build()
}

fn large_grammar() -> Grammar {
    let mut b = GrammarBuilder::new("tb_v9_large");
    for i in 0..15 {
        let tok = format!("t{i}");
        b = b.token(&tok, &tok);
    }
    for i in 0..15 {
        let nt = format!("r{i}");
        let tok = format!("t{i}");
        b = b.rule(&nt, vec![&tok]);
    }
    for i in 0..15 {
        let nt = format!("r{i}");
        b = b.rule("s", vec![&nt]);
    }
    b.start("s").build()
}

// ===================================================================
// 1. to_bytes → non-empty
// ===================================================================

#[test]
fn tb_v9_to_bytes_nonempty_minimal() {
    let pt = make_table("tb_v9_ne1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

#[test]
fn tb_v9_to_bytes_nonempty_pair() {
    let g = pair_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

#[test]
fn tb_v9_to_bytes_nonempty_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

#[test]
fn tb_v9_to_bytes_nonempty_default() {
    assert!(!ParseTable::default().to_bytes().expect("s").is_empty());
}

// ===================================================================
// 2. from_bytes(to_bytes()) → Ok
// ===================================================================

#[test]
fn tb_v9_from_roundtrip_ok_minimal() {
    let pt = make_table("tb_v9_ok1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let bytes = pt.to_bytes().expect("s");
    assert!(ParseTable::from_bytes(&bytes).is_ok());
}

#[test]
fn tb_v9_from_roundtrip_ok_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    assert!(ParseTable::from_bytes(&bytes).is_ok());
}

#[test]
fn tb_v9_from_roundtrip_ok_alt() {
    let g = alt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    assert!(ParseTable::from_bytes(&bytes).is_ok());
}

#[test]
fn tb_v9_from_roundtrip_ok_default() {
    let bytes = ParseTable::default().to_bytes().expect("s");
    assert!(ParseTable::from_bytes(&bytes).is_ok());
}

// ===================================================================
// 3. Roundtrip preserves state_count
// ===================================================================

#[test]
fn tb_v9_rt_state_count_minimal() {
    let pt = make_table("tb_v9_sc1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.state_count, roundtrip(&pt).state_count);
}

#[test]
fn tb_v9_rt_state_count_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.state_count, roundtrip(&pt).state_count);
}

#[test]
fn tb_v9_rt_state_count_chain() {
    let g = chain_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.state_count, roundtrip(&pt).state_count);
}

#[test]
fn tb_v9_rt_state_count_nested() {
    let g = nested_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.state_count, roundtrip(&pt).state_count);
}

// ===================================================================
// 4. Roundtrip preserves symbol_count
// ===================================================================

#[test]
fn tb_v9_rt_symbol_count_minimal() {
    let pt = make_table("tb_v9_syc1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.symbol_count, roundtrip(&pt).symbol_count);
}

#[test]
fn tb_v9_rt_symbol_count_multi_alt() {
    let g = multi_alt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.symbol_count, roundtrip(&pt).symbol_count);
}

#[test]
fn tb_v9_rt_symbol_count_stmt_list() {
    let g = stmt_list_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.symbol_count, roundtrip(&pt).symbol_count);
}

#[test]
fn tb_v9_rt_symbol_count_wide_rhs() {
    let g = wide_rhs_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.symbol_count, roundtrip(&pt).symbol_count);
}

// ===================================================================
// 5. Roundtrip preserves eof_symbol
// ===================================================================

#[test]
fn tb_v9_rt_eof_minimal() {
    let pt = make_table("tb_v9_eof1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.eof_symbol, roundtrip(&pt).eof_symbol);
}

#[test]
fn tb_v9_rt_eof_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.eof_symbol, roundtrip(&pt).eof_symbol);
}

#[test]
fn tb_v9_rt_eof_via_accessor() {
    let pt = make_table("tb_v9_eof2", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_eq!(pt.eof(), roundtrip(&pt).eof());
}

#[test]
fn tb_v9_rt_eof_default() {
    let pt = ParseTable::default();
    assert_eq!(pt.eof_symbol, roundtrip(&pt).eof_symbol);
}

// ===================================================================
// 6. Roundtrip preserves all actions
// ===================================================================

#[test]
fn tb_v9_rt_actions_minimal() {
    let pt = make_table("tb_v9_act1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

#[test]
fn tb_v9_rt_actions_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

#[test]
fn tb_v9_rt_actions_per_state_symbol() {
    let g = multi_alt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    for st in 0..pt.state_count {
        let sid = StateId(st as u16);
        for &sym in pt.symbol_to_index.keys() {
            assert_eq!(
                pt.actions(sid, sym),
                restored.actions(sid, sym),
                "action mismatch state={st} sym={sym:?}"
            );
        }
    }
}

#[test]
fn tb_v9_rt_actions_has_shift() {
    let g = pair_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let has_shift = pt
        .action_table
        .iter()
        .flatten()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(has_shift);
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

#[test]
fn tb_v9_rt_actions_has_reduce() {
    let pt = make_table("tb_v9_actred", &[("a", "a")], &[("s", vec!["a"])], "s");
    let has_reduce = pt
        .action_table
        .iter()
        .flatten()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(has_reduce);
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

#[test]
fn tb_v9_rt_actions_has_accept() {
    let pt = make_table("tb_v9_actacc", &[("a", "a")], &[("s", vec!["a"])], "s");
    let has_accept = pt
        .action_table
        .iter()
        .flatten()
        .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)));
    assert!(has_accept);
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

// ===================================================================
// 7. Roundtrip preserves all goto entries
// ===================================================================

#[test]
fn tb_v9_rt_goto_two_nt() {
    let g = two_nt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.goto_table, roundtrip(&pt).goto_table);
}

#[test]
fn tb_v9_rt_goto_deep_chain() {
    let g = deep_chain_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.goto_table, roundtrip(&pt).goto_table);
}

#[test]
fn tb_v9_rt_goto_per_state_nt() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    for st in 0..pt.state_count {
        let sid = StateId(st as u16);
        for &nt in pt.nonterminal_to_index.keys() {
            assert_eq!(
                pt.goto(sid, nt),
                restored.goto(sid, nt),
                "goto mismatch state={st} nt={nt:?}"
            );
        }
    }
}

#[test]
fn tb_v9_rt_goto_nullable_prefix() {
    let g = nullable_prefix_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.goto_table, roundtrip(&pt).goto_table);
}

// ===================================================================
// 8. Roundtrip preserves all rules
// ===================================================================

#[test]
fn tb_v9_rt_rules_count_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
}

#[test]
fn tb_v9_rt_rules_lhs_rhs_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid), "rule {i} mismatch");
    }
}

#[test]
fn tb_v9_rt_rules_wide_rhs() {
    let g = wide_rhs_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn tb_v9_rt_rules_large() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

// ===================================================================
// 9. Deterministic: same table → same bytes
// ===================================================================

#[test]
fn tb_v9_det_same_table_twice() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let b1 = pt.to_bytes().expect("s1");
    let b2 = pt.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn tb_v9_det_rebuild_same_grammar() {
    let b1 = {
        let g = minimal_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let b2 = {
        let g = minimal_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert_eq!(b1, b2);
}

#[test]
fn tb_v9_det_pair_stable() {
    let b1 = {
        let g = pair_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let b2 = {
        let g = pair_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert_eq!(b1, b2);
}

#[test]
fn tb_v9_det_default_stable() {
    let b1 = ParseTable::default().to_bytes().expect("s1");
    let b2 = ParseTable::default().to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

// ===================================================================
// 10. Different tables → different bytes
// ===================================================================

#[test]
fn tb_v9_diff_minimal_vs_pair() {
    let b1 = {
        let g = minimal_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let b2 = {
        let g = pair_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert_ne!(b1, b2);
}

#[test]
fn tb_v9_diff_left_vs_right_rec() {
    let b1 = {
        let g = left_rec_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let b2 = {
        let g = right_rec_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert_ne!(b1, b2);
}

#[test]
fn tb_v9_diff_chain_vs_deep_chain() {
    let b1 = {
        let g = chain_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let b2 = {
        let g = deep_chain_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert_ne!(b1, b2);
}

#[test]
fn tb_v9_diff_arith_vs_alt() {
    let b1 = {
        let g = arith_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let b2 = {
        let g = alt_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert_ne!(b1, b2);
}

// ===================================================================
// 11. Bytes length > 0
// ===================================================================

#[test]
fn tb_v9_len_gt_zero_chain() {
    let g = chain_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

#[test]
fn tb_v9_len_gt_zero_large() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

#[test]
fn tb_v9_len_gt_zero_prec() {
    let g = prec_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

#[test]
fn tb_v9_len_gt_zero_nested() {
    let g = nested_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert!(!pt.to_bytes().expect("s").is_empty());
}

// ===================================================================
// 12. from_bytes on empty → Err
// ===================================================================

#[test]
fn tb_v9_empty_bytes_err() {
    assert!(ParseTable::from_bytes(&[]).is_err());
}

#[test]
fn tb_v9_empty_slice_err() {
    let empty: &[u8] = &[];
    assert!(ParseTable::from_bytes(empty).is_err());
}

// ===================================================================
// 13. from_bytes on garbage → Err
// ===================================================================

#[test]
fn tb_v9_garbage_random_err() {
    let garbage: Vec<u8> = (0u8..200)
        .map(|i| i.wrapping_mul(73).wrapping_add(19))
        .collect();
    assert!(ParseTable::from_bytes(&garbage).is_err());
}

#[test]
fn tb_v9_garbage_all_zeros_err() {
    assert!(ParseTable::from_bytes(&[0u8; 128]).is_err());
}

#[test]
fn tb_v9_garbage_all_ff_err() {
    assert!(ParseTable::from_bytes(&[0xFF; 128]).is_err());
}

#[test]
fn tb_v9_garbage_single_byte_err() {
    assert!(ParseTable::from_bytes(&[0x42]).is_err());
}

// ===================================================================
// 14. from_bytes on truncated → Err
// ===================================================================

#[test]
fn tb_v9_truncated_half() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    let half = bytes.len() / 2;
    assert!(ParseTable::from_bytes(&bytes[..half]).is_err());
}

#[test]
fn tb_v9_truncated_one_byte_short() {
    let g = pair_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    assert!(ParseTable::from_bytes(&bytes[..bytes.len() - 1]).is_err());
}

#[test]
fn tb_v9_truncated_quarter() {
    let g = stmt_list_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    let quarter = bytes.len() / 4;
    assert!(ParseTable::from_bytes(&bytes[..quarter]).is_err());
}

#[test]
fn tb_v9_truncated_first_byte_only() {
    let g = minimal_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    assert!(ParseTable::from_bytes(&bytes[..1]).is_err());
}

// ===================================================================
// 15. Minimal grammar roundtrip
// ===================================================================

#[test]
fn tb_v9_minimal_rt_all_fields() {
    let pt = make_table("tb_v9_min_rt", &[("a", "a")], &[("s", vec!["a"])], "s");
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.eof_symbol, restored.eof_symbol);
    assert_eq!(pt.start_symbol, restored.start_symbol);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn tb_v9_minimal_rt_rules() {
    let pt = make_table("tb_v9_min_rul", &[("a", "a")], &[("s", vec!["a"])], "s");
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn tb_v9_minimal_rt_symbol_maps() {
    let pt = make_table("tb_v9_min_map", &[("a", "a")], &[("s", vec!["a"])], "s");
    let restored = roundtrip(&pt);
    assert_eq!(pt.symbol_to_index, restored.symbol_to_index);
    assert_eq!(pt.index_to_symbol, restored.index_to_symbol);
    assert_eq!(pt.nonterminal_to_index, restored.nonterminal_to_index);
}

// ===================================================================
// 16. Arithmetic grammar roundtrip
// ===================================================================

#[test]
fn tb_v9_arith_rt_all() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn tb_v9_arith_rt_rules_detail() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    for i in 0..pt.rules.len() {
        let rid = RuleId(i as u16);
        assert_eq!(pt.rule(rid), restored.rule(rid));
    }
}

#[test]
fn tb_v9_arith_rt_extras_and_lex() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.extras, restored.extras);
    assert_eq!(pt.lex_modes.len(), restored.lex_modes.len());
}

// ===================================================================
// 17. Grammar with precedence roundtrip
// ===================================================================

#[test]
fn tb_v9_prec_rt_all() {
    let g = prec_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn tb_v9_prec_rt_dynamic_prec() {
    let g = prec_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.dynamic_prec_by_rule, restored.dynamic_prec_by_rule);
}

#[test]
fn tb_v9_prec_rt_rule_assoc() {
    let g = prec_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.rule_assoc_by_rule, restored.rule_assoc_by_rule);
}

// ===================================================================
// 18. Grammar with alternatives roundtrip
// ===================================================================

#[test]
fn tb_v9_alt_rt_actions_match() {
    let g = alt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

#[test]
fn tb_v9_multi_alt_rt_all() {
    let g = multi_alt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn tb_v9_ambig_rt_multi_action_cells() {
    let g = ambig_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    assert_eq!(pt.action_table, roundtrip(&pt).action_table);
}

// ===================================================================
// 19. Large grammar roundtrip
// ===================================================================

#[test]
fn tb_v9_large_rt_state_symbol() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
}

#[test]
fn tb_v9_large_rt_all_tables() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn tb_v9_large_rt_rules() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.rules.len(), restored.rules.len());
}

#[test]
fn tb_v9_large_rt_under_500kb() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let bytes = pt.to_bytes().expect("s");
    assert!(
        bytes.len() < 500_000,
        "serialized size {} exceeds 500KB",
        bytes.len()
    );
}

// ===================================================================
// 20. Double roundtrip (serialize → deserialize → serialize → same bytes)
// ===================================================================

#[test]
fn tb_v9_double_rt_minimal() {
    let pt = make_table("tb_v9_drt1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let b1 = pt.to_bytes().expect("s1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn tb_v9_double_rt_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let b1 = pt.to_bytes().expect("s1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn tb_v9_double_rt_prec() {
    let g = prec_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let b1 = pt.to_bytes().expect("s1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

#[test]
fn tb_v9_triple_rt_large() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let b1 = pt.to_bytes().expect("s1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("s2");
    let t3 = ParseTable::from_bytes(&b2).expect("d2");
    let b3 = t3.to_bytes().expect("s3");
    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

// ===================================================================
// Bonus: additional coverage
// ===================================================================

#[test]
fn tb_v9_bonus_flipped_bits_no_panic() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let mut bytes = pt.to_bytes().expect("s");
    if bytes.len() > 10 {
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        bytes[mid + 1] ^= 0xAA;
    }
    // Must not panic
    let _ = ParseTable::from_bytes(&bytes);
}

#[test]
fn tb_v9_bonus_field_names_preserved() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.field_names, restored.field_names);
    assert_eq!(pt.field_map, restored.field_map);
}

#[test]
fn tb_v9_bonus_alias_sequences_preserved() {
    let g = stmt_list_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.alias_sequences, restored.alias_sequences);
}

#[test]
fn tb_v9_bonus_initial_state_preserved() {
    let g = nested_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.initial_state, restored.initial_state);
}

#[test]
fn tb_v9_bonus_token_count_preserved() {
    let g = wide_rhs_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.token_count, restored.token_count);
}

#[test]
fn tb_v9_bonus_external_token_count_preserved() {
    let g = minimal_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.external_token_count, restored.external_token_count);
}

#[test]
fn tb_v9_bonus_symbol_metadata_preserved() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.symbol_metadata.len(), restored.symbol_metadata.len());
}

#[test]
fn tb_v9_bonus_more_states_more_bytes() {
    let small = {
        let g = minimal_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    let big = {
        let g = arith_grammar();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        build_lr1_automaton(&g, &ff)
            .expect("t")
            .to_bytes()
            .expect("s")
    };
    assert!(
        big.len() > small.len(),
        "arith ({}) should be larger than minimal ({})",
        big.len(),
        small.len()
    );
}

#[test]
fn tb_v9_bonus_goto_indexing_preserved() {
    let g = two_nt_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.goto_indexing, restored.goto_indexing);
}

#[test]
fn tb_v9_bonus_external_scanner_states_preserved() {
    let g = chain_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let restored = roundtrip(&pt);
    assert_eq!(pt.external_scanner_states, restored.external_scanner_states);
}

#[test]
fn tb_v9_bonus_make_table_helper_roundtrip() {
    let pt = make_table(
        "tb_v9_helper",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x", "y"])],
        "s",
    );
    let restored = roundtrip(&pt);
    assert_eq!(pt.state_count, restored.state_count);
    assert_eq!(pt.symbol_count, restored.symbol_count);
    assert_eq!(pt.action_table, restored.action_table);
    assert_eq!(pt.goto_table, restored.goto_table);
}

#[test]
fn tb_v9_bonus_performance_under_5s() {
    let g = large_grammar();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("t");
    let start = std::time::Instant::now();
    let bytes = pt.to_bytes().expect("s");
    let _ = ParseTable::from_bytes(&bytes).expect("d");
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 5000, "roundtrip took {elapsed:?}");
}
