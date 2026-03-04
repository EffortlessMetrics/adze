//! Comprehensive tests for the grammar validation subsystem.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, Precedence, PrecedenceKind, ProductionId, Rule,
    Symbol, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate(grammar: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

/// Build a minimal valid grammar: start -> NUMBER
fn minimal_valid_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("NUMBER", r"\d+")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build()
}

/// Build a simple arithmetic grammar
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. Valid grammars pass validation
// ===========================================================================

#[test]
fn valid_minimal_grammar_no_errors() {
    let r = validate(&minimal_valid_grammar());
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_minimal_grammar_stats() {
    let r = validate(&minimal_valid_grammar());
    assert!(r.stats.total_tokens >= 1);
    assert!(r.stats.total_rules >= 1);
}

#[test]
fn valid_arith_grammar_no_fatal_errors() {
    let r = validate(&arith_grammar());
    // Recursive grammars report CyclicRule; that's expected — check no *other* errors
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn valid_arith_grammar_stats_rules() {
    let r = validate(&arith_grammar());
    assert_eq!(r.stats.total_rules, 3);
}

#[test]
fn valid_arith_grammar_stats_tokens() {
    let r = validate(&arith_grammar());
    assert!(r.stats.total_tokens >= 2);
}

#[test]
fn valid_recursive_grammar() {
    // list -> item | list item
    let g = GrammarBuilder::new("rec")
        .token("ITEM", "x")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["list", "ITEM"])
        .start("list")
        .build();
    let r = validate(&g);
    // Validator reports CyclicRule for self-recursion; filter those out
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn valid_multi_rule_grammar() {
    let g = GrammarBuilder::new("multi")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("+", "+")
        .token(";", ";")
        .rule("program", vec!["stmt"])
        .rule("program", vec!["program", "stmt"])
        .rule("stmt", vec!["expr", ";"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["IDENT"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("program")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn valid_grammar_reachable_count() {
    let r = validate(&minimal_valid_grammar());
    assert!(r.stats.reachable_symbols >= 1);
}

#[test]
fn valid_grammar_productive_count() {
    let r = validate(&minimal_valid_grammar());
    assert!(r.stats.productive_symbols >= 1);
}

#[test]
fn valid_grammar_max_rule_length() {
    let r = validate(&arith_grammar());
    assert_eq!(r.stats.max_rule_length, 3);
}

#[test]
fn valid_grammar_avg_rule_length() {
    let r = validate(&arith_grammar());
    // 3 rules: lengths 3, 3, 1 → avg ≈ 2.33
    assert!(r.stats.avg_rule_length > 2.0);
}

// ===========================================================================
// 2. Invalid grammars caught
// ===========================================================================

#[test]
fn empty_grammar_error() {
    let g = Grammar::new("empty".into());
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn undefined_symbol_detected() {
    let mut g = Grammar::new("undef".into());
    let a = SymbolId(1);
    let undef = SymbolId(99);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(undef)],
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
fn non_productive_cycle_detected() {
    let mut g = Grammar::new("np".into());
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
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

#[test]
fn cyclic_rule_detected() {
    let mut g = Grammar::new("cyc".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let c = SymbolId(3);
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
        rhs: vec![Symbol::NonTerminal(c)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.add_rule(Rule {
        lhs: c,
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
fn invalid_field_index_detected() {
    let mut g = Grammar::new("fld".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    g.tokens.insert(
        num,
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 10)],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
}

#[test]
fn conflicting_precedence_detected() {
    let mut g = Grammar::new("prec".into());
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
    let r = validate(&g);
    assert!(r.errors.iter().any(
        |e| matches!(e, ValidationError::ConflictingPrecedence { symbol, .. } if *symbol == plus)
    ));
}

#[test]
fn external_token_conflict_detected() {
    let mut g = Grammar::new("ext".into());
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(1),
    });
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
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
fn invalid_regex_empty_pattern() {
    let mut g = Grammar::new("re".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "BAD".into(),
            pattern: TokenPattern::Regex("".into()),
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

// ===========================================================================
// 3. ValidationError display formatting
// ===========================================================================

#[test]
fn display_empty_grammar() {
    let s = format!("{}", ValidationError::EmptyGrammar);
    assert!(s.contains("no rules"));
}

#[test]
fn display_undefined_symbol() {
    let e = ValidationError::UndefinedSymbol {
        symbol: SymbolId(42),
        location: "rule for expr".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("Undefined"));
    assert!(s.contains("42"));
}

#[test]
fn display_unreachable_symbol() {
    let e = ValidationError::UnreachableSymbol {
        symbol: SymbolId(5),
        name: "orphan".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("unreachable"));
    assert!(s.contains("orphan"));
}

#[test]
fn display_non_productive_symbol() {
    let e = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(3),
        name: "dead".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("terminal strings"));
}

#[test]
fn display_cyclic_rule() {
    let e = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let s = format!("{e}");
    assert!(s.contains("Cyclic"));
}

#[test]
fn display_duplicate_rule() {
    let e = ValidationError::DuplicateRule {
        symbol: SymbolId(1),
        existing_count: 3,
    };
    let s = format!("{e}");
    assert!(s.contains("3"));
}

#[test]
fn display_invalid_field() {
    let e = ValidationError::InvalidField {
        field_id: FieldId(0),
        rule_symbol: SymbolId(1),
    };
    let s = format!("{e}");
    assert!(s.contains("Invalid field"));
}

#[test]
fn display_no_explicit_start_rule() {
    let s = format!("{}", ValidationError::NoExplicitStartRule);
    assert!(s.contains("start rule"));
}

#[test]
fn display_conflicting_precedence() {
    let e = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(1),
        precedences: vec![1, 2],
    };
    let s = format!("{e}");
    assert!(s.contains("conflicting"));
}

#[test]
fn display_invalid_regex() {
    let e = ValidationError::InvalidRegex {
        token: SymbolId(1),
        pattern: "[bad".into(),
        error: "unclosed".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("Invalid regex"));
    assert!(s.contains("[bad"));
}

#[test]
fn display_external_token_conflict() {
    let e = ValidationError::ExternalTokenConflict {
        token1: "A".into(),
        token2: "B".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("conflict"));
}

// ===========================================================================
// 4. ValidationWarning for unused tokens / rules
// ===========================================================================

#[test]
fn unused_token_warning() {
    let g = GrammarBuilder::new("unused")
        .token("NUMBER", r"\d+")
        .token("UNUSED_TOK", "zzz")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build();
    let r = validate(&g);
    // The unused token should show up in warnings (as UnusedToken)
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
    );
}

#[test]
fn duplicate_token_pattern_warning() {
    let mut g = Grammar::new("dup".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(r.warnings.iter().any(
        |w| matches!(w, ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+")
    ));
}

#[test]
fn display_unused_token_warning() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".into(),
    };
    let s = format!("{w}");
    assert!(s.contains("FOO"));
    assert!(s.contains("never used"));
}

#[test]
fn display_duplicate_token_pattern_warning() {
    let w = ValidationWarning::DuplicateTokenPattern {
        tokens: vec![SymbolId(1), SymbolId(2)],
        pattern: "+".into(),
    };
    let s = format!("{w}");
    assert!(s.contains("+"));
}

#[test]
fn display_ambiguous_grammar_warning() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "shift/reduce".into(),
    };
    let s = format!("{w}");
    assert!(s.contains("shift/reduce"));
}

#[test]
fn display_missing_field_names_warning() {
    let w = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(1),
    };
    let s = format!("{w}");
    assert!(s.contains("field names"));
}

#[test]
fn display_inefficient_rule_warning() {
    let w = ValidationWarning::InefficientRule {
        symbol: SymbolId(1),
        suggestion: "inline it".into(),
    };
    let s = format!("{w}");
    assert!(s.contains("inline it"));
}

#[test]
fn missing_field_names_warning_multi_rhs() {
    // Rules with >1 RHS symbol and no fields trigger MissingFieldNames warning
    let g = GrammarBuilder::new("mf")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::MissingFieldNames { .. }))
    );
}

#[test]
fn inefficient_trivial_rule_warning() {
    // A rule like `wrapper -> inner` triggers inefficiency warning
    let g = GrammarBuilder::new("triv")
        .token("NUMBER", r"\d+")
        .rule("inner", vec!["NUMBER"])
        .rule("wrapper", vec!["inner"])
        .start("wrapper")
        .build();
    let r = validate(&g);
    assert!(r.warnings.iter().any(|w| matches!(w, ValidationWarning::InefficientRule { suggestion, .. } if suggestion.contains("inlining"))));
}

// ===========================================================================
// 5. Validator with normalized grammars
// ===========================================================================

#[test]
fn normalized_grammar_still_valid() {
    let mut g = GrammarBuilder::new("norm")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    g.normalize();
    let r = validate(&g);
    // After normalization the grammar may have auxiliary rules but should not introduce *new*
    // validation errors of the "EmptyGrammar" kind.
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn normalized_grammar_has_rules() {
    let mut g = minimal_valid_grammar();
    g.normalize();
    let r = validate(&g);
    assert!(r.stats.total_rules >= 1);
}

#[test]
fn normalized_grammar_productive() {
    let mut g = minimal_valid_grammar();
    g.normalize();
    let r = validate(&g);
    assert!(r.stats.productive_symbols >= 1);
}

// ===========================================================================
// 6. Validator with precedence grammars
// ===========================================================================

#[test]
fn precedence_grammar_no_fatal_errors() {
    let g = GrammarBuilder::new("prec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn precedence_grammar_right_assoc() {
    let g = GrammarBuilder::new("right")
        .token("NUMBER", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn precedence_grammar_none_assoc() {
    let g = GrammarBuilder::new("none")
        .token("NUMBER", r"\d+")
        .token("==", "==")
        .rule_with_precedence("expr", vec!["expr", "==", "expr"], 0, Associativity::None)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn precedence_grammar_mixed_assoc() {
    let g = GrammarBuilder::new("mix")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn precedence_grammar_negative_level() {
    let g = GrammarBuilder::new("neg")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], -5, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn edge_empty_grammar() {
    let g = Grammar::new("e".into());
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
    assert_eq!(r.stats.total_rules, 0);
}

#[test]
fn edge_single_token_grammar() {
    let g = minimal_valid_grammar();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn edge_self_recursive_grammar() {
    // self-referential: expr -> expr  (no terminal base case)
    let mut g = Grammar::new("self_rec".into());
    let expr = SymbolId(1);
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::NonTerminal(expr)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    // Should detect cycle or non-productive
    let has_cycle = r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::CyclicRule { .. }));
    let has_non_prod = r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }));
    assert!(
        has_cycle || has_non_prod,
        "expected cycle or non-productive error, got: {:?}",
        r.errors
    );
}

#[test]
fn edge_self_recursive_with_base_case() {
    // expr -> expr "+" NUMBER | NUMBER — valid but cycle is still reported
    let g = GrammarBuilder::new("self_ok")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUMBER"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "unexpected errors: {:?}", non_cycle);
}

#[test]
fn edge_epsilon_rule() {
    // start -> ε | NUMBER
    let g = GrammarBuilder::new("eps")
        .token("NUMBER", r"\d+")
        .rule("start", vec![])
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn edge_only_epsilon_rule() {
    let g = GrammarBuilder::new("eps_only")
        .rule("start", vec![])
        .start("start")
        .build();
    let r = validate(&g);
    // An epsilon-only grammar is technically valid (nullable start)
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn edge_many_alternatives() {
    let mut builder = GrammarBuilder::new("alt");
    for i in 0..20 {
        let tok_name = format!("T{i}");
        builder = builder.token(&tok_name, &tok_name);
    }
    for i in 0..20 {
        let tok_name = format!("T{i}");
        builder = builder.rule("start", vec![&tok_name]);
    }
    // Need to reborrow token names — use a different approach
    let g = builder.start("start").build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn edge_long_rhs_warning() {
    // A rule with >10 symbols triggers inefficiency warning
    let mut g = Grammar::new("long".into());
    let start = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: start,
        rhs: (0..12).map(|_| Symbol::Terminal(tok)).collect(),
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(r.warnings.iter().any(|w| matches!(w, ValidationWarning::InefficientRule { suggestion, .. } if suggestion.contains("12"))));
}

#[test]
fn edge_fragile_token_grammar() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("SEMI", ";")
        .token("NUMBER", r"\d+")
        .rule("start", vec!["NUMBER", "SEMI"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ===========================================================================
// 8. Multiple validation passes produce same result
// ===========================================================================

#[test]
fn idempotent_validation_errors() {
    let g = Grammar::new("empty".into());
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.errors, r2.errors);
}

#[test]
fn idempotent_validation_warnings() {
    let g = arith_grammar();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.warnings, r2.warnings);
}

#[test]
fn idempotent_validation_stats_rules() {
    let g = arith_grammar();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.stats.total_rules, r2.stats.total_rules);
}

#[test]
fn idempotent_validation_stats_tokens() {
    let g = arith_grammar();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.stats.total_tokens, r2.stats.total_tokens);
}

#[test]
fn idempotent_validation_reachable() {
    let g = arith_grammar();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.stats.reachable_symbols, r2.stats.reachable_symbols);
}

#[test]
fn idempotent_validation_productive() {
    let g = arith_grammar();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.stats.productive_symbols, r2.stats.productive_symbols);
}

#[test]
fn same_validator_reused() {
    let mut v = GrammarValidator::new();
    let g1 = minimal_valid_grammar();
    let g2 = Grammar::new("empty".into());
    let r1 = v.validate(&g1);
    let r2 = v.validate(&g2);
    assert!(r1.errors.is_empty());
    assert!(
        r2.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

// ===========================================================================
// 9. Validator after grammar modification
// ===========================================================================

#[test]
fn grammar_starts_valid_then_modified() {
    let mut g = minimal_valid_grammar();
    let r1 = validate(&g);
    assert!(r1.errors.is_empty());

    // Add a dangling non-productive rule
    let dead = SymbolId(200);
    let dead2 = SymbolId(201);
    g.add_rule(Rule {
        lhs: dead,
        rhs: vec![Symbol::NonTerminal(dead2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(100),
    });
    g.add_rule(Rule {
        lhs: dead2,
        rhs: vec![Symbol::NonTerminal(dead)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(101),
    });

    let r2 = validate(&g);
    assert!(
        r2.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

#[test]
fn grammar_add_token_after_build() {
    let mut g = minimal_valid_grammar();
    g.tokens.insert(
        SymbolId(50),
        Token {
            name: "EXTRA".into(),
            pattern: TokenPattern::String("extra".into()),
            fragile: false,
        },
    );
    let r = validate(&g);
    // Extra token not referenced → should be in warnings (unreachable)
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
    );
}

#[test]
fn grammar_clear_rules_makes_empty() {
    let mut g = minimal_valid_grammar();
    g.rules.clear();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

// ===========================================================================
// 10. Large grammars with many rules
// ===========================================================================

#[test]
fn large_grammar_50_tokens() {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..50 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    // start -> T0 | T1 | ... | T49
    for i in 0..50 {
        let name = format!("T{i}");
        builder = builder.rule("start", vec![&name]);
    }
    let g = builder.start("start").build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    assert!(r.stats.total_tokens >= 50);
    assert_eq!(r.stats.total_rules, 50);
}

#[test]
fn large_grammar_chain() {
    // chain: r0 -> T0 r1, r1 -> T1 r2, ..., rN -> TN
    let n = 30;
    let mut builder = GrammarBuilder::new("chain");
    for i in 0..=n {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    for i in 0..n {
        let cur = format!("r{i}");
        let tok = format!("T{i}");
        let next = format!("r{}", i + 1);
        builder = builder.rule(&cur, vec![&tok, &next]);
    }
    let last = format!("r{n}");
    let last_tok = format!("T{n}");
    builder = builder.rule(&last, vec![&last_tok]);
    let g = builder.start("r0").build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn large_grammar_stats_external_tokens() {
    let mut g = GrammarBuilder::new("ext")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .external("EXT1")
        .external("EXT2")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.external_tokens, 2);
}

#[test]
fn large_grammar_many_alternatives_per_symbol() {
    let mut builder = GrammarBuilder::new("alts");
    for i in 0..40 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    for i in 0..40 {
        let tok = format!("T{i}");
        builder = builder.rule("start", vec![&tok]);
    }
    let g = builder.start("start").build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    assert_eq!(r.stats.total_rules, 40);
}

// ===========================================================================
// Additional coverage — builder presets
// ===========================================================================

#[test]
fn python_like_grammar_validates() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    // Python-like grammar should not have EmptyGrammar error
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn javascript_like_grammar_validates() {
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn python_like_grammar_has_externals() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    assert!(r.stats.external_tokens >= 2);
}

#[test]
fn javascript_like_grammar_has_precedence_rules() {
    let g = GrammarBuilder::javascript_like();
    // Should have rules with precedence set
    let has_prec = g.all_rules().any(|r| r.precedence.is_some());
    assert!(has_prec);
}

// ===========================================================================
// ValidationError equality
// ===========================================================================

#[test]
fn validation_error_eq_empty_grammar() {
    assert_eq!(ValidationError::EmptyGrammar, ValidationError::EmptyGrammar);
}

#[test]
fn validation_error_eq_undefined_symbol() {
    let a = ValidationError::UndefinedSymbol {
        symbol: SymbolId(1),
        location: "x".into(),
    };
    let b = ValidationError::UndefinedSymbol {
        symbol: SymbolId(1),
        location: "x".into(),
    };
    assert_eq!(a, b);
}

#[test]
fn validation_error_ne_different_symbol() {
    let a = ValidationError::UndefinedSymbol {
        symbol: SymbolId(1),
        location: "x".into(),
    };
    let b = ValidationError::UndefinedSymbol {
        symbol: SymbolId(2),
        location: "x".into(),
    };
    assert_ne!(a, b);
}

#[test]
fn validation_warning_eq() {
    let a = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "X".into(),
    };
    let b = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "X".into(),
    };
    assert_eq!(a, b);
}

#[test]
fn validation_warning_ne() {
    let a = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "X".into(),
    };
    let b = ValidationWarning::UnusedToken {
        token: SymbolId(2),
        name: "Y".into(),
    };
    assert_ne!(a, b);
}

// ===========================================================================
// ValidationError / ValidationWarning clone + debug
// ===========================================================================

#[test]
fn validation_error_clone() {
    let e = ValidationError::EmptyGrammar;
    let e2 = e.clone();
    assert_eq!(e, e2);
}

#[test]
fn validation_error_debug() {
    let e = ValidationError::EmptyGrammar;
    let s = format!("{e:?}");
    assert!(s.contains("EmptyGrammar"));
}

#[test]
fn validation_warning_clone() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "test".into(),
    };
    let w2 = w.clone();
    assert_eq!(w, w2);
}

#[test]
fn validation_warning_debug() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "test".into(),
    };
    let s = format!("{w:?}");
    assert!(s.contains("AmbiguousGrammar"));
}

// ===========================================================================
// Stats edge cases
// ===========================================================================

#[test]
fn stats_empty_grammar_zeroes() {
    let r = validate(&Grammar::new("e".into()));
    assert_eq!(r.stats.total_rules, 0);
    assert_eq!(r.stats.total_tokens, 0);
    assert_eq!(r.stats.max_rule_length, 0);
}

#[test]
fn stats_avg_rule_length_single_rule() {
    let g = minimal_valid_grammar();
    let r = validate(&g);
    assert!((r.stats.avg_rule_length - 1.0).abs() < 0.01);
}

#[test]
fn stats_reachable_includes_start() {
    let g = minimal_valid_grammar();
    let r = validate(&g);
    assert!(r.stats.reachable_symbols >= 1);
}

// ===========================================================================
// Extras / whitespace handling
// ===========================================================================

#[test]
fn grammar_with_extras_valid() {
    let g = GrammarBuilder::new("ws")
        .token("NUMBER", r"\d+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn grammar_with_multiple_extras() {
    let g = GrammarBuilder::new("ws2")
        .token("NUMBER", r"\d+")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ===========================================================================
// Builder-specific validation
// ===========================================================================

#[test]
fn builder_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("multi_lhs")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    assert_eq!(r.stats.total_rules, 3);
}

#[test]
fn builder_token_reuse_across_rules() {
    let g = GrammarBuilder::new("reuse")
        .token("+", "+")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER", "+", "NUMBER"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn builder_indirect_recursion_valid() {
    // A -> B "x", B -> A "y" | "z" — has base case via B -> "z"
    let g = GrammarBuilder::new("indirect")
        .token("X", "x")
        .token("Y", "y")
        .token("Z", "z")
        .rule("a", vec!["b", "X"])
        .rule("b", vec!["a", "Y"])
        .rule("b", vec!["Z"])
        .start("a")
        .build();
    let r = validate(&g);
    // Should be productive because b has a base case
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. })),
        "should not have non-productive errors: {:?}",
        r.errors
    );
}

// ===========================================================================
// Validator default
// ===========================================================================

#[test]
fn validator_default_same_as_new() {
    let v1 = GrammarValidator::default();
    let v2 = GrammarValidator::new();
    let g = minimal_valid_grammar();
    let mut v1 = v1;
    let mut v2 = v2;
    let r1 = v1.validate(&g);
    let r2 = v2.validate(&g);
    assert_eq!(r1.errors, r2.errors);
    assert_eq!(r1.warnings, r2.warnings);
}
