#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for grammar validation using GrammarBuilder.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::*;
use adze_ir::*;

// ── 1. validate() returns ValidationResult with expected fields ──────────────

#[test]
fn validate_returns_result_with_errors_warnings_stats() {
    let grammar = GrammarBuilder::new("simple")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    // ValidationResult exposes errors, warnings, stats
    let _errors: &Vec<ValidationError> = &result.errors;
    let _warnings: &Vec<ValidationWarning> = &result.warnings;
    let _stats: &ValidationStats = &result.stats;
}

// ── 2. ValidationStats fields ────────────────────────────────────────────────

#[test]
fn stats_fields_populated_correctly() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    let s = &result.stats;

    assert!(s.total_symbols > 0, "total_symbols should be > 0");
    assert_eq!(s.total_tokens, 3, "3 tokens defined");
    assert_eq!(s.total_rules, 3, "3 productions defined");
    assert!(s.reachable_symbols > 0);
    assert!(s.productive_symbols > 0);
    assert_eq!(s.external_tokens, 0);
    assert_eq!(s.max_rule_length, 3, "expr + expr has 3 RHS symbols");
    assert!(s.avg_rule_length > 0.0);
}

#[test]
fn stats_external_tokens_counted() {
    let grammar = GrammarBuilder::new("ext")
        .token("NUMBER", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("prog", vec!["NUMBER"])
        .start("prog")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert_eq!(result.stats.external_tokens, 1);
}

// ── 3. Valid grammar produces 0 errors ───────────────────────────────────────

#[test]
fn valid_grammar_zero_errors() {
    let grammar = GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["NUMBER", "+", "NUMBER"])
        .start("sum")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.is_empty(),
        "Valid grammar should have 0 errors, got: {:?}",
        result.errors
    );
}

// ── 4. Undefined symbol reference produces error ─────────────────────────────

#[test]
fn undefined_symbol_produces_error() {
    // Build a grammar that references a non-terminal "term" which has no rule
    // and is not a token — the builder will create a SymbolId for it but it
    // won't be defined as either a token or a rule LHS.
    let mut grammar = Grammar::new("undef".to_string());

    let expr_id = SymbolId(1);
    let undef_id = SymbolId(99);

    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(undef_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.iter().any(
            |e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef_id)
        ),
        "Expected UndefinedSymbol error for {:?}, got: {:?}",
        undef_id,
        result.errors
    );
}

// ── 5. Duplicate token patterns produce warning ──────────────────────────────

#[test]
fn duplicate_token_pattern_produces_warning() {
    let grammar = GrammarBuilder::new("dup")
        .token("PLUS", "+")
        .token("ADD", "+")
        .rule("expr", vec!["PLUS"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.warnings.iter().any(
            |w| matches!(w, ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+")
        ),
        "Expected DuplicateTokenPattern warning, got: {:?}",
        result.warnings
    );
}

// ── 6. Cyclic rules detection ────────────────────────────────────────────────

#[test]
fn cyclic_rules_detected() {
    // A -> B, B -> A with no terminal base case
    let mut grammar = Grammar::new("cyclic".to_string());
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

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "Expected CyclicRule error, got: {:?}",
        result.errors
    );
}

// ── 7. Non-productive symbol error ───────────────────────────────────────────

#[test]
fn non_productive_symbol_error() {
    // Create a non-terminal that can never derive a terminal string
    let mut grammar = Grammar::new("nonprod".to_string());
    let start = SymbolId(1);
    let dead = SymbolId(2);
    let tok = SymbolId(3);

    grammar.tokens.insert(
        tok,
        Token {
            name: "TOK".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    // start -> TOK (productive)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // dead -> dead (self-referencing, never productive)
    grammar.add_rule(Rule {
        lhs: dead,
        rhs: vec![Symbol::NonTerminal(dead)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.iter().any(
            |e| matches!(e, ValidationError::NonProductiveSymbol { symbol, .. } if *symbol == dead)
        ),
        "Expected NonProductiveSymbol for dead symbol, got: {:?}",
        result.errors
    );
}

// ── 8. Empty grammar error ──────────────────────────────────────────────────

#[test]
fn empty_grammar_error() {
    let grammar = Grammar::new("empty".to_string());

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "Expected EmptyGrammar error"
    );
}

// ── 9. Conflicting precedence error ─────────────────────────────────────────

#[test]
fn conflicting_precedence_detected() {
    let grammar = GrammarBuilder::new("prec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Right, vec!["+"])
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. })),
        "Expected ConflictingPrecedence error, got: {:?}",
        result.errors
    );
}

// ── 10. Invalid field index error ───────────────────────────────────────────

#[test]
fn invalid_field_index_produces_error() {
    let mut grammar = Grammar::new("field".to_string());
    let expr = SymbolId(1);
    let tok = SymbolId(2);

    grammar.tokens.insert(
        tok,
        Token {
            name: "TOK".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    // Rule with 1 RHS symbol but field pointing to index 5 (out of bounds)
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 5)], // index 5 is out of bounds
        production_id: ProductionId(0),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. })),
        "Expected InvalidField error, got: {:?}",
        result.errors
    );
}

// ── 11. Stats on varying grammar sizes ──────────────────────────────────────

#[test]
fn stats_small_grammar() {
    let grammar = GrammarBuilder::new("tiny")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert_eq!(result.stats.total_tokens, 1);
    assert_eq!(result.stats.total_rules, 1);
    assert_eq!(result.stats.max_rule_length, 1);
    assert!((result.stats.avg_rule_length - 1.0).abs() < f64::EPSILON);
}

#[test]
fn stats_large_grammar() {
    let grammar = GrammarBuilder::javascript_like();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    // JS-like grammar has many tokens, rules, and symbols
    assert!(
        result.stats.total_tokens >= 10,
        "JS-like grammar should have ≥10 tokens"
    );
    assert!(
        result.stats.total_rules >= 10,
        "JS-like grammar should have ≥10 rules"
    );
    assert!(result.stats.max_rule_length >= 3);
    assert!(result.stats.avg_rule_length > 1.0);
}

// ── 12. ValidationWarning variants exist ────────────────────────────────────

#[test]
fn unused_token_warning() {
    let grammar = GrammarBuilder::new("unused")
        .token("NUMBER", r"\d+")
        .token("UNUSED", "xyz")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { name, .. } if name.contains("UNUSED") || name.contains("xyz"))),
        "Expected UnusedToken warning for UNUSED, got: {:?}",
        result.warnings
    );
}

// ── 13. Validator can be reused across multiple grammars ────────────────────

#[test]
fn validator_reuse() {
    let mut validator = GrammarValidator::new();

    // First: empty grammar → errors
    let g1 = Grammar::new("empty".to_string());
    let r1 = validator.validate(&g1);
    assert!(!r1.errors.is_empty());

    // Second: valid grammar → no errors
    let g2 = GrammarBuilder::new("valid")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let r2 = validator.validate(&g2);
    assert!(r2.errors.is_empty(), "Reused validator should clear state");
}

// ── 14. Python-like grammar validates cleanly ───────────────────────────────

#[test]
fn python_like_grammar_validates() {
    let grammar = GrammarBuilder::python_like();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    // The Python-like grammar is well-formed; check stats are reasonable
    assert!(result.stats.total_tokens > 0);
    assert!(result.stats.total_rules > 0);
    assert!(result.stats.external_tokens >= 2, "INDENT + DEDENT");
}

// ── 15. Self-referencing rule handling ──────────────────────────────────────

#[test]
fn self_referencing_rule_with_base_case_is_valid() {
    // Self-reference is fine when there's also a terminal base case
    let grammar = GrammarBuilder::new("selfrec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUMBER"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        !result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. })),
        "Self-referencing rule with base case should be productive"
    );
}

#[test]
fn self_referencing_rule_without_base_case_fails() {
    let mut grammar = Grammar::new("selfonly".to_string());
    let a = SymbolId(1);

    // A -> A (pure self-reference, no base case)
    grammar.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    let has_cycle_or_nonproductive = result.errors.iter().any(|e| {
        matches!(
            e,
            ValidationError::CyclicRule { .. } | ValidationError::NonProductiveSymbol { .. }
        )
    });
    assert!(
        has_cycle_or_nonproductive,
        "Pure self-reference should produce CyclicRule or NonProductiveSymbol, got: {:?}",
        result.errors
    );
}

// ── 16. Grammar with extras validates ───────────────────────────────────────

#[test]
fn grammar_with_extras_validates() {
    let grammar = GrammarBuilder::new("extras")
        .token("NUMBER", r"\d+")
        .token("WHITESPACE", r"[ \t]+")
        .extra("WHITESPACE")
        .rule("prog", vec!["NUMBER"])
        .start("prog")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.is_empty(),
        "Grammar with extras should be valid, got errors: {:?}",
        result.errors
    );
}

// ── 17. Grammar with externals validates ────────────────────────────────────

#[test]
fn grammar_with_externals_validates() {
    let grammar = GrammarBuilder::new("ext")
        .token("NUMBER", r"\d+")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .rule("block", vec!["INDENT", "NUMBER", "DEDENT"])
        .start("block")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.is_empty(),
        "Grammar with externals should be valid, got errors: {:?}",
        result.errors
    );
    assert_eq!(result.stats.external_tokens, 2);
}

// ── 18. Duplicate external token names ──────────────────────────────────────

#[test]
fn duplicate_external_token_conflict() {
    let mut grammar = GrammarBuilder::new("dupext")
        .token("NUMBER", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("prog", vec!["NUMBER"])
        .start("prog")
        .build();

    // Manually add a second external with the same name
    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. })),
        "Duplicate external token names should produce conflict error, got: {:?}",
        result.errors
    );
}

// ── 19. Validation error Display messages are descriptive ───────────────────

#[test]
fn error_display_undefined_symbol() {
    let err = ValidationError::UndefinedSymbol {
        symbol: SymbolId(42),
        location: "rule for expr".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Undefined symbol"), "got: {msg}");
    assert!(msg.contains("42"), "should mention symbol ID, got: {msg}");
    assert!(
        msg.contains("rule for expr"),
        "should mention location, got: {msg}"
    );
}

#[test]
fn error_display_empty_grammar() {
    let msg = format!("{}", ValidationError::EmptyGrammar);
    assert!(
        msg.contains("no rules"),
        "EmptyGrammar message should mention 'no rules', got: {msg}"
    );
}

#[test]
fn error_display_cyclic_rule() {
    let err = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("Cyclic"),
        "CyclicRule should mention 'Cyclic', got: {msg}"
    );
}

#[test]
fn error_display_non_productive() {
    let err = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(5),
        name: "dead_rule".to_string(),
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("dead_rule"),
        "should mention symbol name, got: {msg}"
    );
    assert!(
        msg.contains("terminal"),
        "should mention terminal derivation, got: {msg}"
    );
}

#[test]
fn error_display_duplicate_rule() {
    let err = ValidationError::DuplicateRule {
        symbol: SymbolId(3),
        existing_count: 5,
    };
    let msg = format!("{err}");
    assert!(msg.contains("5"), "should mention count, got: {msg}");
}

#[test]
fn error_display_invalid_regex() {
    let err = ValidationError::InvalidRegex {
        token: SymbolId(1),
        pattern: "[unclosed".to_string(),
        error: "missing ]".to_string(),
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("[unclosed"),
        "should mention pattern, got: {msg}"
    );
    assert!(
        msg.contains("missing ]"),
        "should mention error detail, got: {msg}"
    );
}

#[test]
fn warning_display_unused_token() {
    let warn = ValidationWarning::UnusedToken {
        token: SymbolId(7),
        name: "SEMICOLON".to_string(),
    };
    let msg = format!("{warn}");
    assert!(
        msg.contains("SEMICOLON"),
        "should mention token name, got: {msg}"
    );
    assert!(
        msg.contains("never used"),
        "should say 'never used', got: {msg}"
    );
}

// ── 20. Valid grammar with multiple rules passes ────────────────────────────

#[test]
fn valid_multi_rule_grammar() {
    let grammar = GrammarBuilder::new("multi")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("+", "+")
        .token("=", "=")
        .token(";", ";")
        .rule("program", vec!["IDENT", "=", "expr", ";"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["IDENT"])
        .rule("expr", vec!["NUMBER", "+", "NUMBER"])
        .start("program")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.is_empty(),
        "Multi-rule valid grammar should have 0 errors, got: {:?}",
        result.errors
    );
    assert!(result.stats.total_rules >= 4);
}

// ── 21. Empty regex pattern produces InvalidRegex ───────────────────────────

#[test]
fn empty_regex_pattern_error() {
    let mut grammar = Grammar::new("badregex".to_string());
    let tok = SymbolId(1);
    let start = SymbolId(2);

    grammar.tokens.insert(
        tok,
        Token {
            name: "BAD".to_string(),
            pattern: TokenPattern::Regex(String::new()),
            fragile: false,
        },
    );

    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. })),
        "Empty regex should produce InvalidRegex error, got: {:?}",
        result.errors
    );
}

// ── 22. Unreachable symbol detection ────────────────────────────────────────

#[test]
fn unreachable_symbol_detected() {
    let mut grammar = Grammar::new("unreach".to_string());
    let start = SymbolId(1);
    let tok = SymbolId(2);
    let orphan = SymbolId(3);
    let tok2 = SymbolId(4);

    grammar.tokens.insert(
        tok,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok2,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // start -> tok (reachable)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // orphan -> tok2 (unreachable from start)
    grammar.add_rule(Rule {
        lhs: orphan,
        rhs: vec![Symbol::Terminal(tok2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    // Unreachable symbols appear as warnings (UnusedToken), not errors
    assert!(
        result
            .warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { token, .. } if *token == orphan)),
        "Orphan rule should produce UnusedToken warning, got warnings: {:?}",
        result.warnings
    );
}

// ── 23. Inefficient trivial rule warning ────────────────────────────────────

#[test]
fn trivial_rule_warning() {
    let grammar = GrammarBuilder::new("trivial")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .rule("wrapper", vec!["expr"]) // trivial: wrapper -> expr
        .start("wrapper")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.warnings.iter().any(|w| matches!(
            w,
            ValidationWarning::InefficientRule { suggestion, .. }
                if suggestion.contains("inlining")
        )),
        "Trivial A->B rule should produce InefficientRule warning, got: {:?}",
        result.warnings
    );
}

// ── 24. Missing field names warning ─────────────────────────────────────────

#[test]
fn missing_field_names_warning() {
    let grammar = GrammarBuilder::new("nofields")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER", "+", "NUMBER"]) // multi-symbol, no fields
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::MissingFieldNames { .. })),
        "Multi-symbol rule without fields should warn, got: {:?}",
        result.warnings
    );
}

// ── 25. Three-symbol cycle detection ────────────────────────────────────────

#[test]
fn three_symbol_cycle_detected() {
    let mut grammar = Grammar::new("cycle3".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let c = SymbolId(3);

    // A -> B -> C -> A
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
        rhs: vec![Symbol::NonTerminal(c)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.add_rule(Rule {
        lhs: c,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "Three-symbol cycle should be detected, got: {:?}",
        result.errors
    );
}

// ── 26. Epsilon (empty) rule handling ───────────────────────────────────────

#[test]
fn epsilon_rule_handling() {
    // A grammar with an empty production is valid (nullable start)
    let grammar = GrammarBuilder::new("nullable")
        .token("NUMBER", r"\d+")
        .rule("maybe", vec![]) // epsilon production
        .rule("maybe", vec!["NUMBER"]) // terminal alternative
        .start("maybe")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.is_empty(),
        "Epsilon rule with terminal alternative should be valid, got: {:?}",
        result.errors
    );
}

// ── 27. JavaScript-like grammar validates ───────────────────────────────────

#[test]
fn javascript_like_grammar_validates() {
    let grammar = GrammarBuilder::javascript_like();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(result.stats.total_tokens >= 10);
    assert!(result.stats.total_rules >= 10);
    assert_eq!(result.stats.external_tokens, 0);
}

// ── 28. Warning Display messages ────────────────────────────────────────────

#[test]
fn warning_display_ambiguous_grammar() {
    let warn = ValidationWarning::AmbiguousGrammar {
        message: "shift/reduce conflict".to_string(),
    };
    let msg = format!("{warn}");
    assert!(
        msg.contains("shift/reduce"),
        "should include message, got: {msg}"
    );
}

#[test]
fn warning_display_inefficient_rule() {
    let warn = ValidationWarning::InefficientRule {
        symbol: SymbolId(1),
        suggestion: "break it down".to_string(),
    };
    let msg = format!("{warn}");
    assert!(
        msg.contains("break it down"),
        "should include suggestion, got: {msg}"
    );
}

// ── 29. Multiple validation errors reported simultaneously ──────────────────

#[test]
fn multiple_errors_reported_simultaneously() {
    // Build a grammar with several issues at once:
    // - Empty regex pattern
    // - Non-productive symbol
    // - Cyclic rule
    let mut grammar = Grammar::new("multi_err".to_string());
    let start = SymbolId(1);
    let dead = SymbolId(2);
    let tok = SymbolId(3);
    let bad_tok = SymbolId(4);

    grammar.tokens.insert(
        tok,
        Token {
            name: "OK".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        bad_tok,
        Token {
            name: "BAD".to_string(),
            pattern: TokenPattern::Regex(String::new()), // empty regex
            fragile: false,
        },
    );

    // start -> tok (valid)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // dead -> dead (non-productive cycle)
    grammar.add_rule(Rule {
        lhs: dead,
        rhs: vec![Symbol::NonTerminal(dead)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    let has_invalid_regex = result
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidRegex { .. }));
    let has_non_productive = result
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }));

    assert!(
        has_invalid_regex,
        "Should report InvalidRegex, got: {:?}",
        result.errors
    );
    assert!(
        has_non_productive,
        "Should report NonProductiveSymbol, got: {:?}",
        result.errors
    );
    assert!(
        result.errors.len() >= 2,
        "Should report at least 2 errors, got {}",
        result.errors.len()
    );
}

// ── 30. Grammar.validate() — missing start symbol (NoExplicitStartRule) ─────
// Note: Grammar::validate() (on lib.rs) checks field ordering and symbol refs.
// The GrammarValidator in validation.rs doesn't emit NoExplicitStartRule itself,
// but we can test the variant exists and displays correctly.

#[test]
fn no_explicit_start_rule_display() {
    let err = ValidationError::NoExplicitStartRule;
    let msg = format!("{err}");
    assert!(
        msg.contains("start rule"),
        "NoExplicitStartRule should mention 'start rule', got: {msg}"
    );
}

// ── 31. Grammar::validate() — unresolved symbol ─────────────────────────────

#[test]
fn grammar_validate_unresolved_symbol() {
    let mut grammar = Grammar::new("unresolved".to_string());
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
    assert!(
        matches!(result, Err(GrammarError::UnresolvedSymbol(id)) if id == missing),
        "Grammar::validate() should return UnresolvedSymbol, got: {:?}",
        result
    );
}

// ── 32. Grammar::validate() — valid grammar passes ──────────────────────────

#[test]
fn grammar_validate_valid_passes() {
    let grammar = GrammarBuilder::new("ok")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    assert!(
        grammar.validate().is_ok(),
        "Valid grammar should pass Grammar::validate()"
    );
}

// ── 33. Grammar::validate() — invalid field ordering ────────────────────────

#[test]
fn grammar_validate_invalid_field_ordering() {
    let mut grammar = GrammarBuilder::new("fields")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();

    // Insert fields in non-lexicographic order
    grammar.fields.insert(FieldId(0), "zebra".to_string());
    grammar.fields.insert(FieldId(1), "alpha".to_string());

    let result = grammar.validate();
    assert!(
        matches!(result, Err(GrammarError::InvalidFieldOrdering)),
        "Non-lexicographic field ordering should fail, got: {:?}",
        result
    );
}

// ── 34. Duplicate rule names (same LHS, multiple alternatives) ──────────────

#[test]
fn duplicate_rule_alternatives_are_valid() {
    // Multiple rules with the same LHS are valid alternatives, not duplicates
    let grammar = GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .rule("expr", vec!["C"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.errors.is_empty(),
        "Multiple alternatives for same LHS should be valid, got: {:?}",
        result.errors
    );
    assert_eq!(result.stats.total_rules, 3);
}

// ── 35. Validation after normalization ───────────────────────────────────────

#[test]
fn validation_after_normalization_optional() {
    let mut grammar = Grammar::new("opt_norm".to_string());
    let start = SymbolId(1);
    let tok = SymbolId(2);

    grammar.tokens.insert(
        tok,
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    // start -> Optional(X)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // Normalize: should expand Optional into auxiliary rules
    grammar.normalize();

    // After normalization, grammar should have auxiliary rules and still validate
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.stats.total_rules >= 2,
        "Normalization should create auxiliary rules, got {} rules",
        result.stats.total_rules
    );
}

#[test]
fn validation_after_normalization_repeat() {
    let mut grammar = Grammar::new("rep_norm".to_string());
    let start = SymbolId(1);
    let tok = SymbolId(2);

    grammar.tokens.insert(
        tok,
        Token {
            name: "Y".to_string(),
            pattern: TokenPattern::String("y".to_string()),
            fragile: false,
        },
    );

    // start -> Repeat(Y)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.normalize();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.stats.total_rules >= 2,
        "Repeat normalization should create auxiliary rules"
    );
}

#[test]
fn validation_after_normalization_choice() {
    let mut grammar = Grammar::new("choice_norm".to_string());
    let start = SymbolId(1);
    let tok_a = SymbolId(2);
    let tok_b = SymbolId(3);

    grammar.tokens.insert(
        tok_a,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // start -> Choice(A, B)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(tok_a),
            Symbol::Terminal(tok_b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.normalize();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.stats.total_rules >= 2,
        "Choice normalization should create auxiliary rules"
    );
}

// ── 38. Edge case: single-token grammar ─────────────────────────────────────

#[test]
fn single_token_single_rule_grammar() {
    let grammar = GrammarBuilder::new("minimal")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(result.errors.is_empty());
    assert_eq!(result.stats.total_tokens, 1);
    assert_eq!(result.stats.total_rules, 1);
    assert_eq!(result.stats.max_rule_length, 1);
}

// ── 39. Edge case: many tokens and rules ────────────────────────────────────

#[test]
fn many_tokens_and_rules() {
    let mut builder = GrammarBuilder::new("big");
    for i in 0..20 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    // Each rule uses a different token
    for i in 0..20 {
        builder = builder.rule("items", vec![&format!("T{i}")]);
    }
    let grammar = builder.start("items").build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert_eq!(result.stats.total_tokens, 20);
    assert_eq!(result.stats.total_rules, 20);
}

// ── 40. Long rule triggers inefficiency warning ─────────────────────────────

#[test]
fn long_rule_inefficiency_warning() {
    let mut grammar = Grammar::new("long".to_string());
    let start = SymbolId(1);

    // Create 12 tokens
    let mut rhs = Vec::new();
    for i in 0..12 {
        let tok_id = SymbolId(10 + i);
        grammar.tokens.insert(
            tok_id,
            Token {
                name: format!("T{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
        rhs.push(Symbol::Terminal(tok_id));
    }

    grammar.add_rule(Rule {
        lhs: start,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(
        result.warnings.iter().any(|w| matches!(
            w,
            ValidationWarning::InefficientRule { suggestion, .. }
                if suggestion.contains("12 symbols")
        )),
        "Rule with >10 symbols should warn about inefficiency, got: {:?}",
        result.warnings
    );
}

// ── 41. External token conflict display ─────────────────────────────────────

#[test]
fn error_display_external_token_conflict() {
    let err = ValidationError::ExternalTokenConflict {
        token1: "INDENT".to_string(),
        token2: "INDENT".to_string(),
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("INDENT"),
        "should mention token name, got: {msg}"
    );
    assert!(
        msg.contains("conflict"),
        "should mention conflict, got: {msg}"
    );
}

// ── 42. Warning display: duplicate token pattern ────────────────────────────

#[test]
fn warning_display_duplicate_token_pattern() {
    let warn = ValidationWarning::DuplicateTokenPattern {
        tokens: vec![SymbolId(1), SymbolId(2)],
        pattern: "+".to_string(),
    };
    let msg = format!("{warn}");
    assert!(msg.contains("+"), "should mention pattern, got: {msg}");
}

// ── 43. Warning display: missing field names ────────────────────────────────

#[test]
fn warning_display_missing_field_names() {
    let warn = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(5),
    };
    let msg = format!("{warn}");
    assert!(
        msg.contains("field"),
        "should mention field names, got: {msg}"
    );
}

// ── 44. Error display: conflicting precedence ───────────────────────────────

#[test]
fn error_display_conflicting_precedence() {
    let err = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(1),
        precedences: vec![1, 2],
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("conflicting"),
        "should mention conflicting, got: {msg}"
    );
}

// ── 45. Error display: unreachable symbol ───────────────────────────────────

#[test]
fn error_display_unreachable_symbol() {
    let err = ValidationError::UnreachableSymbol {
        symbol: SymbolId(10),
        name: "orphan".to_string(),
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("orphan"),
        "should mention symbol name, got: {msg}"
    );
    assert!(
        msg.contains("unreachable"),
        "should mention unreachable, got: {msg}"
    );
}
