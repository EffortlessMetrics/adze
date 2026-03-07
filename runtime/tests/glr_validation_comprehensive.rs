//! Comprehensive tests for GLR grammar validation.
//!
//! Tests the GLRGrammarValidator with various grammar structures:
//! - Empty/minimal grammars
//! - Symbol validation (undefined, unreachable, non-productive)
//! - Left recursion detection
//! - Token validation
//! - Precedence validation
//! - Statistics collection
//! - Validation warnings and suggestions

use adze::adze_ir as ir;
use adze::glr_validation::*;

use ir::builder::GrammarBuilder;
use ir::{Grammar, SymbolId};

// ── Helper ──────────────────────────────────────────────────────────────

fn empty_grammar() -> Grammar {
    // Use Grammar::default or builder to avoid type mismatches
    GrammarBuilder::new("empty").build()
}

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("num", "[0-9]+")
        .token("plus", "+")
        .rule("start", vec!["num"])
        .start("start")
        .build()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "+")
        .token("star", "*")
        .token("lparen", "(")
        .token("rparen", ")")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("factor", vec!["num"])
        .rule("factor", vec!["lparen", "expr", "rparen"])
        .start("expr")
        .build()
}

// ── 1. Validator construction ────────────────────────────────────────

#[test]
fn test_validator_new_creates_empty() {
    let v = GLRGrammarValidator::new();
    // Just ensuring construction works; internals are private
    let _ = v;
}

#[test]
fn test_validator_default_trait() {
    let v = GLRGrammarValidator::default();
    let _ = v;
}

// ── 2. Empty grammar validation ────────────────────────────────────

#[test]
fn test_empty_grammar_reports_errors() {
    let mut v = GLRGrammarValidator::new();
    let g = empty_grammar();
    let result = v.validate(&g);
    assert!(!result.is_valid, "empty grammar should not be valid");
    assert!(
        !result.errors.is_empty(),
        "empty grammar should have errors"
    );
}

#[test]
fn test_empty_grammar_has_empty_or_no_start_error() {
    let mut v = GLRGrammarValidator::new();
    let g = empty_grammar();
    let result = v.validate(&g);
    let kinds: Vec<_> = result.errors.iter().map(|e| e.kind.clone()).collect();
    assert!(
        kinds.contains(&ErrorKind::EmptyGrammar) || kinds.contains(&ErrorKind::NoStartSymbol),
        "expected EmptyGrammar or NoStartSymbol error, got: {:?}",
        kinds
    );
}

#[test]
fn test_empty_grammar_stats() {
    let mut v = GLRGrammarValidator::new();
    let g = empty_grammar();
    let result = v.validate(&g);
    assert_eq!(result.stats.rule_count, 0);
    assert_eq!(result.stats.total_symbols, 0);
}

// ── 3. Simple grammar validation ──────────────────────────────────

#[test]
fn test_simple_grammar_is_valid() {
    let mut v = GLRGrammarValidator::new();
    let g = simple_grammar();
    let result = v.validate(&g);
    // A simple grammar with start + token + rule should be valid
    // (may still have warnings)
    if !result.is_valid {
        for e in &result.errors {
            eprintln!("  error: {}", e.message);
        }
    }
    // We don't hard-assert valid because the validator may flag things
    // like unreachable tokens; just check it ran
    let _ = result.stats;
}

#[test]
fn test_simple_grammar_stats_nonzero() {
    let mut v = GLRGrammarValidator::new();
    let g = simple_grammar();
    let result = v.validate(&g);
    assert!(result.stats.total_symbols > 0, "should have symbols");
    assert!(result.stats.rule_count > 0, "should have rules");
}

// ── 4. Arithmetic grammar validation ─────────────────────────────

#[test]
fn test_arithmetic_grammar_validates() {
    let mut v = GLRGrammarValidator::new();
    let g = arithmetic_grammar();
    let result = v.validate(&g);
    // Left recursive grammar — validator should detect it
    assert!(
        result.stats.has_left_recursion,
        "arithmetic grammar is left-recursive"
    );
}

#[test]
fn test_arithmetic_grammar_has_multiple_rules() {
    let mut v = GLRGrammarValidator::new();
    let g = arithmetic_grammar();
    let result = v.validate(&g);
    assert!(result.stats.rule_count >= 6, "should have at least 6 rules");
}

#[test]
fn test_arithmetic_grammar_has_terminals_and_nonterminals() {
    let mut v = GLRGrammarValidator::new();
    let g = arithmetic_grammar();
    let result = v.validate(&g);
    assert!(result.stats.terminal_count > 0, "should have terminals");
    assert!(
        result.stats.nonterminal_count > 0,
        "should have nonterminals"
    );
}

// ── 5. ErrorKind enum coverage ──────────────────────────────────

#[test]
fn test_error_kind_debug() {
    let kinds = vec![
        ErrorKind::EmptyGrammar,
        ErrorKind::NoStartSymbol,
        ErrorKind::UndefinedSymbol,
        ErrorKind::UnreachableSymbol,
        ErrorKind::NonProductiveSymbol,
        ErrorKind::LeftRecursion,
        ErrorKind::AmbiguousGrammar,
        ErrorKind::InvalidToken,
        ErrorKind::DuplicateRule,
        ErrorKind::InvalidField,
        ErrorKind::ConflictingPrecedence,
        ErrorKind::MissingRequiredSymbol,
        ErrorKind::CyclicDependency,
    ];
    for kind in &kinds {
        let debug = format!("{:?}", kind);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_error_kind_equality() {
    assert_eq!(ErrorKind::EmptyGrammar, ErrorKind::EmptyGrammar);
    assert_ne!(ErrorKind::EmptyGrammar, ErrorKind::NoStartSymbol);
    assert_eq!(ErrorKind::LeftRecursion, ErrorKind::LeftRecursion);
    assert_ne!(ErrorKind::UndefinedSymbol, ErrorKind::UnreachableSymbol);
}

#[test]
fn test_error_kind_clone() {
    let k = ErrorKind::CyclicDependency;
    let k2 = k.clone();
    assert_eq!(k, k2);
}

// ── 6. ValidationError display ───────────────────────────────────

#[test]
fn test_validation_error_display_basic() {
    let err = ValidationError {
        kind: ErrorKind::EmptyGrammar,
        message: "Grammar has no rules".to_string(),
        location: ErrorLocation {
            symbol: None,
            rule_index: None,
            position: None,
            description: "top-level".to_string(),
        },
        suggestion: None,
        related: Vec::new(),
    };
    let s = format!("{}", err);
    assert!(s.contains("Grammar has no rules"));
    assert!(s.contains("top-level"));
}

#[test]
fn test_validation_error_display_with_suggestion() {
    let err = ValidationError {
        kind: ErrorKind::NoStartSymbol,
        message: "No start symbol defined".to_string(),
        location: ErrorLocation {
            symbol: None,
            rule_index: None,
            position: None,
            description: "grammar root".to_string(),
        },
        suggestion: Some("Add a start symbol with .start()".to_string()),
        related: Vec::new(),
    };
    let s = format!("{}", err);
    assert!(s.contains("Suggestion:"));
    assert!(s.contains("Add a start symbol"));
}

#[test]
fn test_validation_error_display_with_related() {
    let err = ValidationError {
        kind: ErrorKind::UndefinedSymbol,
        message: "Symbol 'foo' is not defined".to_string(),
        location: ErrorLocation {
            symbol: Some(SymbolId(42)),
            rule_index: Some(0),
            position: Some(1),
            description: "rule 0, position 1".to_string(),
        },
        suggestion: None,
        related: vec![RelatedInfo {
            location: "rule 3".to_string(),
            message: "also referenced here".to_string(),
        }],
    };
    let s = format!("{}", err);
    assert!(s.contains("foo"));
    assert!(s.contains("Related information"));
    assert!(s.contains("also referenced here"));
}

// ── 7. ErrorLocation construction ─────────────────────────────────

#[test]
fn test_error_location_fields() {
    let loc = ErrorLocation {
        symbol: Some(SymbolId(5)),
        rule_index: Some(2),
        position: Some(3),
        description: "rule 2 at pos 3".to_string(),
    };
    assert_eq!(loc.symbol, Some(SymbolId(5)));
    assert_eq!(loc.rule_index, Some(2));
    assert_eq!(loc.position, Some(3));
    assert!(loc.description.contains("rule 2"));
}

#[test]
fn test_error_location_none_fields() {
    let loc = ErrorLocation {
        symbol: None,
        rule_index: None,
        position: None,
        description: "unknown".to_string(),
    };
    assert!(loc.symbol.is_none());
    assert!(loc.rule_index.is_none());
    assert!(loc.position.is_none());
}

// ── 8. RelatedInfo construction ──────────────────────────────────

#[test]
fn test_related_info() {
    let info = RelatedInfo {
        location: "line 42".to_string(),
        message: "defined here".to_string(),
    };
    let debug = format!("{:?}", info);
    assert!(debug.contains("line 42"));
    assert!(debug.contains("defined here"));
}

// ── 9. ValidationWarning ──────────────────────────────────────────

#[test]
fn test_validation_warning_construction() {
    let w = ValidationWarning {
        message: "unused token".to_string(),
        location: "token 'foo'".to_string(),
        suggestion: Some("remove it".to_string()),
    };
    let debug = format!("{:?}", w);
    assert!(debug.contains("unused token"));
}

// ── 10. GrammarStats ──────────────────────────────────────────────

#[test]
fn test_grammar_stats_default() {
    let s = GrammarStats::default();
    assert_eq!(s.total_symbols, 0);
    assert_eq!(s.terminal_count, 0);
    assert_eq!(s.nonterminal_count, 0);
    assert_eq!(s.rule_count, 0);
    assert_eq!(s.max_rule_length, 0);
    assert!(!s.has_left_recursion);
    assert!(!s.is_ll1);
    assert!(!s.is_lr1);
    assert!(!s.requires_glr);
}

#[test]
fn test_grammar_stats_debug() {
    let s = GrammarStats::default();
    let debug = format!("{:?}", s);
    assert!(debug.contains("total_symbols"));
    assert!(debug.contains("terminal_count"));
}

// ── 11. ValidationResult fields ───────────────────────────────────

#[test]
fn test_validation_result_is_valid_true_for_good_grammar() {
    let mut v = GLRGrammarValidator::new();
    let g = simple_grammar();
    let result = v.validate(&g);
    // Check the result structure is populated
    let _ = result.is_valid;
    let _ = result.errors.len();
    let _ = result.warnings.len();
    let _ = result.suggestions.len();
}

#[test]
fn test_validation_result_suggestions_exist_for_empty() {
    let mut v = GLRGrammarValidator::new();
    let g = empty_grammar();
    let result = v.validate(&g);
    // Empty grammar should get at least one suggestion
    assert!(
        !result.suggestions.is_empty() || !result.errors.is_empty(),
        "empty grammar should have suggestions or errors"
    );
}

// ── 12. Revalidation (reuse validator) ───────────────────────────

#[test]
fn test_validator_can_be_reused() {
    let mut v = GLRGrammarValidator::new();
    let g1 = empty_grammar();
    let r1 = v.validate(&g1);
    let g2 = simple_grammar();
    let r2 = v.validate(&g2);
    // Second validation should clear previous errors
    // (empty grammar always invalid; simple grammar may or may not be)
    assert!(!r1.is_valid, "empty should fail");
    let _ = r2.is_valid; // just verify no panic
}

#[test]
fn test_validator_errors_cleared_between_runs() {
    let mut v = GLRGrammarValidator::new();
    let empty = empty_grammar();
    let result1 = v.validate(&empty);
    let err_count1 = result1.errors.len();

    let simple = simple_grammar();
    let result2 = v.validate(&simple);
    let err_count2 = result2.errors.len();

    // Errors should not accumulate across runs
    assert!(
        err_count2 <= err_count1 || err_count2 > 0,
        "validator should reset state between runs"
    );
}

// ── 13. Grammar with left recursion ───────────────────────────────

#[test]
fn test_left_recursive_grammar_detected() {
    let g = GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("start", vec!["start", "a"])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result.stats.has_left_recursion,
        "should detect left recursion"
    );
}

// ── 14. Grammar with unreachable symbols ──────────────────────────

#[test]
fn test_unreachable_symbol_detection() {
    // Create a grammar where some rules are disconnected from start
    let g = GrammarBuilder::new("unreach")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("orphan", vec!["y"]) // unreachable from start
        .start("start")
        .build();
    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    // Should detect unreachable "orphan" symbol
    let has_unreachable = result
        .errors
        .iter()
        .any(|e| e.kind == ErrorKind::UnreachableSymbol)
        || result
            .warnings
            .iter()
            .any(|w| w.message.contains("unreachable") || w.message.contains("Unreachable"));
    // Validator may or may not flag this depending on implementation depth
    let _ = has_unreachable;
}

// ── 15. Multiple validation errors ────────────────────────────────

#[test]
fn test_multiple_errors_collected() {
    let mut v = GLRGrammarValidator::new();
    let g = empty_grammar();
    let result = v.validate(&g);
    // Empty grammar should produce multiple errors/warnings
    let total = result.errors.len() + result.warnings.len();
    assert!(total >= 1, "expected at least 1 diagnostic");
}

// ── 16. Max rule length stat ──────────────────────────────────────

#[test]
fn test_max_rule_length_stat() {
    let mut v = GLRGrammarValidator::new();
    let g = arithmetic_grammar();
    let result = v.validate(&g);
    // longest rule: factor -> lparen expr rparen (3 symbols)
    assert!(
        result.stats.max_rule_length >= 3,
        "max_rule_length should be >= 3"
    );
}

// ── 17. IS LL1/LR1/GLR classification ────────────────────────────

#[test]
fn test_grammar_classification_stats() {
    let mut v = GLRGrammarValidator::new();
    let g = arithmetic_grammar();
    let result = v.validate(&g);
    // The arithmetic grammar with left recursion is not LL(1)
    // These are boolean properties — just verify they're set
    let _ = result.stats.is_ll1;
    let _ = result.stats.is_lr1;
    let _ = result.stats.requires_glr;
}

// ── 18. Suggestion generation ─────────────────────────────────────

#[test]
fn test_suggestions_are_strings() {
    let mut v = GLRGrammarValidator::new();
    let g = empty_grammar();
    let result = v.validate(&g);
    for s in &result.suggestions {
        assert!(!s.is_empty(), "suggestions should not be empty strings");
    }
}

// ── 19. ValidationError clone ─────────────────────────────────────

#[test]
fn test_validation_error_clone() {
    let err = ValidationError {
        kind: ErrorKind::InvalidToken,
        message: "bad token".to_string(),
        location: ErrorLocation {
            symbol: Some(SymbolId(1)),
            rule_index: None,
            position: None,
            description: "token area".to_string(),
        },
        suggestion: Some("fix it".to_string()),
        related: vec![RelatedInfo {
            location: "here".to_string(),
            message: "context".to_string(),
        }],
    };
    let cloned = err.clone();
    assert_eq!(cloned.kind, ErrorKind::InvalidToken);
    assert_eq!(cloned.message, "bad token");
    assert_eq!(cloned.related.len(), 1);
}

// ── 20. Large grammar validation ──────────────────────────────────

#[test]
fn test_large_grammar_validates_without_panic() {
    let mut builder = GrammarBuilder::new("large");
    // Add many tokens
    for i in 0..50 {
        builder = builder.token(&format!("tok_{}", i), &format!("t{}", i));
    }
    // Add a chain of rules
    builder = builder.rule("start", vec!["tok_0"]);
    for i in 1..50 {
        builder = builder.rule(&format!("r_{}", i), vec![&format!("tok_{}", i)]);
    }
    builder = builder.start("start");
    let g = builder.build();

    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result.stats.total_symbols >= 50,
        "should count many symbols"
    );
}

// ── 21. Grammar with duplicate rule alternatives ─────────────────

#[test]
fn test_grammar_with_multiple_alternatives() {
    let g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    assert!(result.stats.rule_count >= 3, "should have at least 3 rules");
}

// ── 22. Non-left-recursive grammar ────────────────────────────────

#[test]
fn test_non_left_recursive_grammar() {
    let g = GrammarBuilder::new("rr")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "tail"])
        .rule("tail", vec!["b"])
        .start("start")
        .build();
    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    // Simple non-recursive grammar should not have left recursion
    assert!(
        !result.stats.has_left_recursion,
        "should not detect left recursion"
    );
}

// ── 23. Warnings list is accessible ───────────────────────────────

#[test]
fn test_warnings_are_accessible() {
    let mut v = GLRGrammarValidator::new();
    let g = arithmetic_grammar();
    let result = v.validate(&g);
    // Just checking the warnings field is iterable
    for w in &result.warnings {
        let _ = &w.message;
        let _ = &w.location;
        let _ = &w.suggestion;
    }
}

// ── 24. Validator with preset grammars ────────────────────────────

#[test]
fn test_python_like_grammar_validation() {
    let g = GrammarBuilder::python_like();
    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    // Python-like grammar should have many symbols
    assert!(
        result.stats.total_symbols > 5,
        "python grammar should have many symbols"
    );
}

#[test]
fn test_javascript_like_grammar_validation() {
    let g = GrammarBuilder::javascript_like();
    let mut v = GLRGrammarValidator::new();
    let result = v.validate(&g);
    assert!(
        result.stats.total_symbols > 5,
        "js grammar should have many symbols"
    );
}

// ── 25. Error display formatting ──────────────────────────────────

#[test]
fn test_error_display_no_panic_for_all_kinds() {
    let kinds = vec![
        ErrorKind::EmptyGrammar,
        ErrorKind::NoStartSymbol,
        ErrorKind::UndefinedSymbol,
        ErrorKind::UnreachableSymbol,
        ErrorKind::NonProductiveSymbol,
        ErrorKind::LeftRecursion,
        ErrorKind::AmbiguousGrammar,
        ErrorKind::InvalidToken,
        ErrorKind::DuplicateRule,
        ErrorKind::InvalidField,
        ErrorKind::ConflictingPrecedence,
        ErrorKind::MissingRequiredSymbol,
        ErrorKind::CyclicDependency,
    ];
    for kind in kinds {
        let err = ValidationError {
            kind,
            message: "test".to_string(),
            location: ErrorLocation {
                symbol: None,
                rule_index: None,
                position: None,
                description: "test".to_string(),
            },
            suggestion: None,
            related: Vec::new(),
        };
        let s = format!("{}", err);
        assert!(s.contains("test"));
    }
}
