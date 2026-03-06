#![allow(clippy::needless_range_loop)]

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId, Grammar,
    Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

/// Helper: build a minimal grammar with one rule and one token.
fn simple_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Helper: build a grammar with multiple rules and precedence.
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .start("expr")
        .build()
}

// ---------------------------------------------------------------------------
// 1. Clone produces equal grammar
// ---------------------------------------------------------------------------

#[test]
fn clone_produces_equal_empty_grammar() {
    let g = Grammar::default();
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn clone_produces_equal_named_grammar() {
    let g = Grammar::new("test_lang".to_string());
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn clone_produces_equal_simple_grammar() {
    let g = simple_grammar("simple");
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn clone_produces_equal_arithmetic_grammar() {
    let g = arithmetic_grammar();
    let g2 = g.clone();
    assert_eq!(g, g2);
}

// ---------------------------------------------------------------------------
// 2. Clone is independent (modifying clone doesn't affect original)
// ---------------------------------------------------------------------------

#[test]
fn clone_independence_name_change() {
    let g = simple_grammar("original");
    let mut g2 = g.clone();
    g2.name = "modified".to_string();
    assert_ne!(g.name, g2.name);
    assert_eq!(g.name, "original");
}

#[test]
fn clone_independence_add_rule() {
    let g = simple_grammar("lang");
    let original_rule_count = g.rules.len();
    let mut g2 = g.clone();
    g2.add_rule(Rule {
        lhs: SymbolId(999),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(99),
    });
    assert_eq!(g.rules.len(), original_rule_count);
    assert!(g2.rules.len() > original_rule_count);
}

#[test]
fn clone_independence_add_token() {
    let g = simple_grammar("lang");
    let original_token_count = g.tokens.len();
    let mut g2 = g.clone();
    g2.tokens.insert(
        SymbolId(500),
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    assert_eq!(g.tokens.len(), original_token_count);
    assert_eq!(g2.tokens.len(), original_token_count + 1);
}

#[test]
fn clone_independence_modify_extras() {
    let g = Grammar::new("lang".to_string());
    let mut g2 = g.clone();
    g2.extras.push(SymbolId(42));
    assert!(g.extras.is_empty());
    assert_eq!(g2.extras.len(), 1);
}

#[test]
fn clone_independence_modify_precedences() {
    let g = Grammar::new("lang".to_string());
    let mut g2 = g.clone();
    g2.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    assert!(g.precedences.is_empty());
    assert_eq!(g2.precedences.len(), 1);
}

#[test]
fn clone_independence_modify_fields() {
    let g = Grammar::new("lang".to_string());
    let mut g2 = g.clone();
    g2.fields.insert(FieldId(0), "name".to_string());
    assert!(g.fields.is_empty());
    assert_eq!(g2.fields.len(), 1);
}

#[test]
fn clone_independence_modify_externals() {
    let g = Grammar::new("lang".to_string());
    let mut g2 = g.clone();
    g2.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(10),
    });
    assert!(g.externals.is_empty());
    assert_eq!(g2.externals.len(), 1);
}

#[test]
fn clone_independence_modify_conflicts() {
    let g = Grammar::new("lang".to_string());
    let mut g2 = g.clone();
    g2.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    assert!(g.conflicts.is_empty());
    assert_eq!(g2.conflicts.len(), 1);
}

// ---------------------------------------------------------------------------
// 3. PartialEq reflexive (g == g)
// ---------------------------------------------------------------------------

#[test]
fn partial_eq_reflexive_default() {
    let g = Grammar::default();
    assert_eq!(g, g);
}

#[test]
fn partial_eq_reflexive_named() {
    let g = Grammar::new("reflexive_test".to_string());
    assert_eq!(g, g);
}

#[test]
fn partial_eq_reflexive_with_rules() {
    let g = arithmetic_grammar();
    assert_eq!(g, g);
}

// ---------------------------------------------------------------------------
// 4. PartialEq symmetric (g1 == g2 implies g2 == g1)
// ---------------------------------------------------------------------------

#[test]
fn partial_eq_symmetric_default() {
    let g1 = Grammar::default();
    let g2 = Grammar::default();
    assert_eq!(g1, g2);
    assert_eq!(g2, g1);
}

#[test]
fn partial_eq_symmetric_named() {
    let g1 = Grammar::new("sym".to_string());
    let g2 = Grammar::new("sym".to_string());
    assert_eq!(g1, g2);
    assert_eq!(g2, g1);
}

#[test]
fn partial_eq_symmetric_cloned() {
    let g1 = arithmetic_grammar();
    let g2 = g1.clone();
    assert_eq!(g1, g2);
    assert_eq!(g2, g1);
}

// ---------------------------------------------------------------------------
// 5. Grammar with rules equality
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_same_rules_are_equal() {
    let g1 = simple_grammar("lang");
    let g2 = simple_grammar("lang");
    assert_eq!(g1, g2);
}

#[test]
fn grammar_with_same_arithmetic_rules_are_equal() {
    let g1 = arithmetic_grammar();
    let g2 = arithmetic_grammar();
    assert_eq!(g1, g2);
}

#[test]
fn grammar_equality_sensitive_to_rule_rhs() {
    let mut g1 = Grammar::new("lang".to_string());
    let mut g2 = Grammar::new("lang".to_string());

    g1.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g2.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    assert_ne!(g1, g2);
}

#[test]
fn grammar_equality_sensitive_to_precedence_in_rule() {
    let mut g1 = Grammar::new("lang".to_string());
    let mut g2 = Grammar::new("lang".to_string());

    let base_rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    g1.add_rule(base_rule.clone());

    let mut rule_with_prec = base_rule;
    rule_with_prec.precedence = Some(PrecedenceKind::Static(5));
    g2.add_rule(rule_with_prec);

    assert_ne!(g1, g2);
}

// ---------------------------------------------------------------------------
// 6. Grammar with different names not equal
// ---------------------------------------------------------------------------

#[test]
fn grammars_with_different_names_not_equal() {
    let g1 = Grammar::new("alpha".to_string());
    let g2 = Grammar::new("beta".to_string());
    assert_ne!(g1, g2);
}

#[test]
fn grammars_same_rules_different_names_not_equal() {
    let g1 = simple_grammar("lang_a");
    let g2 = simple_grammar("lang_b");
    assert_ne!(g1, g2);
}

#[test]
fn grammar_empty_name_vs_nonempty_name() {
    let g1 = Grammar::new(String::new());
    let g2 = Grammar::new("nonempty".to_string());
    assert_ne!(g1, g2);
}

// ---------------------------------------------------------------------------
// 7. Grammar Debug output is deterministic
// ---------------------------------------------------------------------------

#[test]
fn debug_output_deterministic_default() {
    let g = Grammar::default();
    let d1 = format!("{:?}", g);
    let d2 = format!("{:?}", g);
    assert_eq!(d1, d2);
}

#[test]
fn debug_output_deterministic_with_rules() {
    let g = arithmetic_grammar();
    let d1 = format!("{:?}", g);
    let d2 = format!("{:?}", g);
    assert_eq!(d1, d2);
}

#[test]
fn debug_output_deterministic_across_clones() {
    let g1 = arithmetic_grammar();
    let g2 = g1.clone();
    assert_eq!(format!("{:?}", g1), format!("{:?}", g2));
}

#[test]
fn debug_output_contains_grammar_name() {
    let g = Grammar::new("my_language".to_string());
    let dbg = format!("{:?}", g);
    assert!(
        dbg.contains("my_language"),
        "Debug output should contain the grammar name"
    );
}

// ---------------------------------------------------------------------------
// 8. Grammar Default is valid
// ---------------------------------------------------------------------------

#[test]
fn default_grammar_has_empty_name() {
    let g = Grammar::default();
    assert_eq!(g.name, "");
}

#[test]
fn default_grammar_has_no_rules() {
    let g = Grammar::default();
    assert!(g.rules.is_empty());
}

#[test]
fn default_grammar_all_collections_empty() {
    let g = Grammar::default();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
    assert!(g.precedences.is_empty());
    assert!(g.conflicts.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.fields.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.inline_rules.is_empty());
    assert!(g.alias_sequences.is_empty());
    assert!(g.production_ids.is_empty());
    assert_eq!(g.max_alias_sequence_length, 0);
    assert!(g.rule_names.is_empty());
    assert!(g.symbol_registry.is_none());
}

#[test]
fn default_grammar_equals_new_empty() {
    let g1 = Grammar::default();
    let g2 = Grammar::new(String::new());
    assert_eq!(g1, g2);
}

#[test]
fn default_grammar_validates_ok() {
    let g = Grammar::default();
    assert!(g.validate().is_ok());
}
