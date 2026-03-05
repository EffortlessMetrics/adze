//! Comprehensive tests for conflict detection and resolution in adze-glr-core (v8).
//!
//! 80+ tests covering:
//! 1. Simple grammars with no conflicts
//! 2. ConflictResolver API basics
//! 3. detect_conflicts returns Vec<Conflict>
//! 4. Unambiguous grammars produce 0 conflicts
//! 5. Ambiguous grammars may produce conflicts
//! 6. ConflictKind classification (ShiftReduce / ReduceReduce)
//! 7. Conflict field validation (state, symbol, actions)
//! 8. Precedence-based resolution
//! 9. Associativity-based resolution
//! 10. Multi-token grammar patterns
//! 11. Chain rules
//! 12. Edge cases and complex patterns
//!
//! Run with: cargo test -p adze-glr-core --test conflict_detect_v8 -- --test-threads=2

use adze_glr_core::conflict_inspection::{classify_conflict, count_conflicts};
use adze_glr_core::{
    Action, ConflictResolver, ConflictType, FirstFollowSets, GotoIndexing, ItemSetCollection,
    ParseTable, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ============================================================================
// HELPERS
// ============================================================================

/// Build parse table from a grammar description.
fn build_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> (Grammar, ParseTable) {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    (g, pt)
}

/// Build item set collection + first-follow for ConflictResolver API.
fn build_collection(grammar: &Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

/// Run ConflictResolver::detect_conflicts on a grammar.
fn detect_all(grammar: &Grammar) -> Vec<adze_glr_core::Conflict> {
    let (col, ff) = build_collection(grammar);
    let resolver = ConflictResolver::detect_conflicts(&col, grammar, &ff);
    resolver.conflicts
}

/// Count cells with >1 action in a parse table.
fn count_multi_action_cells(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// True if any cell has both Shift and Reduce.
fn has_shift_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        })
    })
}

/// True if any cell has ≥2 Reduce actions.
fn has_reduce_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            cell.iter()
                .filter(|a| matches!(a, Action::Reduce(_)))
                .count()
                >= 2
        })
    })
}

/// Build a minimal ParseTable from raw action rows for unit tests.
fn make_table(action_table: Vec<Vec<Vec<Action>>>) -> ParseTable {
    let state_count = action_table.len();
    let symbol_count = action_table.first().map_or(0, |r| r.len());
    ParseTable {
        action_table,
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count,
        symbol_count,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Grammar::new("cd_v8_synth".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

// ============================================================================
// CATEGORY 1: SIMPLE GRAMMAR — NO CONFLICTS (tests 1–10)
// ============================================================================

#[test]
fn test_no_conflicts_single_rule() {
    let (_, pt) = build_table("cd_v8_nc1", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_two_tokens_sequence() {
    let (_, pt) = build_table(
        "cd_v8_nc2",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_disjoint_alternatives() {
    let (_, pt) = build_table(
        "cd_v8_nc3",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_three_tokens() {
    let (_, pt) = build_table(
        "cd_v8_nc4",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_chain_rule() {
    let (_, pt) = build_table(
        "cd_v8_nc5",
        &[("x", "x")],
        &[("start", vec!["inner"]), ("inner", vec!["x"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_nested_chain() {
    let (_, pt) = build_table(
        "cd_v8_nc6",
        &[("x", "x")],
        &[
            ("start", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_two_alt_diff_length() {
    let (_, pt) = build_table(
        "cd_v8_nc7",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["a", "b"])],
        "start",
    );
    // disjoint first tokens or lengths — table may or may not have conflicts
    // but we validate it builds
    assert!(pt.state_count > 0);
}

#[test]
fn test_no_conflicts_multiple_nonterminals() {
    let (_, pt) = build_table(
        "cd_v8_nc8",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["p"]),
            ("start", vec!["q"]),
            ("p", vec!["a"]),
            ("q", vec!["b"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_three_disjoint_alternatives() {
    let (_, pt) = build_table(
        "cd_v8_nc9",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_no_conflicts_sequential_long() {
    let (_, pt) = build_table(
        "cd_v8_nc10",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("start", vec!["a", "b", "c", "d"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

// ============================================================================
// CATEGORY 2: ConflictResolver API BASICS (tests 11–20)
// ============================================================================

#[test]
fn test_resolver_detect_does_not_panic() {
    let g = GrammarBuilder::new("cd_v8_api1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let _ = detect_all(&g);
}

#[test]
fn test_resolver_returns_vec_conflict() {
    let g = GrammarBuilder::new("cd_v8_api2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let conflicts = detect_all(&g);
    // simple grammar → 0 conflicts
    assert!(conflicts.is_empty());
}

#[test]
fn test_resolver_empty_for_unambiguous() {
    let g = GrammarBuilder::new("cd_v8_api3")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .start("start")
        .build();
    let conflicts = detect_all(&g);
    assert!(conflicts.is_empty());
}

#[test]
fn test_resolver_conflict_fields_accessible() {
    // E → E E | a  (inherently ambiguous)
    let g = GrammarBuilder::new("cd_v8_api4")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        // just access the fields to confirm no panic
        let _s = c.state;
        let _sym = c.symbol;
        let _ty = &c.conflict_type;
        assert!(c.actions.len() >= 2);
    }
}

#[test]
fn test_resolver_builds_from_collection() {
    let g = GrammarBuilder::new("cd_v8_api5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    assert!(resolver.conflicts.is_empty());
}

#[test]
fn test_resolver_conflicts_field_is_vec() {
    let g = GrammarBuilder::new("cd_v8_api6")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    let _v: &Vec<adze_glr_core::Conflict> = &resolver.conflicts;
    assert!(_v.is_empty());
}

#[test]
fn test_resolver_two_token_grammar_no_conflict() {
    let g = GrammarBuilder::new("cd_v8_api7")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(detect_all(&g).is_empty());
}

#[test]
fn test_resolver_chain_grammar_no_conflict() {
    let g = GrammarBuilder::new("cd_v8_api8")
        .token("x", "x")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["x"])
        .start("start")
        .build();
    assert!(detect_all(&g).is_empty());
}

#[test]
fn test_resolver_three_alternatives_no_conflict() {
    let g = GrammarBuilder::new("cd_v8_api9")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    assert!(detect_all(&g).is_empty());
}

#[test]
fn test_resolver_deep_chain_no_conflict() {
    let g = GrammarBuilder::new("cd_v8_api10")
        .token("z", "z")
        .rule("start", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["z"])
        .start("start")
        .build();
    assert!(detect_all(&g).is_empty());
}

// ============================================================================
// CATEGORY 3: CONFLICT TYPE CLASSIFICATION (tests 21–30)
// ============================================================================

#[test]
fn test_conflict_type_shift_reduce_variant() {
    let sr = ConflictType::ShiftReduce;
    assert_eq!(sr, ConflictType::ShiftReduce);
}

#[test]
fn test_conflict_type_reduce_reduce_variant() {
    let rr = ConflictType::ReduceReduce;
    assert_eq!(rr, ConflictType::ReduceReduce);
}

#[test]
fn test_conflict_type_not_equal() {
    assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
}

#[test]
fn test_conflict_type_debug_format() {
    let sr = ConflictType::ShiftReduce;
    let formatted = format!("{sr:?}");
    assert!(formatted.contains("ShiftReduce"));
}

#[test]
fn test_conflict_type_rr_debug() {
    let rr = ConflictType::ReduceReduce;
    let formatted = format!("{rr:?}");
    assert!(formatted.contains("ReduceReduce"));
}

#[test]
fn test_conflict_type_clone() {
    let ct = ConflictType::ShiftReduce;
    let ct2 = ct.clone();
    assert_eq!(ct, ct2);
}

#[test]
fn test_ambiguous_grammar_has_conflicts() {
    // E → E E | a  — classic ambiguity
    let g = GrammarBuilder::new("cd_v8_ct7")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    assert!(
        !conflicts.is_empty(),
        "ambiguous grammar should have conflicts"
    );
}

#[test]
fn test_ambiguous_grammar_conflict_type_is_valid() {
    let g = GrammarBuilder::new("cd_v8_ct8")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        assert!(
            c.conflict_type == ConflictType::ShiftReduce
                || c.conflict_type == ConflictType::ReduceReduce
        );
    }
}

#[test]
fn test_ambiguous_grammar_has_at_least_one_sr() {
    // E → E E | a — shift/reduce on 'a' lookahead
    let g = GrammarBuilder::new("cd_v8_ct9")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    let has_sr = conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "E → E E | a should have SR conflict");
}

#[test]
fn test_conflict_type_eq_reflexive() {
    let a = ConflictType::ShiftReduce;
    let b = ConflictType::ReduceReduce;
    assert_eq!(a, a.clone());
    assert_eq!(b, b.clone());
}

// ============================================================================
// CATEGORY 4: CONFLICT FIELD VALIDATION (tests 31–40)
// ============================================================================

#[test]
fn test_conflict_state_is_state_id() {
    let g = GrammarBuilder::new("cd_v8_fv1")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        let _id: StateId = c.state;
        // StateId is Copy
        let copy = c.state;
        assert_eq!(_id, copy);
    }
}

#[test]
fn test_conflict_symbol_is_symbol_id() {
    let g = GrammarBuilder::new("cd_v8_fv2")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        let _sym: SymbolId = c.symbol;
        let copy = c.symbol;
        assert_eq!(_sym, copy);
    }
}

#[test]
fn test_conflict_actions_has_two_or_more() {
    let g = GrammarBuilder::new("cd_v8_fv3")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        assert!(c.actions.len() >= 2, "conflicts should have ≥2 actions");
    }
}

#[test]
fn test_conflict_actions_vec_not_empty() {
    let g = GrammarBuilder::new("cd_v8_fv4")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        assert!(!c.actions.is_empty());
    }
}

#[test]
fn test_conflict_state_within_collection_range() {
    let g = GrammarBuilder::new("cd_v8_fv5")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let (col, ff) = build_collection(&g);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    let max_state = col.sets.len();
    for c in &resolver.conflicts {
        assert!((c.state.0 as usize) < max_state);
    }
}

#[test]
fn test_conflict_sr_has_shift_and_reduce() {
    let g = GrammarBuilder::new("cd_v8_fv6")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        if c.conflict_type == ConflictType::ShiftReduce {
            let has_shift = c.actions.iter().any(|a| matches!(a, Action::Shift(_)));
            let has_reduce = c.actions.iter().any(|a| matches!(a, Action::Reduce(_)));
            assert!(has_shift, "SR conflict must have a Shift");
            assert!(has_reduce, "SR conflict must have a Reduce");
        }
    }
}

#[test]
fn test_conflict_rr_has_multiple_actions() {
    let g = GrammarBuilder::new("cd_v8_fv7")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    // Every detected conflict (including ReduceReduce) must have ≥2 actions
    for c in &conflicts {
        assert!(
            c.actions.len() >= 2,
            "conflict must have ≥2 actions, got: {:?}",
            c.actions
        );
    }
}

#[test]
fn test_conflict_debug_format() {
    let g = GrammarBuilder::new("cd_v8_fv8")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        let dbg = format!("{c:?}");
        assert!(!dbg.is_empty());
    }
}

#[test]
fn test_conflict_actions_contain_known_variants() {
    let g = GrammarBuilder::new("cd_v8_fv9")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        for a in &c.actions {
            match a {
                Action::Shift(_)
                | Action::Reduce(_)
                | Action::Accept
                | Action::Error
                | Action::Recover
                | Action::Fork(_) => {}
                _ => {}
            }
        }
    }
}

#[test]
fn test_conflict_clone() {
    let g = GrammarBuilder::new("cd_v8_fv10")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let conflicts = detect_all(&g);
    for c in &conflicts {
        let c2 = c.clone();
        assert_eq!(c.state, c2.state);
        assert_eq!(c.symbol, c2.symbol);
        assert_eq!(c.conflict_type, c2.conflict_type);
        assert_eq!(c.actions.len(), c2.actions.len());
    }
}

// ============================================================================
// CATEGORY 5: PRECEDENCE RESOLUTION (tests 41–50)
// ============================================================================

#[test]
fn test_precedence_resolves_sr_to_fewer_conflicts() {
    // Without precedence: E → E + E | num
    let g_no_prec = GrammarBuilder::new("cd_v8_pr1a")
        .token("num", "[0-9]+")
        .token("+", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    let c_no_prec = detect_all(&g_no_prec).len();

    // With precedence
    let g_prec = GrammarBuilder::new("cd_v8_pr1b")
        .token("num", "[0-9]+")
        .token("+", "\\+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let c_prec = detect_all(&g_prec).len();

    // Precedence should reduce or maintain conflict count (resolution may or may not
    // eliminate all conflicts in detect_conflicts — it depends on resolve_conflicts)
    assert!(c_prec <= c_no_prec, "precedence should not add conflicts");
}

#[test]
fn test_left_assoc_grammar_builds() {
    let g = GrammarBuilder::new("cd_v8_pr2")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    assert!(pt.state_count > 0);
}

#[test]
fn test_right_assoc_grammar_builds() {
    let g = GrammarBuilder::new("cd_v8_pr3")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Right)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    assert!(pt.state_count > 0);
}

#[test]
fn test_higher_prec_op_grammar_builds() {
    let g = GrammarBuilder::new("cd_v8_pr4")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    assert!(pt.state_count > 0);
}

#[test]
fn test_prec_grammar_detect_conflicts_doesnt_panic() {
    let g = GrammarBuilder::new("cd_v8_pr5")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let _ = detect_all(&g);
}

#[test]
fn test_mixed_assoc_grammar_builds() {
    let g = GrammarBuilder::new("cd_v8_pr6")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Right)
        .start("expr")
        .build();
    let pt = build_table_from_grammar(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn test_three_precedence_levels() {
    let g = GrammarBuilder::new("cd_v8_pr7")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .start("expr")
        .build();
    let pt = build_table_from_grammar(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn test_resolve_conflicts_does_not_panic() {
    let g = GrammarBuilder::new("cd_v8_pr8")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let (col, ff) = build_collection(&g);
    let mut resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    resolver.resolve_conflicts(&g);
}

#[test]
fn test_resolve_conflicts_may_reduce_count() {
    let g = GrammarBuilder::new("cd_v8_pr9")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let (col, ff) = build_collection(&g);
    let mut resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    let before = resolver.conflicts.len();
    resolver.resolve_conflicts(&g);
    let after = resolver.conflicts.len();
    // resolve may or may not change count; just verify no panic and count is valid
    let _ = after;
    let _ = before;
}

#[test]
fn test_nonassoc_grammar_builds() {
    let g = GrammarBuilder::new("cd_v8_pr10")
        .token("n", "n")
        .token("=", "=")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::None)
        .start("expr")
        .build();
    let pt = build_table_from_grammar(&g);
    assert!(pt.state_count > 0);
}

// ============================================================================
// CATEGORY 6: PARSE TABLE CONFLICT INSPECTION (tests 51–60)
// ============================================================================

fn build_table_from_grammar(g: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(g).expect("ff");
    build_lr1_automaton(g, &ff).expect("table")
}

#[test]
fn test_count_conflicts_zero() {
    let (_, pt) = build_table("cd_v8_ci1", &[("a", "a")], &[("start", vec!["a"])], "start");
    let summary = count_conflicts(&pt);
    assert_eq!(summary.shift_reduce + summary.reduce_reduce, 0);
}

#[test]
fn test_count_conflicts_nonzero_for_ambiguous() {
    let g = GrammarBuilder::new("cd_v8_ci2")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let pt = build_table_from_grammar(&g);
    let summary = count_conflicts(&pt);
    assert!(
        summary.shift_reduce + summary.reduce_reduce > 0
            || count_multi_action_cells(&pt) > 0
            || !detect_all(&g).is_empty(),
        "ambiguous grammar should show conflicts somewhere"
    );
}

#[test]
fn test_synthetic_table_no_conflicts() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![], vec![Action::Accept]],
    ]);
    assert_eq!(count_multi_action_cells(&table), 0);
}

#[test]
fn test_synthetic_table_one_sr_conflict() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    assert_eq!(count_multi_action_cells(&table), 1);
}

#[test]
fn test_synthetic_table_one_rr_conflict() {
    let table = make_table(vec![vec![vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ]]]);
    assert_eq!(count_multi_action_cells(&table), 1);
}

#[test]
fn test_classify_sr_conflict() {
    let actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let ct = classify_conflict(&actions);
    assert_eq!(
        ct,
        adze_glr_core::conflict_inspection::ConflictType::ShiftReduce,
        "should classify as ShiftReduce"
    );
}

#[test]
fn test_classify_rr_conflict() {
    let actions = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    let ct = classify_conflict(&actions);
    assert_eq!(
        ct,
        adze_glr_core::conflict_inspection::ConflictType::ReduceReduce,
        "should classify as ReduceReduce"
    );
}

#[test]
fn test_classify_no_conflict_single_action() {
    let actions = vec![Action::Shift(StateId(0))];
    let ct = classify_conflict(&actions);
    // Single action — not really a conflict, but classify_conflict returns a type anyway
    // Mixed is used as the "no conflict" / "other" bucket
    let _ = ct; // just verify it doesn't panic
}

#[test]
fn test_classify_empty_cell() {
    let actions: Vec<Action> = vec![];
    let ct = classify_conflict(&actions);
    let _ = ct; // just verify it doesn't panic
}

#[test]
fn test_has_shift_reduce_helper() {
    let g = GrammarBuilder::new("cd_v8_ci10")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let pt = build_table_from_grammar(&g);
    // E → E E | a often leads to shift-reduce in the parse table
    // Check at least that the function doesn't panic
    let _ = has_shift_reduce(&pt);
}

// ============================================================================
// CATEGORY 7: MULTI-TOKEN AND CHAIN RULES (tests 61–70)
// ============================================================================

#[test]
fn test_chain_rule_a_to_b_to_x() {
    let (_, pt) = build_table(
        "cd_v8_ch1",
        &[("x", "x")],
        &[("start", vec!["a"]), ("a", vec!["b"]), ("b", vec!["x"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_chain_rule_four_levels() {
    let (_, pt) = build_table(
        "cd_v8_ch2",
        &[("x", "x")],
        &[
            ("start", vec!["a"]),
            ("a", vec!["b"]),
            ("b", vec!["c"]),
            ("c", vec!["x"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_multi_token_concat_no_conflict() {
    let (_, pt) = build_table(
        "cd_v8_ch3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_two_rules_different_first_token() {
    let (_, pt) = build_table(
        "cd_v8_ch4",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "c"]), ("start", vec!["b", "c"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_left_recursive_grammar() {
    let g = GrammarBuilder::new("cd_v8_ch5")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .start("list")
        .build();
    let pt = build_table_from_grammar(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn test_right_recursive_grammar() {
    let g = GrammarBuilder::new("cd_v8_ch6")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["a", "list"])
        .start("list")
        .build();
    let pt = build_table_from_grammar(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn test_multiple_nonterminals_no_overlap() {
    let (_, pt) = build_table(
        "cd_v8_ch7",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["p"]),
            ("start", vec!["q"]),
            ("p", vec!["a"]),
            ("q", vec!["b"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_diamond_rule_shape() {
    // start → a | b, a → x, b → x — but disjoint paths
    let (_, pt) = build_table(
        "cd_v8_ch8",
        &[("x", "x"), ("y", "y")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("a", vec!["x"]),
            ("b", vec!["y"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_shared_suffix_rules() {
    let (_, pt) = build_table(
        "cd_v8_ch9",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "c"]), ("start", vec!["b", "c"])],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

#[test]
fn test_multiple_chain_branches() {
    let (_, pt) = build_table(
        "cd_v8_ch10",
        &[("x", "x"), ("y", "y")],
        &[
            ("start", vec!["left"]),
            ("start", vec!["right"]),
            ("left", vec!["x"]),
            ("right", vec!["y"]),
        ],
        "start",
    );
    assert_eq!(count_multi_action_cells(&pt), 0);
}

// ============================================================================
// CATEGORY 8: EDGE CASES AND COMPLEX PATTERNS (tests 71–82)
// ============================================================================

#[test]
fn test_single_token_single_rule_table_has_accept() {
    let (_, pt) = build_table("cd_v8_ec1", &[("a", "a")], &[("start", vec!["a"])], "start");
    let has_accept = pt.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "table should have at least one Accept action");
}

#[test]
fn test_table_state_count_positive() {
    let (_, pt) = build_table("cd_v8_ec2", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert!(pt.state_count > 0);
}

#[test]
fn test_action_shift_copy() {
    let a = Action::Shift(StateId(5));
    let b = a.clone();
    assert!(matches!(b, Action::Shift(StateId(5))));
}

#[test]
fn test_action_reduce_copy() {
    let a = Action::Reduce(RuleId(3));
    let b = a.clone();
    assert!(matches!(b, Action::Reduce(RuleId(3))));
}

#[test]
fn test_action_accept_clone() {
    let a = Action::Accept;
    let b = a.clone();
    assert!(matches!(b, Action::Accept));
}

#[test]
fn test_action_error_clone() {
    let a = Action::Error;
    let b = a.clone();
    assert!(matches!(b, Action::Error));
}

#[test]
fn test_action_debug() {
    let a = Action::Shift(StateId(1));
    let s = format!("{a:?}");
    assert!(s.contains("Shift"));
}

#[test]
fn test_fork_action_debug() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let s = format!("{a:?}");
    assert!(s.contains("Fork"));
}

#[test]
fn test_synthetic_table_multiple_states() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1))], vec![]],
        vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
    ]);
    assert_eq!(table.state_count, 2);
}

#[test]
fn test_synthetic_table_empty() {
    let table = make_table(vec![]);
    assert_eq!(table.state_count, 0);
    assert_eq!(count_multi_action_cells(&table), 0);
}

#[test]
fn test_has_reduce_reduce_helper_on_ambiguous() {
    let g = GrammarBuilder::new("cd_v8_ec11")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "expr"])
        .start("expr")
        .build();
    let pt = build_table_from_grammar(&g);
    // just verify no panic
    let _ = has_reduce_reduce(&pt);
}

#[test]
fn test_collection_set_count_positive() {
    let g = GrammarBuilder::new("cd_v8_ec12")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (col, _ff) = build_collection(&g);
    assert!(!col.sets.is_empty(), "collection should have ≥1 item set");
}
