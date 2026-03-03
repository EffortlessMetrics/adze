#![allow(clippy::needless_range_loop)]

//! Property-based tests for Grammar validation edge cases in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{
    ExternalToken, FieldId, Grammar, Precedence, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_rule(lhs: u16, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn make_token(id: u16, name: &str, pattern: &str) -> (SymbolId, Token) {
    (
        SymbolId(id),
        Token {
            name: name.to_string(),
            pattern: TokenPattern::String(pattern.to_string()),
            fragile: false,
        },
    )
}

fn minimal_valid_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn has_error(result: &adze_ir::validation::ValidationResult, pred: impl Fn(&ValidationError) -> bool) -> bool {
    result.errors.iter().any(pred)
}

fn has_warning(
    result: &adze_ir::validation::ValidationResult,
    pred: impl Fn(&ValidationWarning) -> bool,
) -> bool {
    result.warnings.iter().any(pred)
}

// ---------------------------------------------------------------------------
// 1. Valid grammar passes validation (minimal)
// ---------------------------------------------------------------------------

#[test]
fn test_01_minimal_valid_grammar_passes() {
    let g = minimal_valid_grammar("t01");
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ---------------------------------------------------------------------------
// 2. Valid grammar passes for arbitrary names
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_02_valid_grammar_arbitrary_name(name in "[a-z]{1,16}") {
        let g = minimal_valid_grammar(&name);
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        prop_assert!(r.errors.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3. Grammar missing start rule (empty) fails
// ---------------------------------------------------------------------------

#[test]
fn test_03_empty_grammar_fails_validation() {
    let g = Grammar::new("empty".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)));
}

// ---------------------------------------------------------------------------
// 4. Empty grammar for any name
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_04_empty_grammar_any_name_fails(name in "[a-zA-Z_]{0,20}") {
        let g = Grammar::new(name);
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        prop_assert!(r.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)));
    }
}

// ---------------------------------------------------------------------------
// 5. Grammar with duplicate external token names
// ---------------------------------------------------------------------------

#[test]
fn test_05_duplicate_external_tokens_detected() {
    let mut g = minimal_valid_grammar("dup");
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(100),
    });
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(101),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::ExternalTokenConflict { .. }
    )));
}

// ---------------------------------------------------------------------------
// 6. Arbitrary count of duplicate externals
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_06_n_duplicate_externals(count in 2u16..6) {
        let mut g = minimal_valid_grammar("dup_n");
        for i in 0..count {
            g.externals.push(ExternalToken {
                name: "SAME".into(),
                symbol_id: SymbolId(200 + i),
            });
        }
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        let has_conflict = r.errors.iter().any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }));
        prop_assert!(has_conflict);
    }
}

// ---------------------------------------------------------------------------
// 7. Undefined nonterminal symbol reference
// ---------------------------------------------------------------------------

#[test]
fn test_07_undefined_nonterminal_detected() {
    let mut g = Grammar::new("undef".into());
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(99))], 0));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(99)
    )));
}

// ---------------------------------------------------------------------------
// 8. Undefined terminal symbol reference
// ---------------------------------------------------------------------------

#[test]
fn test_08_undefined_terminal_detected() {
    let mut g = Grammar::new("undef_term".into());
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(50))], 0));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(50)
    )));
}

// ---------------------------------------------------------------------------
// 9. Random undefined symbol IDs detected
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_09_random_undefined_id(id in 50u16..500) {
        let mut g = Grammar::new("rnd".into());
        g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(id))], 0));
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        let has_undef = r.errors.iter().any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }));
        prop_assert!(has_undef);
    }
}

// ---------------------------------------------------------------------------
// 10. Circular references: A -> B, B -> A (non-productive)
// ---------------------------------------------------------------------------

#[test]
fn test_10_circular_two_symbols_non_productive() {
    let mut g = Grammar::new("circ2".into());
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0));
    g.add_rule(make_rule(2, vec![Symbol::NonTerminal(SymbolId(1))], 1));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::NonProductiveSymbol { .. }
    )));
}

// ---------------------------------------------------------------------------
// 11. Circular references: A -> B -> C -> A
// ---------------------------------------------------------------------------

#[test]
fn test_11_circular_three_symbols() {
    let mut g = Grammar::new("circ3".into());
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 0));
    g.add_rule(make_rule(2, vec![Symbol::NonTerminal(SymbolId(3))], 1));
    g.add_rule(make_rule(3, vec![Symbol::NonTerminal(SymbolId(1))], 2));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

// ---------------------------------------------------------------------------
// 12. Self-recursive with base case IS productive
// ---------------------------------------------------------------------------

#[test]
fn test_12_self_recursive_with_base_productive() {
    let g = GrammarBuilder::new("rec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. })),
        "recursive with base should be productive"
    );
}

// ---------------------------------------------------------------------------
// 13. Self-referencing rule without base case is non-productive
// ---------------------------------------------------------------------------

#[test]
fn test_13_self_recursive_no_base_non_productive() {
    let mut g = Grammar::new("selfloop".into());
    // A -> A (no terminal base case)
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(1))], 0));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::NonProductiveSymbol { .. }
    )));
}

// ---------------------------------------------------------------------------
// 14. Proptest: circular chains of length N are non-productive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_14_circular_chain_non_productive(n in 2u16..8) {
        let mut g = Grammar::new("chain".into());
        for i in 0..n {
            let next = (i + 1) % n;
            g.add_rule(make_rule(i + 1, vec![Symbol::NonTerminal(SymbolId(next + 1))], i));
        }
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        let has_np = r.errors.iter().any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }));
        prop_assert!(has_np);
    }
}

// ---------------------------------------------------------------------------
// 15. Empty rules (epsilon-only) are valid for the grammar
// ---------------------------------------------------------------------------

#[test]
fn test_15_epsilon_rule_passes_validation() {
    let mut g = Grammar::new("eps".into());
    let (tid, tok) = make_token(2, "x", "x");
    g.tokens.insert(tid, tok);
    // Start rule has a terminal alternative
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(2))], 0));
    // Also has an epsilon alternative
    g.add_rule(make_rule(1, vec![Symbol::Epsilon], 1));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

// ---------------------------------------------------------------------------
// 16. Grammar with only epsilon rule (no terminals) is still "non-empty"
// ---------------------------------------------------------------------------

#[test]
fn test_16_epsilon_only_rule_not_empty_grammar() {
    let mut g = Grammar::new("eps_only".into());
    g.add_rule(make_rule(1, vec![Symbol::Epsilon], 0));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    // Should NOT be flagged as EmptyGrammar since it has a rule
    assert!(!has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)));
}

// ---------------------------------------------------------------------------
// 17. Error message: EmptyGrammar
// ---------------------------------------------------------------------------

#[test]
fn test_17_empty_grammar_error_message() {
    let msg = format!("{}", ValidationError::EmptyGrammar);
    assert!(msg.contains("no rules"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 18. Error message: UndefinedSymbol
// ---------------------------------------------------------------------------

#[test]
fn test_18_undefined_symbol_error_message() {
    let err = ValidationError::UndefinedSymbol {
        symbol: SymbolId(42),
        location: "rule for expr".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Undefined"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 19. Error message: NonProductiveSymbol
// ---------------------------------------------------------------------------

#[test]
fn test_19_non_productive_error_message() {
    let err = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(5),
        name: "broken".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("broken"), "got: {msg}");
    assert!(msg.contains("terminal"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 20. Error message: CyclicRule
// ---------------------------------------------------------------------------

#[test]
fn test_20_cyclic_rule_error_message() {
    let err = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("Cyclic") || msg.contains("cyclic") || msg.contains("dependency"),
        "got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// 21. Error message: DuplicateRule
// ---------------------------------------------------------------------------

#[test]
fn test_21_duplicate_rule_error_message() {
    let err = ValidationError::DuplicateRule {
        symbol: SymbolId(3),
        existing_count: 2,
    };
    let msg = format!("{err}");
    assert!(msg.contains("2"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 22. Error message: ExternalTokenConflict
// ---------------------------------------------------------------------------

#[test]
fn test_22_external_token_conflict_error_message() {
    let err = ValidationError::ExternalTokenConflict {
        token1: "INDENT".into(),
        token2: "INDENT".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("INDENT"), "got: {msg}");
    assert!(msg.contains("conflict") || msg.contains("Conflict"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 23. Error message: InvalidRegex
// ---------------------------------------------------------------------------

#[test]
fn test_23_invalid_regex_error_message() {
    let err = ValidationError::InvalidRegex {
        token: SymbolId(7),
        pattern: "[bad".into(),
        error: "unclosed bracket".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("[bad"), "got: {msg}");
    assert!(msg.contains("unclosed"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 24. Undefined symbol inside Optional wrapper
// ---------------------------------------------------------------------------

#[test]
fn test_24_undefined_inside_optional() {
    let mut g = Grammar::new("opt_undef".into());
    g.add_rule(make_rule(
        1,
        vec![Symbol::Optional(Box::new(Symbol::NonTerminal(SymbolId(77))))],
        0,
    ));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(77)
    )));
}

// ---------------------------------------------------------------------------
// 25. Undefined symbol inside Repeat wrapper
// ---------------------------------------------------------------------------

#[test]
fn test_25_undefined_inside_repeat() {
    let mut g = Grammar::new("rep_undef".into());
    g.add_rule(make_rule(
        1,
        vec![Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(88))))],
        0,
    ));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(88)
    )));
}

// ---------------------------------------------------------------------------
// 26. Undefined symbol inside Choice
// ---------------------------------------------------------------------------

#[test]
fn test_26_undefined_inside_choice() {
    let mut g = Grammar::new("choice_undef".into());
    let (tid, tok) = make_token(2, "ok", "ok");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(
        1,
        vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(55)),
        ])],
        0,
    ));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(55)
    )));
}

// ---------------------------------------------------------------------------
// 27. Undefined symbol inside Sequence
// ---------------------------------------------------------------------------

#[test]
fn test_27_undefined_inside_sequence() {
    let mut g = Grammar::new("seq_undef".into());
    let (tid, tok) = make_token(2, "ok", "ok");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(
        1,
        vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(66)),
        ])],
        0,
    ));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(66)
    )));
}

// ---------------------------------------------------------------------------
// 28. Grammar.validate() on empty grammar succeeds (no rules to check)
// ---------------------------------------------------------------------------

#[test]
fn test_28_grammar_validate_method_empty_ok() {
    let g = Grammar::new("empty".into());
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// 29. Grammar.validate() detects unresolved symbol
// ---------------------------------------------------------------------------

#[test]
fn test_29_grammar_validate_unresolved() {
    let mut g = Grammar::new("bad".into());
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(999))], 0));
    assert!(g.validate().is_err());
}

// ---------------------------------------------------------------------------
// 30. Grammar.validate() detects bad field ordering
// ---------------------------------------------------------------------------

#[test]
fn test_30_grammar_validate_bad_field_ordering() {
    let mut g = Grammar::new("fields".into());
    g.fields.insert(FieldId(0), "zebra".into());
    g.fields.insert(FieldId(1), "alpha".into());
    assert!(g.validate().is_err());
}

// ---------------------------------------------------------------------------
// 31. Grammar with many rules all valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_31_many_rules_all_valid(n in 1usize..20) {
        let mut b = GrammarBuilder::new("many")
            .token("NUM", r"\d+")
            .token("+", "+");
        b = b.rule("expr", vec!["NUM"]);
        for _ in 1..n {
            b = b.rule("expr", vec!["expr", "+", "NUM"]);
        }
        let g = b.start("expr").build();
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        let non_cycle: Vec<_> = r.errors.iter()
            .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
            .collect();
        prop_assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
    }
}

// ---------------------------------------------------------------------------
// 32. Validation does not modify grammar
// ---------------------------------------------------------------------------

#[test]
fn test_32_validation_does_not_modify_grammar() {
    let g = minimal_valid_grammar("immut");
    let before = g.clone();
    let mut v = GrammarValidator::new();
    let _ = v.validate(&g);
    assert_eq!(g, before);
}

// ---------------------------------------------------------------------------
// 33. Stats reflect correct token count
// ---------------------------------------------------------------------------

#[test]
fn test_33_stats_token_count() {
    let g = GrammarBuilder::new("stats")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert_eq!(r.stats.total_tokens, 2);
}

// ---------------------------------------------------------------------------
// 34. Stats reflect correct rule count
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_34_stats_rule_count(n in 1usize..8) {
        let mut b = GrammarBuilder::new("st")
            .token("NUM", r"\d+")
            .token("+", "+");
        b = b.rule("expr", vec!["NUM"]);
        for _ in 1..n {
            b = b.rule("expr", vec!["expr", "+", "NUM"]);
        }
        let g = b.start("expr").build();
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        prop_assert_eq!(r.stats.total_rules, n);
    }
}

// ---------------------------------------------------------------------------
// 35. Duplicate token patterns generate warnings
// ---------------------------------------------------------------------------

#[test]
fn test_35_duplicate_token_patterns_warning() {
    let mut g = Grammar::new("dup_tok".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "plus1".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "plus2".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.add_rule(make_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0));
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+"
    )));
}
