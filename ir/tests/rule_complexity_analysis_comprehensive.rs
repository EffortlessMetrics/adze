#![allow(clippy::needless_range_loop)]

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, GrammarOptimizer, OptimizationStats, PrecedenceKind, ProductionId,
    Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn sym(id: u16) -> SymbolId {
    SymbolId(id)
}

fn term(id: u16) -> Symbol {
    Symbol::Terminal(SymbolId(id))
}

fn nonterm(id: u16) -> Symbol {
    Symbol::NonTerminal(SymbolId(id))
}

fn simple_rule(lhs: u16, rhs: Vec<Symbol>) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

fn rule_with_prod(lhs: u16, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn add_token(g: &mut Grammar, id: u16, name: &str, pattern: &str) {
    g.tokens.insert(
        SymbolId(id),
        Token {
            name: name.into(),
            pattern: TokenPattern::String(pattern.into()),
            fragile: false,
        },
    );
}

fn add_regex_token(g: &mut Grammar, id: u16, name: &str, pattern: &str) {
    g.tokens.insert(
        SymbolId(id),
        Token {
            name: name.into(),
            pattern: TokenPattern::Regex(pattern.into()),
            fragile: false,
        },
    );
}

/// Compute the nesting depth of a Symbol tree.
fn symbol_depth(sym: &Symbol) -> usize {
    match sym {
        Symbol::Terminal(_)
        | Symbol::NonTerminal(_)
        | Symbol::External(_)
        | Symbol::Epsilon => 0,
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            1 + symbol_depth(inner)
        }
        Symbol::Choice(children) | Symbol::Sequence(children) => {
            1 + children.iter().map(|c| symbol_depth(c)).max().unwrap_or(0)
        }
    }
}

/// Count total symbols (recursively) in a Symbol tree.
fn symbol_count(sym: &Symbol) -> usize {
    match sym {
        Symbol::Terminal(_)
        | Symbol::NonTerminal(_)
        | Symbol::External(_)
        | Symbol::Epsilon => 1,
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            1 + symbol_count(inner)
        }
        Symbol::Choice(children) | Symbol::Sequence(children) => {
            1 + children.iter().map(|c| symbol_count(c)).sum::<usize>()
        }
    }
}

/// Classify a rule as "simple" (flat terminals/nonterminals only) or "complex"
/// (contains nested wrappers like Optional, Repeat, Choice, Sequence).
fn is_complex_rule(rule: &Rule) -> bool {
    rule.rhs.iter().any(|s| {
        matches!(
            s,
            Symbol::Optional(_)
                | Symbol::Repeat(_)
                | Symbol::RepeatOne(_)
                | Symbol::Choice(_)
                | Symbol::Sequence(_)
        )
    })
}

/// Max nesting depth across all RHS symbols in a rule.
fn rule_max_depth(rule: &Rule) -> usize {
    rule.rhs.iter().map(|s| symbol_depth(s)).max().unwrap_or(0)
}

/// Total number of leaf symbols (recursive) in a rule's RHS.
fn rule_symbol_count(rule: &Rule) -> usize {
    rule.rhs.iter().map(|s| symbol_count(s)).sum()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Rule count by symbol
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rule_count_single_symbol_single_rule() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "a", "a");
    g.add_rule(simple_rule(10, vec![term(1)]));
    assert_eq!(g.get_rules_for_symbol(sym(10)).unwrap().len(), 1);
}

#[test]
fn rule_count_single_symbol_multiple_rules() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "a", "a");
    add_token(&mut g, 2, "b", "b");
    g.add_rule(rule_with_prod(10, vec![term(1)], 0));
    g.add_rule(rule_with_prod(10, vec![term(2)], 1));
    g.add_rule(rule_with_prod(10, vec![term(1), term(2)], 2));
    assert_eq!(g.get_rules_for_symbol(sym(10)).unwrap().len(), 3);
}

#[test]
fn rule_count_multiple_symbols() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "x", "x");
    g.add_rule(rule_with_prod(10, vec![term(1)], 0));
    g.add_rule(rule_with_prod(10, vec![term(1), term(1)], 1));
    g.add_rule(rule_with_prod(20, vec![nonterm(10)], 2));
    assert_eq!(g.get_rules_for_symbol(sym(10)).unwrap().len(), 2);
    assert_eq!(g.get_rules_for_symbol(sym(20)).unwrap().len(), 1);
}

#[test]
fn rule_count_missing_symbol_returns_none() {
    let g = Grammar::new("empty".into());
    assert!(g.get_rules_for_symbol(sym(99)).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Production count per nonterminal
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn production_count_via_all_rules() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "t", "t");
    for i in 0..5 {
        g.add_rule(rule_with_prod(10, vec![term(1)], i));
    }
    let total: usize = g.all_rules().count();
    assert_eq!(total, 5);
}

#[test]
fn production_count_across_nonterminals() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "t", "t");
    g.add_rule(rule_with_prod(10, vec![term(1)], 0));
    g.add_rule(rule_with_prod(10, vec![term(1), term(1)], 1));
    g.add_rule(rule_with_prod(20, vec![nonterm(10)], 2));
    g.add_rule(rule_with_prod(20, vec![nonterm(10), term(1)], 3));
    g.add_rule(rule_with_prod(20, vec![term(1)], 4));
    assert_eq!(g.rules.len(), 2); // 2 nonterminals
    assert_eq!(g.all_rules().count(), 5); // 5 total productions
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Symbol nesting depth
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn depth_flat_terminal() {
    assert_eq!(symbol_depth(&term(1)), 0);
}

#[test]
fn depth_flat_nonterminal_and_epsilon() {
    assert_eq!(symbol_depth(&nonterm(1)), 0);
    assert_eq!(symbol_depth(&Symbol::Epsilon), 0);
}

#[test]
fn depth_single_optional() {
    let s = Symbol::Optional(Box::new(term(1)));
    assert_eq!(symbol_depth(&s), 1);
}

#[test]
fn depth_nested_optional_repeat() {
    // Repeat(Optional(Terminal))  →  depth 2
    let s = Symbol::Repeat(Box::new(Symbol::Optional(Box::new(term(1)))));
    assert_eq!(symbol_depth(&s), 2);
}

#[test]
fn depth_choice_with_varying_children() {
    // Choice([Terminal, Optional(Terminal)])  →  depth max(0,1)+1 = 2
    let s = Symbol::Choice(vec![
        term(1),
        Symbol::Optional(Box::new(term(2))),
    ]);
    assert_eq!(symbol_depth(&s), 2);
}

#[test]
fn depth_deeply_nested_structure() {
    // Sequence([Choice([RepeatOne(Optional(Terminal))])])  →  depth 4
    let inner = Symbol::Optional(Box::new(term(1)));
    let repeat = Symbol::RepeatOne(Box::new(inner));
    let choice = Symbol::Choice(vec![repeat]);
    let seq = Symbol::Sequence(vec![choice]);
    assert_eq!(symbol_depth(&seq), 4);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Grammar statistics computation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_stats_empty() {
    let g = Grammar::new("empty".into());
    assert_eq!(g.rules.len(), 0);
    assert_eq!(g.tokens.len(), 0);
    assert_eq!(g.all_rules().count(), 0);
    assert_eq!(g.externals.len(), 0);
    assert_eq!(g.precedences.len(), 0);
}

#[test]
fn grammar_stats_counts_tokens_and_rules() {
    let mut g = Grammar::new("stats".into());
    add_token(&mut g, 1, "a", "a");
    add_token(&mut g, 2, "b", "b");
    add_regex_token(&mut g, 3, "num", r"\d+");
    g.add_rule(rule_with_prod(10, vec![term(1), term(2)], 0));
    g.add_rule(rule_with_prod(10, vec![term(3)], 1));
    g.add_rule(rule_with_prod(20, vec![nonterm(10)], 2));

    assert_eq!(g.tokens.len(), 3);
    assert_eq!(g.rules.len(), 2); // 2 nonterminal symbols
    assert_eq!(g.all_rules().count(), 3); // 3 total productions
}

#[test]
fn grammar_stats_externals_and_extras() {
    let g = GrammarBuilder::new("test")
        .token("WS", r"[ \t]+")
        .token("NUM", r"\d+")
        .external("INDENT")
        .external("DEDENT")
        .extra("WS")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn optimization_stats_total_and_default() {
    let stats = OptimizationStats {
        removed_unused_symbols: 2,
        inlined_rules: 3,
        merged_tokens: 1,
        optimized_left_recursion: 1,
        eliminated_unit_rules: 4,
    };
    assert_eq!(stats.total(), 11);

    let default_stats = OptimizationStats::default();
    assert_eq!(default_stats.total(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Complex vs simple rule classification
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn classify_simple_terminal_and_mixed() {
    let r1 = simple_rule(10, vec![term(1), term(2)]);
    assert!(!is_complex_rule(&r1));
    let r2 = simple_rule(10, vec![term(1), nonterm(20)]);
    assert!(!is_complex_rule(&r2));
}

#[test]
fn classify_complex_optional() {
    let r = simple_rule(10, vec![Symbol::Optional(Box::new(term(1)))]);
    assert!(is_complex_rule(&r));
}

#[test]
fn classify_complex_repeat_and_repeat_one() {
    let r1 = simple_rule(10, vec![Symbol::Repeat(Box::new(nonterm(20)))]);
    assert!(is_complex_rule(&r1));
    let r2 = simple_rule(10, vec![Symbol::RepeatOne(Box::new(nonterm(20)))]);
    assert!(is_complex_rule(&r2));
}

#[test]
fn classify_complex_choice_and_sequence() {
    let r1 = simple_rule(10, vec![Symbol::Choice(vec![term(1), term(2)])]);
    assert!(is_complex_rule(&r1));
    let r2 = simple_rule(10, vec![Symbol::Sequence(vec![term(1), term(2)])]);
    assert!(is_complex_rule(&r2));
}

#[test]
fn classify_epsilon_as_simple() {
    let r = simple_rule(10, vec![Symbol::Epsilon]);
    assert!(!is_complex_rule(&r));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Rule with many alternatives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn many_alternatives_per_nonterminal() {
    let mut g = Grammar::new("many_alts".into());
    add_token(&mut g, 1, "t", "t");
    let n = 20;
    for i in 0..n {
        g.add_rule(rule_with_prod(10, vec![term(1)], i));
    }
    assert_eq!(g.get_rules_for_symbol(sym(10)).unwrap().len(), n as usize);
}

#[test]
fn many_alternatives_distinct_productions() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "a", "a");
    add_token(&mut g, 2, "b", "b");
    let alts: Vec<Vec<Symbol>> = vec![
        vec![term(1)],
        vec![term(2)],
        vec![term(1), term(2)],
        vec![term(2), term(1)],
        vec![term(1), term(1), term(2)],
    ];
    for (i, rhs) in alts.iter().enumerate() {
        g.add_rule(rule_with_prod(10, rhs.clone(), i as u16));
    }
    let rules = g.get_rules_for_symbol(sym(10)).unwrap();
    assert_eq!(rules.len(), 5);
    // Verify each production's RHS length
    let lengths: Vec<usize> = rules.iter().map(|r| r.rhs.len()).collect();
    assert_eq!(lengths, vec![1, 1, 2, 2, 3]);
}

#[test]
fn choice_symbol_with_many_branches() {
    let branches: Vec<Symbol> = (1..=10).map(|i| term(i)).collect();
    let choice = Symbol::Choice(branches);
    assert_eq!(symbol_depth(&choice), 1);
    assert_eq!(symbol_count(&choice), 11); // 1 Choice + 10 terminals
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Token pattern complexity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_pattern_string_and_regex() {
    let t_str = Token {
        name: "plus".into(),
        pattern: TokenPattern::String("+".into()),
        fragile: false,
    };
    assert!(matches!(t_str.pattern, TokenPattern::String(_)));
    if let TokenPattern::String(ref s) = t_str.pattern {
        assert_eq!(s.len(), 1);
    }

    let patterns = [
        r"\d+",
        r"[a-zA-Z_][a-zA-Z0-9_]*",
        r"0[xX][0-9a-fA-F]+",
        r#""([^"\\]|\\.)*""#,
    ];
    for pat in &patterns {
        let t = Token {
            name: "tok".into(),
            pattern: TokenPattern::Regex(pat.to_string()),
            fragile: false,
        };
        if let TokenPattern::Regex(ref r) = t.pattern {
            assert!(!r.is_empty());
        }
    }
}

#[test]
fn token_fragile_flag() {
    let fragile = Token {
        name: "err".into(),
        pattern: TokenPattern::String("error".into()),
        fragile: true,
    };
    let normal = Token {
        name: "ok".into(),
        pattern: TokenPattern::String("ok".into()),
        fragile: false,
    };
    assert!(fragile.fragile);
    assert!(!normal.fragile);
}

#[test]
fn token_pattern_string_vs_regex_distinction() {
    let string_tok = TokenPattern::String("if".into());
    let regex_tok = TokenPattern::Regex("if".into());
    // Same content but different variants
    assert_ne!(string_tok, regex_tok);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Grammar size metrics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_size_total_rhs_symbols() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "a", "a");
    add_token(&mut g, 2, "b", "b");
    g.add_rule(rule_with_prod(10, vec![term(1), term(2), nonterm(10)], 0));
    g.add_rule(rule_with_prod(10, vec![term(1)], 1));
    let total_rhs: usize = g.all_rules().map(|r| r.rhs.len()).sum();
    assert_eq!(total_rhs, 4);
}

#[test]
fn grammar_size_builder_medium() {
    let g = GrammarBuilder::new("medium")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    assert_eq!(g.tokens.len(), 5);
    assert_eq!(g.rules.len(), 3); // expr, term, factor
    assert_eq!(g.all_rules().count(), 6);
}

#[test]
fn grammar_size_rule_names_and_nonterminal_count() {
    let g = GrammarBuilder::new("names")
        .token("NUM", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["NUM"])
        .rule("stmt", vec!["expr"])
        .rule("other", vec!["A", "B"])
        .start("expr")
        .build();
    // GrammarBuilder registers nonterminal names
    assert!(g.rule_names.values().any(|n| n == "expr"));
    assert!(g.rule_names.values().any(|n| n == "stmt"));
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn grammar_size_max_rhs_length() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "t", "t");
    g.add_rule(rule_with_prod(10, vec![term(1)], 0));
    g.add_rule(rule_with_prod(10, vec![term(1), term(1), term(1), term(1), term(1)], 1));
    g.add_rule(rule_with_prod(20, vec![term(1), term(1)], 2));
    let max_len = g.all_rules().map(|r| r.rhs.len()).max().unwrap_or(0);
    assert_eq!(max_len, 5);
}

#[test]
fn grammar_size_complex_rule_fraction() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "t", "t");
    // 2 simple rules
    g.add_rule(rule_with_prod(10, vec![term(1)], 0));
    g.add_rule(rule_with_prod(10, vec![term(1), term(1)], 1));
    // 1 complex rule
    g.add_rule(Rule {
        lhs: sym(20),
        rhs: vec![Symbol::Optional(Box::new(term(1)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    let total = g.all_rules().count();
    let complex = g.all_rules().filter(|r| is_complex_rule(r)).count();
    assert_eq!(total, 3);
    assert_eq!(complex, 1);
}

#[test]
fn rule_max_depth_across_rhs() {
    let r = Rule {
        lhs: sym(10),
        rhs: vec![
            term(1),
            Symbol::Optional(Box::new(Symbol::Repeat(Box::new(term(2))))),
            nonterm(3),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(rule_max_depth(&r), 2);
}

#[test]
fn rule_symbol_count_flat_and_nested() {
    let flat = simple_rule(10, vec![term(1), term(2), nonterm(3)]);
    assert_eq!(rule_symbol_count(&flat), 3);

    let nested = Rule {
        lhs: sym(10),
        rhs: vec![
            Symbol::Choice(vec![term(1), term(2), term(3)]),
            Symbol::Optional(Box::new(nonterm(20))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    // Choice(3 terminals) = 1 + 3 = 4,  Optional(1 nonterm) = 1 + 1 = 2
    assert_eq!(rule_symbol_count(&nested), 6);
}

#[test]
fn optimizer_stats_on_simple_grammar() {
    let mut g = Grammar::new("test".into());
    add_token(&mut g, 1, "a", "a");
    g.rule_names.insert(sym(10), "start".into());
    g.add_rule(rule_with_prod(10, vec![term(1)], 0));

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut g);
    // Simple grammar — no major optimizations expected
    assert_eq!(stats.optimized_left_recursion, 0);
    assert_eq!(stats.merged_tokens, 0);
}


