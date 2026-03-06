//! Comprehensive v2 test suite for FirstFollowSets.
//!
//! Covers: compute(), first(), follow(), is_nullable(), FixedBitSet ops,
//! epsilon propagation, EOF placement, chain rules, recursive grammars,
//! alternatives, and edge cases.

use adze_glr_core::FirstFollowSets;
use adze_ir::Symbol;
use adze_ir::builder::GrammarBuilder;

// ---------------------------------------------------------------------------
// Helper: look up a symbol ID by name in rule_names or tokens
// ---------------------------------------------------------------------------
fn sym_id(grammar: &adze_ir::Grammar, name: &str) -> adze_ir::SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .or_else(|| {
            grammar
                .tokens
                .iter()
                .find(|(_, t)| t.name == name)
                .map(|(id, _)| *id)
        })
        .unwrap_or_else(|| panic!("symbol `{name}` not found in grammar"))
}

// ===== 1. Basic compute succeeds ===========================================

#[test]
fn test_compute_succeeds_simple_grammar() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(FirstFollowSets::compute(&g).is_ok());
}

// ===== 2. compute with empty grammar =======================================

#[test]
fn test_compute_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    // Should succeed even with no rules/tokens
    assert!(FirstFollowSets::compute(&g).is_ok());
}

// ===== 3. first() returns Some for defined nonterminal =====================

#[test]
fn test_first_returns_some_for_nonterminal() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(sym_id(&g, "s")).is_some());
}

// ===== 4. first() returns Some for terminal ================================

#[test]
fn test_first_returns_some_for_terminal() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(sym_id(&g, "x")).is_some());
}

// ===== 5. first() returns None for unknown symbol ==========================

#[test]
fn test_first_returns_none_for_unknown_symbol() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(adze_ir::SymbolId(9999)).is_none());
}

// ===== 6. follow() returns Some for start ==================================

#[test]
fn test_follow_returns_some_for_start() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.follow(sym_id(&g, "s")).is_some());
}

// ===== 7. follow() returns None for unknown symbol =========================

#[test]
fn test_follow_returns_none_for_unknown_symbol() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.follow(adze_ir::SymbolId(9999)).is_none());
}

// ===== 8. FIRST of terminal is empty (implementation doesn't self-populate) =

#[test]
fn test_first_of_terminal_is_empty() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    let first_a = ff.first(a_id).unwrap();
    // Terminals get initialized sets but the algorithm doesn't self-populate them
    assert!(first_a.is_clear());
}

// ===== 9. FIRST of nonterminal contains the leading terminal ===============

#[test]
fn test_first_of_nonterminal_contains_leading_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = sym_id(&g, "s");
    let a_id = sym_id(&g, "a");
    let first_s = ff.first(s_id).unwrap();
    assert!(first_s.contains(a_id.0 as usize));
}

// ===== 10. FIRST with two alternatives =====================================

#[test]
fn test_first_two_alternatives() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(first_s.contains(sym_id(&g, "a").0 as usize));
    assert!(first_s.contains(sym_id(&g, "b").0 as usize));
    assert!(first_s.count_ones(..) >= 2);
}

// ===== 11. FIRST with three alternatives ===================================

#[test]
fn test_first_three_alternatives() {
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
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(first_s.count_ones(..) >= 3);
}

// ===== 12. FOLLOW of start contains EOF (bit 0) ============================

#[test]
fn test_follow_of_start_contains_eof() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_s = ff.follow(sym_id(&g, "s")).unwrap();
    assert!(
        follow_s.contains(0),
        "FOLLOW(start) must contain EOF (bit 0)"
    );
}

// ===== 13. is_nullable for epsilon rule ====================================

#[test]
fn test_is_nullable_epsilon_rule() {
    let g = GrammarBuilder::new("t")
        .rule("n", vec![])
        .rule("s", vec!["n"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym_id(&g, "n")));
}

// ===== 14. is_nullable returns false for terminal-only rule =================

#[test]
fn test_not_nullable_terminal_only() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(sym_id(&g, "s")));
}

// ===== 15. Nullable propagation: A -> B, B -> ε ============================

#[test]
fn test_nullable_propagation() {
    let g = GrammarBuilder::new("t")
        .rule("b", vec![])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym_id(&g, "b")));
    assert!(ff.is_nullable(sym_id(&g, "s")));
}

// ===== 16. Chain rule: S -> A -> a; FIRST(S) contains a ====================

#[test]
fn test_first_chain_rule() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("mid", vec!["a"])
        .rule("s", vec!["mid"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(first_s.contains(a_id.0 as usize));
}

// ===== 17. Long chain: S -> A -> B -> C -> tok =============================

#[test]
fn test_first_long_chain() {
    let g = GrammarBuilder::new("t")
        .token("tok", "t")
        .rule("cc", vec!["tok"])
        .rule("bb", vec!["cc"])
        .rule("aa", vec!["bb"])
        .rule("s", vec!["aa"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tok_id = sym_id(&g, "tok");
    assert!(
        ff.first(sym_id(&g, "s"))
            .unwrap()
            .contains(tok_id.0 as usize)
    );
    assert!(
        ff.first(sym_id(&g, "aa"))
            .unwrap()
            .contains(tok_id.0 as usize)
    );
    assert!(
        ff.first(sym_id(&g, "bb"))
            .unwrap()
            .contains(tok_id.0 as usize)
    );
}

// ===== 18. FOLLOW propagation: S -> X Y; FIRST(Y) ⊆ FOLLOW(X) =============

#[test]
fn test_follow_propagation_first_of_next() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let b_id = sym_id(&g, "b");
    let follow_x = ff.follow(sym_id(&g, "x")).unwrap();
    assert!(
        follow_x.contains(b_id.0 as usize),
        "FOLLOW(x) should contain FIRST(y) = {{b}}"
    );
}

// ===== 19. FOLLOW of last nonterminal inherits FOLLOW of LHS ===============

#[test]
fn test_follow_last_nonterminal_inherits_lhs() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_s = ff.follow(sym_id(&g, "s")).unwrap();
    let follow_y = ff.follow(sym_id(&g, "y")).unwrap();
    // EOF should be in both FOLLOW(s) and FOLLOW(y) since y is last in s
    assert!(follow_s.contains(0));
    assert!(follow_y.contains(0));
}

// ===== 20. Direct left recursion: S -> S a | a =============================

#[test]
fn test_left_recursion_first() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(first_s.contains(a_id.0 as usize));
}

// ===== 21. Right recursion: S -> a S | a ===================================

#[test]
fn test_right_recursion_first() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    assert!(ff.first(sym_id(&g, "s")).unwrap().contains(a_id.0 as usize));
}

// ===== 22. Mutual recursion: A -> B a, B -> A b | c =======================

#[test]
fn test_mutual_recursion() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("aa", vec!["bb", "a"])
        .rule("bb", vec!["aa", "b"])
        .rule("bb", vec!["c"])
        .rule("s", vec!["aa"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c_id = sym_id(&g, "c");
    // FIRST(bb) contains c, and FIRST(aa) should also contain c (via bb)
    assert!(
        ff.first(sym_id(&g, "bb"))
            .unwrap()
            .contains(c_id.0 as usize)
    );
    assert!(
        ff.first(sym_id(&g, "aa"))
            .unwrap()
            .contains(c_id.0 as usize)
    );
}

// ===== 23. Epsilon + terminal alternative: S -> ε | a ======================

#[test]
fn test_epsilon_and_terminal_alternative() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym_id(&g, "s")));
    let a_id = sym_id(&g, "a");
    assert!(ff.first(sym_id(&g, "s")).unwrap().contains(a_id.0 as usize));
}

// ===== 24. Epsilon propagation: S -> A B, A -> ε, B -> b ==================

#[test]
fn test_epsilon_propagation_first_through_nullable() {
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .rule("aa", vec![])
        .rule("s", vec!["aa", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let b_id = sym_id(&g, "b");
    // FIRST(s) should contain b because aa is nullable
    assert!(ff.first(sym_id(&g, "s")).unwrap().contains(b_id.0 as usize));
}

// ===== 25. Nullable middle: S -> X B Y, B -> ε; FOLLOW(X) contains FIRST(Y)

#[test]
fn test_follow_through_nullable_middle() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("bb", vec![])
        .rule("y", vec!["c"])
        .rule("s", vec!["x", "bb", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c_id = sym_id(&g, "c");
    // FOLLOW(x) should contain FIRST(y) = {c} because bb is nullable
    let follow_x = ff.follow(sym_id(&g, "x")).unwrap();
    assert!(follow_x.contains(c_id.0 as usize));
}

// ===== 26. All nullable: S -> A B, A -> ε, B -> ε =========================

#[test]
fn test_all_nullable_chain() {
    let g = GrammarBuilder::new("t")
        .rule("aa", vec![])
        .rule("bb", vec![])
        .rule("s", vec!["aa", "bb"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym_id(&g, "aa")));
    assert!(ff.is_nullable(sym_id(&g, "bb")));
    assert!(ff.is_nullable(sym_id(&g, "s")));
}

// ===== 27. FixedBitSet::count_ones basic check =============================

#[test]
fn test_fixedbitset_count_ones() {
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
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert_eq!(first_s.count_ones(..), 3);
}

// ===== 28. FixedBitSet::contains for missing element =======================

#[test]
fn test_fixedbitset_contains_false() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let b_id = sym_id(&g, "b");
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    // b is NOT in FIRST(s) because s only derives a
    assert!(!first_s.contains(b_id.0 as usize));
}

// ===== 29. FixedBitSet::len is non-zero ====================================

#[test]
fn test_fixedbitset_len_nonzero() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(!first_s.is_empty());
}

// ===== 30. FIRST of nonterminal with single terminal rule is singleton =====

#[test]
fn test_first_of_nonterminal_singleton() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert_eq!(first_s.count_ones(..), 1);
    assert!(first_s.contains(a_id.0 as usize));
}

// ===== 31. Multiple rules same LHS aggregate FIRST ========================

#[test]
fn test_multiple_rules_aggregate_first() {
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
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(first_s.contains(sym_id(&g, "a").0 as usize));
    assert!(first_s.contains(sym_id(&g, "c").0 as usize));
    assert!(!first_s.contains(sym_id(&g, "b").0 as usize));
    assert!(!first_s.contains(sym_id(&g, "d").0 as usize));
}

// ===== 32. FOLLOW propagation through nonterminal in middle ================

#[test]
fn test_follow_nonterminal_in_middle() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("mid", vec!["b"])
        .rule("s", vec!["a", "mid", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c_id = sym_id(&g, "c");
    let follow_mid = ff.follow(sym_id(&g, "mid")).unwrap();
    assert!(
        follow_mid.contains(c_id.0 as usize),
        "FOLLOW(mid) should contain c"
    );
}

// ===== 33. FOLLOW of nonterminal at end gets FOLLOW of LHS =================

#[test]
fn test_follow_nonterminal_at_end() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("tail", vec!["b"])
        .rule("s", vec!["a", "tail"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // FOLLOW(tail) should contain EOF since tail is at the end of s and FOLLOW(s) has EOF
    assert!(ff.follow(sym_id(&g, "tail")).unwrap().contains(0));
}

// ===== 34. Arithmetic grammar: expr -> expr + term | term ==================

#[test]
fn test_arithmetic_grammar_first() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let num_id = sym_id(&g, "NUM");
    let lp_id = sym_id(&g, "LPAREN");
    // FIRST(expr) = FIRST(term) = FIRST(factor) = { NUM, ( }
    for name in &["expr", "term", "factor"] {
        let first = ff.first(sym_id(&g, name)).unwrap();
        assert!(
            first.contains(num_id.0 as usize),
            "FIRST({name}) should have NUM"
        );
        assert!(
            first.contains(lp_id.0 as usize),
            "FIRST({name}) should have ("
        );
    }
}

// ===== 35. Arithmetic grammar: FOLLOW checks ==============================

#[test]
fn test_arithmetic_grammar_follow() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // FOLLOW(expr) should contain EOF and )
    let follow_expr = ff.follow(sym_id(&g, "expr")).unwrap();
    assert!(follow_expr.contains(0), "FOLLOW(expr) has EOF");
    assert!(follow_expr.contains(sym_id(&g, "RPAREN").0 as usize));
    // FOLLOW(factor) should contain PLUS, STAR, EOF, )
    let follow_factor = ff.follow(sym_id(&g, "factor")).unwrap();
    assert!(follow_factor.contains(sym_id(&g, "PLUS").0 as usize));
    assert!(follow_factor.contains(sym_id(&g, "STAR").0 as usize));
}

// ===== 36. compute is deterministic ========================================

#[test]
fn test_compute_deterministic() {
    let build = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build()
    };
    let ff1 = FirstFollowSets::compute(&build()).unwrap();
    let ff2 = FirstFollowSets::compute(&build()).unwrap();
    let g = build();
    let s = sym_id(&g, "s");
    assert_eq!(
        ff1.first(s).unwrap().count_ones(..),
        ff2.first(s).unwrap().count_ones(..)
    );
}

// ===== 37. Single-token grammar ============================================

#[test]
fn test_single_token_grammar() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x_id = sym_id(&g, "x");
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert_eq!(first_s.count_ones(..), 1);
    assert!(first_s.contains(x_id.0 as usize));
}

// ===== 38. Many alternatives ===============================================

#[test]
fn test_many_alternatives() {
    let g = GrammarBuilder::new("t")
        .token("t1", "1")
        .token("t2", "2")
        .token("t3", "3")
        .token("t4", "4")
        .token("t5", "5")
        .rule("s", vec!["t1"])
        .rule("s", vec!["t2"])
        .rule("s", vec!["t3"])
        .rule("s", vec!["t4"])
        .rule("s", vec!["t5"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_eq!(ff.first(sym_id(&g, "s")).unwrap().count_ones(..), 5);
}

// ===== 39. FIRST doesn't leak non-leading terminals ========================

#[test]
fn test_first_no_leak_non_leading() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    assert!(first_s.contains(sym_id(&g, "a").0 as usize));
    assert!(!first_s.contains(sym_id(&g, "b").0 as usize));
    assert!(!first_s.contains(sym_id(&g, "c").0 as usize));
}

// ===== 40. Nullable with terminal: S -> A b, A -> ε; FIRST(S) has b =======

#[test]
fn test_first_includes_past_nullable() {
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .rule("aa", vec![])
        .rule("s", vec!["aa", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(
        ff.first(sym_id(&g, "s"))
            .unwrap()
            .contains(sym_id(&g, "b").0 as usize)
    );
}

// ===== 41. FOLLOW of non-start nonterminal not having EOF ==================

#[test]
fn test_follow_non_start_may_lack_eof() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("mid", vec!["a"])
        .rule("s", vec!["mid", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_mid = ff.follow(sym_id(&g, "mid")).unwrap();
    // mid is followed by b, so FOLLOW(mid) contains b but not necessarily EOF
    assert!(follow_mid.contains(sym_id(&g, "b").0 as usize));
}

// ===== 42. first_of_sequence with single terminal ==========================

#[test]
fn test_first_of_sequence_single_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    let result = ff.first_of_sequence(&[Symbol::Terminal(a_id)]).unwrap();
    assert!(result.contains(a_id.0 as usize));
    assert_eq!(result.count_ones(..), 1);
}

// ===== 43. first_of_sequence with nonterminal then terminal ================

#[test]
fn test_first_of_sequence_nonterminal_then_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s_id = sym_id(&g, "s");
    let b_id = sym_id(&g, "b");
    let result = ff
        .first_of_sequence(&[Symbol::NonTerminal(s_id), Symbol::Terminal(b_id)])
        .unwrap();
    // s is not nullable, so FIRST of sequence is FIRST(s) = {a}
    assert!(result.contains(sym_id(&g, "a").0 as usize));
    assert!(!result.contains(b_id.0 as usize));
}

// ===== 44. first_of_sequence with nullable prefix ==========================

#[test]
fn test_first_of_sequence_nullable_prefix() {
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .rule("aa", vec![])
        .rule("s", vec!["aa", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let aa_id = sym_id(&g, "aa");
    let b_id = sym_id(&g, "b");
    let result = ff
        .first_of_sequence(&[Symbol::NonTerminal(aa_id), Symbol::Terminal(b_id)])
        .unwrap();
    assert!(result.contains(b_id.0 as usize));
}

// ===== 45. first_of_sequence with empty sequence ===========================

#[test]
fn test_first_of_sequence_empty() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let result = ff.first_of_sequence(&[]).unwrap();
    // Empty sequence should yield an empty FIRST set
    assert_eq!(result.count_ones(..), 0);
}

// ===== 46. Two nonterminals, both nullable: S -> A B c =====================

#[test]
fn test_first_two_nullable_prefix() {
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("aa", vec![])
        .rule("bb", vec![])
        .rule("s", vec!["aa", "bb", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c_id = sym_id(&g, "c");
    assert!(ff.first(sym_id(&g, "s")).unwrap().contains(c_id.0 as usize));
}

// ===== 47. Partial nullable: S -> A B c, A -> ε, B -> b ===================

#[test]
fn test_first_partial_nullable() {
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .token("c", "c")
        .rule("aa", vec![])
        .rule("bb", vec!["b"])
        .rule("s", vec!["aa", "bb", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let b_id = sym_id(&g, "b");
    let c_id = sym_id(&g, "c");
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    // aa nullable → FIRST(s) includes FIRST(bb) = {b}
    assert!(first_s.contains(b_id.0 as usize));
    // bb not nullable → c should NOT appear
    assert!(!first_s.contains(c_id.0 as usize));
}

// ===== 48. Disjoint alternatives share no FIRST ===========================

#[test]
fn test_disjoint_alternatives_disjoint_first() {
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
    let first_x = ff.first(sym_id(&g, "x")).unwrap();
    let first_y = ff.first(sym_id(&g, "y")).unwrap();
    // No overlap
    let a_id = sym_id(&g, "a");
    let b_id = sym_id(&g, "b");
    assert!(first_x.contains(a_id.0 as usize));
    assert!(!first_x.contains(b_id.0 as usize));
    assert!(first_y.contains(b_id.0 as usize));
    assert!(!first_y.contains(a_id.0 as usize));
}

// ===== 49. Nullable start symbol has EOF in FOLLOW =========================

#[test]
fn test_nullable_start_follow_has_eof() {
    let g = GrammarBuilder::new("t")
        .rule("s", vec![])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(sym_id(&g, "s")));
    assert!(ff.follow(sym_id(&g, "s")).unwrap().contains(0));
}

// ===== 50. FixedBitSet::is_clear for no-FIRST case ========================

#[test]
fn test_first_of_only_epsilon_rule_is_clear() {
    let g = GrammarBuilder::new("t")
        .rule("s", vec![])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    // A purely nullable nonterminal with no terminal alternative has empty FIRST
    assert!(first_s.is_clear());
}

// ===== 51. Overlapping FIRST from two nonterminals =========================

#[test]
fn test_overlapping_first_sets() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("x", vec!["b"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    // s -> x (a|b) | y (b)  →  FIRST(s) = {a, b}
    assert!(first_s.contains(sym_id(&g, "a").0 as usize));
    assert!(first_s.contains(sym_id(&g, "b").0 as usize));
    assert_eq!(first_s.count_ones(..), 2);
}

// ===== 52. Diamond-shaped grammar: S -> A | B, A -> c, B -> c ==============

#[test]
fn test_diamond_grammar() {
    let g = GrammarBuilder::new("t")
        .token("c", "c")
        .rule("aa", vec!["c"])
        .rule("bb", vec!["c"])
        .rule("s", vec!["aa"])
        .rule("s", vec!["bb"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c_id = sym_id(&g, "c");
    assert!(ff.first(sym_id(&g, "s")).unwrap().contains(c_id.0 as usize));
    assert_eq!(ff.first(sym_id(&g, "s")).unwrap().count_ones(..), 1);
}

// ===== 53. FOLLOW propagation with multiple uses of same nonterminal =======

#[test]
fn test_follow_multiple_uses_same_nonterminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("p", vec!["x", "b"])
        .rule("q", vec!["x", "c"])
        .rule("s", vec!["p"])
        .rule("s", vec!["q"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_x = ff.follow(sym_id(&g, "x")).unwrap();
    // x appears before b in p and before c in q, so FOLLOW(x) has both
    assert!(follow_x.contains(sym_id(&g, "b").0 as usize));
    assert!(follow_x.contains(sym_id(&g, "c").0 as usize));
}

// ===== 54. is_nullable false for nonterminal with only terminal rules ======

#[test]
fn test_not_nullable_only_terminal_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(sym_id(&g, "s")));
}

// ===== 55. is_nullable false for terminal ===================================

#[test]
fn test_not_nullable_terminal() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(sym_id(&g, "a")));
}

// ===== 56. FOLLOW of nonterminals in sequence ==============================

#[test]
fn test_follow_of_nonterminals_in_sequence() {
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
    let follow_x = ff.follow(sym_id(&g, "x")).unwrap();
    let follow_y = ff.follow(sym_id(&g, "y")).unwrap();
    assert!(follow_x.contains(sym_id(&g, "b").0 as usize));
    assert!(follow_y.contains(sym_id(&g, "c").0 as usize));
}

// ===== 57. Deeply nested chain FOLLOW propagation ==========================

#[test]
fn test_deeply_nested_follow_propagation() {
    // s -> p, p -> q, q -> tok
    // FOLLOW(tok) and FOLLOW(q) and FOLLOW(p) should all contain EOF
    let g = GrammarBuilder::new("t")
        .token("tok", "t")
        .rule("q", vec!["tok"])
        .rule("p", vec!["q"])
        .rule("s", vec!["p"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.follow(sym_id(&g, "s")).unwrap().contains(0));
    assert!(ff.follow(sym_id(&g, "p")).unwrap().contains(0));
    assert!(ff.follow(sym_id(&g, "q")).unwrap().contains(0));
}

// ===== 58. FIRST unchanged by FOLLOW computation ==========================

#[test]
fn test_first_unaffected_by_follow() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(sym_id(&g, "s")).unwrap();
    // FIRST(s) should only contain a, not b or EOF
    assert!(first_s.contains(sym_id(&g, "a").0 as usize));
    assert!(!first_s.contains(sym_id(&g, "b").0 as usize));
    assert!(!first_s.contains(0)); // no EOF in FIRST
}

// ===== 59. Python-like nullable start grammar ==============================

#[test]
fn test_python_like_nullable_start() {
    let g = GrammarBuilder::python_like();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let module_id = g.start_symbol().unwrap();
    assert!(ff.is_nullable(module_id));
    assert!(ff.follow(module_id).unwrap().contains(0));
}

// ===== 60. JavaScript-like non-nullable start ==============================

#[test]
fn test_javascript_like_non_nullable_start() {
    let g = GrammarBuilder::javascript_like();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let program_id = g.start_symbol().unwrap();
    assert!(!ff.is_nullable(program_id));
    let first_prog = ff.first(program_id).unwrap();
    assert!(!first_prog.is_clear());
}

// ===== 61. Right-recursive list: L -> a | a L ==============================

#[test]
fn test_right_recursive_list() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("lst", vec!["a"])
        .rule("lst", vec!["a", "lst"])
        .rule("s", vec!["lst"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    assert_eq!(ff.first(sym_id(&g, "lst")).unwrap().count_ones(..), 1);
    assert!(
        ff.first(sym_id(&g, "lst"))
            .unwrap()
            .contains(a_id.0 as usize)
    );
}

// ===== 62. Left-recursive list: L -> a | L a, FOLLOW(L) has a =============

#[test]
fn test_left_recursive_list_follow() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("lst", vec!["a"])
        .rule("lst", vec!["lst", "a"])
        .rule("s", vec!["lst"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_id = sym_id(&g, "a");
    let follow_lst = ff.follow(sym_id(&g, "lst")).unwrap();
    assert!(follow_lst.contains(a_id.0 as usize));
    assert!(follow_lst.contains(0)); // also EOF
}
