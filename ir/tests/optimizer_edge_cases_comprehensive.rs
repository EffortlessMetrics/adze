//! Comprehensive edge-case tests for the Grammar optimizer and normalize().

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_rule(lhs: u16, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn make_token(id: u16, name: &str, pattern: &str) -> (SymbolId, Token) {
    (
        SymbolId(id),
        Token {
            name: name.to_string(),
            pattern: TokenPattern::String(pattern.to_string()),
            fragile: false,
        },
    )
}

fn simple_expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn count_all_rules(g: &Grammar) -> usize {
    g.all_rules().count()
}

// ===================================================================
// 1. normalize() on simple grammar is idempotent
// ===================================================================

#[test]
fn normalize_simple_grammar_idempotent_rule_count() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let count1 = count_all_rules(&g);
    g.normalize();
    let count2 = count_all_rules(&g);
    assert_eq!(count1, count2, "second normalize changed rule count");
}

#[test]
fn normalize_simple_grammar_idempotent_token_count() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let t1 = g.tokens.len();
    g.normalize();
    let t2 = g.tokens.len();
    assert_eq!(t1, t2);
}

#[test]
fn normalize_simple_grammar_idempotent_name() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let n1 = g.name.clone();
    g.normalize();
    assert_eq!(n1, g.name);
}

#[test]
fn normalize_simple_idempotent_no_complex_symbols() {
    let mut g = simple_expr_grammar();
    g.normalize();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(
                !matches!(
                    sym,
                    Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                ),
                "complex symbol survived normalize"
            );
        }
    }
}

#[test]
fn normalize_single_terminal_rule_idempotent() {
    let mut g = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g.normalize();
    let snap1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(snap1, count_all_rules(&g));
}

#[test]
fn normalize_epsilon_rule_idempotent() {
    let mut g = GrammarBuilder::new("eps")
        .token("x", "x")
        .rule("start", vec![])
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

// ===================================================================
// 2. normalize() on already-normalized grammar
// ===================================================================

#[test]
fn normalize_already_normalized_no_change() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let before = g.clone();
    g.normalize();
    assert_eq!(before.rules.len(), g.rules.len());
    assert_eq!(before.tokens.len(), g.tokens.len());
    assert_eq!(before.name, g.name);
}

#[test]
fn normalize_already_normalized_preserves_rule_names() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let names_before: Vec<_> = g.rule_names.values().cloned().collect();
    g.normalize();
    let names_after: Vec<_> = g.rule_names.values().cloned().collect();
    assert_eq!(names_before, names_after);
}

#[test]
fn normalize_already_normalized_all_rules_count_stable() {
    let mut g = GrammarBuilder::new("stable")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

// ===================================================================
// 3. normalize() preserves start_symbol
// ===================================================================

#[test]
fn normalize_preserves_start_symbol_simple() {
    let mut g = simple_expr_grammar();
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn normalize_preserves_start_symbol_with_optionals() {
    let mut g = Grammar::new("opt_start".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "start".into());
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn normalize_preserves_start_symbol_multi_rule() {
    let mut g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a"])
        .rule("root", vec!["b"])
        .rule("other", vec!["a", "b"])
        .start("root")
        .build();
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

#[test]
fn normalize_preserves_start_symbol_after_repeat() {
    let mut g = Grammar::new("rep_start".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "start".into());
    let before = g.start_symbol();
    g.normalize();
    assert_eq!(before, g.start_symbol());
}

// ===================================================================
// 4. normalize() preserves grammar name
// ===================================================================

#[test]
fn normalize_preserves_name_simple() {
    let mut g = simple_expr_grammar();
    g.normalize();
    assert_eq!(g.name, "expr");
}

#[test]
fn normalize_preserves_name_empty_grammar() {
    let mut g = Grammar::new("empty_grammar".into());
    g.normalize();
    assert_eq!(g.name, "empty_grammar");
}

#[test]
fn normalize_preserves_name_with_complex_symbols() {
    let mut g = Grammar::new("complex_name".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Epsilon,
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "r".into());
    g.normalize();
    assert_eq!(g.name, "complex_name");
}

#[test]
fn normalize_preserves_name_through_double_normalize() {
    let mut g = simple_expr_grammar();
    g.normalize();
    g.normalize();
    assert_eq!(g.name, "expr");
}

// ===================================================================
// 5. normalize() preserves tokens
// ===================================================================

#[test]
fn normalize_preserves_all_token_names() {
    let mut g = GrammarBuilder::new("tok")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let names_before: Vec<_> = g.tokens.values().map(|t| t.name.clone()).collect();
    g.normalize();
    let names_after: Vec<_> = g.tokens.values().map(|t| t.name.clone()).collect();
    assert_eq!(names_before, names_after);
}

#[test]
fn normalize_preserves_token_count() {
    let mut g = GrammarBuilder::new("tc")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(count, g.tokens.len());
}

#[test]
fn normalize_preserves_token_patterns() {
    let mut g = GrammarBuilder::new("pat")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let patterns_before: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    g.normalize();
    let patterns_after: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    assert_eq!(patterns_before, patterns_after);
}

#[test]
fn normalize_preserves_fragile_flag() {
    let mut g = Grammar::new("fragile".into());
    let (tid, _) = make_token(10, "x", "x");
    g.tokens.insert(
        tid,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: true,
        },
    );
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(10))], 0));
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    assert!(g.tokens[&SymbolId(10)].fragile, "fragile flag lost");
}

#[test]
fn normalize_preserves_tokens_with_complex_rhs() {
    let mut g = Grammar::new("cplx_tok".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    let count = g.tokens.len();
    g.normalize();
    assert_eq!(count, g.tokens.len());
}

// ===================================================================
// 6. Double normalize
// ===================================================================

#[test]
fn double_normalize_rule_count_stable() {
    let mut g = Grammar::new("dn".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

#[test]
fn double_normalize_repeat() {
    let mut g = Grammar::new("dnr".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

#[test]
fn double_normalize_repeat_one() {
    let mut g = Grammar::new("dnro".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

#[test]
fn double_normalize_choice() {
    let mut g = Grammar::new("dnc".into());
    let (tid1, tok1) = make_token(10, "x", "x");
    let (tid2, tok2) = make_token(11, "y", "y");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(11)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

#[test]
fn double_normalize_sequence() {
    let mut g = Grammar::new("dns".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(10)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

#[test]
fn double_normalize_nested_optional_repeat() {
    let mut g = Grammar::new("dnor".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(SymbolId(10)),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

// ===================================================================
// 7. Clone then normalize
// ===================================================================

#[test]
fn clone_then_normalize_does_not_affect_original() {
    let g = simple_expr_grammar();
    let original_count = count_all_rules(&g);
    let mut cloned = g.clone();
    cloned.normalize();
    assert_eq!(original_count, count_all_rules(&g));
}

#[test]
fn clone_then_normalize_preserves_original_name() {
    let g = simple_expr_grammar();
    let mut cloned = g.clone();
    cloned.normalize();
    assert_eq!(g.name, "expr");
    assert_eq!(cloned.name, "expr");
}

#[test]
fn clone_then_normalize_preserves_original_tokens() {
    let g = simple_expr_grammar();
    let orig_tokens = g.tokens.len();
    let mut cloned = g.clone();
    cloned.normalize();
    assert_eq!(orig_tokens, g.tokens.len());
}

#[test]
fn clone_with_complex_then_normalize() {
    let mut g = Grammar::new("clone_cplx".into());
    let (tid, tok) = make_token(10, "a", "a");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());

    let original = g.clone();
    g.normalize();

    // Original should still have complex symbols
    let orig_has_complex = original
        .all_rules()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Optional(_))));
    assert!(orig_has_complex, "clone changed original");

    // Normalized should not
    let norm_has_complex = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Optional(_))));
    assert!(!norm_has_complex, "normalize left Optional");
}

#[test]
fn clone_normalize_both_independently() {
    let base = simple_expr_grammar();
    let mut a = base.clone();
    let mut b = base.clone();
    a.normalize();
    b.normalize();
    assert_eq!(count_all_rules(&a), count_all_rules(&b));
}

// ===================================================================
// 8. Normalize then serde
// ===================================================================

#[test]
fn normalize_then_serde_json_roundtrip() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(count_all_rules(&g), count_all_rules(&g2));
}

#[test]
fn normalize_then_serde_preserves_rule_names() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    let names1: Vec<_> = g.rule_names.values().cloned().collect();
    let names2: Vec<_> = g2.rule_names.values().cloned().collect();
    assert_eq!(names1, names2);
}

#[test]
fn normalize_then_serde_preserves_tokens() {
    let mut g = GrammarBuilder::new("stok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    for (id, tok) in &g.tokens {
        assert_eq!(tok, &g2.tokens[id]);
    }
}

#[test]
fn normalize_complex_then_serde_roundtrip() {
    let mut g = Grammar::new("cserde".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(count_all_rules(&g), count_all_rules(&g2));
}

#[test]
fn normalize_then_serde_no_complex_symbols() {
    let mut g = Grammar::new("no_cplx".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Epsilon,
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    for rule in g2.all_rules() {
        for sym in &rule.rhs {
            assert!(
                !matches!(sym, Symbol::Choice(_)),
                "Choice survived serde roundtrip after normalize"
            );
        }
    }
}

// ===================================================================
// 9. normalize() with many alternatives
// ===================================================================

#[test]
fn normalize_choice_with_many_alternatives_expands() {
    let mut g = Grammar::new("many_alt".into());
    let mut toks = Vec::new();
    for i in 0..10u16 {
        let id = 100 + i;
        let (tid, tok) = make_token(id, &format!("t{i}"), &format!("t{i}"));
        g.tokens.insert(tid, tok);
        toks.push(Symbol::Terminal(SymbolId(id)));
    }
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(toks)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // Each choice alternative should become its own rule
    assert!(
        count_all_rules(&g) >= 10,
        "expected at least 10 rules from choice"
    );
}

#[test]
fn normalize_20_alternatives() {
    let mut g = Grammar::new("twenty".into());
    let mut alts = Vec::new();
    for i in 0..20u16 {
        let id = 100 + i;
        let (tid, tok) = make_token(id, &format!("tok{i}"), &format!("v{i}"));
        g.tokens.insert(tid, tok);
        alts.push(Symbol::Terminal(SymbolId(id)));
    }
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(alts)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    assert!(count_all_rules(&g) >= 20);
}

#[test]
fn normalize_choice_then_idempotent() {
    let mut g = Grammar::new("ci".into());
    let (tid1, tok1) = make_token(10, "a", "a");
    let (tid2, tok2) = make_token(11, "b", "b");
    let (tid3, tok3) = make_token(12, "c", "c");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);
    g.tokens.insert(tid3, tok3);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(11)),
            Symbol::Terminal(SymbolId(12)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    let c1 = count_all_rules(&g);
    g.normalize();
    assert_eq!(c1, count_all_rules(&g));
}

#[test]
fn normalize_many_alt_preserves_name() {
    let mut g = Grammar::new("alt_name".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Epsilon,
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    assert_eq!(g.name, "alt_name");
}

#[test]
fn normalize_many_alt_preserves_tokens() {
    let mut g = Grammar::new("at".into());
    for i in 0..5u16 {
        let (tid, tok) = make_token(100 + i, &format!("t{i}"), &format!("v{i}"));
        g.tokens.insert(tid, tok);
    }
    let alts: Vec<_> = (0..5u16)
        .map(|i| Symbol::Terminal(SymbolId(100 + i)))
        .collect();
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(alts)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    let tok_count = g.tokens.len();
    g.normalize();
    assert_eq!(tok_count, g.tokens.len());
}

// ===================================================================
// 10. normalize() with chain grammars
// ===================================================================

#[test]
fn normalize_chain_grammar_preserves_structure() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build();
    g.normalize();
    // All rules should still exist (no complex symbols to expand)
    assert!(count_all_rules(&g) >= 3);
}

#[test]
fn normalize_chain_with_optional_at_end() {
    let mut g = Grammar::new("chain_opt".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    // a -> b, b -> Optional(x)
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0));
    g.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.rule_names.insert(SymbolId(1), "a".into());
    g.rule_names.insert(SymbolId(2), "b".into());
    g.normalize();
    // Optional should be expanded
    let has_optional = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Optional(_))));
    assert!(!has_optional, "Optional not expanded in chain");
}

#[test]
fn normalize_long_chain_grammar() {
    // a -> b -> c -> d -> e -> x
    let mut g = GrammarBuilder::new("long_chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["d"])
        .rule("d", vec!["e"])
        .rule("e", vec!["x"])
        .start("a")
        .build();
    g.normalize();
    assert!(count_all_rules(&g) >= 5);
    assert_eq!(g.name, "long_chain");
}

#[test]
fn normalize_chain_with_repeat_in_middle() {
    let mut g = Grammar::new("chain_rep".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    // a -> b, b -> Repeat(x), ensures chain structure
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0));
    g.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.rule_names.insert(SymbolId(1), "a".into());
    g.rule_names.insert(SymbolId(2), "b".into());
    g.normalize();
    let has_repeat = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Repeat(_))));
    assert!(!has_repeat, "Repeat not expanded in chain");
}

// ===================================================================
// Additional edge cases
// ===================================================================

#[test]
fn normalize_empty_grammar_no_panic() {
    let mut g = Grammar::new("empty".into());
    let result = g.normalize();
    assert!(result.is_empty());
}

#[test]
fn normalize_grammar_with_only_tokens_no_panic() {
    let mut g = Grammar::new("tokens_only".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    let result = g.normalize();
    assert!(result.is_empty());
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn normalize_returns_all_rules() {
    let mut g = simple_expr_grammar();
    let result = g.normalize();
    assert_eq!(result.len(), count_all_rules(&g));
}

#[test]
fn normalize_sequence_flattens_into_rhs() {
    let mut g = Grammar::new("seq".into());
    let (tid1, tok1) = make_token(10, "a", "a");
    let (tid2, tok2) = make_token(11, "b", "b");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(11)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // The sequence should be flattened, so the rule has 2 terminal symbols
    let rule = g.all_rules().find(|r| r.lhs == SymbolId(1)).unwrap();
    assert_eq!(rule.rhs.len(), 2);
    assert!(matches!(rule.rhs[0], Symbol::Terminal(SymbolId(10))));
    assert!(matches!(rule.rhs[1], Symbol::Terminal(SymbolId(11))));
}

#[test]
fn normalize_optional_creates_epsilon_alternative() {
    let mut g = Grammar::new("opt_eps".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // There should be an aux rule with Epsilon alternative
    let has_eps = g.all_rules().any(|r| r.rhs == vec![Symbol::Epsilon]);
    assert!(
        has_eps,
        "normalize should produce epsilon alternative for Optional"
    );
}

#[test]
fn normalize_repeat_creates_self_recursive_rule() {
    let mut g = Grammar::new("rep_rec".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // Repeat creates aux -> aux inner | epsilon
    let has_self_ref = g.all_rules().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::NonTerminal(id) if *id == r.lhs))
    });
    assert!(
        has_self_ref,
        "Repeat should create self-recursive auxiliary rule"
    );
}

#[test]
fn normalize_repeat_one_no_epsilon() {
    let mut g = Grammar::new("rep1_no_eps".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(10))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // RepeatOne produces aux -> aux inner | inner (no epsilon)
    // Find the auxiliary rules (not the original lhs)
    let aux_rules: Vec<_> = g.all_rules().filter(|r| r.lhs != SymbolId(1)).collect();
    let aux_has_eps = aux_rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]);
    assert!(
        !aux_has_eps,
        "RepeatOne should NOT create epsilon alternative"
    );
}

#[test]
fn optimize_grammar_fn_returns_ok_on_simple() {
    let g = simple_expr_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimizer_struct_default() {
    let mut opt = GrammarOptimizer::default();
    let mut g = simple_expr_grammar();
    let stats = opt.optimize(&mut g);
    // Just ensure it runs without panic
    let _ = stats.total();
}

#[test]
fn optimizer_on_builder_grammar() {
    let g = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
    let g = result.unwrap();
    assert!(!g.rules.is_empty());
}

#[test]
fn normalize_preserves_precedence_on_rules() {
    let mut g = Grammar::new("prec_norm".into());
    let (tid1, tok1) = make_token(10, "x", "x");
    let (tid2, tok2) = make_token(11, "+", "+");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(11)),
            Symbol::NonTerminal(SymbolId(1)),
        ],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(10))], 1));
    g.rule_names.insert(SymbolId(1), "expr".into());
    g.normalize();
    let has_prec = g.all_rules().any(|r| {
        r.precedence == Some(PrecedenceKind::Static(5))
            && r.associativity == Some(Associativity::Left)
    });
    assert!(has_prec, "normalize lost precedence");
}

#[test]
fn normalize_preserves_extras() {
    let mut g = GrammarBuilder::new("extras")
        .token("x", "x")
        .token("ws", r"\s+")
        .rule("s", vec!["x"])
        .extra("ws")
        .start("s")
        .build();
    let extras_before = g.extras.clone();
    g.normalize();
    assert_eq!(extras_before, g.extras);
}

#[test]
fn normalize_preserves_externals() {
    let mut g = GrammarBuilder::new("ext")
        .token("x", "x")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ext_count = g.externals.len();
    g.normalize();
    assert_eq!(ext_count, g.externals.len());
}

#[test]
fn normalize_deeply_nested_optional_repeat_choice() {
    let mut g = Grammar::new("deep".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    // Optional(Repeat(Choice([x, Epsilon])))
    let deep = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(10)),
        Symbol::Epsilon,
    ])))));
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![deep],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // All complex symbols should be gone
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(
                !matches!(
                    sym,
                    Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                ),
                "complex symbol survived deep normalize: {sym:?}"
            );
        }
    }
}

#[test]
fn normalize_multiple_complex_in_single_rule() {
    let mut g = Grammar::new("multi_cplx".into());
    let (tid1, tok1) = make_token(10, "a", "a");
    let (tid2, tok2) = make_token(11, "b", "b");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10)))),
            Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(11)))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // Both should be expanded
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(
                !matches!(sym, Symbol::Optional(_) | Symbol::Repeat(_)),
                "complex symbol survived in multi-complex rule"
            );
        }
    }
}

#[test]
fn normalize_mixed_simple_and_complex_symbols() {
    let mut g = Grammar::new("mixed".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10)))),
            Symbol::Terminal(SymbolId(10)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(SymbolId(1), "s".into());
    g.normalize();
    // The rule for SymbolId(1) should now have 3 elements: terminal, non-terminal(aux), terminal
    let rule = g.all_rules().find(|r| r.lhs == SymbolId(1)).unwrap();
    assert_eq!(rule.rhs.len(), 3);
    assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
    assert!(matches!(rule.rhs[1], Symbol::NonTerminal(_)));
    assert!(matches!(rule.rhs[2], Symbol::Terminal(_)));
}

#[test]
fn optimizer_stats_total_sums_all_fields() {
    let stats = optimizer::OptimizationStats {
        removed_unused_symbols: 1,
        inlined_rules: 2,
        merged_tokens: 3,
        optimized_left_recursion: 4,
        eliminated_unit_rules: 5,
    };
    assert_eq!(stats.total(), 15);
}

#[test]
fn optimizer_default_stats_are_zero() {
    let stats = optimizer::OptimizationStats::default();
    assert_eq!(stats.total(), 0);
}

#[test]
fn normalize_python_like_grammar() {
    let mut g = GrammarBuilder::python_like();
    let start = g.start_symbol();
    g.normalize();
    assert_eq!(start, g.start_symbol());
    assert_eq!(g.name, "python_like");
}

#[test]
fn normalize_javascript_like_grammar() {
    let mut g = GrammarBuilder::javascript_like();
    let start = g.start_symbol();
    let tok_count = g.tokens.len();
    g.normalize();
    assert_eq!(start, g.start_symbol());
    assert_eq!(tok_count, g.tokens.len());
}

#[test]
fn optimize_python_like_grammar() {
    let g = GrammarBuilder::python_like();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}

#[test]
fn optimize_javascript_like_grammar() {
    let g = GrammarBuilder::javascript_like();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
}
