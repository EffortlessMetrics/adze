//! Comprehensive tests for `ParseTable::rule()` information access (v9).
//!
//! 80+ tests verifying that `.rule(RuleId)` returns correct `(SymbolId, u16)`
//! tuples representing (LHS symbol, RHS length) for every rule in the table.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test rule_info_v9 -- --test-threads=2

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, RuleId, StateId, SymbolId};

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
    rules: &[(&str, Vec<&str>, i16, Associativity)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs, prec, assoc) in rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

/// Collect all RuleIds referenced by Reduce actions in the table.
fn collect_reduce_rule_ids(table: &ParseTable) -> Vec<RuleId> {
    let mut ids = Vec::new();
    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rid) = action {
                    ids.push(*rid);
                }
            }
        }
    }
    ids.sort_by_key(|r| r.0);
    ids.dedup();
    ids
}

// ---------------------------------------------------------------------------
// 1–6. Basic rule access and RHS lengths
// ---------------------------------------------------------------------------

#[test]
fn test_rule_id_zero_returns_valid_data() {
    let pt = make_table("ri_v9_r0", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert!(lhs.0 < pt.symbol_count as u16);
    assert!(rhs_len <= 16);
}

#[test]
fn test_rule_lhs_is_valid_symbol_id() {
    let pt = make_table("ri_v9_lhsv", &[("x", "x")], &[("s", vec!["x"])], "s");
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert!(
            lhs.0 < pt.symbol_count as u16,
            "rule {idx} LHS {lhs:?} out of range"
        );
    }
}

#[test]
fn test_single_element_rule_rhs_length_1() {
    let pt = make_table("ri_v9_rhs1", &[("a", "a")], &[("s", vec!["a"])], "s");
    // Find a user rule with RHS length 1
    let has_one = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 1);
    assert!(has_one, "expected at least one rule with RHS length 1");
}

#[test]
fn test_two_element_rule_rhs_length_2() {
    let pt = make_table(
        "ri_v9_rhs2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let has_two = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 2);
    assert!(has_two, "expected at least one rule with RHS length 2");
}

#[test]
fn test_three_element_rule_rhs_length_3() {
    let pt = make_table(
        "ri_v9_rhs3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    let has_three = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 3);
    assert!(has_three, "expected at least one rule with RHS length 3");
}

#[test]
fn test_four_element_rule_rhs_length_4() {
    let pt = make_table(
        "ri_v9_rhs4",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    );
    let has_four = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 4);
    assert!(has_four, "expected at least one rule with RHS length 4");
}

// ---------------------------------------------------------------------------
// 7–9. Multiple rules and determinism
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_rules_all_accessible() {
    let pt = make_table(
        "ri_v9_multi",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(
        pt.rules.len() >= 2,
        "expected at least 2 rules, got {}",
        pt.rules.len()
    );
    for idx in 0..pt.rules.len() {
        let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
        assert!(rhs_len <= 16);
    }
}

#[test]
fn test_rule_info_deterministic() {
    let pt = make_table("ri_v9_det", &[("x", "x")], &[("s", vec!["x"])], "s");
    let first = pt.rule(RuleId(0));
    let second = pt.rule(RuleId(0));
    assert_eq!(first, second);
}

#[test]
fn test_same_grammar_same_rule_info() {
    let pt1 = make_table("ri_v9_eq1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table("ri_v9_eq2", &[("a", "a")], &[("s", vec!["a"])], "s");
    // Same structure → same rule info (LHS and rhs_len match)
    assert_eq!(pt1.rules.len(), pt2.rules.len());
    for idx in 0..pt1.rules.len() {
        let r1 = pt1.rule(RuleId(idx as u16));
        let r2 = pt2.rule(RuleId(idx as u16));
        assert_eq!(r1.1, r2.1, "RHS length should match for rule {idx}");
    }
}

// ---------------------------------------------------------------------------
// 10. Different grammars may differ
// ---------------------------------------------------------------------------

#[test]
fn test_different_grammars_may_differ() {
    let pt1 = make_table("ri_v9_diff1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = make_table(
        "ri_v9_diff2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    // At least one rule should have a different RHS length
    let r1_max = (0..pt1.rules.len())
        .map(|i| pt1.rule(RuleId(i as u16)).1)
        .max()
        .unwrap_or(0);
    let r2_max = (0..pt2.rules.len())
        .map(|i| pt2.rule(RuleId(i as u16)).1)
        .max()
        .unwrap_or(0);
    assert_ne!(
        r1_max, r2_max,
        "different grammars should produce different max RHS lengths"
    );
}

// ---------------------------------------------------------------------------
// 11–12. Terminal vs non-terminal LHS
// ---------------------------------------------------------------------------

#[test]
fn test_rule_lhs_for_nonterminal_rule() {
    let pt = make_table("ri_v9_ntlhs", &[("a", "a")], &[("s", vec!["a"])], "s");
    // The user-defined rule for "s" should have a non-terminal LHS
    let has_nt_lhs = (0..pt.rules.len()).any(|i| {
        let (lhs, _) = pt.rule(RuleId(i as u16));
        !pt.is_terminal(lhs)
    });
    assert!(
        has_nt_lhs,
        "expected at least one rule with non-terminal LHS"
    );
}

#[test]
fn test_no_rule_lhs_is_eof() {
    let pt = make_table("ri_v9_noeof", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof();
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert_ne!(lhs, eof, "rule LHS should never be EOF");
    }
}

// ---------------------------------------------------------------------------
// 13. All Reduce actions reference valid RuleIds
// ---------------------------------------------------------------------------

#[test]
fn test_all_reduce_actions_reference_valid_rule_ids() {
    let pt = make_table(
        "ri_v9_redval",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let rule_count = pt.rules.len();
    for state_idx in 0..pt.state_count {
        let state = StateId(state_idx as u16);
        for &sym in pt.symbol_to_index.keys() {
            for action in pt.actions(state, sym) {
                if let Action::Reduce(rid) = action {
                    assert!(
                        (rid.0 as usize) < rule_count,
                        "Reduce({}) exceeds rule count {rule_count}",
                        rid.0
                    );
                }
            }
        }
    }
}

#[test]
fn test_reduce_rule_ids_have_valid_lhs() {
    let pt = make_table(
        "ri_v9_redlhs",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let ids = collect_reduce_rule_ids(&pt);
    for rid in ids {
        let (lhs, _) = pt.rule(rid);
        assert!(
            lhs.0 < pt.symbol_count as u16,
            "Reduce rule {rid:?} has invalid LHS"
        );
    }
}

// ---------------------------------------------------------------------------
// 14. Rule count consistency
// ---------------------------------------------------------------------------

#[test]
fn test_rule_info_consistent_with_rule_count() {
    let pt = make_table(
        "ri_v9_cnt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // The augmented grammar may add an extra rule; check that rules.len() >= user rules
    assert!(pt.rules.len() >= 2, "expected at least 2 rules");
}

#[test]
fn test_rule_count_at_least_user_rules() {
    let pt = make_table(
        "ri_v9_usrcnt",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(pt.rules.len() >= 3);
}

// ---------------------------------------------------------------------------
// 15. Arithmetic grammar rules
// ---------------------------------------------------------------------------

#[test]
fn test_arithmetic_grammar_rules() {
    let pt = make_table(
        "ri_v9_arith",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[("expr", vec!["num"]), ("expr", vec!["expr", "plus", "num"])],
        "expr",
    );
    // Should have rules with RHS 1 and RHS 3
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(
        rhs_lengths.contains(&1),
        "expected RHS length 1 for expr -> num"
    );
    assert!(
        rhs_lengths.contains(&3),
        "expected RHS length 3 for expr -> expr + num"
    );
}

#[test]
fn test_arithmetic_grammar_all_lhs_valid() {
    let pt = make_table(
        "ri_v9_arithv",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[("expr", vec!["num"]), ("expr", vec!["expr", "plus", "num"])],
        "expr",
    );
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
    }
}

// ---------------------------------------------------------------------------
// 16. Grammar with precedence
// ---------------------------------------------------------------------------

#[test]
fn test_precedence_grammar_rule_info() {
    let pt = make_table_with_prec(
        "ri_v9_prec1",
        &[("num", "[0-9]+"), ("plus", "\\+"), ("star", "\\*")],
        &[
            ("expr", vec!["num"], 0, Associativity::None),
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "star", "expr"], 2, Associativity::Left),
        ],
        "expr",
    );
    assert!(pt.rules.len() >= 3);
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&3));
}

#[test]
fn test_precedence_grammar_lhs_consistency() {
    let pt = make_table_with_prec(
        "ri_v9_prec2",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("expr", vec!["num"], 0, Associativity::None),
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
        ],
        "expr",
    );
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
    }
}

#[test]
fn test_right_associative_grammar_rule_info() {
    let pt = make_table_with_prec(
        "ri_v9_rassoc",
        &[("num", "[0-9]+"), ("caret", "\\^")],
        &[
            ("expr", vec!["num"], 0, Associativity::None),
            (
                "expr",
                vec!["expr", "caret", "expr"],
                1,
                Associativity::Right,
            ),
        ],
        "expr",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&3));
}

// ---------------------------------------------------------------------------
// 17. Grammar with alternatives → multiple rules
// ---------------------------------------------------------------------------

#[test]
fn test_alternatives_produce_multiple_rules() {
    let pt = make_table(
        "ri_v9_alts",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    // All three user alternatives should result in rules with RHS 1
    let count_rhs1 = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 1)
        .count();
    assert!(count_rhs1 >= 3, "expected at least 3 rules with RHS len 1");
}

#[test]
fn test_alternatives_with_different_lengths() {
    let pt = make_table(
        "ri_v9_altlen",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["a", "b"])],
        "s",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&2));
}

// ---------------------------------------------------------------------------
// 18. Grammar with chain → rule info
// ---------------------------------------------------------------------------

#[test]
fn test_chain_grammar_rule_info() {
    let pt = make_table(
        "ri_v9_chain",
        &[("x", "x")],
        &[("s", vec!["mid"]), ("mid", vec!["x"])],
        "s",
    );
    // Both user rules have RHS length 1
    let count_rhs1 = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 1)
        .count();
    assert!(
        count_rhs1 >= 2,
        "chain grammar should have ≥2 rules with RHS 1"
    );
}

#[test]
fn test_deep_chain_grammar() {
    let pt = make_table(
        "ri_v9_dchain",
        &[("x", "x")],
        &[("s", vec!["a"]), ("a", vec!["b"]), ("b", vec!["x"])],
        "s",
    );
    let count_rhs1 = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 1)
        .count();
    assert!(count_rhs1 >= 3);
}

// ---------------------------------------------------------------------------
// 19–20. Rule ID sequence and rule count
// ---------------------------------------------------------------------------

#[test]
fn test_rule_id_sequence_is_contiguous() {
    let pt = make_table(
        "ri_v9_contig",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // Every index from 0..rules.len() should be accessible
    for idx in 0..pt.rules.len() {
        let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
        assert!(rhs_len <= 64);
    }
}

#[test]
fn test_rule_count_positive() {
    let pt = make_table("ri_v9_rcpos", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.rules.is_empty());
}

#[test]
fn test_rule_count_includes_augmented() {
    let pt = make_table(
        "ri_v9_aug",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // Table should contain at least the 2 user rules
    assert!(pt.rules.len() >= 2, "expected at least 2 rules");
}

// ---------------------------------------------------------------------------
// Additional tests for comprehensive coverage (21–80+)
// ---------------------------------------------------------------------------

// 21. State count is positive
#[test]
fn test_state_count_positive() {
    let pt = make_table("ri_v9_stpos", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count > 0);
}

// 22. Symbol count is positive
#[test]
fn test_symbol_count_positive() {
    let pt = make_table("ri_v9_sypos", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.symbol_count > 0);
}

// 23. Every rule LHS is below symbol_count
#[test]
fn test_all_rule_lhs_below_symbol_count() {
    let pt = make_table(
        "ri_v9_lhsub",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b"]), ("s", vec!["c"])],
        "s",
    );
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
    }
}

// 24. RHS length never exceeds grammar maximum
#[test]
fn test_rhs_length_within_bounds() {
    let pt = make_table(
        "ri_v9_rhsbn",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    for idx in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(idx as u16));
        assert!(rhs_len <= 5, "RHS length {rhs_len} exceeds max 5");
    }
}

// 25. Rule with only terminals
#[test]
fn test_rule_all_terminals_rhs() {
    let pt = make_table(
        "ri_v9_alltrm",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let has_rhs2 = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 2);
    assert!(has_rhs2);
}

// 26. Rule mixing terminals and non-terminals
#[test]
fn test_rule_mixed_rhs() {
    let pt = make_table(
        "ri_v9_mixed",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["inner", "b"]), ("inner", vec!["a"])],
        "s",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&2));
    assert!(rhs_lengths.contains(&1));
}

// 27. Repeated accesses are consistent
#[test]
fn test_repeated_access_consistency() {
    let pt = make_table("ri_v9_rep", &[("a", "a")], &[("s", vec!["a"])], "s");
    for idx in 0..pt.rules.len() {
        let rid = RuleId(idx as u16);
        let r1 = pt.rule(rid);
        let r2 = pt.rule(rid);
        let r3 = pt.rule(rid);
        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }
}

// 28. Rule info survives across different queries
#[test]
fn test_rule_info_stable_across_queries() {
    let pt = make_table(
        "ri_v9_stbl",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let before: Vec<(SymbolId, u16)> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)))
        .collect();
    // Query other table data in between
    let _sc = pt.state_count;
    let _syc = pt.symbol_count;
    let _eof = pt.eof();
    let after: Vec<(SymbolId, u16)> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)))
        .collect();
    assert_eq!(before, after);
}

// 29. Multiple non-terminals
#[test]
fn test_multiple_nonterminals_rule_info() {
    let pt = make_table(
        "ri_v9_multnt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["x", "y"]), ("x", vec!["a"]), ("y", vec!["b"])],
        "s",
    );
    assert!(pt.rules.len() >= 3);
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
    }
}

// 30. Rule LHS is not the EOF symbol
#[test]
fn test_rule_lhs_never_eof() {
    let pt = make_table(
        "ri_v9_noeof2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let eof = pt.eof();
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert_ne!(lhs, eof);
    }
}

// 31. Reduce actions can be looked up via rule()
#[test]
fn test_reduce_actions_rule_lookup() {
    let pt = make_table("ri_v9_redlu", &[("a", "a")], &[("s", vec!["a"])], "s");
    let ids = collect_reduce_rule_ids(&pt);
    assert!(!ids.is_empty(), "expected at least one Reduce action");
    for rid in ids {
        let (lhs, rhs_len) = pt.rule(rid);
        assert!(lhs.0 < pt.symbol_count as u16);
        assert!(rhs_len >= 1);
    }
}

// 32. All rules have non-zero LHS
#[test]
fn test_all_rules_nonzero_fields() {
    let pt = make_table(
        "ri_v9_nzlhs",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        // LHS should be a valid symbol (could be 0 for first symbol)
        assert!(lhs.0 < pt.symbol_count as u16);
    }
}

// 33. Grammar with many tokens
#[test]
fn test_many_tokens_grammar() {
    let pt = make_table(
        "ri_v9_manytok",
        &[
            ("a", "a"),
            ("b", "b"),
            ("c", "c"),
            ("d", "d"),
            ("e", "e"),
            ("f", "f"),
        ],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
            ("s", vec!["e"]),
            ("s", vec!["f"]),
        ],
        "s",
    );
    assert!(pt.rules.len() >= 6);
}

// 34. Grammar with recursive rule
#[test]
fn test_recursive_rule_info() {
    let pt = make_table(
        "ri_v9_recur",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&2));
}

// 35. Left-recursive grammar
#[test]
fn test_left_recursive_rule_info() {
    let pt = make_table(
        "ri_v9_lrec",
        &[("a", "a"), ("plus", "\\+")],
        &[("s", vec!["a"]), ("s", vec!["s", "plus", "a"])],
        "s",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&3));
}

// 36. Rule index 0 always valid
#[test]
fn test_rule_zero_always_valid() {
    let pt = make_table("ri_v9_rzv", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert!(lhs.0 < pt.symbol_count as u16);
    assert!(rhs_len <= 32);
}

// 37. Last rule index valid
#[test]
fn test_last_rule_index_valid() {
    let pt = make_table(
        "ri_v9_lastrv",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let last = pt.rules.len() - 1;
    let (lhs, _) = pt.rule(RuleId(last as u16));
    assert!(lhs.0 < pt.symbol_count as u16);
}

// 38. Grammar with only one symbol
#[test]
fn test_single_token_grammar() {
    let pt = make_table("ri_v9_singletok", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert!(!pt.rules.is_empty());
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert!(lhs.0 < pt.symbol_count as u16);
    assert!(rhs_len >= 1);
}

// 39. Grammar start symbol appears as LHS
#[test]
fn test_start_symbol_appears_as_lhs() {
    let pt = make_table("ri_v9_stlhs", &[("a", "a")], &[("s", vec!["a"])], "s");
    let start = pt.start_symbol();
    let has_start_lhs = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).0 == start);
    // The augmented start or user start should appear as LHS
    assert!(
        has_start_lhs,
        "start symbol should appear as LHS in some rule"
    );
}

// 40. Rules with same LHS have same LHS symbol
#[test]
fn test_same_lhs_same_symbol() {
    let pt = make_table(
        "ri_v9_samelhs",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // Find all rules with RHS length 1 (user alternatives)
    let lhs_set: Vec<SymbolId> = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 1)
        .map(|i| pt.rule(RuleId(i as u16)).0)
        .collect();
    // All alternatives for "s" should share a LHS
    if lhs_set.len() >= 2 {
        assert_eq!(lhs_set[0], lhs_set[1], "alternatives should share LHS");
    }
}

// 41. No duplicate (LHS, rhs_len) implies distinct rules
#[test]
fn test_distinct_rule_shapes() {
    let pt = make_table(
        "ri_v9_distsh",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["a", "b"])],
        "s",
    );
    let shapes: Vec<(SymbolId, u16)> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)))
        .collect();
    // At least two distinct shapes (len 1 and len 2)
    let distinct: std::collections::HashSet<(u16, u16)> =
        shapes.iter().map(|(lhs, len)| (lhs.0, *len)).collect();
    assert!(distinct.len() >= 2);
}

// 42. Consistency between rules Vec and rule() method
#[test]
fn test_rules_vec_matches_rule_method() {
    let pt = make_table(
        "ri_v9_vecmtd",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    for (idx, pr) in pt.rules.iter().enumerate() {
        let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
        assert_eq!(lhs, pr.lhs);
        assert_eq!(rhs_len, pr.rhs_len);
    }
}

// 43. Grammar with intermediate non-terminal
#[test]
fn test_intermediate_nonterminal() {
    let pt = make_table(
        "ri_v9_intnt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["pair"]), ("pair", vec!["a", "b"])],
        "s",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&2));
}

// 44. RHS length 0 may exist (augmented rules)
#[test]
fn test_zero_rhs_length_possible() {
    let pt = make_table("ri_v9_zrhs", &[("a", "a")], &[("s", vec!["a"])], "s");
    // Just check that all RHS lengths are non-negative (always true for u16)
    for idx in 0..pt.rules.len() {
        let (_, rhs_len) = pt.rule(RuleId(idx as u16));
        // u16 is always >= 0; this validates that the value is reasonable
        assert!(rhs_len <= 100);
    }
}

// 45. Token count in table is consistent
#[test]
fn test_token_count_consistency() {
    let pt = make_table(
        "ri_v9_tokcnt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(pt.token_count >= 2, "expected at least 2 tokens");
}

// 46. Symbol count covers tokens and non-terminals
#[test]
fn test_symbol_count_covers_all() {
    let pt = make_table(
        "ri_v9_sycov",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // symbol_count should be at least: 2 tokens + 1 non-terminal + EOF
    assert!(pt.symbol_count >= 4);
}

// 47. Nested non-terminals
#[test]
fn test_nested_nonterminals() {
    let pt = make_table(
        "ri_v9_nested",
        &[("a", "a")],
        &[
            ("s", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["a"]),
        ],
        "s",
    );
    assert!(pt.rules.len() >= 3);
}

// 48. Grammar with two distinct LHS symbols
#[test]
fn test_two_distinct_lhs_symbols() {
    let pt = make_table(
        "ri_v9_twolhs",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["inner"]), ("inner", vec!["a", "b"])],
        "s",
    );
    let lhs_set: std::collections::HashSet<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).0.0)
        .collect();
    assert!(
        lhs_set.len() >= 2,
        "expected at least 2 distinct LHS symbols"
    );
}

// 49. Augmented rule has start symbol or wrapper LHS
#[test]
fn test_augmented_rule_exists() {
    let pt = make_table("ri_v9_augex", &[("a", "a")], &[("s", vec!["a"])], "s");
    // At least one rule should reduce to the start symbol or its augmented wrapper
    let start = pt.start_symbol();
    let has_start = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).0 == start);
    assert!(has_start);
}

// 50. Grammar with separator pattern
#[test]
fn test_separator_pattern() {
    let pt = make_table(
        "ri_v9_sep",
        &[("item", "[a-z]+"), ("comma", ",")],
        &[
            ("list", vec!["item"]),
            ("list", vec!["list", "comma", "item"]),
        ],
        "list",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&3));
}

// 51. Grammar with wrapping non-terminal
#[test]
fn test_wrapping_nonterminal() {
    let pt = make_table(
        "ri_v9_wrap",
        &[("lp", "\\("), ("rp", "\\)"), ("id", "[a-z]+")],
        &[("s", vec!["lp", "inner", "rp"]), ("inner", vec!["id"])],
        "s",
    );
    let has_rhs3 = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 3);
    assert!(has_rhs3, "expected parenthesized rule with RHS 3");
}

// 52. Copy semantics of RuleId
#[test]
fn test_rule_id_copy_semantics() {
    let pt = make_table("ri_v9_copy", &[("a", "a")], &[("s", vec!["a"])], "s");
    let rid = RuleId(0);
    let r1 = pt.rule(rid);
    let r2 = pt.rule(rid); // rid not consumed — Copy
    assert_eq!(r1, r2);
}

// 53. SymbolId returned is Copy
#[test]
fn test_symbol_id_copy_from_rule() {
    let pt = make_table("ri_v9_symcp", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (lhs, _) = pt.rule(RuleId(0));
    let lhs2 = lhs; // Copy
    assert_eq!(lhs, lhs2);
}

// 54. Large grammar rule count
#[test]
fn test_large_grammar_rule_count() {
    let tokens: Vec<(&str, &str)> = vec![
        ("t0", "0"),
        ("t1", "1"),
        ("t2", "2"),
        ("t3", "3"),
        ("t4", "4"),
        ("t5", "5"),
        ("t6", "6"),
        ("t7", "7"),
    ];
    let rules: Vec<(&str, Vec<&str>)> = vec![
        ("s", vec!["t0"]),
        ("s", vec!["t1"]),
        ("s", vec!["t2"]),
        ("s", vec!["t3"]),
        ("s", vec!["t4"]),
        ("s", vec!["t5"]),
        ("s", vec!["t6"]),
        ("s", vec!["t7"]),
    ];
    let pt = make_table("ri_v9_large", &tokens, &rules, "s");
    assert!(pt.rules.len() >= 8);
}

// 55. Every Reduce in every state references a rule we can look up
#[test]
fn test_every_reduce_lookupable() {
    let pt = make_table(
        "ri_v9_evred",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["x"]), ("x", vec!["a"]), ("x", vec!["b"])],
        "s",
    );
    for state_idx in 0..pt.state_count {
        let state = StateId(state_idx as u16);
        for &sym in pt.symbol_to_index.keys() {
            for action in pt.actions(state, sym) {
                if let Action::Reduce(rid) = action {
                    let (lhs, _) = pt.rule(*rid);
                    assert!(lhs.0 < pt.symbol_count as u16);
                }
            }
        }
    }
}

// 56. Grammar with token-only RHS across multiple rules
#[test]
fn test_multiple_terminal_only_rules() {
    let pt = make_table(
        "ri_v9_termonly",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("s", vec!["a", "b"]),
            ("s", vec!["b", "c"]),
            ("s", vec!["a", "c"]),
        ],
        "s",
    );
    let count_rhs2 = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 2)
        .count();
    assert!(count_rhs2 >= 3);
}

// 57. All RHS lengths are non-negative (trivially true for u16)
#[test]
fn test_rhs_lengths_non_negative() {
    let pt = make_table(
        "ri_v9_nonneg",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["a", "b"])],
        "s",
    );
    for idx in 0..pt.rules.len() {
        // u16 is inherently >= 0, but we verify the value is sensible
        let (_, rhs_len) = pt.rule(RuleId(idx as u16));
        assert!(rhs_len <= 1000);
    }
}

// 58. Rule for augmented start has RHS 1
#[test]
fn test_augmented_start_rhs_one() {
    let pt = make_table("ri_v9_augr1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let start = pt.start_symbol();
    let aug_rules: Vec<u16> = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).0 == start)
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    // Augmented start typically reduces S' -> S (RHS 1)
    assert!(
        aug_rules.contains(&1),
        "augmented start rule should have RHS 1"
    );
}

// 59. Reduce set not empty
#[test]
fn test_reduce_set_not_empty() {
    let pt = make_table("ri_v9_redne", &[("a", "a")], &[("s", vec!["a"])], "s");
    let ids = collect_reduce_rule_ids(&pt);
    assert!(!ids.is_empty());
}

// 60. Each reduce RuleId is within bounds
#[test]
fn test_reduce_rule_ids_within_bounds() {
    let pt = make_table(
        "ri_v9_ridbnd",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let ids = collect_reduce_rule_ids(&pt);
    for rid in ids {
        assert!((rid.0 as usize) < pt.rules.len());
    }
}

// 61. Grammar with keyword tokens
#[test]
fn test_keyword_tokens() {
    let pt = make_table(
        "ri_v9_kw",
        &[("kw_if", "if"), ("kw_then", "then"), ("id", "[a-z]+")],
        &[("s", vec!["kw_if", "id", "kw_then", "id"])],
        "s",
    );
    let has_rhs4 = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 4);
    assert!(has_rhs4);
}

// 62. EOF symbol is a valid symbol
#[test]
fn test_eof_symbol_valid() {
    let pt = make_table("ri_v9_eofv", &[("a", "a")], &[("s", vec!["a"])], "s");
    let eof = pt.eof();
    // EOF may be at or beyond symbol_count; just verify it is a valid SymbolId
    assert!(eof.0 <= pt.symbol_count as u16);
}

// 63. Start symbol is valid
#[test]
fn test_start_symbol_valid() {
    let pt = make_table("ri_v9_stsv", &[("a", "a")], &[("s", vec!["a"])], "s");
    let start = pt.start_symbol();
    assert!(start.0 < pt.symbol_count as u16);
}

// 64. Binary expression grammar
#[test]
fn test_binary_expression_grammar() {
    let pt = make_table(
        "ri_v9_binex",
        &[("num", "[0-9]+"), ("op", "[+\\-*/]")],
        &[("expr", vec!["num"]), ("expr", vec!["expr", "op", "expr"])],
        "expr",
    );
    assert!(pt.rules.len() >= 2);
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&3));
}

// 65. Rule with 5 symbols
#[test]
fn test_five_symbol_rule() {
    let pt = make_table(
        "ri_v9_five",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    let has_rhs5 = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 5);
    assert!(has_rhs5);
}

// 66. Multiple non-terminal chains produce distinct LHS
#[test]
fn test_multiple_chains_distinct_lhs() {
    let pt = make_table(
        "ri_v9_mchn",
        &[("a", "a"), ("b", "b")],
        &[
            ("s", vec!["left"]),
            ("s", vec!["right"]),
            ("left", vec!["a"]),
            ("right", vec!["b"]),
        ],
        "s",
    );
    let lhs_set: std::collections::HashSet<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).0.0)
        .collect();
    assert!(lhs_set.len() >= 3, "expected at least 3 distinct LHS");
}

// 67. Non-associative grammar
#[test]
fn test_non_associative_grammar() {
    let pt = make_table_with_prec(
        "ri_v9_nassoc",
        &[("num", "[0-9]+"), ("eq", "==")],
        &[
            ("expr", vec!["num"], 0, Associativity::None),
            ("expr", vec!["expr", "eq", "expr"], 1, Associativity::None),
        ],
        "expr",
    );
    assert!(pt.rules.len() >= 2);
}

// 68. Verify LHS for a specific user rule is non-terminal
#[test]
fn test_user_rule_lhs_is_nonterminal() {
    let pt = make_table("ri_v9_untlhs", &[("a", "a")], &[("s", vec!["a"])], "s");
    // User rule "s -> a" should have non-terminal LHS
    for idx in 0..pt.rules.len() {
        let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
        if rhs_len == 1 {
            // This could be the user rule or augmented; LHS should be non-terminal
            assert!(!pt.is_terminal(lhs), "rule LHS should be non-terminal");
        }
    }
}

// 69. Grammar where all user rules have same RHS length
#[test]
fn test_uniform_rhs_length() {
    let pt = make_table(
        "ri_v9_unirhsl",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("s", vec!["a", "b"]),
            ("s", vec!["b", "c"]),
            ("s", vec!["a", "c"]),
        ],
        "s",
    );
    let count_rhs2 = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 2)
        .count();
    assert!(count_rhs2 >= 3);
}

// 70. Table grammar accessor is consistent
#[test]
fn test_grammar_accessor() {
    let pt = make_table("ri_v9_gacc", &[("a", "a")], &[("s", vec!["a"])], "s");
    let grammar = pt.grammar();
    assert_eq!(grammar.name, "ri_v9_gacc");
}

// 71. Rule info for grammar with single alternative
#[test]
fn test_single_alternative() {
    let pt = make_table("ri_v9_sngalt", &[("a", "a")], &[("s", vec!["a"])], "s");
    // At least the single user rule
    assert!(!pt.rules.is_empty());
    let (lhs, rhs_len) = pt.rule(RuleId(0));
    assert!(lhs.0 < pt.symbol_count as u16);
    assert!(rhs_len >= 1);
}

// 72. RHS length distribution
#[test]
fn test_rhs_length_distribution() {
    let pt = make_table(
        "ri_v9_dist",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["a", "b"]),
            ("s", vec!["a", "b", "c"]),
        ],
        "s",
    );
    let mut lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    lengths.sort();
    lengths.dedup();
    assert!(
        lengths.len() >= 3,
        "expected at least 3 distinct RHS lengths"
    );
}

// 73. Parse rules direct field access matches method
#[test]
fn test_parse_rule_fields() {
    let pt = make_table(
        "ri_v9_prfld",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    for (idx, rule) in pt.rules.iter().enumerate() {
        let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
        assert_eq!(rule.lhs, lhs, "ParseRule.lhs mismatch at index {idx}");
        assert_eq!(
            rule.rhs_len, rhs_len,
            "ParseRule.rhs_len mismatch at index {idx}"
        );
    }
}

// 74. Grammar with statement-like pattern
#[test]
fn test_statement_pattern() {
    let pt = make_table(
        "ri_v9_stmt",
        &[
            ("id", "[a-z]+"),
            ("eq", "="),
            ("num", "[0-9]+"),
            ("semi", ";"),
        ],
        &[("s", vec!["id", "eq", "num", "semi"])],
        "s",
    );
    let has_rhs4 = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 4);
    assert!(has_rhs4);
}

// 75. Multiple LHS symbols each have valid rules
#[test]
fn test_multi_lhs_each_valid() {
    let pt = make_table(
        "ri_v9_mlhsv",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["p"]), ("p", vec!["a"]), ("p", vec!["a", "b"])],
        "s",
    );
    for idx in 0..pt.rules.len() {
        let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
        assert!(lhs.0 < pt.symbol_count as u16);
        assert!(rhs_len <= 10);
    }
}

// 76. Grammar with optional-like pattern (two alternatives)
#[test]
fn test_optional_pattern() {
    let pt = make_table(
        "ri_v9_opt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["a", "b"])],
        "s",
    );
    let rhs_lengths: Vec<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).1)
        .collect();
    assert!(rhs_lengths.contains(&1));
    assert!(rhs_lengths.contains(&2));
}

// 77. Symbol_to_index covers all action-table symbols
#[test]
fn test_symbol_to_index_coverage() {
    let pt = make_table(
        "ri_v9_symcov",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(!pt.symbol_to_index.is_empty());
}

// 78. Grammar with triple nesting
#[test]
fn test_triple_nested_grammar() {
    let pt = make_table(
        "ri_v9_triple",
        &[("x", "x")],
        &[
            ("s", vec!["a"]),
            ("a", vec!["b"]),
            ("b", vec!["c"]),
            ("c", vec!["x"]),
        ],
        "s",
    );
    // At least 4 user rules + augmented rule
    assert!(pt.rules.len() >= 4);
    let count_rhs1 = (0..pt.rules.len())
        .filter(|&i| pt.rule(RuleId(i as u16)).1 == 1)
        .count();
    assert!(count_rhs1 >= 4);
}

// 79. Grammar with two tokens in sequence, accessed multiple times
#[test]
fn test_two_token_sequence_repeated_access() {
    let pt = make_table(
        "ri_v9_2tseq",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    for _ in 0..10 {
        for idx in 0..pt.rules.len() {
            let (lhs, rhs_len) = pt.rule(RuleId(idx as u16));
            assert!(lhs.0 < pt.symbol_count as u16);
            assert!(rhs_len <= 10);
        }
    }
}

// 80. All reduce actions reference rules with non-terminal LHS
#[test]
fn test_reduce_rules_have_nt_lhs() {
    let pt = make_table(
        "ri_v9_rednt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["a", "b"])],
        "s",
    );
    let ids = collect_reduce_rule_ids(&pt);
    for rid in ids {
        let (lhs, _) = pt.rule(rid);
        assert!(
            !pt.is_terminal(lhs),
            "Reduce rule LHS should be non-terminal"
        );
    }
}

// 81. Parallel alternatives all produce valid rule info
#[test]
fn test_parallel_alternatives_all_valid() {
    let pt = make_table(
        "ri_v9_paralt",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
        ],
        "s",
    );
    for idx in 0..pt.rules.len() {
        let (lhs, _) = pt.rule(RuleId(idx as u16));
        assert!(!pt.is_terminal(lhs));
    }
}

// 82. Grammar returns expected state count range
#[test]
fn test_state_count_range() {
    let pt = make_table(
        "ri_v9_stcrng",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    // Simple grammar: at least initial + 1 shift + accept
    assert!(pt.state_count >= 3);
}

// 83. Precedence does not change RHS lengths
#[test]
fn test_precedence_preserves_rhs_lengths() {
    let pt_noprec = make_table(
        "ri_v9_pnp1",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[("expr", vec!["num"]), ("expr", vec!["expr", "plus", "num"])],
        "expr",
    );
    let pt_prec = make_table_with_prec(
        "ri_v9_pnp2",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("expr", vec!["num"], 0, Associativity::None),
            ("expr", vec!["expr", "plus", "num"], 1, Associativity::Left),
        ],
        "expr",
    );
    let lengths_noprec: std::collections::HashSet<u16> = (0..pt_noprec.rules.len())
        .map(|i| pt_noprec.rule(RuleId(i as u16)).1)
        .collect();
    let lengths_prec: std::collections::HashSet<u16> = (0..pt_prec.rules.len())
        .map(|i| pt_prec.rule(RuleId(i as u16)).1)
        .collect();
    assert_eq!(lengths_noprec, lengths_prec);
}

// 84. Grammar with diamond shape (two paths to same terminal)
#[test]
fn test_diamond_grammar() {
    let pt = make_table(
        "ri_v9_diamond",
        &[("a", "a")],
        &[
            ("s", vec!["left"]),
            ("s", vec!["right"]),
            ("left", vec!["a"]),
            ("right", vec!["a"]),
        ],
        "s",
    );
    assert!(pt.rules.len() >= 4);
    let lhs_set: std::collections::HashSet<u16> = (0..pt.rules.len())
        .map(|i| pt.rule(RuleId(i as u16)).0.0)
        .collect();
    // s, left, right, augmented → at least 3 distinct
    assert!(lhs_set.len() >= 3);
}

// 85. Rule with six symbols
#[test]
fn test_six_symbol_rule() {
    let pt = make_table(
        "ri_v9_six",
        &[
            ("a", "a"),
            ("b", "b"),
            ("c", "c"),
            ("d", "d"),
            ("e", "e"),
            ("f", "f"),
        ],
        &[("s", vec!["a", "b", "c", "d", "e", "f"])],
        "s",
    );
    let has_rhs6 = (0..pt.rules.len()).any(|i| pt.rule(RuleId(i as u16)).1 == 6);
    assert!(has_rhs6);
}
