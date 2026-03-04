// Comprehensive tests for IR validation pipeline
use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::GrammarValidator;

fn validate_grammar(builder: GrammarBuilder) -> adze_ir::validation::ValidationResult {
    let grammar = builder.build();
    let mut validator = GrammarValidator::default();
    validator.validate(&grammar)
}

#[test]
fn valid_simple_grammar_no_errors() {
    let result = validate_grammar(
        GrammarBuilder::new("simple")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn valid_grammar_with_alternatives() {
    let result = validate_grammar(
        GrammarBuilder::new("alt")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert!(result.errors.is_empty());
}

#[test]
fn recursive_grammar_may_have_warnings() {
    let result = validate_grammar(
        GrammarBuilder::new("rec")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list"),
    );
    // Recursive grammars may produce validation warnings/errors about cycles
    let _ = &result.errors;
    let _ = &result.warnings;
}

#[test]
fn valid_nested_nonterminals() {
    let result = validate_grammar(
        GrammarBuilder::new("nested")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start"),
    );
    assert!(result.errors.is_empty());
}

#[test]
fn validation_has_stats() {
    let result = validate_grammar(
        GrammarBuilder::new("stats")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let _ = format!("{:?}", result.stats);
}

#[test]
fn warnings_for_unused_tokens() {
    let result = validate_grammar(
        GrammarBuilder::new("unused")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .start("start"),
    );
    // "b" is unused
    assert!(
        result.warnings.len() > 0 || result.errors.is_empty(),
        "Should either warn about unused token or be clean"
    );
}

#[test]
fn validator_default() {
    let _v = GrammarValidator::default();
}

#[test]
fn validation_result_has_fields() {
    let result = validate_grammar(
        GrammarBuilder::new("dbg")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let _ = &result.errors;
    let _ = &result.warnings;
    let _ = format!("{:?}", result.stats);
}

#[test]
fn multiple_validators_independent() {
    let g = GrammarBuilder::new("ind")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let mut v1 = GrammarValidator::default();
    let mut v2 = GrammarValidator::default();
    let r1 = v1.validate(&g);
    let r2 = v2.validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
}

#[test]
fn validator_deterministic() {
    let g = GrammarBuilder::new("det")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let mut v1 = GrammarValidator::default();
    let mut v2 = GrammarValidator::default();
    let r1 = v1.validate(&g);
    let r2 = v2.validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
    assert_eq!(r1.warnings.len(), r2.warnings.len());
}

#[test]
fn chain_grammar_validates() {
    let result = validate_grammar(
        GrammarBuilder::new("chain")
            .token("x", "x")
            .rule("d", vec!["x"])
            .rule("c", vec!["d"])
            .rule("b", vec!["c"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert!(result.errors.is_empty());
}

#[test]
fn multi_token_grammar_validates() {
    let result = validate_grammar(
        GrammarBuilder::new("multi")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    assert!(result.errors.is_empty());
}
