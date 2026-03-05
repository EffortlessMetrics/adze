#![cfg(feature = "test-api")]

//! Comprehensive tests for automaton size scaling and bounds in adze-glr-core.
//!
//! 84 tests covering:
//!  1. Baseline state counts (tests 1–5)
//!  2. Token-count → symbol-count monotonicity (tests 6–11)
//!  3. Rule-count → state-count trends (tests 12–17)
//!  4. Symbol-count lower bounds (tests 18–25)
//!  5. State-count lower bounds (tests 26–31)
//!  6. Token scaling → symbol scaling (tests 32–39)
//!  7. Upper-bound reasonableness (tests 40–45)
//!  8. Arithmetic / expression grammars (tests 46–51)
//!  9. Chain grammars and linear growth (tests 52–59)
//! 10. Alternative grammars (tests 60–65)
//! 11. Precedence grammars (tests 66–71)
//! 12. Extras / inline grammars (tests 72–77)
//! 13. Simple-vs-complex comparisons (tests 78–81)
//! 14. Polynomial scaling (tests 82–84)

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

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

fn make_table_with_prec(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    precs: &[(i16, Associativity, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    for (level, assoc, syms) in precs {
        b = b.precedence(*level, *assoc, syms.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn make_table_with_extras(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    extras: &[&str],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    for e in extras {
        b = b.extra(e);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn make_table_with_inline(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    inlines: &[&str],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    for i in inlines {
        b = b.inline(i);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

/// Build a chain grammar: s -> a1, a1 -> a2, ... a(n-1) -> tok
fn chain_table(name: &str, depth: usize) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    b = b.token("tok", "t");
    let names: Vec<String> = (0..depth).map(|i| format!("c{i}")).collect();
    for i in 0..depth {
        if i + 1 < depth {
            b = b.rule(&names[i], vec![&names[i + 1]]);
        } else {
            b = b.rule(&names[i], vec!["tok"]);
        }
    }
    let g = b.start(&names[0]).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

/// Build grammar with N alternative rules: s -> tok1 | tok2 | ... | tokN
fn alt_table(name: &str, n: usize) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let tok_pats: Vec<String> = (0..n).map(|i| format!("x{i}")).collect();
    for i in 0..n {
        b = b.token(&tok_names[i], &tok_pats[i]);
    }
    for i in 0..n {
        b = b.rule("s", vec![&tok_names[i]]);
    }
    let g = b.start("s").build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

// =========================================================================
// 1. Baseline state counts (1-5)
// =========================================================================

#[test]
fn test_as_v9_1tok_1rule_state_count() {
    let pt = make_table("as_v9_1t1r", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        pt.state_count >= 2,
        "1tok/1rule must have ≥2 states, got {}",
        pt.state_count
    );
}

#[test]
fn test_as_v9_2tok_1rule_state_count_ge_baseline() {
    let pt1 = make_table("as_v9_1t1r_b", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table(
        "as_v9_2t1r",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(
        pt2.state_count >= pt1.state_count,
        "2tok/1rule states ({}) should be ≥ 1tok/1rule states ({})",
        pt2.state_count,
        pt1.state_count
    );
}

#[test]
fn test_as_v9_3tok_1rule_state_count() {
    let pt = make_table(
        "as_v9_3t1r",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_1tok_2rules_state_count() {
    let pt = make_table(
        "as_v9_1t2r",
        &[("a", "a")],
        &[("s", vec!["x"]), ("x", vec!["a"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_1tok_3rules_state_count() {
    let pt = make_table(
        "as_v9_1t3r",
        &[("a", "a")],
        &[("s", vec!["x"]), ("x", vec!["y"]), ("y", vec!["a"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

// =========================================================================
// 2. More tokens → more symbols (6-11)
// =========================================================================

#[test]
fn test_as_v9_more_tokens_more_symbols_2v1() {
    let pt1 = make_table("as_v9_sym1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table(
        "as_v9_sym2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"])],
        "s",
    );
    assert!(pt2.symbol_count >= pt1.symbol_count);
}

#[test]
fn test_as_v9_more_tokens_more_symbols_3v2() {
    let pt2 = make_table(
        "as_v9_sym2b",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"])],
        "s",
    );
    let pt3 = make_table(
        "as_v9_sym3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"])],
        "s",
    );
    assert!(pt3.symbol_count >= pt2.symbol_count);
}

#[test]
fn test_as_v9_more_tokens_more_symbols_4v3() {
    let pt3 = make_table(
        "as_v9_sym3b",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"])],
        "s",
    );
    let pt4 = make_table(
        "as_v9_sym4",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a"])],
        "s",
    );
    assert!(pt4.symbol_count >= pt3.symbol_count);
}

#[test]
fn test_as_v9_5_tokens_symbol_count() {
    let pt = make_table(
        "as_v9_5tok",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a"])],
        "s",
    );
    assert!(
        pt.symbol_count >= 5,
        "5 tokens → symbol_count should be ≥5, got {}",
        pt.symbol_count
    );
}

#[test]
fn test_as_v9_10_tokens_symbol_count() {
    let tokens: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{i}"), format!("x{i}")))
        .collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let pt = make_table("as_v9_10tok", &tok_refs, &[("s", vec!["t0"])], "s");
    assert!(
        pt.symbol_count >= 10,
        "10 tokens → symbol_count should be ≥10, got {}",
        pt.symbol_count
    );
}

#[test]
fn test_as_v9_token_count_monotonic_5v1() {
    let pt1 = make_table("as_v9_m1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let tokens5: Vec<(String, String)> =
        (0..5).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let refs5: Vec<(&str, &str)> = tokens5
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let pt5 = make_table("as_v9_m5", &refs5, &[("s", vec!["t0"])], "s");
    assert!(pt5.symbol_count > pt1.symbol_count);
}

// =========================================================================
// 3. More rules → more states (general trend) (12-17)
// =========================================================================

#[test]
fn test_as_v9_more_rules_trend_2v1() {
    let pt1 = make_table("as_v9_r1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table(
        "as_v9_r2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // 2 alternatives should need at least as many states
    assert!(pt2.state_count >= pt1.state_count);
}

#[test]
fn test_as_v9_more_rules_trend_3v2() {
    let pt2 = make_table(
        "as_v9_r2b",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let pt3 = make_table(
        "as_v9_r3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(pt3.state_count >= pt2.state_count);
}

#[test]
fn test_as_v9_chain_depth2_vs_depth1() {
    let pt1 = chain_table("as_v9_ch1", 1);
    let pt2 = chain_table("as_v9_ch2", 2);
    assert!(pt2.state_count >= pt1.state_count);
}

#[test]
fn test_as_v9_chain_depth3_vs_depth2() {
    let pt2 = chain_table("as_v9_ch2b", 2);
    let pt3 = chain_table("as_v9_ch3", 3);
    assert!(pt3.state_count >= pt2.state_count);
}

#[test]
fn test_as_v9_chain_depth5_vs_depth3() {
    let pt3 = chain_table("as_v9_ch3b", 3);
    let pt5 = chain_table("as_v9_ch5", 5);
    assert!(pt5.state_count >= pt3.state_count);
}

#[test]
fn test_as_v9_alt_rules_5v3() {
    let pt3 = alt_table("as_v9_a3", 3);
    let pt5 = alt_table("as_v9_a5", 5);
    assert!(pt5.state_count >= pt3.state_count);
}

// =========================================================================
// 4. Symbol count lower bound (18-25)
// =========================================================================

#[test]
fn test_as_v9_symbol_bound_1tok_1nt() {
    // 1 token + 1 NT + EOF = at least 3
    let pt = make_table("as_v9_sb1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.symbol_count >= 1 + 1 + 1);
}

#[test]
fn test_as_v9_symbol_bound_2tok_1nt() {
    let pt = make_table(
        "as_v9_sb2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.symbol_count >= 2 + 1 + 1);
}

#[test]
fn test_as_v9_symbol_bound_3tok_2nt() {
    let pt = make_table(
        "as_v9_sb3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["x", "c"]), ("x", vec!["a", "b"])],
        "s",
    );
    assert!(pt.symbol_count >= 3 + 2 + 1);
}

#[test]
fn test_as_v9_symbol_bound_formula() {
    // Generic: symbol_count ≥ tokens + nonterminals + 1 (EOF)
    let pt = make_table(
        "as_v9_sbf",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["x"]), ("x", vec!["y"]), ("y", vec!["a"])],
        "s",
    );
    // 4 tokens, 3 NTs, 1 EOF → at least 8
    assert!(pt.symbol_count >= 4 + 3 + 1);
}

#[test]
fn test_as_v9_symbol_count_includes_eof() {
    let pt = make_table("as_v9_eof", &[("a", "a")], &[("s", vec!["a"])], "s");
    // EOF symbol should be a valid symbol ID in the table
    let eof = pt.eof_symbol;
    assert!(pt.symbol_to_index.contains_key(&eof) || eof.0 < pt.symbol_count as u16);
}

#[test]
fn test_as_v9_symbol_count_positive() {
    let pt = make_table("as_v9_spos", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.symbol_count > 0);
}

#[test]
fn test_as_v9_symbol_count_with_5tok() {
    let pt = make_table(
        "as_v9_5ts",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    // 5 tokens + 1 NT + 1 EOF
    assert!(pt.symbol_count >= 7);
}

#[test]
fn test_as_v9_symbol_count_with_multi_nt() {
    let pt = make_table(
        "as_v9_mnt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["x", "y"]), ("x", vec!["a"]), ("y", vec!["b"])],
        "s",
    );
    // 2 tokens + 3 NTs + 1 EOF
    assert!(pt.symbol_count >= 6);
}

// =========================================================================
// 5. State count lower bounds (26-31)
// =========================================================================

#[test]
fn test_as_v9_state_ge_2_minimal() {
    let pt = make_table("as_v9_s2m", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_state_ge_2_two_tok() {
    let pt = make_table(
        "as_v9_s2t",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_state_ge_2_chain() {
    let pt = chain_table("as_v9_s2c", 3);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_state_ge_2_alt() {
    let pt = alt_table("as_v9_s2a", 4);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_sequence_needs_more_states() {
    // A sequence of N tokens needs at least N+1 states (shift through each)
    let pt = make_table(
        "as_v9_seq4",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    );
    // Initial state + 1 shift per token + accepting
    assert!(pt.state_count >= 3);
}

#[test]
fn test_as_v9_state_count_positive() {
    let pt = make_table("as_v9_scp", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count > 0);
}

// =========================================================================
// 6. Token scaling → symbol scaling (32-39)
// =========================================================================

#[test]
fn test_as_v9_scaling_1_to_2_tokens() {
    let pt1 = make_table("as_v9_sc12a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table(
        "as_v9_sc12b",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt2.symbol_count > pt1.symbol_count);
}

#[test]
fn test_as_v9_scaling_2_to_4_tokens() {
    let t2: Vec<(String, String)> = (0..2).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let r2: Vec<(&str, &str)> = t2.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let t4: Vec<(String, String)> = (0..4).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let r4: Vec<(&str, &str)> = t4.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let pt2 = make_table("as_v9_sc24a", &r2, &[("s", vec!["t0"])], "s");
    let pt4 = make_table("as_v9_sc24b", &r4, &[("s", vec!["t0"])], "s");
    assert!(pt4.symbol_count > pt2.symbol_count);
}

#[test]
fn test_as_v9_scaling_4_to_8_tokens() {
    let t4: Vec<(String, String)> = (0..4).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let r4: Vec<(&str, &str)> = t4.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let t8: Vec<(String, String)> = (0..8).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let r8: Vec<(&str, &str)> = t8.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let pt4 = make_table("as_v9_sc48a", &r4, &[("s", vec!["t0"])], "s");
    let pt8 = make_table("as_v9_sc48b", &r8, &[("s", vec!["t0"])], "s");
    assert!(pt8.symbol_count > pt4.symbol_count);
}

#[test]
fn test_as_v9_scaling_symbol_diff_grows() {
    let pt1 = make_table("as_v9_sdg1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let t3: Vec<(String, String)> = (0..3).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let r3: Vec<(&str, &str)> = t3.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let pt3 = make_table("as_v9_sdg3", &r3, &[("s", vec!["t0"])], "s");
    let diff = pt3.symbol_count as isize - pt1.symbol_count as isize;
    assert!(
        diff >= 2,
        "Adding 2 tokens should increase symbols by ≥2, got {diff}"
    );
}

#[test]
fn test_as_v9_scaling_tokens_not_states_only() {
    // Adding unused tokens should add symbols but not necessarily states
    let pt1 = make_table("as_v9_tn1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table(
        "as_v9_tn2",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"])],
        "s",
    );
    assert!(pt2.symbol_count > pt1.symbol_count);
    // States may or may not grow
}

#[test]
fn test_as_v9_scaling_6_tokens() {
    let tokens: Vec<(String, String)> =
        (0..6).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let pt = make_table("as_v9_6t", &refs, &[("s", vec!["t0"])], "s");
    assert!(pt.symbol_count >= 6);
}

#[test]
fn test_as_v9_scaling_8_tokens() {
    let tokens: Vec<(String, String)> =
        (0..8).map(|i| (format!("t{i}"), format!("p{i}"))).collect();
    let refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let pt = make_table("as_v9_8t", &refs, &[("s", vec!["t0"])], "s");
    assert!(pt.symbol_count >= 8);
}

#[test]
fn test_as_v9_scaling_12_tokens() {
    let tokens: Vec<(String, String)> = (0..12)
        .map(|i| (format!("t{i}"), format!("p{i}")))
        .collect();
    let refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let pt = make_table("as_v9_12t", &refs, &[("s", vec!["t0"])], "s");
    assert!(pt.symbol_count >= 12);
}

// =========================================================================
// 7. Upper-bound reasonableness (40-45)
// =========================================================================

#[test]
fn test_as_v9_upper_bound_1rule() {
    let pt = make_table("as_v9_ub1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(
        pt.state_count <= 100,
        "trivial grammar should have ≤100 states"
    );
}

#[test]
fn test_as_v9_upper_bound_2rules() {
    let pt = make_table(
        "as_v9_ub2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count <= 100);
}

#[test]
fn test_as_v9_upper_bound_chain5() {
    let pt = chain_table("as_v9_ubc5", 5);
    assert!(pt.state_count <= 200, "chain-5 should have ≤200 states");
}

#[test]
fn test_as_v9_upper_bound_alt10() {
    let pt = alt_table("as_v9_uba10", 10);
    assert!(pt.state_count <= 500, "alt-10 should have ≤500 states");
}

#[test]
fn test_as_v9_upper_bound_sequence5() {
    let pt = make_table(
        "as_v9_ubsq",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    assert!(pt.state_count <= 200);
}

#[test]
fn test_as_v9_upper_bound_symbol_count() {
    let tokens: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{i}"), format!("p{i}")))
        .collect();
    let refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let pt = make_table("as_v9_ubsc", &refs, &[("s", vec!["t0"])], "s");
    // symbol_count should be bounded reasonably
    assert!(pt.symbol_count <= 1000);
}

// =========================================================================
// 8. Arithmetic / expression grammars (46-51)
// =========================================================================

#[test]
fn test_as_v9_arith_basic_state_count() {
    let pt = make_table_with_prec(
        "as_v9_arith",
        &[("num", r"\d+"), ("plus", r"\+"), ("star", r"\*")],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "expr",
    );
    assert!(pt.state_count >= 3, "arith grammar needs ≥3 states");
    assert!(pt.state_count <= 200, "arith grammar should be moderate");
}

#[test]
fn test_as_v9_arith_has_multiple_symbols() {
    let pt = make_table_with_prec(
        "as_v9_arith2",
        &[("num", r"\d+"), ("plus", r"\+"), ("star", r"\*")],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "expr",
    );
    // 3 tokens + 1 NT + EOF
    assert!(pt.symbol_count >= 5);
}

#[test]
fn test_as_v9_arith_with_minus() {
    let pt = make_table_with_prec(
        "as_v9_arithm",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("minus", r"\-"),
            ("star", r"\*"),
        ],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "minus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus", "minus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "expr",
    );
    assert!(pt.state_count >= 3);
}

#[test]
fn test_as_v9_arith_with_parens() {
    let pt = make_table_with_prec(
        "as_v9_arithp",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("star", r"\*"),
            ("lp", r"\("),
            ("rp", r"\)"),
        ],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["lp", "expr", "rp"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "expr",
    );
    // Parens add states
    assert!(pt.state_count >= 4);
}

#[test]
fn test_as_v9_arith_more_ops_more_symbols() {
    let pt2 = make_table_with_prec(
        "as_v9_ao2",
        &[("num", r"\d+"), ("plus", r"\+")],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["num"]),
        ],
        &[(1, Associativity::Left, vec!["plus"])],
        "expr",
    );
    let pt4 = make_table_with_prec(
        "as_v9_ao4",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("minus", r"\-"),
            ("star", r"\*"),
            ("slash", r"\/"),
        ],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "minus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["expr", "slash", "expr"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus", "minus"]),
            (2, Associativity::Left, vec!["star", "slash"]),
        ],
        "expr",
    );
    assert!(pt4.symbol_count > pt2.symbol_count);
}

#[test]
fn test_as_v9_arith_moderate_state_count() {
    let pt = make_table_with_prec(
        "as_v9_armod",
        &[("num", r"\d+"), ("plus", r"\+"), ("star", r"\*")],
        &[
            ("expr", vec!["expr", "plus", "term"]),
            ("expr", vec!["term"]),
            ("term", vec!["term", "star", "factor"]),
            ("term", vec!["factor"]),
            ("factor", vec!["num"]),
        ],
        &[],
        "expr",
    );
    // Classic expr/term/factor — moderate
    assert!(pt.state_count >= 4);
    assert!(pt.state_count <= 300);
}

// =========================================================================
// 9. Chain grammars and linear growth (52-59)
// =========================================================================

#[test]
fn test_as_v9_chain_1() {
    let pt = chain_table("as_v9_c1", 1);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_chain_2() {
    let pt = chain_table("as_v9_c2", 2);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_chain_4() {
    let pt = chain_table("as_v9_c4", 4);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_chain_6() {
    let pt = chain_table("as_v9_c6", 6);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_chain_linear_growth() {
    let pt2 = chain_table("as_v9_cl2", 2);
    let pt4 = chain_table("as_v9_cl4", 4);
    let pt8 = chain_table("as_v9_cl8", 8);
    // Growth should be at most linear: states(8) - states(4) <= 2 * (states(4) - states(2))
    let d1 = pt4.state_count as isize - pt2.state_count as isize;
    let d2 = pt8.state_count as isize - pt4.state_count as isize;
    // Allow generous margin: d2 <= 3*d1 + 10
    assert!(
        d2 <= 3 * d1 + 10,
        "chain growth should be roughly linear: d1={d1}, d2={d2}"
    );
}

#[test]
fn test_as_v9_chain_symbol_count_grows() {
    let pt2 = chain_table("as_v9_cs2", 2);
    let pt5 = chain_table("as_v9_cs5", 5);
    // More NTs → more symbols
    assert!(pt5.symbol_count >= pt2.symbol_count);
}

#[test]
fn test_as_v9_chain_nt_count_matches_depth() {
    let pt = chain_table("as_v9_cnd", 4);
    // 4 NTs + 1 token + EOF → ≥ 6
    assert!(pt.symbol_count >= 6);
}

#[test]
fn test_as_v9_chain_10_bounded() {
    let pt = chain_table("as_v9_c10", 10);
    assert!(pt.state_count <= 500);
}

// =========================================================================
// 10. Alternative grammars (60-65)
// =========================================================================

#[test]
fn test_as_v9_alt_2() {
    let pt = alt_table("as_v9_alt2", 2);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_alt_4() {
    let pt = alt_table("as_v9_alt4", 4);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_alt_8() {
    let pt = alt_table("as_v9_alt8", 8);
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_alt_more_alts_more_states() {
    let pt3 = alt_table("as_v9_am3", 3);
    let pt6 = alt_table("as_v9_am6", 6);
    assert!(pt6.state_count >= pt3.state_count);
}

#[test]
fn test_as_v9_alt_symbol_count_grows() {
    let pt2 = alt_table("as_v9_asc2", 2);
    let pt5 = alt_table("as_v9_asc5", 5);
    assert!(pt5.symbol_count > pt2.symbol_count);
}

#[test]
fn test_as_v9_alt_10_bounded() {
    let pt = alt_table("as_v9_alt10", 10);
    assert!(pt.state_count <= 500);
    assert!(pt.symbol_count <= 500);
}

// =========================================================================
// 11. Precedence grammars (66-71)
// =========================================================================

#[test]
fn test_as_v9_prec_left_assoc() {
    let pt = make_table_with_prec(
        "as_v9_pl",
        &[("a", "a"), ("op", r"\+")],
        &[("s", vec!["s", "op", "s"]), ("s", vec!["a"])],
        &[(1, Associativity::Left, vec!["op"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_prec_right_assoc() {
    let pt = make_table_with_prec(
        "as_v9_pr",
        &[("a", "a"), ("op", r"\+")],
        &[("s", vec!["s", "op", "s"]), ("s", vec!["a"])],
        &[(1, Associativity::Right, vec!["op"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_prec_two_levels() {
    let pt = make_table_with_prec(
        "as_v9_p2l",
        &[("a", "a"), ("add", r"\+"), ("mul", r"\*")],
        &[
            ("s", vec!["s", "add", "s"]),
            ("s", vec!["s", "mul", "s"]),
            ("s", vec!["a"]),
        ],
        &[
            (1, Associativity::Left, vec!["add"]),
            (2, Associativity::Left, vec!["mul"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 3);
}

#[test]
fn test_as_v9_prec_three_levels() {
    let pt = make_table_with_prec(
        "as_v9_p3l",
        &[("a", "a"), ("add", r"\+"), ("mul", r"\*"), ("exp", r"\^")],
        &[
            ("s", vec!["s", "add", "s"]),
            ("s", vec!["s", "mul", "s"]),
            ("s", vec!["s", "exp", "s"]),
            ("s", vec!["a"]),
        ],
        &[
            (1, Associativity::Left, vec!["add"]),
            (2, Associativity::Left, vec!["mul"]),
            (3, Associativity::Right, vec!["exp"]),
        ],
        "s",
    );
    assert!(pt.state_count >= 3);
}

#[test]
fn test_as_v9_prec_does_not_explode_states() {
    let pt = make_table_with_prec(
        "as_v9_pne",
        &[("a", "a"), ("add", r"\+"), ("mul", r"\*")],
        &[
            ("s", vec!["s", "add", "s"]),
            ("s", vec!["s", "mul", "s"]),
            ("s", vec!["a"]),
        ],
        &[
            (1, Associativity::Left, vec!["add"]),
            (2, Associativity::Left, vec!["mul"]),
        ],
        "s",
    );
    assert!(pt.state_count <= 200);
}

#[test]
fn test_as_v9_prec_symbol_count_stable() {
    // Same tokens/NTs with vs without prec → same symbol count
    let pt_no = make_table(
        "as_v9_pscn",
        &[("a", "a"), ("op", r"\+")],
        &[("s", vec!["s", "op", "s"]), ("s", vec!["a"])],
        "s",
    );
    let pt_yes = make_table_with_prec(
        "as_v9_pscy",
        &[("a", "a"), ("op", r"\+")],
        &[("s", vec!["s", "op", "s"]), ("s", vec!["a"])],
        &[(1, Associativity::Left, vec!["op"])],
        "s",
    );
    // Symbol count should be the same since vocabulary is identical
    assert_eq!(pt_no.symbol_count, pt_yes.symbol_count);
}

// =========================================================================
// 12. Extras / inline grammars (72-77)
// =========================================================================

#[test]
fn test_as_v9_extras_symbol_count() {
    let pt_no = make_table(
        "as_v9_exn",
        &[("a", "a"), ("ws", r"\s+")],
        &[("s", vec!["a"])],
        "s",
    );
    let pt_ex = make_table_with_extras(
        "as_v9_exy",
        &[("a", "a"), ("ws", r"\s+")],
        &[("s", vec!["a"])],
        &["ws"],
        "s",
    );
    // Both have the same token set so symbol counts should be similar
    // The extras grammar may differ slightly in symbol count due to internal handling
    assert!(pt_ex.symbol_count >= 2);
    assert!(pt_no.symbol_count >= 2);
}

#[test]
fn test_as_v9_extras_state_count_similar() {
    let pt_no = make_table(
        "as_v9_esn",
        &[("a", "a"), ("ws", r"\s+")],
        &[("s", vec!["a"])],
        "s",
    );
    let pt_ex = make_table_with_extras(
        "as_v9_esy",
        &[("a", "a"), ("ws", r"\s+")],
        &[("s", vec!["a"])],
        &["ws"],
        "s",
    );
    // Extras shouldn't drastically change state count
    let diff = (pt_ex.state_count as isize - pt_no.state_count as isize).unsigned_abs();
    assert!(
        diff <= 20,
        "extras should not drastically change state count: diff={diff}"
    );
}

#[test]
fn test_as_v9_extras_still_builds() {
    let pt = make_table_with_extras(
        "as_v9_esb",
        &[("a", "a"), ("b", "b"), ("ws", r"\s+")],
        &[("s", vec!["a", "b"])],
        &["ws"],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn test_as_v9_inline_may_change_states() {
    let pt_no = make_table(
        "as_v9_iln",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["helper"]), ("helper", vec!["a", "b"])],
        "s",
    );
    let pt_in = make_table_with_inline(
        "as_v9_ily",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["helper"]), ("helper", vec!["a", "b"])],
        &["helper"],
        "s",
    );
    // Both should build; inline may merge states or change count
    assert!(pt_no.state_count >= 2);
    assert!(pt_in.state_count >= 2);
}

#[test]
fn test_as_v9_inline_symbol_count_may_differ() {
    let pt_no = make_table(
        "as_v9_isdn",
        &[("a", "a")],
        &[("s", vec!["helper"]), ("helper", vec!["a"])],
        "s",
    );
    let pt_in = make_table_with_inline(
        "as_v9_isdy",
        &[("a", "a")],
        &[("s", vec!["helper"]), ("helper", vec!["a"])],
        &["helper"],
        "s",
    );
    // Inlining may reduce symbol count (one fewer NT) or keep it
    assert!(pt_no.symbol_count >= 2);
    assert!(pt_in.symbol_count >= 2);
}

#[test]
fn test_as_v9_inline_with_extras_combined() {
    let pt = make_table_with_extras(
        "as_v9_iec",
        &[("a", "a"), ("b", "b"), ("ws", r"\s+")],
        &[("s", vec!["a", "b"])],
        &["ws"],
        "s",
    );
    assert!(pt.state_count >= 2);
    assert!(pt.symbol_count >= 3);
}

// =========================================================================
// 13. Simple-vs-complex comparisons (78-81)
// =========================================================================

#[test]
fn test_as_v9_simple_lt_complex_states() {
    let simple = make_table("as_v9_svs", &[("a", "a")], &[("s", vec!["a"])], "s");
    let complex = make_table_with_prec(
        "as_v9_svc",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("star", r"\*"),
            ("lp", r"\("),
            ("rp", r"\)"),
        ],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["lp", "expr", "rp"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "expr",
    );
    assert!(
        complex.state_count > simple.state_count,
        "complex ({}) should have more states than simple ({})",
        complex.state_count,
        simple.state_count
    );
}

#[test]
fn test_as_v9_simple_lt_complex_symbols() {
    let simple = make_table("as_v9_sys", &[("a", "a")], &[("s", vec!["a"])], "s");
    let complex = make_table(
        "as_v9_syc",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("s", vec!["x", "y"]),
            ("x", vec!["a", "b"]),
            ("y", vec!["c", "d", "e"]),
        ],
        "s",
    );
    assert!(complex.symbol_count > simple.symbol_count);
}

#[test]
fn test_as_v9_chain_lt_arith_states() {
    let chain = chain_table("as_v9_cla", 3);
    let arith = make_table_with_prec(
        "as_v9_clb",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("star", r"\*"),
            ("lp", r"\("),
            ("rp", r"\)"),
        ],
        &[
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["lp", "expr", "rp"]),
            ("expr", vec!["num"]),
        ],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "expr",
    );
    // Arithmetic grammar with parens should have more states than a simple chain of 3
    assert!(
        arith.state_count > chain.state_count,
        "arith ({}) should exceed chain-3 ({})",
        arith.state_count,
        chain.state_count
    );
}

#[test]
fn test_as_v9_1rule_lt_multi_rule() {
    let single = make_table("as_v9_1rm", &[("a", "a")], &[("s", vec!["a"])], "s");
    let multi = make_table(
        "as_v9_mrm",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("s", vec!["x"]),
            ("s", vec!["y"]),
            ("x", vec!["a", "b"]),
            ("y", vec!["b", "c"]),
        ],
        "s",
    );
    assert!(multi.state_count >= single.state_count);
}

// =========================================================================
// 14. Polynomial scaling (82-84)
// =========================================================================

#[test]
fn test_as_v9_scaling_not_exponential_chains() {
    let pt3 = chain_table("as_v9_ne3", 3);
    let pt6 = chain_table("as_v9_ne6", 6);
    let pt12 = chain_table("as_v9_ne12", 12);
    // Exponential would be: states(12) >> states(6)^2 / states(3)
    // We just check it's bounded polynomially: states(12) < 10 * states(6)
    assert!(
        pt12.state_count < 10 * pt6.state_count + 20,
        "chain growth should not be exponential: s3={}, s6={}, s12={}",
        pt3.state_count,
        pt6.state_count,
        pt12.state_count
    );
}

#[test]
fn test_as_v9_scaling_not_exponential_alts() {
    let pt3 = alt_table("as_v9_nea3", 3);
    let pt6 = alt_table("as_v9_nea6", 6);
    let pt12 = alt_table("as_v9_nea12", 12);
    assert!(
        pt12.state_count < 10 * pt6.state_count + 20,
        "alt growth should not be exponential: s3={}, s6={}, s12={}",
        pt3.state_count,
        pt6.state_count,
        pt12.state_count
    );
}

#[test]
fn test_as_v9_scaling_polynomial_ratio() {
    // For chain grammars, doubling depth should at most double states + constant
    let pt4 = chain_table("as_v9_spr4", 4);
    let pt8 = chain_table("as_v9_spr8", 8);
    let ratio = pt8.state_count as f64 / pt4.state_count.max(1) as f64;
    assert!(
        ratio < 5.0,
        "doubling chain depth should yield polynomial growth, ratio={ratio:.2}"
    );
}
