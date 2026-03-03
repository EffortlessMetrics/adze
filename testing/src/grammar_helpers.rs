//! Convenience helpers for quickly creating test grammars.
//!
//! Re-exports and wraps [`adze_ir::builder::GrammarBuilder`] with additional
//! shorthand constructors used throughout the test suite.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

// Re-export for convenience so downstream tests only need `use adze_testing::grammar_helpers::*`.
pub use adze_ir::builder::GrammarBuilder as Builder;

/// Create a minimal grammar from a list of `(lhs, rhs)` rule pairs.
///
/// The first rule's LHS is used as the start symbol.  Any symbol that
/// appears in an RHS but is never on the LHS is assumed to be a terminal
/// and an identity-pattern token is registered automatically.
///
/// # Examples
///
/// ```
/// use adze_testing::grammar_helpers::test_grammar;
///
/// let g = test_grammar(&[
///     ("sum", &["NUMBER", "+", "NUMBER"]),
/// ]);
/// assert_eq!(g.name, "test");
/// ```
pub fn test_grammar(rules: &[(&str, &[&str])]) -> Grammar {
    assert!(!rules.is_empty(), "test_grammar requires at least one rule");

    // Collect all LHS names so we can distinguish terminals from non-terminals.
    let lhs_names: std::collections::HashSet<&str> = rules.iter().map(|(lhs, _)| *lhs).collect();

    let mut builder = GrammarBuilder::new("test");

    // Register tokens for every RHS symbol that never appears as an LHS.
    let mut registered = std::collections::HashSet::new();
    for (_, rhs) in rules {
        for &sym in *rhs {
            if !lhs_names.contains(sym) && registered.insert(sym) {
                builder = builder.token(sym, sym);
            }
        }
    }

    // Add rules and set the first LHS as the start symbol.
    let start = rules[0].0;
    for (lhs, rhs) in rules {
        builder = builder.rule(lhs, rhs.to_vec());
    }

    builder.start(start).build()
}

/// Create an arithmetic expression grammar suitable for most parser tests.
///
/// The grammar supports `+`, `-`, `*`, `/` with standard precedence
/// and left-associativity, parenthesised sub-expressions, and `NUMBER`
/// literals.
pub fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Create a trivially small grammar with a single terminal rule.
///
/// Useful when the test only needs *some* valid grammar and its shape
/// does not matter.
pub fn trivial_grammar() -> Grammar {
    GrammarBuilder::new("trivial")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Build a [`ParseTable`] from a grammar, returning a descriptive error on failure.
///
/// This is a convenience wrapper around [`FirstFollowSets::compute`] +
/// [`build_lr1_automaton`] that most unit tests can call without importing
/// the GLR core crate directly.
pub fn build_parse_table(grammar: &Grammar) -> Result<ParseTable, String> {
    let ff = FirstFollowSets::compute(grammar).map_err(|e| format!("FIRST/FOLLOW: {e}"))?;
    let table = build_lr1_automaton(grammar, &ff).map_err(|e| format!("LR(1): {e}"))?;
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_creates_valid_grammar() {
        let g = test_grammar(&[("sum", &["NUMBER", "+", "NUMBER"])]);
        assert_eq!(g.name, "test");
        assert!(!g.tokens.is_empty());
        assert!(!g.rules.is_empty());
    }

    #[test]
    fn test_grammar_multi_rule() {
        let g = test_grammar(&[("expr", &["NUMBER"]), ("expr", &["expr", "+", "expr"])]);
        // "expr" is the start symbol (first rule's LHS)
        let start = g.start_symbol();
        assert!(start.is_some());
    }

    #[test]
    fn arithmetic_grammar_builds_parse_table() {
        let g = arithmetic_grammar();
        let table = build_parse_table(&g);
        assert!(table.is_ok(), "parse table build failed: {:?}", table.err());
        assert!(table.unwrap().state_count > 0);
    }

    #[test]
    fn trivial_grammar_is_valid() {
        let g = trivial_grammar();
        assert_eq!(g.name, "trivial");
        let table = build_parse_table(&g);
        assert!(table.is_ok());
    }
}
