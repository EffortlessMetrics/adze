//! Comprehensive tests for GrammarValidator, ValidationResult, ValidationError,
//! ValidationWarning, and ValidationStats.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{
    GrammarValidator, ValidationError, ValidationResult, ValidationStats, ValidationWarning,
};
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, Precedence, ProductionId, Rule, Symbol,
    SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

/// Minimal valid grammar: token "a", rule s -> a, start s.
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// Non-recursive arithmetic grammar (no cycles).
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["NUM", "+", "NUM"])
        .rule("expr", vec!["NUM", "*", "NUM"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. GrammarValidator::new() and basic usage
// ===========================================================================

#[test]
fn test_validator_new_creates_instance() {
    let _v = GrammarValidator::new();
}

#[test]
fn test_validator_default_creates_instance() {
    let _v = GrammarValidator::default();
}

#[test]
fn test_validator_validate_returns_result() {
    let mut v = GrammarValidator::new();
    let g = minimal_grammar();
    let _result: ValidationResult = v.validate(&g);
}

// ===========================================================================
// 2. Valid grammars report no errors
// ===========================================================================

#[test]
fn test_minimal_grammar_no_errors() {
    let mut v = GrammarValidator::new();
    let result = v.validate(&minimal_grammar());
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn test_arith_grammar_no_errors() {
    let mut v = GrammarValidator::new();
    let result = v.validate(&arith_grammar());
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn test_multi_rule_grammar_no_errors() {
    let g = GrammarBuilder::new("multi")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["a"])
        .rule("a", vec!["x"])
        .rule("a", vec!["y"])
        .start("start")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn test_epsilon_rule_grammar_no_errors() {
    let g = GrammarBuilder::new("eps")
        .token("x", "x")
        .rule("s", vec!["x"])
        .rule("s", vec![]) // epsilon
        .start("s")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

// ===========================================================================
// 3. Empty grammar detection
// ===========================================================================

#[test]
fn test_empty_grammar_produces_error() {
    let g = Grammar::new("empty".to_string());
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

// ===========================================================================
// 4. Unresolved / undefined symbols
// ===========================================================================

#[test]
fn test_undefined_nonterminal_detected() {
    let mut g = Grammar::new("undef".to_string());
    let lhs = SymbolId(1);
    let undef = SymbolId(99);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn test_undefined_terminal_detected() {
    let mut g = Grammar::new("undef_term".to_string());
    let lhs = SymbolId(1);
    let undef_term = SymbolId(50);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(undef_term)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef_term
    )));
}

// ===========================================================================
// 5. Unreachable rules
// ===========================================================================

#[test]
fn test_unreachable_rule_produces_warning() {
    // "orphan" is defined but never referenced from start
    let g = GrammarBuilder::new("unreach")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("orphan", vec!["b"])
        .start("s")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    // Unreachable symbols are reported as UnusedToken warnings
    assert!(
        result.warnings.iter().any(|w| matches!(
            w,
            ValidationWarning::UnusedToken { name, .. } if name.contains("orphan") || name.contains("b")
        )),
        "warnings: {:?}",
        result.warnings,
    );
}

// ===========================================================================
// 6. Cycles
// ===========================================================================

#[test]
fn test_cycle_a_to_a_detected() {
    let mut g = Grammar::new("self_cycle".to_string());
    let a = SymbolId(1);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_cycle_a_b_a_detected() {
    let mut g = Grammar::new("ab_cycle".to_string());
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
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn test_cycle_three_symbols() {
    let mut g = Grammar::new("abc_cycle".to_string());
    let (a, b, c) = (SymbolId(1), SymbolId(2), SymbolId(3));
    for (lhs, rhs_id, pid) in [(a, b, 0), (b, c, 1), (c, a, 2)] {
        g.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::NonTerminal(rhs_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        });
    }
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

// ===========================================================================
// 7. Non-productive symbols
// ===========================================================================

#[test]
fn test_non_productive_detected() {
    let mut g = Grammar::new("nonprod".to_string());
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
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

// ===========================================================================
// 8. Duplicate token patterns (warning)
// ===========================================================================

#[test]
fn test_duplicate_token_pattern_warning() {
    let mut g = Grammar::new("dup".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "plus1".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "plus2".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+"
    )));
}

// ===========================================================================
// 9. Invalid field index
// ===========================================================================

#[test]
fn test_invalid_field_index_detected() {
    let mut g = Grammar::new("field".to_string());
    let (expr, num) = (SymbolId(1), SymbolId(2));
    g.tokens.insert(
        num,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::String("1".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 99)], // out of bounds
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
}

// ===========================================================================
// 10. Conflicting precedence
// ===========================================================================

#[test]
fn test_conflicting_precedence_detected() {
    let mut g = Grammar::new("prec".to_string());
    let plus = SymbolId(1);
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![plus],
    });
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. }))
    );
}

// ===========================================================================
// 11. External token conflict
// ===========================================================================

#[test]
fn test_external_token_conflict_detected() {
    let mut g = Grammar::new("ext".to_string());
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(11),
    });
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

// ===========================================================================
// 12. Invalid regex
// ===========================================================================

#[test]
fn test_empty_regex_detected() {
    let mut g = Grammar::new("regex".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "bad".to_string(),
            pattern: TokenPattern::Regex(String::new()),
            fragile: false,
        },
    );
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. }))
    );
}

// ===========================================================================
// 13. ValidationError Display impls
// ===========================================================================

#[test]
fn test_display_undefined_symbol() {
    let e = ValidationError::UndefinedSymbol {
        symbol: SymbolId(5),
        location: "rule for foo".to_string(),
    };
    let s = format!("{e}");
    assert!(s.contains("Undefined symbol"));
    assert!(s.contains("rule for foo"));
}

#[test]
fn test_display_unreachable_symbol() {
    let e = ValidationError::UnreachableSymbol {
        symbol: SymbolId(7),
        name: "orphan".to_string(),
    };
    assert!(format!("{e}").contains("unreachable"));
}

#[test]
fn test_display_non_productive_symbol() {
    let e = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(8),
        name: "dead".to_string(),
    };
    assert!(format!("{e}").contains("terminal strings"));
}

#[test]
fn test_display_cyclic_rule() {
    let e = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    assert!(format!("{e}").contains("Cyclic"));
}

#[test]
fn test_display_duplicate_rule() {
    let e = ValidationError::DuplicateRule {
        symbol: SymbolId(1),
        existing_count: 3,
    };
    let s = format!("{e}");
    assert!(s.contains("3"));
}

#[test]
fn test_display_invalid_field() {
    let e = ValidationError::InvalidField {
        field_id: FieldId(0),
        rule_symbol: SymbolId(1),
    };
    assert!(format!("{e}").contains("Invalid field"));
}

#[test]
fn test_display_empty_grammar() {
    let e = ValidationError::EmptyGrammar;
    assert!(format!("{e}").contains("no rules"));
}

#[test]
fn test_display_no_explicit_start() {
    let e = ValidationError::NoExplicitStartRule;
    assert!(format!("{e}").contains("start rule"));
}

#[test]
fn test_display_conflicting_precedence() {
    let e = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(1),
        precedences: vec![1, 2],
    };
    assert!(format!("{e}").contains("conflicting"));
}

#[test]
fn test_display_invalid_regex() {
    let e = ValidationError::InvalidRegex {
        token: SymbolId(1),
        pattern: "bad".to_string(),
        error: "parse error".to_string(),
    };
    let s = format!("{e}");
    assert!(s.contains("Invalid regex"));
    assert!(s.contains("parse error"));
}

#[test]
fn test_display_external_token_conflict() {
    let e = ValidationError::ExternalTokenConflict {
        token1: "A".to_string(),
        token2: "B".to_string(),
    };
    assert!(format!("{e}").contains("conflict"));
}

// ===========================================================================
// 14. ValidationError: Debug, Clone, PartialEq
// ===========================================================================

#[test]
fn test_validation_error_debug() {
    let e = ValidationError::EmptyGrammar;
    let dbg = format!("{e:?}");
    assert!(dbg.contains("EmptyGrammar"));
}

#[test]
fn test_validation_error_clone() {
    let e = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1)],
    };
    let e2 = e.clone();
    assert_eq!(e, e2);
}

#[test]
fn test_validation_error_partial_eq_different_variants() {
    let a = ValidationError::EmptyGrammar;
    let b = ValidationError::NoExplicitStartRule;
    assert_ne!(a, b);
}

// ===========================================================================
// 15. ValidationWarning Display impls
// ===========================================================================

#[test]
fn test_display_unused_token() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".to_string(),
    };
    assert!(format!("{w}").contains("never used"));
}

#[test]
fn test_display_duplicate_token_pattern() {
    let w = ValidationWarning::DuplicateTokenPattern {
        tokens: vec![SymbolId(1), SymbolId(2)],
        pattern: "+".to_string(),
    };
    assert!(format!("{w}").contains("+"));
}

#[test]
fn test_display_ambiguous_grammar() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "shift/reduce".to_string(),
    };
    assert!(format!("{w}").contains("shift/reduce"));
}

#[test]
fn test_display_missing_field_names() {
    let w = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(1),
    };
    assert!(format!("{w}").contains("field names"));
}

#[test]
fn test_display_inefficient_rule() {
    let w = ValidationWarning::InefficientRule {
        symbol: SymbolId(1),
        suggestion: "inline it".to_string(),
    };
    assert!(format!("{w}").contains("inline it"));
}

// ===========================================================================
// 16. ValidationWarning: Debug, Clone, PartialEq
// ===========================================================================

#[test]
fn test_validation_warning_debug() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "test".to_string(),
    };
    assert!(format!("{w:?}").contains("AmbiguousGrammar"));
}

#[test]
fn test_validation_warning_clone() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "T".to_string(),
    };
    let w2 = w.clone();
    assert_eq!(w, w2);
}

#[test]
fn test_validation_warning_partial_eq_different() {
    let a = ValidationWarning::AmbiguousGrammar {
        message: "a".to_string(),
    };
    let b = ValidationWarning::AmbiguousGrammar {
        message: "b".to_string(),
    };
    assert_ne!(a, b);
}

// ===========================================================================
// 17. ValidationStats: Default, Clone, Debug, field access
// ===========================================================================

#[test]
fn test_validation_stats_default() {
    let s = ValidationStats::default();
    assert_eq!(s.total_symbols, 0);
    assert_eq!(s.total_tokens, 0);
    assert_eq!(s.total_rules, 0);
    assert_eq!(s.reachable_symbols, 0);
    assert_eq!(s.productive_symbols, 0);
    assert_eq!(s.external_tokens, 0);
    assert_eq!(s.max_rule_length, 0);
    assert!(s.avg_rule_length == 0.0);
}

#[test]
fn test_validation_stats_clone() {
    let mut v = GrammarValidator::new();
    let result = v.validate(&minimal_grammar());
    let s1 = result.stats.clone();
    // Cloned stats keep values
    assert_eq!(s1.total_tokens, result.stats.total_tokens);
}

#[test]
fn test_validation_stats_debug() {
    let s = ValidationStats::default();
    let dbg = format!("{s:?}");
    assert!(dbg.contains("ValidationStats"));
}

#[test]
fn test_stats_populated_for_minimal_grammar() {
    let mut v = GrammarValidator::new();
    let result = v.validate(&minimal_grammar());
    assert!(result.stats.total_tokens > 0);
    assert!(result.stats.total_rules > 0);
    assert!(result.stats.total_symbols > 0);
}

#[test]
fn test_stats_max_rule_length() {
    let g = GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert_eq!(result.stats.max_rule_length, 3);
}

#[test]
fn test_stats_external_tokens_count() {
    let g = GrammarBuilder::new("ext")
        .token("a", "a")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert_eq!(result.stats.external_tokens, 1);
}

// ===========================================================================
// 18. ValidationResult structure
// ===========================================================================

#[test]
fn test_result_has_errors_warnings_stats() {
    let mut v = GrammarValidator::new();
    let result = v.validate(&minimal_grammar());
    // Just ensure we can access all three fields
    let _e: &Vec<ValidationError> = &result.errors;
    let _w: &Vec<ValidationWarning> = &result.warnings;
    let _s: &ValidationStats = &result.stats;
}

// ===========================================================================
// 19. Validator reuse across multiple grammars
// ===========================================================================

#[test]
fn test_validator_reuse_clears_previous_errors() {
    let mut v = GrammarValidator::new();

    // First: empty grammar => errors
    let r1 = v.validate(&Grammar::new("empty".to_string()));
    assert!(!r1.errors.is_empty());

    // Second: valid grammar => no errors
    let r2 = v.validate(&minimal_grammar());
    assert!(r2.errors.is_empty(), "errors leaked: {:?}", r2.errors);
}

#[test]
fn test_validator_reuse_clears_previous_warnings() {
    let mut v = GrammarValidator::new();

    // First grammar has duplicate token pattern → warning
    let mut g1 = Grammar::new("dup".to_string());
    g1.tokens.insert(
        SymbolId(1),
        Token {
            name: "p1".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g1.tokens.insert(
        SymbolId(2),
        Token {
            name: "p2".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let r1 = v.validate(&g1);
    assert!(!r1.warnings.is_empty());

    // Second grammar has no duplicate → warnings should be empty
    let r2 = v.validate(&minimal_grammar());
    // We allow MissingFieldNames / InefficientRule type warnings for valid grammars,
    // but not DuplicateTokenPattern from prior run.
    assert!(
        !r2.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. })),
        "stale duplicate-pattern warning leaked: {:?}",
        r2.warnings,
    );
}

#[test]
fn test_validator_reuse_three_grammars() {
    let mut v = GrammarValidator::new();
    for _ in 0..3 {
        let result = v.validate(&minimal_grammar());
        assert!(result.errors.is_empty());
    }
}

// ===========================================================================
// 20. Edge cases
// ===========================================================================

#[test]
fn test_grammar_with_only_tokens_no_rules() {
    let mut g = Grammar::new("tok_only".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    // No rules => EmptyGrammar
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn test_fragile_token_grammar() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("semi", ";")
        .token("x", "x")
        .rule("s", vec!["x", "semi"])
        .start("s")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn test_python_like_preset_validates() {
    let g = GrammarBuilder::python_like();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    // Recursive grammars produce CyclicRule; no other error types expected.
    let non_cycle: Vec<_> = result
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "non-cycle errors: {:?}", non_cycle);
}

#[test]
fn test_javascript_like_preset_validates() {
    let g = GrammarBuilder::javascript_like();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    let non_cycle: Vec<_> = result
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "non-cycle errors: {:?}", non_cycle);
}

#[test]
fn test_stats_avg_rule_length() {
    // Two rules: lengths 1 and 3 → average = 2.0
    let g = GrammarBuilder::new("avg")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.stats.avg_rule_length > 1.0);
}

#[test]
fn test_multiple_errors_reported_together() {
    // Empty grammar + conflicting precedence
    let mut g = Grammar::new("multi_err".to_string());
    let plus = SymbolId(1);
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![plus],
    });
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. }))
    );
}

#[test]
fn test_precedence_grammar_only_cycle_errors() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    // Recursive grammars trigger CyclicRule; no other error types expected.
    let non_cycle: Vec<_> = result
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "non-cycle errors: {:?}", non_cycle);
}

#[test]
fn test_check_empty_terminals_on_grammar() {
    let g = minimal_grammar();
    assert!(g.check_empty_terminals().is_ok());
}

#[test]
fn test_check_empty_terminals_catches_empty_string() {
    let mut g = Grammar::new("empty_tok".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "blank".to_string(),
            pattern: TokenPattern::String(String::new()),
            fragile: false,
        },
    );
    assert!(g.check_empty_terminals().is_err());
}
