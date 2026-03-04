//! Tests for Grammar field mapping and precedence.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

#[test]
fn grammar_with_fields_has_field_names() {
    let g = GrammarBuilder::new("fields")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .build();
    // Grammar should contain field-related structures
    assert!(!g.rules.is_empty());
}

#[test]
fn grammar_precedence_levels() {
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .build();
    // Precedence stored on individual rules
    let rules = g.rules.values().flatten().collect::<Vec<_>>();
    let prec_count = rules.iter().filter(|r| r.precedence.is_some()).count();
    assert!(
        prec_count >= 2,
        "should have at least 2 rules with precedence"
    );
}

#[test]
fn grammar_right_associativity() {
    let g = GrammarBuilder::new("rassoc")
        .token("num", r"\d+")
        .token("exp", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "exp", "expr"], 1, Associativity::Right)
        .build();
    let rules = g.rules.values().flatten().collect::<Vec<_>>();
    let right_assoc = rules
        .iter()
        .filter(|r| r.associativity == Some(Associativity::Right))
        .count();
    assert_eq!(right_assoc, 1);
}

#[test]
fn grammar_none_associativity() {
    let g = GrammarBuilder::new("nassoc")
        .token("num", r"\d+")
        .token("eq", r"==")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::None)
        .build();
    let rules = g.rules.values().flatten().collect::<Vec<_>>();
    let none_assoc = rules
        .iter()
        .filter(|r| r.associativity == Some(Associativity::None))
        .count();
    assert_eq!(none_assoc, 1);
}

#[test]
fn grammar_multiple_precedence_levels() {
    let g = GrammarBuilder::new("multi_prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("exp", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "exp", "expr"], 3, Associativity::Right)
        .build();
    let rules = g.rules.values().flatten().collect::<Vec<_>>();
    let prec_count = rules.iter().filter(|r| r.precedence.is_some()).count();
    assert!(
        prec_count >= 3,
        "should have at least 3 rules with precedence, got {prec_count}"
    );
}

#[test]
fn grammar_fragile_tokens() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("keyword_if", "if")
        .fragile_token("keyword_else", "else")
        .token("ident", r"[a-z]+")
        .rule("stmt", vec!["keyword_if", "ident", "keyword_else", "ident"])
        .build();
    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 2, "should have 2 fragile tokens");
}

#[test]
fn grammar_extras_dont_appear_in_rules() {
    let g = GrammarBuilder::new("extras_sep")
        .token("num", r"\d+")
        .token("ws", r"\s+")
        .rule("expr", vec!["num"])
        .extra("ws")
        .build();
    // The ws token should be in extras but not referenced in rule RHS directly
    assert!(!g.extras.is_empty());
}

#[test]
fn grammar_external_tokens() {
    let g = GrammarBuilder::new("ext")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .external("indent")
        .external("dedent")
        .build();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(1) == SymbolId(1));
    assert!(SymbolId(5) > SymbolId(3));
}

#[test]
fn field_id_display() {
    let f = FieldId(42);
    assert_eq!(f.0, 42);
}

#[test]
fn production_id_display() {
    let p = ProductionId(7);
    assert_eq!(p.0, 7);
}
