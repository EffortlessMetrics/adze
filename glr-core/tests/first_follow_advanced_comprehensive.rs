//! Advanced comprehensive tests for FIRST/FOLLOW set computation edge cases.
#![cfg(feature = "test-api")]

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, SymbolId};

/// Build a grammar, normalize it, look up symbol IDs by name, then compute FIRST/FOLLOW.
struct TestGrammar {
    grammar: Grammar,
    ff: FirstFollowSets,
}

impl TestGrammar {
    fn build(builder_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder) -> Self {
        let mut grammar = builder_fn(GrammarBuilder::new("test")).build();
        grammar.normalize();
        let ff = FirstFollowSets::compute(&grammar).expect("compute should succeed");
        Self { grammar, ff }
    }

    /// Build from a pre-built grammar (for cases needing mut access before compute)
    fn from_grammar(mut grammar: Grammar) -> Self {
        grammar.normalize();
        let ff = FirstFollowSets::compute(&grammar).expect("compute should succeed");
        Self { grammar, ff }
    }

    /// Look up a nonterminal SymbolId by its rule name
    fn nt(&self, name: &str) -> SymbolId {
        *self
            .grammar
            .rule_names
            .iter()
            .find(|(_, n)| n.as_str() == name)
            .unwrap_or_else(|| panic!("nonterminal '{}' not found in rule_names", name))
            .0
    }

    /// Look up a terminal SymbolId by its token name
    fn tok(&self, name: &str) -> SymbolId {
        *self
            .grammar
            .tokens
            .iter()
            .find(|(_, t)| t.name == name)
            .unwrap_or_else(|| panic!("token '{}' not found in tokens", name))
            .0
    }

    /// Check if terminal is in FIRST(nonterminal)
    fn first_contains(&self, nt: SymbolId, terminal: SymbolId) -> bool {
        self.ff
            .first(nt)
            .map_or(false, |set| set.contains(terminal.0 as usize))
    }

    /// Check if terminal is in FOLLOW(symbol)
    fn follow_contains(&self, sym: SymbolId, terminal: SymbolId) -> bool {
        self.ff
            .follow(sym)
            .map_or(false, |set| set.contains(terminal.0 as usize))
    }

    /// Check if EOF (symbol 0) is in FOLLOW(symbol)
    fn follow_has_eof(&self, sym: SymbolId) -> bool {
        self.ff.follow(sym).map_or(false, |set| set.contains(0))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. FIRST sets for simple terminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_single_terminal_rule() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
}

#[test]
fn first_two_terminal_sequence_only_leading() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
    assert!(!tg.first_contains(tg.nt("start"), tg.tok("b")));
}

#[test]
fn first_three_terminal_sequence() {
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("y", "y")
            .token("z", "z")
            .rule("start", vec!["x", "y", "z"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("x")));
    assert!(!tg.first_contains(s, tg.tok("y")));
    assert!(!tg.first_contains(s, tg.tok("z")));
}

#[test]
fn first_single_terminal_does_not_contain_other() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
    assert!(!tg.first_contains(tg.nt("start"), tg.tok("b")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. FIRST sets for nonterminals with single production
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_propagates_through_single_chain() {
    let tg = TestGrammar::build(|b| {
        b.token("t", "t")
            .rule("inner", vec!["t"])
            .rule("start", vec!["inner"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("t")));
}

#[test]
fn first_through_double_chain() {
    let tg = TestGrammar::build(|b| {
        b.token("t", "t")
            .rule("cc", vec!["t"])
            .rule("bb", vec!["cc"])
            .rule("start", vec!["bb"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("t")));
    assert!(tg.first_contains(tg.nt("bb"), tg.tok("t")));
}

#[test]
fn first_nonterminal_then_terminal() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "b"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(!tg.first_contains(s, tg.tok("b")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. FIRST sets for nonterminals with multiple productions (union)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_union_two_alternatives() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(tg.first_contains(s, tg.tok("b")));
}

#[test]
fn first_union_three_alternatives() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(tg.first_contains(s, tg.tok("b")));
    assert!(tg.first_contains(s, tg.tok("c")));
}

#[test]
fn first_union_through_nonterminals() {
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("y", "y")
            .rule("aa", vec!["x"])
            .rule("bb", vec!["y"])
            .rule("start", vec!["aa"])
            .rule("start", vec!["bb"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("x")));
    assert!(tg.first_contains(s, tg.tok("y")));
}

#[test]
fn first_union_mixed_terminal_and_nonterminal() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["b"])
            .rule("start", vec!["a"])
            .rule("start", vec!["inner"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(tg.first_contains(s, tg.tok("b")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. FIRST sets for recursive grammars (left/right)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_left_recursive() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "a"])
            .rule("expr", vec!["a"])
            .start("expr")
    });
    let e = tg.nt("expr");
    assert!(tg.first_contains(e, tg.tok("a")));
    assert!(!tg.first_contains(e, tg.tok("plus")));
}

#[test]
fn first_right_recursive() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("expr", vec!["a", "plus", "expr"])
            .rule("expr", vec!["a"])
            .start("expr")
    });
    assert!(tg.first_contains(tg.nt("expr"), tg.tok("a")));
}

#[test]
fn first_mutual_recursion() {
    // aa → bb "a" | "a"; bb → aa "b" | "b"
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("aa", vec!["bb", "a"])
            .rule("aa", vec!["a"])
            .rule("bb", vec!["aa", "b"])
            .rule("bb", vec!["b"])
            .start("aa")
    });
    // FIRST(aa) should contain both 'a' and 'b' (through bb)
    assert!(tg.first_contains(tg.nt("aa"), tg.tok("a")));
    assert!(tg.first_contains(tg.nt("aa"), tg.tok("b")));
    // FIRST(bb) should contain both
    assert!(tg.first_contains(tg.nt("bb"), tg.tok("a")));
    assert!(tg.first_contains(tg.nt("bb"), tg.tok("b")));
}

#[test]
fn first_deeply_left_recursive_multiple_ops() {
    // expr → expr '*' expr | expr '+' expr | num
    let tg = TestGrammar::build(|b| {
        b.token("num", "[0-9]+")
            .token("star", "\\*")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "star", "expr"])
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["num"])
            .start("expr")
    });
    let e = tg.nt("expr");
    assert!(tg.first_contains(e, tg.tok("num")));
    assert!(!tg.first_contains(e, tg.tok("star")));
    assert!(!tg.first_contains(e, tg.tok("plus")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. FOLLOW sets include EOF for start symbol
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_start_contains_eof() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    assert!(tg.follow_has_eof(tg.nt("start")));
}

#[test]
fn follow_start_eof_in_recursive_grammar() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "a"])
            .rule("expr", vec!["a"])
            .start("expr")
    });
    assert!(tg.follow_has_eof(tg.nt("expr")));
}

#[test]
fn follow_non_start_may_lack_eof() {
    // start → inner "b"; inner → "a"
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "b"])
            .start("start")
    });
    assert!(tg.follow_has_eof(tg.nt("start")));
    // FOLLOW(inner) should contain 'b', not necessarily EOF
    assert!(tg.follow_contains(tg.nt("inner"), tg.tok("b")));
}

#[test]
fn follow_start_eof_two_alternatives() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    assert!(tg.follow_has_eof(tg.nt("start")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. FOLLOW sets propagate through chained rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_propagates_at_rhs_end() {
    // start → inner; inner → "a"  → FOLLOW(inner) ⊇ FOLLOW(start) = {EOF}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner"])
            .start("start")
    });
    assert!(tg.follow_has_eof(tg.nt("inner")));
}

#[test]
fn follow_propagates_through_two_levels() {
    // start → mid; mid → leaf; leaf → "x"
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"])
            .rule("start", vec!["mid"])
            .start("start")
    });
    assert!(tg.follow_has_eof(tg.nt("leaf")));
    assert!(tg.follow_has_eof(tg.nt("mid")));
}

#[test]
fn follow_includes_first_of_following() {
    // start → aa bb; aa → "a"; bb → "b"  →  FOLLOW(aa) ⊇ FIRST(bb) = {"b"}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("aa", vec!["a"])
            .rule("bb", vec!["b"])
            .rule("start", vec!["aa", "bb"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("aa"), tg.tok("b")));
}

#[test]
fn follow_includes_terminal_after_nt() {
    // start → inner "semi"; inner → "a"
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("semi", ";")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "semi"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("inner"), tg.tok("semi")));
}

#[test]
fn follow_multiple_contexts_union() {
    // start → inner "x" | inner "y"; inner → "a"  → FOLLOW(inner) = {"x","y"}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("x", "x")
            .token("y", "y")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "x"])
            .rule("start", vec!["inner", "y"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("inner"), tg.tok("x")));
    assert!(tg.follow_contains(tg.nt("inner"), tg.tok("y")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. FIRST/FOLLOW with precedence grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_with_left_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let tg = TestGrammar::from_grammar(g);
    assert!(tg.first_contains(tg.nt("expr"), tg.tok("num")));
}

#[test]
fn follow_with_precedence() {
    let g = GrammarBuilder::new("prec_f")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let tg = TestGrammar::from_grammar(g);
    let e = tg.nt("expr");
    assert!(tg.follow_contains(e, tg.tok("plus")));
    assert!(tg.follow_contains(e, tg.tok("star")));
    assert!(tg.follow_has_eof(e));
}

#[test]
fn first_with_right_associative() {
    let g = GrammarBuilder::new("rassoc")
        .token("num", "[0-9]+")
        .token("pow", "\\^")
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let tg = TestGrammar::from_grammar(g);
    assert!(tg.first_contains(tg.nt("expr"), tg.tok("num")));
}

#[test]
fn precedence_does_not_change_first_contents() {
    let g_no = GrammarBuilder::new("np")
        .token("nn", "[0-9]+")
        .token("pp", "\\+")
        .rule("expr", vec!["expr", "pp", "expr"])
        .rule("expr", vec!["nn"])
        .start("expr")
        .build();
    let tg_no = TestGrammar::from_grammar(g_no);

    let g_yes = GrammarBuilder::new("wp")
        .token("nn", "[0-9]+")
        .token("pp", "\\+")
        .rule_with_precedence("expr", vec!["expr", "pp", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["nn"])
        .start("expr")
        .build();
    let tg_yes = TestGrammar::from_grammar(g_yes);

    assert_eq!(
        tg_no.first_contains(tg_no.nt("expr"), tg_no.tok("nn")),
        tg_yes.first_contains(tg_yes.nt("expr"), tg_yes.tok("nn")),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. FIRST/FOLLOW after normalize
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compute_normalized_succeeds() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn compute_after_manual_normalize() {
    let mut g = GrammarBuilder::new("man_norm")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    g.normalize();
    assert!(FirstFollowSets::compute(&g).is_ok());
}

#[test]
fn normalize_then_compute_matches_compute_normalized() {
    let build = || {
        GrammarBuilder::new("cmp")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build()
    };
    let mut g1 = build();
    g1.normalize();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();

    let mut g2 = build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let s1 = *g1
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .unwrap()
        .0;
    let s2 = *g2
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .unwrap()
        .0;
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
fn large_ten_alternatives() {
    let mut builder = GrammarBuilder::new("lg10");
    for i in 0..10 {
        let tname: &str = Box::leak(format!("tok{i}").into_boxed_str());
        let patt: &str = Box::leak(format!("t{i}").into_boxed_str());
        builder = builder.token(tname, patt);
        builder = builder.rule("start", vec![tname]);
    }
    builder = builder.start("start");
    let tg = TestGrammar::from_grammar(builder.build());
    let s = tg.nt("start");
    for i in 0..10 {
        let tname: &str = Box::leak(format!("tok{i}").into_boxed_str());
        assert!(tg.first_contains(s, tg.tok(tname)));
    }
}

#[test]
fn large_chained_nonterminals() {
    let mut builder = GrammarBuilder::new("chain10");
    builder = builder.token("x", "x");
    for i in (0..10).rev() {
        let lhs: &str = Box::leak(format!("n{i}").into_boxed_str());
        if i == 9 {
            builder = builder.rule(lhs, vec!["x"]);
        } else {
            let rhs: &str = Box::leak(format!("n{}", i + 1).into_boxed_str());
            builder = builder.rule(lhs, vec![rhs]);
        }
    }
    builder = builder.rule("start", vec!["n0"]).start("start");
    let tg = TestGrammar::from_grammar(builder.build());
    assert!(tg.first_contains(tg.nt("start"), tg.tok("x")));
}

#[test]
fn large_arithmetic_with_parens() {
    let tg = TestGrammar::build(|b| {
        b.token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lparen", "\\(")
            .token("rparen", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["lparen", "expr", "rparen"])
            .rule("factor", vec!["num"])
            .start("expr")
    });
    for nt_name in ["expr", "term", "factor"] {
        let nt = tg.nt(nt_name);
        assert!(
            tg.first_contains(nt, tg.tok("num")),
            "FIRST({nt_name}) should contain num"
        );
        assert!(
            tg.first_contains(nt, tg.tok("lparen")),
            "FIRST({nt_name}) should contain ("
        );
    }
}

#[test]
fn large_sequence_follow_propagation() {
    // start → aa bb cc dd; aa → "a"; bb → "b"; cc → "c"; dd → "d"
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("aa", vec!["a"])
            .rule("bb", vec!["b"])
            .rule("cc", vec!["c"])
            .rule("dd", vec!["d"])
            .rule("start", vec!["aa", "bb", "cc", "dd"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("aa"), tg.tok("b")));
    assert!(tg.follow_contains(tg.nt("bb"), tg.tok("c")));
    assert!(tg.follow_contains(tg.nt("cc"), tg.tok("d")));
    assert!(tg.follow_has_eof(tg.nt("dd")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Determinism (same grammar → same sets)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_first() {
    let mk = || {
        TestGrammar::build(|b| {
            b.token("a", "a")
                .token("b", "b")
                .rule("start", vec!["a"])
                .rule("start", vec!["b"])
                .start("start")
        })
    };
    let tg1 = mk();
    let tg2 = mk();
    let s1 = tg1.nt("start");
    let s2 = tg2.nt("start");
    assert_eq!(
        tg1.first_contains(s1, tg1.tok("a")),
        tg2.first_contains(s2, tg2.tok("a")),
    );
    assert_eq!(
        tg1.first_contains(s1, tg1.tok("b")),
        tg2.first_contains(s2, tg2.tok("b")),
    );
}

#[test]
fn deterministic_follow() {
    let mk = || {
        TestGrammar::build(|b| {
            b.token("a", "a")
                .token("b", "b")
                .rule("inner", vec!["a"])
                .rule("start", vec!["inner", "b"])
                .start("start")
        })
    };
    let tg1 = mk();
    let tg2 = mk();
    assert_eq!(
        tg1.follow_contains(tg1.nt("inner"), tg1.tok("b")),
        tg2.follow_contains(tg2.nt("inner"), tg2.tok("b")),
    );
}

#[test]
fn deterministic_nullable() {
    let mk = || {
        TestGrammar::build(|b| {
            b.token("a", "a")
                .rule("eps", vec![])
                .rule("start", vec!["eps", "a"])
                .start("start")
        })
    };
    let tg1 = mk();
    let tg2 = mk();
    assert_eq!(
        tg1.ff.is_nullable(tg1.nt("eps")),
        tg2.ff.is_nullable(tg2.nt("eps")),
    );
}

#[test]
fn deterministic_across_ten_runs() {
    let mk = || {
        TestGrammar::build(|b| {
            b.token("a", "a")
                .token("b", "b")
                .rule("start", vec!["a"])
                .rule("start", vec!["b"])
                .start("start")
        })
    };
    let reference = mk();
    let s = reference.nt("start");
    let ref_a = reference.first_contains(s, reference.tok("a"));
    for _ in 0..10 {
        let tg = mk();
        assert_eq!(tg.first_contains(tg.nt("start"), tg.tok("a")), ref_a);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Grammar with all terminals in FIRST of start
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_five_terminals_in_first() {
    let tg = TestGrammar::build(|b| {
        b.token("t0", "t0")
            .token("t1", "t1")
            .token("t2", "t2")
            .token("t3", "t3")
            .token("t4", "t4")
            .rule("start", vec!["t0"])
            .rule("start", vec!["t1"])
            .rule("start", vec!["t2"])
            .rule("start", vec!["t3"])
            .rule("start", vec!["t4"])
            .start("start")
    });
    let s = tg.nt("start");
    for name in ["t0", "t1", "t2", "t3", "t4"] {
        assert!(
            tg.first_contains(s, tg.tok(name)),
            "FIRST(start) missing {name}"
        );
    }
}

#[test]
fn all_terminals_through_nt_alternatives() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("xx", vec!["a"])
            .rule("yy", vec!["b"])
            .rule("zz", vec!["c"])
            .rule("start", vec!["xx"])
            .rule("start", vec!["yy"])
            .rule("start", vec!["zz"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(tg.first_contains(s, tg.tok("b")));
    assert!(tg.first_contains(s, tg.tok("c")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. FOLLOW sets for intermediate nonterminals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_intermediate_in_sequence() {
    // start → aa bb cc
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("aa", vec!["a"])
            .rule("bb", vec!["b"])
            .rule("cc", vec!["c"])
            .rule("start", vec!["aa", "bb", "cc"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("bb"), tg.tok("c")));
}

#[test]
fn follow_intermediate_multiple_contexts() {
    // start → inner "x" | other inner "y"
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("x", "x")
            .token("y", "y")
            .rule("inner", vec!["a"])
            .rule("other", vec!["b"])
            .rule("start", vec!["inner", "x"])
            .rule("start", vec!["other", "inner", "y"])
            .start("start")
    });
    let inner = tg.nt("inner");
    assert!(tg.follow_contains(inner, tg.tok("x")));
    assert!(tg.follow_contains(inner, tg.tok("y")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Nullable / epsilon edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nullable_epsilon_production() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("eps", vec![])
            .rule("start", vec!["eps", "a"])
            .start("start")
    });
    assert!(tg.ff.is_nullable(tg.nt("eps")));
}

#[test]
fn non_nullable_terminal_rule() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    assert!(!tg.ff.is_nullable(tg.nt("start")));
}

#[test]
fn nullable_chain() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("eps", vec![])
            .rule("mid", vec!["eps"])
            .rule("start", vec!["mid", "a"])
            .start("start")
    });
    assert!(tg.ff.is_nullable(tg.nt("eps")));
    assert!(tg.ff.is_nullable(tg.nt("mid")));
}

#[test]
fn first_skips_nullable_prefix() {
    // start → eps "a"; eps → ε  → FIRST(start) ⊇ {"a"}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("eps", vec![])
            .rule("start", vec!["eps", "a"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
}

#[test]
fn first_skips_multiple_nullable_prefixes() {
    // start → e1 e2 "a"; e1 → ε; e2 → ε
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("e1", vec![])
            .rule("e2", vec![])
            .rule("start", vec!["e1", "e2", "a"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
}

#[test]
fn follow_through_nullable_suffix() {
    // start → inner eps; eps → ε  → FOLLOW(inner) ⊇ FOLLOW(start) = {EOF}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("inner", vec!["a"])
            .rule("eps", vec![])
            .rule("start", vec!["inner", "eps"])
            .start("start")
    });
    assert!(tg.follow_has_eof(tg.nt("inner")));
}

#[test]
fn follow_nullable_intermediate() {
    // start → aa eps cc; eps → ε  →  FOLLOW(aa) ⊇ FIRST(cc)
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("c", "c")
            .rule("aa", vec!["a"])
            .rule("eps", vec![])
            .rule("cc", vec!["c"])
            .rule("start", vec!["aa", "eps", "cc"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("aa"), tg.tok("c")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Diamond and branching grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn diamond_first() {
    // start → ll | rr; ll → "a"; rr → "a"  → FIRST(start) = {"a"}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("ll", vec!["a"])
            .rule("rr", vec!["a"])
            .rule("start", vec!["ll"])
            .rule("start", vec!["rr"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
}

#[test]
fn diamond_follow() {
    // start → ll "x" | rr "y"
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("x", "x")
            .token("y", "y")
            .rule("ll", vec!["a"])
            .rule("rr", vec!["b"])
            .rule("start", vec!["ll", "x"])
            .rule("start", vec!["rr", "y"])
            .start("start")
    });
    assert!(tg.follow_contains(tg.nt("ll"), tg.tok("x")));
    assert!(tg.follow_contains(tg.nt("rr"), tg.tok("y")));
}

// ═══════════════════════════════════════════════════════════════════════════
// first_of_sequence tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_of_sequence_single_terminal() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    let a = tg.tok("a");
    let result = tg
        .ff
        .first_of_sequence(&[adze_ir::Symbol::Terminal(a)])
        .unwrap();
    assert!(result.contains(a.0 as usize));
}

#[test]
fn first_of_sequence_nonterminal() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner"])
            .start("start")
    });
    let inner = tg.nt("inner");
    let a = tg.tok("a");
    let result = tg
        .ff
        .first_of_sequence(&[adze_ir::Symbol::NonTerminal(inner)])
        .unwrap();
    assert!(result.contains(a.0 as usize));
}

#[test]
fn first_of_sequence_empty() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    let result = tg.ff.first_of_sequence(&[]).unwrap();
    assert_eq!(result.count_ones(..), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Expression grammar variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn if_then_else_grammar() {
    let tg = TestGrammar::build(|b| {
        b.token("if_kw", "if")
            .token("then_kw", "then")
            .token("else_kw", "else")
            .token("a", "a")
            .token("b", "b")
            .rule("cond", vec!["b"])
            .rule(
                "stmt",
                vec!["if_kw", "cond", "then_kw", "stmt", "else_kw", "stmt"],
            )
            .rule("stmt", vec!["if_kw", "cond", "then_kw", "stmt"])
            .rule("stmt", vec!["a"])
            .start("stmt")
    });
    let s = tg.nt("stmt");
    assert!(tg.first_contains(s, tg.tok("if_kw")));
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(!tg.first_contains(s, tg.tok("then_kw")));
}

#[test]
fn list_left_recursive() {
    // lst → lst "comma" item | item; item → "x"
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("comma", ",")
            .rule("item", vec!["x"])
            .rule("lst", vec!["lst", "comma", "item"])
            .rule("lst", vec!["item"])
            .start("lst")
    });
    let l = tg.nt("lst");
    assert!(tg.first_contains(l, tg.tok("x")));
    assert!(tg.follow_contains(l, tg.tok("comma")));
    assert!(tg.follow_has_eof(l));
}

#[test]
fn list_right_recursive() {
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("comma", ",")
            .rule("item", vec!["x"])
            .rule("lst", vec!["item", "comma", "lst"])
            .rule("lst", vec!["item"])
            .start("lst")
    });
    assert!(tg.first_contains(tg.nt("lst"), tg.tok("x")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared prefix
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn shared_prefix_first_only_leading() {
    // start → "a" "b" | "a" "c"  → FIRST(start) = {"a"}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b"])
            .rule("start", vec!["a", "c"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(!tg.first_contains(s, tg.tok("b")));
    assert!(!tg.first_contains(s, tg.tok("c")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Empty grammar
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_no_panic() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    let _ = FirstFollowSets::compute(&g);
}

// ═══════════════════════════════════════════════════════════════════════════
// Debug / Clone
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn implements_debug() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    let dbg = format!("{:?}", tg.ff);
    assert!(dbg.contains("first"));
}

#[test]
fn implements_clone() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    let cloned = tg.ff.clone();
    let s = tg.nt("start");
    let a = tg.tok("a");
    assert_eq!(
        tg.ff.first(s).unwrap().contains(a.0 as usize),
        cloned.first(s).unwrap().contains(a.0 as usize),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Unknown symbol lookups
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_unknown_returns_none() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    assert!(tg.ff.first(SymbolId(9999)).is_none());
}

#[test]
fn follow_unknown_returns_none() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    assert!(tg.ff.follow(SymbolId(9999)).is_none());
}

#[test]
fn is_nullable_unknown_false() {
    let tg = TestGrammar::build(|b| b.token("a", "a").rule("start", vec!["a"]).start("start"));
    assert!(!tg.ff.is_nullable(SymbolId(9999)));
}

// ═══════════════════════════════════════════════════════════════════════════
// FOLLOW with left recursion
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn follow_left_recursive_contains_operator() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "a"])
            .rule("expr", vec!["a"])
            .start("expr")
    });
    let e = tg.nt("expr");
    assert!(tg.follow_contains(e, tg.tok("plus")));
    assert!(tg.follow_has_eof(e));
}

#[test]
fn follow_double_left_recursive() {
    let tg = TestGrammar::build(|b| {
        b.token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["expr", "star", "expr"])
            .rule("expr", vec!["num"])
            .start("expr")
    });
    let e = tg.nt("expr");
    assert!(tg.follow_contains(e, tg.tok("plus")));
    assert!(tg.follow_contains(e, tg.tok("star")));
    assert!(tg.follow_has_eof(e));
}

// ═══════════════════════════════════════════════════════════════════════════
// Parenthesized / nested
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parenthesized_first() {
    let tg = TestGrammar::build(|b| {
        b.token("num", "[0-9]+")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["lp", "expr", "rp"])
            .rule("expr", vec!["num"])
            .start("expr")
    });
    let e = tg.nt("expr");
    assert!(tg.first_contains(e, tg.tok("lp")));
    assert!(tg.first_contains(e, tg.tok("num")));
}

#[test]
fn parenthesized_follow() {
    let tg = TestGrammar::build(|b| {
        b.token("num", "[0-9]+")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["lp", "expr", "rp"])
            .rule("expr", vec!["num"])
            .start("expr")
    });
    let e = tg.nt("expr");
    assert!(tg.follow_contains(e, tg.tok("rp")));
    assert!(tg.follow_has_eof(e));
}

// ═══════════════════════════════════════════════════════════════════════════
// Statement-like
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn statement_list() {
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("semi", ";")
            .rule("stmt", vec!["x", "semi"])
            .rule("stmts", vec!["stmts", "stmt"])
            .rule("stmts", vec!["stmt"])
            .rule("program", vec!["stmts"])
            .start("program")
    });
    assert!(tg.first_contains(tg.nt("program"), tg.tok("x")));
}

#[test]
fn assignment_grammar() {
    let tg = TestGrammar::build(|b| {
        b.token("id", "[a-z]+")
            .token("num", "[0-9]+")
            .token("eq", "=")
            .token("semi", ";")
            .rule("rhs", vec!["num"])
            .rule("rhs", vec!["id"])
            .rule("start", vec!["id", "eq", "rhs", "semi"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("id")));
    assert!(tg.follow_contains(tg.nt("rhs"), tg.tok("semi")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Full arithmetic FOLLOW verification
// ═══════════════════════════════════════════════════════════════════════════

fn arith_grammar() -> TestGrammar {
    TestGrammar::build(|b| {
        b.token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["lp", "expr", "rp"])
            .rule("factor", vec!["num"])
            .start("expr")
    })
}

#[test]
fn arith_follow_of_term() {
    let tg = arith_grammar();
    let t = tg.nt("term");
    assert!(tg.follow_contains(t, tg.tok("plus")));
    assert!(tg.follow_contains(t, tg.tok("rp")));
    assert!(tg.follow_has_eof(t));
}

#[test]
fn arith_follow_of_factor() {
    let tg = arith_grammar();
    let f = tg.nt("factor");
    assert!(tg.follow_contains(f, tg.tok("plus")));
    assert!(tg.follow_contains(f, tg.tok("star")));
    assert!(tg.follow_contains(f, tg.tok("rp")));
    assert!(tg.follow_has_eof(f));
}

#[test]
fn arith_first_of_expr() {
    let tg = arith_grammar();
    let e = tg.nt("expr");
    assert!(tg.first_contains(e, tg.tok("num")));
    assert!(tg.first_contains(e, tg.tok("lp")));
    assert!(!tg.first_contains(e, tg.tok("plus")));
}

#[test]
fn arith_follow_of_expr() {
    let tg = arith_grammar();
    let e = tg.nt("expr");
    assert!(tg.follow_contains(e, tg.tok("plus")));
    assert!(tg.follow_contains(e, tg.tok("rp")));
    assert!(tg.follow_has_eof(e));
}

// ═══════════════════════════════════════════════════════════════════════════
// More edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn start_multiple_nt_alternatives() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("xx", vec!["a"])
            .rule("yy", vec!["b"])
            .rule("zz", vec!["c"])
            .rule("start", vec!["xx"])
            .rule("start", vec!["yy"])
            .rule("start", vec!["zz"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("a")));
    assert!(tg.first_contains(s, tg.tok("b")));
    assert!(tg.first_contains(s, tg.tok("c")));
}

#[test]
fn follow_deep_nesting() {
    // start → aa; aa → bb; bb → cc; cc → "x"
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .rule("cc", vec!["x"])
            .rule("bb", vec!["cc"])
            .rule("aa", vec!["bb"])
            .rule("start", vec!["aa"])
            .start("start")
    });
    assert!(tg.follow_has_eof(tg.nt("aa")));
    assert!(tg.follow_has_eof(tg.nt("bb")));
    assert!(tg.follow_has_eof(tg.nt("cc")));
}

#[test]
fn non_associative_precedence() {
    let g = GrammarBuilder::new("nassoc")
        .token("num", "[0-9]+")
        .token("eq", "==")
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::None)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let tg = TestGrammar::from_grammar(g);
    assert!(tg.first_contains(tg.nt("expr"), tg.tok("num")));
}

#[test]
fn multiple_precedence_levels() {
    let g = GrammarBuilder::new("mp")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("pow", "\\^")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let tg = TestGrammar::from_grammar(g);
    let e = tg.nt("expr");
    assert!(tg.first_contains(e, tg.tok("num")));
    assert!(tg.follow_contains(e, tg.tok("plus")));
    assert!(tg.follow_contains(e, tg.tok("star")));
    assert!(tg.follow_contains(e, tg.tok("pow")));
    assert!(tg.follow_has_eof(e));
}

#[test]
fn nullable_start() {
    let tg = TestGrammar::build(|b| b.rule("start", vec![]).start("start"));
    assert!(tg.ff.is_nullable(tg.nt("start")));
    assert!(tg.follow_has_eof(tg.nt("start")));
}

#[test]
fn same_token_in_multiple_rules() {
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .rule("aa", vec!["x"])
            .rule("bb", vec!["x"])
            .rule("start", vec!["aa"])
            .rule("start", vec!["bb"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("x")));
}

#[test]
fn fragile_token_in_first() {
    let g = GrammarBuilder::new("frag")
        .fragile_token("ws", "\\s+")
        .token("a", "a")
        .rule("start", vec!["ws", "a"])
        .start("start")
        .build();
    let tg = TestGrammar::from_grammar(g);
    assert!(tg.first_contains(tg.nt("start"), tg.tok("ws")));
}

#[test]
fn extra_token_not_in_first() {
    let g = GrammarBuilder::new("ext")
        .token("ws", "\\s+")
        .token("a", "a")
        .extra("ws")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let tg = TestGrammar::from_grammar(g);
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
    assert!(!tg.first_contains(tg.nt("start"), tg.tok("ws")));
}

#[test]
fn twenty_alternatives() {
    let mut builder = GrammarBuilder::new("wide20");
    for i in 0..20 {
        let tname: &str = Box::leak(format!("tok{i}").into_boxed_str());
        let patt: &str = Box::leak(format!("p{i}").into_boxed_str());
        builder = builder.token(tname, patt);
        builder = builder.rule("start", vec![tname]);
    }
    builder = builder.start("start");
    let tg = TestGrammar::from_grammar(builder.build());
    let s = tg.nt("start");
    let count = tg
        .grammar
        .tokens
        .keys()
        .filter(|tid| tg.ff.first(s).unwrap().contains(tid.0 as usize))
        .count();
    assert_eq!(count, 20);
}

#[test]
fn terminal_only_in_follow() {
    // start → inner "end"; inner → "x"  → "end" not in FIRST(start)
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("end", "end")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner", "end"])
            .start("start")
    });
    assert!(!tg.first_contains(tg.nt("start"), tg.tok("end")));
    assert!(tg.follow_contains(tg.nt("inner"), tg.tok("end")));
}

#[test]
fn disjoint_first_sets() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("xx", vec!["a"])
            .rule("yy", vec!["b"])
            .rule("start", vec!["xx"])
            .rule("start", vec!["yy"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("xx"), tg.tok("a")));
    assert!(!tg.first_contains(tg.nt("xx"), tg.tok("b")));
    assert!(tg.first_contains(tg.nt("yy"), tg.tok("b")));
    assert!(!tg.first_contains(tg.nt("yy"), tg.tok("a")));
}

#[test]
fn self_recursive_with_base() {
    // start → start "x" | "y"
    let tg = TestGrammar::build(|b| {
        b.token("x", "x")
            .token("y", "y")
            .rule("start", vec!["start", "x"])
            .rule("start", vec!["y"])
            .start("start")
    });
    let s = tg.nt("start");
    assert!(tg.first_contains(s, tg.tok("y")));
    assert!(!tg.first_contains(s, tg.tok("x")));
}

#[test]
fn compute_normalized_with_precedence() {
    let mut g = GrammarBuilder::new("cnp")
        .token("nn", "[0-9]+")
        .token("plus", "\\+")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["nn"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e_id = *g
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expr")
        .unwrap()
        .0;
    let n_id = *g.tokens.iter().find(|(_, t)| t.name == "nn").unwrap().0;
    assert!(ff.first(e_id).unwrap().contains(n_id.0 as usize));
}

#[test]
fn single_production_nonterminals() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("xx", vec!["a"])
            .rule("yy", vec!["b"])
            .rule("zz", vec!["c"])
            .rule("start", vec!["xx", "yy", "zz"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("xx"), tg.tok("a")));
    assert!(tg.first_contains(tg.nt("yy"), tg.tok("b")));
    assert!(tg.first_contains(tg.nt("zz"), tg.tok("c")));
}

#[test]
fn nullable_with_alternative_first() {
    // start → eps | "a"; eps → ε
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("eps", vec![])
            .rule("start", vec!["eps"])
            .rule("start", vec!["a"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
    assert!(tg.ff.is_nullable(tg.nt("start")));
}

#[test]
fn nullable_then_nonnullable() {
    // start → eps inner; eps → ε; inner → "a"  → FIRST(start) = {"a"}
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .rule("eps", vec![])
            .rule("inner", vec!["a"])
            .rule("start", vec!["eps", "inner"])
            .start("start")
    });
    assert!(tg.first_contains(tg.nt("start"), tg.tok("a")));
}

#[test]
fn unused_token_still_has_first_entry() {
    let tg = TestGrammar::build(|b| {
        b.token("a", "a")
            .token("unused", "u")
            .rule("start", vec!["a"])
            .start("start")
    });
    assert!(tg.ff.first(tg.tok("unused")).is_some());
}
