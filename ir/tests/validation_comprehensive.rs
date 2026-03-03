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
