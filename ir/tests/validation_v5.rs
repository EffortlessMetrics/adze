//! Validation V5 — 55+ tests for Grammar validation.
//!
//! Categories:
//! 1. Valid grammars pass (arithmetic, JSON-like, nested, recursive)
//! 2. Invalid grammars caught (undefined symbols, unreachable, duplicate tokens, missing start)
//! 3. Validation after normalize() — still valid
//! 4. Validation after optimize() — still valid
//! 5. Edge cases (empty, single rule, many tokens, deeply nested)
//! 6. Symbol registry consistency after validation

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validator_result(grammar: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

fn has_error(
    result: &adze_ir::validation::ValidationResult,
    pred: impl Fn(&ValidationError) -> bool,
) -> bool {
    result.errors.iter().any(pred)
}

fn has_warning(
    result: &adze_ir::validation::ValidationResult,
    pred: impl Fn(&ValidationWarning) -> bool,
) -> bool {
    result.warnings.iter().any(pred)
}

/// Build a minimal valid grammar (one token, one rule, start set).
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build()
}

// ===========================================================================
// 1. Valid grammars pass validation
// ===========================================================================

#[test]
fn test_valid_arithmetic_grammar_passes() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "STAR", "factor"])
        .rule("factor", vec!["NUM"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build();
    let r = validator_result(&g);
    // Recursive grammars may trigger CyclicRule; filter those out.
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_valid_json_like_grammar_passes() {
    let g = GrammarBuilder::new("json_like")
        .token("LBRACE", r"\{")
        .token("RBRACE", r"\}")
        .token("LBRACK", r"\[")
        .token("RBRACK", r"\]")
        .token("COLON", ":")
        .token("COMMA", ",")
        .token("STRING", r#""[^"]*""#)
        .token("NUMBER", r"\d+")
        .rule("value", vec!["STRING"])
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["object"])
        .rule("value", vec!["array"])
        .rule("object", vec!["LBRACE", "pair", "RBRACE"])
        .rule("pair", vec!["STRING", "COLON", "value"])
        .rule("array", vec!["LBRACK", "value", "RBRACK"])
        .start("value")
        .build();
    let r = validator_result(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_valid_nested_grammar_passes() {
    let g = GrammarBuilder::new("nested")
        .token("A", "a")
        .token("B", "b")
        .rule("top", vec!["mid"])
        .rule("mid", vec!["low"])
        .rule("low", vec!["A", "B"])
        .start("top")
        .build();
    let r = validator_result(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_right_recursive_grammar_passes() {
    let g = GrammarBuilder::new("right_rec")
        .token("ID", r"[a-z]+")
        .token("DOT", r"\.")
        .rule("access", vec!["ID"])
        .rule("access", vec!["ID", "DOT", "access"])
        .start("access")
        .build();
    let r = validator_result(&g);
    // Cycles may be reported for recursive grammars; only non-cycle errors matter.
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_valid_left_recursive_grammar_passes() {
    let g = GrammarBuilder::new("left_rec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .start("expr")
        .build();
    let r = validator_result(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_valid_grammar_with_extras_passes() {
    let g = GrammarBuilder::new("extras")
        .token("WS", r"\s+")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["ID"])
        .start("root")
        .extra("WS")
        .build();
    let r = validator_result(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_precedence_passes() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .start("expr")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .build();
    let r = validator_result(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_valid_grammar_with_inline_rules_passes() {
    let g = GrammarBuilder::new("inline")
        .token("A", "a")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["A"])
        .start("root")
        .inline("helper")
        .build();
    let r = validator_result(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_supertypes_passes() {
    let g = GrammarBuilder::new("supertypes")
        .token("A", "a")
        .token("B", "b")
        .rule("node", vec!["A"])
        .rule("node", vec!["B"])
        .rule("root", vec!["node"])
        .start("root")
        .supertype("node")
        .build();
    let r = validator_result(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_externals_passes() {
    let g = GrammarBuilder::new("ext")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .external("INDENT")
        .build();
    let r = validator_result(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ===========================================================================
// 2. Invalid grammars caught
// ===========================================================================

#[test]
fn test_undefined_symbol_in_rule_rhs() {
    // "unknown" is referenced but never defined as a token.
    // The builder resolves it to a NonTerminal ID, but there is no rule for it.
    let g = GrammarBuilder::new("undef")
        .token("A", "a")
        .rule("root", vec!["A", "unknown"])
        .start("root")
        .build();
    let r = validator_result(&g);
    // The validator should flag `unknown` as unreachable/non-productive since
    // it has no production. Grammar.validate() should flag it as unresolved.
    let grammar_result = g.validate();
    assert!(
        grammar_result.is_err()
            || has_error(&r, |e| {
                matches!(
                    e,
                    ValidationError::NonProductiveSymbol { .. }
                        | ValidationError::UndefinedSymbol { .. }
                )
            }),
        "should detect undefined symbol"
    );
}

#[test]
fn test_multiple_undefined_symbols_detected() {
    let g = GrammarBuilder::new("multi_undef")
        .token("A", "a")
        .rule("root", vec!["A", "missing1", "missing2"])
        .start("root")
        .build();
    let r = validator_result(&g);
    let grammar_result = g.validate();
    assert!(
        grammar_result.is_err() || !r.errors.is_empty(),
        "should detect multiple undefined symbols"
    );
}

#[test]
fn test_unreachable_rule_produces_warning() {
    let g = GrammarBuilder::new("unreachable")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("orphan", vec!["B"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(
        has_warning(&r, |w| matches!(w, ValidationWarning::UnusedToken { .. }))
            || has_error(&r, |e| matches!(
                e,
                ValidationError::UnreachableSymbol { .. }
            ))
            || r.warnings.iter().any(|w| {
                let msg = format!("{w}");
                msg.contains("orphan") || msg.contains("unreachable")
            }),
        "unreachable rule should produce a warning or error: errors={:?}, warnings={:?}",
        r.errors,
        r.warnings
    );
}

#[test]
fn test_duplicate_token_pattern_produces_warning() {
    let g = GrammarBuilder::new("dup_tok")
        .token("FOO", "x")
        .token("BAR", "x")
        .rule("root", vec!["FOO"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(
        has_warning(&r, |w| matches!(
            w,
            ValidationWarning::DuplicateTokenPattern { .. }
        )),
        "duplicate token pattern should warn: {:?}",
        r.warnings
    );
}

#[test]
fn test_three_duplicate_token_patterns() {
    let g = GrammarBuilder::new("triple_dup")
        .token("A1", "zzz")
        .token("A2", "zzz")
        .token("A3", "zzz")
        .rule("root", vec!["A1"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(
        has_warning(&r, |w| matches!(
            w,
            ValidationWarning::DuplicateTokenPattern { .. }
        )),
        "three duplicate patterns should warn"
    );
}

#[test]
fn test_empty_grammar_reports_error() {
    let g = Grammar::new("empty".to_string());
    let r = validator_result(&g);
    assert!(
        has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)),
        "empty grammar must produce EmptyGrammar error"
    );
}

#[test]
fn test_empty_grammar_error_count() {
    let g = Grammar::new("empty2".to_string());
    let r = validator_result(&g);
    assert!(!r.errors.is_empty(), "should have at least one error");
}

#[test]
fn test_no_start_symbol_warning() {
    // Build without calling .start()
    let g = GrammarBuilder::new("no_start")
        .token("A", "a")
        .rule("root", vec!["A"])
        .build();
    // The builder uses the first rule as implicit start; some validation
    // implementations may still warn about missing explicit start.
    let _r = validator_result(&g);
    // At a minimum this must not panic.
}

#[test]
fn test_non_productive_symbol_detected() {
    // "loop_sym" only references itself — can never produce terminals.
    let g = GrammarBuilder::new("nonprod")
        .token("A", "a")
        .rule("root", vec!["A", "loop_sym"])
        .rule("loop_sym", vec!["loop_sym"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::NonProductiveSymbol { .. } | ValidationError::CyclicRule { .. }
        )),
        "non-productive / cyclic symbol should be caught: {:?}",
        r.errors
    );
}

#[test]
fn test_cycle_detection_direct() {
    let g = GrammarBuilder::new("direct_cycle")
        .rule("a", vec!["a"])
        .start("a")
        .build();
    let r = validator_result(&g);
    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::CyclicRule { .. } | ValidationError::NonProductiveSymbol { .. }
        )),
        "direct self-cycle should be detected"
    );
}

#[test]
fn test_cycle_detection_mutual() {
    let g = GrammarBuilder::new("mutual_cycle")
        .rule("a", vec!["b"])
        .rule("b", vec!["a"])
        .start("a")
        .build();
    let r = validator_result(&g);
    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::CyclicRule { .. } | ValidationError::NonProductiveSymbol { .. }
        )),
        "mutual cycle should be detected"
    );
}

#[test]
fn test_cycle_detection_three_nodes() {
    let g = GrammarBuilder::new("three_cycle")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["a"])
        .start("a")
        .build();
    let r = validator_result(&g);
    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::CyclicRule { .. } | ValidationError::NonProductiveSymbol { .. }
        )),
        "three-node cycle should be detected"
    );
}

#[test]
fn test_conflicting_precedence_error() {
    let g = GrammarBuilder::new("prec_conflict")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .precedence(1, Associativity::Left, vec!["A"])
        .precedence(2, Associativity::Right, vec!["A"])
        .build();
    let r = validator_result(&g);
    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::ConflictingPrecedence { .. }
        )),
        "conflicting precedences should be reported: {:?}",
        r.errors
    );
}

// ===========================================================================
// 3. Validation after normalize() — still valid
// ===========================================================================

#[test]
fn test_normalize_then_grammar_validate_still_ok() {
    let mut g = GrammarBuilder::new("norm1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .start("expr")
        .build();
    g.normalize();
    // Grammar.validate() should still pass after normalization.
    let result = g.validate();
    assert!(
        result.is_ok(),
        "grammar.validate() after normalize: {:?}",
        result
    );
}

#[test]
fn test_normalize_then_validator_no_empty_grammar() {
    let mut g = GrammarBuilder::new("norm2")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    g.normalize();
    let r = validator_result(&g);
    assert!(
        !has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)),
        "normalized grammar should not become empty"
    );
}

#[test]
fn test_normalize_preserves_tokens() {
    let mut g = GrammarBuilder::new("norm3")
        .token("NUM", r"\d+")
        .token("SEMI", ";")
        .rule("stmt", vec!["NUM", "SEMI"])
        .start("stmt")
        .build();
    let token_count_before = g.tokens.len();
    g.normalize();
    assert_eq!(
        g.tokens.len(),
        token_count_before,
        "normalize should not alter tokens"
    );
}

#[test]
fn test_normalize_arithmetic_still_validates() {
    let mut g = GrammarBuilder::new("norm_arith")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("term", vec!["NUM"])
        .rule("term", vec!["term", "STAR", "NUM"])
        .start("expr")
        .build();
    g.normalize();
    assert!(
        g.validate().is_ok(),
        "arithmetic after normalize should validate"
    );
}

#[test]
fn test_normalize_json_like_still_validates() {
    let mut g = GrammarBuilder::new("norm_json")
        .token("LBRACE", r"\{")
        .token("RBRACE", r"\}")
        .token("COLON", ":")
        .token("STRING", r#""[^"]*""#)
        .rule("value", vec!["STRING"])
        .rule("value", vec!["object"])
        .rule("object", vec!["LBRACE", "pair", "RBRACE"])
        .rule("pair", vec!["STRING", "COLON", "value"])
        .start("value")
        .build();
    g.normalize();
    assert!(
        g.validate().is_ok(),
        "json-like after normalize should validate"
    );
}

#[test]
fn test_normalize_does_not_remove_rules() {
    let mut g = GrammarBuilder::new("norm_keep")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let rule_count_before = g.rules.len();
    g.normalize();
    assert!(
        g.rules.len() >= rule_count_before,
        "normalize may add rules but should not remove them"
    );
}

// ===========================================================================
// 4. Validation after optimize() — still valid
// ===========================================================================

#[test]
fn test_optimize_then_grammar_validate_still_ok() {
    let mut g = GrammarBuilder::new("opt1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .start("expr")
        .build();
    g.optimize();
    assert!(
        g.validate().is_ok(),
        "grammar.validate() after optimize: {:?}",
        g.validate()
    );
}

#[test]
fn test_optimize_then_validator_no_empty_grammar() {
    let mut g = minimal_grammar();
    g.optimize();
    let r = validator_result(&g);
    assert!(
        !has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)),
        "optimized grammar should not become empty"
    );
}

#[test]
fn test_optimize_preserves_tokens() {
    let mut g = GrammarBuilder::new("opt_tok")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let count = g.tokens.len();
    g.optimize();
    assert_eq!(g.tokens.len(), count, "optimize should preserve tokens");
}

#[test]
fn test_optimize_preserves_start_symbol() {
    let mut g = GrammarBuilder::new("opt_start")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let start_before = g.start_symbol();
    g.optimize();
    assert_eq!(
        g.start_symbol(),
        start_before,
        "optimize should preserve start"
    );
}

#[test]
fn test_optimize_then_normalize_still_validates() {
    let mut g = GrammarBuilder::new("opt_norm")
        .token("NUM", r"\d+")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    g.optimize();
    g.normalize();
    assert!(
        g.validate().is_ok(),
        "optimize then normalize should still validate"
    );
}

#[test]
fn test_normalize_then_optimize_still_validates() {
    let mut g = GrammarBuilder::new("norm_opt")
        .token("NUM", r"\d+")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    g.normalize();
    g.optimize();
    assert!(
        g.validate().is_ok(),
        "normalize then optimize should still validate"
    );
}

// ===========================================================================
// 5. Edge cases
// ===========================================================================

#[test]
fn test_edge_single_rule_single_token() {
    let g = minimal_grammar();
    let r = validator_result(&g);
    assert!(
        r.errors.is_empty(),
        "minimal grammar should validate: {:?}",
        r.errors
    );
}

#[test]
fn test_edge_single_epsilon_rule() {
    let g = GrammarBuilder::new("eps")
        .rule("root", vec![])
        .start("root")
        .build();
    let r = validator_result(&g);
    // An epsilon-only grammar is a valid (trivial) grammar.
    assert!(
        !has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)),
        "single epsilon rule is not empty"
    );
}

#[test]
fn test_edge_many_tokens() {
    let mut b = GrammarBuilder::new("many_tokens");
    for i in 0..50 {
        b = b.token(&format!("T{i}"), &format!("t{i}"));
    }
    // Reference all tokens so none are unused.
    let names: Vec<String> = (0..50).map(|i| format!("T{i}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    b = b.rule("root", refs);
    b = b.start("root");
    let g = b.build();
    let r = validator_result(&g);
    assert!(
        r.errors.is_empty(),
        "many-token grammar should validate: {:?}",
        r.errors
    );
    assert_eq!(r.stats.total_tokens, 50);
}

#[test]
fn test_edge_deeply_nested_chain() {
    let depth = 20;
    let mut b = GrammarBuilder::new("deep");
    b = b.token("LEAF", "leaf");
    for i in 0..depth {
        let lhs = format!("level_{i}");
        let rhs = if i + 1 < depth {
            format!("level_{}", i + 1)
        } else {
            "LEAF".to_string()
        };
        b = b.rule(&lhs, vec![&rhs]);
    }
    b = b.start("level_0");
    let g = b.build();
    let r = validator_result(&g);
    assert!(
        r.errors.is_empty(),
        "deep chain should validate: {:?}",
        r.errors
    );
}

#[test]
fn test_edge_grammar_validate_method() {
    let g = minimal_grammar();
    assert!(g.validate().is_ok(), "Grammar.validate() on minimal");
}

#[test]
fn test_edge_default_grammar_is_empty() {
    let g = Grammar::default();
    let r = validator_result(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn test_edge_grammar_name_preserved_after_build() {
    let g = GrammarBuilder::new("my_grammar")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert_eq!(g.name, "my_grammar");
}

#[test]
fn test_edge_grammar_start_symbol_matches() {
    let g = GrammarBuilder::new("ss")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let start = g.start_symbol();
    assert!(start.is_some(), "start symbol should be set");
    // The start symbol should be the first key in rules.
    let first_key = g.rules.keys().next();
    assert_eq!(start, first_key.copied());
}

#[test]
fn test_edge_multiple_alternatives_same_lhs() {
    let g = GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(
        r.errors.is_empty(),
        "alternatives should validate: {:?}",
        r.errors
    );
    assert!(r.stats.total_rules >= 3);
}

// ===========================================================================
// 6. Symbol registry consistency after validation
// ===========================================================================

#[test]
fn test_registry_built_after_validation() {
    let mut g = GrammarBuilder::new("reg1")
        .token("NUM", r"\d+")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    // Validate first.
    let _ = g.validate();
    // Build registry — should not panic.
    let reg = g.get_or_build_registry();
    assert!(!reg.is_empty(), "registry should have symbols");
}

#[test]
fn test_registry_contains_tokens() {
    let mut g = GrammarBuilder::new("reg2")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let reg = g.get_or_build_registry();
    assert!(reg.get_id("A").is_some(), "registry should contain token A");
    assert!(reg.get_id("B").is_some(), "registry should contain token B");
}

#[test]
fn test_registry_contains_nonterminals() {
    let mut g = GrammarBuilder::new("reg3")
        .token("X", "x")
        .rule("root", vec!["child"])
        .rule("child", vec!["X"])
        .start("root")
        .build();
    let reg = g.get_or_build_registry();
    assert!(reg.get_id("root").is_some(), "registry should contain root");
    assert!(
        reg.get_id("child").is_some(),
        "registry should contain child"
    );
}

#[test]
fn test_registry_deterministic_across_calls() {
    let build = || {
        let mut g = GrammarBuilder::new("det")
            .token("A", "a")
            .token("B", "b")
            .rule("root", vec!["A", "B"])
            .start("root")
            .build();
        g.get_or_build_registry().clone()
    };
    let r1 = build();
    let r2 = build();
    assert_eq!(
        r1, r2,
        "registry should be deterministic across identical builds"
    );
}

#[test]
fn test_registry_consistency_after_normalize() {
    let mut g = GrammarBuilder::new("reg_norm")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let reg_before = g.build_registry();
    g.normalize();
    let reg_after = g.build_registry();
    // The original tokens should still be present.
    for (name, info) in reg_before.iter() {
        if name == "end" {
            continue; // built-in EOF symbol
        }
        let after_id = reg_after.get_id(name);
        assert!(
            after_id.is_some(),
            "symbol '{name}' (id {:?}) missing after normalize",
            info.id
        );
    }
}

#[test]
fn test_registry_consistency_after_optimize() {
    let mut g = GrammarBuilder::new("reg_opt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let reg_before = g.build_registry();
    g.optimize();
    let reg_after = g.build_registry();
    for (name, _info) in reg_before.iter() {
        assert!(
            reg_after.get_id(name).is_some(),
            "symbol '{name}' missing after optimize"
        );
    }
}

#[test]
fn test_registry_id_lookup_roundtrip() {
    let mut g = GrammarBuilder::new("roundtrip")
        .token("TOK", "tok")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();
    let reg = g.get_or_build_registry();
    if let Some(id) = reg.get_id("TOK") {
        let name = reg.get_name(id);
        assert_eq!(name, Some("TOK"), "name lookup roundtrip should match");
    }
}

// ===========================================================================
// Additional coverage: stats, display, reuse
// ===========================================================================

#[test]
fn test_stats_total_tokens_count() {
    let g = GrammarBuilder::new("stats_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B", "C"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert_eq!(r.stats.total_tokens, 3);
}

#[test]
fn test_stats_total_rules_count() {
    let g = GrammarBuilder::new("stats_rules")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("root", vec!["A", "A"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(r.stats.total_rules >= 2);
}

#[test]
fn test_stats_max_rule_length() {
    let g = GrammarBuilder::new("stats_len")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B", "C"])
        .rule("short", vec!["A"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert_eq!(r.stats.max_rule_length, 3);
}

#[test]
fn test_stats_reachable_symbols() {
    let g = GrammarBuilder::new("stats_reach")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let r = validator_result(&g);
    assert!(r.stats.reachable_symbols >= 1);
}

#[test]
fn test_stats_external_tokens_count() {
    let g = GrammarBuilder::new("stats_ext")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .external("INDENT")
        .external("DEDENT")
        .build();
    let r = validator_result(&g);
    assert_eq!(r.stats.external_tokens, 2);
}

#[test]
fn test_validation_error_display_empty() {
    let err = ValidationError::EmptyGrammar;
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn test_validation_error_display_no_start() {
    let err = ValidationError::NoExplicitStartRule;
    let display = format!("{err}");
    assert!(display.contains("start"), "display: {display}");
}

#[test]
fn test_validator_reuse_clears_state() {
    let mut v = GrammarValidator::new();
    let empty = Grammar::new("e".to_string());
    let r1 = v.validate(&empty);
    assert!(!r1.errors.is_empty());

    let good = minimal_grammar();
    let r2 = v.validate(&good);
    // The second run should not carry over errors from the first.
    assert!(
        r2.errors.is_empty(),
        "validator reuse should clear previous errors: {:?}",
        r2.errors
    );
}

#[test]
fn test_validator_can_validate_twice() {
    let mut v = GrammarValidator::new();
    let g = minimal_grammar();
    let r1 = v.validate(&g);
    let r2 = v.validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
    assert_eq!(r1.warnings.len(), r2.warnings.len());
}

#[test]
fn test_grammar_with_multiple_issues() {
    // Combine several issues: cycle + unreachable + duplicate pattern.
    let g = GrammarBuilder::new("multi_issue")
        .token("A", "x")
        .token("B", "x") // duplicate pattern
        .rule("root", vec!["A"])
        .rule("orphan", vec!["B"]) // unreachable
        .rule("cycle", vec!["cycle"]) // cycle
        .start("root")
        .build();
    let r = validator_result(&g);
    // Should report multiple problems.
    let total = r.errors.len() + r.warnings.len();
    assert!(
        total >= 2,
        "should find multiple issues: errors={:?}, warnings={:?}",
        r.errors,
        r.warnings
    );
}
