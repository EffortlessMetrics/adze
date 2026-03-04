//! Comprehensive tests for GrammarError, IrError, and validation error handling in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{
    GrammarValidator, ValidationError, ValidationResult, ValidationStats, ValidationWarning,
};
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, GrammarError, IrError, Precedence,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helper: build a minimal valid grammar
// ---------------------------------------------------------------------------
fn minimal_valid_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ===========================================================================
// GrammarError – construction
// ===========================================================================

#[test]
fn grammar_error_parse_error_from_serde() {
    let bad_json = "{ not valid json }}}";
    let serde_err = serde_json::from_str::<Grammar>(bad_json).unwrap_err();
    let ge = GrammarError::ParseError(serde_err);
    assert!(format!("{ge}").contains("Failed to parse grammar"));
}

#[test]
fn grammar_error_parse_error_via_from_macro_output() {
    let result = Grammar::from_macro_output("not json");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, GrammarError::ParseError(_)));
}

#[test]
fn grammar_error_invalid_field_ordering() {
    let err = GrammarError::InvalidFieldOrdering;
    assert!(format!("{err}").contains("lexicographic order"));
}

#[test]
fn grammar_error_unresolved_symbol() {
    let err = GrammarError::UnresolvedSymbol(SymbolId(42));
    let msg = format!("{err}");
    assert!(msg.contains("Unresolved symbol reference"));
    assert!(msg.contains("42"));
}

#[test]
fn grammar_error_unresolved_external_symbol() {
    let err = GrammarError::UnresolvedExternalSymbol(SymbolId(7));
    let msg = format!("{err}");
    assert!(msg.contains("Unresolved external symbol reference"));
    assert!(msg.contains("7"));
}

#[test]
fn grammar_error_conflict_error() {
    let err = GrammarError::ConflictError("shift/reduce".into());
    assert!(format!("{err}").contains("shift/reduce"));
}

#[test]
fn grammar_error_invalid_precedence() {
    let err = GrammarError::InvalidPrecedence("negative level".into());
    assert!(format!("{err}").contains("negative level"));
}

// ===========================================================================
// GrammarError – Debug trait
// ===========================================================================

#[test]
fn grammar_error_debug_parse_error() {
    let serde_err = serde_json::from_str::<Grammar>("!!!").unwrap_err();
    let err = GrammarError::ParseError(serde_err);
    let dbg = format!("{err:?}");
    assert!(dbg.contains("ParseError"));
}

#[test]
fn grammar_error_debug_invalid_field_ordering() {
    let dbg = format!("{:?}", GrammarError::InvalidFieldOrdering);
    assert_eq!(dbg, "InvalidFieldOrdering");
}

#[test]
fn grammar_error_debug_unresolved_symbol() {
    let dbg = format!("{:?}", GrammarError::UnresolvedSymbol(SymbolId(1)));
    assert!(dbg.contains("UnresolvedSymbol"));
}

#[test]
fn grammar_error_debug_conflict_error() {
    let dbg = format!("{:?}", GrammarError::ConflictError("x".into()));
    assert!(dbg.contains("ConflictError"));
}

#[test]
fn grammar_error_debug_invalid_precedence() {
    let dbg = format!("{:?}", GrammarError::InvalidPrecedence("y".into()));
    assert!(dbg.contains("InvalidPrecedence"));
}

#[test]
fn grammar_error_debug_unresolved_external() {
    let dbg = format!("{:?}", GrammarError::UnresolvedExternalSymbol(SymbolId(99)));
    assert!(dbg.contains("UnresolvedExternalSymbol"));
}

// ===========================================================================
// GrammarError – std::error::Error trait (source chain)
// ===========================================================================

#[test]
fn grammar_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(GrammarError::ConflictError("test".into()));
    // Ensure it can be used as a trait object
    assert!(!err.to_string().is_empty());
}

#[test]
fn grammar_error_parse_error_has_source() {
    use std::error::Error;
    let serde_err = serde_json::from_str::<Grammar>("bad").unwrap_err();
    let ge = GrammarError::ParseError(serde_err);
    // thiserror #[from] populates source()
    assert!(ge.source().is_some());
}

#[test]
fn grammar_error_non_parse_variants_have_no_source() {
    use std::error::Error;
    assert!(GrammarError::InvalidFieldOrdering.source().is_none());
    assert!(
        GrammarError::UnresolvedSymbol(SymbolId(0))
            .source()
            .is_none()
    );
    assert!(GrammarError::ConflictError("x".into()).source().is_none());
    assert!(
        GrammarError::InvalidPrecedence("x".into())
            .source()
            .is_none()
    );
    assert!(
        GrammarError::UnresolvedExternalSymbol(SymbolId(0))
            .source()
            .is_none()
    );
}

// ===========================================================================
// GrammarError – triggered by Grammar::validate()
// ===========================================================================

#[test]
fn grammar_validate_ok_for_valid_grammar() {
    let g = minimal_valid_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_validate_unresolved_symbol() {
    let mut g = Grammar::new("test".into());
    let expr = SymbolId(1);
    let missing = SymbolId(99);
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::NonTerminal(missing)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let err = g.validate().unwrap_err();
    assert!(matches!(err, GrammarError::UnresolvedSymbol(id) if id == missing));
}

#[test]
fn grammar_validate_unresolved_external_symbol() {
    let mut g = Grammar::new("test".into());
    let expr = SymbolId(1);
    let ext = SymbolId(50);
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::External(ext)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let err = g.validate().unwrap_err();
    assert!(matches!(err, GrammarError::UnresolvedExternalSymbol(id) if id == ext));
}

#[test]
fn grammar_validate_invalid_field_ordering() {
    let mut g = minimal_valid_grammar();
    // Insert fields out of lexicographic order
    g.fields.insert(FieldId(0), "zebra".into());
    g.fields.insert(FieldId(1), "alpha".into());
    let err = g.validate().unwrap_err();
    assert!(matches!(err, GrammarError::InvalidFieldOrdering));
}

#[test]
fn grammar_validate_fields_in_order_is_ok() {
    let mut g = minimal_valid_grammar();
    g.fields.insert(FieldId(0), "alpha".into());
    g.fields.insert(FieldId(1), "beta".into());
    assert!(g.validate().is_ok());
}

// ===========================================================================
// GrammarError – from serde_json::Error conversion
// ===========================================================================

#[test]
fn grammar_error_from_serde_error_conversion() {
    let serde_err = serde_json::from_str::<Grammar>("{").unwrap_err();
    let ge: GrammarError = serde_err.into();
    assert!(matches!(ge, GrammarError::ParseError(_)));
}

// ===========================================================================
// IrError – construction and Display
// ===========================================================================

#[test]
fn ir_error_invalid_symbol_display() {
    let err = IrError::InvalidSymbol("foo".into());
    assert_eq!(format!("{err}"), "invalid symbol: foo");
}

#[test]
fn ir_error_duplicate_rule_display() {
    let err = IrError::DuplicateRule("bar".into());
    assert_eq!(format!("{err}"), "duplicate rule: bar");
}

#[test]
fn ir_error_internal_display() {
    let err = IrError::Internal("oops".into());
    assert_eq!(format!("{err}"), "internal error: oops");
}

#[test]
fn ir_error_debug() {
    let err = IrError::InvalidSymbol("x".into());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("InvalidSymbol"));
}

#[test]
fn ir_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(IrError::Internal("hi".into()));
    assert!(!err.to_string().is_empty());
}

// ===========================================================================
// ValidationError – construction and Display
// ===========================================================================

#[test]
fn validation_error_undefined_symbol_display() {
    let e = ValidationError::UndefinedSymbol {
        symbol: SymbolId(5),
        location: "rule for expr".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("Undefined symbol"));
    assert!(s.contains("rule for expr"));
}

#[test]
fn validation_error_unreachable_symbol_display() {
    let e = ValidationError::UnreachableSymbol {
        symbol: SymbolId(10),
        name: "unused_nt".into(),
    };
    assert!(format!("{e}").contains("unreachable from start"));
}

#[test]
fn validation_error_non_productive_symbol_display() {
    let e = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(3),
        name: "bad_rule".into(),
    };
    assert!(format!("{e}").contains("cannot derive any terminal"));
}

#[test]
fn validation_error_cyclic_rule_display() {
    let e = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    assert!(format!("{e}").contains("Cyclic dependency"));
}

#[test]
fn validation_error_duplicate_rule_display() {
    let e = ValidationError::DuplicateRule {
        symbol: SymbolId(4),
        existing_count: 3,
    };
    let s = format!("{e}");
    assert!(s.contains("3 rule definitions"));
}

#[test]
fn validation_error_invalid_field_display() {
    let e = ValidationError::InvalidField {
        field_id: FieldId(7),
        rule_symbol: SymbolId(2),
    };
    assert!(format!("{e}").contains("Invalid field"));
}

#[test]
fn validation_error_empty_grammar_display() {
    let e = ValidationError::EmptyGrammar;
    assert!(format!("{e}").contains("no rules"));
}

#[test]
fn validation_error_no_explicit_start_rule_display() {
    let e = ValidationError::NoExplicitStartRule;
    assert!(format!("{e}").contains("No explicit start rule"));
}

#[test]
fn validation_error_conflicting_precedence_display() {
    let e = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(1),
        precedences: vec![1, 2],
    };
    assert!(format!("{e}").contains("conflicting precedences"));
}

#[test]
fn validation_error_invalid_regex_display() {
    let e = ValidationError::InvalidRegex {
        token: SymbolId(3),
        pattern: "[bad".into(),
        error: "unclosed bracket".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("[bad"));
    assert!(s.contains("unclosed bracket"));
}

#[test]
fn validation_error_external_token_conflict_display() {
    let e = ValidationError::ExternalTokenConflict {
        token1: "indent".into(),
        token2: "indent".into(),
    };
    assert!(format!("{e}").contains("conflict"));
}

// ===========================================================================
// ValidationError – Clone + PartialEq + Eq
// ===========================================================================

#[test]
fn validation_error_clone_and_eq() {
    let e = ValidationError::EmptyGrammar;
    let cloned = e.clone();
    assert_eq!(e, cloned);
}

#[test]
fn validation_error_ne_different_variants() {
    let a = ValidationError::EmptyGrammar;
    let b = ValidationError::NoExplicitStartRule;
    assert_ne!(a, b);
}

// ===========================================================================
// ValidationWarning – construction and Display
// ===========================================================================

#[test]
fn validation_warning_unused_token_display() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(9),
        name: "SEMICOLON".into(),
    };
    assert!(format!("{w}").contains("never used"));
}

#[test]
fn validation_warning_duplicate_token_pattern_display() {
    let w = ValidationWarning::DuplicateTokenPattern {
        tokens: vec![SymbolId(1), SymbolId(2)],
        pattern: "+".into(),
    };
    assert!(format!("{w}").contains("same pattern"));
}

#[test]
fn validation_warning_ambiguous_grammar_display() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "dangling else".into(),
    };
    assert!(format!("{w}").contains("dangling else"));
}

#[test]
fn validation_warning_missing_field_names_display() {
    let w = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(4),
    };
    assert!(format!("{w}").contains("no field names"));
}

#[test]
fn validation_warning_inefficient_rule_display() {
    let w = ValidationWarning::InefficientRule {
        symbol: SymbolId(2),
        suggestion: "inline it".into(),
    };
    assert!(format!("{w}").contains("inline it"));
}

// ===========================================================================
// ValidationWarning – Clone + PartialEq + Eq
// ===========================================================================

#[test]
fn validation_warning_clone_and_eq() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "test".into(),
    };
    assert_eq!(w, w.clone());
}

#[test]
fn validation_warning_ne_different_variants() {
    let a = ValidationWarning::AmbiguousGrammar {
        message: "x".into(),
    };
    let b = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(1),
    };
    assert_ne!(a, b);
}

// ===========================================================================
// ValidationStats – Default and fields
// ===========================================================================

#[test]
fn validation_stats_default_is_zero() {
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
fn validation_stats_clone() {
    let mut s = ValidationStats::default();
    s.total_symbols = 42;
    let c = s.clone();
    assert_eq!(c.total_symbols, 42);
}

#[test]
fn validation_stats_debug() {
    let s = ValidationStats::default();
    let dbg = format!("{s:?}");
    assert!(dbg.contains("ValidationStats"));
}

// ===========================================================================
// GrammarValidator – basic API
// ===========================================================================

#[test]
fn validator_new_creates_instance() {
    let v = GrammarValidator::new();
    // Verify it can validate an empty grammar
    let g = Grammar::new("empty".into());
    let mut v = v;
    let result = v.validate(&g);
    assert!(!result.errors.is_empty()); // EmptyGrammar error
}

#[test]
fn validator_default_same_as_new() {
    let _ = GrammarValidator::default();
}

#[test]
fn validator_empty_grammar_reports_error() {
    let mut v = GrammarValidator::new();
    let result = v.validate(&Grammar::new("empty".into()));
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn validator_valid_grammar_no_errors() {
    let g = minimal_valid_grammar();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.errors.is_empty());
}

#[test]
fn validator_populates_stats_for_valid_grammar() {
    let g = minimal_valid_grammar();
    let mut v = GrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.stats.total_tokens >= 1);
    assert!(result.stats.total_rules >= 1);
}

#[test]
fn validator_can_be_reused() {
    let mut v = GrammarValidator::new();
    let g1 = Grammar::new("empty".into());
    let r1 = v.validate(&g1);
    assert!(!r1.errors.is_empty());
    // Second validation on a valid grammar should start fresh
    let g2 = minimal_valid_grammar();
    let r2 = v.validate(&g2);
    assert!(r2.errors.is_empty());
}

// ===========================================================================
// Validator – detects specific issues
// ===========================================================================

#[test]
fn validator_detects_undefined_symbol() {
    let mut g = Grammar::new("test".into());
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(999))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
    );
}

#[test]
fn validator_detects_non_productive_symbols() {
    let mut g = Grammar::new("test".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    // A -> B, B -> A  (circular, no terminals)
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
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

#[test]
fn validator_detects_cyclic_rules() {
    let mut g = Grammar::new("test".into());
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
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn validator_detects_invalid_field_index() {
    let mut g = Grammar::new("test".into());
    let num = SymbolId(2);
    g.tokens.insert(
        num,
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 100)], // out-of-bounds
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
}

#[test]
fn validator_detects_duplicate_token_patterns() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }))
    );
}

#[test]
fn validator_detects_conflicting_precedence() {
    let mut g = Grammar::new("test".into());
    let sym = SymbolId(1);
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![sym],
    });
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![sym],
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. }))
    );
}

#[test]
fn validator_detects_external_token_conflict() {
    let mut g = Grammar::new("test".into());
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(11),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

#[test]
fn validator_warns_missing_field_names() {
    let mut g = Grammar::new("test".into());
    let num = SymbolId(2);
    let plus = SymbolId(3);
    g.tokens.insert(
        num,
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(num),
            Symbol::Terminal(plus),
            Symbol::Terminal(num),
        ],
        precedence: None,
        associativity: None,
        fields: vec![], // no fields on a multi-symbol rule
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::MissingFieldNames { .. }))
    );
}

#[test]
fn validator_warns_inefficient_trivial_rule() {
    // A trivial rule: expr -> term (single non-terminal)
    let g = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("term", vec!["NUMBER"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.warnings.iter().any(
        |w| matches!(w, ValidationWarning::InefficientRule { suggestion, .. } if suggestion.contains("inlining"))
    ));
}

// ===========================================================================
// ValidationResult – structure checks
// ===========================================================================

#[test]
fn validation_result_has_errors_warnings_stats() {
    let g = minimal_valid_grammar();
    let mut v = GrammarValidator::new();
    let r: ValidationResult = v.validate(&g);
    // Just verify we can access all three fields
    let _ = &r.errors;
    let _ = &r.warnings;
    let _ = &r.stats;
}

#[test]
fn validation_result_stats_match_grammar() {
    let g = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.stats.total_tokens >= 2);
    assert!(r.stats.total_rules >= 2);
    assert!(r.stats.max_rule_length >= 1);
}

// ===========================================================================
// Grammar.check_empty_terminals()
// ===========================================================================

#[test]
fn check_empty_terminals_ok_for_normal_tokens() {
    let g = minimal_valid_grammar();
    assert!(g.check_empty_terminals().is_ok());
}

#[test]
fn check_empty_terminals_detects_empty_string_pattern() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "BAD".into(),
            pattern: TokenPattern::String(String::new()),
            fragile: false,
        },
    );
    let result = g.check_empty_terminals();
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("empty string"));
}

#[test]
fn check_empty_terminals_detects_empty_regex_pattern() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "BAD".into(),
            pattern: TokenPattern::Regex(String::new()),
            fragile: false,
        },
    );
    let result = g.check_empty_terminals();
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("empty regex"));
}

// ===========================================================================
// Grammar::validate() – nested symbol validation
// ===========================================================================

#[test]
fn grammar_validate_optional_with_unresolved_inner() {
    let mut g = Grammar::new("test".into());
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Optional(Box::new(Symbol::NonTerminal(SymbolId(
            99,
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(matches!(
        g.validate().unwrap_err(),
        GrammarError::UnresolvedSymbol(_)
    ));
}

#[test]
fn grammar_validate_repeat_with_unresolved_inner() {
    let mut g = Grammar::new("test".into());
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(88))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(g.validate().is_err());
}

#[test]
fn grammar_validate_choice_with_unresolved_branch() {
    let mut g = Grammar::new("test".into());
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "X".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(tok),
            Symbol::NonTerminal(SymbolId(77)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(g.validate().is_err());
}

#[test]
fn grammar_validate_sequence_with_unresolved_element() {
    let mut g = Grammar::new("test".into());
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "Y".into(),
            pattern: TokenPattern::String("y".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(tok),
            Symbol::NonTerminal(SymbolId(66)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(g.validate().is_err());
}

#[test]
fn grammar_validate_epsilon_is_ok() {
    let mut g = Grammar::new("test".into());
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(g.validate().is_ok());
}

// ===========================================================================
// ValidationError – Debug trait
// ===========================================================================

#[test]
fn validation_error_debug_all_variants() {
    let variants: Vec<ValidationError> = vec![
        ValidationError::EmptyGrammar,
        ValidationError::NoExplicitStartRule,
        ValidationError::UndefinedSymbol {
            symbol: SymbolId(1),
            location: "x".into(),
        },
        ValidationError::UnreachableSymbol {
            symbol: SymbolId(2),
            name: "y".into(),
        },
        ValidationError::NonProductiveSymbol {
            symbol: SymbolId(3),
            name: "z".into(),
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
            pattern: "x".into(),
            error: "e".into(),
        },
        ValidationError::ExternalTokenConflict {
            token1: "a".into(),
            token2: "b".into(),
        },
    ];
    for v in &variants {
        let dbg = format!("{v:?}");
        assert!(!dbg.is_empty());
    }
}

// ===========================================================================
// ValidationWarning – Debug trait
// ===========================================================================

#[test]
fn validation_warning_debug_all_variants() {
    let variants: Vec<ValidationWarning> = vec![
        ValidationWarning::UnusedToken {
            token: SymbolId(1),
            name: "A".into(),
        },
        ValidationWarning::DuplicateTokenPattern {
            tokens: vec![SymbolId(1)],
            pattern: "+".into(),
        },
        ValidationWarning::AmbiguousGrammar {
            message: "m".into(),
        },
        ValidationWarning::MissingFieldNames {
            rule_symbol: SymbolId(1),
        },
        ValidationWarning::InefficientRule {
            symbol: SymbolId(1),
            suggestion: "s".into(),
        },
    ];
    for w in &variants {
        let dbg = format!("{w:?}");
        assert!(!dbg.is_empty());
    }
}

// ===========================================================================
// Edge case: GrammarBuilder-produced grammar through validator
// ===========================================================================

#[test]
fn builder_grammar_validates_via_grammar_validate() {
    let g = GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    // Grammar::validate checks field ordering and symbol resolution
    assert!(g.validate().is_ok());
    // GrammarValidator performs deeper analysis; just verify it runs without panic
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    // The validator may report warnings (e.g., missing field names) but shouldn't crash
    let _ = r.errors.len();
    let _ = r.warnings.len();
}
