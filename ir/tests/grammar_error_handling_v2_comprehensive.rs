//! Comprehensive tests for GrammarError and error handling in the IR crate.
//!
//! Covers: GrammarError variants, Display/Debug, ValidationError/ValidationWarning,
//! error recovery via catch_unwind, error message content, multiple errors, counts,
//! and Result patterns.

use std::panic::{AssertUnwindSafe, catch_unwind};

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, GrammarError, IrError, Precedence,
    PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ═══════════════════════════════════════════════════════════════════════════════
// 1. GrammarError variant construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_error_parse_error_from_bad_json() {
    let result = Grammar::from_macro_output("not json");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, GrammarError::ParseError(_)));
}

#[test]
fn grammar_error_invalid_field_ordering() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("0".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // Insert fields in non-lexicographic order: "z" before "a"
    grammar.fields.insert(FieldId(0), "z_field".into());
    grammar.fields.insert(FieldId(1), "a_field".into());

    let result = grammar.validate();
    assert!(matches!(result, Err(GrammarError::InvalidFieldOrdering)));
}

#[test]
fn grammar_error_unresolved_symbol() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let missing = SymbolId(99);
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::NonTerminal(missing)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let result = grammar.validate();
    assert!(matches!(
        result,
        Err(GrammarError::UnresolvedSymbol(SymbolId(99)))
    ));
}

#[test]
fn grammar_error_unresolved_external_symbol() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let ext = SymbolId(50);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num), Symbol::External(ext)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let result = grammar.validate();
    assert!(matches!(
        result,
        Err(GrammarError::UnresolvedExternalSymbol(SymbolId(50)))
    ));
}

#[test]
fn grammar_error_conflict_error_creation() {
    let err = GrammarError::ConflictError("shift/reduce conflict".into());
    assert!(matches!(err, GrammarError::ConflictError(_)));
}

#[test]
fn grammar_error_invalid_precedence_creation() {
    let err = GrammarError::InvalidPrecedence("bad prec".into());
    assert!(matches!(err, GrammarError::InvalidPrecedence(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. GrammarError Debug and Display
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_error_debug_parse_error() {
    let err = Grammar::from_macro_output("{bad}").unwrap_err();
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("ParseError"));
}

#[test]
fn grammar_error_display_parse_error() {
    let err = Grammar::from_macro_output("{bad}").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Failed to parse grammar"));
}

#[test]
fn grammar_error_display_invalid_field_ordering() {
    let err = GrammarError::InvalidFieldOrdering;
    let msg = format!("{}", err);
    assert!(msg.contains("lexicographic"));
}

#[test]
fn grammar_error_display_unresolved_symbol() {
    let err = GrammarError::UnresolvedSymbol(SymbolId(42));
    let msg = format!("{}", err);
    assert!(msg.contains("Unresolved symbol"));
    assert!(msg.contains("42"));
}

#[test]
fn grammar_error_display_unresolved_external() {
    let err = GrammarError::UnresolvedExternalSymbol(SymbolId(7));
    let msg = format!("{}", err);
    assert!(msg.contains("external"));
    assert!(msg.contains("7"));
}

#[test]
fn grammar_error_display_conflict() {
    let err = GrammarError::ConflictError("ambiguity in state 5".into());
    let msg = format!("{}", err);
    assert!(msg.contains("ambiguity in state 5"));
}

#[test]
fn grammar_error_display_invalid_prec() {
    let err = GrammarError::InvalidPrecedence("negative level".into());
    let msg = format!("{}", err);
    assert!(msg.contains("negative level"));
}

#[test]
fn grammar_error_debug_all_variants() {
    let variants: Vec<GrammarError> = vec![
        Grammar::from_macro_output("").unwrap_err(),
        GrammarError::InvalidFieldOrdering,
        GrammarError::UnresolvedSymbol(SymbolId(0)),
        GrammarError::UnresolvedExternalSymbol(SymbolId(0)),
        GrammarError::ConflictError("x".into()),
        GrammarError::InvalidPrecedence("y".into()),
    ];
    for v in &variants {
        let dbg = format!("{:?}", v);
        assert!(!dbg.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Error creation patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_error_from_serde_json() {
    let json_err: serde_json::Error = serde_json::from_str::<Grammar>("!!!").unwrap_err();
    let ge: GrammarError = json_err.into();
    assert!(matches!(ge, GrammarError::ParseError(_)));
}

#[test]
fn grammar_validate_ok_for_valid_grammar() {
    let grammar = GrammarBuilder::new("ok")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert!(grammar.validate().is_ok());
}

#[test]
fn grammar_validate_fields_in_order_is_ok() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("0".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.fields.insert(FieldId(0), "alpha".into());
    grammar.fields.insert(FieldId(1), "beta".into());
    assert!(grammar.validate().is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. ValidationError patterns — all variants
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn validation_error_empty_grammar() {
    let grammar = Grammar::new("empty".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn validation_error_undefined_symbol() {
    let mut grammar = Grammar::new("test".into());
    let a = SymbolId(1);
    let undef = SymbolId(100);
    grammar.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)
        )
    );
}

#[test]
fn validation_error_non_productive_mutual_recursion() {
    let mut grammar = Grammar::new("test".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    grammar.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

#[test]
fn validation_error_cyclic_rule() {
    let mut grammar = Grammar::new("test".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let num = SymbolId(3);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    // a -> b
    grammar.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // b -> a (cycle, but has no terminal base case via this path)
    grammar.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn validation_error_invalid_field() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 999)], // index 999 out of bounds
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
}

#[test]
fn validation_error_conflicting_precedence() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![expr],
    });
    grammar.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![expr],
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. }))
    );
}

#[test]
fn validation_error_invalid_regex() {
    let mut grammar = Grammar::new("test".into());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "bad_re".into(),
            pattern: TokenPattern::Regex("".into()), // empty regex
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. }))
    );
}

#[test]
fn validation_error_external_token_conflict() {
    let mut grammar = Grammar::new("test".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(10),
    });
    grammar.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(11),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. ValidationWarning patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn validation_warning_unused_token() {
    let grammar = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .token("UNUSED", "xyz")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
    );
}

#[test]
fn validation_warning_duplicate_token_pattern() {
    let mut grammar = Grammar::new("test".into());
    let t1 = SymbolId(1);
    let t2 = SymbolId(2);
    let expr = SymbolId(3);
    grammar.tokens.insert(
        t1,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("same".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        t2,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("same".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }))
    );
}

#[test]
fn validation_warning_missing_field_names() {
    let grammar = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"]) // 2+ RHS symbols, no fields
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::MissingFieldNames { .. }))
    );
}

#[test]
fn validation_warning_inefficient_trivial_rule() {
    let grammar = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .rule("value", vec!["NUM"])
        .rule("expr", vec!["value"]) // trivial A -> B
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::InefficientRule { .. }))
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Error recovery (catch_unwind for panics)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn catch_unwind_empty_grammar_name() {
    // Grammar::new with empty string doesn't panic, just verify it's fine
    let result = catch_unwind(AssertUnwindSafe(|| Grammar::new("".into())));
    assert!(result.is_ok());
}

#[test]
fn catch_unwind_validate_on_default_grammar() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let g = Grammar::default();
        g.validate()
    }));
    assert!(result.is_ok());
}

#[test]
fn catch_unwind_from_macro_output_null() {
    let result = catch_unwind(AssertUnwindSafe(|| Grammar::from_macro_output("null")));
    // "null" is valid JSON but not a Grammar — should be Err, not panic
    assert!(result.is_ok());
    assert!(result.unwrap().is_err());
}

#[test]
fn catch_unwind_builder_build() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        GrammarBuilder::new("safe").build();
    }));
    assert!(result.is_ok());
}

#[test]
fn catch_unwind_normalize_empty() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut g = Grammar::new("empty".into());
        g.normalize();
    }));
    assert!(result.is_ok());
}

#[test]
fn catch_unwind_check_empty_terminals() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let g = Grammar::new("t".into());
        g.check_empty_terminals()
    }));
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Error message content
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn validation_error_display_empty_grammar() {
    let err = ValidationError::EmptyGrammar;
    let msg = format!("{}", err);
    assert!(msg.contains("no rules"));
}

#[test]
fn validation_error_display_undefined_symbol() {
    let err = ValidationError::UndefinedSymbol {
        symbol: SymbolId(42),
        location: "rule for expr".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Undefined"));
    assert!(msg.contains("rule for expr"));
}

#[test]
fn validation_error_display_unreachable() {
    let err = ValidationError::UnreachableSymbol {
        symbol: SymbolId(5),
        name: "orphan".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("orphan"));
    assert!(msg.contains("unreachable"));
}

#[test]
fn validation_error_display_non_productive() {
    let err = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(3),
        name: "deadend".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("deadend"));
    assert!(msg.contains("terminal"));
}

#[test]
fn validation_error_display_cyclic() {
    let err = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Cyclic"));
}

#[test]
fn validation_error_display_duplicate_rule() {
    let err = ValidationError::DuplicateRule {
        symbol: SymbolId(1),
        existing_count: 3,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("3"));
}

#[test]
fn validation_error_display_invalid_field() {
    let err = ValidationError::InvalidField {
        field_id: FieldId(9),
        rule_symbol: SymbolId(1),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid field"));
}

#[test]
fn validation_error_display_no_explicit_start() {
    let err = ValidationError::NoExplicitStartRule;
    let msg = format!("{}", err);
    assert!(msg.contains("start rule"));
}

#[test]
fn validation_error_display_conflicting_prec() {
    let err = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(1),
        precedences: vec![1, 5],
    };
    let msg = format!("{}", err);
    assert!(msg.contains("conflicting"));
}

#[test]
fn validation_error_display_invalid_regex() {
    let err = ValidationError::InvalidRegex {
        token: SymbolId(1),
        pattern: "[bad".into(),
        error: "unclosed bracket".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("[bad"));
    assert!(msg.contains("unclosed bracket"));
}

#[test]
fn validation_error_display_external_conflict() {
    let err = ValidationError::ExternalTokenConflict {
        token1: "indent".into(),
        token2: "indent".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("indent"));
    assert!(msg.contains("conflict"));
}

#[test]
fn validation_warning_display_unused() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".into(),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("FOO"));
    assert!(msg.contains("never used"));
}

#[test]
fn validation_warning_display_duplicate_pattern() {
    let w = ValidationWarning::DuplicateTokenPattern {
        tokens: vec![SymbolId(1), SymbolId(2)],
        pattern: "+".into(),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("+"));
}

#[test]
fn validation_warning_display_ambiguous() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "dangling else".into(),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("dangling else"));
}

#[test]
fn validation_warning_display_missing_fields() {
    let w = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(1),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("no field names"));
}

#[test]
fn validation_warning_display_inefficient() {
    let w = ValidationWarning::InefficientRule {
        symbol: SymbolId(1),
        suggestion: "inline it".into(),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("inline it"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Multiple errors in one validation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_undefined_symbols_detected() {
    let mut grammar = Grammar::new("multi".into());
    let a = SymbolId(1);
    let undef1 = SymbolId(90);
    let undef2 = SymbolId(91);
    grammar.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(undef1), Symbol::NonTerminal(undef2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    let undef_count = r
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
        .count();
    assert!(undef_count >= 2);
}

#[test]
fn empty_grammar_plus_other_errors() {
    // An empty grammar should still produce EmptyGrammar error via validator
    let grammar = Grammar::new("empty".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
    // Should have at least one error
    assert!(!r.errors.is_empty());
}

#[test]
fn mixed_errors_and_warnings() {
    let mut grammar = Grammar::new("mix".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    let unused = SymbolId(3);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        unused,
        Token {
            name: "unused_tok".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 999)], // invalid field
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    // Should have at least one error (invalid field)
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
    // Should have at least one warning (unused token)
    assert!(!r.warnings.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. Error counts and stats
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn validation_stats_on_empty_grammar() {
    let grammar = Grammar::new("stats".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert_eq!(r.stats.total_rules, 0);
    assert_eq!(r.stats.total_tokens, 0);
    assert_eq!(r.stats.total_symbols, 0);
}

#[test]
fn validation_stats_counts_tokens() {
    let grammar = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert_eq!(r.stats.total_tokens, 2);
}

#[test]
fn validation_stats_counts_rules() {
    let grammar = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert_eq!(r.stats.total_rules, 2);
}

#[test]
fn validation_stats_max_rule_length() {
    let grammar = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("expr", vec!["A", "B", "C"])
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert_eq!(r.stats.max_rule_length, 3);
}

#[test]
fn validation_stats_external_tokens() {
    let mut grammar = Grammar::new("ext".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    grammar.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::String("1".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(10),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    assert_eq!(r.stats.external_tokens, 1);
}

#[test]
fn validation_error_count_matches_vec_len() {
    let grammar = Grammar::new("empty".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    // errors vec length is the count
    assert_eq!(r.errors.len(), r.errors.iter().count());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. Result patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_validate_result_is_ok_type() {
    let grammar = GrammarBuilder::new("ok")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let result: Result<(), GrammarError> = grammar.validate();
    assert!(result.is_ok());
}

#[test]
fn grammar_validate_result_is_err_type() {
    let result = Grammar::from_macro_output("invalid json here");
    assert!(result.is_err());
}

#[test]
fn grammar_error_can_be_unwrapped() {
    let result = Grammar::from_macro_output("{}");
    match result {
        Ok(_) => {} // may or may not succeed depending on serde defaults
        Err(e) => {
            let _ = format!("{}", e); // just ensure Display works
        }
    }
}

#[test]
fn ir_error_invalid_symbol() {
    let err = IrError::InvalidSymbol("missing_sym".into());
    let msg = format!("{}", err);
    assert!(msg.contains("missing_sym"));
}

#[test]
fn ir_error_duplicate_rule() {
    let err = IrError::DuplicateRule("expr -> NUM".into());
    let msg = format!("{}", err);
    assert!(msg.contains("duplicate rule"));
}

#[test]
fn ir_error_internal() {
    let err = IrError::Internal("unexpected state".into());
    let msg = format!("{}", err);
    assert!(msg.contains("internal error"));
}

#[test]
fn ir_result_ok_pattern() {
    let r: adze_ir::IrResult<u32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[test]
fn ir_result_err_pattern() {
    let r: adze_ir::IrResult<u32> = Err(IrError::Internal("boom".into()));
    assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional: edge cases and trait coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_error_is_std_error() {
    fn check<T: std::error::Error>() {}
    check::<GrammarError>();
}

#[test]
fn validation_error_clone() {
    let err = ValidationError::EmptyGrammar;
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn validation_error_eq() {
    let a = ValidationError::EmptyGrammar;
    let b = ValidationError::EmptyGrammar;
    assert_eq!(a, b);
}

#[test]
fn validation_warning_clone() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "test".into(),
    };
    let cloned = w.clone();
    assert_eq!(w, cloned);
}

#[test]
fn validation_warning_eq() {
    let a = ValidationWarning::AmbiguousGrammar {
        message: "x".into(),
    };
    let b = ValidationWarning::AmbiguousGrammar {
        message: "x".into(),
    };
    assert_eq!(a, b);
}

#[test]
fn check_empty_terminals_ok_on_clean_grammar() {
    let grammar = GrammarBuilder::new("clean")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert!(grammar.check_empty_terminals().is_ok());
}

#[test]
fn check_empty_terminals_err_on_empty_string_pattern() {
    let mut grammar = Grammar::new("test".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "empty_tok".into(),
            pattern: TokenPattern::String("".into()),
            fragile: false,
        },
    );
    let result = grammar.check_empty_terminals();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("empty string"));
}

#[test]
fn check_empty_terminals_err_on_empty_regex_pattern() {
    let mut grammar = Grammar::new("test".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "empty_re".into(),
            pattern: TokenPattern::Regex("".into()),
            fragile: false,
        },
    );
    let result = grammar.check_empty_terminals();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("empty regex"));
}

#[test]
fn validator_can_be_reused() {
    let mut v = GrammarValidator::new();
    let g1 = Grammar::new("e1".into());
    let r1 = v.validate(&g1);
    assert!(!r1.errors.is_empty());

    let g2 = GrammarBuilder::new("ok")
        .token("N", r"\d+")
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let r2 = v.validate(&g2);
    // Second validation should be independent (errors cleared)
    assert!(
        r2.errors.is_empty()
            || !r2
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn validation_result_has_stats_field() {
    let grammar = Grammar::new("test".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);
    // stats should be accessible
    let _ = r.stats.total_symbols;
    let _ = r.stats.total_tokens;
    let _ = r.stats.total_rules;
    let _ = r.stats.reachable_symbols;
    let _ = r.stats.productive_symbols;
    let _ = r.stats.external_tokens;
    let _ = r.stats.max_rule_length;
    let _ = r.stats.avg_rule_length;
}

#[test]
fn grammar_error_display_is_nonempty_for_all_variants() {
    let variants: Vec<GrammarError> = vec![
        Grammar::from_macro_output("").unwrap_err(),
        GrammarError::InvalidFieldOrdering,
        GrammarError::UnresolvedSymbol(SymbolId(1)),
        GrammarError::UnresolvedExternalSymbol(SymbolId(2)),
        GrammarError::ConflictError("c".into()),
        GrammarError::InvalidPrecedence("p".into()),
    ];
    for v in &variants {
        assert!(!format!("{}", v).is_empty());
    }
}

#[test]
fn validation_error_debug_is_nonempty_for_all_variants() {
    let variants: Vec<ValidationError> = vec![
        ValidationError::EmptyGrammar,
        ValidationError::NoExplicitStartRule,
        ValidationError::UndefinedSymbol {
            symbol: SymbolId(1),
            location: "loc".into(),
        },
        ValidationError::UnreachableSymbol {
            symbol: SymbolId(2),
            name: "n".into(),
        },
        ValidationError::NonProductiveSymbol {
            symbol: SymbolId(3),
            name: "n".into(),
        },
        ValidationError::CyclicRule {
            symbols: vec![SymbolId(1)],
        },
        ValidationError::DuplicateRule {
            symbol: SymbolId(1),
            existing_count: 2,
        },
        ValidationError::InvalidField {
            field_id: FieldId(0),
            rule_symbol: SymbolId(1),
        },
        ValidationError::ConflictingPrecedence {
            symbol: SymbolId(1),
            precedences: vec![1],
        },
        ValidationError::InvalidRegex {
            token: SymbolId(1),
            pattern: "p".into(),
            error: "e".into(),
        },
        ValidationError::ExternalTokenConflict {
            token1: "a".into(),
            token2: "b".into(),
        },
    ];
    for v in &variants {
        assert!(!format!("{:?}", v).is_empty());
    }
}
