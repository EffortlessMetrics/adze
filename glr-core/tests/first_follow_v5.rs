//! FIRST/FOLLOW set computation tests (v5) using the GrammarBuilder API.
//!
//! Categories:
//! 1. FIRST sets for terminal-only rules (single and multi-token)
//! 2. FIRST sets for nonterminal rules (propagation through rule chains)
//! 3. FIRST sets with epsilon (nullable nonterminals)
//! 4. FOLLOW sets include EOF for start symbol
//! 5. FOLLOW sets propagation through rules
//! 6. Complex grammar FIRST/FOLLOW (arithmetic, JSON-like)
//! 7. Edge cases: single rule, many alternatives, left recursion
//! 8. Determinism: same grammar → same sets

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EOF: SymbolId = SymbolId(0);

/// Look up a symbol by name in the grammar.
fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.find_symbol_by_name(name)
        .unwrap_or_else(|| panic!("symbol '{name}' not found"))
}

/// Assert that FIRST(symbol) contains exactly the given symbol IDs.
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

/// Assert that FIRST(symbol) contains at least the given symbol IDs.
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

/// Assert that FIRST(symbol) does NOT contain the given symbol IDs.
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

/// Assert that FOLLOW(symbol) contains at least the given symbol IDs.
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

/// Assert that FOLLOW(symbol) contains exactly the given symbol IDs.
fn assert_follow_eq(ff: &FirstFollowSets, id: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .follow(id)
        .unwrap_or_else(|| panic!("no FOLLOW set for {id:?}"));
    let actual: Vec<u16> = (0..set.len())
        .filter(|&i| set.contains(i))
        .map(|i| i as u16)
        .collect();
    let mut exp: Vec<u16> = expected.iter().map(|s| s.0).collect();
    exp.sort();
    assert_eq!(actual, exp, "FOLLOW({id:?}) mismatch");
}

/// Assert that FOLLOW(symbol) does NOT contain the given symbol IDs.
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

// ===========================================================================
// 1. FIRST sets for terminal-only rules (single and multi-token) — 7 tests
// ===========================================================================

#[test]
fn first_terminal_single_token() {
    // S -> a  =>  FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_terminal_multi_token_only_leading() {
    // S -> a b  =>  FIRST(S) = {a}, b excluded
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_contains(&ff, start, &[sym(&g, "a")]);
    assert_first_excludes(&ff, start, &[sym(&g, "b")]);
}

#[test]
fn first_terminal_regex_pattern() {
    // S -> num  =>  FIRST(S) = {num}
    let g = GrammarBuilder::new("t")
        .token("num", r"[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "num")]);
}

#[test]
fn first_terminal_keyword() {
    // S -> kw_if  =>  FIRST(S) = {kw_if}
    let g = GrammarBuilder::new("t")
        .token("kw_if", "if")
        .rule("start", vec!["kw_if"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "kw_if")]);
}

#[test]
fn first_terminal_each_nonterminal_reflects_its_terminal() {
    // S -> x, W -> y, V -> z  =>  each has its own terminal in FIRST
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["x"])
        .rule("wrap_y", vec!["y"])
        .rule("wrap_z", vec!["z"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "x")]);
    assert_first_eq(&ff, sym(&g, "wrap_y"), &[sym(&g, "y")]);
    assert_first_eq(&ff, sym(&g, "wrap_z"), &[sym(&g, "z")]);
}

#[test]
fn first_terminal_punctuation() {
    // S -> semi  =>  FIRST(S) = {semi}
    let g = GrammarBuilder::new("t")
        .token("semi", ";")
        .rule("start", vec!["semi"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "semi")]);
}

#[test]
fn first_terminal_three_token_rule() {
    // S -> num plus num  =>  FIRST(S) = {num}, plus excluded
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .rule("start", vec!["num", "plus", "num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_contains(&ff, start, &[sym(&g, "num")]);
    assert_first_excludes(&ff, start, &[sym(&g, "plus")]);
}

// ===========================================================================
// 2. FIRST sets for nonterminal rules (propagation through chains) — 7 tests
// ===========================================================================

#[test]
fn first_nonterminal_single_chain() {
    // S -> A, A -> a  =>  FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_nonterminal_deep_chain() {
    // S -> A, A -> B, B -> c  =>  FIRST(S) = {c}
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "c")]);
}

#[test]
fn first_nonterminal_two_alternatives_via_nonterminals() {
    // S -> A | B, A -> a, B -> b  =>  FIRST(S) = {a, b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["aa"])
        .rule("start", vec!["bb"])
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_nonterminal_sequence_only_leading() {
    // S -> A B c, A -> a, B -> b  =>  FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["left", "right", "c"])
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_contains(&ff, start, &[sym(&g, "a")]);
    assert_first_excludes(&ff, start, &[sym(&g, "b"), sym(&g, "c")]);
}

#[test]
fn first_nonterminal_four_level_chain() {
    // S -> A, A -> B, B -> C, C -> x | y
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["la"])
        .rule("la", vec!["lb"])
        .rule("lb", vec!["lc"])
        .rule("lc", vec!["x"])
        .rule("lc", vec!["y"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let xy = &[sym(&g, "x"), sym(&g, "y")];
    assert_first_contains(&ff, sym(&g, "start"), xy);
    assert_first_contains(&ff, sym(&g, "la"), xy);
    assert_first_contains(&ff, sym(&g, "lb"), xy);
    assert_first_contains(&ff, sym(&g, "lc"), xy);
}

#[test]
fn first_nonterminal_right_recursion_propagates() {
    // S -> a S | a  =>  FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_nonterminal_direct_alternatives() {
    // S -> a | b  =>  FIRST(S) = {a, b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

// ===========================================================================
// 3. FIRST sets with epsilon (nullable nonterminals) — 7 tests
// ===========================================================================

#[test]
fn first_epsilon_nullable_with_terminal() {
    // S -> ε | a  =>  S is nullable, FIRST(S) includes a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert!(ff.is_nullable(start));
    assert_first_contains(&ff, start, &[sym(&g, "a")]);
}

#[test]
fn first_epsilon_chain_nullable() {
    // S -> A, A -> ε  =>  both nullable
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "mid")));
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_epsilon_nullable_prefix_reveals_next() {
    // S -> A b, A -> ε  =>  FIRST(S) includes b
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .rule("start", vec!["opt", "b"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "b")]);
}

#[test]
fn first_epsilon_two_nullable_prefixes() {
    // S -> A B c, A -> ε, B -> ε  =>  FIRST(S) includes c
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["opt1", "opt2", "c"])
        .rule("opt1", vec![])
        .rule("opt2", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "c")]);
}

#[test]
fn first_epsilon_nullable_alternative_union() {
    // S -> A | b, A -> ε | c  =>  FIRST(S) includes {b, c}
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["inner"])
        .rule("start", vec!["b"])
        .rule("inner", vec![])
        .rule("inner", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "b"), sym(&g, "c")]);
}

#[test]
fn first_epsilon_all_rhs_nullable() {
    // S -> A B, A -> ε, B -> ε  =>  S is nullable
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["opt_a", "opt_b"])
        .rule("opt_a", vec![])
        .rule("opt_b", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_epsilon_nullable_prefix_union_with_terminal() {
    // S -> A b, A -> c | ε  =>  FIRST(S) = {c, b}
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["opt", "b"])
        .rule("opt", vec!["c"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "b"), sym(&g, "c")]);
}

// ===========================================================================
// 4. FOLLOW sets include EOF for start symbol — 7 tests
// ===========================================================================

#[test]
fn follow_eof_minimal() {
    // S -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_two_alternatives() {
    // S -> a | b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_through_chain() {
    // S -> A, A -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_left_recursive() {
    // S -> S a | b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_nullable_start() {
    // S -> ε | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_multi_token_rule() {
    // S -> a b c
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_right_recursive() {
    // S -> a S | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

// ===========================================================================
// 5. FOLLOW sets propagation through rules — 8 tests
// ===========================================================================

#[test]
fn follow_prop_trailing_terminal() {
    // S -> A b  =>  FOLLOW(A) includes b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "b")]);
}

#[test]
fn follow_prop_nonterminal_first_set() {
    // S -> A B, B -> b  =>  FOLLOW(A) includes FIRST(B) = {b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["left", "right"])
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "left"), &[sym(&g, "b")]);
}

#[test]
fn follow_prop_end_of_rule_inherits_parent() {
    // S -> A, A -> a  =>  FOLLOW(A) includes FOLLOW(S) = {EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[EOF]);
}

#[test]
fn follow_prop_through_nullable_suffix() {
    // S -> A B, B -> ε  =>  FOLLOW(A) includes FOLLOW(S) = {EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["left", "opt"])
        .rule("left", vec!["a"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "left"), &[EOF]);
}

#[test]
fn follow_prop_multiple_contexts() {
    // S -> A b | A c  =>  FOLLOW(A) includes {b, c}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["inner", "b"])
        .rule("start", vec!["inner", "c"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "b"), sym(&g, "c")]);
}

#[test]
fn follow_prop_chain_through_nonterminals() {
    // S -> A b, A -> B, B -> c  =>  FOLLOW(B) includes b via chain
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["mid", "b"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "leaf"), &[sym(&g, "b")]);
}

#[test]
fn follow_prop_excludes_unrelated() {
    // S -> A b, A -> a  =>  FOLLOW(A) should NOT contain a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_excludes(&ff, sym(&g, "inner"), &[sym(&g, "a")]);
}

#[test]
fn follow_prop_recursive_includes_self_context() {
    // S -> S a | b  =>  FOLLOW(S) includes {a, EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), EOF]);
}

// ===========================================================================
// 6. Complex grammar FIRST/FOLLOW (arithmetic, JSON-like) — 8 tests
// ===========================================================================

#[test]
fn complex_arithmetic_first() {
    // E -> E + T | T, T -> T * F | F, F -> num
    let g = GrammarBuilder::new("t")
        .token("num", r"[0-9]+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let num = sym(&g, "num");
    assert_first_contains(&ff, sym(&g, "expr"), &[num]);
    assert_first_contains(&ff, sym(&g, "term"), &[num]);
    assert_first_contains(&ff, sym(&g, "factor"), &[num]);
}

#[test]
fn complex_arithmetic_follow() {
    // E -> E + T | T, T -> T * F | F, F -> num
    let g = GrammarBuilder::new("t")
        .token("num", r"[0-9]+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let plus = sym(&g, "plus");
    let star = sym(&g, "star");
    assert_follow_contains(&ff, sym(&g, "expr"), &[plus, EOF]);
    assert_follow_contains(&ff, sym(&g, "term"), &[plus, star, EOF]);
    assert_follow_contains(&ff, sym(&g, "factor"), &[plus, star, EOF]);
}

#[test]
fn complex_json_value_first() {
    // value -> obj | arr | str_tok | num_tok | kw_true | kw_false
    let g = GrammarBuilder::new("t")
        .token("lbrace", "{")
        .token("rbrace", "}")
        .token("lbrack", "[")
        .token("rbrack", "]")
        .token("str_tok", r#""[^"]*""#)
        .token("num_tok", r"[0-9]+")
        .token("kw_true", "true")
        .token("kw_false", "false")
        .rule("value", vec!["obj"])
        .rule("value", vec!["arr"])
        .rule("value", vec!["str_tok"])
        .rule("value", vec!["num_tok"])
        .rule("value", vec!["kw_true"])
        .rule("value", vec!["kw_false"])
        .rule("obj", vec!["lbrace", "rbrace"])
        .rule("arr", vec!["lbrack", "rbrack"])
        .start("value")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(
        &ff,
        sym(&g, "value"),
        &[
            sym(&g, "lbrace"),
            sym(&g, "lbrack"),
            sym(&g, "str_tok"),
            sym(&g, "num_tok"),
            sym(&g, "kw_true"),
            sym(&g, "kw_false"),
        ],
    );
}

#[test]
fn complex_json_follow() {
    // obj and arr at end of value rules => inherit FOLLOW(value)
    let g = GrammarBuilder::new("t")
        .token("lbrace", "{")
        .token("rbrace", "}")
        .token("lbrack", "[")
        .token("rbrack", "]")
        .token("str_tok", r#""[^"]*""#)
        .token("num_tok", r"[0-9]+")
        .token("kw_true", "true")
        .token("kw_false", "false")
        .rule("value", vec!["obj"])
        .rule("value", vec!["arr"])
        .rule("value", vec!["str_tok"])
        .rule("value", vec!["num_tok"])
        .rule("value", vec!["kw_true"])
        .rule("value", vec!["kw_false"])
        .rule("obj", vec!["lbrace", "rbrace"])
        .rule("arr", vec!["lbrack", "rbrack"])
        .start("value")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "obj"), &[EOF]);
    assert_follow_contains(&ff, sym(&g, "arr"), &[EOF]);
}

#[test]
fn complex_statement_list() {
    // program -> stmts, stmts -> stmt stmts | stmt, stmt -> kw semi
    let g = GrammarBuilder::new("t")
        .token("kw", "return")
        .token("semi", ";")
        .rule("program", vec!["stmts"])
        .rule("stmts", vec!["stmt", "stmts"])
        .rule("stmts", vec!["stmt"])
        .rule("stmt", vec!["kw", "semi"])
        .start("program")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let kw = sym(&g, "kw");
    assert_first_contains(&ff, sym(&g, "program"), &[kw]);
    assert_first_contains(&ff, sym(&g, "stmts"), &[kw]);
    assert_first_contains(&ff, sym(&g, "stmt"), &[kw]);
    // stmt can be followed by another stmt (via stmts) or EOF
    assert_follow_contains(&ff, sym(&g, "stmt"), &[kw, EOF]);
}

#[test]
fn complex_if_else() {
    // S -> kw_if cond body else_part
    // else_part -> kw_else body | ε
    let g = GrammarBuilder::new("t")
        .token("kw_if", "if")
        .token("kw_else", "else")
        .token("ident", "x")
        .rule("start", vec!["kw_if", "cond", "body", "else_part"])
        .rule("else_part", vec!["kw_else", "body"])
        .rule("else_part", vec![])
        .rule("cond", vec!["ident"])
        .rule("body", vec!["ident"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let else_part = sym(&g, "else_part");
    assert!(ff.is_nullable(else_part));
    assert_first_contains(&ff, else_part, &[sym(&g, "kw_else")]);
    assert_follow_contains(&ff, else_part, &[EOF]);
}

#[test]
fn complex_nested_parens() {
    // S -> ( S ) | a
    let g = GrammarBuilder::new("t")
        .token("lp", "(")
        .token("rp", ")")
        .token("a", "a")
        .rule("start", vec!["lp", "start", "rp"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_contains(&ff, start, &[sym(&g, "lp"), sym(&g, "a")]);
    assert_follow_contains(&ff, start, &[sym(&g, "rp"), EOF]);
}

#[test]
fn complex_optional_list_nullable_items() {
    // S -> items, items -> items item | ε, item -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["items"])
        .rule("items", vec!["items", "item"])
        .rule("items", vec![])
        .rule("item", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let items = sym(&g, "items");
    assert!(ff.is_nullable(items));
    assert_first_contains(&ff, items, &[sym(&g, "a")]);
}

// ===========================================================================
// 7. Edge cases: single rule, many alternatives, left recursion — 8 tests
// ===========================================================================

#[test]
fn edge_single_rule_grammar() {
    // Simplest possible grammar: S -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert_first_eq(&ff, start, &[a]);
    assert_follow_contains(&ff, start, &[EOF]);
    assert!(!ff.is_nullable(start));
}

#[test]
fn edge_many_alternatives() {
    // S -> a | b | c | d | e | f
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .rule("start", vec!["f"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_contains(
        &ff,
        start,
        &[
            sym(&g, "a"),
            sym(&g, "b"),
            sym(&g, "c"),
            sym(&g, "d"),
            sym(&g, "e"),
            sym(&g, "f"),
        ],
    );
}

#[test]
fn edge_left_recursion_first_and_follow() {
    // S -> S a | b  =>  FIRST(S) = {b}, FOLLOW(S) = {a, EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[b]);
    assert_first_excludes(&ff, start, &[a]);
    assert_follow_contains(&ff, start, &[a, EOF]);
}

#[test]
fn edge_mutual_recursion() {
    // A -> B c, B -> A d | e  =>  both derive e first
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("aa", vec!["bb", "c"])
        .rule("bb", vec!["aa", "d"])
        .rule("bb", vec!["e"])
        .start("aa")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let e = sym(&g, "e");
    assert_first_contains(&ff, sym(&g, "aa"), &[e]);
    assert_first_contains(&ff, sym(&g, "bb"), &[e]);
}

#[test]
fn edge_epsilon_only_grammar() {
    // S -> ε  =>  nullable, FOLLOW(S) = {EOF}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert!(ff.is_nullable(start));
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn edge_middle_recursion() {
    // S -> a S b | c  =>  FIRST(S) = {a, c}, FOLLOW(S) = {b, EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "start", "b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_contains(&ff, start, &[sym(&g, "a"), sym(&g, "c")]);
    assert_follow_contains(&ff, start, &[sym(&g, "b"), EOF]);
}

#[test]
fn edge_nullable_chain_with_follow_propagation() {
    // S -> A B c, A -> ε, B -> ε  =>  FIRST(S) = {c}, FOLLOW(A) includes {c}
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["opt_a", "opt_b", "c"])
        .rule("opt_a", vec![])
        .rule("opt_b", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "c")]);
    assert_follow_contains(&ff, sym(&g, "opt_a"), &[sym(&g, "c")]);
}

#[test]
fn edge_long_rhs_follow() {
    // S -> a b c d e  =>  FOLLOW(S) = {EOF}, FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_first_eq(&ff, start, &[sym(&g, "a")]);
    assert_follow_eq(&ff, start, &[EOF]);
}

// ===========================================================================
// 8. Determinism: same grammar → same sets — 7 tests
// ===========================================================================

#[test]
fn determinism_first_simple() {
    let build = || {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    assert_eq!(
        ff1.first(sym(&g1, "start")).unwrap(),
        ff2.first(sym(&g2, "start")).unwrap(),
    );
}

#[test]
fn determinism_follow_simple() {
    let build = || {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["inner", "b"])
            .rule("inner", vec!["a"])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    assert_eq!(
        ff1.follow(sym(&g1, "inner")).unwrap(),
        ff2.follow(sym(&g2, "inner")).unwrap(),
    );
}

#[test]
fn determinism_nullable() {
    let build = || {
        GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec![])
            .rule("start", vec!["a"])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    assert_eq!(
        ff1.is_nullable(sym(&g1, "start")),
        ff2.is_nullable(sym(&g2, "start")),
    );
}

#[test]
fn determinism_arithmetic_grammar() {
    let build = || {
        GrammarBuilder::new("t")
            .token("num", r"[0-9]+")
            .token("plus", "+")
            .token("star", "*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    for name in &["expr", "term", "factor"] {
        let s1 = sym(&g1, name);
        let s2 = sym(&g2, name);
        assert_eq!(
            ff1.first(s1).unwrap(),
            ff2.first(s2).unwrap(),
            "FIRST({name}) differs",
        );
        assert_eq!(
            ff1.follow(s1).unwrap(),
            ff2.follow(s2).unwrap(),
            "FOLLOW({name}) differs",
        );
    }
}

#[test]
fn determinism_chain_grammar() {
    let build = || {
        GrammarBuilder::new("t")
            .token("x", "x")
            .rule("start", vec!["mid"])
            .rule("mid", vec!["leaf"])
            .rule("leaf", vec!["x"])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    for name in &["start", "mid", "leaf"] {
        let s1 = sym(&g1, name);
        let s2 = sym(&g2, name);
        assert_eq!(ff1.first(s1).unwrap(), ff2.first(s2).unwrap());
    }
}

#[test]
fn determinism_nullable_complex() {
    let build = || {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["opt", "b"])
            .rule("opt", vec!["a"])
            .rule("opt", vec![])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    assert_eq!(
        ff1.first(sym(&g1, "start")).unwrap(),
        ff2.first(sym(&g2, "start")).unwrap(),
    );
    assert_eq!(
        ff1.follow(sym(&g1, "opt")).unwrap(),
        ff2.follow(sym(&g2, "opt")).unwrap(),
    );
    assert_eq!(
        ff1.is_nullable(sym(&g1, "opt")),
        ff2.is_nullable(sym(&g2, "opt")),
    );
}

#[test]
fn determinism_compute_vs_normalized() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff1 = FirstFollowSets::compute(&g).unwrap();

    let mut g2 = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    assert_eq!(
        ff1.first(sym(&g, "start")).unwrap(),
        ff2.first(sym(&g2, "start")).unwrap(),
    );
}
