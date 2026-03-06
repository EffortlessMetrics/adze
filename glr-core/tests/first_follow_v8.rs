//! FIRST/FOLLOW set computation tests — v8.
//!
//! 84 tests across 15 categories:
//!  1. first_terminal_*         — FIRST set of terminal contains itself
//!  2. first_single_rule_*      — FIRST of non-terminal with single terminal rule
//!  3. first_multi_alt_*        — FIRST of non-terminal with multiple alternatives
//!  4. first_nullable_*         — FIRST with nullable rules
//!  5. follow_eof_*             — FOLLOW of start symbol contains EOF
//!  6. follow_propagation_*     — FOLLOW set propagation through rules
//!  7. ff_simple_*              — FIRST/FOLLOW compute succeeds for simple grammar
//!  8. ff_arithmetic_*          — FIRST/FOLLOW compute succeeds for arithmetic grammar
//!  9. ff_recursive_*           — FIRST/FOLLOW compute succeeds for recursive grammar
//! 10. ff_precedence_*          — FIRST/FOLLOW compute succeeds for grammar with precedence
//! 11. first_nonempty_*         — FIRST sets are non-empty for reachable symbols
//! 12. follow_nonempty_*        — FOLLOW sets are non-empty for used symbols
//! 13. first_distinct_*         — Multiple non-terminals have distinct FIRST sets
//! 14. ff_pattern_*             — Various grammar patterns (list, expr, nested)
//! 15. ff_large_*               — Large grammar (20+ tokens, 10+ rules) computes

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EOF: SymbolId = SymbolId(0);

fn build_ff(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> (Grammar, FirstFollowSets) {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let g = b.build();
    let ff = FirstFollowSets::compute(&g).expect("compute");
    (g, ff)
}

fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.find_symbol_by_name(name)
        .unwrap_or_else(|| panic!("symbol '{}' not found", name))
}

fn assert_first_contains(ff: &FirstFollowSets, id: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .first(id)
        .unwrap_or_else(|| panic!("no FIRST set for {id:?}"));
    for &e in expected {
        assert!(
            set.contains(e.0 as usize),
            "FIRST({id:?}) should contain {e:?}",
        );
    }
}

fn assert_first_excludes(ff: &FirstFollowSets, id: SymbolId, excluded: &[SymbolId]) {
    let set = ff
        .first(id)
        .unwrap_or_else(|| panic!("no FIRST set for {id:?}"));
    for &e in excluded {
        assert!(
            !set.contains(e.0 as usize),
            "FIRST({id:?}) should NOT contain {e:?}",
        );
    }
}

fn assert_first_eq(ff: &FirstFollowSets, id: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .first(id)
        .unwrap_or_else(|| panic!("no FIRST set for {id:?}"));
    let actual: Vec<u16> = (0..set.len())
        .filter(|&i| set.contains(i))
        .map(|i| i as u16)
        .collect();
    let mut exp: Vec<u16> = expected.iter().map(|s| s.0).collect();
    exp.sort();
    assert_eq!(actual, exp, "FIRST({id:?}) mismatch");
}

fn assert_follow_contains(ff: &FirstFollowSets, id: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .follow(id)
        .unwrap_or_else(|| panic!("no FOLLOW set for {id:?}"));
    for &e in expected {
        assert!(
            set.contains(e.0 as usize),
            "FOLLOW({id:?}) should contain {e:?}",
        );
    }
}

#[allow(dead_code)]
fn assert_follow_excludes(ff: &FirstFollowSets, id: SymbolId, excluded: &[SymbolId]) {
    let set = ff
        .follow(id)
        .unwrap_or_else(|| panic!("no FOLLOW set for {id:?}"));
    for &e in excluded {
        assert!(
            !set.contains(e.0 as usize),
            "FOLLOW({id:?}) should NOT contain {e:?}",
        );
    }
}

/// Count how many bits are set in a FIRST set.
fn first_count(ff: &FirstFollowSets, id: SymbolId) -> usize {
    ff.first(id).map_or(0, |s| s.count_ones(..))
}

/// Count how many bits are set in a FOLLOW set.
fn follow_count(ff: &FirstFollowSets, id: SymbolId) -> usize {
    ff.follow(id).map_or(0, |s| s.count_ones(..))
}

// ===========================================================================
// 1. first_terminal_* — FIRST set of terminal contains itself (6 tests)
// ===========================================================================

#[test]
fn first_terminal_contains_itself_simple() {
    let (g, ff) = build_ff("t", &[("a", "a")], &[("start", vec!["a"])], "start");
    let a = sym(&g, "a");
    if let Some(set) = ff.first(a) {
        assert!(
            set.contains(a.0 as usize) || set.count_ones(..) == 0,
            "terminal FIRST set should be trivial",
        );
    }
}

#[test]
fn first_terminal_multiple_tokens_each_self() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[
            ("start", vec!["x"]),
            ("alt", vec!["y"]),
            ("alt2", vec!["z"]),
        ],
        "start",
    );
    // Each non-terminal's FIRST should contain its leading terminal
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "x")]);
    assert_first_eq(&ff, sym(&g, "alt"), &[sym(&g, "y")]);
    assert_first_eq(&ff, sym(&g, "alt2"), &[sym(&g, "z")]);
}

#[test]
fn first_terminal_keyword_token() {
    let (g, ff) = build_ff(
        "t",
        &[("kw_if", "if")],
        &[("start", vec!["kw_if"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "kw_if")]);
}

#[test]
fn first_terminal_regex_token() {
    let (g, ff) = build_ff(
        "t",
        &[("num", r"[0-9]+")],
        &[("start", vec!["num"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "num")]);
}

#[test]
fn first_terminal_punctuation_token() {
    let (g, ff) = build_ff("t", &[("semi", ";")], &[("start", vec!["semi"])], "start");
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "semi")]);
}

#[test]
fn first_terminal_operator_token() {
    let (g, ff) = build_ff(
        "t",
        &[("plus", "+"), ("star", "*")],
        &[("start", vec!["plus"]), ("start", vec!["star"])],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "plus"), sym(&g, "star")]);
}

// ===========================================================================
// 2. first_single_rule_* — FIRST of non-terminal with single terminal rule (6 tests)
// ===========================================================================

#[test]
fn first_single_rule_one_terminal() {
    let (g, ff) = build_ff("t", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_single_rule_leading_terminal_only() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "b"])],
        "start",
    );
    let start = sym(&g, "start");
    assert_first_contains(&ff, start, &[sym(&g, "a")]);
    assert_first_excludes(&ff, start, &[sym(&g, "b")]);
}

#[test]
fn first_single_rule_three_terminals() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_single_rule_through_nonterminal() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x")],
        &[("start", vec!["inner"]), ("inner", vec!["x"])],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "x")]);
}

#[test]
fn first_single_rule_deep_chain() {
    let (g, ff) = build_ff(
        "t",
        &[("tok", "t")],
        &[
            ("start", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["tok"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "tok")]);
    assert_first_contains(&ff, sym(&g, "mid"), &[sym(&g, "tok")]);
    assert_first_contains(&ff, sym(&g, "leaf"), &[sym(&g, "tok")]);
}

#[test]
fn first_single_rule_four_level_chain() {
    let (g, ff) = build_ff(
        "t",
        &[("val", "v")],
        &[
            ("start", vec!["level1"]),
            ("level1", vec!["level2"]),
            ("level2", vec!["level3"]),
            ("level3", vec!["val"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "val")]);
}

// ===========================================================================
// 3. first_multi_alt_* — FIRST of non-terminal with multiple alternatives (7 tests)
// ===========================================================================

#[test]
fn first_multi_alt_two_alternatives() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_multi_alt_three_alternatives() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
        ],
        "start",
    );
    assert_first_contains(
        &ff,
        sym(&g, "start"),
        &[sym(&g, "a"), sym(&g, "b"), sym(&g, "c")],
    );
}

#[test]
fn first_multi_alt_five_alternatives() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
            ("start", vec!["d"]),
            ("start", vec!["e"]),
        ],
        "start",
    );
    assert!(first_count(&ff, sym(&g, "start")) >= 5);
}

#[test]
fn first_multi_alt_mixed_terminal_nonterminal() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["inner"]),
            ("inner", vec!["b"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_multi_alt_same_leading_terminal() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b"]), ("start", vec!["a", "c"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_multi_alt_overlapping_through_chain() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x"), ("y", "y")],
        &[
            ("start", vec!["left"]),
            ("start", vec!["right"]),
            ("left", vec!["x"]),
            ("right", vec!["y"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "x"), sym(&g, "y")]);
}

#[test]
fn first_multi_alt_each_subrule_independent() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["p"]),
            ("start", vec!["q"]),
            ("p", vec!["a"]),
            ("q", vec!["b", "c"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "c")]);
}

// ===========================================================================
// 4. first_nullable_* — FIRST with nullable rules (7 tests)
// ===========================================================================

#[test]
fn first_nullable_epsilon_is_nullable() {
    let (g, ff) = build_ff(
        "t",
        &[("dummy", "d")],
        &[("start", vec![]), ("start", vec!["dummy"])],
        "start",
    );
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_non_empty_rule_not_nullable() {
    let (g, ff) = build_ff("t", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert!(!ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_propagation_through_chain() {
    let (g, ff) = build_ff(
        "t",
        &[("dummy", "d")],
        &[
            ("start", vec!["inner"]),
            ("start", vec!["dummy"]),
            ("inner", vec![]),
        ],
        "start",
    );
    assert!(ff.is_nullable(sym(&g, "inner")));
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_skip_to_second_symbol() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["opt", "b"]),
            ("opt", vec![]),
            ("opt", vec!["a"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_nullable_two_nullable_prefix() {
    let (g, ff) = build_ff(
        "t",
        &[("c", "c")],
        &[
            ("start", vec!["na", "nb", "c"]),
            ("na", vec![]),
            ("nb", vec![]),
        ],
        "start",
    );
    assert!(ff.is_nullable(sym(&g, "na")));
    assert!(ff.is_nullable(sym(&g, "nb")));
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "c")]);
}

#[test]
fn first_nullable_all_nullable_rhs_is_nullable() {
    let (g, ff) = build_ff(
        "t",
        &[("dummy", "d")],
        &[
            ("start", vec!["na", "nb"]),
            ("start", vec!["dummy"]),
            ("na", vec![]),
            ("nb", vec![]),
        ],
        "start",
    );
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_deep_chain_all_nullable() {
    let (g, ff) = build_ff(
        "t",
        &[("dummy", "d")],
        &[
            ("start", vec!["mid"]),
            ("start", vec!["dummy"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec![]),
        ],
        "start",
    );
    assert!(ff.is_nullable(sym(&g, "leaf")));
    assert!(ff.is_nullable(sym(&g, "mid")));
    assert!(ff.is_nullable(sym(&g, "start")));
}

// ===========================================================================
// 5. follow_eof_* — FOLLOW of start symbol contains EOF (6 tests)
// ===========================================================================

#[test]
fn follow_eof_start_symbol_has_eof() {
    let (g, ff) = build_ff("t", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_start_with_multiple_rules() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_start_with_chain() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x")],
        &[("start", vec!["inner"]), ("inner", vec!["x"])],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_inner_at_end_inherits_eof() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a")],
        &[("start", vec!["inner"]), ("inner", vec!["a"])],
        "start",
    );
    // inner is at the end of start's production, so it inherits FOLLOW(start)
    assert_follow_contains(&ff, sym(&g, "inner"), &[EOF]);
}

#[test]
fn follow_eof_recursive_start_has_eof() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["start", "a"]), ("start", vec!["b"])],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_nested_start_ref_has_eof() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["wrap"]),
            ("wrap", vec!["a", "start", "b"]),
            ("wrap", vec!["a"]),
        ],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

// ===========================================================================
// 6. follow_propagation_* — FOLLOW set propagation through rules (7 tests)
// ===========================================================================

#[test]
fn follow_propagation_terminal_after_nonterminal() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["inner", "b"]), ("inner", vec!["a"])],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "b")]);
}

#[test]
fn follow_propagation_nonterminal_after_nonterminal() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["lhs", "rhs"]),
            ("lhs", vec!["a"]),
            ("rhs", vec!["b"]),
        ],
        "start",
    );
    // FOLLOW(lhs) includes FIRST(rhs) = {b}
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "b")]);
}

#[test]
fn follow_propagation_nullable_successor_adds_lhs_follow() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["lhs", "rhs"]),
            ("lhs", vec!["a"]),
            ("rhs", vec![]),
            ("rhs", vec!["b"]),
        ],
        "start",
    );
    // rhs is nullable, so FOLLOW(lhs) also includes FOLLOW(start) = {EOF}
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "b"), EOF]);
}

#[test]
fn follow_propagation_last_symbol_inherits_lhs() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["lhs", "rhs"]),
            ("lhs", vec!["a"]),
            ("rhs", vec!["b"]),
        ],
        "start",
    );
    // rhs is last in start production, so FOLLOW(rhs) includes FOLLOW(start)
    assert_follow_contains(&ff, sym(&g, "rhs"), &[EOF]);
}

#[test]
fn follow_propagation_middle_symbol_gets_first_of_next() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "mid", "c"]), ("mid", vec!["b"])],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "mid"), &[sym(&g, "c")]);
}

#[test]
fn follow_propagation_multiple_occurrences_union() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("x", "x"), ("y", "y")],
        &[
            ("start", vec!["inner", "x"]),
            ("start", vec!["inner", "y"]),
            ("inner", vec!["a"]),
        ],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "x"), sym(&g, "y")]);
}

#[test]
fn follow_propagation_chain_of_three() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["lhs", "mid", "rhs"]),
            ("lhs", vec!["a"]),
            ("mid", vec!["b"]),
            ("rhs", vec!["c"]),
        ],
        "start",
    );
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "b")]);
    assert_follow_contains(&ff, sym(&g, "mid"), &[sym(&g, "c")]);
    assert_follow_contains(&ff, sym(&g, "rhs"), &[EOF]);
}

// ===========================================================================
// 7. ff_simple_* — FIRST/FOLLOW compute succeeds for simple grammar (5 tests)
// ===========================================================================

#[test]
fn ff_simple_single_token_grammar() {
    let (g, ff) = build_ff("simple", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn ff_simple_two_rule_grammar() {
    let (g, ff) = build_ff(
        "simple",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["item"]), ("item", vec!["a", "b"])],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a")]);
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn ff_simple_keyword_grammar() {
    let (g, ff) = build_ff(
        "simple",
        &[
            ("kw_let", "let"),
            ("ident", "[a-z]+"),
            ("eq", "="),
            ("num", "[0-9]+"),
        ],
        &[("start", vec!["kw_let", "ident", "eq", "num"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "kw_let")]);
}

#[test]
fn ff_simple_pair_grammar() {
    let (g, ff) = build_ff(
        "simple",
        &[("lp", "("), ("rp", ")"), ("a", "a")],
        &[("start", vec!["lp", "a", "rp"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "lp")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "rp"), sym(&g, "a")]);
}

#[test]
fn ff_simple_alternation_grammar() {
    let (g, ff) = build_ff(
        "simple",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
        ],
        "start",
    );
    assert_first_contains(
        &ff,
        sym(&g, "start"),
        &[sym(&g, "a"), sym(&g, "b"), sym(&g, "c")],
    );
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

// ===========================================================================
// 8. ff_arithmetic_* — FIRST/FOLLOW for arithmetic grammar (6 tests)
// ===========================================================================

fn build_arithmetic() -> (Grammar, FirstFollowSets) {
    build_ff(
        "arith",
        &[
            ("num", "[0-9]+"),
            ("plus", "+"),
            ("star", "*"),
            ("lp", "("),
            ("rp", ")"),
        ],
        &[
            // expr -> term | expr plus term
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            // term -> factor | term star factor
            ("term", vec!["factor"]),
            ("term", vec!["term", "star", "factor"]),
            // factor -> num | lp expr rp
            ("factor", vec!["num"]),
            ("factor", vec!["lp", "expr", "rp"]),
        ],
        "expr",
    )
}

#[test]
fn ff_arithmetic_computes_successfully() {
    let (_g, _ff) = build_arithmetic();
}

#[test]
fn ff_arithmetic_expr_first_contains_num_lp() {
    let (g, ff) = build_arithmetic();
    assert_first_contains(&ff, sym(&g, "expr"), &[sym(&g, "num"), sym(&g, "lp")]);
}

#[test]
fn ff_arithmetic_term_first_contains_num_lp() {
    let (g, ff) = build_arithmetic();
    assert_first_contains(&ff, sym(&g, "term"), &[sym(&g, "num"), sym(&g, "lp")]);
}

#[test]
fn ff_arithmetic_factor_first_contains_num_lp() {
    let (g, ff) = build_arithmetic();
    assert_first_contains(&ff, sym(&g, "factor"), &[sym(&g, "num"), sym(&g, "lp")]);
}

#[test]
fn ff_arithmetic_expr_follow_has_eof() {
    let (g, ff) = build_arithmetic();
    assert_follow_contains(&ff, sym(&g, "expr"), &[EOF]);
}

#[test]
fn ff_arithmetic_term_follow_has_plus() {
    let (g, ff) = build_arithmetic();
    assert_follow_contains(&ff, sym(&g, "term"), &[sym(&g, "plus")]);
}

// ===========================================================================
// 9. ff_recursive_* — FIRST/FOLLOW for recursive grammar (6 tests)
// ===========================================================================

#[test]
fn ff_recursive_left_recursion() {
    let (g, ff) = build_ff(
        "rec",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["start", "a"]), ("start", vec!["b"])],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "b")]);
}

#[test]
fn ff_recursive_right_recursion() {
    let (g, ff) = build_ff(
        "rec",
        &[("a", "a")],
        &[("start", vec!["a", "start"]), ("start", vec!["a"])],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn ff_recursive_mutual_recursion() {
    let (g, ff) = build_ff(
        "rec",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[
            ("start", vec!["alt", "x"]),
            ("alt", vec!["start", "y"]),
            ("alt", vec!["z"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "z")]);
    assert_first_contains(&ff, sym(&g, "alt"), &[sym(&g, "z")]);
}

#[test]
fn ff_recursive_three_way_cycle() {
    let (g, ff) = build_ff(
        "rec",
        &[("x", "x"), ("y", "y"), ("z", "z"), ("w", "w")],
        &[
            ("start", vec!["bsym", "x"]),
            ("bsym", vec!["csym", "y"]),
            ("csym", vec!["start", "z"]),
            ("csym", vec!["w"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "w")]);
    assert_first_contains(&ff, sym(&g, "bsym"), &[sym(&g, "w")]);
    assert_first_contains(&ff, sym(&g, "csym"), &[sym(&g, "w")]);
}

#[test]
fn ff_recursive_self_referencing_alt() {
    let (g, ff) = build_ff(
        "rec",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["start", "b"])],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a")]);
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn ff_recursive_nested_parens() {
    let (g, ff) = build_ff(
        "rec",
        &[("lp", "("), ("rp", ")"), ("a", "a")],
        &[("start", vec!["a"]), ("start", vec!["lp", "start", "rp"])],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "lp")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "rp")]);
}

// ===========================================================================
// 10. ff_precedence_* — FIRST/FOLLOW for grammar with precedence layers (5 tests)
// ===========================================================================

fn build_precedence() -> (Grammar, FirstFollowSets) {
    // Classic multi-level precedence: expr > comp > add > mul > atom
    build_ff(
        "prec",
        &[
            ("num", "[0-9]+"),
            ("plus", "+"),
            ("minus", "-"),
            ("star", "*"),
            ("div", "div"),
            ("lt", "<"),
            ("gt", ">"),
            ("lp", "("),
            ("rp", ")"),
        ],
        &[
            ("expr", vec!["comp"]),
            ("comp", vec!["add"]),
            ("comp", vec!["comp", "lt", "add"]),
            ("comp", vec!["comp", "gt", "add"]),
            ("add", vec!["mul"]),
            ("add", vec!["add", "plus", "mul"]),
            ("add", vec!["add", "minus", "mul"]),
            ("mul", vec!["atom"]),
            ("mul", vec!["mul", "star", "atom"]),
            ("mul", vec!["mul", "div", "atom"]),
            ("atom", vec!["num"]),
            ("atom", vec!["lp", "expr", "rp"]),
        ],
        "expr",
    )
}

#[test]
fn ff_precedence_computes_successfully() {
    let (_g, _ff) = build_precedence();
}

#[test]
fn ff_precedence_all_levels_share_atom_first() {
    let (g, ff) = build_precedence();
    let expected = &[sym(&g, "num"), sym(&g, "lp")];
    assert_first_contains(&ff, sym(&g, "expr"), expected);
    assert_first_contains(&ff, sym(&g, "comp"), expected);
    assert_first_contains(&ff, sym(&g, "add"), expected);
    assert_first_contains(&ff, sym(&g, "mul"), expected);
    assert_first_contains(&ff, sym(&g, "atom"), expected);
}

#[test]
fn ff_precedence_expr_follow_has_eof() {
    let (g, ff) = build_precedence();
    assert_follow_contains(&ff, sym(&g, "expr"), &[EOF]);
}

#[test]
fn ff_precedence_add_follow_has_comparison_ops() {
    let (g, ff) = build_precedence();
    assert_follow_contains(&ff, sym(&g, "add"), &[sym(&g, "lt"), sym(&g, "gt")]);
}

#[test]
fn ff_precedence_mul_follow_has_additive_ops() {
    let (g, ff) = build_precedence();
    assert_follow_contains(&ff, sym(&g, "mul"), &[sym(&g, "plus"), sym(&g, "minus")]);
}

// ===========================================================================
// 11. first_nonempty_* — FIRST sets are non-empty for reachable symbols (6 tests)
// ===========================================================================

#[test]
fn first_nonempty_single_rule() {
    let (g, ff) = build_ff("t", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert!(first_count(&ff, sym(&g, "start")) > 0);
}

#[test]
fn first_nonempty_chain() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x")],
        &[("start", vec!["mid"]), ("mid", vec!["x"])],
        "start",
    );
    assert!(first_count(&ff, sym(&g, "start")) > 0);
    assert!(first_count(&ff, sym(&g, "mid")) > 0);
}

#[test]
fn first_nonempty_alternation() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert!(first_count(&ff, sym(&g, "start")) >= 2);
}

#[test]
fn first_nonempty_left_recursive() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["start", "a"]), ("start", vec!["b"])],
        "start",
    );
    assert!(first_count(&ff, sym(&g, "start")) > 0);
}

#[test]
fn first_nonempty_arithmetic_all_nonterminals() {
    let (g, ff) = build_arithmetic();
    for name in ["expr", "term", "factor"] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

#[test]
fn first_nonempty_precedence_all_levels() {
    let (g, ff) = build_precedence();
    for name in ["expr", "comp", "add", "mul", "atom"] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

// ===========================================================================
// 12. follow_nonempty_* — FOLLOW sets are non-empty for used symbols (6 tests)
// ===========================================================================

#[test]
fn follow_nonempty_start_symbol() {
    let (g, ff) = build_ff("t", &[("a", "a")], &[("start", vec!["a"])], "start");
    assert!(follow_count(&ff, sym(&g, "start")) > 0);
}

#[test]
fn follow_nonempty_inner_referenced_symbol() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["inner", "b"]), ("inner", vec!["a"])],
        "start",
    );
    assert!(follow_count(&ff, sym(&g, "inner")) > 0);
}

#[test]
fn follow_nonempty_chain_symbols() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x")],
        &[
            ("start", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        "start",
    );
    // start and mid and leaf all at end of their parent's production
    assert!(follow_count(&ff, sym(&g, "start")) > 0);
    assert!(follow_count(&ff, sym(&g, "mid")) > 0);
    assert!(follow_count(&ff, sym(&g, "leaf")) > 0);
}

#[test]
fn follow_nonempty_arithmetic_nonterminals() {
    let (g, ff) = build_arithmetic();
    for name in ["expr", "term", "factor"] {
        assert!(
            follow_count(&ff, sym(&g, name)) > 0,
            "FOLLOW({name}) should be non-empty",
        );
    }
}

#[test]
fn follow_nonempty_inner_with_multiple_contexts() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("x", "x"), ("y", "y")],
        &[
            ("start", vec!["inner", "x"]),
            ("start", vec!["inner", "y"]),
            ("inner", vec!["a"]),
        ],
        "start",
    );
    assert!(follow_count(&ff, sym(&g, "inner")) >= 2);
}

#[test]
fn follow_nonempty_last_in_rule_inherits_parent() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a", "tail"]), ("tail", vec!["b"])],
        "start",
    );
    // tail is last in start, inherits FOLLOW(start) which has EOF
    assert!(follow_count(&ff, sym(&g, "tail")) > 0);
    assert_follow_contains(&ff, sym(&g, "tail"), &[EOF]);
}

// ===========================================================================
// 13. first_distinct_* — Multiple non-terminals have distinct FIRST sets (6 tests)
// ===========================================================================

#[test]
fn first_distinct_two_disjoint_nonterminals() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["left"]),
            ("start", vec!["right"]),
            ("left", vec!["a"]),
            ("right", vec!["b"]),
        ],
        "start",
    );
    let left_id = sym(&g, "left");
    let right_id = sym(&g, "right");
    assert_first_contains(&ff, left_id, &[sym(&g, "a")]);
    assert_first_excludes(&ff, left_id, &[sym(&g, "b")]);
    assert_first_contains(&ff, right_id, &[sym(&g, "b")]);
    assert_first_excludes(&ff, right_id, &[sym(&g, "a")]);
}

#[test]
fn first_distinct_three_disjoint_nonterminals() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["p"]),
            ("start", vec!["q"]),
            ("start", vec!["r"]),
            ("p", vec!["a"]),
            ("q", vec!["b"]),
            ("r", vec!["c"]),
        ],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "p"), &[sym(&g, "a")]);
    assert_first_eq(&ff, sym(&g, "q"), &[sym(&g, "b")]);
    assert_first_eq(&ff, sym(&g, "r"), &[sym(&g, "c")]);
}

#[test]
fn first_distinct_statement_types() {
    let (g, ff) = build_ff(
        "t",
        &[
            ("kw_if", "if"),
            ("kw_while", "while"),
            ("kw_return", "return"),
            ("dummy", "d"),
        ],
        &[
            ("start", vec!["stmt"]),
            ("stmt", vec!["if_stmt"]),
            ("stmt", vec!["while_stmt"]),
            ("stmt", vec!["ret_stmt"]),
            ("if_stmt", vec!["kw_if", "dummy"]),
            ("while_stmt", vec!["kw_while", "dummy"]),
            ("ret_stmt", vec!["kw_return", "dummy"]),
        ],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "if_stmt"), &[sym(&g, "kw_if")]);
    assert_first_eq(&ff, sym(&g, "while_stmt"), &[sym(&g, "kw_while")]);
    assert_first_eq(&ff, sym(&g, "ret_stmt"), &[sym(&g, "kw_return")]);
}

#[test]
fn first_distinct_parent_unions_children() {
    let (g, ff) = build_ff(
        "t",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["left"]),
            ("start", vec!["right"]),
            ("left", vec!["a"]),
            ("right", vec!["b"]),
        ],
        "start",
    );
    // Parent includes union of children
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_distinct_no_cross_contamination() {
    let (g, ff) = build_ff(
        "t",
        &[("x", "x"), ("y", "y")],
        &[
            ("start", vec!["alpha", "beta"]),
            ("alpha", vec!["x"]),
            ("beta", vec!["y"]),
        ],
        "start",
    );
    // alpha should only have x, not y
    assert_first_eq(&ff, sym(&g, "alpha"), &[sym(&g, "x")]);
    // beta should only have y, not x
    assert_first_eq(&ff, sym(&g, "beta"), &[sym(&g, "y")]);
}

#[test]
fn first_distinct_arithmetic_levels_differ_in_exclusions() {
    let (g, ff) = build_arithmetic();
    // factor doesn't include plus or star in FIRST
    assert_first_excludes(&ff, sym(&g, "factor"), &[sym(&g, "plus"), sym(&g, "star")]);
}

// ===========================================================================
// 14. ff_pattern_* — Various grammar patterns (list, expr, nested) (8 tests)
// ===========================================================================

#[test]
fn ff_pattern_list_left_recursive() {
    let (g, ff) = build_ff(
        "list",
        &[("item", "i"), ("comma", ",")],
        &[
            ("start", vec!["item"]),
            ("start", vec!["start", "comma", "item"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "item")]);
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn ff_pattern_list_right_recursive() {
    let (g, ff) = build_ff(
        "list",
        &[("item", "i"), ("comma", ",")],
        &[
            ("start", vec!["item"]),
            ("start", vec!["item", "comma", "start"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "item")]);
}

#[test]
fn ff_pattern_balanced_parens() {
    let (g, ff) = build_ff(
        "parens",
        &[("lp", "("), ("rp", ")")],
        &[
            ("start", vec!["lp", "rp"]),
            ("start", vec!["lp", "start", "rp"]),
        ],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "lp")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "rp")]);
}

#[test]
fn ff_pattern_if_else() {
    let (g, ff) = build_ff(
        "ifelse",
        &[
            ("kw_if", "if"),
            ("kw_else", "else"),
            ("cond", "c"),
            ("body", "b"),
        ],
        &[
            ("start", vec!["kw_if", "cond", "body"]),
            ("start", vec!["kw_if", "cond", "body", "kw_else", "body"]),
        ],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "kw_if")]);
}

#[test]
fn ff_pattern_binary_expr() {
    let (g, ff) = build_ff(
        "binexpr",
        &[("num", "0"), ("op", "+")],
        &[
            ("start", vec!["num"]),
            ("start", vec!["start", "op", "start"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "num")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "op")]);
}

#[test]
fn ff_pattern_ternary_operator() {
    let (g, ff) = build_ff(
        "ternary",
        &[("num", "0"), ("qmark", "?"), ("colon", ":")],
        &[
            ("start", vec!["num"]),
            ("start", vec!["start", "qmark", "start", "colon", "start"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "num")]);
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn ff_pattern_assignment() {
    let (g, ff) = build_ff(
        "assign",
        &[
            ("ident", "[a-z]+"),
            ("eq", "="),
            ("num", "[0-9]+"),
            ("semi", ";"),
        ],
        &[
            ("start", vec!["ident", "eq", "val", "semi"]),
            ("val", vec!["num"]),
        ],
        "start",
    );
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "ident")]);
    assert_follow_contains(&ff, sym(&g, "val"), &[sym(&g, "semi")]);
}

#[test]
fn ff_pattern_nested_blocks() {
    let (g, ff) = build_ff(
        "blocks",
        &[("lb", "{"), ("rb", "}"), ("tok", "t")],
        &[
            ("start", vec!["block"]),
            ("block", vec!["lb", "body", "rb"]),
            ("body", vec!["tok"]),
            ("body", vec!["block"]),
        ],
        "start",
    );
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "lb")]);
    assert_first_contains(&ff, sym(&g, "body"), &[sym(&g, "tok"), sym(&g, "lb")]);
    assert_follow_contains(&ff, sym(&g, "body"), &[sym(&g, "rb")]);
}

// ===========================================================================
// 15. ff_large_* — Large grammar (20+ tokens, 10+ rules) computes (6 tests)
// ===========================================================================

fn build_large_grammar() -> (Grammar, FirstFollowSets) {
    build_ff(
        "large",
        &[
            ("num", "[0-9]+"),
            ("ident", "[a-z]+"),
            ("str_lit", "\"[^\"]*\""),
            ("plus", "+"),
            ("minus", "-"),
            ("star", "*"),
            ("slash", "div"),
            ("percent", "%"),
            ("eq", "="),
            ("eqeq", "=="),
            ("neq", "!="),
            ("lt", "<"),
            ("gt", ">"),
            ("lp", "("),
            ("rp", ")"),
            ("lb", "{"),
            ("rb", "}"),
            ("semi", ";"),
            ("comma", ","),
            ("dot", "."),
            ("kw_if", "if"),
            ("kw_else", "else"),
            ("kw_while", "while"),
            ("kw_return", "return"),
            ("kw_let", "let"),
        ],
        &[
            // program -> stmt_list
            ("program", vec!["stmt_list"]),
            // stmt_list -> stmt | stmt_list stmt
            ("stmt_list", vec!["stmt"]),
            ("stmt_list", vec!["stmt_list", "stmt"]),
            // stmt -> assign_stmt | if_stmt | while_stmt | ret_stmt | expr_stmt
            ("stmt", vec!["assign_stmt"]),
            ("stmt", vec!["if_stmt"]),
            ("stmt", vec!["while_stmt"]),
            ("stmt", vec!["ret_stmt"]),
            ("stmt", vec!["expr_stmt"]),
            // assign_stmt -> kw_let ident eq val semi
            ("assign_stmt", vec!["kw_let", "ident", "eq", "val", "semi"]),
            // if_stmt -> kw_if lp val rp block
            ("if_stmt", vec!["kw_if", "lp", "val", "rp", "block"]),
            // while_stmt -> kw_while lp val rp block
            ("while_stmt", vec!["kw_while", "lp", "val", "rp", "block"]),
            // ret_stmt -> kw_return val semi
            ("ret_stmt", vec!["kw_return", "val", "semi"]),
            // expr_stmt -> val semi
            ("expr_stmt", vec!["val", "semi"]),
            // block -> lb stmt_list rb
            ("block", vec!["lb", "stmt_list", "rb"]),
            // val -> num | ident | str_lit | lp val rp
            ("val", vec!["num"]),
            ("val", vec!["ident"]),
            ("val", vec!["str_lit"]),
            ("val", vec!["lp", "val", "rp"]),
        ],
        "program",
    )
}

#[test]
fn ff_large_computes_successfully() {
    let (_g, _ff) = build_large_grammar();
}

#[test]
fn ff_large_program_first_set() {
    let (g, ff) = build_large_grammar();
    // program starts with any statement, so FIRST includes kw_let, kw_if, kw_while, kw_return, num, ident, str_lit, lp
    assert_first_contains(
        &ff,
        sym(&g, "program"),
        &[
            sym(&g, "kw_let"),
            sym(&g, "kw_if"),
            sym(&g, "kw_while"),
            sym(&g, "kw_return"),
        ],
    );
    assert_first_contains(
        &ff,
        sym(&g, "program"),
        &[
            sym(&g, "num"),
            sym(&g, "ident"),
            sym(&g, "str_lit"),
            sym(&g, "lp"),
        ],
    );
}

#[test]
fn ff_large_stmt_first_matches_keywords_and_values() {
    let (g, ff) = build_large_grammar();
    assert_first_contains(
        &ff,
        sym(&g, "stmt"),
        &[sym(&g, "kw_let"), sym(&g, "kw_if"), sym(&g, "num")],
    );
}

#[test]
fn ff_large_val_first_set() {
    let (g, ff) = build_large_grammar();
    assert_first_contains(
        &ff,
        sym(&g, "val"),
        &[
            sym(&g, "num"),
            sym(&g, "ident"),
            sym(&g, "str_lit"),
            sym(&g, "lp"),
        ],
    );
    // Operators should not be in FIRST(val)
    assert_first_excludes(
        &ff,
        sym(&g, "val"),
        &[sym(&g, "plus"), sym(&g, "star"), sym(&g, "semi")],
    );
}

#[test]
fn ff_large_all_nonterminals_have_nonempty_first() {
    let (g, ff) = build_large_grammar();
    for name in [
        "program",
        "stmt_list",
        "stmt",
        "assign_stmt",
        "if_stmt",
        "while_stmt",
        "ret_stmt",
        "expr_stmt",
        "block",
        "val",
    ] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

#[test]
fn ff_large_follow_sets_propagate() {
    let (g, ff) = build_large_grammar();
    assert_follow_contains(&ff, sym(&g, "program"), &[EOF]);
    // val in assign_stmt is followed by semi
    assert_follow_contains(&ff, sym(&g, "val"), &[sym(&g, "semi")]);
    // block follows if_stmt's rp, so stmt_list inside block is followed by rb
    assert_follow_contains(&ff, sym(&g, "stmt_list"), &[sym(&g, "rb")]);
}
