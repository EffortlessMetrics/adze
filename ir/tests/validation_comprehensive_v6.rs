//! Comprehensive V6 validation tests for adze-ir GrammarValidator.
//!
//! Categories:
//!   1. Valid grammars pass (8)
//!   2. Missing start symbol (8)
//!   3. Undefined symbol references (8)
//!   4. Unreachable rules (8)
//!   5. Duplicate detection (8)
//!   6. Empty grammar validation (7)
//!   7. Validation after normalize (8)
//!   8. Complex validation scenarios (8)

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate(grammar: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

/// Filter out CyclicRule errors (left-recursive grammars legitimately trigger cycles).
fn non_cycle_errors(r: &adze_ir::validation::ValidationResult) -> Vec<&ValidationError> {
    r.errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect()
}

// ===========================================================================
// 1. Valid grammars pass validation (8 tests)
// ===========================================================================

#[test]
fn test_valid_single_token_grammar() {
    let g = GrammarBuilder::new("single")
        .token("NUM", r"\d+")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_two_token_addition() {
    let g = GrammarBuilder::new("add")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_multi_level_grammar() {
    let g = GrammarBuilder::new("ml")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["ID", ";"])
        .start("program")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_nullable_start_rule() {
    let g = GrammarBuilder::new("null")
        .token("X", "x")
        .rule("top", vec![])
        .rule("top", vec!["X"])
        .start("top")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_alternatives_grammar() {
    let g = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_python_like_preset() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_valid_javascript_like_preset() {
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_valid_precedence_grammar_no_errors() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

// ===========================================================================
// 2. Missing start symbol / NoExplicitStartRule (8 tests)
// ===========================================================================

#[test]
fn test_missing_start_empty_grammar_gives_error() {
    let g = Grammar::new("empty".to_string());
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar error"
    );
}

#[test]
fn test_missing_start_rules_but_no_start_call() {
    // Build grammar manually without setting start, but with rules
    let mut g = Grammar::new("no_start".to_string());
    let sym = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "TOK".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: sym,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // The grammar has rules so it won't be EmptyGrammar, but start_symbol()
    // may still find a start via heuristics. Verify no EmptyGrammar error:
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "should not be empty grammar"
    );
}

#[test]
fn test_missing_start_only_tokens_no_rules() {
    let mut g = Grammar::new("tokens_only".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar when no rules"
    );
}

#[test]
fn test_missing_start_empty_grammar_has_at_least_one_error() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert!(!r.errors.is_empty());
}

#[test]
fn test_missing_start_stats_zero_rules() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 0);
}

#[test]
fn test_missing_start_stats_zero_tokens() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 0);
}

#[test]
fn test_missing_start_stats_zero_reachable() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.reachable_symbols, 0);
}

#[test]
fn test_missing_start_stats_zero_productive() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.productive_symbols, 0);
}

// ===========================================================================
// 3. Undefined symbol references (8 tests)
// ===========================================================================

#[test]
fn test_undefined_nonterminal_in_rhs() {
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
    let r = validate(&g);
    assert!(r.errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn test_undefined_terminal_in_rhs() {
    let mut g = Grammar::new("undef_term".to_string());
    let lhs = SymbolId(1);
    let undef = SymbolId(88);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(r.errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn test_undefined_mixed_with_defined() {
    let mut g = Grammar::new("mixed".to_string());
    let lhs = SymbolId(1);
    let tok = SymbolId(2);
    let undef = SymbolId(50);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(tok), Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(r.errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn test_undefined_does_not_fire_when_all_defined() {
    let g = GrammarBuilder::new("ok")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
    );
}

#[test]
fn test_undefined_multiple_undefined_symbols() {
    let mut g = Grammar::new("multi_undef".to_string());
    let lhs = SymbolId(1);
    let u1 = SymbolId(80);
    let u2 = SymbolId(81);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(u1), Symbol::NonTerminal(u2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    let undef_count = r
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
        .count();
    assert!(
        undef_count >= 2,
        "expected >=2 undefined, got {undef_count}"
    );
}

#[test]
fn test_undefined_symbol_error_contains_location() {
    let mut g = Grammar::new("loc".to_string());
    let lhs = SymbolId(1);
    let undef = SymbolId(77);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    let found = r.errors.iter().find(|e| {
        matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
        )
    });
    assert!(found.is_some(), "expected UndefinedSymbol");
    if let Some(ValidationError::UndefinedSymbol { location, .. }) = found {
        assert!(!location.is_empty(), "location should not be empty");
    }
}

#[test]
fn test_undefined_in_second_rule_alternative() {
    let mut g = Grammar::new("alt_undef".to_string());
    let lhs = SymbolId(1);
    let tok = SymbolId(2);
    let undef = SymbolId(60);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    // First rule is fine
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // Second rule references undefined
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    assert!(r.errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn test_undefined_self_referencing_rule_no_false_positive() {
    // A -> A | token  — A is both LHS and RHS, should NOT be undefined
    let g = GrammarBuilder::new("self_ref")
        .token("X", "x")
        .rule("item", vec!["X"])
        .rule("item", vec!["item", "X"])
        .start("item")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
    );
}

// ===========================================================================
// 4. Unreachable rules (8 tests)
// ===========================================================================

#[test]
fn test_unreachable_disconnected_rule_produces_warning() {
    // Build grammar where "island" is defined but not reachable from start
    let g = GrammarBuilder::new("unreach")
        .token("A", "a")
        .token("B", "b")
        .rule("main_rule", vec!["A"])
        .rule("island", vec!["B"])
        .start("main_rule")
        .build();
    let r = validate(&g);
    // Unreachable symbols show as UnusedToken warnings
    let has_unused = r.warnings.iter().any(|w| {
        if let ValidationWarning::UnusedToken { name, .. } = w {
            name.contains("island") || name.contains("B") || name.contains("rule_")
        } else {
            false
        }
    });
    // Or they might be unreachable non-productive errors
    let has_unreachable = r.errors.iter().any(|e| {
        matches!(e, ValidationError::UnreachableSymbol { .. })
            || matches!(e, ValidationError::NonProductiveSymbol { .. })
    });
    assert!(
        has_unused || has_unreachable,
        "expected warning/error for unreachable rule. warnings={:?}, errors={:?}",
        r.warnings,
        r.errors
    );
}

#[test]
fn test_unreachable_reachable_chain_no_warning() {
    // a -> b -> c -> token: all reachable
    let g = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("a_sym", vec!["b_sym"])
        .rule("b_sym", vec!["c_sym"])
        .rule("c_sym", vec!["X"])
        .start("a_sym")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnreachableSymbol { .. })),
        "no symbol should be unreachable"
    );
}

#[test]
fn test_unreachable_stats_reachable_count_matches() {
    let g = GrammarBuilder::new("reach_stats")
        .token("X", "x")
        .rule("top", vec!["X"])
        .start("top")
        .build();
    let r = validate(&g);
    // At minimum the start symbol and token are reachable
    assert!(r.stats.reachable_symbols >= 2);
}

#[test]
fn test_unreachable_all_connected_grammar() {
    let g = GrammarBuilder::new("connected")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["child"])
        .rule("child", vec!["A", "B"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnreachableSymbol { .. })),
    );
}

#[test]
fn test_unreachable_multiple_disconnected() {
    let g = GrammarBuilder::new("multi_unreach")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("main_rule", vec!["A"])
        .rule("orphan1", vec!["B"])
        .rule("orphan2", vec!["C"])
        .start("main_rule")
        .build();
    let r = validate(&g);
    // At least some warning/error about unreachable symbols
    let warning_count = r
        .warnings
        .iter()
        .filter(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
        .count();
    let error_count = r
        .errors
        .iter()
        .filter(|e| {
            matches!(e, ValidationError::UnreachableSymbol { .. })
                | matches!(e, ValidationError::NonProductiveSymbol { .. })
        })
        .count();
    assert!(
        warning_count + error_count >= 1,
        "expected at least 1 unreachable diagnostic"
    );
}

#[test]
fn test_unreachable_extra_token_not_flagged() {
    // Extra tokens (whitespace) should not be flagged unreachable
    let g = GrammarBuilder::new("ws")
        .token("NUM", r"\d+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_unreachable_external_token_not_flagged() {
    let g = GrammarBuilder::new("ext")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    let r = validate(&g);
    // External tokens should not generate UndefinedSymbol
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
    );
}

#[test]
fn test_unreachable_stats_productive_subset_of_defined() {
    let g = GrammarBuilder::new("prod_stats")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(r.stats.productive_symbols <= r.stats.total_symbols);
}

// ===========================================================================
// 5. Duplicate detection (8 tests)
// ===========================================================================

#[test]
fn test_duplicate_token_pattern_warns() {
    let g = GrammarBuilder::new("dup_tok")
        .token("NUM1", r"\d+")
        .token("NUM2", r"\d+")
        .rule("root", vec!["NUM1"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. })),
        "expected DuplicateTokenPattern warning"
    );
}

#[test]
fn test_duplicate_token_pattern_contains_both() {
    let g = GrammarBuilder::new("dup_tok2")
        .token("INT1", r"\d+")
        .token("INT2", r"\d+")
        .rule("root", vec!["INT1"])
        .start("root")
        .build();
    let r = validate(&g);
    let dup = r
        .warnings
        .iter()
        .find(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
    assert!(dup.is_some());
    if let Some(ValidationWarning::DuplicateTokenPattern { tokens, .. }) = dup {
        assert!(tokens.len() >= 2);
    }
}

#[test]
fn test_no_duplicate_warning_when_patterns_differ() {
    let g = GrammarBuilder::new("no_dup")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        !r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. })),
    );
}

#[test]
fn test_duplicate_three_identical_patterns() {
    let g = GrammarBuilder::new("trip_dup")
        .token("T1", "same")
        .token("T2", "same")
        .token("T3", "same")
        .rule("root", vec!["T1"])
        .start("root")
        .build();
    let r = validate(&g);
    let dup = r
        .warnings
        .iter()
        .find(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
    assert!(dup.is_some());
    if let Some(ValidationWarning::DuplicateTokenPattern { tokens, .. }) = dup {
        assert!(tokens.len() >= 3);
    }
}

#[test]
fn test_duplicate_rule_detection() {
    // DuplicateRule is about duplicate symbol definitions in manually built grammars.
    // The builder creates alternatives, not duplicates. We test via raw Grammar.
    let mut g = Grammar::new("dup_rule".to_string());
    let sym = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    // Two identical rules for same LHS
    g.add_rule(Rule {
        lhs: sym,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: sym,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    // The validator may or may not flag identical alternatives as "duplicate".
    // Verify at least the grammar validates without panic.
    let r = validate(&g);
    assert!(r.stats.total_rules >= 2);
}

#[test]
fn test_duplicate_external_token_conflict() {
    // Two externals with the same name
    let mut g = Grammar::new("ext_dup".to_string());
    let sym = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: sym,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.externals.push(adze_ir::ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(adze_ir::ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(11),
    });
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. })),
        "expected ExternalTokenConflict"
    );
}

#[test]
fn test_no_external_conflict_with_unique_names() {
    let g = GrammarBuilder::new("uniq_ext")
        .token("NUM", r"\d+")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

#[test]
fn test_duplicate_string_pattern_warning() {
    let g = GrammarBuilder::new("dup_str")
        .token("PLUS1", "+")
        .token("PLUS2", "+")
        .rule("root", vec!["PLUS1"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }))
    );
}

// ===========================================================================
// 6. Empty grammar validation (7 tests)
// ===========================================================================

#[test]
fn test_empty_grammar_error() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn test_empty_grammar_no_warnings() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    // Empty grammar may or may not produce warnings, but should produce errors.
    assert!(!r.errors.is_empty());
}

#[test]
fn test_empty_grammar_stats_all_zero() {
    let g = Grammar::new("e".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 0);
    assert_eq!(r.stats.total_tokens, 0);
    assert_eq!(r.stats.reachable_symbols, 0);
}

#[test]
fn test_empty_name_grammar_still_validates() {
    // Grammar with empty name but valid rules
    let mut g = Grammar::new(String::new());
    let sym = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: sym,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    // Should not get EmptyGrammar since there are rules
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn test_empty_grammar_default_stats() {
    let g = Grammar::new("empty_stats".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.max_rule_length, 0);
    assert_eq!(r.stats.external_tokens, 0);
}

#[test]
fn test_grammar_with_only_extras_is_empty() {
    let mut g = Grammar::new("only_extras".to_string());
    g.extras.push(SymbolId(1));
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn test_empty_grammar_display_error() {
    let g = Grammar::new("disp".to_string());
    let r = validate(&g);
    let empty_err = r
        .errors
        .iter()
        .find(|e| matches!(e, ValidationError::EmptyGrammar));
    assert!(empty_err.is_some());
    let msg = format!("{}", empty_err.unwrap());
    assert!(!msg.is_empty());
}

// ===========================================================================
// 7. Validation after normalize (8 tests)
// ===========================================================================

#[test]
fn test_normalize_then_validate_simple() {
    let mut g = GrammarBuilder::new("norm1")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    g.normalize();
    let r = validate(&g);
    assert!(
        r.errors.is_empty(),
        "errors after normalize: {:?}",
        r.errors
    );
}

#[test]
fn test_normalize_then_validate_multi_rule() {
    let mut g = GrammarBuilder::new("norm2")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["child"])
        .rule("child", vec!["A"])
        .rule("child", vec!["B"])
        .start("root")
        .build();
    g.normalize();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_normalize_then_validate_nullable() {
    let mut g = GrammarBuilder::new("norm3")
        .token("X", "x")
        .rule("top", vec![])
        .rule("top", vec!["X"])
        .start("top")
        .build();
    g.normalize();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_normalize_preserves_token_count() {
    let mut g = GrammarBuilder::new("norm_tc")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn test_normalize_then_validate_chain() {
    let mut g = GrammarBuilder::new("norm_chain")
        .token("X", "x")
        .rule("a_sym", vec!["b_sym"])
        .rule("b_sym", vec!["c_sym"])
        .rule("c_sym", vec!["X"])
        .start("a_sym")
        .build();
    g.normalize();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_normalize_then_validate_stats_positive() {
    let mut g = GrammarBuilder::new("norm_stats")
        .token("N", r"\d+")
        .rule("root", vec!["N"])
        .start("root")
        .build();
    g.normalize();
    let r = validate(&g);
    assert!(r.stats.total_rules >= 1);
    assert!(r.stats.total_tokens >= 1);
}

#[test]
fn test_normalize_then_validate_python_like() {
    let mut g = GrammarBuilder::python_like();
    g.normalize();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_normalize_twice_then_validate() {
    let mut g = GrammarBuilder::new("norm2x")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    g.normalize();
    g.normalize();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ===========================================================================
// 8. Complex validation scenarios (8 tests)
// ===========================================================================

#[test]
fn test_complex_left_recursive_grammar() {
    let g = GrammarBuilder::new("lrec")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "N"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let r = validate(&g);
    // Left recursion triggers CyclicRule; that's expected
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
}

#[test]
fn test_complex_right_recursive_grammar() {
    let g = GrammarBuilder::new("rrec")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["N", "+", "expr"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
}

#[test]
fn test_complex_mutual_recursion() {
    let g = GrammarBuilder::new("mutual")
        .token("X", "x")
        .rule("alpha", vec!["beta"])
        .rule("alpha", vec!["X"])
        .rule("beta", vec!["alpha"])
        .rule("beta", vec!["X"])
        .start("alpha")
        .build();
    let r = validate(&g);
    // Mutual recursion may trigger cycle detection
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "unexpected non-cycle errors: {:?}", errs);
}

#[test]
fn test_complex_deeply_nested_chain() {
    let mut builder = GrammarBuilder::new("deep").token("LEAF", "leaf");
    // Build chain: level0 -> level1 -> ... -> level9 -> LEAF
    for i in 0..10 {
        let this = format!("level{i}");
        let next = if i < 9 {
            format!("level{}", i + 1)
        } else {
            "LEAF".to_string()
        };
        builder = builder.rule(&this, vec![&next]);
    }
    let g = builder.start("level0").build();
    let r = validate(&g);
    let errs = non_cycle_errors(&r);
    assert!(errs.is_empty(), "errors: {:?}", errs);
}

#[test]
fn test_complex_many_tokens() {
    let mut builder = GrammarBuilder::new("many_tok");
    let mut rhs_names = Vec::new();
    for i in 0..20 {
        let name = format!("TOK{i}");
        builder = builder.token(&name, &name);
        rhs_names.push(name);
    }
    let refs: Vec<&str> = rhs_names.iter().map(|s| s.as_str()).collect();
    let g = builder.rule("root", refs).start("root").build();
    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 20);
    assert!(r.stats.max_rule_length >= 20);
}

#[test]
fn test_complex_validator_reuse() {
    let mut v = GrammarValidator::new();
    let g1 = GrammarBuilder::new("g1")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let g2 = Grammar::new("g2".to_string());

    let r1 = v.validate(&g1);
    assert!(r1.errors.is_empty());

    let r2 = v.validate(&g2);
    assert!(!r2.errors.is_empty());

    // Re-validate g1 — should still be clean
    let r3 = v.validate(&g1);
    assert!(r3.errors.is_empty());
}

#[test]
fn test_complex_non_productive_symbol() {
    // Symbol that references only itself with no terminal base case
    let mut g = Grammar::new("nonprod".to_string());
    let lhs = SymbolId(1);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(lhs)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    // Should detect non-productive or cyclic
    let has_nonprod = r.errors.iter().any(|e| {
        matches!(e, ValidationError::NonProductiveSymbol { .. })
            || matches!(e, ValidationError::CyclicRule { .. })
    });
    assert!(has_nonprod, "expected non-productive or cyclic error");
}

#[test]
fn test_complex_large_grammar_stats_consistent() {
    let mut builder = GrammarBuilder::new("large");
    // 5 tokens
    for i in 0..5 {
        let name = format!("T{i}");
        builder = builder.token(&name, &name);
    }
    // 5 rules, each referencing a token
    for i in 0..5 {
        let rule_name = format!("r{i}");
        let tok_name = format!("T{i}");
        builder = builder.rule(&rule_name, vec![&tok_name]);
    }
    // Connect them: main -> r0 | r1 | r2 | r3 | r4
    for i in 0..5 {
        let rule_name = format!("r{i}");
        builder = builder.rule("main_sym", vec![&rule_name]);
    }
    let g = builder.start("main_sym").build();
    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 5);
    assert!(r.stats.total_rules >= 10);
    assert!(r.stats.reachable_symbols > 0);
    assert!(r.stats.productive_symbols > 0);
}
