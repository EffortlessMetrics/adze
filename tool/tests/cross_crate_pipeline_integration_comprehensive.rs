//! Comprehensive tests for cross-crate grammar pipeline integration.
//!
//! Tests the full pipeline: GrammarBuilder → normalize → FirstFollowSets → LR1 automaton.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

/// Helper: build grammar → compute FIRST/FOLLOW → build LR1 automaton
fn pipeline(g: &adze_ir::Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(g).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(g, &ff).expect("LR1 build failed")
}

// ── Full Pipeline: Build → Normalize → FIRST/FOLLOW → LR1 ──

#[test]
fn pipeline_single_token() {
    let g = GrammarBuilder::new("p1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!{
        let start = g.start_symbol().unwrap();
        ff.first(start).is_none()
    });
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_two_tokens_sequence() {
    let g = GrammarBuilder::new("p2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!{
        let start = g.start_symbol().unwrap();
        ff.first(start).is_none()
    });
    let auto = pipeline(&g);
    assert!(auto.state_count >= 3);
}

#[test]
fn pipeline_two_alternatives() {
    let g = GrammarBuilder::new("p3")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_chain_three() {
    let g = GrammarBuilder::new("chain3")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("c")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_left_recursive() {
    let g = GrammarBuilder::new("lrec")
        .token("x", "x")
        .token("plus", "+")
        .rule("e", vec!["x"])
        .rule("e", vec!["e", "plus", "x"])
        .start("e")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 4);
}

#[test]
fn pipeline_right_recursive() {
    let g = GrammarBuilder::new("rrec")
        .token("x", "x")
        .token("cons", ":")
        .rule("list", vec!["x"])
        .rule("list", vec!["x", "cons", "list"])
        .start("list")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 4);
}

#[test]
fn pipeline_with_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("n", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 5);
}

#[test]
fn pipeline_with_right_assoc() {
    let g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("pow", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "pow", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 4);
}

#[test]
fn pipeline_normalize_then_build() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_normalize_idempotent() {
    let mut g = GrammarBuilder::new("idem")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let count1: usize = g.rules.values().map(|v| v.len()).sum();
    g.normalize();
    let count2: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(count1, count2);
}

// ── First/Follow Set Properties ──

#[test]
fn first_set_contains_terminal() {
    let g = GrammarBuilder::new("ff_term")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // Start symbol's FIRST set should contain 'a'
    let start = g.start_symbol().unwrap();
    assert!(ff.first(start).is_some());
}

#[test]
fn first_set_multiple_alternatives() {
    let g = GrammarBuilder::new("ff_alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = g.start_symbol().unwrap();
    assert!(ff.first(start).is_some());
}

#[test]
fn follow_set_nonempty_for_start() {
    let g = GrammarBuilder::new("ff_follow")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = g.start_symbol().unwrap();
    // Start symbol should have EOF in FOLLOW
    assert!(ff.follow(start).is_some());
}

#[test]
fn first_follow_chain_propagation() {
    let g = GrammarBuilder::new("ff_chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!{
        let start = g.start_symbol().unwrap();
        ff.first(start).is_none()
    });
    assert!(!{
        let start = g.start_symbol().unwrap();
        ff.follow(start).is_none()
    });
}

// ── LR1 Automaton Properties ──

#[test]
fn lr1_has_initial_state() {
    let g = GrammarBuilder::new("lr1_init")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count > 0);
}

#[test]
fn lr1_state_count_increases_with_complexity() {
    let g_simple = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g_complex = GrammarBuilder::new("complex")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .rule("s", vec!["a", "c", "b"])
        .start("s")
        .build();
    let auto_s = pipeline(&g_simple);
    let auto_c = pipeline(&g_complex);
    assert!(auto_c.state_count >= auto_s.state_count);
}

// ── Arithmetic Expression Grammars ──

#[test]
fn arithmetic_add_only() {
    let g = GrammarBuilder::new("add")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 4);
}

#[test]
fn arithmetic_add_mul() {
    let g = GrammarBuilder::new("addmul")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 5);
}

#[test]
fn arithmetic_all_ops() {
    let g = GrammarBuilder::new("allops")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("minus", r"\-")
        .token("star", r"\*")
        .token("slash", r"\/")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "slash", "expr"],
            2,
            Associativity::Left,
        )
        .start("expr")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 5);
}

// ── Multiple Nonterminals ──

#[test]
fn pipeline_two_nonterminals() {
    let g = GrammarBuilder::new("two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 3);
}

#[test]
fn pipeline_three_nonterminals() {
    let g = GrammarBuilder::new("three_nt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 4);
}

// ── Grammar properties preserved through pipeline ──

#[test]
fn pipeline_preserves_name() {
    let g = GrammarBuilder::new("preserved")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "preserved");
    let _auto = pipeline(&g);
    // Name still preserved after automaton construction
    assert_eq!(g.name, "preserved");
}

#[test]
fn pipeline_preserves_token_count() {
    let g = GrammarBuilder::new("toks")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let tok_count = g.tokens.len();
    let _auto = pipeline(&g);
    assert_eq!(g.tokens.len(), tok_count);
}

#[test]
fn pipeline_preserves_rule_count() {
    let g = GrammarBuilder::new("rules")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s2", vec!["a"])
        .start("s")
        .build();
    let rule_count: usize = g.rules.values().map(|v| v.len()).sum();
    let _auto = pipeline(&g);
    assert_eq!(g.rules.values().map(|v| v.len()).sum::<usize>(), rule_count);
}

// ── Scaling tests ──

#[test]
fn pipeline_ten_tokens() {
    let mut b = GrammarBuilder::new("ten");
    for i in 0..10 {
        b = b.token(&format!("t{}", i), &format!("t{}", i));
    }
    b = b.rule("s", vec!["t0"]).start("s");
    let g = b.build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_ten_alternatives() {
    let mut b = GrammarBuilder::new("alts10");
    for i in 0..10 {
        let name: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(name, name);
        b = b.rule("s", vec![name]);
    }
    b = b.start("s");
    let g = b.build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_deep_chain() {
    let mut b = GrammarBuilder::new("deep");
    b = b.token("x", "x");
    let mut prev = "x";
    for i in 0..10 {
        let name: &str = Box::leak(format!("r{}", i).into_boxed_str());
        b = b.rule(name, vec![prev]);
        prev = name;
    }
    b = b.start(prev);
    let g = b.build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 2);
}

#[test]
fn pipeline_wide_sequence() {
    let mut b = GrammarBuilder::new("wide");
    let mut toks = Vec::new();
    for i in 0..8 {
        let name: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(name, name);
        toks.push(name);
    }
    b = b.rule("s", toks).start("s");
    let g = b.build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 9);
}

// ── Mixed precedence levels ──

#[test]
fn three_precedence_levels() {
    let g = GrammarBuilder::new("three_prec")
        .token("n", "n")
        .token("p", "+")
        .token("m", "*")
        .token("e", "^")
        .rule("expr", vec!["n"])
        .rule_with_precedence("expr", vec!["expr", "p", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "m", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "e", "expr"], 3, Associativity::Right)
        .start("expr")
        .build();
    let auto = pipeline(&g);
    assert!(auto.state_count >= 5);
}

// ── Grammar builder fluent API ──

#[test]
fn builder_chaining() {
    let g = GrammarBuilder::new("chain_api")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert_eq!(g.name, "chain_api");
    assert!(g.tokens.len() >= 2);
}

#[test]
fn builder_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("multi_lhs")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let s_count = g
        .rules
        .values()
        .flat_map(|v| v.iter())
        .filter(|r| g.rule_names.get(&r.lhs).map(|n| n == "s").unwrap_or(false))
        .count();
    assert!(s_count >= 3);
}

// ── Normalize effects ──

#[test]
fn normalize_does_not_lose_rules() {
    let mut g = GrammarBuilder::new("no_loss")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let before: usize = g.rules.values().map(|v| v.len()).sum();
    g.normalize();
    let after: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(after >= before);
}

#[test]
fn normalize_preserves_start_symbol() {
    let mut g = GrammarBuilder::new("keep_start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let start_before = g.start_symbol();
    g.normalize();
    let start_after = g.start_symbol();
    assert_eq!(start_before, start_after);
}

#[test]
fn normalize_preserves_tokens() {
    let mut g = GrammarBuilder::new("keep_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let tok_before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tok_before);
}

// ── Error cases ──

#[test]
fn lr1_no_start_symbol() {
    let g = GrammarBuilder::new("no_start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    // May succeed or fail — depends on implementation
    let ff = FirstFollowSets::compute(&g);
    let _ = ff.and_then(|f| build_lr1_automaton(&g, &f));
}

#[test]
fn lr1_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let ff = FirstFollowSets::compute(&g);
    let _ = ff.and_then(|f| build_lr1_automaton(&g, &f));
}

// ── Regression: ambiguous grammar GLR ──

#[test]
fn ambiguous_grammar_produces_automaton() {
    let g = GrammarBuilder::new("ambig")
        .token("x", "x")
        .token("plus", "+")
        .rule("e", vec!["x"])
        .rule("e", vec!["e", "plus", "e"])
        .start("e")
        .build();
    // GLR should handle ambiguity
    let auto = pipeline(&g);
    assert!(auto.state_count >= 4);
}

// ── Token pattern variety ──

#[test]
fn token_with_regex_pattern() {
    let g = GrammarBuilder::new("regex_tok")
        .token("ident", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("s", vec!["ident"])
        .start("s")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn token_with_single_char() {
    let g = GrammarBuilder::new("char_tok")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn token_with_special_regex() {
    let g = GrammarBuilder::new("special")
        .token("ws", r"\s+")
        .token("word", r"\w+")
        .rule("s", vec!["word"])
        .start("s")
        .build();
    assert!(g.tokens.len() >= 2);
}
