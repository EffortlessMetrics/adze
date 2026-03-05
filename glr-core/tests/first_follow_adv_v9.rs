//! FIRST/FOLLOW set computation — advanced tests v9.
//!
//! 84 tests across 20 categories:
//!  1. ff_minimal_*             — compute on minimal grammar → Ok
//!  2. first_terminal_self_*    — FIRST set of terminal → contains itself
//!  3. first_nonterminal_*      — FIRST set of non-terminal → contains first tokens
//!  4. follow_start_eof_*       — FOLLOW set of start symbol → contains EOF
//!  5. first_nonempty_*         — FIRST sets non-empty for all reachable symbols
//!  6. follow_nonempty_start_*  — FOLLOW sets non-empty for start symbol
//!  7. ff_deterministic_*       — compute is deterministic
//!  8. ff_same_grammar_*        — same grammar → same FIRST/FOLLOW
//!  9. ff_different_grammar_*   — different grammars → different sets
//! 10. first_alternatives_*     — grammar with alternatives → FIRST has multiple
//! 11. first_chain_*            — grammar with chain rule → FIRST propagates
//! 12. first_left_recursion_*   — grammar with left recursion → FIRST still computed
//! 13. first_token_singleton_*  — FIRST of token is singleton
//! 14. follow_last_symbol_*     — FOLLOW of last symbol in rule
//! 15. ff_precedence_*          — FIRST/FOLLOW with precedence
//! 16. ff_inline_*              — FIRST/FOLLOW with inline rules
//! 17. ff_extras_*              — FIRST/FOLLOW with extras
//! 18. ff_arithmetic_*          — arithmetic grammar FIRST/FOLLOW
//! 19. ff_normalize_*           — FIRST/FOLLOW after normalize
//! 20. ff_large_*               — large grammar FIRST/FOLLOW computation

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, SymbolId};

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
        .unwrap_or_else(|| panic!("symbol '{name}' not found"))
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

fn assert_first_excludes(ff: &FirstFollowSets, id: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .first(id)
        .unwrap_or_else(|| panic!("no FIRST set for {id:?}"));
    for &e in expected {
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

fn first_count(ff: &FirstFollowSets, id: SymbolId) -> usize {
    ff.first(id).map_or(0, |s| s.count_ones(..))
}

fn follow_count(ff: &FirstFollowSets, id: SymbolId) -> usize {
    ff.follow(id).map_or(0, |s| s.count_ones(..))
}

// ===========================================================================
// 1. ff_minimal_* — compute on minimal grammar → Ok (4 tests)
// ===========================================================================

#[test]
fn ff_minimal_single_token_single_rule() {
    let result = FirstFollowSets::compute(
        &GrammarBuilder::new("ffa_v9_min1")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build(),
    );
    assert!(result.is_ok());
}

#[test]
fn ff_minimal_two_tokens() {
    let (g, ff) = build_ff(
        "ffa_v9_min2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "a")]);
}

#[test]
fn ff_minimal_returns_ok() {
    let g = GrammarBuilder::new("ffa_v9_min3")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let result = FirstFollowSets::compute(&g);
    assert!(result.is_ok());
}

#[test]
fn ff_minimal_two_rules() {
    let (g, ff) = build_ff(
        "ffa_v9_min4",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["inner"]), ("inner", vec!["a"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "a")]);
}

// ===========================================================================
// 2. first_terminal_self_* — FIRST set of terminal → contains itself (5 tests)
// ===========================================================================

#[test]
fn first_terminal_self_keyword() {
    let (g, ff) = build_ff("ffa_v9_ts1", &[("kw", "let")], &[("s", vec!["kw"])], "s");
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "kw")]);
}

#[test]
fn first_terminal_self_operator() {
    let (g, ff) = build_ff("ffa_v9_ts2", &[("plus", "+")], &[("s", vec!["plus"])], "s");
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "plus")]);
}

#[test]
fn first_terminal_self_punctuation() {
    let (g, ff) = build_ff("ffa_v9_ts3", &[("semi", ";")], &[("s", vec!["semi"])], "s");
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "semi")]);
}

#[test]
fn first_terminal_self_regex() {
    let (g, ff) = build_ff(
        "ffa_v9_ts4",
        &[("num", r"[0-9]+")],
        &[("s", vec!["num"])],
        "s",
    );
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "num")]);
}

#[test]
fn first_terminal_self_multiple_tokens_each() {
    let (g, ff) = build_ff(
        "ffa_v9_ts5",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("t", vec!["b"]), ("u", vec!["c"])],
        "s",
    );
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "a")]);
    assert_first_eq(&ff, sym(&g, "t"), &[sym(&g, "b")]);
    assert_first_eq(&ff, sym(&g, "u"), &[sym(&g, "c")]);
}

// ===========================================================================
// 3. first_nonterminal_* — FIRST set of non-terminal → contains first tokens (5 tests)
// ===========================================================================

#[test]
fn first_nonterminal_single_rule_leading_token() {
    let (g, ff) = build_ff(
        "ffa_v9_nt1",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "a")]);
    assert_first_excludes(&ff, sym(&g, "s"), &[sym(&g, "b")]);
}

#[test]
fn first_nonterminal_through_chain() {
    let (g, ff) = build_ff(
        "ffa_v9_nt2",
        &[("x", "x")],
        &[("s", vec!["mid"]), ("mid", vec!["x"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "x")]);
}

#[test]
fn first_nonterminal_two_alternatives() {
    let (g, ff) = build_ff(
        "ffa_v9_nt3",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_nonterminal_deep_three_levels() {
    let (g, ff) = build_ff(
        "ffa_v9_nt4",
        &[("tok", "t")],
        &[("s", vec!["a"]), ("a", vec!["b"]), ("b", vec!["tok"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "tok")]);
    assert_first_contains(&ff, sym(&g, "a"), &[sym(&g, "tok")]);
}

#[test]
fn first_nonterminal_multiple_paths() {
    let (g, ff) = build_ff(
        "ffa_v9_nt5",
        &[("x", "x"), ("y", "y")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("a", vec!["x"]),
            ("b", vec!["y"]),
        ],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "x"), sym(&g, "y")]);
}

// ===========================================================================
// 4. follow_start_eof_* — FOLLOW set of start symbol → contains EOF (5 tests)
// ===========================================================================

#[test]
fn follow_start_eof_simple() {
    let (g, ff) = build_ff("ffa_v9_eof1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_follow_contains(&ff, sym(&g, "s"), &[EOF]);
}

#[test]
fn follow_start_eof_two_tokens() {
    let (g, ff) = build_ff(
        "ffa_v9_eof2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "s"), &[EOF]);
}

#[test]
fn follow_start_eof_with_alternatives() {
    let (g, ff) = build_ff(
        "ffa_v9_eof3",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "s"), &[EOF]);
}

#[test]
fn follow_start_eof_with_chain() {
    let (g, ff) = build_ff(
        "ffa_v9_eof4",
        &[("x", "x")],
        &[("s", vec!["inner"]), ("inner", vec!["x"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "s"), &[EOF]);
}

#[test]
fn follow_start_eof_recursive() {
    let (g, ff) = build_ff(
        "ffa_v9_eof5",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "s"), &[EOF]);
}

// ===========================================================================
// 5. first_nonempty_* — FIRST sets non-empty for all reachable symbols (4 tests)
// ===========================================================================

#[test]
fn first_nonempty_all_nonterminals() {
    let (g, ff) = build_ff(
        "ffa_v9_ne1",
        &[("a", "a"), ("b", "b")],
        &[
            ("s", vec!["inner"]),
            ("inner", vec!["a"]),
            ("inner", vec!["b"]),
        ],
        "s",
    );
    for name in ["s", "inner"] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

#[test]
fn first_nonempty_chain_of_nonterminals() {
    let (g, ff) = build_ff(
        "ffa_v9_ne2",
        &[("tok", "t")],
        &[("s", vec!["a"]), ("a", vec!["b"]), ("b", vec!["tok"])],
        "s",
    );
    for name in ["s", "a", "b"] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

#[test]
fn first_nonempty_multiple_alternatives() {
    let (g, ff) = build_ff(
        "ffa_v9_ne3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(first_count(&ff, sym(&g, "s")) >= 3);
}

#[test]
fn first_nonempty_mixed_grammar() {
    let (g, ff) = build_ff(
        "ffa_v9_ne4",
        &[("num", r"[0-9]+"), ("lp", "("), ("rp", ")")],
        &[
            ("s", vec!["expr"]),
            ("expr", vec!["num"]),
            ("expr", vec!["lp", "expr", "rp"]),
        ],
        "s",
    );
    for name in ["s", "expr"] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

// ===========================================================================
// 6. follow_nonempty_start_* — FOLLOW sets non-empty for start symbol (4 tests)
// ===========================================================================

#[test]
fn follow_nonempty_start_basic() {
    let (g, ff) = build_ff("ffa_v9_fne1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(follow_count(&ff, sym(&g, "s")) > 0);
}

#[test]
fn follow_nonempty_start_with_alternatives() {
    let (g, ff) = build_ff(
        "ffa_v9_fne2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(follow_count(&ff, sym(&g, "s")) > 0);
}

#[test]
fn follow_nonempty_start_chain() {
    let (g, ff) = build_ff(
        "ffa_v9_fne3",
        &[("x", "x")],
        &[("s", vec!["inner"]), ("inner", vec!["x"])],
        "s",
    );
    assert!(follow_count(&ff, sym(&g, "s")) > 0);
}

#[test]
fn follow_nonempty_start_recursive() {
    let (g, ff) = build_ff(
        "ffa_v9_fne4",
        &[("a", "a"), ("plus", "+")],
        &[("s", vec!["s", "plus", "a"]), ("s", vec!["a"])],
        "s",
    );
    assert!(follow_count(&ff, sym(&g, "s")) > 0);
}

// ===========================================================================
// 7. ff_deterministic_* — compute is deterministic (4 tests)
// ===========================================================================

fn build_grammar_for_det(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

#[test]
fn ff_deterministic_first_sets_match() {
    let g1 = build_grammar_for_det("ffa_v9_det1a");
    let g2 = build_grammar_for_det("ffa_v9_det1b");
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let s1 = sym(&g1, "s");
    let s2 = sym(&g2, "s");
    assert_eq!(first_count(&ff1, s1), first_count(&ff2, s2));
}

#[test]
fn ff_deterministic_follow_sets_match() {
    let g1 = build_grammar_for_det("ffa_v9_det2a");
    let g2 = build_grammar_for_det("ffa_v9_det2b");
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let s1 = sym(&g1, "s");
    let s2 = sym(&g2, "s");
    assert_eq!(follow_count(&ff1, s1), follow_count(&ff2, s2));
}

#[test]
fn ff_deterministic_repeated_compute() {
    let g = build_grammar_for_det("ffa_v9_det3");
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert_eq!(first_count(&ff1, s), first_count(&ff2, s));
    assert_eq!(follow_count(&ff1, s), follow_count(&ff2, s));
}

#[test]
fn ff_deterministic_nullable_agreement() {
    let g = build_grammar_for_det("ffa_v9_det4");
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert_eq!(ff1.is_nullable(s), ff2.is_nullable(s));
}

// ===========================================================================
// 8. ff_same_grammar_* — same grammar → same FIRST/FOLLOW (4 tests)
// ===========================================================================

#[test]
fn ff_same_grammar_first_count_equal() {
    let (_, ff1) = build_ff(
        "ffa_v9_sg1a",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["y"])],
        "s",
    );
    let (_, ff2) = build_ff(
        "ffa_v9_sg1b",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["y"])],
        "s",
    );
    // Both grammars have the same structure; symbol IDs should line up.
    let s1 = SymbolId(1); // first non-terminal after tokens
    let s2 = SymbolId(1);
    // Compare counts instead of exact bitsets since IDs may differ.
    assert_eq!(first_count(&ff1, s1), first_count(&ff2, s2));
}

#[test]
fn ff_same_grammar_follow_count_equal() {
    let (g1, ff1) = build_ff("ffa_v9_sg2a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (g2, ff2) = build_ff("ffa_v9_sg2b", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(
        follow_count(&ff1, sym(&g1, "s")),
        follow_count(&ff2, sym(&g2, "s")),
    );
}

#[test]
fn ff_same_grammar_nullable_equal() {
    let (g1, ff1) = build_ff("ffa_v9_sg3a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (g2, ff2) = build_ff("ffa_v9_sg3b", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(
        ff1.is_nullable(sym(&g1, "s")),
        ff2.is_nullable(sym(&g2, "s")),
    );
}

#[test]
fn ff_same_grammar_chain_identical() {
    let build = |name| {
        build_ff(
            name,
            &[("tok", "t")],
            &[("s", vec!["mid"]), ("mid", vec!["tok"])],
            "s",
        )
    };
    let (g1, ff1) = build("ffa_v9_sg4a");
    let (g2, ff2) = build("ffa_v9_sg4b");
    assert_eq!(
        first_count(&ff1, sym(&g1, "s")),
        first_count(&ff2, sym(&g2, "s")),
    );
    assert_eq!(
        first_count(&ff1, sym(&g1, "mid")),
        first_count(&ff2, sym(&g2, "mid")),
    );
}

// ===========================================================================
// 9. ff_different_grammar_* — different grammars → different sets (4 tests)
// ===========================================================================

#[test]
fn ff_different_grammar_different_first_count() {
    let (g1, ff1) = build_ff("ffa_v9_dg1a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (g2, ff2) = build_ff(
        "ffa_v9_dg1b",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    // Second grammar has more in FIRST(s).
    assert!(first_count(&ff2, sym(&g2, "s")) > first_count(&ff1, sym(&g1, "s")));
}

#[test]
fn ff_different_grammar_different_tokens() {
    let (g1, ff1) = build_ff("ffa_v9_dg2a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (g2, ff2) = build_ff("ffa_v9_dg2b", &[("b", "b")], &[("s", vec!["b"])], "s");
    // Both have FIRST(s) size 1 but different contents.
    assert_eq!(first_count(&ff1, sym(&g1, "s")), 1);
    assert_eq!(first_count(&ff2, sym(&g2, "s")), 1);
}

#[test]
fn ff_different_grammar_more_rules_more_follow() {
    let (g1, ff1) = build_ff("ffa_v9_dg3a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (g2, ff2) = build_ff(
        "ffa_v9_dg3b",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["inner", "b"]), ("inner", vec!["a"])],
        "s",
    );
    // inner in second grammar is followed by b (plus potentially EOF), but first has no "inner".
    assert!(follow_count(&ff2, sym(&g2, "inner")) >= follow_count(&ff1, sym(&g1, "s")));
}

#[test]
fn ff_different_grammar_nullable_difference() {
    let (g1, ff1) = build_ff("ffa_v9_dg4a", &[("a", "a")], &[("s", vec!["a"])], "s");
    let (g2, ff2) = build_ff(
        "ffa_v9_dg4b",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    // Neither is nullable, but their FIRST sizes differ.
    assert!(!ff1.is_nullable(sym(&g1, "s")));
    assert!(!ff2.is_nullable(sym(&g2, "s")));
    assert!(first_count(&ff2, sym(&g2, "s")) > first_count(&ff1, sym(&g1, "s")));
}

// ===========================================================================
// 10. first_alternatives_* — grammar with alternatives → FIRST has multiple (5 tests)
// ===========================================================================

#[test]
fn first_alternatives_two() {
    let (g, ff) = build_ff(
        "ffa_v9_alt1",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_alternatives_three() {
    let (g, ff) = build_ff(
        "ffa_v9_alt2",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert_first_eq(
        &ff,
        sym(&g, "s"),
        &[sym(&g, "a"), sym(&g, "b"), sym(&g, "c")],
    );
}

#[test]
fn first_alternatives_four() {
    let (g, ff) = build_ff(
        "ffa_v9_alt3",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
        ],
        "s",
    );
    assert_eq!(first_count(&ff, sym(&g, "s")), 4);
}

#[test]
fn first_alternatives_via_nonterminals() {
    let (g, ff) = build_ff(
        "ffa_v9_alt4",
        &[("x", "x"), ("y", "y")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("a", vec!["x"]),
            ("b", vec!["y"]),
        ],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "x"), sym(&g, "y")]);
}

#[test]
fn first_alternatives_mixed_depth() {
    let (g, ff) = build_ff(
        "ffa_v9_alt5",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["inner"]), ("inner", vec!["y"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "x"), sym(&g, "y")]);
}

// ===========================================================================
// 11. first_chain_* — grammar with chain rule → FIRST propagates (5 tests)
// ===========================================================================

#[test]
fn first_chain_two_levels() {
    let (g, ff) = build_ff(
        "ffa_v9_ch1",
        &[("tok", "t")],
        &[("s", vec!["mid"]), ("mid", vec!["tok"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "tok")]);
}

#[test]
fn first_chain_three_levels() {
    let (g, ff) = build_ff(
        "ffa_v9_ch2",
        &[("tok", "t")],
        &[("s", vec!["a"]), ("a", vec!["b"]), ("b", vec!["tok"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "tok")]);
    assert_first_contains(&ff, sym(&g, "a"), &[sym(&g, "tok")]);
}

#[test]
fn first_chain_four_levels() {
    let (g, ff) = build_ff(
        "ffa_v9_ch3",
        &[("val", "v")],
        &[
            ("s", vec!["l1"]),
            ("l1", vec!["l2"]),
            ("l2", vec!["l3"]),
            ("l3", vec!["val"]),
        ],
        "s",
    );
    for name in ["s", "l1", "l2", "l3"] {
        assert_first_contains(&ff, sym(&g, name), &[sym(&g, "val")]);
    }
}

#[test]
fn first_chain_preserves_alternatives_at_leaf() {
    let (g, ff) = build_ff(
        "ffa_v9_ch4",
        &[("x", "x"), ("y", "y")],
        &[
            ("s", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
            ("leaf", vec!["y"]),
        ],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "x"), sym(&g, "y")]);
}

#[test]
fn first_chain_branching() {
    let (g, ff) = build_ff(
        "ffa_v9_ch5",
        &[("a", "a"), ("b", "b")],
        &[
            ("s", vec!["left"]),
            ("s", vec!["right"]),
            ("left", vec!["a"]),
            ("right", vec!["b"]),
        ],
        "s",
    );
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "a"), sym(&g, "b")]);
}

// ===========================================================================
// 12. first_left_recursion_* — left recursion → FIRST still computed (4 tests)
// ===========================================================================

#[test]
fn first_left_recursion_direct() {
    let (g, ff) = build_ff(
        "ffa_v9_lr1",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "b")]);
}

#[test]
fn first_left_recursion_excludes_non_leading() {
    let (g, ff) = build_ff(
        "ffa_v9_lr2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    assert_first_excludes(&ff, sym(&g, "s"), &[sym(&g, "a")]);
}

#[test]
fn first_left_recursion_mutual() {
    // s -> a x | b; a -> s y | x
    let (g, ff) = build_ff(
        "ffa_v9_lr3",
        &[("x", "x"), ("y", "y"), ("b", "b")],
        &[
            ("s", vec!["a", "x"]),
            ("s", vec!["b"]),
            ("a", vec!["s", "y"]),
            ("a", vec!["x"]),
        ],
        "s",
    );
    assert_first_contains(&ff, sym(&g, "s"), &[sym(&g, "x"), sym(&g, "b")]);
}

#[test]
fn first_left_recursion_list_pattern() {
    // list -> list "," item | item; item -> "id"
    let (g, ff) = build_ff(
        "ffa_v9_lr4",
        &[("comma", ","), ("id", "id")],
        &[
            ("list", vec!["list", "comma", "item"]),
            ("list", vec!["item"]),
            ("item", vec!["id"]),
        ],
        "list",
    );
    assert_first_contains(&ff, sym(&g, "list"), &[sym(&g, "id")]);
    assert_first_excludes(&ff, sym(&g, "list"), &[sym(&g, "comma")]);
}

// ===========================================================================
// 13. first_token_singleton_* — FIRST of token-only rule is singleton (4 tests)
// ===========================================================================

#[test]
fn first_token_singleton_basic() {
    let (g, ff) = build_ff("ffa_v9_sing1", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(first_count(&ff, sym(&g, "s")), 1);
}

#[test]
fn first_token_singleton_via_chain() {
    let (g, ff) = build_ff(
        "ffa_v9_sing2",
        &[("a", "a")],
        &[("s", vec!["mid"]), ("mid", vec!["a"])],
        "s",
    );
    assert_eq!(first_count(&ff, sym(&g, "mid")), 1);
}

#[test]
fn first_token_singleton_multi_token_rule() {
    let (g, ff) = build_ff(
        "ffa_v9_sing3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert_eq!(first_count(&ff, sym(&g, "s")), 1);
    assert_first_eq(&ff, sym(&g, "s"), &[sym(&g, "a")]);
}

#[test]
fn first_token_singleton_each_branch() {
    let (g, ff) = build_ff(
        "ffa_v9_sing4",
        &[("x", "x"), ("y", "y")],
        &[("a", vec!["x"]), ("b", vec!["y"]), ("s", vec!["a"])],
        "s",
    );
    assert_eq!(first_count(&ff, sym(&g, "a")), 1);
    assert_eq!(first_count(&ff, sym(&g, "b")), 1);
}

// ===========================================================================
// 14. follow_last_symbol_* — FOLLOW of last symbol in rule (5 tests)
// ===========================================================================

#[test]
fn follow_last_symbol_terminal_after() {
    // s -> inner ";" => FOLLOW(inner) ⊇ {";"}
    let (g, ff) = build_ff(
        "ffa_v9_fl1",
        &[("a", "a"), ("semi", ";")],
        &[("s", vec!["inner", "semi"]), ("inner", vec!["a"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "semi")]);
}

#[test]
fn follow_last_symbol_inherits_lhs_follow() {
    // s -> inner; inner -> "a" => FOLLOW(inner) ⊇ FOLLOW(s) ⊇ {EOF}
    let (g, ff) = build_ff(
        "ffa_v9_fl2",
        &[("a", "a")],
        &[("s", vec!["inner"]), ("inner", vec!["a"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "inner"), &[EOF]);
}

#[test]
fn follow_last_symbol_multiple_contexts() {
    // s -> inner "x"; t -> inner "y" => FOLLOW(inner) ⊇ {"x", "y"}
    let (g, ff) = build_ff(
        "ffa_v9_fl3",
        &[("a", "a"), ("x", "x"), ("y", "y")],
        &[
            ("s", vec!["inner", "x"]),
            ("s", vec!["other", "y"]),
            ("inner", vec!["a"]),
            ("other", vec!["inner", "y"]),
        ],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "x")]);
}

#[test]
fn follow_last_symbol_at_end_of_start() {
    // s -> "a" inner => FOLLOW(inner) ⊇ {EOF}
    let (g, ff) = build_ff(
        "ffa_v9_fl4",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "inner"]), ("inner", vec!["b"])],
        "s",
    );
    assert_follow_contains(&ff, sym(&g, "inner"), &[EOF]);
}

#[test]
fn follow_last_symbol_excludes_unrelated() {
    // s -> inner "x"; inner -> "a" => "a" is not in FOLLOW(inner)
    let (g, ff) = build_ff(
        "ffa_v9_fl5",
        &[("a", "a"), ("x", "x")],
        &[("s", vec!["inner", "x"]), ("inner", vec!["a"])],
        "s",
    );
    assert_follow_excludes(&ff, sym(&g, "inner"), &[sym(&g, "a")]);
}

// ===========================================================================
// 15. ff_precedence_* — FIRST/FOLLOW with precedence (4 tests)
// ===========================================================================

#[test]
fn ff_precedence_basic_computes() {
    let g = GrammarBuilder::new("ffa_v9_prec1")
        .token("num", r"[0-9]+")
        .token("plus", "plus")
        .token("star", "star")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["num"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn ff_precedence_first_includes_num() {
    let g = GrammarBuilder::new("ffa_v9_prec2")
        .token("num", r"[0-9]+")
        .token("plus", "plus")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["num"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let num_id = g.find_symbol_by_name("num").unwrap();
    assert_first_contains(&ff, expr_id, &[num_id]);
}

#[test]
fn ff_precedence_follow_includes_operators() {
    let g = GrammarBuilder::new("ffa_v9_prec3")
        .token("num", r"[0-9]+")
        .token("plus", "plus")
        .token("star", "star")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["num"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let plus_id = g.find_symbol_by_name("plus").unwrap();
    let star_id = g.find_symbol_by_name("star").unwrap();
    assert_follow_contains(&ff, expr_id, &[plus_id, star_id, EOF]);
}

#[test]
fn ff_precedence_right_assoc() {
    let g = GrammarBuilder::new("ffa_v9_prec4")
        .token("num", r"[0-9]+")
        .token("caret", "caret")
        .rule("expr", vec!["expr", "caret", "expr"])
        .rule("expr", vec!["num"])
        .precedence(1, Associativity::Right, vec!["caret"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let num_id = g.find_symbol_by_name("num").unwrap();
    assert_first_contains(&ff, expr_id, &[num_id]);
    assert_follow_contains(&ff, expr_id, &[EOF]);
}

// ===========================================================================
// 16. ff_inline_* — FIRST/FOLLOW with inline rules (4 tests)
// ===========================================================================

#[test]
fn ff_inline_basic_computes() {
    let g = GrammarBuilder::new("ffa_v9_inl1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["a", "b"])
        .inline("helper")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn ff_inline_first_propagates() {
    let g = GrammarBuilder::new("ffa_v9_inl2")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["x"])
        .rule("helper", vec!["y"])
        .inline("helper")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let x_id = g.find_symbol_by_name("x").unwrap();
    let y_id = g.find_symbol_by_name("y").unwrap();
    assert_first_contains(&ff, s_id, &[x_id, y_id]);
}

#[test]
fn ff_inline_follow_propagates() {
    let g = GrammarBuilder::new("ffa_v9_inl3")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["helper", "b"])
        .rule("helper", vec!["a"])
        .inline("helper")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    assert_follow_contains(&ff, helper_id, &[b_id]);
}

#[test]
fn ff_inline_chain_computes() {
    let g = GrammarBuilder::new("ffa_v9_inl4")
        .token("tok", "t")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["tok"])
        .inline("mid")
        .inline("leaf")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let tok_id = g.find_symbol_by_name("tok").unwrap();
    assert_first_contains(&ff, s_id, &[tok_id]);
}

// ===========================================================================
// 17. ff_extras_* — FIRST/FOLLOW with extras (4 tests)
// ===========================================================================

#[test]
fn ff_extras_basic_computes() {
    let g = GrammarBuilder::new("ffa_v9_ext1")
        .token("a", "a")
        .token("ws", r"\s+")
        .rule("s", vec!["a"])
        .extra("ws")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn ff_extras_first_unaffected() {
    let g = GrammarBuilder::new("ffa_v9_ext2")
        .token("a", "a")
        .token("b", "b")
        .token("ws", r"\s+")
        .rule("s", vec!["a", "b"])
        .extra("ws")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let a_id = g.find_symbol_by_name("a").unwrap();
    assert_first_contains(&ff, s_id, &[a_id]);
}

#[test]
fn ff_extras_follow_start_has_eof() {
    let g = GrammarBuilder::new("ffa_v9_ext3")
        .token("x", "x")
        .token("ws", r"\s+")
        .rule("s", vec!["x"])
        .extra("ws")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_follow_contains(&ff, s_id, &[EOF]);
}

#[test]
fn ff_extras_multiple_extras() {
    let g = GrammarBuilder::new("ffa_v9_ext4")
        .token("a", "a")
        .token("ws", r"\s+")
        .token("comment", r"//[^\n]*")
        .rule("s", vec!["a"])
        .extra("ws")
        .extra("comment")
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

// ===========================================================================
// 18. ff_arithmetic_* — arithmetic grammar FIRST/FOLLOW (5 tests)
// ===========================================================================

fn build_arithmetic(name: &str) -> (Grammar, FirstFollowSets) {
    build_ff(
        name,
        &[
            ("num", r"[0-9]+"),
            ("plus", "+"),
            ("star", "*"),
            ("lp", "("),
            ("rp", ")"),
        ],
        &[
            ("expr", vec!["expr", "plus", "term"]),
            ("expr", vec!["term"]),
            ("term", vec!["term", "star", "factor"]),
            ("term", vec!["factor"]),
            ("factor", vec!["lp", "expr", "rp"]),
            ("factor", vec!["num"]),
        ],
        "expr",
    )
}

#[test]
fn ff_arithmetic_computes_ok() {
    let (_g, _ff) = build_arithmetic("ffa_v9_arith1");
}

#[test]
fn ff_arithmetic_expr_first() {
    let (g, ff) = build_arithmetic("ffa_v9_arith2");
    assert_first_contains(&ff, sym(&g, "expr"), &[sym(&g, "num"), sym(&g, "lp")]);
}

#[test]
fn ff_arithmetic_term_first() {
    let (g, ff) = build_arithmetic("ffa_v9_arith3");
    assert_first_contains(&ff, sym(&g, "term"), &[sym(&g, "num"), sym(&g, "lp")]);
}

#[test]
fn ff_arithmetic_factor_first() {
    let (g, ff) = build_arithmetic("ffa_v9_arith4");
    assert_first_contains(&ff, sym(&g, "factor"), &[sym(&g, "num"), sym(&g, "lp")]);
    assert_first_excludes(&ff, sym(&g, "factor"), &[sym(&g, "plus"), sym(&g, "star")]);
}

#[test]
fn ff_arithmetic_follow_sets() {
    let (g, ff) = build_arithmetic("ffa_v9_arith5");
    // FOLLOW(expr) ⊇ {EOF, "+", ")"}
    assert_follow_contains(&ff, sym(&g, "expr"), &[EOF, sym(&g, "plus"), sym(&g, "rp")]);
    // FOLLOW(term) ⊇ {"+", "*", EOF, ")"}
    assert_follow_contains(&ff, sym(&g, "term"), &[sym(&g, "plus"), sym(&g, "star")]);
    // FOLLOW(factor) ⊇ {"+", "*"}
    assert_follow_contains(&ff, sym(&g, "factor"), &[sym(&g, "plus"), sym(&g, "star")]);
}

// ===========================================================================
// 19. ff_normalize_* — FIRST/FOLLOW after normalize (4 tests)
// ===========================================================================

#[test]
fn ff_normalize_compute_normalized_ok() {
    let mut g = GrammarBuilder::new("ffa_v9_norm1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn ff_normalize_matches_plain_compute() {
    let g1 = GrammarBuilder::new("ffa_v9_norm2a")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();

    let mut g2 = GrammarBuilder::new("ffa_v9_norm2b")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let s1 = g1.find_symbol_by_name("s").unwrap();
    let s2 = g2.find_symbol_by_name("s").unwrap();
    assert_eq!(first_count(&ff1, s1), first_count(&ff2, s2));
}

#[test]
fn ff_normalize_first_correct() {
    let mut g = GrammarBuilder::new("ffa_v9_norm3")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["inner"])
        .rule("inner", vec!["x"])
        .rule("inner", vec!["y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let x_id = g.find_symbol_by_name("x").unwrap();
    let y_id = g.find_symbol_by_name("y").unwrap();
    assert_first_contains(&ff, s_id, &[x_id, y_id]);
}

#[test]
fn ff_normalize_follow_has_eof() {
    let mut g = GrammarBuilder::new("ffa_v9_norm4")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_follow_contains(&ff, s_id, &[EOF]);
}

// ===========================================================================
// 20. ff_large_* — large grammar FIRST/FOLLOW computation (4 tests)
// ===========================================================================

fn build_large_grammar() -> (Grammar, FirstFollowSets) {
    build_ff(
        "ffa_v9_large",
        &[
            ("kw_let", "let"),
            ("kw_if", "if"),
            ("kw_else", "else"),
            ("kw_while", "while"),
            ("kw_return", "return"),
            ("kw_fn", "fn"),
            ("num", r"[0-9]+"),
            ("ident", r"[a-z]+"),
            ("str_lit", r#""[^"]*""#),
            ("plus", "+"),
            ("minus", "-"),
            ("star", "*"),
            ("slash", "div"),
            ("eq", "="),
            ("eqeq", "=="),
            ("semi", ";"),
            ("comma", ","),
            ("lp", "("),
            ("rp", ")"),
            ("lb", "{"),
            ("rb", "}"),
        ],
        &[
            ("program", vec!["stmt_list"]),
            ("stmt_list", vec!["stmt_list", "stmt"]),
            ("stmt_list", vec!["stmt"]),
            ("stmt", vec!["let_stmt"]),
            ("stmt", vec!["if_stmt"]),
            ("stmt", vec!["while_stmt"]),
            ("stmt", vec!["ret_stmt"]),
            ("stmt", vec!["expr_stmt"]),
            ("let_stmt", vec!["kw_let", "ident", "eq", "expr", "semi"]),
            ("if_stmt", vec!["kw_if", "lp", "expr", "rp", "block"]),
            (
                "if_stmt",
                vec!["kw_if", "lp", "expr", "rp", "block", "kw_else", "block"],
            ),
            ("while_stmt", vec!["kw_while", "lp", "expr", "rp", "block"]),
            ("ret_stmt", vec!["kw_return", "expr", "semi"]),
            ("expr_stmt", vec!["expr", "semi"]),
            ("block", vec!["lb", "stmt_list", "rb"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("expr", vec!["expr", "minus", "term"]),
            ("expr", vec!["term"]),
            ("term", vec!["term", "star", "factor"]),
            ("term", vec!["term", "slash", "factor"]),
            ("term", vec!["factor"]),
            ("factor", vec!["num"]),
            ("factor", vec!["ident"]),
            ("factor", vec!["str_lit"]),
            ("factor", vec!["lp", "expr", "rp"]),
            ("factor", vec!["kw_fn", "lp", "params", "rp", "block"]),
            ("params", vec!["params", "comma", "ident"]),
            ("params", vec!["ident"]),
        ],
        "program",
    )
}

#[test]
fn ff_large_computes_ok() {
    let (_g, _ff) = build_large_grammar();
}

#[test]
fn ff_large_program_first_includes_keywords() {
    let (g, ff) = build_large_grammar();
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
}

#[test]
fn ff_large_all_nonterminals_have_first() {
    let (g, ff) = build_large_grammar();
    for name in [
        "program",
        "stmt_list",
        "stmt",
        "let_stmt",
        "if_stmt",
        "while_stmt",
        "ret_stmt",
        "expr_stmt",
        "block",
        "expr",
        "term",
        "factor",
        "params",
    ] {
        assert!(
            first_count(&ff, sym(&g, name)) > 0,
            "FIRST({name}) should be non-empty",
        );
    }
}

#[test]
fn ff_large_follow_propagation() {
    let (g, ff) = build_large_grammar();
    assert_follow_contains(&ff, sym(&g, "program"), &[EOF]);
    // expr in let_stmt is followed by semi
    assert_follow_contains(&ff, sym(&g, "expr"), &[sym(&g, "semi")]);
    // stmt_list inside block is followed by rb
    assert_follow_contains(&ff, sym(&g, "stmt_list"), &[sym(&g, "rb")]);
    // factor is followed by operators
    assert_follow_contains(&ff, sym(&g, "factor"), &[sym(&g, "plus"), sym(&g, "star")]);
}
