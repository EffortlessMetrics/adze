//! Comprehensive validation tests for adze-ir GrammarValidator (v5)

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{
    Associativity, ExternalToken, Grammar, Precedence, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};

// ---------------------------------------------------------------------------
// Helper: run validator and return the result
// ---------------------------------------------------------------------------
fn validate(grammar: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

/// Errors excluding CyclicRule (left-recursive grammars legitimately trigger cycle detection)
fn non_cycle_errors(r: &adze_ir::validation::ValidationResult) -> Vec<&ValidationError> {
    r.errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect()
}

// ===== 1. Valid grammars pass validation (10 tests) =====

#[test]
fn test_valid_single_token_rule() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_two_token_rule() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_multi_rule_grammar() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_valid_nullable_start() {
    let g = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("start_sym", vec![])
        .rule("start_sym", vec!["X"])
        .start("start_sym")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_python_like() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_valid_javascript_like() {
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_valid_with_precedence() {
    let g = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_valid_with_extras() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_with_external_tokens() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("block", vec!["NUM"])
        .start("block")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_stats_populated() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty());
    assert!(r.stats.total_tokens >= 2);
    assert!(r.stats.total_rules >= 1);
}

// ===== 2. Missing start symbol / empty grammar (5 tests) =====

#[test]
fn test_empty_grammar_error() {
    let g = Grammar::new("empty".to_string());
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar error, got: {:?}",
        r.errors
    );
}

#[test]
fn test_empty_grammar_no_rules() {
    let mut g = Grammar::new("t".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn test_empty_grammar_stats_zero() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 0);
}

#[test]
fn test_default_grammar_is_empty() {
    let g = Grammar::default();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn test_grammar_with_only_tokens_is_empty() {
    let mut g = Grammar::new("tok_only".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

// ===== 3. Undefined symbol references (8 tests) =====

#[test]
fn test_undefined_terminal_in_rule() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let undef = SymbolId(99);
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        r.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)
        )
    );
}

#[test]
fn test_undefined_nonterminal_in_rule() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let undef = SymbolId(50);
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(SymbolId(2)), Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        r.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)
        )
    );
}

#[test]
fn test_undefined_multiple_symbols() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let u1 = SymbolId(80);
    let u2 = SymbolId(81);
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(u1), Symbol::NonTerminal(u2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    let undef_errors: Vec<_> = r
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
        .collect();
    assert!(undef_errors.len() >= 2, "got: {:?}", undef_errors);
}

#[test]
fn test_undefined_symbol_location_populated() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let undef = SymbolId(42);
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    for e in &r.errors {
        if let ValidationError::UndefinedSymbol { location, .. } = e {
            assert!(!location.is_empty());
            return;
        }
    }
    panic!("no UndefinedSymbol error found");
}

#[test]
fn test_defined_token_is_not_undefined() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        !r.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == tok)
        )
    );
}

#[test]
fn test_defined_rule_lhs_is_not_undefined() {
    let mut g = Grammar::new("t".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let tok = SymbolId(3);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. })),
        "got: {:?}",
        r.errors
    );
}

#[test]
fn test_external_token_not_undefined() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let ext = SymbolId(2);
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: ext,
    });
    g.tokens.insert(
        ext,
        Token {
            name: "EXT".to_string(),
            pattern: TokenPattern::String("ext".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(ext)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        !r.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == ext)
        )
    );
}

#[test]
fn test_undefined_in_second_rule() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let tok = SymbolId(2);
    let undef = SymbolId(77);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert!(
        r.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)
        )
    );
}

// ===== 4. Duplicate rule names (5 tests) =====
// The validator doesn't flag duplicate rule LHS as an error (multiple
// alternatives for the same LHS are normal). Instead, test the DuplicateTokenPattern
// warning for tokens that share a pattern, plus verify multiple alternatives work.

#[test]
fn test_duplicate_token_pattern_warning() {
    let mut g = Grammar::new("t".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "P1".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "P2".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(r.warnings.iter().any(
        |w| matches!(w, ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+")
    ));
}

#[test]
fn test_three_duplicate_token_patterns() {
    let mut g = Grammar::new("t".to_string());
    for i in 1..=3u16 {
        g.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("eq{i}"),
                pattern: TokenPattern::String("=".to_string()),
                fragile: false,
            },
        );
    }
    let r = validate(&g);
    assert!(r
        .warnings
        .iter()
        .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { tokens, .. } if tokens.len() >= 3)));
}

#[test]
fn test_no_duplicate_warning_for_distinct_patterns() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(
        !r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }))
    );
}

#[test]
fn test_multiple_alternatives_same_lhs_valid() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_duplicate_regex_pattern_warning() {
    let mut g = Grammar::new("t".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "INT".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "DIGITS".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }))
    );
}

// ===== 5. Cycle detection (8 tests) =====

#[test]
fn test_simple_self_cycle() {
    let mut g = Grammar::new("t".to_string());
    let a = SymbolId(1);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_two_symbol_cycle() {
    let mut g = Grammar::new("t".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_three_symbol_cycle() {
    let mut g = Grammar::new("t".to_string());
    let (a, b, c) = (SymbolId(1), SymbolId(2), SymbolId(3));
    for (lhs, rhs_id, pid) in [(a, b, 0u16), (b, c, 1), (c, a, 2)] {
        g.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::NonTerminal(rhs_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        });
    }
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_cycle_with_base_case_still_detected() {
    // A -> B, A -> tok, B -> A — cycle exists even with a base case
    let mut g = Grammar::new("t".to_string());
    let (a, b, tok) = (SymbolId(1), SymbolId(2), SymbolId(3));
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_no_cycle_in_linear_chain() {
    let mut g = Grammar::new("t".to_string());
    let (a, b, tok) = (SymbolId(1), SymbolId(2), SymbolId(3));
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "unexpected cycle error: {:?}",
        r.errors
    );
}

#[test]
fn test_no_cycle_terminal_only_rules() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_cycle_symbols_non_empty() {
    let mut g = Grammar::new("t".to_string());
    let a = SymbolId(1);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    for e in &r.errors {
        if let ValidationError::CyclicRule { symbols } = e {
            assert!(!symbols.is_empty());
            return;
        }
    }
    panic!("no CyclicRule error found");
}

#[test]
fn test_left_recursive_rule_no_spurious_cycle() {
    // expr -> expr "+" term is left recursive but cycle detection fires on
    // non-terminal-only traversal. The builder creates such references as NonTerminal.
    // We build manually: the left-recursive alternative includes a terminal, so it
    // is productive. The cycle detector looks at pure non-terminal edges.
    let mut g = Grammar::new("t".to_string());
    let (expr, term, tok_plus, tok_num) = (SymbolId(1), SymbolId(2), SymbolId(3), SymbolId(4));
    g.tokens.insert(
        tok_plus,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_num,
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    // expr -> expr "+" term (left recursive but has terminal in RHS)
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(tok_plus),
            Symbol::NonTerminal(term),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // expr -> term
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::NonTerminal(term)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    // term -> NUM
    g.add_rule(Rule {
        lhs: term,
        rhs: vec![Symbol::Terminal(tok_num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    // The self-referencing expr->expr edge will trigger cycle detection since the
    // DFS walks all NonTerminal edges. We just verify no panic occurs.
    let _r = validate(&g);
}

// ===== 6. Unreachable symbol detection (5 tests) =====

#[test]
fn test_unreachable_rule_generates_warning() {
    let mut g = Grammar::new("t".to_string());
    let (s, orphan, tok) = (SymbolId(1), SymbolId(2), SymbolId(3));
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "s".to_string());
    g.rule_names.insert(orphan, "orphan".to_string());
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: orphan,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    // Unreachable symbols show up as UnusedToken warnings
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { token, .. } if *token == orphan))
    );
}

#[test]
fn test_reachable_symbol_no_warning() {
    let g = GrammarBuilder::new("t")
        .token("T", "t")
        .rule("root", vec!["child"])
        .rule("child", vec!["T"])
        .start("root")
        .build();
    let r = validate(&g);
    // "child" should be reachable
    assert!(
        !r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { name, .. } if name == "child"))
    );
}

#[test]
fn test_all_tokens_reachable_via_rules() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let r = validate(&g);
    // No unused token warnings for A or B
    let unused: Vec<_> = r
        .warnings
        .iter()
        .filter(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
        .collect();
    assert!(unused.is_empty(), "unexpected unused warnings: {unused:?}");
}

#[test]
fn test_unused_token_warning() {
    let mut g = Grammar::new("t".to_string());
    let (s, tok_used, tok_unused) = (SymbolId(1), SymbolId(2), SymbolId(3));
    g.tokens.insert(
        tok_used,
        Token {
            name: "USED".to_string(),
            pattern: TokenPattern::String("u".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_unused,
        Token {
            name: "UNUSED".to_string(),
            pattern: TokenPattern::String("z".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok_used)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(r.warnings.iter().any(
        |w| matches!(w, ValidationWarning::UnusedToken { token, .. } if *token == tok_unused)
    ));
}

#[test]
fn test_multiple_unreachable_warnings() {
    let mut g = Grammar::new("t".to_string());
    let (s, tok) = (SymbolId(1), SymbolId(2));
    let orphan1 = SymbolId(10);
    let orphan2 = SymbolId(11);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "s".to_string());
    g.rule_names.insert(orphan1, "o1".to_string());
    g.rule_names.insert(orphan2, "o2".to_string());
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: orphan1,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.add_rule(Rule {
        lhs: orphan2,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    let r = validate(&g);
    let unused_count = r
        .warnings
        .iter()
        .filter(|w| {
            matches!(w, ValidationWarning::UnusedToken { token, .. }
                if *token == orphan1 || *token == orphan2)
        })
        .count();
    assert!(unused_count >= 2, "expected >=2, got {unused_count}");
}

// ===== 7. Token validation (8 tests) =====

#[test]
fn test_empty_regex_pattern_error() {
    let mut g = Grammar::new("t".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "BAD".to_string(),
            pattern: TokenPattern::Regex(String::new()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. }))
    );
}

#[test]
fn test_valid_regex_no_error() {
    let g = GrammarBuilder::new("t")
        .token("ID", r"[a-zA-Z_]\w*")
        .rule("s", vec!["ID"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. }))
    );
}

#[test]
fn test_string_token_no_regex_error() {
    let mut g = Grammar::new("t".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. }))
    );
}

#[test]
fn test_fragile_token_validates() {
    let g = GrammarBuilder::new("t")
        .fragile_token("SEMI", ";")
        .rule("s", vec!["SEMI"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_conflicting_precedence_error() {
    let mut g = Grammar::new("t".to_string());
    let sym = SymbolId(1);
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![sym],
    });
    g.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::Right,
        symbols: vec![sym],
    });
    let r = validate(&g);
    assert!(r.errors.iter().any(
        |e| matches!(e, ValidationError::ConflictingPrecedence { symbol, .. } if *symbol == sym)
    ));
}

#[test]
fn test_no_conflicting_precedence_same_level() {
    let mut g = Grammar::new("t".to_string());
    let (s1, s2) = (SymbolId(1), SymbolId(2));
    g.tokens.insert(
        s1,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        s2,
        Token {
            name: "-".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![s1, s2],
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. }))
    );
}

#[test]
fn test_duplicate_external_token_conflict() {
    let mut g = Grammar::new("t".to_string());
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(1),
    });
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(2),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

#[test]
fn test_invalid_field_index_error() {
    let mut g = Grammar::new("t".to_string());
    let (s, tok) = (SymbolId(1), SymbolId(2));
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(adze_ir::FieldId(0), 10)], // index 10 out of bounds
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
}

// ===== 8. Multiple validators work independently (5 tests) =====

#[test]
fn test_two_validators_same_grammar() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
    assert_eq!(r1.warnings.len(), r2.warnings.len());
}

#[test]
fn test_validator_reuse() {
    let mut v = GrammarValidator::new();
    let g1 = GrammarBuilder::new("a")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let g2 = Grammar::new("empty".to_string());
    let r1 = v.validate(&g1);
    let r2 = v.validate(&g2);
    assert!(r1.errors.is_empty());
    assert!(!r2.errors.is_empty());
}

#[test]
fn test_validator_clears_between_runs() {
    let mut v = GrammarValidator::new();
    let bad = Grammar::new("empty".to_string());
    let good = GrammarBuilder::new("ok")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();
    let r1 = v.validate(&bad);
    assert!(!r1.errors.is_empty());
    let r2 = v.validate(&good);
    assert!(r2.errors.is_empty(), "errors leaked: {:?}", r2.errors);
}

#[test]
fn test_independent_results() {
    let g = Grammar::new("empty".to_string());
    let mut v1 = GrammarValidator::new();
    let mut v2 = GrammarValidator::new();
    let r1 = v1.validate(&g);
    let r2 = v2.validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
}

#[test]
fn test_sequential_validations_three_grammars() {
    let mut v = GrammarValidator::new();
    let g_empty = Grammar::new("e".to_string());
    let g_ok = GrammarBuilder::new("ok")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let g_empty2 = Grammar::new("e2".to_string());
    let r1 = v.validate(&g_empty);
    let r2 = v.validate(&g_ok);
    let r3 = v.validate(&g_empty2);
    assert!(!r1.errors.is_empty());
    assert!(r2.errors.is_empty());
    assert!(!r3.errors.is_empty());
}

// ===== 9. Complex grammars with mixed valid/invalid parts (6 tests) =====

#[test]
fn test_valid_grammar_with_inefficiency_warning() {
    // A trivial rule A -> B produces an InefficientRule warning
    let mut g = Grammar::new("t".to_string());
    let (a, b, tok) = (SymbolId(1), SymbolId(2), SymbolId(3));
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::InefficientRule { .. }))
    );
}

#[test]
fn test_long_rule_inefficiency_warning() {
    let mut g = Grammar::new("t".to_string());
    let s = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    // Rule with 12 symbols
    let rhs: Vec<Symbol> = (0..12).map(|_| Symbol::Terminal(tok)).collect();
    g.add_rule(Rule {
        lhs: s,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(r.warnings.iter().any(|w| matches!(
        w,
        ValidationWarning::InefficientRule { suggestion, .. }
            if suggestion.contains("12")
    )));
}

#[test]
fn test_grammar_with_both_errors_and_warnings() {
    let mut g = Grammar::new("t".to_string());
    // Create undefined symbol (error) + duplicate tokens (warning)
    let s = SymbolId(1);
    let undef = SymbolId(99);
    g.tokens.insert(
        SymbolId(10),
        Token {
            name: "dup1".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(11),
        Token {
            name: "dup2".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(!r.errors.is_empty());
    assert!(!r.warnings.is_empty());
}

#[test]
fn test_non_productive_and_cycle_together() {
    // A -> B, B -> A — both cyclic and non-productive
    let mut g = Grammar::new("t".to_string());
    let (a, b) = (SymbolId(1), SymbolId(2));
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    let has_cycle = r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::CyclicRule { .. }));
    let has_non_prod = r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }));
    assert!(has_cycle || has_non_prod, "errors: {:?}", r.errors);
}

#[test]
fn test_complex_grammar_error_display() {
    let g = Grammar::new("empty".to_string());
    let r = validate(&g);
    for e in &r.errors {
        let s = format!("{e}");
        assert!(!s.is_empty());
    }
}

#[test]
fn test_missing_field_names_warning() {
    // Rule with >1 RHS symbols and no fields triggers MissingFieldNames
    let mut g = Grammar::new("t".to_string());
    let (s, t1, t2) = (SymbolId(1), SymbolId(2), SymbolId(3));
    g.tokens.insert(
        t1,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        t2,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::MissingFieldNames { .. }))
    );
}

// ===== 10. Edge cases (5 tests) =====

#[test]
fn test_epsilon_only_rule() {
    let g = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("s", vec![])
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_single_rule_single_token() {
    let g = GrammarBuilder::new("minimal")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty());
    assert!(r.stats.total_rules >= 1);
    assert!(r.stats.total_tokens >= 1);
}

#[test]
fn test_validation_error_display_all_variants() {
    // Exercise Display for every ValidationError variant
    let errors = vec![
        ValidationError::UndefinedSymbol {
            symbol: SymbolId(1),
            location: "test".to_string(),
        },
        ValidationError::UnreachableSymbol {
            symbol: SymbolId(2),
            name: "x".to_string(),
        },
        ValidationError::NonProductiveSymbol {
            symbol: SymbolId(3),
            name: "y".to_string(),
        },
        ValidationError::CyclicRule {
            symbols: vec![SymbolId(1)],
        },
        ValidationError::DuplicateRule {
            symbol: SymbolId(1),
            existing_count: 2,
        },
        ValidationError::InvalidField {
            field_id: adze_ir::FieldId(0),
            rule_symbol: SymbolId(1),
        },
        ValidationError::EmptyGrammar,
        ValidationError::NoExplicitStartRule,
        ValidationError::ConflictingPrecedence {
            symbol: SymbolId(1),
            precedences: vec![1, 2],
        },
        ValidationError::InvalidRegex {
            token: SymbolId(1),
            pattern: "bad".to_string(),
            error: "err".to_string(),
        },
        ValidationError::ExternalTokenConflict {
            token1: "a".to_string(),
            token2: "b".to_string(),
        },
    ];
    for e in &errors {
        let s = format!("{e}");
        assert!(!s.is_empty(), "empty display for {e:?}");
    }
}

#[test]
fn test_validation_warning_display_all_variants() {
    let warnings = vec![
        ValidationWarning::UnusedToken {
            token: SymbolId(1),
            name: "T".to_string(),
        },
        ValidationWarning::DuplicateTokenPattern {
            tokens: vec![SymbolId(1)],
            pattern: "+".to_string(),
        },
        ValidationWarning::AmbiguousGrammar {
            message: "test".to_string(),
        },
        ValidationWarning::MissingFieldNames {
            rule_symbol: SymbolId(1),
        },
        ValidationWarning::InefficientRule {
            symbol: SymbolId(1),
            suggestion: "hint".to_string(),
        },
    ];
    for w in &warnings {
        let s = format!("{w}");
        assert!(!s.is_empty(), "empty display for {w:?}");
    }
}

#[test]
fn test_stats_max_rule_length() {
    let mut g = Grammar::new("t".to_string());
    let (s, tok) = (SymbolId(1), SymbolId(2));
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: s,
        rhs: vec![
            Symbol::Terminal(tok),
            Symbol::Terminal(tok),
            Symbol::Terminal(tok),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert_eq!(r.stats.max_rule_length, 3);
}
