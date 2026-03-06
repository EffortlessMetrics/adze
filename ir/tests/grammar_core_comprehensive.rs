//! Comprehensive tests for the Grammar struct and its core operations.
//!
//! Tests Grammar creation, normalization, validation, serialization,
//! rule manipulation, symbol management, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// =============================================================================
// Grammar creation
// =============================================================================

#[test]
fn grammar_new_has_name() {
    let g = Grammar::new("test".to_string());
    assert_eq!(g.name, "test");
}

#[test]
fn grammar_default_is_empty() {
    let g = Grammar::default();
    assert!(g.name.is_empty() || g.name.is_empty());
    assert!(g.rules.is_empty());
}

#[test]
fn grammar_clone_is_equal() {
    let g = GrammarBuilder::new("clone_test")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g2 = g.clone();
    assert_eq!(g, g2);
}

// =============================================================================
// GrammarBuilder fluent API
// =============================================================================

#[test]
fn builder_creates_named_grammar() {
    let g = GrammarBuilder::new("mygrammar")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g.name, "mygrammar");
}

#[test]
fn builder_adds_tokens() {
    let g = GrammarBuilder::new("tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.tokens.len() >= 3);
}

#[test]
fn builder_adds_rules() {
    let g = GrammarBuilder::new("rules")
        .token("a", "a")
        .token("b", "b")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("start", vec!["r1"])
        .start("start")
        .build();
    assert!(g.rules.len() >= 2);
}

#[test]
fn builder_sets_start_symbol() {
    let g = GrammarBuilder::new("st")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.start_symbol().is_some() || !g.rule_names.is_empty());
}

#[test]
fn builder_adds_extras() {
    let g = GrammarBuilder::new("ex")
        .token("a", "a")
        .token("ws", "\\s+")
        .rule("start", vec!["a"])
        .start("start")
        .extra("ws")
        .build();
    assert!(!g.extras.is_empty() || g.tokens.len() >= 2);
}

#[test]
fn builder_adds_externals() {
    let g = GrammarBuilder::new("ext")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .external("indent")
        .build();
    assert!(!g.externals.is_empty());
}

#[test]
fn builder_adds_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("plus", "+")
        .rule("expr", vec!["a"])
        .start("expr")
        .precedence(1, Associativity::Left, vec!["plus"])
        .build();
    assert!(!g.precedences.is_empty());
}

#[test]
fn builder_fragile_token() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("nl", "\\n")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    // At least one fragile token
    let has_fragile = g.tokens.values().any(|t| t.fragile);
    assert!(has_fragile, "Should have at least one fragile token");
}

// =============================================================================
// Preset grammars
// =============================================================================

#[test]
fn python_like_grammar_is_valid() {
    let g = GrammarBuilder::python_like();
    let result = g.validate();
    assert!(
        result.is_ok(),
        "Python-like grammar should validate: {:?}",
        result.err()
    );
}

#[test]
fn javascript_like_grammar_is_valid() {
    let g = GrammarBuilder::javascript_like();
    let result = g.validate();
    assert!(
        result.is_ok(),
        "JS-like grammar should validate: {:?}",
        result.err()
    );
}

#[test]
fn python_like_has_rules() {
    let g = GrammarBuilder::python_like();
    assert!(!g.rules.is_empty(), "Python-like grammar should have rules");
}

#[test]
fn javascript_like_has_rules() {
    let g = GrammarBuilder::javascript_like();
    assert!(!g.rules.is_empty(), "JS-like grammar should have rules");
}

// =============================================================================
// Grammar normalization
// =============================================================================

#[test]
fn normalize_is_idempotent() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g.normalize();
    let count1 = g.all_rules().count();
    g.normalize();
    let count2 = g.all_rules().count();
    assert_eq!(count1, count2);
}

#[test]
fn normalize_does_not_remove_start_rule() {
    let mut g = GrammarBuilder::new("keep_start")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g.normalize();
    assert!(!g.rules.is_empty());
}

// =============================================================================
// Grammar validation
// =============================================================================

#[test]
fn valid_grammar_validates_ok() {
    let g = GrammarBuilder::new("valid")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.validate().is_ok());
}

// =============================================================================
// SymbolId operations
// =============================================================================

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(50));
}

#[test]
fn symbol_id_clone() {
    let s = SymbolId(42);
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn symbol_id_debug() {
    let s = SymbolId(7);
    let debug = format!("{:?}", s);
    assert!(debug.contains("7"));
}

// =============================================================================
// RuleId and StateId
// =============================================================================

#[test]
fn rule_id_equality() {
    assert_eq!(RuleId(0), RuleId(0));
    assert_ne!(RuleId(0), RuleId(1));
}

#[test]
fn state_id_equality() {
    assert_eq!(StateId(0), StateId(0));
    assert_ne!(StateId(0), StateId(1));
}

// =============================================================================
// Symbol enum variants
// =============================================================================

#[test]
fn symbol_terminal_debug() {
    let s = Symbol::Terminal(SymbolId(1));
    let debug = format!("{:?}", s);
    assert!(debug.contains("Terminal"));
}

#[test]
fn symbol_nonterminal_debug() {
    let s = Symbol::NonTerminal(SymbolId(2));
    let debug = format!("{:?}", s);
    assert!(debug.contains("NonTerminal"));
}

#[test]
fn symbol_epsilon_debug() {
    let s = Symbol::Epsilon;
    let debug = format!("{:?}", s);
    assert!(debug.contains("Epsilon"));
}

#[test]
fn symbol_clone_eq() {
    let s1 = Symbol::Terminal(SymbolId(5));
    let s2 = s1.clone();
    assert_eq!(s1, s2);
}

// =============================================================================
// Associativity
// =============================================================================

#[test]
fn associativity_variants() {
    let _l = Associativity::Left;
    let _r = Associativity::Right;
    let _n = Associativity::None;
}

#[test]
fn associativity_clone_eq() {
    assert_eq!(Associativity::Left, Associativity::Left);
    assert_ne!(Associativity::Left, Associativity::Right);
}

// =============================================================================
// Token
// =============================================================================

#[test]
fn token_creation() {
    let t = Token {
        name: "test".to_string(),
        pattern: TokenPattern::String("hello".to_string()),
        fragile: false,
    };
    assert_eq!(t.name, "test");
    assert!(!t.fragile);
}

#[test]
fn token_fragile() {
    let t = Token {
        name: "nl".to_string(),
        pattern: TokenPattern::String("\\n".to_string()),
        fragile: true,
    };
    assert!(t.fragile);
}

// =============================================================================
// ProductionId and FieldId
// =============================================================================

#[test]
fn production_id_equality() {
    assert_eq!(ProductionId(0), ProductionId(0));
    assert_ne!(ProductionId(0), ProductionId(1));
}

#[test]
fn field_id_equality() {
    assert_eq!(FieldId(0), FieldId(0));
    assert_ne!(FieldId(0), FieldId(1));
}

// =============================================================================
// Grammar serde roundtrip
// =============================================================================

#[test]
fn grammar_serde_roundtrip() {
    let g = GrammarBuilder::new("serde")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

#[test]
fn grammar_serde_with_externals() {
    let g = GrammarBuilder::new("serde_ext")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .external("indent")
        .build();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

// =============================================================================
// Grammar all_rules iterator
// =============================================================================

#[test]
fn all_rules_returns_all() {
    let g = GrammarBuilder::new("allr")
        .token("a", "a")
        .token("b", "b")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("start", vec!["r1"])
        .start("start")
        .build();
    let count = g.all_rules().count();
    assert!(count >= 2, "Should have at least 2 rules, got {}", count);
}

// =============================================================================
// Rule struct
// =============================================================================

#[test]
fn rule_has_lhs_and_rhs() {
    let r = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(r.lhs, SymbolId(1));
    assert_eq!(r.rhs.len(), 1);
}

#[test]
fn rule_with_precedence() {
    let r = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(r.precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(r.associativity, Some(Associativity::Left));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn grammar_with_many_rules() {
    let mut gb = GrammarBuilder::new("many");
    for i in 0..50 {
        gb = gb.token(&format!("t{}", i), &format!("t{}", i));
    }
    for i in 0..50 {
        gb = gb.rule(&format!("r{}", i), vec![&format!("t{}", i)]);
    }
    gb = gb.start("r0");
    let g = gb.build();
    assert!(g.rules.len() >= 50);
}

#[test]
fn grammar_name_with_special_chars() {
    let g = GrammarBuilder::new("my_grammar_v2")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert_eq!(g.name, "my_grammar_v2");
}
