#![allow(clippy::needless_range_loop)]

//! Property-based tests for Grammar::validate() and GrammarValidator in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, Precedence, PrecedenceKind, ProductionId, Rule,
    Symbol, SymbolId, Token, TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..200).prop_map(SymbolId)
}

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(|_| TokenPattern::Regex(r"[a-z]+".to_string())),
        any::<String>()
            .prop_filter("non-empty", |s| !s.is_empty())
            .prop_map(|s| {
                let sanitized: String = s.chars().filter(|c| c.is_alphanumeric()).take(8).collect();
                if sanitized.is_empty() {
                    TokenPattern::String("x".to_string())
                } else {
                    TokenPattern::String(sanitized)
                }
            }),
    ]
}

fn arb_token() -> impl Strategy<Value = (String, Token)> {
    ("[a-z]{1,8}", arb_token_pattern()).prop_map(|(name, pattern)| {
        (
            name.clone(),
            Token {
                name,
                pattern,
                fragile: false,
            },
        )
    })
}

/// Build a minimal valid grammar: one rule S -> T where T is a token.
fn minimal_valid_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Build a grammar with N terminal-only rules sharing one token.
fn multi_rule_grammar(n: usize) -> Grammar {
    let mut b = GrammarBuilder::new("multi")
        .token("NUMBER", r"\d+")
        .token("+", "+");
    // First alternative: expr -> NUMBER
    b = b.rule("expr", vec!["NUMBER"]);
    for _ in 1..n {
        b = b.rule("expr", vec!["expr", "+", "NUMBER"]);
    }
    b.start("expr").build()
}

// ---------------------------------------------------------------------------
// 1–6  Valid grammars pass validation
// ---------------------------------------------------------------------------

#[test]
fn test_01_minimal_valid_grammar_no_errors() {
    let g = minimal_valid_grammar("t01");
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

proptest! {
    #[test]
    fn test_02_valid_grammar_name_irrelevant(name in "[a-z]{1,20}") {
        let g = minimal_valid_grammar(&name);
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        prop_assert!(r.errors.is_empty());
    }

    #[test]
    fn test_03_valid_grammar_multiple_alternatives(n in 1usize..6) {
        let g = multi_rule_grammar(n);
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        // Recursive grammars legitimately trigger CyclicRule; filter those out.
        let non_cycle: Vec<_> = r.errors.iter()
            .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
            .collect();
        prop_assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
    }
}

#[test]
fn test_04_javascript_like_grammar_valid() {
    let g = GrammarBuilder::javascript_like();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    // Recursive grammars legitimately trigger CyclicRule; filter those out.
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_05_python_like_grammar_valid() {
    let g = GrammarBuilder::python_like();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    let non_cycle: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(non_cycle.is_empty(), "errors: {:?}", non_cycle);
}

#[test]
fn test_06_valid_grammar_with_externals() {
    let g = GrammarBuilder::new("ext")
        .token("NUM", r"\d+")
        .external("INDENT")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ---------------------------------------------------------------------------
// 7–9  Empty grammar / empty rule set
// ---------------------------------------------------------------------------

#[test]
fn test_07_empty_grammar_fails() {
    let g = Grammar::new("empty".to_string());
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)));
}

#[test]
fn test_08_default_grammar_is_empty() {
    let g = Grammar::default();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)));
}

proptest! {
    #[test]
    fn test_09_empty_grammar_any_name(name in ".*") {
        let g = Grammar::new(name);
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        prop_assert!(r.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)));
    }
}

// ---------------------------------------------------------------------------
// 10–13  Missing start rule / no explicit start
// ---------------------------------------------------------------------------

#[test]
fn test_10_grammar_validate_method_empty() {
    let g = Grammar::new("empty".to_string());
    // Grammar::validate() (the method on Grammar itself) should succeed for
    // an empty grammar (no rules to be invalid).
    assert!(g.validate().is_ok());
}

#[test]
fn test_11_grammar_validate_unresolved_symbol() {
    let mut g = Grammar::new("bad".to_string());
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(999))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(g.validate().is_err());
}

#[test]
fn test_12_grammar_validate_field_ordering() {
    let mut g = Grammar::new("fields".to_string());
    g.fields.insert(FieldId(0), "zebra".to_string());
    g.fields.insert(FieldId(1), "alpha".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn test_13_grammar_validate_field_ordering_correct() {
    let mut g = minimal_valid_grammar("ok");
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "zebra".to_string());
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// 14–16  Duplicate rule names / external token conflicts
// ---------------------------------------------------------------------------

#[test]
fn test_14_duplicate_external_token_detected() {
    let mut g = minimal_valid_grammar("dup_ext");
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(101),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. })),
        "expected ExternalTokenConflict, got: {:?}",
        r.errors
    );
}

proptest! {
    #[test]
    fn test_15_duplicate_external_names_detected(count in 2usize..5) {
        let mut g = minimal_valid_grammar("dup");
        for i in 0..count {
            g.externals.push(ExternalToken {
                name: "SAME".to_string(),
                symbol_id: SymbolId(200 + i as u16),
            });
        }
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        let has_conflict = r.errors.iter().any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }));
        prop_assert!(has_conflict);
    }
}

#[test]
fn test_16_distinct_externals_no_conflict() {
    let mut g = minimal_valid_grammar("no_dup");
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    g.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(101),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

// ---------------------------------------------------------------------------
// 17–21  Referenced symbols exist / undefined symbol detection
// ---------------------------------------------------------------------------

#[test]
fn test_17_undefined_nonterminal_detected() {
    let mut g = Grammar::new("undef".to_string());
    let expr = SymbolId(1);
    let undef = SymbolId(99);
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)));
}

#[test]
fn test_18_undefined_terminal_detected() {
    let mut g = Grammar::new("undef_term".to_string());
    let start = SymbolId(1);
    let undef = SymbolId(50);
    g.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)));
}

proptest! {
    #[test]
    fn test_19_random_undefined_id_detected(id in 50u16..500) {
        let mut g = Grammar::new("rnd".to_string());
        g.add_rule(Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::NonTerminal(SymbolId(id))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        let mut v = GrammarValidator::new();
        let r = v.validate(&g);
        let has_undef = r.errors.iter().any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }));
        prop_assert!(has_undef);
    }
}

#[test]
fn test_20_defined_token_not_flagged_undefined() {
    let g = minimal_valid_grammar("ok");
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(!r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. })));
}

#[test]
fn test_21_nested_optional_undefined() {
    let mut g = Grammar::new("nested".to_string());
    let start = SymbolId(1);
    let undef = SymbolId(77);
    g.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Optional(Box::new(Symbol::NonTerminal(undef)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef)));
}

// ---------------------------------------------------------------------------
// 22–25  Validation error messages are descriptive
// ---------------------------------------------------------------------------

#[test]
fn test_22_empty_grammar_error_message() {
    let msg = format!("{}", ValidationError::EmptyGrammar);
    assert!(msg.contains("no rules"), "got: {msg}");
}

#[test]
fn test_23_undefined_symbol_error_message() {
    let err = ValidationError::UndefinedSymbol {
        symbol: SymbolId(42),
        location: "rule for expr".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Undefined"), "got: {msg}");
    assert!(msg.contains("42") || msg.contains("SymbolId"), "got: {msg}");
}

#[test]
fn test_24_non_productive_error_message() {
    let err = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(5),
        name: "broken".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("broken"), "got: {msg}");
    assert!(msg.contains("terminal"), "got: {msg}");
}

#[test]
fn test_25_conflicting_precedence_message() {
    let err = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(3),
        precedences: vec![1, 5],
    };
    let msg = format!("{err}");
    assert!(msg.contains("conflicting") || msg.contains("Conflicting"), "got: {msg}");
}

// ---------------------------------------------------------------------------
// 26–28  Validation doesn't modify grammar
// ---------------------------------------------------------------------------

#[test]
fn test_26_validate_does_not_modify_grammar() {
    let g = minimal_valid_grammar("immut");
    let before = g.clone();
    let mut v = GrammarValidator::new();
    let _ = v.validate(&g);
    assert_eq!(g, before);
}

proptest! {
    #[test]
    fn test_27_validate_preserves_grammar_across_runs(n in 1usize..4) {
        let g = multi_rule_grammar(n);
        let before = g.clone();
        let mut v = GrammarValidator::new();
        let _ = v.validate(&g);
        let _ = v.validate(&g);
        prop_assert_eq!(g, before);
    }
}

#[test]
fn test_28_grammar_validate_method_does_not_modify() {
    let g = minimal_valid_grammar("immut2");
    let before = g.clone();
    let _ = g.validate();
    assert_eq!(g, before);
}

// ---------------------------------------------------------------------------
// 29–31  Non-productive / cyclic rules
// ---------------------------------------------------------------------------

#[test]
fn test_29_mutually_recursive_non_productive() {
    let mut g = Grammar::new("cycle".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. })));
}

#[test]
fn test_30_self_recursive_with_base_case_is_productive() {
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
        "errors: {:?}",
        r.errors
    );
}

#[test]
fn test_31_cyclic_rule_detected() {
    let mut g = Grammar::new("cyc".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r
        .errors
        .iter()
        .any(|e| matches!(e, ValidationError::CyclicRule { .. })));
}

// ---------------------------------------------------------------------------
// 32–35  Stats, warnings, and edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_32_stats_populated_for_valid_grammar() {
    let g = minimal_valid_grammar("stats");
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.stats.total_rules > 0);
    assert!(r.stats.total_tokens > 0);
    assert!(r.stats.total_symbols > 0);
}

#[test]
fn test_33_invalid_field_index_detected() {
    let mut g = minimal_valid_grammar("fields");
    // Patch a rule to have an out-of-bounds field index
    if let Some(rules) = g.rules.values_mut().next() {
        if let Some(rule) = rules.first_mut() {
            rule.fields.push((FieldId(0), 999)); // index 999 >> rhs.len()
        }
    }
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. })),
        "errors: {:?}",
        r.errors
    );
}

#[test]
fn test_34_conflicting_precedence_detected() {
    let mut g = minimal_valid_grammar("prec");
    let sym = *g.rules.keys().next().unwrap();
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![sym],
    });
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![sym],
    });
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. })),
        "errors: {:?}",
        r.errors
    );
}

#[test]
fn test_35_duplicate_token_pattern_warning() {
    let mut g = minimal_valid_grammar("dup_pat");
    g.tokens.insert(
        SymbolId(200),
        Token {
            name: "NUM2".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. })),
        "warnings: {:?}",
        r.warnings
    );
}
