#![allow(clippy::needless_range_loop)]

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol, TokenPattern};

// ── Helper ──────────────────────────────────────────────────────────────

/// Look up a SymbolId by its human-readable name stored in `rule_names`.
fn sym_id(grammar: &adze_ir::Grammar, name: &str) -> adze_ir::SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in rule_names"))
}

/// Look up a SymbolId that is registered as a token.
fn tok_id(grammar: &adze_ir::Grammar, name: &str) -> adze_ir::SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

// ── 1. Builder creation and basic chain ─────────────────────────────────

#[test]
fn test_builder_new_sets_name() {
    let g = GrammarBuilder::new("my_lang").build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn test_empty_grammar_has_no_rules_or_tokens() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
    assert!(g.precedences.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.conflicts.is_empty());
}

#[test]
fn test_builder_chaining_returns_same_builder() {
    // Ensures every method returns Self so the chain compiles.
    let g = GrammarBuilder::new("chain")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();
    assert_eq!(g.name, "chain");
    assert_eq!(g.tokens.len(), 2);
    assert_eq!(g.rules.len(), 1);
}

// ── 2. Adding tokens ────────────────────────────────────────────────────

#[test]
fn test_token_string_literal() {
    let g = GrammarBuilder::new("t").token("hello", "hello").build();
    let (_id, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.name, "hello");
    assert_eq!(tok.pattern, TokenPattern::String("hello".to_string()));
    assert!(!tok.fragile);
}

#[test]
fn test_token_regex_pattern() {
    let g = GrammarBuilder::new("t").token("NUMBER", r"\d+").build();
    let (_id, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\d+".to_string()));
}

#[test]
fn test_token_punctuation_literal() {
    let g = GrammarBuilder::new("t").token("+", "+").build();
    let (_id, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("+".to_string()));
}

#[test]
fn test_fragile_token() {
    let g = GrammarBuilder::new("t").fragile_token("SEMI", ";").build();
    let (_id, tok) = g.tokens.iter().next().unwrap();
    assert!(tok.fragile);
    assert_eq!(tok.name, "SEMI");
}

#[test]
fn test_multiple_tokens_unique_ids() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .build();
    assert_eq!(g.tokens.len(), 3);
    let ids: Vec<_> = g.tokens.keys().collect();
    // All IDs should be distinct
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }
}

// ── 3. Adding rules with various symbol types ───────────────────────────

#[test]
fn test_rule_with_terminals() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["NUM", "+", "NUM"])
        .start("sum")
        .build();
    let sum_id = sym_id(&g, "sum");
    let rules = &g.rules[&sum_id];
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 3);
    // All should be Terminal since they were registered as tokens first
    for sym in &rules[0].rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

#[test]
fn test_rule_with_nonterminals() {
    let g = GrammarBuilder::new("t")
        .token("ID", r"[a-z]+")
        .rule("atom", vec!["ID"])
        .rule("expr", vec!["atom"])
        .start("expr")
        .build();
    let expr_id = sym_id(&g, "expr");
    let rules = &g.rules[&expr_id];
    // "atom" is not a token so it should be NonTerminal
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn test_epsilon_rule() {
    let g = GrammarBuilder::new("t")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let id = sym_id(&g, "empty");
    let rules = &g.rules[&id];
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn test_multiple_alternatives_same_lhs() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("x", vec!["C"])
        .start("x")
        .build();
    let id = sym_id(&g, "x");
    assert_eq!(g.rules[&id].len(), 3);
}

// ── 4. Setting start symbol ─────────────────────────────────────────────

#[test]
fn test_start_symbol_comes_first_in_rules() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("beta", vec!["A"])
        .rule("alpha", vec!["beta"])
        .start("alpha")
        .build();
    // "alpha" should be the first key in the rules map
    let first_key = g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "alpha");
}

#[test]
fn test_start_symbol_not_set() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("r", vec!["A"])
        .build();
    // Without explicit start, rules appear in insertion order
    assert_eq!(g.rules.len(), 1);
}

// ── 5. External tokens ──────────────────────────────────────────────────

#[test]
fn test_external_token() {
    let g = GrammarBuilder::new("t").external("INDENT").build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn test_multiple_externals() {
    let g = GrammarBuilder::new("t")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    assert_eq!(g.externals.len(), 3);
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"INDENT"));
    assert!(names.contains(&"DEDENT"));
    assert!(names.contains(&"NEWLINE"));
}

// ── 6. Extras ───────────────────────────────────────────────────────────

#[test]
fn test_extra_token() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

// ── 7. Precedence and associativity ─────────────────────────────────────

#[test]
fn test_rule_with_precedence_left() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym_id(&g, "e");
    let prec_rule = g.rules[&eid]
        .iter()
        .find(|r| r.precedence.is_some())
        .unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_rule_with_precedence_right() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym_id(&g, "e");
    let prec_rule = g.rules[&eid]
        .iter()
        .find(|r| r.precedence.is_some())
        .unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_rule_with_precedence_none() {
    let g = GrammarBuilder::new("t")
        .token("N", r"\d+")
        .token("==", "==")
        .rule_with_precedence("e", vec!["e", "==", "e"], 0, Associativity::None)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let eid = sym_id(&g, "e");
    let prec_rule = g.rules[&eid]
        .iter()
        .find(|r| r.precedence.is_some())
        .unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::None));
}

#[test]
fn test_precedence_declaration() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[1].level, 2);
    assert_eq!(g.precedences[0].associativity, Associativity::Left);
}

#[test]
fn test_multiple_symbols_in_precedence_level() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("-", "-")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .build();
    assert_eq!(g.precedences[0].symbols.len(), 2);
}

// ── 8. Production IDs ───────────────────────────────────────────────────

#[test]
fn test_production_ids_are_sequential() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("r1", vec!["A"])
        .rule("r2", vec!["B"])
        .rule("r3", vec!["A", "B"])
        .build();
    let all_ids: Vec<u16> = g.all_rules().map(|r| r.production_id.0).collect();
    for i in 0..all_ids.len() {
        assert_eq!(all_ids[i], i as u16);
    }
}

// ── 9. Rule names / symbol registry ─────────────────────────────────────

#[test]
fn test_rule_names_populated_for_nonterminals() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .rule("stmt", vec!["expr"])
        .build();
    let names: Vec<&str> = g.rule_names.values().map(|n| n.as_str()).collect();
    assert!(names.contains(&"expr"));
    assert!(names.contains(&"stmt"));
}

#[test]
fn test_punctuation_not_in_rule_names() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .build();
    // Punctuation symbols should NOT appear in rule_names
    let names: Vec<&str> = g.rule_names.values().map(|n| n.as_str()).collect();
    assert!(!names.contains(&"+"));
    assert!(!names.contains(&"("));
    assert!(!names.contains(&")"));
}

// ── 10. Complex grammar construction ────────────────────────────────────

#[test]
fn test_arithmetic_grammar() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build();

    assert_eq!(g.tokens.len(), 5);
    let eid = sym_id(&g, "expr");
    assert_eq!(g.rules[&eid].len(), 4);
    // Start symbol first
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(*first_key, eid);
}

#[test]
fn test_python_like_helper() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
    // Has externals
    assert!(!g.externals.is_empty());
    // Has extras
    assert!(!g.extras.is_empty());
    // Module is the first rule (start symbol)
    let first_key = g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "module");
}

#[test]
fn test_javascript_like_helper() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(g.name, "javascript_like");
    // Has precedence-annotated rules
    let has_prec = g.all_rules().any(|r| r.precedence.is_some());
    assert!(has_prec);
    // Program is the first rule
    let first_key = g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "program");
}

// ── 11. Builder reuse patterns ──────────────────────────────────────────

#[test]
fn test_same_symbol_reused_across_rules() {
    let g = GrammarBuilder::new("t")
        .token("ID", r"[a-z]+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .build();
    // Both rules reference the same token SymbolId
    let id_tok = tok_id(&g, "ID");
    for rules in g.rules.values() {
        for rule in rules {
            for sym in &rule.rhs {
                if let Symbol::Terminal(sid) = sym {
                    assert_eq!(*sid, id_tok);
                }
            }
        }
    }
}

#[test]
fn test_token_then_rule_order_independence() {
    // Declaring token before or after rule should still classify correctly.
    let g1 = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("r", vec!["X"])
        .build();
    let g2 = GrammarBuilder::new("t")
        .rule("r", vec!["X"])
        .token("X", "x")
        .build();

    // In g1, X should be Terminal (token declared first)
    let rid1 = sym_id(&g1, "r");
    assert!(matches!(g1.rules[&rid1][0].rhs[0], Symbol::Terminal(_)));
    // In g2, X should be NonTerminal (token declared after rule)
    let rid2 = sym_id(&g2, "r");
    assert!(matches!(g2.rules[&rid2][0].rhs[0], Symbol::NonTerminal(_)));
}

// ── 12. Fields (default empty) ──────────────────────────────────────────

#[test]
fn test_rules_have_empty_fields_by_default() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("r", vec!["A"])
        .build();
    for rule in g.all_rules() {
        assert!(rule.fields.is_empty());
    }
    assert!(g.fields.is_empty());
}

// ── 13. Alias sequences (default empty) ─────────────────────────────────

#[test]
fn test_alias_sequences_empty_by_default() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("r", vec!["A"])
        .build();
    assert!(g.alias_sequences.is_empty());
    assert_eq!(g.max_alias_sequence_length, 0);
}

// ── 14. Conflict declarations ───────────────────────────────────────────

#[test]
fn test_no_conflicts_by_default() {
    let g = GrammarBuilder::new("t").build();
    assert!(g.conflicts.is_empty());
}

// ── 15. Edge cases ──────────────────────────────────────────────────────

#[test]
fn test_negative_precedence() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule_with_precedence("r", vec!["A"], -5, Associativity::Left)
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(-5)));
}

#[test]
fn test_grammar_with_many_rules() {
    let mut builder = GrammarBuilder::new("big");
    builder = builder.token("T", "t");
    for i in 0..20 {
        let name_owned = format!("rule_{i}");
        // Leak is fine in tests
        let name: &'static str = Box::leak(name_owned.into_boxed_str());
        builder = builder.rule(name, vec!["T"]);
    }
    let g = builder.start("rule_0").build();
    assert_eq!(g.rules.len(), 20);
    // start comes first
    let first_key = g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "rule_0");
}

#[test]
fn test_symbol_id_starts_at_one() {
    // SymbolId(0) is reserved for EOF
    let g = GrammarBuilder::new("t").token("A", "a").build();
    let id = tok_id(&g, "A");
    assert!(id.0 >= 1, "SymbolId should start at 1 (0 reserved for EOF)");
}

#[test]
fn test_regex_slash_delimited_pattern() {
    let g = GrammarBuilder::new("t")
        .token("FLOAT", r"/\d+\.\d+/")
        .build();
    let (_id, tok) = g.tokens.iter().next().unwrap();
    // Slash-delimited patterns should strip the slashes
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\d+\.\d+".to_string()));
}
