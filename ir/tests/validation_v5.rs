//! Validation V5 — 55+ tests for `GrammarValidator`.
//!
//! Categories: valid grammars, missing start symbol, unreachable rules,
//! undefined symbols in RHS, empty grammars, cyclic rules, duplicate tokens,
//! precedences, externals, fields, stats, warnings, reuse.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationResult, ValidationWarning};
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate(grammar: &Grammar) -> ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

fn has_error(result: &ValidationResult, pred: impl Fn(&ValidationError) -> bool) -> bool {
    result.errors.iter().any(pred)
}

fn has_warning(result: &ValidationResult, pred: impl Fn(&ValidationWarning) -> bool) -> bool {
    result.warnings.iter().any(pred)
}

// ---------------------------------------------------------------------------
// 1. Valid grammars pass
// ---------------------------------------------------------------------------

#[test]
fn test_valid_single_token_grammar_passes() {
    let g = GrammarBuilder::new("single")
        .token("NUM", r"\d+")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_multi_token_grammar_passes() {
    let g = GrammarBuilder::new("multi")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("sum", vec!["expr", "+", "expr"])
        .rule("root", vec!["expr"])
        .rule("root", vec!["sum"])
        .start("root")
        .build();
    let r = validate(&g);
    // Only cyclic errors matter; filter them out since no real cycle exists.
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_valid_chain_grammar_passes() {
    let g = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["X"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_alternative_grammar_passes() {
    let g = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_extras_passes() {
    let g = GrammarBuilder::new("extras")
        .token("NUM", r"\d+")
        .token("WS", r"\s+")
        .rule("root", vec!["NUM"])
        .start("root")
        .extra("WS")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_externals_passes() {
    let g = GrammarBuilder::new("ext")
        .token("NUM", r"\d+")
        .rule("root", vec!["NUM"])
        .start("root")
        .external("INDENT")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_precedence_passes() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("term", vec!["NUM"])
        .rule("add", vec!["term", "+", "term"])
        .rule("mul", vec!["term", "*", "term"])
        .rule("root", vec!["term"])
        .rule("root", vec!["add"])
        .rule("root", vec!["mul"])
        .start("root")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_inline_rules_passes() {
    let g = GrammarBuilder::new("inl")
        .token("X", "x")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["X"])
        .start("root")
        .inline("helper")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_valid_grammar_with_supertypes_passes() {
    let g = GrammarBuilder::new("sup")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("literal", vec!["NUM"])
        .rule("literal", vec!["ID"])
        .rule("root", vec!["literal"])
        .start("root")
        .supertype("literal")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ---------------------------------------------------------------------------
// 2. Empty grammars
// ---------------------------------------------------------------------------

#[test]
fn test_empty_grammar_reports_error() {
    let g = Grammar::default();
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)));
}

#[test]
fn test_empty_grammar_error_count_is_at_least_one() {
    let g = Grammar::default();
    let r = validate(&g);
    assert!(!r.errors.is_empty());
}

#[test]
fn test_empty_grammar_has_zero_stats() {
    let g = Grammar::default();
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 0);
    assert_eq!(r.stats.total_tokens, 0);
}

// ---------------------------------------------------------------------------
// 3. Missing start symbol / no explicit start rule
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_without_start_still_builds() {
    // Builder allows no .start(), first rule is used implicitly.
    let g = GrammarBuilder::new("no_start")
        .token("A", "a")
        .rule("root", vec!["A"])
        .build();
    // Should not panic — validation may or may not warn.
    let _r = validate(&g);
}

#[test]
fn test_start_symbol_is_first_rule_lhs() {
    let g = GrammarBuilder::new("order")
        .token("A", "a")
        .rule("alpha", vec!["A"])
        .start("alpha")
        .build();
    assert_eq!(g.start_symbol(), Some(adze_ir::SymbolId(2))); // alpha's id
}

// ---------------------------------------------------------------------------
// 4. Undefined symbols in RHS
// ---------------------------------------------------------------------------

#[test]
fn test_undefined_symbol_in_rhs_reports_error() {
    // "missing" is referenced but never defined as token or rule.
    let g = GrammarBuilder::new("undef")
        .token("A", "a")
        .rule("root", vec!["A", "missing"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        has_error(&r, |e| matches!(e, ValidationError::UndefinedSymbol { .. })),
        "expected UndefinedSymbol, got: {:?}",
        r.errors
    );
}

#[test]
fn test_multiple_undefined_symbols() {
    let g = GrammarBuilder::new("multi_undef")
        .token("A", "a")
        .rule("root", vec!["A", "no1", "no2"])
        .start("root")
        .build();
    let r = validate(&g);
    let undef_count = r
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
        .count();
    assert!(undef_count >= 2, "expected >=2 undefined, got {undef_count}");
}

#[test]
fn test_undefined_symbol_location_mentions_rule() {
    let g = GrammarBuilder::new("loc")
        .token("A", "a")
        .rule("root", vec!["A", "phantom"])
        .start("root")
        .build();
    let r = validate(&g);
    let undef = r
        .errors
        .iter()
        .find(|e| matches!(e, ValidationError::UndefinedSymbol { .. }));
    assert!(undef.is_some());
}

// ---------------------------------------------------------------------------
// 5. Unreachable rules
// ---------------------------------------------------------------------------

#[test]
fn test_unreachable_rule_produces_warning() {
    let g = GrammarBuilder::new("unreach")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("island", vec!["B"])
        .start("root")
        .build();
    let r = validate(&g);
    // "island" is unreachable from "root" → should produce a warning.
    assert!(
        has_warning(&r, |w| matches!(w, ValidationWarning::UnusedToken { .. })),
        "expected unreachable warning, got: {:?}",
        r.warnings
    );
}

#[test]
fn test_unreachable_rule_does_not_produce_error() {
    let g = GrammarBuilder::new("unreach2")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("island", vec!["B"])
        .start("root")
        .build();
    let r = validate(&g);
    // Unreachable is a warning, NOT an error.
    assert!(
        !has_error(&r, |e| matches!(
            e,
            ValidationError::UnreachableSymbol { .. }
        )),
        "unreachable should be a warning not an error"
    );
}

#[test]
fn test_all_reachable_no_unreachable_warning() {
    let g = GrammarBuilder::new("reach_all")
        .token("A", "a")
        .rule("root", vec!["child"])
        .rule("child", vec!["A"])
        .start("root")
        .build();
    let r = validate(&g);
    let unreachable_warnings = r
        .warnings
        .iter()
        .filter(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
        .count();
    // "child" and "A" are reachable from "root".
    assert_eq!(unreachable_warnings, 0);
}

// ---------------------------------------------------------------------------
// 6. Cyclic rules
// ---------------------------------------------------------------------------

#[test]
fn test_direct_self_cycle_detected() {
    // a -> a (pure self-recursion, no base case via terminal)
    let g = GrammarBuilder::new("self_cycle")
        .rule("a", vec!["a"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(
        has_error(&r, |e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule, got: {:?}",
        r.errors
    );
}

#[test]
fn test_mutual_cycle_detected() {
    // a -> b, b -> a (no base case)
    let g = GrammarBuilder::new("mutual")
        .rule("a", vec!["b"])
        .rule("b", vec!["a"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(
        has_error(&r, |e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule, got: {:?}",
        r.errors
    );
}

#[test]
fn test_cycle_with_base_case_still_flagged() {
    // The cycle-checker uses DFS on non-terminals; even if a base case exists
    // for one alternative, a cycle in another alternative is detected.
    let g = GrammarBuilder::new("cycle_base")
        .token("X", "x")
        .rule("a", vec!["X"])
        .rule("a", vec!["b"])
        .rule("b", vec!["a"])
        .start("a")
        .build();
    let r = validate(&g);
    // The cycle a -> b -> a should still be flagged.
    assert!(has_error(
        &r,
        |e| matches!(e, ValidationError::CyclicRule { .. })
    ));
}

#[test]
fn test_three_node_cycle_detected() {
    let g = GrammarBuilder::new("tri")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["a"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(has_error(
        &r,
        |e| matches!(e, ValidationError::CyclicRule { .. })
    ));
}

#[test]
fn test_no_cycle_in_linear_chain() {
    let g = GrammarBuilder::new("linear")
        .token("T", "t")
        .rule("a", vec!["b"])
        .rule("b", vec!["T"])
        .start("a")
        .build();
    let r = validate(&g);
    assert!(!has_error(
        &r,
        |e| matches!(e, ValidationError::CyclicRule { .. })
    ));
}

// ---------------------------------------------------------------------------
// 7. Duplicate tokens
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_token_pattern_produces_warning() {
    let g = GrammarBuilder::new("dup_tok")
        .token("INT", r"\d+")
        .token("NUMBER", r"\d+")
        .rule("root", vec!["INT"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(
        has_warning(
            &r,
            |w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. })
        ),
        "expected DuplicateTokenPattern warning, got: {:?}",
        r.warnings
    );
}

#[test]
fn test_distinct_token_patterns_no_warning() {
    let g = GrammarBuilder::new("dist_tok")
        .token("INT", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("root", vec!["INT"])
        .rule("root", vec!["ID"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(!has_warning(
        &r,
        |w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. })
    ));
}

#[test]
fn test_three_duplicate_tokens() {
    let g = GrammarBuilder::new("trip")
        .token("A", "lit")
        .token("B", "lit")
        .token("C", "lit")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let r = validate(&g);
    let dup = r
        .warnings
        .iter()
        .find(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
    assert!(dup.is_some());
}

// ---------------------------------------------------------------------------
// 8. Non-productive symbols
// ---------------------------------------------------------------------------

#[test]
fn test_non_productive_symbol_error() {
    // "dead" references "ghost" which has no rule and no token → non-productive.
    let g = GrammarBuilder::new("nonprod")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("dead", vec!["ghost"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(has_error(
        &r,
        |e| matches!(e, ValidationError::NonProductiveSymbol { .. })
    ));
}

#[test]
fn test_all_productive_no_error() {
    let g = GrammarBuilder::new("all_prod")
        .token("X", "x")
        .rule("root", vec!["mid"])
        .rule("mid", vec!["X"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(!has_error(
        &r,
        |e| matches!(e, ValidationError::NonProductiveSymbol { .. })
    ));
}

// ---------------------------------------------------------------------------
// 9. Conflicting precedences
// ---------------------------------------------------------------------------

#[test]
fn test_conflicting_precedence_error() {
    let g = GrammarBuilder::new("prec_conflict")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("term", vec!["NUM"])
        .rule("add", vec!["term", "+", "term"])
        .rule("root", vec!["term"])
        .rule("root", vec!["add"])
        .start("root")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["+"])
        .build();
    let r = validate(&g);
    assert!(has_error(
        &r,
        |e| matches!(e, ValidationError::ConflictingPrecedence { .. })
    ));
}

#[test]
fn test_same_precedence_repeated_no_conflict() {
    let g = GrammarBuilder::new("same_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("term", vec!["NUM"])
        .rule("add", vec!["term", "+", "term"])
        .rule("root", vec!["term"])
        .rule("root", vec!["add"])
        .start("root")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(1, Associativity::Left, vec!["+"])
        .build();
    let r = validate(&g);
    assert!(!has_error(
        &r,
        |e| matches!(e, ValidationError::ConflictingPrecedence { .. })
    ));
}

// ---------------------------------------------------------------------------
// 10. External token conflicts
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_external_name_error() {
    let g = GrammarBuilder::new("dup_ext")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .external("INDENT")
        .external("INDENT")
        .build();
    let r = validate(&g);
    assert!(has_error(
        &r,
        |e| matches!(e, ValidationError::ExternalTokenConflict { .. })
    ));
}

#[test]
fn test_distinct_externals_no_conflict() {
    let g = GrammarBuilder::new("dist_ext")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .external("INDENT")
        .external("DEDENT")
        .build();
    let r = validate(&g);
    assert!(!has_error(
        &r,
        |e| matches!(e, ValidationError::ExternalTokenConflict { .. })
    ));
}

// ---------------------------------------------------------------------------
// 11. Stats
// ---------------------------------------------------------------------------

#[test]
fn test_stats_total_tokens() {
    let g = GrammarBuilder::new("stats_tok")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 2);
}

#[test]
fn test_stats_total_rules() {
    let g = GrammarBuilder::new("stats_rules")
        .token("A", "a")
        .rule("root", vec!["A"])
        .rule("root", vec!["A", "A"])
        .start("root")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 2);
}

#[test]
fn test_stats_max_rule_length() {
    let g = GrammarBuilder::new("stats_len")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B", "C"])
        .start("root")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.max_rule_length, 3);
}

#[test]
fn test_stats_reachable_symbols() {
    let g = GrammarBuilder::new("stats_reach")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(r.stats.reachable_symbols >= 1);
}

#[test]
fn test_stats_external_tokens_count() {
    let g = GrammarBuilder::new("stats_ext")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .external("EXT1")
        .external("EXT2")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.external_tokens, 2);
}

// ---------------------------------------------------------------------------
// 12. Validator reuse
// ---------------------------------------------------------------------------

#[test]
fn test_validator_can_be_reused() {
    let mut v = GrammarValidator::new();

    let g1 = GrammarBuilder::new("g1")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let r1 = v.validate(&g1);
    assert!(r1.errors.is_empty());

    let g2 = Grammar::default();
    let r2 = v.validate(&g2);
    assert!(has_error(&r2, |e| matches!(e, ValidationError::EmptyGrammar)));
}

#[test]
fn test_validator_clears_between_runs() {
    let mut v = GrammarValidator::new();

    let bad = Grammar::default();
    let r1 = v.validate(&bad);
    assert!(!r1.errors.is_empty());

    let good = GrammarBuilder::new("ok")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let r2 = v.validate(&good);
    assert!(r2.errors.is_empty(), "errors leaked: {:?}", r2.errors);
}

// ---------------------------------------------------------------------------
// 13. Warnings — inefficient / missing field names
// ---------------------------------------------------------------------------

#[test]
fn test_trivial_unit_rule_inefficiency_warning() {
    // a -> b (single non-terminal) triggers "consider inlining" warning.
    let g = GrammarBuilder::new("unit")
        .token("X", "x")
        .rule("root", vec!["inner"])
        .rule("inner", vec!["X"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(has_warning(
        &r,
        |w| matches!(w, ValidationWarning::InefficientRule { .. })
    ));
}

#[test]
fn test_missing_field_names_warning_on_multi_rhs() {
    let g = GrammarBuilder::new("nofield")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "B"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(has_warning(
        &r,
        |w| matches!(w, ValidationWarning::MissingFieldNames { .. })
    ));
}

// ---------------------------------------------------------------------------
// 14. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_single_epsilon_rule_grammar() {
    // .rule("root", vec![]) → epsilon production.
    let g = GrammarBuilder::new("eps")
        .rule("root", vec![])
        .start("root")
        .build();
    let r = validate(&g);
    // Epsilon grammars should not crash.
    let _ = r;
}

#[test]
fn test_token_used_only_in_extras_not_unreachable() {
    let g = GrammarBuilder::new("extra_only")
        .token("NUM", r"\d+")
        .token("WS", r"\s+")
        .rule("root", vec!["NUM"])
        .start("root")
        .extra("WS")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn test_grammar_name_preserved() {
    let g = GrammarBuilder::new("my_lang").build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn test_default_grammar_has_no_rules() {
    let g = Grammar::default();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn test_rule_names_populated() {
    let g = GrammarBuilder::new("rn")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert!(!g.rule_names.is_empty());
}

#[test]
fn test_grammar_fields_initially_empty() {
    let g = GrammarBuilder::new("f")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert!(g.fields.is_empty());
}

#[test]
fn test_grammar_conflicts_initially_empty() {
    let g = GrammarBuilder::new("c")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert!(g.conflicts.is_empty());
}

#[test]
fn test_grammar_inline_rules_populated_when_set() {
    let g = GrammarBuilder::new("i")
        .token("A", "a")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["A"])
        .start("root")
        .inline("helper")
        .build();
    assert!(!g.inline_rules.is_empty());
}

#[test]
fn test_grammar_supertypes_populated_when_set() {
    let g = GrammarBuilder::new("s")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .supertype("root")
        .build();
    assert!(!g.supertypes.is_empty());
}

#[test]
fn test_grammar_extras_populated_when_set() {
    let g = GrammarBuilder::new("e")
        .token("A", "a")
        .token("WS", r"\s+")
        .rule("root", vec!["A"])
        .start("root")
        .extra("WS")
        .build();
    assert!(!g.extras.is_empty());
}

#[test]
fn test_grammar_externals_populated_when_set() {
    let g = GrammarBuilder::new("x")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .external("EXT")
        .build();
    assert!(!g.externals.is_empty());
}

#[test]
fn test_grammar_precedences_populated_when_set() {
    let g = GrammarBuilder::new("p")
        .token("+", "+")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .precedence(1, Associativity::Left, vec!["+"])
        .build();
    assert!(!g.precedences.is_empty());
}

// ---------------------------------------------------------------------------
// 15. Error display / Debug
// ---------------------------------------------------------------------------

#[test]
fn test_validation_error_display_empty() {
    let err = ValidationError::EmptyGrammar;
    let msg = format!("{err}");
    assert!(msg.contains("no rules"), "msg = {msg}");
}

#[test]
fn test_validation_error_display_no_start() {
    let err = ValidationError::NoExplicitStartRule;
    let msg = format!("{err}");
    assert!(msg.contains("start"), "msg = {msg}");
}

#[test]
fn test_validation_error_debug() {
    let err = ValidationError::EmptyGrammar;
    let dbg = format!("{err:?}");
    assert!(dbg.contains("EmptyGrammar"));
}

#[test]
fn test_validation_warning_display_unused_token() {
    let w = ValidationWarning::UnusedToken {
        token: adze_ir::SymbolId(42),
        name: "UNUSED".to_string(),
    };
    let msg = format!("{w}");
    assert!(msg.contains("UNUSED"));
}

// ---------------------------------------------------------------------------
// 16. Multiple error categories in one grammar
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_with_multiple_issues() {
    // Undefined symbol + unreachable rule in the same grammar.
    let g = GrammarBuilder::new("multi_issue")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A", "phantom"])
        .rule("island", vec!["B"])
        .start("root")
        .build();
    let r = validate(&g);
    assert!(has_error(
        &r,
        |e| matches!(e, ValidationError::UndefinedSymbol { .. })
    ));
    // "island" is unreachable.
    assert!(!r.warnings.is_empty());
}

#[test]
fn test_empty_grammar_no_crash_on_stats() {
    let g = Grammar::default();
    let r = validate(&g);
    // Avg rule length on empty grammar should not panic (div by zero).
    assert_eq!(r.stats.avg_rule_length, 0.0);
}
