//! Conflict detection and resolution tests v4 for GLR core.
//!
//! Covers: conflict-free grammars, shift-reduce detection, reduce-reduce
//! detection, precedence resolution, associativity, conflict counting,
//! conflict classification, edge cases, and multi-operator interactions.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_detection_v4 -- --test-threads=2

use adze_glr_core::conflict_inspection::{
    classify_conflict, count_conflicts, find_conflicts_for_symbol, get_state_conflicts,
    state_has_conflicts, ConflictType as CIConflictType,
};
use adze_glr_core::{
    Action, ConflictResolver, ConflictType, FirstFollowSets, GotoIndexing, ItemSetCollection,
    ParseTable, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a parse table from a grammar.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).unwrap();
    build_lr1_automaton(grammar, &ff).unwrap()
}

/// Count total multi-action cells (conflicts) in the parse table.
fn total_conflicts(table: &ParseTable) -> usize {
    let summary = count_conflicts(table);
    summary.shift_reduce + summary.reduce_reduce
}

/// Check whether any cell in the table has more than one action.
fn has_any_multi_action(table: &ParseTable) -> bool {
    table
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| cell.len() > 1))
}

/// Check whether any cell contains a Shift alongside a Reduce.
fn has_shift_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            let shifts = cell.iter().any(|a| matches!(a, Action::Shift(_)));
            let reduces = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
            shifts && reduces
        })
    })
}

/// Detect R/R conflicts via ConflictResolver on item sets.
fn detect_rr_conflicts(grammar: &Grammar) -> Vec<adze_glr_core::Conflict> {
    let ff = FirstFollowSets::compute(grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, grammar, &ff);
    resolver
        .conflicts
        .into_iter()
        .filter(|c| c.conflict_type == ConflictType::ReduceReduce)
        .collect()
}

/// Detect S/R conflicts via ConflictResolver on item sets.
fn detect_sr_conflicts(grammar: &Grammar) -> Vec<adze_glr_core::Conflict> {
    let ff = FirstFollowSets::compute(grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, grammar, &ff);
    resolver
        .conflicts
        .into_iter()
        .filter(|c| c.conflict_type == ConflictType::ShiftReduce)
        .collect()
}

/// Create a minimal ParseTable from an action table for unit-level tests.
fn make_table(action_table: Vec<Vec<Vec<Action>>>) -> ParseTable {
    let state_count = action_table.len();
    ParseTable {
        action_table,
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Grammar::new("test".to_string()),
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

// ===========================================================================
// 1. No conflicts in simple grammars (8 tests)
// ===========================================================================

#[test]
fn test_no_conflict_single_rule_single_token() {
    let g = GrammarBuilder::new("single_tok")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
    assert_eq!(total_conflicts(&table), 0);
}

#[test]
fn test_no_conflict_two_disjoint_tokens() {
    let g = GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_no_conflict_sequence_two_tokens() {
    let g = GrammarBuilder::new("seq2")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_no_conflict_common_prefix_disjoint_suffix() {
    let g = GrammarBuilder::new("pref")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["a", "c"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_no_conflict_indirect_nonterminal() {
    let g = GrammarBuilder::new("indir")
        .token("x", "x")
        .rule("s", vec!["inner"])
        .rule("inner", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_no_conflict_left_recursive_list() {
    let g = GrammarBuilder::new("lrec")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_no_conflict_right_recursive_list() {
    let g = GrammarBuilder::new("rrec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_no_conflict_long_deterministic_chain() {
    let g = GrammarBuilder::new("chain4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

// ===========================================================================
// 2. Shift-reduce conflicts detected (8 tests)
// ===========================================================================

#[test]
fn test_sr_classic_addition() {
    let g = GrammarBuilder::new("sr_add")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table), "E→E+E|num must have S/R conflict");
}

#[test]
fn test_sr_multiplication() {
    let g = GrammarBuilder::new("sr_mul")
        .token("num", r"\d+")
        .token("*", "*")
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table));
}

#[test]
fn test_sr_two_operators_no_prec() {
    let g = GrammarBuilder::new("sr2op")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table));
}

#[test]
fn test_sr_dangling_else() {
    let g = GrammarBuilder::new("dangle")
        .token("if_kw", "if")
        .token("then_kw", "then")
        .token("else_kw", "else")
        .token("cond", "c")
        .token("a", "a")
        .rule("s", vec!["if_kw", "cond", "then_kw", "s", "else_kw", "s"])
        .rule("s", vec!["if_kw", "cond", "then_kw", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table), "Dangling-else must produce S/R conflict");
}

#[test]
fn test_sr_subtraction() {
    let g = GrammarBuilder::new("sr_sub")
        .token("num", r"\d+")
        .token("-", "-")
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table));
}

#[test]
fn test_sr_parenthesized_still_ambiguous() {
    let g = GrammarBuilder::new("sr_paren")
        .token("id", "x")
        .token("op", "+")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "op", "expr"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["id"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table));
}

#[test]
fn test_sr_three_operators_no_prec() {
    let g = GrammarBuilder::new("sr3op")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table));
}

#[test]
fn test_sr_detected_via_item_sets() {
    let g = GrammarBuilder::new("sr_items")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let conflicts = detect_sr_conflicts(&g);
    assert!(!conflicts.is_empty(), "Item-set level must detect S/R");
}

// ===========================================================================
// 3. Reduce-reduce conflicts detected (7 tests)
// ===========================================================================

#[test]
fn test_rr_classic_same_token_two_nonterminals() {
    let g = GrammarBuilder::new("rr_cls")
        .token("x", "x")
        .rule("s", vec!["branch_a"])
        .rule("s", vec!["branch_b"])
        .rule("branch_a", vec!["x"])
        .rule("branch_b", vec!["x"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty(), "S→A|B with A→x, B→x must produce R/R");
}

#[test]
fn test_rr_same_prefix_same_suffix() {
    let g = GrammarBuilder::new("rr_suf")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["branch_a", "y"])
        .rule("s", vec!["branch_b", "y"])
        .rule("branch_a", vec!["x"])
        .rule("branch_b", vec!["x"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

#[test]
fn test_rr_longer_overlap() {
    let g = GrammarBuilder::new("rr_long")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["branch_a", "z"])
        .rule("s", vec!["branch_b", "z"])
        .rule("branch_a", vec!["x", "y"])
        .rule("branch_b", vec!["x", "y"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

#[test]
fn test_rr_three_way_conflict() {
    let g = GrammarBuilder::new("rr_3way")
        .token("x", "x")
        .rule("s", vec!["branch_a"])
        .rule("s", vec!["branch_b"])
        .rule("s", vec!["branch_c"])
        .rule("branch_a", vec!["x"])
        .rule("branch_b", vec!["x"])
        .rule("branch_c", vec!["x"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

#[test]
fn test_rr_partial_token_overlap() {
    let g = GrammarBuilder::new("rr_part")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .token("w", "w")
        .rule("s", vec!["branch_a", "w"])
        .rule("s", vec!["branch_b", "w"])
        .rule("branch_a", vec!["x"])
        .rule("branch_a", vec!["y"])
        .rule("branch_b", vec!["x"])
        .rule("branch_b", vec!["z"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

#[test]
fn test_rr_deep_nesting() {
    let g = GrammarBuilder::new("rr_deep")
        .token("num", r"\d+")
        .rule("s", vec!["wrap"])
        .rule("wrap", vec!["branch_a"])
        .rule("wrap", vec!["branch_b"])
        .rule("branch_a", vec!["num"])
        .rule("branch_b", vec!["num"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

#[test]
fn test_rr_table_resolves_but_item_sets_detect() {
    let g = GrammarBuilder::new("rr_resolve")
        .token("x", "x")
        .rule("s", vec!["branch_a"])
        .rule("s", vec!["branch_b"])
        .rule("branch_a", vec!["x"])
        .rule("branch_b", vec!["x"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty(), "Item sets must detect R/R");
    // But the final table resolves R/R by picking the earlier rule
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
}

// ===========================================================================
// 4. Precedence resolves conflicts (7 tests)
// ===========================================================================

#[test]
fn test_prec_left_assoc_addition_resolves() {
    let g = GrammarBuilder::new("prec_add")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_prec_two_ops_different_levels() {
    let g = GrammarBuilder::new("prec_2lvl")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_prec_reduces_conflict_count_vs_no_prec() {
    let no_prec = GrammarBuilder::new("np")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let with_prec = GrammarBuilder::new("wp")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let c1 = total_conflicts(&build_table(&no_prec));
    let c2 = total_conflicts(&build_table(&with_prec));
    assert!(c1 > c2, "Prec should reduce conflict count: {c1} vs {c2}");
}

#[test]
fn test_prec_three_ops_all_resolved() {
    let g = GrammarBuilder::new("prec_3op")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_prec_high_level_beats_low() {
    let g = GrammarBuilder::new("prec_hi")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 10, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_prec_right_assoc_resolves() {
    let g = GrammarBuilder::new("prec_rt")
        .token("num", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_prec_four_ops_fully_resolved() {
    let g = GrammarBuilder::new("prec_4op")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

// ===========================================================================
// 5. Left associativity (6 tests)
// ===========================================================================

#[test]
fn test_left_assoc_eliminates_sr_on_plus() {
    let g = GrammarBuilder::new("la_plus")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn test_left_assoc_eliminates_sr_on_minus() {
    let g = GrammarBuilder::new("la_min")
        .token("num", r"\d+")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn test_left_assoc_two_ops_same_prec() {
    let g = GrammarBuilder::new("la_2op")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_left_assoc_with_parentheses() {
    let g = GrammarBuilder::new("la_paren")
        .token("num", r"\d+")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn test_left_assoc_resolves_vs_no_assoc() {
    let without = GrammarBuilder::new("no_la")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let with_left = GrammarBuilder::new("with_la")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    assert!(has_shift_reduce(&build_table(&without)));
    assert!(!has_shift_reduce(&build_table(&with_left)));
}

#[test]
fn test_left_assoc_different_prec_levels() {
    let g = GrammarBuilder::new("la_diff")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

// ===========================================================================
// 6. Right associativity (6 tests)
// ===========================================================================

#[test]
fn test_right_assoc_exponentiation() {
    let g = GrammarBuilder::new("ra_exp")
        .token("num", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn test_right_assoc_assignment() {
    let g = GrammarBuilder::new("ra_asgn")
        .token("id", "x")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["id"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn test_right_assoc_resolves_vs_no_assoc() {
    let without = GrammarBuilder::new("no_ra")
        .token("num", r"\d+")
        .token("^", "^")
        .rule("expr", vec!["expr", "^", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let with_right = GrammarBuilder::new("with_ra")
        .token("num", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    assert!(has_shift_reduce(&build_table(&without)));
    assert!(!has_shift_reduce(&build_table(&with_right)));
}

#[test]
fn test_right_assoc_coexists_with_left() {
    let g = GrammarBuilder::new("ra_la")
        .token("num", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_right_assoc_two_ops_same_prec() {
    let g = GrammarBuilder::new("ra_2op")
        .token("num", r"\d+")
        .token("^", "^")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 5, Associativity::Right)
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 5, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_right_low_left_high() {
    let g = GrammarBuilder::new("rl_inv")
        .token("num", r"\d+")
        .token("=", "=")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::Right)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

// ===========================================================================
// 7. Conflict counting (8 tests)
// ===========================================================================

#[test]
fn test_count_zero_for_clean_grammar() {
    let g = GrammarBuilder::new("cnt0")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert_eq!(total_conflicts(&table), 0);
}

#[test]
fn test_count_positive_for_ambiguous_expr() {
    let g = GrammarBuilder::new("cnt1")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(total_conflicts(&table) > 0);
}

#[test]
fn test_count_two_ops_more_than_one() {
    let one_op = GrammarBuilder::new("1op")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let two_ops = GrammarBuilder::new("2op")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let c1 = total_conflicts(&build_table(&one_op));
    let c2 = total_conflicts(&build_table(&two_ops));
    assert!(c2 > c1, "Two ops ({c2}) should have more conflicts than one ({c1})");
}

#[test]
fn test_count_prec_drops_to_zero() {
    let g = GrammarBuilder::new("cnt_p0")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert_eq!(total_conflicts(&table), 0);
}

#[test]
fn test_count_dangling_else_positive() {
    let g = GrammarBuilder::new("cnt_de")
        .token("if_kw", "if")
        .token("then_kw", "then")
        .token("else_kw", "else")
        .token("cond", "c")
        .token("a", "a")
        .rule("s", vec!["if_kw", "cond", "then_kw", "s", "else_kw", "s"])
        .rule("s", vec!["if_kw", "cond", "then_kw", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert!(summary.shift_reduce > 0);
}

#[test]
fn test_count_states_with_conflicts_nonempty() {
    let g = GrammarBuilder::new("cnt_st")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert!(!summary.states_with_conflicts.is_empty());
}

#[test]
fn test_count_states_with_conflicts_empty_when_clean() {
    let g = GrammarBuilder::new("cnt_cl")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert!(summary.states_with_conflicts.is_empty());
}

#[test]
fn test_count_details_match_totals() {
    let g = GrammarBuilder::new("cnt_det")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    let total = summary.shift_reduce + summary.reduce_reduce;
    assert_eq!(summary.conflict_details.len(), total);
}

// ===========================================================================
// 8. Conflict classification on synthetic tables (6 tests)
// ===========================================================================

#[test]
fn test_classify_shift_reduce_actions() {
    let actions = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(3))];
    assert_eq!(classify_conflict(&actions), CIConflictType::ShiftReduce);
}

#[test]
fn test_classify_reduce_reduce_actions() {
    let actions = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    assert_eq!(classify_conflict(&actions), CIConflictType::ReduceReduce);
}

#[test]
fn test_classify_double_shift_is_mixed() {
    let actions = vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))];
    assert_eq!(classify_conflict(&actions), CIConflictType::Mixed);
}

#[test]
fn test_classify_fork_with_shift_reduce() {
    let actions = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
    ])];
    assert_eq!(classify_conflict(&actions), CIConflictType::ShiftReduce);
}

#[test]
fn test_classify_fork_with_two_reduces() {
    let actions = vec![Action::Fork(vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
    ])];
    assert_eq!(classify_conflict(&actions), CIConflictType::ReduceReduce);
}

#[test]
fn test_classify_accept_only_is_mixed() {
    let actions = vec![Action::Accept, Action::Accept];
    assert_eq!(classify_conflict(&actions), CIConflictType::Mixed);
}

// ===========================================================================
// 9. Synthetic table state-level inspection (5 tests)
// ===========================================================================

#[test]
fn test_state_has_conflicts_synthetic_true() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    assert!(state_has_conflicts(&table, StateId(0)));
}

#[test]
fn test_state_has_conflicts_synthetic_false() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    assert!(!state_has_conflicts(&table, StateId(0)));
}

#[test]
fn test_state_has_conflicts_out_of_bounds() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    assert!(!state_has_conflicts(&table, StateId(99)));
}

#[test]
fn test_get_state_conflicts_returns_details() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]]);
    let details = get_state_conflicts(&table, StateId(0));
    assert!(!details.is_empty());
    assert_eq!(details[0].conflict_type, CIConflictType::ShiftReduce);
}

#[test]
fn test_find_conflicts_for_symbol_empty_table() {
    let table = make_table(vec![vec![vec![Action::Shift(StateId(1))]]]);
    let details = find_conflicts_for_symbol(&table, SymbolId(42));
    assert!(details.is_empty());
}

// ===========================================================================
// 10. Mixed associativity and edge cases (7 tests)
// ===========================================================================

#[test]
fn test_mixed_left_right_at_different_prec() {
    let g = GrammarBuilder::new("mix_lr")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

#[test]
fn test_conflict_summary_display_nonempty() {
    let g = GrammarBuilder::new("disp")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    let display = format!("{summary}");
    assert!(display.contains("Shift/Reduce"));
    assert!(display.contains("Conflict"));
}

#[test]
fn test_empty_action_cell_is_not_conflict() {
    let table = make_table(vec![vec![vec![]]]);
    assert_eq!(total_conflicts(&table), 0);
}

#[test]
fn test_single_action_cell_is_not_conflict() {
    let table = make_table(vec![vec![vec![Action::Reduce(RuleId(0))]]]);
    assert_eq!(total_conflicts(&table), 0);
}

#[test]
fn test_multiple_states_only_one_conflicting() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![
            Action::Shift(StateId(2)),
            Action::Reduce(RuleId(0)),
        ]],
        vec![vec![Action::Accept]],
    ]);
    let summary = count_conflicts(&table);
    assert_eq!(summary.states_with_conflicts.len(), 1);
    assert_eq!(summary.states_with_conflicts[0], StateId(1));
}

#[test]
fn test_sr_conflict_cell_has_both_action_types() {
    let g = GrammarBuilder::new("sr_types")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    for detail in &summary.conflict_details {
        if detail.conflict_type == CIConflictType::ShiftReduce {
            let has_s = detail.actions.iter().any(|a| matches!(a, Action::Shift(_)));
            let has_r = detail.actions.iter().any(|a| matches!(a, Action::Reduce(_)));
            assert!(has_s && has_r, "S/R detail must have both shift and reduce");
        }
    }
}

#[test]
fn test_disjoint_nonterminals_no_conflict() {
    let g = GrammarBuilder::new("disj_nt")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["branch_a"])
        .rule("s", vec!["branch_b"])
        .rule("branch_a", vec!["x", "y"])
        .rule("branch_b", vec!["z"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}
