//! Comprehensive V3 tests for GrammarValidator.
//!
//! Covers: construction, simple/multi-token/alternative/chain grammars,
//! error/warning/stats access, reuse of validator, and post-normalize validation.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{
    GrammarValidator, ValidationError, ValidationResult, ValidationStats, ValidationWarning,
};
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate(grammar: &Grammar) -> ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("NUMBER", r"\d+")
        .rule("start", vec!["NUMBER"])
        .start("start")
        .build()
}

fn multi_token_grammar() -> Grammar {
    GrammarBuilder::new("multi_tok")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .start("expr")
        .build()
}

fn alternative_grammar() -> Grammar {
    GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .start("root")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["X"])
        .start("a")
        .build()
}

fn two_level_grammar() -> Grammar {
    GrammarBuilder::new("two_level")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["ID", ";"])
        .start("program")
        .build()
}

fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("A", "a")
        .rule("start", vec![])
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build()
}

fn keyword_grammar() -> Grammar {
    GrammarBuilder::new("kw")
        .token("IF", "if")
        .token("THEN", "then")
        .token("ELSE", "else")
        .token("TRUE", "true")
        .token("FALSE", "false")
        .rule("expr", vec!["TRUE"])
        .rule("expr", vec!["FALSE"])
        .rule("expr", vec!["IF", "expr", "THEN", "expr", "ELSE", "expr"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. GrammarValidator construction
// ===========================================================================

#[test]
fn validator_new_returns_instance() {
    let _v = GrammarValidator::new();
}

#[test]
fn validator_default_returns_instance() {
    let _v: GrammarValidator = Default::default();
}

#[test]
fn validator_new_can_validate_immediately() {
    let mut v = GrammarValidator::new();
    let _r = v.validate(&minimal_grammar());
}

// ===========================================================================
// 2. Validate simple grammar — no errors
// ===========================================================================

#[test]
fn simple_grammar_no_errors() {
    let r = validate(&minimal_grammar());
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn simple_grammar_stats_total_tokens() {
    let r = validate(&minimal_grammar());
    assert_eq!(r.stats.total_tokens, 1);
}

#[test]
fn simple_grammar_stats_total_rules() {
    let r = validate(&minimal_grammar());
    assert!(r.stats.total_rules >= 1);
}

#[test]
fn simple_grammar_stats_reachable_positive() {
    let r = validate(&minimal_grammar());
    assert!(r.stats.reachable_symbols > 0);
}

#[test]
fn simple_grammar_stats_productive_positive() {
    let r = validate(&minimal_grammar());
    assert!(r.stats.productive_symbols > 0);
}

#[test]
fn simple_grammar_max_rule_length() {
    let r = validate(&minimal_grammar());
    assert!(r.stats.max_rule_length >= 1);
}

#[test]
fn simple_grammar_avg_rule_length_positive() {
    let r = validate(&minimal_grammar());
    assert!(r.stats.avg_rule_length > 0.0);
}

// ===========================================================================
// 3. Validate multi-token grammar
// ===========================================================================

#[test]
fn multi_token_no_fatal_errors() {
    let r = validate(&multi_token_grammar());
    // Recursive rules produce CyclicRule — filter those out
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "unexpected errors: {:?}", non_cyclic);
}

#[test]
fn multi_token_stats_four_tokens() {
    let r = validate(&multi_token_grammar());
    assert_eq!(r.stats.total_tokens, 4);
}

#[test]
fn multi_token_stats_rules_at_least_four() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.total_rules >= 4);
}

#[test]
fn multi_token_max_rule_length_three() {
    let r = validate(&multi_token_grammar());
    assert_eq!(r.stats.max_rule_length, 3);
}

#[test]
fn multi_token_reachable_covers_all_symbols() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.reachable_symbols >= r.stats.total_tokens);
}

#[test]
fn multi_token_productive_symbols_positive() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.productive_symbols > 0);
}

#[test]
fn multi_token_avg_rule_length_between_1_and_3() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.avg_rule_length >= 1.0);
    assert!(r.stats.avg_rule_length <= 3.0);
}

// ===========================================================================
// 4. Validate alternative grammar
// ===========================================================================

#[test]
fn alternative_grammar_no_errors() {
    let r = validate(&alternative_grammar());
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn alternative_grammar_three_tokens() {
    let r = validate(&alternative_grammar());
    assert_eq!(r.stats.total_tokens, 3);
}

#[test]
fn alternative_grammar_three_rules() {
    let r = validate(&alternative_grammar());
    assert_eq!(r.stats.total_rules, 3);
}

#[test]
fn alternative_grammar_max_rule_length_one() {
    let r = validate(&alternative_grammar());
    assert_eq!(r.stats.max_rule_length, 1);
}

#[test]
fn alternative_grammar_avg_rule_length_one() {
    let r = validate(&alternative_grammar());
    assert!((r.stats.avg_rule_length - 1.0).abs() < f64::EPSILON);
}

#[test]
fn alternative_grammar_all_symbols_productive() {
    let r = validate(&alternative_grammar());
    assert!(r.stats.productive_symbols >= r.stats.total_symbols);
}

// ===========================================================================
// 5. Validate chain grammar
// ===========================================================================

#[test]
fn chain_grammar_no_non_cyclic_errors() {
    let r = validate(&chain_grammar());
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "unexpected errors: {:?}", non_cyclic);
}

#[test]
fn chain_grammar_one_token() {
    let r = validate(&chain_grammar());
    assert_eq!(r.stats.total_tokens, 1);
}

#[test]
fn chain_grammar_three_rules() {
    let r = validate(&chain_grammar());
    assert_eq!(r.stats.total_rules, 3);
}

#[test]
fn chain_grammar_all_reachable() {
    let r = validate(&chain_grammar());
    assert!(r.stats.reachable_symbols >= 3);
}

#[test]
fn chain_grammar_max_rule_length_one() {
    let r = validate(&chain_grammar());
    assert_eq!(r.stats.max_rule_length, 1);
}

#[test]
fn chain_grammar_productive_symbols_positive() {
    let r = validate(&chain_grammar());
    assert!(r.stats.productive_symbols > 0);
}

// ===========================================================================
// 6. ValidationResult.errors is empty for valid grammars
// ===========================================================================

#[test]
fn two_level_grammar_errors_empty() {
    let r = validate(&two_level_grammar());
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn nullable_grammar_errors_empty() {
    let r = validate(&nullable_grammar());
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn keyword_grammar_errors_empty() {
    let r = validate(&keyword_grammar());
    // Recursive grammar may produce CyclicRule
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn python_like_no_fatal_non_cyclic_errors() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn javascript_like_no_fatal_non_cyclic_errors() {
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn empty_grammar_has_errors() {
    let g = GrammarBuilder::new("empty").build();
    let r = validate(&g);
    assert!(!r.errors.is_empty());
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

// ===========================================================================
// 7. ValidationResult.warnings access
// ===========================================================================

#[test]
fn minimal_grammar_warnings_is_vec() {
    let r = validate(&minimal_grammar());
    let _: &Vec<ValidationWarning> = &r.warnings;
}

#[test]
fn alternative_grammar_warnings_len() {
    let r = validate(&alternative_grammar());
    let _ = r.warnings.len();
}

#[test]
fn warnings_can_be_iterated() {
    let r = validate(&minimal_grammar());
    for _w in &r.warnings {
        // access each warning
    }
}

#[test]
fn warnings_can_be_counted() {
    let r = validate(&multi_token_grammar());
    let count = r.warnings.iter().count();
    assert!(count <= 100); // sanity bound
}

#[test]
fn unused_token_warning_detected() {
    // Build grammar with an extra token that no rule uses
    let g = GrammarBuilder::new("extra_tok")
        .token("USED", "u")
        .token("UNUSED", "v")
        .rule("start", vec!["USED"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
    );
}

#[test]
fn no_unused_token_warning_when_all_used() {
    let r = validate(&minimal_grammar());
    assert!(
        !r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
    );
}

// ===========================================================================
// 8. ValidationResult.stats access
// ===========================================================================

#[test]
fn stats_total_symbols_positive_for_valid_grammar() {
    let r = validate(&minimal_grammar());
    assert!(r.stats.total_symbols > 0);
}

#[test]
fn stats_external_tokens_zero_for_simple_grammar() {
    let r = validate(&minimal_grammar());
    assert_eq!(r.stats.external_tokens, 0);
}

#[test]
fn stats_external_tokens_for_python_like() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    assert!(r.stats.external_tokens >= 2);
}

#[test]
fn stats_total_tokens_matches_builder() {
    let r = validate(&alternative_grammar());
    assert_eq!(r.stats.total_tokens, 3);
}

#[test]
fn stats_max_rule_length_for_keyword_grammar() {
    let r = validate(&keyword_grammar());
    // IF expr THEN expr ELSE expr => length 6
    assert_eq!(r.stats.max_rule_length, 6);
}

#[test]
fn stats_avg_rule_length_is_finite() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.avg_rule_length.is_finite());
}

#[test]
fn stats_avg_rule_length_for_alternative_grammar() {
    let r = validate(&alternative_grammar());
    assert!((r.stats.avg_rule_length - 1.0).abs() < f64::EPSILON);
}

#[test]
fn stats_reachable_le_total() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.reachable_symbols <= r.stats.total_symbols);
}

#[test]
fn stats_productive_le_total() {
    let r = validate(&multi_token_grammar());
    assert!(r.stats.productive_symbols <= r.stats.total_symbols);
}

#[test]
fn stats_clone_is_equal() {
    let r = validate(&minimal_grammar());
    let s = r.stats.clone();
    assert_eq!(s.total_tokens, r.stats.total_tokens);
    assert_eq!(s.total_rules, r.stats.total_rules);
    assert_eq!(s.total_symbols, r.stats.total_symbols);
}

// ===========================================================================
// 9. Multiple validations on same validator
// ===========================================================================

#[test]
fn reuse_validator_two_grammars() {
    let mut v = GrammarValidator::new();
    let r1 = v.validate(&minimal_grammar());
    let r2 = v.validate(&alternative_grammar());
    // Both should succeed independently
    assert!(r1.errors.is_empty(), "r1 errors: {:?}", r1.errors);
    assert!(r2.errors.is_empty(), "r2 errors: {:?}", r2.errors);
}

#[test]
fn reuse_validator_valid_then_invalid() {
    let mut v = GrammarValidator::new();
    let r1 = v.validate(&minimal_grammar());
    assert!(r1.errors.is_empty());
    let empty = GrammarBuilder::new("empty").build();
    let r2 = v.validate(&empty);
    assert!(!r2.errors.is_empty());
}

#[test]
fn reuse_validator_invalid_then_valid() {
    let mut v = GrammarValidator::new();
    let empty = GrammarBuilder::new("empty").build();
    let r1 = v.validate(&empty);
    assert!(!r1.errors.is_empty());
    let r2 = v.validate(&minimal_grammar());
    assert!(r2.errors.is_empty(), "errors: {:?}", r2.errors);
}

#[test]
fn reuse_validator_three_times() {
    let mut v = GrammarValidator::new();
    let _r1 = v.validate(&minimal_grammar());
    let _r2 = v.validate(&multi_token_grammar());
    let _r3 = v.validate(&chain_grammar());
}

#[test]
fn reuse_validator_errors_not_accumulated() {
    let mut v = GrammarValidator::new();
    let empty = GrammarBuilder::new("empty").build();
    let r1 = v.validate(&empty);
    let err_count_1 = r1.errors.len();
    let r2 = v.validate(&empty);
    let err_count_2 = r2.errors.len();
    // Second run should have same error count, not double
    assert_eq!(err_count_1, err_count_2);
}

#[test]
fn reuse_validator_stats_change_between_grammars() {
    let mut v = GrammarValidator::new();
    let r1 = v.validate(&minimal_grammar());
    let r2 = v.validate(&multi_token_grammar());
    assert_ne!(r1.stats.total_tokens, r2.stats.total_tokens);
}

// ===========================================================================
// 10. Validation after normalize
// ===========================================================================

#[test]
fn validate_after_normalize_minimal() {
    let mut g = minimal_grammar();
    let _new_rules = g.normalize();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn validate_after_normalize_multi_token() {
    let mut g = multi_token_grammar();
    let _new_rules = g.normalize();
    let r = validate(&g);
    // CyclicRule may still appear for recursive rules
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn validate_after_normalize_alternative() {
    let mut g = alternative_grammar();
    let _new_rules = g.normalize();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn validate_after_normalize_chain() {
    let mut g = chain_grammar();
    let _new_rules = g.normalize();
    let r = validate(&g);
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn normalize_returns_rules() {
    let mut g = minimal_grammar();
    let new_rules = g.normalize();
    // For a simple grammar normalize may return empty or the same rules
    let _ = new_rules.len();
}

#[test]
fn stats_after_normalize_tokens_unchanged() {
    let r_before = validate(&minimal_grammar());
    let mut g = minimal_grammar();
    let _ = g.normalize();
    let r_after = validate(&g);
    assert_eq!(r_before.stats.total_tokens, r_after.stats.total_tokens);
}

// ===========================================================================
// Additional coverage: error variant detection & edge cases
// ===========================================================================

#[test]
fn cyclic_rule_detected_for_recursive_grammar() {
    let g = GrammarBuilder::new("rec")
        .token("T", "t")
        .rule("a", vec!["a"])
        .rule("a", vec!["T"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn cyclic_rule_detected_for_mutual_recursion() {
    let g = GrammarBuilder::new("mutual")
        .token("T", "t")
        .rule("a", vec!["b"])
        .rule("b", vec!["a"])
        .rule("a", vec!["T"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. }))
    );
}

#[test]
fn empty_grammar_error_variant() {
    let g = GrammarBuilder::new("empty").build();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn validation_error_display() {
    let err = ValidationError::EmptyGrammar;
    let s = format!("{}", err);
    assert!(!s.is_empty());
}

#[test]
fn validation_error_cyclic_display() {
    use adze_ir::SymbolId;
    let err = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let s = format!("{}", err);
    assert!(s.contains("Cyclic"));
}

#[test]
fn validation_error_clone() {
    let err = ValidationError::EmptyGrammar;
    let err2 = err.clone();
    assert_eq!(err, err2);
}

#[test]
fn validation_error_eq() {
    let a = ValidationError::EmptyGrammar;
    let b = ValidationError::EmptyGrammar;
    assert_eq!(a, b);
}

#[test]
fn validation_warning_clone() {
    use adze_ir::SymbolId;
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".to_string(),
    };
    let w2 = w.clone();
    assert_eq!(w, w2);
}

#[test]
fn validation_warning_eq() {
    use adze_ir::SymbolId;
    let a = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".to_string(),
    };
    let b = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".to_string(),
    };
    assert_eq!(a, b);
}

#[test]
fn validation_warning_display() {
    use adze_ir::SymbolId;
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".to_string(),
    };
    let s = format!("{}", w);
    assert!(!s.is_empty());
}

#[test]
fn stats_debug_output() {
    let r = validate(&minimal_grammar());
    let s = format!("{:?}", r.stats);
    assert!(s.contains("total_tokens"));
}

#[test]
fn stats_default_zeros() {
    let s = ValidationStats::default();
    assert_eq!(s.total_tokens, 0);
    assert_eq!(s.total_rules, 0);
    assert_eq!(s.total_symbols, 0);
    assert_eq!(s.external_tokens, 0);
}

#[test]
fn precedence_grammar_validates() {
    let r = validate(&precedence_grammar());
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn precedence_grammar_stats_three_tokens() {
    let r = validate(&precedence_grammar());
    assert_eq!(r.stats.total_tokens, 3);
}

#[test]
fn fragile_token_grammar_validates() {
    let g = GrammarBuilder::new("fragile")
        .token("OK", "ok")
        .fragile_token("ERR", "err")
        .rule("start", vec!["OK"])
        .start("start")
        .build();
    let r = validate(&g);
    // Unused fragile token may produce warning but not a fatal non-cyclic error
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn extra_token_grammar_validates() {
    let g = GrammarBuilder::new("extras")
        .token("ID", r"[a-z]+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn stats_for_python_like_has_many_rules() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    assert!(r.stats.total_rules >= 5);
}

#[test]
fn stats_for_javascript_like_has_many_tokens() {
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    assert!(r.stats.total_tokens >= 10);
}

#[test]
fn validate_after_normalize_nullable() {
    let mut g = nullable_grammar();
    let _ = g.normalize();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn validate_after_normalize_keyword() {
    let mut g = keyword_grammar();
    let _ = g.normalize();
    let r = validate(&g);
    let non_cyclic: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cyclic.is_empty(), "errors: {:?}", non_cyclic);
}

#[test]
fn validate_after_normalize_preserves_token_count() {
    let r_before = validate(&multi_token_grammar());
    let mut g = multi_token_grammar();
    let _ = g.normalize();
    let r_after = validate(&g);
    assert_eq!(r_before.stats.total_tokens, r_after.stats.total_tokens);
}
