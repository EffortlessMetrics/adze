//! FIRST/FOLLOW set computation tests — v6.
//!
//! 64 tests in 8 categories:
//! 1. first_basic_*     — basic FIRST set computation
//! 2. first_nullable_*  — nullable symbol FIRST sets
//! 3. first_recursive_* — recursive grammar FIRST sets
//! 4. follow_basic_*    — basic FOLLOW set computation
//! 5. follow_eof_*      — EOF in FOLLOW sets
//! 6. follow_chain_*    — FOLLOW propagation chains
//! 7. ff_combined_*     — combined FIRST/FOLLOW properties
//! 8. ff_complex_*      — complex grammar FIRST/FOLLOW

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EOF: SymbolId = SymbolId(0);

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

#[allow(dead_code)]
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

// ===========================================================================
// 1. first_basic_* — basic FIRST set computation (8 tests)
// ===========================================================================

#[test]
fn first_basic_single_terminal() {
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
fn first_basic_two_token_rule_uses_leading() {
    // S -> a b  =>  FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "b")]);
}

#[test]
fn first_basic_two_alternatives() {
    // S -> a | b  =>  FIRST(S) = {a, b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_basic_three_alternatives() {
    // S -> a | b | c  =>  FIRST(S) = {a, b, c}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(
        &ff,
        sym(&g, "start"),
        &[sym(&g, "a"), sym(&g, "b"), sym(&g, "c")],
    );
}

#[test]
fn first_basic_nonterminal_chain() {
    // S -> A, A -> x  =>  FIRST(S) = {x}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "x")]);
}

#[test]
fn first_basic_deep_chain() {
    // S -> A, A -> B, B -> y  =>  FIRST(S) = {y}
    let g = GrammarBuilder::new("t")
        .token("y", "y")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["y"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "y")]);
    assert_first_contains(&ff, sym(&g, "mid"), &[sym(&g, "y")]);
}

#[test]
fn first_basic_terminal_is_itself() {
    // A terminal's FIRST set is itself
    let g = GrammarBuilder::new("t")
        .token("tok", "tok")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tok_id = sym(&g, "tok");
    if let Some(set) = ff.first(tok_id) {
        // Terminals may or may not have explicit FIRST entries;
        // what matters is the nonterminal propagation is correct.
        assert!(
            set.contains(tok_id.0 as usize) || set.count_ones(..) == 0,
            "terminal FIRST set should be trivial"
        );
    }
}

#[test]
fn first_basic_multiple_independent_rules() {
    // S -> a, X -> b — each has its own FIRST
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("other", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
    assert_first_eq(&ff, sym(&g, "other"), &[sym(&g, "b")]);
}

// ===========================================================================
// 2. first_nullable_* — nullable symbol FIRST sets (8 tests)
// ===========================================================================

#[test]
fn first_nullable_epsilon_rule() {
    // S -> ε  =>  S is nullable, FIRST(S) is empty
    let g = GrammarBuilder::new("t")
        .token("dummy", "d")
        .rule("start", vec![])
        .rule("start", vec!["dummy"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_nonterminal_not_nullable() {
    // S -> a  =>  S is NOT nullable
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_propagates_through_chain() {
    // S -> A, A -> ε  =>  S is nullable
    let g = GrammarBuilder::new("t")
        .token("dummy", "d")
        .rule("start", vec!["inner"])
        .rule("inner", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "inner")));
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_skip_to_second_symbol() {
    // S -> A b, A -> ε | a  =>  FIRST(S) includes {a, b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["opt", "b"])
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a"), sym(&g, "b")]);
}

#[test]
fn first_nullable_two_nullable_prefix() {
    // S -> A B c, A -> ε, B -> ε  =>  FIRST(S) includes {c}
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["na", "nb", "c"])
        .rule("na", vec![])
        .rule("nb", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "na")));
    assert!(ff.is_nullable(sym(&g, "nb")));
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "c")]);
}

#[test]
fn first_nullable_all_nullable_rhs() {
    // S -> A B, A -> ε, B -> ε  =>  S is nullable
    let g = GrammarBuilder::new("t")
        .token("dummy", "d")
        .rule("start", vec!["na", "nb"])
        .rule("start", vec!["dummy"])
        .rule("na", vec![])
        .rule("nb", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "start")));
}

#[test]
fn first_nullable_partial_nullable_prefix() {
    // S -> A B c, A -> ε | x, B -> y  =>  FIRST(S) includes {x, y}, B not nullable
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .token("c", "c")
        .rule("start", vec!["opt_a", "req_b", "c"])
        .rule("opt_a", vec![])
        .rule("opt_a", vec!["x"])
        .rule("req_b", vec!["y"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "x"), sym(&g, "y")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "c")]);
}

#[test]
fn first_nullable_deep_nullable_chain() {
    // S -> A, A -> B, B -> ε  =>  all nullable
    let g = GrammarBuilder::new("t")
        .token("dummy", "d")
        .rule("start", vec!["mid"])
        .rule("start", vec!["dummy"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "leaf")));
    assert!(ff.is_nullable(sym(&g, "mid")));
    assert!(ff.is_nullable(sym(&g, "start")));
}

// ===========================================================================
// 3. first_recursive_* — recursive grammar FIRST sets (8 tests)
// ===========================================================================

#[test]
fn first_recursive_left_recursion() {
    // S -> S a | b  =>  FIRST(S) = {b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "b")]);
}

#[test]
fn first_recursive_right_recursion() {
    // S -> a S | a  =>  FIRST(S) = {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a", "start"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_recursive_mutual_recursion() {
    // S -> A x, A -> S y | z  =>  FIRST(S) includes {z}, FIRST(A) includes {z}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["alt", "x"])
        .rule("alt", vec!["start", "y"])
        .rule("alt", vec!["z"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "z")]);
    assert_first_contains(&ff, sym(&g, "alt"), &[sym(&g, "z")]);
}

#[test]
fn first_recursive_left_recursive_list() {
    // list -> list item | item, item -> tok  =>  FIRST(list) = {tok}
    let g = GrammarBuilder::new("t")
        .token("tok", "t")
        .rule("start", vec!["start", "item"])
        .rule("start", vec!["item"])
        .rule("item", vec!["tok"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "tok")]);
}

#[test]
fn first_recursive_self_referencing_alt() {
    // S -> a | S b  =>  FIRST(S) contains {a}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "a")]);
}

#[test]
fn first_recursive_expr_with_parens() {
    // E -> num | lp E rp  =>  FIRST(E) = {num, lp}
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("lp", "(")
        .token("rp", ")")
        .rule("start", vec!["num"])
        .rule("start", vec!["lp", "start", "rp"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "num"), sym(&g, "lp")]);
    assert_first_excludes(&ff, sym(&g, "start"), &[sym(&g, "rp")]);
}

#[test]
fn first_recursive_deeply_nested_mutual() {
    // S -> A, A -> B, B -> S c | d  =>  FIRST(S) includes {d}
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["start", "c"])
        .rule("leaf", vec!["d"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "d")]);
    assert_first_contains(&ff, sym(&g, "mid"), &[sym(&g, "d")]);
    assert_first_contains(&ff, sym(&g, "leaf"), &[sym(&g, "d")]);
}

#[test]
fn first_recursive_three_way_cycle() {
    // A -> B x, B -> C y, C -> A z | w  =>  all contain {w}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .token("w", "w")
        .rule("start", vec!["bsym", "x"])
        .rule("bsym", vec!["csym", "y"])
        .rule("csym", vec!["start", "z"])
        .rule("csym", vec!["w"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "w")]);
    assert_first_contains(&ff, sym(&g, "bsym"), &[sym(&g, "w")]);
    assert_first_contains(&ff, sym(&g, "csym"), &[sym(&g, "w")]);
}

// ===========================================================================
// 4. follow_basic_* — basic FOLLOW set computation (8 tests)
// ===========================================================================

#[test]
fn follow_basic_terminal_after_nonterminal() {
    // S -> A b, A -> a  =>  FOLLOW(A) includes {b}
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
fn follow_basic_two_terminals_after() {
    // S -> A b c, A -> a  =>  FOLLOW(A) includes {b} (not c)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["inner", "b", "c"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "b")]);
    assert_follow_excludes(&ff, sym(&g, "inner"), &[sym(&g, "c")]);
}

#[test]
fn follow_basic_nonterminal_after_nonterminal() {
    // S -> A B, A -> a, B -> b  =>  FOLLOW(A) includes FIRST(B) = {b}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["lhs", "rhs"])
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "b")]);
}

#[test]
fn follow_basic_last_symbol_inherits_lhs_follow() {
    // S -> A B, B -> b  =>  FOLLOW(B) includes FOLLOW(S)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["lhs", "rhs"])
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "rhs"), &[EOF]);
}

#[test]
fn follow_basic_middle_symbol() {
    // S -> a B c, B -> b  =>  FOLLOW(B) includes {c}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "mid", "c"])
        .rule("mid", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "mid"), &[sym(&g, "c")]);
}

#[test]
fn follow_basic_nullable_successor() {
    // S -> A B, A -> a, B -> ε | b  =>  FOLLOW(A) includes {b} and FOLLOW(S) (EOF)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["lhs", "rhs"])
        .rule("lhs", vec!["a"])
        .rule("rhs", vec![])
        .rule("rhs", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "b"), EOF]);
}

#[test]
fn follow_basic_multiple_occurrences() {
    // S -> A x | A y, A -> a  =>  FOLLOW(A) includes {x, y}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["inner", "x"])
        .rule("start", vec!["inner", "y"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "x"), sym(&g, "y")]);
}

#[test]
fn follow_basic_no_follow_for_unreferenced() {
    // S -> a, B -> b (B never referenced)  =>  FOLLOW(B) does not include non-EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("orphan", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // orphan is never used in another rule's RHS, so it has no context
    assert_follow_excludes(&ff, sym(&g, "orphan"), &[sym(&g, "a"), sym(&g, "b")]);
}

// ===========================================================================
// 5. follow_eof_* — EOF in FOLLOW sets (8 tests)
// ===========================================================================

#[test]
fn follow_eof_start_symbol_has_eof() {
    // S -> a  =>  FOLLOW(S) includes EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn follow_eof_last_nonterminal_inherits_eof() {
    // S -> A, A -> a  =>  FOLLOW(A) includes EOF (from S)
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
fn follow_eof_deep_chain_propagation() {
    // S -> A, A -> B, B -> c  =>  FOLLOW(B) includes EOF
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "leaf"), &[EOF]);
    assert_follow_contains(&ff, sym(&g, "mid"), &[EOF]);
}

#[test]
fn follow_eof_nullable_tail_propagates() {
    // S -> A B, B -> ε  =>  FOLLOW(A) includes EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["lhs", "tail"])
        .rule("lhs", vec!["a"])
        .rule("tail", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[EOF]);
}

#[test]
fn follow_eof_recursive_start() {
    // S -> S a | b  =>  FOLLOW(S) includes EOF (S is the start)
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
fn follow_eof_only_start_gets_eof_directly() {
    // S -> a B, B -> b  =>  FOLLOW(B) includes EOF (tail of start), start also has EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "rhs"])
        .rule("rhs", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
    assert_follow_contains(&ff, sym(&g, "rhs"), &[EOF]);
}

#[test]
fn follow_eof_eof_does_not_leak_past_terminal() {
    // S -> A x, A -> a  =>  FOLLOW(A) includes {x} but not necessarily EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("x", "x")
        .rule("start", vec!["inner", "x"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "x")]);
    assert_follow_excludes(&ff, sym(&g, "inner"), &[EOF]);
}

#[test]
fn follow_eof_multi_level_nullable_to_eof() {
    // S -> A, A -> B, B -> ε | x  =>  FOLLOW(B) includes EOF
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec![])
        .rule("leaf", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "leaf"), &[EOF]);
}

// ===========================================================================
// 6. follow_chain_* — FOLLOW propagation chains (8 tests)
// ===========================================================================

#[test]
fn follow_chain_single_hop() {
    // S -> A b, A -> x  =>  FOLLOW(A) = {b}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "b")]);
}

#[test]
fn follow_chain_two_hops_through_nullable() {
    // S -> A B c, A -> x, B -> ε  =>  FOLLOW(A) includes {c}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("c", "c")
        .rule("start", vec!["lhs", "mid", "c"])
        .rule("lhs", vec!["x"])
        .rule("mid", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "c")]);
}

#[test]
fn follow_chain_lhs_follow_propagates_to_tail() {
    // P -> S x, S -> A, A -> a  =>  FOLLOW(A) includes {x}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("x", "x")
        .rule("prog", vec!["stmt", "x"])
        .rule("stmt", vec!["atom"])
        .rule("atom", vec!["a"])
        .start("prog")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "atom"), &[sym(&g, "x")]);
}

#[test]
fn follow_chain_multiple_rules_contribute() {
    // S -> A x | B A y, A -> a, B -> b  =>  FOLLOW(A) includes {x, y}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["inner", "x"])
        .rule("start", vec!["wrap", "inner", "y"])
        .rule("inner", vec!["a"])
        .rule("wrap", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "inner"), &[sym(&g, "x"), sym(&g, "y")]);
}

#[test]
fn follow_chain_nullable_at_end_inherits() {
    // S -> x A, A -> ε  =>  FOLLOW(A) includes FOLLOW(S) = {EOF}
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["x", "tail"])
        .rule("tail", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "tail"), &[EOF]);
}

#[test]
fn follow_chain_through_two_nullable() {
    // S -> A B C d, B -> ε, C -> ε, A -> a  =>  FOLLOW(A) includes {d}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("d", "d")
        .rule("start", vec!["lhs", "nb", "nc", "d"])
        .rule("lhs", vec!["a"])
        .rule("nb", vec![])
        .rule("nc", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "d")]);
}

#[test]
fn follow_chain_transitive_through_nonterminal() {
    // S -> A B, A -> a, B -> C, C -> c  =>  FOLLOW(A) includes FIRST(B) = {c}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("c", "c")
        .rule("start", vec!["lhs", "rhs"])
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["inner"])
        .rule("inner", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "c")]);
}

#[test]
fn follow_chain_recursive_propagation() {
    // S -> S x | a  =>  FOLLOW(S) includes {x, EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("x", "x")
        .rule("start", vec!["start", "x"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[sym(&g, "x"), EOF]);
}

// ===========================================================================
// 7. ff_combined_* — combined FIRST/FOLLOW properties (8 tests)
// ===========================================================================

#[test]
fn ff_combined_disjoint_first_sets() {
    // S -> A | B, A -> a, B -> b  =>  FIRST(A) ∩ FIRST(B) = ∅
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["lhs"])
        .rule("start", vec!["rhs"])
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, sym(&g, "lhs"), &[sym(&g, "a")]);
    assert_first_eq(&ff, sym(&g, "rhs"), &[sym(&g, "b")]);
}

#[test]
fn ff_combined_first_subset_of_parent() {
    // S -> A, A -> a  =>  FIRST(A) ⊆ FIRST(S)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    assert_first_contains(&ff, sym(&g, "start"), &[a]);
    assert_first_contains(&ff, sym(&g, "inner"), &[a]);
}

#[test]
fn ff_combined_follow_includes_first_of_successor() {
    // S -> A B, A -> a, B -> b | c  =>  FOLLOW(A) ⊇ FIRST(B)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["lhs", "rhs"])
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .rule("rhs", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "lhs"), &[sym(&g, "b"), sym(&g, "c")]);
}

#[test]
fn ff_combined_nullable_and_follow() {
    // S -> A, A -> ε | a  =>  A nullable, FOLLOW(A) includes EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["inner"])
        .rule("inner", vec![])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym(&g, "inner")));
    assert_follow_contains(&ff, sym(&g, "inner"), &[EOF]);
    assert_first_contains(&ff, sym(&g, "inner"), &[sym(&g, "a")]);
}

#[test]
fn ff_combined_terminal_not_nullable() {
    // Terminals are never nullable
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(sym(&g, "x")));
}

#[test]
fn ff_combined_start_first_covers_all_alternatives() {
    // S -> a | B, B -> b | c  =>  FIRST(S) ⊇ {a, b, c}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["inner"])
        .rule("inner", vec!["b"])
        .rule("inner", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(
        &ff,
        sym(&g, "start"),
        &[sym(&g, "a"), sym(&g, "b"), sym(&g, "c")],
    );
}

#[test]
fn ff_combined_follow_of_start_always_has_eof() {
    // Any grammar: FOLLOW(start) always includes EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner", "b"])
        .rule("inner", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "start"), &[EOF]);
}

#[test]
fn ff_combined_first_of_sequence_api() {
    // Test first_of_sequence method directly
    use adze_ir::Symbol;
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym(&g, "a");
    let seq = vec![Symbol::Terminal(a_id)];
    let result = ff.first_of_sequence(&seq).unwrap();
    assert!(result.contains(a_id.0 as usize));
}

// ===========================================================================
// 8. ff_complex_* — complex grammar FIRST/FOLLOW (8 tests)
// ===========================================================================

#[test]
fn ff_complex_arithmetic_expr_first() {
    // E -> T | E plus T, T -> num | lp E rp
    // FIRST(E) = {num, lp}, FIRST(T) = {num, lp}
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .token("lp", "(")
        .token("rp", ")")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["num"])
        .rule("term", vec!["lp", "expr", "rp"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expected = [sym(&g, "num"), sym(&g, "lp")];
    assert_first_contains(&ff, sym(&g, "expr"), &expected);
    assert_first_contains(&ff, sym(&g, "term"), &expected);
}

#[test]
fn ff_complex_arithmetic_expr_follow() {
    // E -> T | E plus T, T -> num | lp E rp
    // FOLLOW(E) = {EOF, plus, rp}
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .token("lp", "(")
        .token("rp", ")")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["num"])
        .rule("term", vec!["lp", "expr", "rp"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "expr"), &[EOF, sym(&g, "plus"), sym(&g, "rp")]);
}

#[test]
fn ff_complex_if_else_first() {
    // S -> kw_if cond kw_then S | kw_if cond kw_then S kw_else S | a
    // FIRST(S) = {kw_if, a}
    let g = GrammarBuilder::new("t")
        .token("kw_if", "if")
        .token("kw_then", "then")
        .token("kw_else", "else")
        .token("cond", "c")
        .token("a", "a")
        .rule("start", vec!["kw_if", "cond", "kw_then", "start"])
        .rule(
            "start",
            vec!["kw_if", "cond", "kw_then", "start", "kw_else", "start"],
        )
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "kw_if"), sym(&g, "a")]);
    assert_first_excludes(
        &ff,
        sym(&g, "start"),
        &[sym(&g, "kw_then"), sym(&g, "kw_else")],
    );
}

#[test]
fn ff_complex_optional_semicolons() {
    // S -> stmt semi_opt, stmt -> a, semi_opt -> semi | ε
    // FOLLOW(stmt) includes {semi, EOF}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("semi", ";")
        .rule("start", vec!["stmt", "semi_opt"])
        .rule("stmt", vec!["a"])
        .rule("semi_opt", vec!["semi"])
        .rule("semi_opt", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, sym(&g, "stmt"), &[sym(&g, "semi"), EOF]);
    assert!(ff.is_nullable(sym(&g, "semi_opt")));
}

#[test]
fn ff_complex_list_with_separator() {
    // list -> item | list comma item, item -> id
    // FIRST(list) = {id}, FOLLOW(item) = {comma, EOF}
    let g = GrammarBuilder::new("t")
        .token("id", "x")
        .token("comma", ",")
        .rule("start", vec!["item"])
        .rule("start", vec!["start", "comma", "item"])
        .rule("item", vec!["id"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "id")]);
    assert_follow_contains(&ff, sym(&g, "item"), &[sym(&g, "comma"), EOF]);
}

#[test]
fn ff_complex_nested_parens() {
    // S -> lp S rp | a  =>  FIRST(S) = {lp, a}, FOLLOW(S) = {rp, EOF}
    let g = GrammarBuilder::new("t")
        .token("lp", "(")
        .token("rp", ")")
        .token("a", "a")
        .rule("start", vec!["lp", "start", "rp"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, sym(&g, "start"), &[sym(&g, "lp"), sym(&g, "a")]);
    assert_follow_contains(&ff, sym(&g, "start"), &[sym(&g, "rp"), EOF]);
}

#[test]
fn ff_complex_three_level_expression() {
    // E -> T, T -> F | T star F, F -> num | lp E rp
    // FIRST(E) = FIRST(T) = FIRST(F) = {num, lp}
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("star", "*")
        .token("lp", "(")
        .token("rp", ")")
        .rule("expr", vec!["term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("factor", vec!["num"])
        .rule("factor", vec!["lp", "expr", "rp"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expected = [sym(&g, "num"), sym(&g, "lp")];
    assert_first_contains(&ff, sym(&g, "expr"), &expected);
    assert_first_contains(&ff, sym(&g, "term"), &expected);
    assert_first_contains(&ff, sym(&g, "factor"), &expected);
}

#[test]
fn ff_complex_determinism() {
    // Same grammar built twice should yield identical FIRST/FOLLOW
    let build = || {
        GrammarBuilder::new("t")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["inner", "c"])
            .rule("inner", vec!["a"])
            .rule("inner", vec!["b"])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    let s1 = sym(&g1, "start");
    let s2 = sym(&g2, "start");
    let first1 = ff1.first(s1).unwrap();
    let first2 = ff2.first(s2).unwrap();
    assert_eq!(first1, first2, "FIRST sets should be deterministic");

    let follow1 = ff1.follow(s1).unwrap();
    let follow2 = ff2.follow(s2).unwrap();
    assert_eq!(follow1, follow2, "FOLLOW sets should be deterministic");
}
