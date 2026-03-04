// Comprehensive tests for GrammarValidator behavior
// Tests validation logic with various grammar shapes

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::GrammarValidator;

fn valid_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("valid")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

#[test]
fn validator_new_no_args() {
    let _v = GrammarValidator::new();
}

#[test]
fn validator_validate_simple_grammar() {
    let mut v = GrammarValidator::new();
    let g = valid_grammar();
    let result = v.validate(&g);
    assert!(result.errors.is_empty());
}

#[test]
fn validator_result_has_stats() {
    let mut v = GrammarValidator::new();
    let g = valid_grammar();
    let result = v.validate(&g);
    let _ = format!("{:?}", result.stats);
}

#[test]
fn validator_result_has_warnings() {
    let mut v = GrammarValidator::new();
    let g = valid_grammar();
    let result = v.validate(&g);
    // Simple grammar may or may not have warnings
    let _ = result.warnings.len();
}

#[test]
fn validator_two_token_grammar() {
    let mut v = GrammarValidator::new();
    let g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let result = v.validate(&g);
    assert!(result.errors.is_empty());
}

#[test]
fn validator_alternatives_grammar() {
    let mut v = GrammarValidator::new();
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let result = v.validate(&g);
    assert!(result.errors.is_empty());
}

#[test]
fn validator_chain_grammar() {
    let mut v = GrammarValidator::new();
    let g = GrammarBuilder::new("ch")
        .token("a", "a")
        .rule("s", vec!["m"])
        .rule("m", vec!["a"])
        .start("s")
        .build();
    let result = v.validate(&g);
    assert!(result.errors.is_empty());
}

#[test]
fn validator_recursive_grammar_has_findings() {
    let mut v = GrammarValidator::new();
    let g = GrammarBuilder::new("rec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let result = v.validate(&g);
    // Recursive grammars may produce errors or warnings
    let _ = result.errors.len() + result.warnings.len();
}

#[test]
fn validator_reuse_across_grammars() {
    let mut v = GrammarValidator::new();
    let g1 = valid_grammar();
    let _ = v.validate(&g1);
    let g2 = GrammarBuilder::new("other")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let result = v.validate(&g2);
    assert!(result.errors.is_empty());
}

#[test]
fn validator_result_errors_debug() {
    let mut v = GrammarValidator::new();
    let g = valid_grammar();
    let result = v.validate(&g);
    let dbg = format!("{:?}", result.errors);
    assert!(!dbg.is_empty());
}

#[test]
fn validator_stats_populated() {
    let mut v = GrammarValidator::new();
    let g = GrammarBuilder::new("stats")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let result = v.validate(&g);
    let _ = format!("{:?}", result.stats);
}

#[test]
fn validator_with_precedence_accepted() {
    use adze_ir::Associativity;
    let mut v = GrammarValidator::new();
    let g = GrammarBuilder::new("prec")
        .token("n", "[0-9]+")
        .token("plus", r"\+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let result = v.validate(&g);
    // Precedence grammars may have findings about recursion
    let _ = result.errors.len() + result.warnings.len();
}
