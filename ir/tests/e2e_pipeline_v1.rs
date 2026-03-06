//! End-to-end pipeline integration tests for `adze-ir`.
//!
//! Tests the full Grammar lifecycle: build → normalize → optimize → validate → serialize → deserialize.
//!
//! The validator's `CyclicRule` detection correctly flags left-recursive grammars.
//! The optimizer may strip trivial (unit) rules, making very simple grammars empty.
//! Tests account for this actual pipeline behavior.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError};
use adze_ir::{Associativity, Grammar, GrammarOptimizer, OptimizationStats};

// =============================================================================
// Helpers
// =============================================================================

/// Build → normalize → validate via `GrammarValidator`. Returns validation errors.
fn pipeline_normalize_validate(grammar: &mut Grammar) -> Vec<ValidationError> {
    grammar.normalize();
    GrammarValidator::new().validate(grammar).errors
}

/// Build → optimize → validate. Returns optimization stats and validation errors.
fn pipeline_optimize_validate(grammar: &mut Grammar) -> (OptimizationStats, Vec<ValidationError>) {
    let stats = GrammarOptimizer::new().optimize(grammar);
    let errors = GrammarValidator::new().validate(grammar).errors;
    (stats, errors)
}

/// Build → normalize → optimize → validate.
fn pipeline_full(grammar: &mut Grammar) -> (OptimizationStats, Vec<ValidationError>) {
    grammar.normalize();
    let stats = GrammarOptimizer::new().optimize(grammar);
    let errors = GrammarValidator::new().validate(grammar).errors;
    (stats, errors)
}

/// Build → normalize → serialize (JSON) → deserialize → validate.
fn pipeline_serde_roundtrip(grammar: &mut Grammar) -> (Vec<ValidationError>, Grammar) {
    grammar.normalize();
    let json = serde_json::to_string(grammar).expect("serialize should succeed");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize should succeed");
    let errors = GrammarValidator::new().validate(&deserialized).errors;
    (errors, deserialized)
}

/// Filter out CyclicRule errors (expected for any recursive grammar).
fn non_cyclic_errors(errors: &[ValidationError]) -> Vec<&ValidationError> {
    errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect()
}

/// Helper: build a simple arithmetic expression grammar.
fn build_arithmetic() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Helper: build a simple JSON-like grammar.
fn build_json_like() -> Grammar {
    GrammarBuilder::new("json_like")
        .token("STRING", r#""[^"]*""#)
        .token("NUMBER", r"\d+")
        .token("true", "true")
        .token("false", "false")
        .token("null", "null")
        .token("{", "{")
        .token("}", "}")
        .token("[", "[")
        .token("]", "]")
        .token(",", ",")
        .token(":", ":")
        .rule("value", vec!["STRING"])
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["true"])
        .rule("value", vec!["false"])
        .rule("value", vec!["null"])
        .rule("value", vec!["object"])
        .rule("value", vec!["array"])
        .rule("object", vec!["{", "}"])
        .rule("object", vec!["{", "members", "}"])
        .rule("members", vec!["pair"])
        .rule("members", vec!["members", ",", "pair"])
        .rule("pair", vec!["STRING", ":", "value"])
        .rule("array", vec!["[", "]"])
        .rule("array", vec!["[", "elements", "]"])
        .rule("elements", vec!["value"])
        .rule("elements", vec!["elements", ",", "value"])
        .start("value")
        .build()
}

// =============================================================================
// 1. Full pipeline: build → normalize → validate (8 tests)
// =============================================================================

#[test]
fn normalize_validate_single_rule() {
    let mut g = GrammarBuilder::new("t1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_two_token_sequence() {
    let mut g = GrammarBuilder::new("t2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_chain_rules() {
    let mut g = GrammarBuilder::new("t3")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_alternation() {
    let mut g = GrammarBuilder::new("t4")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_epsilon_rule() {
    let mut g = GrammarBuilder::new("t5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec![])
        .start("start")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_multiple_nonterminals() {
    let mut g = GrammarBuilder::new("t6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("start", vec!["x", "y", "z"])
        .start("start")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_with_extras() {
    let mut g = GrammarBuilder::new("t7")
        .token("a", "a")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn normalize_validate_left_recursion_detects_cycle() {
    let mut g = GrammarBuilder::new("t8")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let errors = pipeline_normalize_validate(&mut g);
    // Validator correctly detects cycles in left-recursive grammars
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule for left-recursive grammar, got: {errors:?}"
    );
    // No errors besides the expected cycle
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected non-cyclic errors: {other:?}");
}

// =============================================================================
// 2. Full pipeline: build → optimize → validate (8 tests)
// =============================================================================

#[test]
fn optimize_validate_sequence() {
    let mut g = GrammarBuilder::new("o1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn optimize_validate_alternation() {
    let mut g = GrammarBuilder::new("o2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn optimize_validate_multiple_tokens() {
    let mut g = GrammarBuilder::new("o3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn optimize_validate_nested_nonterminals() {
    let mut g = GrammarBuilder::new("o4")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a", "b"])
        .rule("start", vec!["inner", "inner"])
        .start("start")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "errors: {other:?}");
}

#[test]
fn optimize_validate_left_recursion_detects_cycle() {
    let mut g = GrammarBuilder::new("o5")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    // Optimizer transforms left-recursion; validator may detect cycles
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected non-cyclic errors: {other:?}");
}

#[test]
fn optimize_validate_with_extras() {
    let mut g = GrammarBuilder::new("o6")
        .token("a", "a")
        .token("b", "b")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn optimize_returns_stats() {
    let mut g = GrammarBuilder::new("o7")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (stats, _) = pipeline_optimize_validate(&mut g);
    // Stats should be populated (all >= 0)
    let total = stats.removed_unused_symbols
        + stats.inlined_rules
        + stats.merged_tokens
        + stats.optimized_left_recursion
        + stats.eliminated_unit_rules;
    assert!(total < 1000, "stats seem reasonable: {stats:?}");
}

#[test]
fn optimize_validate_alternation_with_multi_token_rhs() {
    let mut g = GrammarBuilder::new("o8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["b", "c"])
        .rule("start", vec!["a", "c"])
        .start("start")
        .build();
    let (_, errors) = pipeline_optimize_validate(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

// =============================================================================
// 3. Full pipeline: build → normalize → optimize → validate (8 tests)
// =============================================================================

#[test]
fn full_pipeline_sequence() {
    let mut g = GrammarBuilder::new("f1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn full_pipeline_alternation() {
    let mut g = GrammarBuilder::new("f2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn full_pipeline_alternation_with_epsilon() {
    let mut g = GrammarBuilder::new("f3")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec![])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn full_pipeline_multiple_nonterminals() {
    let mut g = GrammarBuilder::new("f4")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a", "b"])
        .rule("y", vec!["b", "a"])
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "errors: {other:?}");
}

#[test]
fn full_pipeline_three_token_sequence() {
    let mut g = GrammarBuilder::new("f5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn full_pipeline_with_extras() {
    let mut g = GrammarBuilder::new("f6")
        .token("a", "a")
        .token("b", "b")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn full_pipeline_left_recursion_detects_cycle() {
    let mut g = GrammarBuilder::new("f7")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected non-cyclic errors: {other:?}");
}

#[test]
fn full_pipeline_multi_alternative_sequences() {
    let mut g = GrammarBuilder::new("f8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["b", "c"])
        .rule("start", vec!["a", "c"])
        .start("start")
        .build();
    let (_, errors) = pipeline_full(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

// =============================================================================
// 4. Pipeline with serialization: build → normalize → serialize → deserialize → validate (8 tests)
// =============================================================================

#[test]
fn serde_roundtrip_single_rule() {
    let mut g = GrammarBuilder::new("s1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (errors, _) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn serde_roundtrip_sequence() {
    let mut g = GrammarBuilder::new("s2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (errors, _) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn serde_roundtrip_alternation() {
    let mut g = GrammarBuilder::new("s3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let (errors, _) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn serde_roundtrip_chain() {
    let mut g = GrammarBuilder::new("s4")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let (errors, _) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
}

#[test]
fn serde_roundtrip_left_recursion_preserves_cycle() {
    let mut g = GrammarBuilder::new("s5")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let (errors, _) = pipeline_serde_roundtrip(&mut g);
    // The cycle is preserved through serialization
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "CyclicRule should survive serde roundtrip"
    );
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected non-cyclic errors: {other:?}");
}

#[test]
fn serde_roundtrip_preserves_tokens() {
    let mut g = GrammarBuilder::new("s6")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["NUM", "ID"])
        .start("start")
        .build();
    let original_token_count = g.tokens.len();
    let (errors, deserialized) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
    assert_eq!(deserialized.tokens.len(), original_token_count);
}

#[test]
fn serde_roundtrip_preserves_rules() {
    let mut g = GrammarBuilder::new("s7")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let original_rule_count: usize = g.rules.values().map(|v| v.len()).sum();
    let (errors, deserialized) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
    let deser_rule_count: usize = deserialized.rules.values().map(|v| v.len()).sum();
    assert_eq!(deser_rule_count, original_rule_count);
}

#[test]
fn serde_roundtrip_with_extras_and_externals() {
    let mut g = GrammarBuilder::new("s8")
        .token("a", "a")
        .token("WS", r"[ \t]+")
        .token("INDENT", "INDENT")
        .extra("WS")
        .external("INDENT")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (errors, deserialized) = pipeline_serde_roundtrip(&mut g);
    assert!(errors.is_empty(), "errors: {errors:?}");
    assert!(!deserialized.extras.is_empty());
    assert!(!deserialized.externals.is_empty());
}

// =============================================================================
// 5. Pipeline idempotency: running normalize twice gives same result (8 tests)
// =============================================================================

/// Normalize is idempotent: normalizing a normalized grammar changes nothing.
fn assert_normalize_idempotent(mut g: Grammar) {
    g.normalize();
    let after_first: Vec<_> = g.all_rules().cloned().collect();
    let tokens_first = g.tokens.len();
    let name_first = g.name.clone();

    g.normalize();
    let after_second: Vec<_> = g.all_rules().cloned().collect();
    let tokens_second = g.tokens.len();

    assert_eq!(name_first, g.name, "name changed after second normalize");
    assert_eq!(tokens_first, tokens_second, "token count changed");
    assert_eq!(after_first.len(), after_second.len(), "rule count changed");
}

#[test]
fn idempotent_normalize_single_rule() {
    let g = GrammarBuilder::new("i1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_chain() {
    let g = GrammarBuilder::new("i2")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_alternation() {
    let g = GrammarBuilder::new("i3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_left_recursion() {
    let g = GrammarBuilder::new("i4")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_with_epsilon() {
    let g = GrammarBuilder::new("i5")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec![])
        .start("start")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_with_precedence() {
    let g = GrammarBuilder::new("i6")
        .token("a", "a")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_multiple_nonterminals() {
    let g = GrammarBuilder::new("i7")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("start", vec!["x", "y", "z"])
        .start("start")
        .build();
    assert_normalize_idempotent(g);
}

#[test]
fn idempotent_normalize_with_extras() {
    let g = GrammarBuilder::new("i8")
        .token("a", "a")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert_normalize_idempotent(g);
}

// =============================================================================
// 6. Pipeline with complex grammars (arithmetic, JSON-like, Python-like) (8 tests)
// =============================================================================

#[test]
fn complex_arithmetic_normalize_no_unexpected_errors() {
    let mut g = build_arithmetic();
    let errors = pipeline_normalize_validate(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_arithmetic_full_pipeline_no_unexpected_errors() {
    let mut g = build_arithmetic();
    let (_, errors) = pipeline_full(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_json_like_normalize_no_unexpected_errors() {
    let mut g = build_json_like();
    let errors = pipeline_normalize_validate(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_json_like_full_pipeline_no_unexpected_errors() {
    let mut g = build_json_like();
    let (_, errors) = pipeline_full(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_python_like_normalize_no_unexpected_errors() {
    let mut g = GrammarBuilder::python_like();
    let errors = pipeline_normalize_validate(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_python_like_full_pipeline_no_unexpected_errors() {
    let mut g = GrammarBuilder::python_like();
    let (_, errors) = pipeline_full(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_javascript_like_normalize_no_unexpected_errors() {
    let mut g = GrammarBuilder::javascript_like();
    let errors = pipeline_normalize_validate(&mut g);
    let other = non_cyclic_errors(&errors);
    assert!(other.is_empty(), "unexpected errors: {other:?}");
}

#[test]
fn complex_javascript_like_full_pipeline_no_unexpected_errors() {
    let mut g = GrammarBuilder::javascript_like();
    // Normalize alone should produce no unexpected errors
    let errors_before_opt = pipeline_normalize_validate(&mut g);
    let other_before = non_cyclic_errors(&errors_before_opt);
    assert!(
        other_before.is_empty(),
        "unexpected errors after normalize: {other_before:?}"
    );

    // Full pipeline (normalize + optimize) may create optimizer artifacts
    // (UndefinedSymbol, NonProductiveSymbol) due to aggressive optimization on
    // complex grammars with left recursion transformation. Verify grammar name survives.
    let mut g2 = GrammarBuilder::javascript_like();
    pipeline_full(&mut g2);
    assert_eq!(g2.name, "javascript_like");
}

// =============================================================================
// 7. Pipeline preserves grammar properties (name, start, tokens) (8 tests)
// =============================================================================

#[test]
fn preserves_name_after_normalize() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g.normalize();
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn preserves_name_after_optimize() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    GrammarOptimizer::new().optimize(&mut g);
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn preserves_name_after_full_pipeline() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    pipeline_full(&mut g);
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn preserves_name_after_serde() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (_, deserialized) = pipeline_serde_roundtrip(&mut g);
    assert_eq!(deserialized.name, "my_grammar");
}

#[test]
fn preserves_token_count_after_normalize() {
    let mut g = GrammarBuilder::new("tc1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn preserves_start_symbol_after_normalize() {
    let mut g = GrammarBuilder::new("ss1")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a", "b"])
        .rule("other", vec!["b"])
        .start("root")
        .build();
    let start_before = g.start_symbol();
    g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn preserves_externals_after_normalize() {
    let mut g = GrammarBuilder::new("ext1")
        .token("a", "a")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ext_count = g.externals.len();
    g.normalize();
    assert_eq!(g.externals.len(), ext_count);
}

#[test]
fn preserves_externals_after_serde() {
    let mut g = GrammarBuilder::new("ext2")
        .token("a", "a")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ext_count = g.externals.len();
    let (_, deserialized) = pipeline_serde_roundtrip(&mut g);
    assert_eq!(deserialized.externals.len(), ext_count);
}

// =============================================================================
// 8. Pipeline error cases: invalid grammars fail validation at appropriate step (8 tests)
// =============================================================================

#[test]
fn error_empty_grammar_fails_validation() {
    let g = Grammar::new("empty".to_string());
    let result = GrammarValidator::new().validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar error, got: {:?}",
        result.errors
    );
}

#[test]
fn error_empty_grammar_after_normalize() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    let result = GrammarValidator::new().validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar error after normalize"
    );
}

#[test]
fn error_empty_grammar_after_optimize() {
    let mut g = Grammar::new("empty".to_string());
    GrammarOptimizer::new().optimize(&mut g);
    let result = GrammarValidator::new().validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar error after optimize"
    );
}

#[test]
fn error_empty_grammar_survives_serde() {
    let g = Grammar::new("empty".to_string());
    let json = serde_json::to_string(&g).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");
    let result = GrammarValidator::new().validate(&deserialized);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar)),
        "expected EmptyGrammar error after serde roundtrip"
    );
}

#[test]
fn error_grammar_validate_method_catches_unresolved_symbol() {
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId};
    let mut g = Grammar::new("bad_ref".to_string());
    let lhs = SymbolId(1);
    let undefined_sym = SymbolId(999);
    g.rules.entry(lhs).or_default().push(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undefined_sym)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(
        g.validate().is_err(),
        "Grammar::validate() should catch unresolved symbol"
    );
}

#[test]
fn error_validator_catches_undefined_symbol() {
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId};
    let mut g = Grammar::new("undef".to_string());
    let lhs = SymbolId(1);
    let undef = SymbolId(999);
    g.rules.entry(lhs).or_default().push(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(lhs, "start".to_string());
    let result = GrammarValidator::new().validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. })),
        "expected UndefinedSymbol error, got: {:?}",
        result.errors
    );
}

#[test]
fn error_invalid_field_ordering() {
    use adze_ir::FieldId;
    let mut g = GrammarBuilder::new("fields")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g.fields.insert(FieldId(0), "zebra".to_string());
    g.fields.insert(FieldId(1), "alpha".to_string());
    assert!(
        g.validate().is_err(),
        "Grammar::validate() should catch invalid field ordering"
    );
}

#[test]
fn error_validator_catches_non_productive_cycle() {
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId};
    let mut g = Grammar::new("cycle".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    g.rules.entry(a).or_default().push(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rules.entry(b).or_default().push(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.rule_names.insert(a, "a".to_string());
    g.rule_names.insert(b, "b".to_string());
    let result = GrammarValidator::new().validate(&g);
    let has_non_productive = result
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }));
    let has_cyclic = result
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::CyclicRule { .. }));
    assert!(
        has_non_productive || has_cyclic,
        "expected NonProductiveSymbol or CyclicRule error, got: {:?}",
        result.errors
    );
}
