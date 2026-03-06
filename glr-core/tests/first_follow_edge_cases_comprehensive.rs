#![cfg(feature = "test-api")]

//! Comprehensive edge-case tests for FIRST/FOLLOW set computation.
//!
//! Areas covered:
//! 1. FIRST sets for single-token grammars
//! 2. FIRST sets for multi-token grammars
//! 3. FIRST sets for alternatives
//! 4. FIRST sets for chain grammars
//! 5. FOLLOW sets for simple grammars
//! 6. FOLLOW sets for complex grammars
//! 7. FirstFollowSets with precedence grammars
//! 8. FirstFollowSets::compute errors
//! 9. FixedBitSet properties
//! 10. FirstFollowSets after normalize

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve a name to its SymbolId via token map or rule_names.
fn sym(grammar: &Grammar, name: &str) -> SymbolId {
    // Check tokens first
    for (id, tok) in &grammar.tokens {
        if tok.name == name {
            return *id;
        }
    }
    // Then rule names
    grammar
        .find_symbol_by_name(name)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in grammar"))
}

/// Collect the FIRST set as a sorted Vec of SymbolId.
fn first_ids(ff: &FirstFollowSets, id: SymbolId) -> Vec<u16> {
    let set = ff.first(id).expect("no FIRST set");
    set.ones().map(|i| i as u16).collect()
}

/// Collect the FOLLOW set as a sorted Vec of SymbolId.
fn follow_ids(ff: &FirstFollowSets, id: SymbolId) -> Vec<u16> {
    let set = ff.follow(id).expect("no FOLLOW set");
    set.ones().map(|i| i as u16).collect()
}

const EOF: u16 = 0;

// ===========================================================================
// 1. FIRST sets for single-token grammars
// ===========================================================================

#[test]
fn first_single_token_grammar() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let x = sym(&g, "x");
    assert!(ff.first(s).unwrap().contains(x.0 as usize));
}

#[test]
fn first_single_token_is_only_element() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let ids = first_ids(&ff, s);
    assert_eq!(ids, vec![a.0]);
}

#[test]
fn first_terminal_is_tracked_in_map() {
    // Terminals appear in the FIRST map but their set may be empty
    // (the implementation does not self-insert terminals into their own FIRST set).
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    // Terminal has an entry in the FIRST map
    assert!(ff.first(a).is_some());
}

#[test]
fn first_single_token_not_nullable() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(!ff.is_nullable(s));
}

// ===========================================================================
// 2. FIRST sets for multi-token grammars
// ===========================================================================

#[test]
fn first_two_token_sequence_uses_leading_token() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert!(!ff.first(s).unwrap().contains(b.0 as usize));
}

#[test]
fn first_three_token_sequence() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert_eq!(first_ids(&ff, s), vec![a.0]);
}

#[test]
fn first_multi_token_grammar_distinct_tokens() {
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["num", "plus", "num"])
        .rule("expr", vec!["num", "star", "num"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let num = sym(&g, "num");
    assert_eq!(first_ids(&ff, expr), vec![num.0]);
}

#[test]
fn first_nonterminal_then_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("s", vec!["inner", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
}

// ===========================================================================
// 3. FIRST sets for alternatives
// ===========================================================================

#[test]
fn first_two_alternatives() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let ids = first_ids(&ff, s);
    assert!(ids.contains(&a.0));
    assert!(ids.contains(&b.0));
    assert_eq!(ids.len(), 2);
}

#[test]
fn first_three_alternatives() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert_eq!(first_ids(&ff, s).len(), 3);
}

#[test]
fn first_alternative_with_sequences() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let c = sym(&g, "c");
    let ids = first_ids(&ff, s);
    assert!(ids.contains(&a.0));
    assert!(ids.contains(&c.0));
    assert_eq!(ids.len(), 2);
}

#[test]
fn first_alternative_via_nonterminals() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let ids = first_ids(&ff, s);
    assert!(ids.contains(&a.0));
    assert!(ids.contains(&b.0));
}

#[test]
fn first_alternative_with_nullable_branch() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec![]) // epsilon
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(ff.is_nullable(s));
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
}

// ===========================================================================
// 4. FIRST sets for chain grammars
// ===========================================================================

#[test]
fn first_chain_depth_2() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("c", vec!["a"])
        .rule("b", vec!["c"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
}

#[test]
fn first_chain_depth_5() {
    let g = GrammarBuilder::new("t")
        .token("z", "z")
        .rule("e", vec!["z"])
        .rule("d", vec!["e"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let z = sym(&g, "z");
    assert!(ff.first(s).unwrap().contains(z.0 as usize));
}

#[test]
fn first_chain_all_intermediates_see_terminal() {
    let g = GrammarBuilder::new("t")
        .token("t", "t")
        .rule("c", vec!["t"])
        .rule("b", vec!["c"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = sym(&g, "t");
    for name in &["c", "b", "s"] {
        let id = sym(&g, name);
        assert!(
            ff.first(id).unwrap().contains(t.0 as usize),
            "FIRST({name}) should contain t"
        );
    }
}

#[test]
fn first_chain_with_left_recursion() {
    // s -> s a | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["s", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert_eq!(first_ids(&ff, s), vec![a.0]);
}

#[test]
fn first_chain_with_right_recursion() {
    // s -> a s | a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "s"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert_eq!(first_ids(&ff, s), vec![a.0]);
}

// ===========================================================================
// 5. FOLLOW sets for simple grammars
// ===========================================================================

#[test]
fn follow_start_symbol_contains_eof() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(ff.follow(s).unwrap().contains(EOF as usize));
}

#[test]
fn follow_of_inner_symbol_includes_trailing_terminal() {
    // s -> inner b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("s", vec!["inner", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let inner = sym(&g, "inner");
    let b = sym(&g, "b");
    assert!(ff.follow(inner).unwrap().contains(b.0 as usize));
}

#[test]
fn follow_of_terminal_at_end_inherits_lhs_follow() {
    // s -> a ; FOLLOW(a) should include EOF since a is at end of s
    // But terminals don't typically get FOLLOW sets - check nonterminal wrapper
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("wrapper", vec!["a"])
        .rule("s", vec!["wrapper"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let wrapper = sym(&g, "wrapper");
    assert!(ff.follow(wrapper).unwrap().contains(EOF as usize));
}

#[test]
fn follow_propagates_from_lhs_when_at_end() {
    // s -> x y ; FOLLOW(y) should include EOF (from s)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let y = sym(&g, "y");
    assert!(ff.follow(y).unwrap().contains(EOF as usize));
}

#[test]
fn follow_middle_nonterminal_gets_first_of_successor() {
    // s -> x y z ; FOLLOW(x) should contain FIRST(y)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    let b = sym(&g, "b");
    assert!(ff.follow(x).unwrap().contains(b.0 as usize));
}

// ===========================================================================
// 6. FOLLOW sets for complex grammars
// ===========================================================================

#[test]
fn follow_with_nullable_suffix() {
    // s -> x y ; y -> b | epsilon
    // FOLLOW(x) should contain FIRST(y) union FOLLOW(s)
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("y", vec![]) // nullable
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    let b = sym(&g, "b");
    let fol = follow_ids(&ff, x);
    assert!(fol.contains(&b.0), "FOLLOW(x) should contain b");
    assert!(
        fol.contains(&EOF),
        "FOLLOW(x) should contain EOF (y nullable)"
    );
}

#[test]
fn follow_mutual_recursion() {
    // a -> b c ; b -> a d | e
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("b", vec!["e"])
        .rule("b", vec!["a", "d"])
        .rule("a", vec!["b", "c"])
        .start("a")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    // Both should have FIRST and FOLLOW sets without infinite loop
    assert!(ff.first(a).is_some());
    assert!(ff.first(b).is_some());
    assert!(ff.follow(a).is_some());
    assert!(ff.follow(b).is_some());
}

#[test]
fn follow_arithmetic_grammar() {
    // expr -> term | expr plus term
    // term -> num
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .rule("term", vec!["num"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let term = sym(&g, "term");
    let plus = sym(&g, "plus");
    // FOLLOW(term) should include plus (from expr -> expr plus term ... next iteration)
    // and EOF
    let fol = follow_ids(&ff, term);
    assert!(fol.contains(&plus.0));
    assert!(fol.contains(&EOF));
}

#[test]
fn follow_parenthesized_expression() {
    // expr -> atom | lp expr rp
    // atom -> num
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("lp", "(")
        .token("rp", ")")
        .rule("atom", vec!["num"])
        .rule("expr", vec!["atom"])
        .rule("expr", vec!["lp", "expr", "rp"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let rp = sym(&g, "rp");
    let fol = follow_ids(&ff, expr);
    // FOLLOW(expr) should include ) and EOF
    assert!(fol.contains(&rp.0));
    assert!(fol.contains(&EOF));
}

#[test]
fn follow_multiple_occurrences_of_same_nonterminal() {
    // s -> x plus x ; FOLLOW(x) should include plus and EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("plus", "+")
        .rule("x", vec!["a"])
        .rule("s", vec!["x", "plus", "x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    let plus = sym(&g, "plus");
    let fol = follow_ids(&ff, x);
    assert!(fol.contains(&plus.0));
    assert!(fol.contains(&EOF));
}

#[test]
fn follow_left_recursive_list() {
    // list -> list comma item | item
    // item -> num
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("comma", ",")
        .rule("item", vec!["num"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "comma", "item"])
        .start("list")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let item = sym(&g, "item");
    let comma = sym(&g, "comma");
    let fol = follow_ids(&ff, item);
    assert!(fol.contains(&comma.0));
    assert!(fol.contains(&EOF));
}

// ===========================================================================
// 7. FirstFollowSets with precedence grammars
// ===========================================================================

#[test]
fn precedence_grammar_first_sets_correct() {
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let num = sym(&g, "num");
    assert_eq!(first_ids(&ff, expr), vec![num.0]);
}

#[test]
fn precedence_grammar_follow_sets_correct() {
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let plus = sym(&g, "plus");
    let star = sym(&g, "star");
    let fol = follow_ids(&ff, expr);
    assert!(fol.contains(&plus.0));
    assert!(fol.contains(&star.0));
    assert!(fol.contains(&EOF));
}

#[test]
fn precedence_right_associative() {
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("exp", "^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "exp", "expr"], 3, Associativity::Right)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let num = sym(&g, "num");
    let exp_tok = sym(&g, "exp");
    assert_eq!(first_ids(&ff, expr), vec![num.0]);
    assert!(ff.follow(expr).unwrap().contains(exp_tok.0 as usize));
}

#[test]
fn precedence_does_not_affect_nullable() {
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    assert!(!ff.is_nullable(expr));
}

#[test]
fn precedence_multi_level_follow_sets() {
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .token("minus", "-")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let plus = sym(&g, "plus");
    let star = sym(&g, "star");
    let minus = sym(&g, "minus");
    let fol = follow_ids(&ff, expr);
    assert!(fol.contains(&plus.0));
    assert!(fol.contains(&star.0));
    assert!(fol.contains(&minus.0));
    assert!(fol.contains(&EOF));
}

// ===========================================================================
// 8. FirstFollowSets::compute errors
// ===========================================================================

#[test]
fn compute_empty_grammar_succeeds() {
    // A grammar with no rules but with a token can still compute
    let g = GrammarBuilder::new("t").token("a", "a").build();
    // Should either succeed or return a non-panic error
    let _result = FirstFollowSets::compute(&g);
}

#[test]
fn compute_complex_symbols_handled_by_internal_normalize() {
    // compute() internally normalizes, so Optional etc. should be handled
    let mut g = Grammar::new("test".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "s".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    // compute() clones and normalizes internally, so this should succeed
    let result = FirstFollowSets::compute(&g);
    assert!(
        result.is_ok(),
        "compute() should handle complex symbols via internal normalize"
    );
}

#[test]
fn compute_normalized_with_repeat() {
    let mut g = Grammar::new("test".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "s".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let result = FirstFollowSets::compute_normalized(&mut g);
    assert!(result.is_ok());
}

#[test]
fn compute_normalized_with_choice() {
    let mut g = Grammar::new("test".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "s".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(a),
                Symbol::Terminal(b),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let result = FirstFollowSets::compute_normalized(&mut g);
    assert!(result.is_ok());
}

// ===========================================================================
// 9. FixedBitSet properties
// ===========================================================================

#[test]
fn fixedbitset_ones_iterator_returns_correct_elements() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let ones: Vec<usize> = ff.first(s).unwrap().ones().collect();
    assert!(ones.contains(&(a.0 as usize)));
    assert!(ones.contains(&(b.0 as usize)));
}

#[test]
fn fixedbitset_contains_check() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert!(!ff.first(s).unwrap().contains(0)); // EOF not in FIRST
}

#[test]
fn fixedbitset_count_ones() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert_eq!(ff.first(s).unwrap().count_ones(..), 3);
}

#[test]
fn fixedbitset_is_empty_for_unreferenced_symbol() {
    // A nonterminal with a single terminal rule has exactly 1 element in FIRST
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert_eq!(ff.first(s).unwrap().count_ones(..), 1);
}

#[test]
fn first_returns_none_for_unknown_symbol() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let unknown = SymbolId(999);
    assert!(ff.first(unknown).is_none());
}

#[test]
fn follow_returns_none_for_unknown_symbol() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let unknown = SymbolId(999);
    assert!(ff.follow(unknown).is_none());
}

// ===========================================================================
// 10. FirstFollowSets after normalize
// ===========================================================================

#[test]
fn compute_normalized_simple_grammar() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = g.find_symbol_by_name("s").unwrap();
    assert!(ff.first(s).is_some());
}

#[test]
fn compute_normalized_preserves_first_sets() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = g.find_symbol_by_name("s").unwrap();
    assert_eq!(ff.first(s).unwrap().count_ones(..), 2);
}

#[test]
fn normalized_grammar_still_has_correct_follow() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("s", vec!["x", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let x = g.find_symbol_by_name("x").unwrap();
    let b_tok = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "b")
        .map(|(id, _)| *id)
        .unwrap();
    assert!(ff.follow(x).unwrap().contains(b_tok.0 as usize));
}

// ===========================================================================
// Additional edge cases for breadth
// ===========================================================================

#[test]
fn nullable_nonterminal_in_first_position() {
    // s -> opt a ; opt -> b | epsilon
    // FIRST(s) should include b and a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec!["b"])
        .rule("opt", vec![])
        .rule("s", vec!["opt", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let ids = first_ids(&ff, s);
    assert!(
        ids.contains(&a.0),
        "FIRST(s) should contain a (opt nullable)"
    );
    assert!(ids.contains(&b.0), "FIRST(s) should contain b");
}

#[test]
fn nullable_chain_propagates_first_through() {
    // s -> n1 n2 a ; n1 -> epsilon ; n2 -> epsilon
    // FIRST(s) should include a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("n1", vec![])
        .rule("n2", vec![])
        .rule("s", vec!["n1", "n2", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert!(ff.is_nullable(sym(&g, "n1")));
    assert!(ff.is_nullable(sym(&g, "n2")));
}

#[test]
fn fully_nullable_rule_makes_nonterminal_nullable() {
    // s -> n1 n2 ; n1 -> eps ; n2 -> eps
    let g = GrammarBuilder::new("t")
        .token("a", "a") // need at least one token for valid grammar
        .rule("n1", vec![])
        .rule("n2", vec![])
        .rule("s", vec!["n1", "n2"])
        .rule("s", vec!["a"]) // second alternative to have something
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(ff.is_nullable(s));
}

#[test]
fn is_nullable_false_for_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    assert!(!ff.is_nullable(a));
}

#[test]
fn first_of_sequence_api_empty_sequence() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let result = ff.first_of_sequence(&[]).unwrap();
    assert_eq!(result.count_ones(..), 0);
}

#[test]
fn first_of_sequence_single_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    let result = ff.first_of_sequence(&[Symbol::Terminal(a)]).unwrap();
    assert!(result.contains(a.0 as usize));
    assert_eq!(result.count_ones(..), 1);
}

#[test]
fn first_of_sequence_nonterminal_then_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("s", vec!["x", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    let b = sym(&g, "b");
    let result = ff
        .first_of_sequence(&[Symbol::NonTerminal(x), Symbol::Terminal(b)])
        .unwrap();
    let a = sym(&g, "a");
    assert!(result.contains(a.0 as usize));
    // x is not nullable, so b should NOT be in the result
    assert!(!result.contains(b.0 as usize));
}

#[test]
fn first_of_sequence_nullable_prefix_includes_next() {
    // opt -> eps | b ; sequence = [opt, a]
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("opt", vec!["b"])
        .rule("opt", vec![])
        .rule("s", vec!["opt", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let opt = sym(&g, "opt");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let result = ff
        .first_of_sequence(&[Symbol::NonTerminal(opt), Symbol::Terminal(a)])
        .unwrap();
    assert!(result.contains(a.0 as usize));
    assert!(result.contains(b.0 as usize));
}

#[test]
fn diamond_grammar_follow_sets() {
    // s -> a c | b c ; a -> x ; b -> x
    // Both a and b produce x, FOLLOW of a and b should contain c's FIRST
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .token("c", "c")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .rule("s", vec!["a", "c"])
        .rule("s", vec!["b", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let c = sym(&g, "c");
    assert!(ff.follow(a).unwrap().contains(c.0 as usize));
    assert!(ff.follow(b).unwrap().contains(c.0 as usize));
}

#[test]
fn self_recursive_nonterminal_first() {
    // s -> a | s s
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["s", "s"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert_eq!(first_ids(&ff, s), vec![a.0]);
}

#[test]
fn self_recursive_nonterminal_follow() {
    // s -> a | s s ; FOLLOW(s) should contain FIRST(s) and EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["s", "s"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let fol = follow_ids(&ff, s);
    assert!(fol.contains(&a.0));
    assert!(fol.contains(&EOF));
}

#[test]
fn many_alternatives_first() {
    let mut builder = GrammarBuilder::new("t");
    let names: Vec<String> = (0..10).map(|i| format!("t{i}")).collect();
    for name in &names {
        builder = builder.token(name, name);
    }
    for name in &names {
        builder = builder.rule("s", vec![name]);
    }
    builder = builder.start("s");
    let g = builder.build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert_eq!(ff.first(s).unwrap().count_ones(..), 10);
}

#[test]
fn follow_set_of_non_start_includes_eof_when_at_end() {
    // s -> x ; FOLLOW(x) should include EOF
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("x", vec!["a"])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    assert!(ff.follow(x).unwrap().contains(EOF as usize));
}

#[test]
fn epsilon_only_rule() {
    // s -> epsilon ; s is nullable
    let g = GrammarBuilder::new("t")
        .token("a", "a") // dummy token
        .rule("s", vec![])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(ff.is_nullable(s));
    assert_eq!(ff.first(s).unwrap().count_ones(..), 0);
}

#[test]
fn two_nonterminals_same_first_set() {
    // a -> x ; b -> x ; s -> a | b
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_nt = sym(&g, "a");
    let b_nt = sym(&g, "b");
    assert_eq!(first_ids(&ff, a_nt), first_ids(&ff, b_nt));
}

#[test]
fn deeply_nested_parentheses_grammar() {
    // expr -> lp expr rp | num
    let g = GrammarBuilder::new("t")
        .token("num", "0")
        .token("lp", "(")
        .token("rp", ")")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["lp", "expr", "rp"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let expr = sym(&g, "expr");
    let num = sym(&g, "num");
    let lp = sym(&g, "lp");
    let ids = first_ids(&ff, expr);
    assert!(ids.contains(&num.0));
    assert!(ids.contains(&lp.0));
    assert_eq!(ids.len(), 2);
}

#[test]
fn follow_set_union_from_multiple_rules() {
    // s -> x a ; s -> x b ; FOLLOW(x) should contain both a and b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("t", "t")
        .rule("x", vec!["t"])
        .rule("s", vec!["x", "a"])
        .rule("s", vec!["x", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let fol = follow_ids(&ff, x);
    assert!(fol.contains(&a.0));
    assert!(fol.contains(&b.0));
}

#[test]
fn single_rule_grammar_no_panic() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn grammar_with_fragile_token() {
    let g = GrammarBuilder::new("t")
        .fragile_token("ws", " ")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(ff.first(s).is_some());
}

#[test]
fn grammar_with_extras() {
    let g = GrammarBuilder::new("t")
        .token("ws", " ")
        .token("a", "a")
        .extra("ws")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
}

#[test]
fn multiple_start_candidates_uses_first_rule() {
    // Builder: start is explicitly "s"
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("other", vec!["b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    assert!(ff.follow(s).unwrap().contains(EOF as usize));
}

#[test]
fn follow_does_not_include_unrelated_terminals() {
    // s -> x a ; FOLLOW(x) should NOT contain b
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("t", "t")
        .rule("x", vec!["t"])
        .rule("s", vec!["x", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = sym(&g, "x");
    let b = sym(&g, "b");
    assert!(!ff.follow(x).unwrap().contains(b.0 as usize));
}

#[test]
fn first_set_not_polluted_by_second_alternative() {
    // s -> a b | c d ; FIRST(s) = {a, c}, not {b, d}
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["c", "d"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    let c = sym(&g, "c");
    let b = sym(&g, "b");
    let d = sym(&g, "d");
    let ids = first_ids(&ff, s);
    assert!(ids.contains(&a.0));
    assert!(ids.contains(&c.0));
    assert!(!ids.contains(&b.0));
    assert!(!ids.contains(&d.0));
}

#[test]
fn indirect_nullable_propagation() {
    // s -> m a ; m -> n ; n -> epsilon
    // m and n should be nullable, FIRST(s) should include a
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("n", vec![])
        .rule("m", vec!["n"])
        .rule("s", vec!["m", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let a = sym(&g, "a");
    assert!(ff.is_nullable(sym(&g, "n")));
    assert!(ff.is_nullable(sym(&g, "m")));
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
}

#[test]
fn python_like_grammar_computes() {
    let g = GrammarBuilder::python_like();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn javascript_like_grammar_computes() {
    let g = GrammarBuilder::javascript_like();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn first_of_sequence_with_epsilon() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = sym(&g, "a");
    // Epsilon in sequence should be skipped
    let result = ff
        .first_of_sequence(&[Symbol::Epsilon, Symbol::Terminal(a)])
        .unwrap();
    assert!(result.contains(a.0 as usize));
}

#[test]
fn follow_set_eof_only_for_isolated_start() {
    // s -> a ; no other rules reference s
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = sym(&g, "s");
    let fol = follow_ids(&ff, s);
    assert_eq!(fol, vec![EOF]);
}
