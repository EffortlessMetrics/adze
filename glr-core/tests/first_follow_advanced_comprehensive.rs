//! Advanced comprehensive tests for FIRST/FOLLOW set computation edge cases.
#![cfg(feature = "test-api")]

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, SymbolId};

/// Helper: build grammar, normalize, compute FIRST/FOLLOW.
fn compute_ff(
    name: &str,
    builder_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> FirstFollowSets {
    let mut g = builder_fn(GrammarBuilder::new(name)).build();
    g.normalize();
    FirstFollowSets::compute(&g).expect("compute should succeed")
}

/// Helper: get the SymbolId assigned to a name by rebuilding the builder lookup.
fn sid(name: &str, builder_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder) -> SymbolId {
    let g = builder_fn(GrammarBuilder::new("lookup")).build();
    // Search tokens then rules for the name
    for (&id, tok) in &g.tokens {
        if tok.name == name {
            return id;
        }
    }
    for (&id, rname) in &g.rule_names {
        if rname == name {
            return id;
        }
    }
    panic!("symbol '{}' not found", name);
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. FIRST sets for simple terminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_of_single_terminal_rule_contains_that_terminal() {
    let build = |b: GrammarBuilder| b.token("a", "a").rule("S", vec!["a"]).start("S");
    let ff = compute_ff("t01", build);
    let s_id = sid("S", |b| b.token("a", "a").rule("S", vec!["a"]).start("S"));
    let a_id = sid("a", |b| b.token("a", "a").rule("S", vec!["a"]).start("S"));
    let first = ff.first(s_id).expect("FIRST(S) should exist");
    assert!(first.contains(a_id.0 as usize));
}

#[test]
fn first_of_terminal_itself_contains_itself() {
    let build = |b: GrammarBuilder| b.token("x", "x").rule("S", vec!["x"]).start("S");
    let ff = compute_ff("t02", build);
    let x_id = sid("x", |b| b.token("x", "x").rule("S", vec!["x"]).start("S"));
    let first = ff.first(x_id).expect("FIRST(x) should exist");
    assert!(first.contains(x_id.0 as usize));
}

#[test]
fn first_of_rule_with_two_terminals_contains_only_first() {
    let build = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S")
    };
    let ff = compute_ff("t03", build);
    let s_id = sid("S", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S")
    });
    let a_id = sid("a", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S")
    });
    let b_id = sid("b", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S")
    });
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(!first.contains(b_id.0 as usize));
}

#[test]
fn first_of_three_terminal_sequence_contains_only_leading() {
    let build = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("y", "y")
            .token("z", "z")
            .rule("S", vec!["x", "y", "z"])
            .start("S")
    };
    let ff = compute_ff("t04", build);
    let g = GrammarBuilder::new("t04")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("S", vec!["x", "y", "z"])
        .start("S")
        .build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let y_id = *g.tokens.iter().find(|(_, t)| t.name == "y").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(x_id.0 as usize));
    assert!(!first.contains(y_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. FIRST sets for nonterminals with single production
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_propagates_through_single_nonterminal_chain() {
    let mk = |b: GrammarBuilder| {
        b.token("t", "t")
            .rule("A", vec!["t"])
            .rule("S", vec!["A"])
            .start("S")
    };
    let ff = compute_ff("t05", mk);
    let g = mk(GrammarBuilder::new("t05")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let t_id = *g.tokens.iter().find(|(_, t)| t.name == "t").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(t_id.0 as usize));
}

#[test]
fn first_through_double_chain() {
    let mk = |b: GrammarBuilder| {
        b.token("t", "t")
            .rule("C", vec!["t"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .start("A")
    };
    let ff = compute_ff("t06", mk);
    let g = mk(GrammarBuilder::new("t06")).build();
    let a_id = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let t_id = *g.tokens.iter().find(|(_, t)| t.name == "t").unwrap().0;
    assert!(ff.first(a_id).unwrap().contains(t_id.0 as usize));
}

#[test]
fn first_nonterminal_followed_by_terminal() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("X", vec!["a"])
            .rule("S", vec!["X", "b"])
            .start("S")
    };
    let ff = compute_ff("t07", mk);
    let g = mk(GrammarBuilder::new("t07")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(!first.contains(b_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. FIRST sets for nonterminals with multiple productions (union)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_union_of_two_alternatives() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .start("S")
    };
    let ff = compute_ff("t08", mk);
    let g = mk(GrammarBuilder::new("t08")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(first.contains(b_id.0 as usize));
}

#[test]
fn first_union_of_three_alternatives() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .rule("S", vec!["c"])
            .start("S")
    };
    let ff = compute_ff("t09", mk);
    let g = mk(GrammarBuilder::new("t09")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let c_id = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(first.contains(b_id.0 as usize));
    assert!(first.contains(c_id.0 as usize));
}

#[test]
fn first_union_through_nonterminal_alternatives() {
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("y", "y")
            .rule("A", vec!["x"])
            .rule("B", vec!["y"])
            .rule("S", vec!["A"])
            .rule("S", vec!["B"])
            .start("S")
    };
    let ff = compute_ff("t10", mk);
    let g = mk(GrammarBuilder::new("t10")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let y_id = *g.tokens.iter().find(|(_, t)| t.name == "y").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(x_id.0 as usize));
    assert!(first.contains(y_id.0 as usize));
}

#[test]
fn first_union_mixed_terminal_and_nonterminal_alternatives() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("X", vec!["b"])
            .rule("S", vec!["a"])
            .rule("S", vec!["X"])
            .start("S")
    };
    let ff = compute_ff("t11", mk);
    let g = mk(GrammarBuilder::new("t11")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(first.contains(b_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. FIRST sets for recursive grammars (left/right)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_left_recursive_grammar() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("E", vec!["E", "plus", "a"])
            .rule("E", vec!["a"])
            .start("E")
    };
    let ff = compute_ff("t12", mk);
    let g = mk(GrammarBuilder::new("t12")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let first = ff.first(e_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(!first.contains(plus_id.0 as usize));
}

#[test]
fn first_right_recursive_grammar() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("E", vec!["a", "plus", "E"])
            .rule("E", vec!["a"])
            .start("E")
    };
    let ff = compute_ff("t13", mk);
    let g = mk(GrammarBuilder::new("t13")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert!(ff.first(e_id).unwrap().contains(a_id.0 as usize));
}

#[test]
fn first_mutual_recursion() {
    // A → B a | a ; B → A b | b
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("A", vec!["B", "a"])
            .rule("A", vec!["a"])
            .rule("B", vec!["A", "b"])
            .rule("B", vec!["b"])
            .start("A")
    };
    let ff = compute_ff("t14", mk);
    let g = mk(GrammarBuilder::new("t14")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let b_nt = *g.rule_names.iter().find(|(_, n)| *n == "B").unwrap().0;
    let a_tok = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    // FIRST(A) should contain both 'a' and 'b' (through B)
    assert!(ff.first(a_nt).unwrap().contains(a_tok.0 as usize));
    assert!(ff.first(a_nt).unwrap().contains(b_tok.0 as usize));
    // FIRST(B) should contain both 'a' and 'b' (through A)
    assert!(ff.first(b_nt).unwrap().contains(a_tok.0 as usize));
    assert!(ff.first(b_nt).unwrap().contains(b_tok.0 as usize));
}

#[test]
fn first_deeply_left_recursive() {
    // E → E '*' E | E '+' E | num
    let mk = |b: GrammarBuilder| {
        b.token("num", "[0-9]+")
            .token("star", "\\*")
            .token("plus", "\\+")
            .rule("E", vec!["E", "star", "E"])
            .rule("E", vec!["E", "plus", "E"])
            .rule("E", vec!["num"])
            .start("E")
    };
    let ff = compute_ff("t15", mk);
    let g = mk(GrammarBuilder::new("t15")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    let star_id = *g.tokens.iter().find(|(_, t)| t.name == "star").unwrap().0;
    let first = ff.first(e_id).unwrap();
    assert!(first.contains(num_id.0 as usize));
    assert!(!first.contains(star_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. FOLLOW sets include EOF for start symbol
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_start_symbol_contains_eof() {
    let mk = |b: GrammarBuilder| b.token("a", "a").rule("S", vec!["a"]).start("S");
    let ff = compute_ff("t16", mk);
    let g = mk(GrammarBuilder::new("t16")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let follow = ff.follow(s_id).unwrap();
    assert!(follow.contains(0), "FOLLOW(start) must contain EOF (0)");
}

#[test]
fn follow_start_contains_eof_in_recursive_grammar() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("E", vec!["E", "plus", "a"])
            .rule("E", vec!["a"])
            .start("E")
    };
    let ff = compute_ff("t17", mk);
    let g = mk(GrammarBuilder::new("t17")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    assert!(ff.follow(e_id).unwrap().contains(0));
}

#[test]
fn follow_non_start_does_not_necessarily_contain_eof() {
    // S → A b; A → a; EOF only in FOLLOW(S), not necessarily in FOLLOW(A)
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("A", vec!["a"])
            .rule("S", vec!["A", "b"])
            .start("S")
    };
    let ff = compute_ff("t18", mk);
    let g = mk(GrammarBuilder::new("t18")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    assert!(ff.follow(s_id).unwrap().contains(0));
    // FOLLOW(A) should contain 'b' (from S → A . b)
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    assert!(ff.follow(a_id).unwrap().contains(b_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. FOLLOW sets propagate through chained rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_propagates_from_rhs_end() {
    // S → A; A → a  →  FOLLOW(A) ⊇ FOLLOW(S) = {EOF}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("A", vec!["a"])
            .rule("S", vec!["A"])
            .start("S")
    };
    let ff = compute_ff("t19", mk);
    let g = mk(GrammarBuilder::new("t19")).build();
    let a_id = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    assert!(
        ff.follow(a_id).unwrap().contains(0),
        "FOLLOW(A) should contain EOF via FOLLOW(S)"
    );
}

#[test]
fn follow_propagates_through_two_levels() {
    // S → B; B → C; C → x  →  FOLLOW(C) ⊇ FOLLOW(B) ⊇ FOLLOW(S)
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("S", vec!["B"])
            .start("S")
    };
    let ff = compute_ff("t20", mk);
    let g = mk(GrammarBuilder::new("t20")).build();
    let c_id = *g.rule_names.iter().find(|(_, n)| *n == "C").unwrap().0;
    assert!(ff.follow(c_id).unwrap().contains(0));
}

#[test]
fn follow_includes_first_of_following_symbol() {
    // S → A B; A → a; B → b  → FOLLOW(A) ⊇ FIRST(B) = {b}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("A", vec!["a"])
            .rule("B", vec!["b"])
            .rule("S", vec!["A", "B"])
            .start("S")
    };
    let ff = compute_ff("t21", mk);
    let g = mk(GrammarBuilder::new("t21")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    assert!(ff.follow(a_nt).unwrap().contains(b_tok.0 as usize));
}

#[test]
fn follow_includes_terminal_after_nonterminal() {
    // S → A ";" ; A → a
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("semi", ";")
            .rule("A", vec!["a"])
            .rule("S", vec!["A", "semi"])
            .start("S")
    };
    let ff = compute_ff("t22", mk);
    let g = mk(GrammarBuilder::new("t22")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let semi_id = *g.tokens.iter().find(|(_, t)| t.name == "semi").unwrap().0;
    assert!(ff.follow(a_nt).unwrap().contains(semi_id.0 as usize));
}

#[test]
fn follow_multiple_contexts_union() {
    // S → A "x" | A "y"; A → a → FOLLOW(A) = {x, y}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("x", "x")
            .token("y", "y")
            .rule("A", vec!["a"])
            .rule("S", vec!["A", "x"])
            .rule("S", vec!["A", "y"])
            .start("S")
    };
    let ff = compute_ff("t23", mk);
    let g = mk(GrammarBuilder::new("t23")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let x_tok = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let y_tok = *g.tokens.iter().find(|(_, t)| t.name == "y").unwrap().0;
    let follow = ff.follow(a_nt).unwrap();
    assert!(follow.contains(x_tok.0 as usize));
    assert!(follow.contains(y_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. FIRST/FOLLOW with precedence grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_with_precedence_rules() {
    let mut g = GrammarBuilder::new("prec")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("E", vec!["E", "plus", "E"], 1, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "star", "E"], 2, Associativity::Left)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    assert!(ff.first(e_id).unwrap().contains(num_id.0 as usize));
}

#[test]
fn follow_with_precedence_rules() {
    let mut g = GrammarBuilder::new("prec_follow")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("E", vec!["E", "plus", "E"], 1, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "star", "E"], 2, Associativity::Left)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let star_id = *g.tokens.iter().find(|(_, t)| t.name == "star").unwrap().0;
    let follow = ff.follow(e_id).unwrap();
    assert!(follow.contains(plus_id.0 as usize));
    assert!(follow.contains(star_id.0 as usize));
    assert!(follow.contains(0)); // EOF
}

#[test]
fn first_with_right_associative_precedence() {
    let mut g = GrammarBuilder::new("rassoc")
        .token("num", "[0-9]+")
        .token("pow", "\\^")
        .rule_with_precedence("E", vec!["E", "pow", "E"], 3, Associativity::Right)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    assert!(ff.first(e_id).unwrap().contains(num_id.0 as usize));
}

#[test]
fn precedence_does_not_alter_first_set_contents() {
    // Same grammar with and without precedence should have same FIRST sets
    let mk_no_prec = |b: GrammarBuilder| {
        b.token("n", "[0-9]+")
            .token("p", "\\+")
            .rule("E", vec!["E", "p", "E"])
            .rule("E", vec!["n"])
            .start("E")
    };
    let mk_prec = || {
        let mut g = GrammarBuilder::new("wp")
            .token("n", "[0-9]+")
            .token("p", "\\+")
            .rule_with_precedence("E", vec!["E", "p", "E"], 1, Associativity::Left)
            .rule("E", vec!["n"])
            .start("E")
            .build();
        g.normalize();
        FirstFollowSets::compute(&g).unwrap()
    };
    let ff_no = compute_ff("np", mk_no_prec);
    let ff_yes = mk_prec();
    let g_no = mk_no_prec(GrammarBuilder::new("np")).build();
    let g_yes = GrammarBuilder::new("wp")
        .token("n", "[0-9]+")
        .token("p", "\\+")
        .rule_with_precedence("E", vec!["E", "p", "E"], 1, Associativity::Left)
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let e_no = *g_no.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let e_yes = *g_yes.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let n_no = *g_no.tokens.iter().find(|(_, t)| t.name == "n").unwrap().0;
    let n_yes = *g_yes.tokens.iter().find(|(_, t)| t.name == "n").unwrap().0;
    assert_eq!(
        ff_no.first(e_no).unwrap().contains(n_no.0 as usize),
        ff_yes.first(e_yes).unwrap().contains(n_yes.0 as usize),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. FIRST/FOLLOW after normalize (with complex symbols)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compute_normalized_succeeds() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn compute_after_manual_normalize() {
    let mut g = GrammarBuilder::new("manual_norm")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn compute_normalized_matches_manual_normalize() {
    let build = || {
        GrammarBuilder::new("cmp")
            .token("a", "a")
            .rule("S", vec!["a"])
            .start("S")
            .build()
    };
    let mut g1 = build();
    g1.normalize();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();

    let mut g2 = build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let s1 = *g1.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let s2 = *g2.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a1 = *g1.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let a2 = *g2.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert_eq!(
        ff1.first(s1).unwrap().contains(a1.0 as usize),
        ff2.first(s2).unwrap().contains(a2.0 as usize),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Large grammars (10+ rules)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn large_grammar_ten_alternatives() {
    let mut builder = GrammarBuilder::new("large10");
    for i in 0..10 {
        let tname: &str = Box::leak(format!("tok{i}").into_boxed_str());
        let patt: &str = Box::leak(format!("t{i}").into_boxed_str());
        builder = builder.token(tname, patt);
        builder = builder.rule("S", vec![tname]);
    }
    builder = builder.start("S");
    let mut g = builder.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let first = ff.first(s_id).unwrap();
    for (_, tok) in &g.tokens {
        let tid = *g.tokens.iter().find(|(_, t)| t.name == tok.name).unwrap().0;
        assert!(first.contains(tid.0 as usize));
    }
}

#[test]
fn large_grammar_chained_nonterminals() {
    // S → A0; A0 → A1; ... A9 → x
    let mut builder = GrammarBuilder::new("chain10");
    builder = builder.token("x", "x");
    for i in (0..10).rev() {
        let lhs: &str = Box::leak(format!("A{i}").into_boxed_str());
        if i == 9 {
            builder = builder.rule(lhs, vec!["x"]);
        } else {
            let rhs: &str = Box::leak(format!("A{}", i + 1).into_boxed_str());
            builder = builder.rule(lhs, vec![rhs]);
        }
    }
    builder = builder.rule("S", vec!["A0"]).start("S");
    let mut g = builder.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(x_id.0 as usize));
}

#[test]
fn large_grammar_arithmetic_with_parens() {
    // Full arithmetic: E → E+T | T; T → T*F | F; F → (E) | num
    let mk = |b: GrammarBuilder| {
        b.token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lparen", "\\(")
            .token("rparen", "\\)")
            .rule("E", vec!["E", "plus", "T"])
            .rule("E", vec!["T"])
            .rule("T", vec!["T", "star", "F"])
            .rule("T", vec!["F"])
            .rule("F", vec!["lparen", "E", "rparen"])
            .rule("F", vec!["num"])
            .start("E")
    };
    let ff = compute_ff("arith_paren", mk);
    let g = mk(GrammarBuilder::new("arith_paren")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let t_id = *g.rule_names.iter().find(|(_, n)| *n == "T").unwrap().0;
    let f_id = *g.rule_names.iter().find(|(_, n)| *n == "F").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    let lp_id = *g.tokens.iter().find(|(_, t)| t.name == "lparen").unwrap().0;

    // FIRST(E) = FIRST(T) = FIRST(F) = {num, (}
    for nt in [e_id, t_id, f_id] {
        let first = ff.first(nt).unwrap();
        assert!(first.contains(num_id.0 as usize));
        assert!(first.contains(lp_id.0 as usize));
    }
}

#[test]
fn large_grammar_many_rules_follow_propagation() {
    // S → A B C D; A → a; B → b; C → c; D → d
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("A", vec!["a"])
            .rule("B", vec!["b"])
            .rule("C", vec!["c"])
            .rule("D", vec!["d"])
            .rule("S", vec!["A", "B", "C", "D"])
            .start("S")
    };
    let ff = compute_ff("seq4", mk);
    let g = mk(GrammarBuilder::new("seq4")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let b_nt = *g.rule_names.iter().find(|(_, n)| *n == "B").unwrap().0;
    let c_nt = *g.rule_names.iter().find(|(_, n)| *n == "C").unwrap().0;
    let d_nt = *g.rule_names.iter().find(|(_, n)| *n == "D").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let c_tok = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    let d_tok = *g.tokens.iter().find(|(_, t)| t.name == "d").unwrap().0;

    // FOLLOW(A) ⊇ FIRST(B) = {b}
    assert!(ff.follow(a_nt).unwrap().contains(b_tok.0 as usize));
    // FOLLOW(B) ⊇ FIRST(C) = {c}
    assert!(ff.follow(b_nt).unwrap().contains(c_tok.0 as usize));
    // FOLLOW(C) ⊇ FIRST(D) = {d}
    assert!(ff.follow(c_nt).unwrap().contains(d_tok.0 as usize));
    // FOLLOW(D) ⊇ FOLLOW(S) = {EOF}
    assert!(ff.follow(d_nt).unwrap().contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Determinism (same grammar → same sets)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_first_sets() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .start("S")
    };
    let ff1 = compute_ff("det1", mk);
    let ff2 = compute_ff("det2", mk);
    let g = mk(GrammarBuilder::new("det")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    assert_eq!(
        ff1.first(s_id).unwrap().contains(a_id.0 as usize),
        ff2.first(s_id).unwrap().contains(a_id.0 as usize),
    );
    assert_eq!(
        ff1.first(s_id).unwrap().contains(b_id.0 as usize),
        ff2.first(s_id).unwrap().contains(b_id.0 as usize),
    );
}

#[test]
fn deterministic_follow_sets() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("A", vec!["a"])
            .rule("S", vec!["A", "b"])
            .start("S")
    };
    let ff1 = compute_ff("dfl1", mk);
    let ff2 = compute_ff("dfl2", mk);
    let g = mk(GrammarBuilder::new("dfl")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    assert_eq!(
        ff1.follow(a_nt).unwrap().contains(b_tok.0 as usize),
        ff2.follow(a_nt).unwrap().contains(b_tok.0 as usize),
    );
}

#[test]
fn deterministic_nullable() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("E", vec![]) // epsilon
            .rule("S", vec!["E", "a"])
            .start("S")
    };
    let ff1 = compute_ff("dn1", mk);
    let ff2 = compute_ff("dn2", mk);
    let g = mk(GrammarBuilder::new("dn")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    assert_eq!(ff1.is_nullable(e_id), ff2.is_nullable(e_id));
}

#[test]
fn deterministic_across_ten_runs() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .start("S")
    };
    let g = mk(GrammarBuilder::new("d10")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let reference = compute_ff("d10_ref", mk);
    let ref_val = reference.first(s_id).unwrap().contains(a_id.0 as usize);
    for i in 0..10 {
        let name: &str = Box::leak(format!("d10_{i}").into_boxed_str());
        let ff = compute_ff(name, mk);
        assert_eq!(ff.first(s_id).unwrap().contains(a_id.0 as usize), ref_val);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Grammar with all terminals in FIRST of start
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_terminals_in_first_of_start_five_tokens() {
    let mk = |b: GrammarBuilder| {
        b.token("t0", "t0")
            .token("t1", "t1")
            .token("t2", "t2")
            .token("t3", "t3")
            .token("t4", "t4")
            .rule("S", vec!["t0"])
            .rule("S", vec!["t1"])
            .rule("S", vec!["t2"])
            .rule("S", vec!["t3"])
            .rule("S", vec!["t4"])
            .start("S")
    };
    let ff = compute_ff("all5", mk);
    let g = mk(GrammarBuilder::new("all5")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let first = ff.first(s_id).unwrap();
    for (_, tok) in &g.tokens {
        let tid = *g.tokens.iter().find(|(_, t)| t.name == tok.name).unwrap().0;
        assert!(
            first.contains(tid.0 as usize),
            "FIRST(S) should contain {}",
            tok.name
        );
    }
}

#[test]
fn all_terminals_through_nonterminal_alternatives() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("X", vec!["a"])
            .rule("Y", vec!["b"])
            .rule("Z", vec!["c"])
            .rule("S", vec!["X"])
            .rule("S", vec!["Y"])
            .rule("S", vec!["Z"])
            .start("S")
    };
    let ff = compute_ff("all_nt", mk);
    let g = mk(GrammarBuilder::new("all_nt")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let c_id = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(first.contains(b_id.0 as usize));
    assert!(first.contains(c_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. FOLLOW sets for intermediate nonterminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_intermediate_in_sequence() {
    // S → A B C; A → a; B → b; C → c
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("A", vec!["a"])
            .rule("B", vec!["b"])
            .rule("C", vec!["c"])
            .rule("S", vec!["A", "B", "C"])
            .start("S")
    };
    let ff = compute_ff("inter", mk);
    let g = mk(GrammarBuilder::new("inter")).build();
    let b_nt = *g.rule_names.iter().find(|(_, n)| *n == "B").unwrap().0;
    let c_tok = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    assert!(
        ff.follow(b_nt).unwrap().contains(c_tok.0 as usize),
        "FOLLOW(B) should contain FIRST(C)"
    );
}

#[test]
fn follow_intermediate_multiple_occurrences() {
    // S → A "x" | B A "y"; A → a; B → b  →  FOLLOW(A) ⊇ {x, y}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("x", "x")
            .token("y", "y")
            .rule("A", vec!["a"])
            .rule("B", vec!["b"])
            .rule("S", vec!["A", "x"])
            .rule("S", vec!["B", "A", "y"])
            .start("S")
    };
    let ff = compute_ff("multi_ctx", mk);
    let g = mk(GrammarBuilder::new("multi_ctx")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let x_tok = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let y_tok = *g.tokens.iter().find(|(_, t)| t.name == "y").unwrap().0;
    let follow = ff.follow(a_nt).unwrap();
    assert!(follow.contains(x_tok.0 as usize));
    assert!(follow.contains(y_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional edge cases: nullable, epsilon, is_nullable
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nullable_epsilon_production() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("E", vec![]) // epsilon
            .rule("S", vec!["E", "a"])
            .start("S")
    };
    let ff = compute_ff("eps", mk);
    let g = mk(GrammarBuilder::new("eps")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    assert!(ff.is_nullable(e_id));
}

#[test]
fn non_nullable_terminal_rule() {
    let mk = |b: GrammarBuilder| b.token("a", "a").rule("S", vec!["a"]).start("S");
    let ff = compute_ff("non_null", mk);
    let g = mk(GrammarBuilder::new("non_null")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    assert!(!ff.is_nullable(s_id));
}

#[test]
fn nullable_chain() {
    // E → ε; F → E; both nullable
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("E", vec![])
            .rule("F", vec!["E"])
            .rule("S", vec!["F", "a"])
            .start("S")
    };
    let ff = compute_ff("nchain", mk);
    let g = mk(GrammarBuilder::new("nchain")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let f_id = *g.rule_names.iter().find(|(_, n)| *n == "F").unwrap().0;
    assert!(ff.is_nullable(e_id));
    assert!(ff.is_nullable(f_id));
}

#[test]
fn first_skips_nullable_prefix() {
    // S → E a; E → ε  →  FIRST(S) ⊇ {a}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("E", vec![])
            .rule("S", vec!["E", "a"])
            .start("S")
    };
    let ff = compute_ff("skip_null", mk);
    let g = mk(GrammarBuilder::new("skip_null")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(a_id.0 as usize));
}

#[test]
fn first_skips_multiple_nullable_prefixes() {
    // S → E F a; E → ε; F → ε  →  FIRST(S) ⊇ {a}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("E", vec![])
            .rule("F", vec![])
            .rule("S", vec!["E", "F", "a"])
            .start("S")
    };
    let ff = compute_ff("skip_multi", mk);
    let g = mk(GrammarBuilder::new("skip_multi")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(a_id.0 as usize));
}

#[test]
fn follow_through_nullable_suffix() {
    // S → A B; A → a; B → ε  →  FOLLOW(A) ⊇ FOLLOW(S) = {EOF}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("A", vec!["a"])
            .rule("B", vec![])
            .rule("S", vec!["A", "B"])
            .start("S")
    };
    let ff = compute_ff("null_suffix", mk);
    let g = mk(GrammarBuilder::new("null_suffix")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    assert!(
        ff.follow(a_nt).unwrap().contains(0),
        "FOLLOW(A) should contain EOF when B is nullable"
    );
}

#[test]
fn follow_nullable_intermediate() {
    // S → A B C; B → ε; A → a; C → c  →  FOLLOW(A) ⊇ FIRST(C) = {c}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("c", "c")
            .rule("A", vec!["a"])
            .rule("B", vec![])
            .rule("C", vec!["c"])
            .rule("S", vec!["A", "B", "C"])
            .start("S")
    };
    let ff = compute_ff("null_mid", mk);
    let g = mk(GrammarBuilder::new("null_mid")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let c_tok = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    assert!(
        ff.follow(a_nt).unwrap().contains(c_tok.0 as usize),
        "FOLLOW(A) should contain c when B is nullable"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Diamond and branching grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn diamond_grammar_first_sets() {
    // S → L | R; L → a; R → a  →  FIRST(S) = {a}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("L", vec!["a"])
            .rule("R", vec!["a"])
            .rule("S", vec!["L"])
            .rule("S", vec!["R"])
            .start("S")
    };
    let ff = compute_ff("diamond", mk);
    let g = mk(GrammarBuilder::new("diamond")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(a_id.0 as usize));
}

#[test]
fn diamond_grammar_follow_sets() {
    // S → L "x" | R "y"; L → a; R → b
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("x", "x")
            .token("y", "y")
            .rule("L", vec!["a"])
            .rule("R", vec!["b"])
            .rule("S", vec!["L", "x"])
            .rule("S", vec!["R", "y"])
            .start("S")
    };
    let ff = compute_ff("diamond_f", mk);
    let g = mk(GrammarBuilder::new("diamond_f")).build();
    let l_nt = *g.rule_names.iter().find(|(_, n)| *n == "L").unwrap().0;
    let r_nt = *g.rule_names.iter().find(|(_, n)| *n == "R").unwrap().0;
    let x_tok = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let y_tok = *g.tokens.iter().find(|(_, t)| t.name == "y").unwrap().0;
    assert!(ff.follow(l_nt).unwrap().contains(x_tok.0 as usize));
    assert!(ff.follow(r_nt).unwrap().contains(y_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// first_of_sequence tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_of_sequence_single_terminal() {
    let mk = |b: GrammarBuilder| b.token("a", "a").rule("S", vec!["a"]).start("S");
    let ff = compute_ff("seq_t", mk);
    let g = mk(GrammarBuilder::new("seq_t")).build();
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let result = ff
        .first_of_sequence(&[adze_ir::Symbol::Terminal(a_id)])
        .unwrap();
    assert!(result.contains(a_id.0 as usize));
}

#[test]
fn first_of_sequence_nonterminal() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("X", vec!["a"])
            .rule("S", vec!["X"])
            .start("S")
    };
    let ff = compute_ff("seq_nt", mk);
    let g = mk(GrammarBuilder::new("seq_nt")).build();
    let x_id = *g.rule_names.iter().find(|(_, n)| *n == "X").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let result = ff
        .first_of_sequence(&[adze_ir::Symbol::NonTerminal(x_id)])
        .unwrap();
    assert!(result.contains(a_id.0 as usize));
}

#[test]
fn first_of_sequence_empty() {
    let mk = |b: GrammarBuilder| b.token("a", "a").rule("S", vec!["a"]).start("S");
    let ff = compute_ff("seq_empty", mk);
    let result = ff.first_of_sequence(&[]).unwrap();
    // Empty sequence → empty FIRST set
    assert_eq!(result.count_ones(..), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Expression grammar variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn if_then_else_grammar() {
    // S → if E then S else S | if E then S | a; E → b
    let mk = |b: GrammarBuilder| {
        b.token("if_kw", "if")
            .token("then_kw", "then")
            .token("else_kw", "else")
            .token("a", "a")
            .token("b", "b")
            .rule("E", vec!["b"])
            .rule("S", vec!["if_kw", "E", "then_kw", "S", "else_kw", "S"])
            .rule("S", vec!["if_kw", "E", "then_kw", "S"])
            .rule("S", vec!["a"])
            .start("S")
    };
    let ff = compute_ff("ite", mk);
    let g = mk(GrammarBuilder::new("ite")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let if_id = *g.tokens.iter().find(|(_, t)| t.name == "if_kw").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(if_id.0 as usize));
    assert!(first.contains(a_id.0 as usize));
}

#[test]
fn list_grammar_left_recursive() {
    // L → L "," item | item; item → "x"
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("comma", ",")
            .rule("item", vec!["x"])
            .rule("L", vec!["L", "comma", "item"])
            .rule("L", vec!["item"])
            .start("L")
    };
    let ff = compute_ff("list_lr", mk);
    let g = mk(GrammarBuilder::new("list_lr")).build();
    let l_id = *g.rule_names.iter().find(|(_, n)| *n == "L").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let comma_id = *g.tokens.iter().find(|(_, t)| t.name == "comma").unwrap().0;
    assert!(ff.first(l_id).unwrap().contains(x_id.0 as usize));
    // FOLLOW(L) should contain comma and EOF
    let follow = ff.follow(l_id).unwrap();
    assert!(follow.contains(comma_id.0 as usize));
    assert!(follow.contains(0));
}

#[test]
fn list_grammar_right_recursive() {
    // L → item "," L | item; item → "x"
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("comma", ",")
            .rule("item", vec!["x"])
            .rule("L", vec!["item", "comma", "L"])
            .rule("L", vec!["item"])
            .start("L")
    };
    let ff = compute_ff("list_rr", mk);
    let g = mk(GrammarBuilder::new("list_rr")).build();
    let l_id = *g.rule_names.iter().find(|(_, n)| *n == "L").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    assert!(ff.first(l_id).unwrap().contains(x_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared-prefix / ambiguous FIRST sets
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn shared_prefix_first_only_leading() {
    // S → a b | a c; FIRST(S) = {a}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("S", vec!["a", "b"])
            .rule("S", vec!["a", "c"])
            .start("S")
    };
    let ff = compute_ff("shared", mk);
    let g = mk(GrammarBuilder::new("shared")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    // only 'a' (one terminal)
    let count = g
        .tokens
        .keys()
        .filter(|tid| first.contains(tid.0 as usize))
        .count();
    assert_eq!(count, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Empty grammar edge case
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_does_not_panic() {
    let mut g = adze_ir::Grammar::new("empty".to_string());
    g.normalize();
    let _ = FirstFollowSets::compute(&g);
}

// ═══════════════════════════════════════════════════════════════════════════
// Debug/Clone/formatting
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_follow_sets_implements_debug() {
    let ff = compute_ff("dbg", |b| b.token("a", "a").rule("S", vec!["a"]).start("S"));
    let dbg = format!("{ff:?}");
    assert!(dbg.contains("first"));
}

#[test]
fn first_follow_sets_implements_clone() {
    let ff = compute_ff("cln", |b| b.token("a", "a").rule("S", vec!["a"]).start("S"));
    let ff2 = ff.clone();
    let g = GrammarBuilder::new("cln")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert_eq!(
        ff.first(s_id).unwrap().contains(a_id.0 as usize),
        ff2.first(s_id).unwrap().contains(a_id.0 as usize),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Missing symbol lookups
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_of_unknown_symbol_returns_none() {
    let ff = compute_ff("unk", |b| b.token("a", "a").rule("S", vec!["a"]).start("S"));
    assert!(ff.first(SymbolId(9999)).is_none());
}

#[test]
fn follow_of_unknown_symbol_returns_none() {
    let ff = compute_ff("unk2", |b| {
        b.token("a", "a").rule("S", vec!["a"]).start("S")
    });
    assert!(ff.follow(SymbolId(9999)).is_none());
}

#[test]
fn is_nullable_unknown_symbol_false() {
    let ff = compute_ff("unk3", |b| {
        b.token("a", "a").rule("S", vec!["a"]).start("S")
    });
    assert!(!ff.is_nullable(SymbolId(9999)));
}

// ═══════════════════════════════════════════════════════════════════════════
// FOLLOW set interaction with left recursion
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_left_recursive_contains_operator() {
    // E → E "+" a | a  →  FOLLOW(E) ⊇ {+, EOF}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("E", vec!["E", "plus", "a"])
            .rule("E", vec!["a"])
            .start("E")
    };
    let ff = compute_ff("lr_follow", mk);
    let g = mk(GrammarBuilder::new("lr_follow")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let follow = ff.follow(e_id).unwrap();
    assert!(follow.contains(plus_id.0 as usize));
    assert!(follow.contains(0));
}

#[test]
fn follow_double_left_recursive() {
    // E → E "+" E | E "*" E | num
    let mk = |b: GrammarBuilder| {
        b.token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("E", vec!["E", "plus", "E"])
            .rule("E", vec!["E", "star", "E"])
            .rule("E", vec!["num"])
            .start("E")
    };
    let ff = compute_ff("dlr", mk);
    let g = mk(GrammarBuilder::new("dlr")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let star_id = *g.tokens.iter().find(|(_, t)| t.name == "star").unwrap().0;
    let follow = ff.follow(e_id).unwrap();
    assert!(follow.contains(plus_id.0 as usize));
    assert!(follow.contains(star_id.0 as usize));
    assert!(follow.contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════
// Grammars with only one production per nonterminal
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_production_nonterminals() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("X", vec!["a"])
            .rule("Y", vec!["b"])
            .rule("Z", vec!["c"])
            .rule("S", vec!["X", "Y", "Z"])
            .start("S")
    };
    let ff = compute_ff("single_prod", mk);
    let g = mk(GrammarBuilder::new("single_prod")).build();
    let x_id = *g.rule_names.iter().find(|(_, n)| *n == "X").unwrap().0;
    let y_id = *g.rule_names.iter().find(|(_, n)| *n == "Y").unwrap().0;
    let z_id = *g.rule_names.iter().find(|(_, n)| *n == "Z").unwrap().0;
    let a_tok = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let c_tok = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    assert!(ff.first(x_id).unwrap().contains(a_tok.0 as usize));
    assert!(ff.first(y_id).unwrap().contains(b_tok.0 as usize));
    assert!(ff.first(z_id).unwrap().contains(c_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Parenthesized / nested grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parenthesized_expression_first() {
    // E → "(" E ")" | num
    let mk = |b: GrammarBuilder| {
        b.token("num", "[0-9]+")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("E", vec!["lp", "E", "rp"])
            .rule("E", vec!["num"])
            .start("E")
    };
    let ff = compute_ff("paren", mk);
    let g = mk(GrammarBuilder::new("paren")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let lp_id = *g.tokens.iter().find(|(_, t)| t.name == "lp").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    let first = ff.first(e_id).unwrap();
    assert!(first.contains(lp_id.0 as usize));
    assert!(first.contains(num_id.0 as usize));
}

#[test]
fn parenthesized_expression_follow() {
    // E → "(" E ")" | num  →  FOLLOW(E) ⊇ {")", EOF}
    let mk = |b: GrammarBuilder| {
        b.token("num", "[0-9]+")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("E", vec!["lp", "E", "rp"])
            .rule("E", vec!["num"])
            .start("E")
    };
    let ff = compute_ff("paren_f", mk);
    let g = mk(GrammarBuilder::new("paren_f")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let rp_id = *g.tokens.iter().find(|(_, t)| t.name == "rp").unwrap().0;
    let follow = ff.follow(e_id).unwrap();
    assert!(follow.contains(rp_id.0 as usize));
    assert!(follow.contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════
// Statement-like grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn statement_list_grammar() {
    // program → stmts; stmts → stmts stmt | stmt; stmt → "x" ";"
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("semi", ";")
            .rule("stmt", vec!["x", "semi"])
            .rule("stmts", vec!["stmts", "stmt"])
            .rule("stmts", vec!["stmt"])
            .rule("program", vec!["stmts"])
            .start("program")
    };
    let ff = compute_ff("stmts", mk);
    let g = mk(GrammarBuilder::new("stmts")).build();
    let prog = *g
        .rule_names
        .iter()
        .find(|(_, n)| *n == "program")
        .unwrap()
        .0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    assert!(ff.first(prog).unwrap().contains(x_id.0 as usize));
}

#[test]
fn assignment_grammar() {
    // S → id "=" E ";"; E → num | id
    let mk = |b: GrammarBuilder| {
        b.token("id", "[a-z]+")
            .token("num", "[0-9]+")
            .token("eq", "=")
            .token("semi", ";")
            .rule("E", vec!["num"])
            .rule("E", vec!["id"])
            .rule("S", vec!["id", "eq", "E", "semi"])
            .start("S")
    };
    let ff = compute_ff("assign", mk);
    let g = mk(GrammarBuilder::new("assign")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let id_tok = *g.tokens.iter().find(|(_, t)| t.name == "id").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(id_tok.0 as usize));
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let semi_tok = *g.tokens.iter().find(|(_, t)| t.name == "semi").unwrap().0;
    assert!(ff.follow(e_id).unwrap().contains(semi_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// FIRST of terminal tokens directly
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_of_terminal_is_itself() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a", "b"])
            .start("S")
    };
    let ff = compute_ff("term_self", mk);
    let g = mk(GrammarBuilder::new("term_self")).build();
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_id = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    assert!(ff.first(a_id).unwrap().contains(a_id.0 as usize));
    assert!(ff.first(b_id).unwrap().contains(b_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Mixed nullable / non-nullable sequences
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_of_nullable_then_nonnullable() {
    // S → N T; N → ε; T → a  →  FIRST(S) = {a}
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("N", vec![])
            .rule("T", vec!["a"])
            .rule("S", vec!["N", "T"])
            .start("S")
    };
    let ff = compute_ff("null_nn", mk);
    let g = mk(GrammarBuilder::new("null_nn")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(a_id.0 as usize));
}

#[test]
fn nullable_with_alternative_first() {
    // S → N | a; N → ε  →  FIRST(S) = {a} (and S nullable via N)
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .rule("N", vec![])
            .rule("S", vec!["N"])
            .rule("S", vec!["a"])
            .start("S")
    };
    let ff = compute_ff("null_alt", mk);
    let g = mk(GrammarBuilder::new("null_alt")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(a_id.0 as usize));
    assert!(ff.is_nullable(s_id));
}

// ═══════════════════════════════════════════════════════════════════════════
// Tokens not used in any rule
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unused_token_has_first_set() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("unused", "u")
            .rule("S", vec!["a"])
            .start("S")
    };
    let ff = compute_ff("unused", mk);
    let g = mk(GrammarBuilder::new("unused")).build();
    let u_id = *g.tokens.iter().find(|(_, t)| t.name == "unused").unwrap().0;
    // Even unused tokens should have a FIRST set entry
    assert!(ff.first(u_id).is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Complex: full arithmetic with parentheses FOLLOW verification
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn full_arithmetic_follow_of_term() {
    // E → E+T | T; T → T*F | F; F → (E) | n
    // FOLLOW(T) ⊇ {+, ), EOF}
    let mk = |b: GrammarBuilder| {
        b.token("n", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("E", vec!["E", "plus", "T"])
            .rule("E", vec!["T"])
            .rule("T", vec!["T", "star", "F"])
            .rule("T", vec!["F"])
            .rule("F", vec!["lp", "E", "rp"])
            .rule("F", vec!["n"])
            .start("E")
    };
    let ff = compute_ff("arith_ft", mk);
    let g = mk(GrammarBuilder::new("arith_ft")).build();
    let t_id = *g.rule_names.iter().find(|(_, n)| *n == "T").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let rp_id = *g.tokens.iter().find(|(_, t)| t.name == "rp").unwrap().0;
    let follow = ff.follow(t_id).unwrap();
    assert!(follow.contains(plus_id.0 as usize));
    assert!(follow.contains(rp_id.0 as usize));
    assert!(follow.contains(0));
}

#[test]
fn full_arithmetic_follow_of_factor() {
    // FOLLOW(F) ⊇ {+, *, ), EOF}
    let mk = |b: GrammarBuilder| {
        b.token("n", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("E", vec!["E", "plus", "T"])
            .rule("E", vec!["T"])
            .rule("T", vec!["T", "star", "F"])
            .rule("T", vec!["F"])
            .rule("F", vec!["lp", "E", "rp"])
            .rule("F", vec!["n"])
            .start("E")
    };
    let ff = compute_ff("arith_ff", mk);
    let g = mk(GrammarBuilder::new("arith_ff")).build();
    let f_id = *g.rule_names.iter().find(|(_, n)| *n == "F").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let star_id = *g.tokens.iter().find(|(_, t)| t.name == "star").unwrap().0;
    let rp_id = *g.tokens.iter().find(|(_, t)| t.name == "rp").unwrap().0;
    let follow = ff.follow(f_id).unwrap();
    assert!(follow.contains(plus_id.0 as usize));
    assert!(follow.contains(star_id.0 as usize));
    assert!(follow.contains(rp_id.0 as usize));
    assert!(follow.contains(0));
}

#[test]
fn full_arithmetic_first_of_expr() {
    let mk = |b: GrammarBuilder| {
        b.token("n", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("E", vec!["E", "plus", "T"])
            .rule("E", vec!["T"])
            .rule("T", vec!["T", "star", "F"])
            .rule("T", vec!["F"])
            .rule("F", vec!["lp", "E", "rp"])
            .rule("F", vec!["n"])
            .start("E")
    };
    let ff = compute_ff("arith_fe", mk);
    let g = mk(GrammarBuilder::new("arith_fe")).build();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let n_id = *g.tokens.iter().find(|(_, t)| t.name == "n").unwrap().0;
    let lp_id = *g.tokens.iter().find(|(_, t)| t.name == "lp").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let first = ff.first(e_id).unwrap();
    assert!(first.contains(n_id.0 as usize));
    assert!(first.contains(lp_id.0 as usize));
    assert!(!first.contains(plus_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Multiple start-symbol alternatives with different leading nonterminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn start_with_multiple_nonterminal_alternatives() {
    // S → A | B | C; A → "a"; B → "b"; C → "c"
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("A", vec!["a"])
            .rule("B", vec!["b"])
            .rule("C", vec!["c"])
            .rule("S", vec!["A"])
            .rule("S", vec!["B"])
            .rule("S", vec!["C"])
            .start("S")
    };
    let ff = compute_ff("multi_start", mk);
    let g = mk(GrammarBuilder::new("multi_start")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_tok = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    let c_tok = *g.tokens.iter().find(|(_, t)| t.name == "c").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_tok.0 as usize));
    assert!(first.contains(b_tok.0 as usize));
    assert!(first.contains(c_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// FOLLOW set for deeply nested nonterminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_deep_nesting() {
    // S → A; A → B; B → C; C → "x" → FOLLOW(C) ⊇ FOLLOW(B) ⊇ FOLLOW(A) ⊇ {EOF}
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("S", vec!["A"])
            .start("S")
    };
    let ff = compute_ff("deep_follow", mk);
    let g = mk(GrammarBuilder::new("deep_follow")).build();
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let b_nt = *g.rule_names.iter().find(|(_, n)| *n == "B").unwrap().0;
    let c_nt = *g.rule_names.iter().find(|(_, n)| *n == "C").unwrap().0;
    assert!(ff.follow(a_nt).unwrap().contains(0));
    assert!(ff.follow(b_nt).unwrap().contains(0));
    assert!(ff.follow(c_nt).unwrap().contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════
// Non-associative precedence
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn non_associative_precedence_computes() {
    let mut g = GrammarBuilder::new("nassoc")
        .token("num", "[0-9]+")
        .token("eq", "==")
        .rule_with_precedence("E", vec!["E", "eq", "E"], 1, Associativity::None)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    assert!(ff.first(e_id).unwrap().contains(num_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Multiple precedence levels
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_precedence_levels() {
    let mut g = GrammarBuilder::new("multiprec")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("pow", "\\^")
        .rule_with_precedence("E", vec!["E", "plus", "E"], 1, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "star", "E"], 2, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "pow", "E"], 3, Associativity::Right)
        .rule("E", vec!["num"])
        .start("E")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let num_id = *g.tokens.iter().find(|(_, t)| t.name == "num").unwrap().0;
    let plus_id = *g.tokens.iter().find(|(_, t)| t.name == "plus").unwrap().0;
    let star_id = *g.tokens.iter().find(|(_, t)| t.name == "star").unwrap().0;
    let pow_id = *g.tokens.iter().find(|(_, t)| t.name == "pow").unwrap().0;

    assert!(ff.first(e_id).unwrap().contains(num_id.0 as usize));
    let follow = ff.follow(e_id).unwrap();
    assert!(follow.contains(plus_id.0 as usize));
    assert!(follow.contains(star_id.0 as usize));
    assert!(follow.contains(pow_id.0 as usize));
    assert!(follow.contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════
// Nullable start symbol
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nullable_start_symbol() {
    // S → ε
    let mk = |b: GrammarBuilder| b.rule("S", vec![]).start("S");
    let ff = compute_ff("null_start", mk);
    let g = mk(GrammarBuilder::new("null_start")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    assert!(ff.is_nullable(s_id));
    assert!(ff.follow(s_id).unwrap().contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════
// Token with same name used in multiple rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn same_token_in_multiple_rules() {
    // A → "x"; B → "x"; S → A | B
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .rule("A", vec!["x"])
            .rule("B", vec!["x"])
            .rule("S", vec!["A"])
            .rule("S", vec!["B"])
            .start("S")
    };
    let ff = compute_ff("same_tok", mk);
    let g = mk(GrammarBuilder::new("same_tok")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(x_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Fragile tokens
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fragile_token_in_first_set() {
    let mut g = GrammarBuilder::new("fragile")
        .fragile_token("ws", "\\s+")
        .token("a", "a")
        .rule("S", vec!["ws", "a"])
        .start("S")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let ws_id = *g.tokens.iter().find(|(_, t)| t.name == "ws").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(ws_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Extra tokens
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extra_token_does_not_affect_first() {
    let mut g = GrammarBuilder::new("extra")
        .token("ws", "\\s+")
        .token("a", "a")
        .extra("ws")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_id = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let ws_id = *g.tokens.iter().find(|(_, t)| t.name == "ws").unwrap().0;
    let first = ff.first(s_id).unwrap();
    assert!(first.contains(a_id.0 as usize));
    assert!(!first.contains(ws_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Wide grammar: 20 alternatives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn twenty_alternatives() {
    let mut builder = GrammarBuilder::new("wide20");
    for i in 0..20 {
        let tname: &str = Box::leak(format!("t{i}").into_boxed_str());
        let patt: &str = Box::leak(format!("tok{i}").into_boxed_str());
        builder = builder.token(tname, patt);
        builder = builder.rule("S", vec![tname]);
    }
    builder = builder.start("S");
    let mut g = builder.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let first = ff.first(s_id).unwrap();
    // All 20 tokens should be in FIRST(S)
    let token_count = g
        .tokens
        .keys()
        .filter(|tid| first.contains(tid.0 as usize))
        .count();
    assert_eq!(token_count, 20);
}

// ═══════════════════════════════════════════════════════════════════════════
// Regression: terminal not in any FIRST
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn terminal_only_in_follow() {
    // S → A "end"; A → "x"  → "end" not in any FIRST of nonterminals
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("end", "end")
            .rule("A", vec!["x"])
            .rule("S", vec!["A", "end"])
            .start("S")
    };
    let ff = compute_ff("end_follow", mk);
    let g = mk(GrammarBuilder::new("end_follow")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let a_nt = *g.rule_names.iter().find(|(_, n)| *n == "A").unwrap().0;
    let end_id = *g.tokens.iter().find(|(_, t)| t.name == "end").unwrap().0;
    // "end" should NOT be in FIRST(S), only "x" is
    assert!(!ff.first(s_id).unwrap().contains(end_id.0 as usize));
    // "end" should be in FOLLOW(A)
    assert!(ff.follow(a_nt).unwrap().contains(end_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Two separate nonterminals with disjoint FIRST sets
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn disjoint_first_sets() {
    let mk = |b: GrammarBuilder| {
        b.token("a", "a")
            .token("b", "b")
            .rule("X", vec!["a"])
            .rule("Y", vec!["b"])
            .rule("S", vec!["X"])
            .rule("S", vec!["Y"])
            .start("S")
    };
    let ff = compute_ff("disjoint", mk);
    let g = mk(GrammarBuilder::new("disjoint")).build();
    let x_nt = *g.rule_names.iter().find(|(_, n)| *n == "X").unwrap().0;
    let y_nt = *g.rule_names.iter().find(|(_, n)| *n == "Y").unwrap().0;
    let a_tok = *g.tokens.iter().find(|(_, t)| t.name == "a").unwrap().0;
    let b_tok = *g.tokens.iter().find(|(_, t)| t.name == "b").unwrap().0;
    // X has only 'a', Y has only 'b'
    assert!(ff.first(x_nt).unwrap().contains(a_tok.0 as usize));
    assert!(!ff.first(x_nt).unwrap().contains(b_tok.0 as usize));
    assert!(ff.first(y_nt).unwrap().contains(b_tok.0 as usize));
    assert!(!ff.first(y_nt).unwrap().contains(a_tok.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// Self-recursive nonterminal (S → S)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn self_recursive_with_base() {
    // S → S "x" | "y"
    let mk = |b: GrammarBuilder| {
        b.token("x", "x")
            .token("y", "y")
            .rule("S", vec!["S", "x"])
            .rule("S", vec!["y"])
            .start("S")
    };
    let ff = compute_ff("self_rec", mk);
    let g = mk(GrammarBuilder::new("self_rec")).build();
    let s_id = *g.rule_names.iter().find(|(_, n)| *n == "S").unwrap().0;
    let y_id = *g.tokens.iter().find(|(_, t)| t.name == "y").unwrap().0;
    let x_id = *g.tokens.iter().find(|(_, t)| t.name == "x").unwrap().0;
    assert!(ff.first(s_id).unwrap().contains(y_id.0 as usize));
    assert!(!ff.first(s_id).unwrap().contains(x_id.0 as usize));
}

// ═══════════════════════════════════════════════════════════════════════════
// compute_normalized with precedence
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compute_normalized_with_precedence() {
    let mut g = GrammarBuilder::new("cnp")
        .token("n", "[0-9]+")
        .token("plus", "\\+")
        .rule_with_precedence("E", vec!["E", "plus", "E"], 1, Associativity::Left)
        .rule("E", vec!["n"])
        .start("E")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e_id = *g.rule_names.iter().find(|(_, n)| *n == "E").unwrap().0;
    let n_id = *g.tokens.iter().find(|(_, t)| t.name == "n").unwrap().0;
    assert!(ff.first(e_id).unwrap().contains(n_id.0 as usize));
}
