//! FIRST/FOLLOW set computation tests using the GrammarBuilder API.
//!
//! Categories:
//! 1. FIRST of terminal is itself
//! 2. FIRST of nonterminal
//! 3. FIRST with epsilon / nullable
//! 4. FOLLOW of start symbol
//! 5. FOLLOW propagation
//! 6. compute vs compute_normalized
//! 7. Complex grammar FIRST/FOLLOW

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EOF: SymbolId = SymbolId(0);

/// Look up a symbol by name in the grammar's rule_names map.
fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.find_symbol_by_name(name)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in rule_names"))
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
// 1. FIRST of terminal is itself (8 tests)
//
// Terminals are not tracked in the FIRST map; we verify the invariant
// indirectly: a nonterminal whose sole production is a single terminal
// must have FIRST = {that terminal}.
// ===========================================================================

#[test]
fn first_terminal_single_token_via_nonterminal() {
    // S -> a  =>  FIRST(S) = {a}, proving terminal a contributes itself
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert_first_eq(&ff, start, &[a]);
}

#[test]
fn first_terminal_digit_via_nonterminal() {
    // S -> num  =>  FIRST(S) = {num}
    let g = GrammarBuilder::new("t")
        .token("num", r"[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let num = sym(&g, "num");
    assert_first_eq(&ff, start, &[num]);
}

#[test]
fn first_terminal_keyword_via_nonterminal() {
    // S -> kw_if  =>  FIRST(S) = {kw_if}
    let g = GrammarBuilder::new("t")
        .token("kw_if", "if")
        .rule("start", vec!["kw_if"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let kw = sym(&g, "kw_if");
    assert_first_eq(&ff, start, &[kw]);
}

#[test]
fn first_terminal_each_contributes_itself() {
    // S -> x,  A -> y,  B -> z  =>  FIRST matches respective terminal
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
    let start = sym(&g, "start");
    let wy = sym(&g, "wrap_y");
    let wz = sym(&g, "wrap_z");
    let x = sym(&g, "x");
    let y = sym(&g, "y");
    let z = sym(&g, "z");
    assert_first_eq(&ff, start, &[x]);
    assert_first_eq(&ff, wy, &[y]);
    assert_first_eq(&ff, wz, &[z]);
}

#[test]
fn first_terminal_regex_via_nonterminal() {
    // S -> ident  =>  FIRST(S) = {ident}
    let g = GrammarBuilder::new("t")
        .token("ident", r"[a-z]+")
        .rule("start", vec!["ident"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let id = sym(&g, "ident");
    assert_first_eq(&ff, start, &[id]);
}

#[test]
fn first_terminal_only_leading_contributes() {
    // S -> a b  =>  FIRST(S) = {a} (b does NOT appear)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[a]);
    let fs = ff.first(start).unwrap();
    assert!(
        !fs.contains(b.0 as usize),
        "FIRST(start) must not contain b"
    );
}

#[test]
fn first_terminal_string_literal_via_nonterminal() {
    // S -> semi  =>  FIRST(S) = {semi}
    let g = GrammarBuilder::new("t")
        .token("semi", ";")
        .rule("start", vec!["semi"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let semi = sym(&g, "semi");
    assert_first_eq(&ff, start, &[semi]);
}

#[test]
fn first_terminal_operator_via_nonterminal() {
    // S -> num plus num  =>  FIRST(S) = {num}, plus is not in FIRST
    let g = GrammarBuilder::new("t")
        .token("plus", "+")
        .token("num", "0")
        .rule("start", vec!["num", "plus", "num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let num = sym(&g, "num");
    let plus = sym(&g, "plus");
    assert_first_contains(&ff, start, &[num]);
    let fs = ff.first(start).unwrap();
    assert!(
        !fs.contains(plus.0 as usize),
        "plus should not be in FIRST(start)"
    );
}

// ===========================================================================
// 2. FIRST of nonterminal (8 tests)
// ===========================================================================

#[test]
fn first_nonterminal_single_production() {
    // S -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert_first_contains(&ff, start, &[a]);
}

#[test]
fn first_nonterminal_two_alternatives() {
    // S -> a | b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[a, b]);
}

#[test]
fn first_nonterminal_chain() {
    // S -> A, A -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["item"])
        .rule("item", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert_first_contains(&ff, start, &[a]);
}

#[test]
fn first_nonterminal_deep_chain() {
    // S -> A, A -> B, B -> c
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let c = sym(&g, "c");
    assert_first_contains(&ff, start, &[c]);
}

#[test]
fn first_nonterminal_left_recursion() {
    // S -> S a | b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[b]);
}

#[test]
fn first_nonterminal_multiple_levels() {
    // S -> A | B,  A -> a,  B -> b
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
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[a, b]);
}

#[test]
fn first_nonterminal_skips_second_symbol() {
    // S -> a b   =>  FIRST(S) = {a}, not {a, b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[a]);
    let fs = ff.first(start).unwrap();
    assert!(
        !fs.contains(b.0 as usize),
        "FIRST(start) should not contain b"
    );
}

#[test]
fn first_nonterminal_right_recursion() {
    // S -> a S | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert_first_contains(&ff, start, &[a]);
}

// ===========================================================================
// 3. FIRST with epsilon / nullable (7 tests)
// ===========================================================================

#[test]
fn first_epsilon_nullable_rule() {
    // S -> ε
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert!(ff.is_nullable(start), "start should be nullable");
}

#[test]
fn first_epsilon_nonterminal_also_has_terminal() {
    // S -> a | ε  =>  FIRST(S) includes a, and S is nullable
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert!(ff.is_nullable(start));
    assert_first_contains(&ff, start, &[a]);
}

#[test]
fn first_epsilon_chain_nullable() {
    // S -> A, A -> ε   => S is nullable
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let mid = sym(&g, "mid");
    assert!(ff.is_nullable(mid));
    assert!(ff.is_nullable(start));
}

#[test]
fn first_through_nullable_prefix() {
    // S -> A b,  A -> ε   => FIRST(S) includes b (through nullable A)
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .rule("start", vec!["opt", "b"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[b]);
}

#[test]
fn first_through_two_nullable_prefixes() {
    // S -> A B c,  A -> ε,  B -> ε   => FIRST(S) includes c
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["opt1", "opt2", "c"])
        .rule("opt1", vec![])
        .rule("opt2", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let c = sym(&g, "c");
    assert_first_contains(&ff, start, &[c]);
}

#[test]
fn first_nullable_with_nonterminal_alternative() {
    // S -> A | b,  A -> ε | c   =>  FIRST(S) includes {b, c}
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
    let start = sym(&g, "start");
    let b = sym(&g, "b");
    let c = sym(&g, "c");
    assert_first_contains(&ff, start, &[b, c]);
}

#[test]
fn first_all_rhs_nullable_makes_lhs_nullable() {
    // S -> A B,  A -> ε,  B -> ε   => S is nullable
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["opt_a", "opt_b"])
        .rule("opt_a", vec![])
        .rule("opt_b", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert!(ff.is_nullable(start));
}

// ===========================================================================
// 4. FOLLOW of start symbol (8 tests)
// ===========================================================================

#[test]
fn follow_start_has_eof_minimal() {
    // S -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_two_rules() {
    // S -> a | b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_chain() {
    // S -> A,  A -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_recursive() {
    // S -> S a | b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_nullable() {
    // S -> ε | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_multi_token() {
    // S -> a b c
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_with_nonterminal_child() {
    // S -> A b,  A -> a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

#[test]
fn follow_start_has_eof_right_recursive() {
    // S -> a S | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    assert_follow_contains(&ff, start, &[EOF]);
}

// ===========================================================================
// 5. FOLLOW propagation (8 tests)
// ===========================================================================

#[test]
fn follow_from_trailing_terminal() {
    // S -> A b   => FOLLOW(A) includes b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let inner = sym(&g, "inner");
    let b = sym(&g, "b");
    assert_follow_contains(&ff, inner, &[b]);
}

#[test]
fn follow_from_trailing_nonterminal_first() {
    // S -> A B,  B -> b   => FOLLOW(A) includes FIRST(B) = {b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["left", "right"])
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let left = sym(&g, "left");
    let b = sym(&g, "b");
    assert_follow_contains(&ff, left, &[b]);
}

#[test]
fn follow_propagates_from_parent_at_end() {
    // S -> A,  A -> a   => FOLLOW(A) includes FOLLOW(S) = {EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let inner = sym(&g, "inner");
    assert_follow_contains(&ff, inner, &[EOF]);
}

#[test]
fn follow_through_nullable_suffix() {
    // S -> A B,  B -> ε   => FOLLOW(A) includes FOLLOW(S)={EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["left", "opt"])
        .rule("left", vec!["a"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let left = sym(&g, "left");
    assert_follow_contains(&ff, left, &[EOF]);
}

#[test]
fn follow_multiple_contexts() {
    // S -> A b | A c   => FOLLOW(A) includes {b, c}
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
    let inner = sym(&g, "inner");
    let b = sym(&g, "b");
    let c = sym(&g, "c");
    assert_follow_contains(&ff, inner, &[b, c]);
}

#[test]
fn follow_chain_propagation() {
    // S -> A b,  A -> B,  B -> c   => FOLLOW(B) includes FOLLOW(A) which includes b
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["mid", "b"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let leaf = sym(&g, "leaf");
    let b = sym(&g, "b");
    assert_follow_contains(&ff, leaf, &[b]);
}

#[test]
fn follow_does_not_include_unrelated() {
    // S -> A b,  A -> a   => FOLLOW(A) should NOT contain a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let inner = sym(&g, "inner");
    let a = sym(&g, "a");
    assert_follow_excludes(&ff, inner, &[a]);
}

#[test]
fn follow_recursive_includes_self_context() {
    // S -> S a | b   => FOLLOW(S) includes {a, EOF}
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
    assert_follow_contains(&ff, start, &[a, EOF]);
}

// ===========================================================================
// 6. compute vs compute_normalized (8 tests)
// ===========================================================================

#[test]
fn compute_and_normalized_both_succeed() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff1 = FirstFollowSets::compute(&g);
    assert!(ff1.is_ok(), "compute should succeed");

    let mut g2 = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2);
    assert!(ff2.is_ok(), "compute_normalized should succeed");
}

#[test]
fn compute_and_normalized_agree_on_first_simple() {
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

    let start1 = sym(&g, "start");
    let start2 = sym(&g2, "start");
    let a1 = sym(&g, "a");
    let a2 = sym(&g2, "a");

    assert_first_contains(&ff1, start1, &[a1]);
    assert_first_contains(&ff2, start2, &[a2]);
}

#[test]
fn compute_and_normalized_agree_on_nullable() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff1 = FirstFollowSets::compute(&g).unwrap();

    let mut g2 = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let s1 = sym(&g, "start");
    let s2 = sym(&g2, "start");
    assert!(ff1.is_nullable(s1));
    assert!(ff2.is_nullable(s2));
}

#[test]
fn compute_and_normalized_agree_on_eof_in_follow() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let ff1 = FirstFollowSets::compute(&g).unwrap();

    let mut g2 = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let s1 = sym(&g, "start");
    let s2 = sym(&g2, "start");
    assert_follow_contains(&ff1, s1, &[EOF]);
    assert_follow_contains(&ff2, s2, &[EOF]);
}

#[test]
fn compute_and_normalized_chain_grammar() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("start")
        .build();

    let ff1 = FirstFollowSets::compute(&g).unwrap();

    let mut g2 = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["x"])
        .start("start")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let start1 = sym(&g, "start");
    let start2 = sym(&g2, "start");
    let x1 = sym(&g, "x");
    let x2 = sym(&g2, "x");
    assert_first_contains(&ff1, start1, &[x1]);
    assert_first_contains(&ff2, start2, &[x2]);
}

#[test]
fn normalized_handles_recursive_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let start = sym(&g, "start");
    let b = sym(&g, "b");
    assert_first_contains(&ff, start, &[b]);
}

#[test]
fn normalized_handles_nullable_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["opt", "a"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    assert_first_contains(&ff, start, &[a]);
}

#[test]
fn normalized_handles_multiple_alternatives() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let c = sym(&g, "c");
    assert_first_contains(&ff, start, &[a, b, c]);
}

// ===========================================================================
// 7. Complex grammar FIRST/FOLLOW (8 tests)
// ===========================================================================

#[test]
fn complex_expr_grammar_first() {
    // E -> E + T | T,  T -> T * F | F,  F -> num
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
    let expr = sym(&g, "expr");
    let term = sym(&g, "term");
    let factor = sym(&g, "factor");
    let num = sym(&g, "num");
    // All nonterminals ultimately derive num first
    assert_first_contains(&ff, expr, &[num]);
    assert_first_contains(&ff, term, &[num]);
    assert_first_contains(&ff, factor, &[num]);
}

#[test]
fn complex_expr_grammar_follow() {
    // E -> E + T | T,  T -> T * F | F,  F -> num
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
    let expr = sym(&g, "expr");
    let term = sym(&g, "term");
    let factor = sym(&g, "factor");
    let plus = sym(&g, "plus");
    let star = sym(&g, "star");
    // FOLLOW(expr) = {+, EOF}
    assert_follow_contains(&ff, expr, &[plus, EOF]);
    // FOLLOW(term) = {+, *, EOF}
    assert_follow_contains(&ff, term, &[plus, star, EOF]);
    // FOLLOW(factor) should include {+, *, EOF}
    assert_follow_contains(&ff, factor, &[plus, star, EOF]);
}

#[test]
fn complex_optional_list() {
    // S -> items,  items -> items item | ε,  item -> a
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
    let a = sym(&g, "a");
    assert!(ff.is_nullable(items));
    assert_first_contains(&ff, items, &[a]);
}

#[test]
fn complex_mutual_first_propagation() {
    // S -> A B c,  A -> a | ε,  B -> b | ε
    // FIRST(S) = {a, b, c}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["opt_a", "opt_b", "c"])
        .rule("opt_a", vec!["a"])
        .rule("opt_a", vec![])
        .rule("opt_b", vec!["b"])
        .rule("opt_b", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let c = sym(&g, "c");
    assert_first_contains(&ff, start, &[a, b, c]);
}

#[test]
fn complex_statement_list_grammar() {
    // program -> stmts,  stmts -> stmt stmts | stmt,  stmt -> kw semi
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
    let program = sym(&g, "program");
    let stmts = sym(&g, "stmts");
    let stmt = sym(&g, "stmt");
    let kw = sym(&g, "kw");
    // FIRST propagates through to kw
    assert_first_contains(&ff, program, &[kw]);
    assert_first_contains(&ff, stmts, &[kw]);
    assert_first_contains(&ff, stmt, &[kw]);
    // FOLLOW(stmt) should include FIRST(stmts)={kw} and FOLLOW(stmts)
    assert_follow_contains(&ff, stmt, &[kw, EOF]);
}

#[test]
fn complex_if_else_grammar() {
    // S -> kw_if cond body else_part
    // else_part -> kw_else body | ε
    // cond -> ident,  body -> ident
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
    let kw_else = sym(&g, "kw_else");
    assert!(ff.is_nullable(else_part));
    assert_first_contains(&ff, else_part, &[kw_else]);
    assert_follow_contains(&ff, else_part, &[EOF]);
}

#[test]
fn complex_nested_parens_grammar() {
    // S -> lp S rp | a
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
    let lp = sym(&g, "lp");
    let a = sym(&g, "a");
    let rp = sym(&g, "rp");
    assert_first_contains(&ff, start, &[lp, a]);
    assert_follow_contains(&ff, start, &[rp, EOF]);
}

#[test]
fn complex_three_level_delegation() {
    // S -> A,  A -> B,  B -> C,  C -> x | y
    // All nonterminals should have FIRST = {x, y}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["level_a"])
        .rule("level_a", vec!["level_b"])
        .rule("level_b", vec!["level_c"])
        .rule("level_c", vec!["x"])
        .rule("level_c", vec!["y"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = sym(&g, "start");
    let la = sym(&g, "level_a");
    let lb = sym(&g, "level_b");
    let lc = sym(&g, "level_c");
    let x = sym(&g, "x");
    let y = sym(&g, "y");
    assert_first_contains(&ff, start, &[x, y]);
    assert_first_contains(&ff, la, &[x, y]);
    assert_first_contains(&ff, lb, &[x, y]);
    assert_first_contains(&ff, lc, &[x, y]);
    // FOLLOW propagates EOF through the chain
    assert_follow_contains(&ff, la, &[EOF]);
    assert_follow_contains(&ff, lb, &[EOF]);
    assert_follow_contains(&ff, lc, &[EOF]);
}
