//! Conflict detection and resolution tests v3 for GLR core.
//!
//! Covers: simple conflict-free grammars, shift-reduce detection, reduce-reduce
//! detection, precedence resolution, left/right associativity, and conflict
//! counting across states.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_detection_v3

use adze_glr_core::conflict_inspection::count_conflicts;
use adze_glr_core::{
    Action, ConflictResolver, ConflictType as CRConflictType, FirstFollowSets, ItemSetCollection,
    build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a parse table from a grammar, returning it alongside the grammar.
fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).unwrap();
    build_lr1_automaton(grammar, &ff).unwrap()
}

/// Count total multi-action cells (conflicts) in the parse table.
fn total_conflicts(table: &adze_glr_core::ParseTable) -> usize {
    let summary = count_conflicts(table);
    summary.shift_reduce + summary.reduce_reduce
}

/// Check whether any cell in the table has more than one action.
fn has_any_multi_action(table: &adze_glr_core::ParseTable) -> bool {
    table
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| cell.len() > 1))
}

/// Check whether any cell contains a Shift alongside a Reduce.
fn has_shift_reduce(table: &adze_glr_core::ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            let shifts = cell.iter().any(|a| matches!(a, Action::Shift(_)));
            let reduces = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
            shifts && reduces
        })
    })
}

/// Detect R/R conflicts using the item-set-level ConflictResolver.
/// The final parse table resolves R/R by picking the earlier rule, so we
/// must inspect item sets directly.
fn detect_rr_conflicts(grammar: &Grammar) -> Vec<adze_glr_core::Conflict> {
    let ff = FirstFollowSets::compute(grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, grammar, &ff);
    resolver
        .conflicts
        .into_iter()
        .filter(|c| c.conflict_type == CRConflictType::ReduceReduce)
        .collect()
}

// ===========================================================================
// 1. No conflicts in simple grammars (8 tests)
// ===========================================================================

/// S → a — trivially unambiguous single-token grammar.
#[test]
fn test_no_conflict_single_token() {
    let g = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// S → a | b — disjoint alternatives, no overlap.
#[test]
fn test_no_conflict_disjoint_alternatives() {
    let g = GrammarBuilder::new("disj")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// S → a b — simple concatenation, deterministic.
#[test]
fn test_no_conflict_concatenation() {
    let g = GrammarBuilder::new("concat")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// S → a b | a c — common prefix but second token resolves.
#[test]
fn test_no_conflict_common_prefix_resolved() {
    let g = GrammarBuilder::new("prefix")
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

/// S → A; A → x — indirect single production.
#[test]
fn test_no_conflict_indirect_single_production() {
    let g = GrammarBuilder::new("indirect")
        .token("x", "x")
        .rule("s", vec!["item"])
        .rule("item", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// L → L a | a — left-recursive list, deterministic.
#[test]
fn test_no_conflict_left_recursive_list() {
    let g = GrammarBuilder::new("lr_list")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// S → a b c d — longer deterministic chain.
#[test]
fn test_no_conflict_long_chain() {
    let g = GrammarBuilder::new("chain")
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

/// S → A | B; A → x y; B → z — fully disjoint multi-rule grammar.
#[test]
fn test_no_conflict_multiple_nonterminals_disjoint() {
    let g = GrammarBuilder::new("multi_nt")
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

// ===========================================================================
// 2. Shift-reduce conflicts detected (8 tests)
// ===========================================================================

/// E → E + E | num — classic ambiguous expression grammar.
#[test]
fn test_sr_classic_expr() {
    let g = GrammarBuilder::new("sr_expr")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_shift_reduce(&table), "E→E+E|num must have S/R conflict");
}

/// E → E * E | num — multiplication variant.
#[test]
fn test_sr_multiplication_expr() {
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

/// E → E + E | E * E | num — two operators, no precedence.
#[test]
fn test_sr_two_ops_no_prec() {
    let g = GrammarBuilder::new("sr_two")
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

/// E → E - E | num — subtraction is not associative by default.
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

/// S → if E then S else S | if E then S | a (dangling-else).
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
    assert!(
        has_shift_reduce(&table),
        "Dangling-else must produce S/R conflict"
    );
}

/// E → E op E | ( E ) | id — parenthesized expressions still produce S/R.
#[test]
fn test_sr_parens_still_ambiguous() {
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

/// E → E + E | E - E | E * E | num — three ops without precedence.
#[test]
fn test_sr_three_ops() {
    let g = GrammarBuilder::new("sr_three")
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

/// E → E + E | E + E + E | num — overlapping lengths.
#[test]
fn test_sr_overlapping_lengths() {
    let g = GrammarBuilder::new("sr_overlap")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "+", "expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_any_multi_action(&table));
}

// ===========================================================================
// 3. Reduce-reduce conflicts detected (7 tests)
//
// The LR(1) table builder resolves R/R conflicts by preferring the earlier
// rule, so R/R does NOT appear as multi-action cells. We detect R/R via
// ConflictResolver::detect_conflicts on item sets.
// ===========================================================================

/// S → A | B; A → x; B → x — classic R/R: same token, two nonterminals.
#[test]
fn test_rr_classic_same_token() {
    let g = GrammarBuilder::new("rr_classic")
        .token("x", "x")
        .rule("s", vec!["branch_a"])
        .rule("s", vec!["branch_b"])
        .rule("branch_a", vec!["x"])
        .rule("branch_b", vec!["x"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(
        !rr.is_empty(),
        "S→A|B with A→x, B→x must produce R/R conflict"
    );
}

/// S → A y | B y; A → x; B → x — R/R on same prefix followed by same suffix.
#[test]
fn test_rr_same_prefix_same_suffix() {
    let g = GrammarBuilder::new("rr_suffix")
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

/// S → A z | B z; A → x y; B → x y — longer overlapping rules.
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

/// S → A | B | C; A → x; B → x; C → x — three-way reduce conflict.
#[test]
fn test_rr_three_way() {
    let g = GrammarBuilder::new("rr_three")
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

/// S → A w | B w; A → x | y; B → x | z — partial overlap produces R/R on x.
#[test]
fn test_rr_partial_overlap() {
    let g = GrammarBuilder::new("rr_partial")
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

/// S → E; E → A | B; A → num; B → num — deeper nesting still R/R.
#[test]
fn test_rr_deep_nesting() {
    let g = GrammarBuilder::new("rr_deep")
        .token("num", r"\d+")
        .rule("s", vec!["wrapper"])
        .rule("wrapper", vec!["branch_a"])
        .rule("wrapper", vec!["branch_b"])
        .rule("branch_a", vec!["num"])
        .rule("branch_b", vec!["num"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

/// S → A end | B end; A → x y; B → x y — identical RHS means inevitable R/R.
#[test]
fn test_rr_identical_rhs_different_nonterminals() {
    let g = GrammarBuilder::new("rr_ident")
        .token("x", "x")
        .token("y", "y")
        .token("end", "$")
        .rule("s", vec!["branch_a", "end"])
        .rule("s", vec!["branch_b", "end"])
        .rule("branch_a", vec!["x", "y"])
        .rule("branch_b", vec!["x", "y"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty());
}

// ===========================================================================
// 4. Precedence resolves conflicts (8 tests)
// ===========================================================================

/// Adding left-assoc prec to E→E+E|num eliminates multi-action cells.
#[test]
fn test_prec_addition_left_resolves() {
    let g = GrammarBuilder::new("prec_add")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(
        !has_any_multi_action(&table),
        "Precedence should resolve all conflicts"
    );
}

/// E→E+E prec=1 left, E→E*E prec=2 left — no conflicts.
#[test]
fn test_prec_two_ops_resolved() {
    let g = GrammarBuilder::new("prec_two")
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

/// Without prec: conflicts. With prec: fewer or no conflicts.
#[test]
fn test_prec_reduces_conflict_count() {
    let no_prec = GrammarBuilder::new("no_prec")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let with_prec = GrammarBuilder::new("with_prec")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let t1 = build_table(&no_prec);
    let t2 = build_table(&with_prec);
    assert!(total_conflicts(&t1) > total_conflicts(&t2));
}

/// E→E+E prec=1 left, E→E*E prec=2 left, E→E-E prec=1 left — all resolved.
#[test]
fn test_prec_three_ops_same_and_different_levels() {
    let g = GrammarBuilder::new("prec_three")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// High-prec op has no conflict even if low-prec op would.
#[test]
fn test_prec_high_beats_low() {
    let g = GrammarBuilder::new("prec_high")
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

/// Right-assoc prec also resolves S/R conflicts.
#[test]
fn test_prec_right_assoc_resolves() {
    let g = GrammarBuilder::new("prec_right")
        .token("num", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

/// Four ops with distinct prec levels → no conflicts.
#[test]
fn test_prec_four_ops_fully_resolved() {
    let g = GrammarBuilder::new("prec_four")
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

/// Mixed assoc: + left prec=1, ^ right prec=3, * left prec=2 → resolved.
#[test]
fn test_prec_mixed_assoc_resolved() {
    let g = GrammarBuilder::new("prec_mix")
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

// ===========================================================================
// 5. Left associativity (8 tests)
// ===========================================================================

/// Left-assoc + means `a+b+c` parses as `(a+b)+c` → reduce wins on +.
#[test]
fn test_left_assoc_addition() {
    let g = GrammarBuilder::new("left_add")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

/// Left-assoc * at same prec level → no S/R.
#[test]
fn test_left_assoc_multiplication() {
    let g = GrammarBuilder::new("left_mul")
        .token("num", r"\d+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

/// Left-assoc - → reduce on -.
#[test]
fn test_left_assoc_subtraction() {
    let g = GrammarBuilder::new("left_sub")
        .token("num", r"\d+")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

/// Two left-assoc ops at same prec → no conflicts between them.
#[test]
fn test_left_assoc_two_ops_same_prec() {
    let g = GrammarBuilder::new("left_two")
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

/// Left-assoc with parens still works.
#[test]
fn test_left_assoc_with_parens() {
    let g = GrammarBuilder::new("left_paren")
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

/// Left-assoc resolves: without it there are conflicts, with it there are none.
#[test]
fn test_left_assoc_resolves_vs_no_assoc() {
    let without = GrammarBuilder::new("no_assoc")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let with_left = GrammarBuilder::new("with_left")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    assert!(has_shift_reduce(&build_table(&without)));
    assert!(!has_shift_reduce(&build_table(&with_left)));
}

/// Left-assoc at different prec levels: + prec=1 left, * prec=2 left.
#[test]
fn test_left_assoc_different_prec_levels() {
    let g = GrammarBuilder::new("left_diff")
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

/// Left-assoc / at prec=2 works alongside + at prec=1.
#[test]
fn test_left_assoc_division() {
    let g = GrammarBuilder::new("left_div")
        .token("num", r"\d+")
        .token("+", "+")
        .token("/", "/")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_any_multi_action(&table));
}

// ===========================================================================
// 6. Right associativity (7 tests)
// ===========================================================================

/// Right-assoc ^ means `a^b^c` parses as `a^(b^c)` → shift wins on ^.
#[test]
fn test_right_assoc_exponentiation() {
    let g = GrammarBuilder::new("right_exp")
        .token("num", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

/// Right-assoc = (assignment) resolves S/R.
#[test]
fn test_right_assoc_assignment() {
    let g = GrammarBuilder::new("right_assign")
        .token("id", "x")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["id"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

/// Right-assoc resolves: without it conflicts, with it none.
#[test]
fn test_right_assoc_resolves_vs_no_assoc() {
    let without = GrammarBuilder::new("no_a")
        .token("num", r"\d+")
        .token("^", "^")
        .rule("expr", vec!["expr", "^", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let with_right = GrammarBuilder::new("with_r")
        .token("num", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    assert!(has_shift_reduce(&build_table(&without)));
    assert!(!has_shift_reduce(&build_table(&with_right)));
}

/// Right-assoc with higher prec coexists with left-assoc lower prec.
#[test]
fn test_right_assoc_coexists_with_left() {
    let g = GrammarBuilder::new("rl_mix")
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

/// Two right-assoc ops at same prec level.
#[test]
fn test_right_assoc_two_ops_same_prec() {
    let g = GrammarBuilder::new("right_two")
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

/// Right-assoc with parens.
#[test]
fn test_right_assoc_with_parens() {
    let g = GrammarBuilder::new("right_p")
        .token("num", r"\d+")
        .token("^", "^")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(!has_shift_reduce(&table));
}

/// Right-assoc at prec=1, left-assoc at prec=2 — both resolve.
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
// 7. Conflict counting (9 tests)
// ===========================================================================

/// Conflict-free grammar has zero conflicts.
#[test]
fn test_count_zero_conflicts() {
    let g = GrammarBuilder::new("cnt_zero")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert_eq!(total_conflicts(&table), 0);
}

/// Single ambiguous op has at least 1 conflict.
#[test]
fn test_count_single_op_positive() {
    let g = GrammarBuilder::new("cnt_one")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(total_conflicts(&table) > 0);
}

/// Two ambiguous ops have more conflicts than one.
#[test]
fn test_count_two_ops_more_than_one() {
    let one_op = GrammarBuilder::new("one_op")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let two_ops = GrammarBuilder::new("two_ops")
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
    assert!(
        c2 > c1,
        "Two ops ({c2}) should have more conflicts than one ({c1})"
    );
}

/// Precedence drops conflict count to zero.
#[test]
fn test_count_prec_drops_to_zero() {
    let g = GrammarBuilder::new("cnt_prec")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert_eq!(total_conflicts(&table), 0);
}

/// Dangling-else has at least 1 S/R conflict.
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

/// states_with_conflicts is non-empty for ambiguous grammars.
#[test]
fn test_count_states_with_conflicts_nonempty() {
    let g = GrammarBuilder::new("cnt_states")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert!(
        !summary.states_with_conflicts.is_empty(),
        "Ambiguous grammar must have states with conflicts"
    );
}

/// states_with_conflicts is empty for conflict-free grammar.
#[test]
fn test_count_states_with_conflicts_empty_when_clean() {
    let g = GrammarBuilder::new("cnt_clean")
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

/// R/R-only grammar: ConflictResolver detects R/R but table resolves it.
#[test]
fn test_count_rr_only() {
    let g = GrammarBuilder::new("cnt_rr")
        .token("x", "x")
        .rule("s", vec!["branch_a"])
        .rule("s", vec!["branch_b"])
        .rule("branch_a", vec!["x"])
        .rule("branch_b", vec!["x"])
        .start("s")
        .build();
    let rr = detect_rr_conflicts(&g);
    assert!(!rr.is_empty(), "Must have R/R conflicts at item-set level");
    // The final table resolves R/R, so table-level S/R count is 0
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "Must have no S/R conflicts");
}

/// conflict_details has entries matching total count.
#[test]
fn test_count_details_match_totals() {
    let g = GrammarBuilder::new("cnt_detail")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let summary = count_conflicts(&table);
    let total_from_counts = summary.shift_reduce + summary.reduce_reduce;
    assert_eq!(
        summary.conflict_details.len(),
        total_from_counts,
        "Detail entries must match SR + RR counts"
    );
}
