#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for ExternalToken lifecycle in adze-ir:
//! construction → grammar registration → validation → normalization,
//! serde roundtrip, Display/Debug, Clone/Eq, and rule references.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, Grammar, GrammarError, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ext(name: &str, id: u16) -> ExternalToken {
    ExternalToken {
        name: name.into(),
        symbol_id: SymbolId(id),
    }
}

fn make_rule(lhs: u16, rhs: Vec<Symbol>) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

fn base_grammar() -> Grammar {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(SymbolId(10), "expr".into());
    g.add_rule(make_rule(10, vec![Symbol::Terminal(SymbolId(1))]));
    g
}

// ===========================================================================
// 1. Construction
// ===========================================================================

#[test]
fn construction_basic() {
    let et = ext("indent", 42);
    assert_eq!(et.name, "indent");
    assert_eq!(et.symbol_id, SymbolId(42));
}

#[test]
fn construction_zero_id() {
    let et = ext("eof_marker", 0);
    assert_eq!(et.symbol_id, SymbolId(0));
}

#[test]
fn construction_max_id() {
    let et = ext("max_tok", u16::MAX);
    assert_eq!(et.symbol_id, SymbolId(u16::MAX));
}

#[test]
fn construction_empty_name() {
    let et = ext("", 5);
    assert_eq!(et.name, "");
}

// ===========================================================================
// 2. Clone / Eq behaviour
// ===========================================================================

#[test]
fn clone_produces_equal_fields() {
    let et = ext("dedent", 7);
    let cloned = et.clone();
    assert_eq!(cloned.name, et.name);
    assert_eq!(cloned.symbol_id, et.symbol_id);
}

#[test]
fn clone_is_independent() {
    let mut et = ext("newline", 3);
    let cloned = et.clone();
    et.name = "changed".into();
    assert_eq!(cloned.name, "newline");
}

#[test]
fn two_tokens_same_fields_are_structurally_equal() {
    let a = ext("indent", 10);
    let b = ext("indent", 10);
    // ExternalToken doesn't derive PartialEq, so compare field-by-field
    assert_eq!(a.name, b.name);
    assert_eq!(a.symbol_id, b.symbol_id);
}

#[test]
fn two_tokens_different_ids_differ() {
    let a = ext("tok", 1);
    let b = ext("tok", 2);
    assert_ne!(a.symbol_id, b.symbol_id);
}

// ===========================================================================
// 3. Debug formatting
// ===========================================================================

#[test]
fn debug_contains_struct_name() {
    let et = ext("newline", 1);
    let dbg = format!("{et:?}");
    assert!(dbg.contains("ExternalToken"), "missing struct name: {dbg}");
}

#[test]
fn debug_contains_field_values() {
    let et = ext("template_chars", 55);
    let dbg = format!("{et:?}");
    assert!(dbg.contains("template_chars"), "missing name: {dbg}");
    assert!(dbg.contains("55"), "missing symbol id: {dbg}");
}

// ===========================================================================
// 4. Serde roundtrip
// ===========================================================================

#[test]
fn serde_json_roundtrip_single() {
    let et = ext("string_content", 99);
    let json = serde_json::to_string(&et).unwrap();
    let back: ExternalToken = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, et.name);
    assert_eq!(back.symbol_id, et.symbol_id);
}

#[test]
fn serde_json_roundtrip_vec() {
    let tokens = vec![ext("indent", 1), ext("dedent", 2), ext("newline", 3)];
    let json = serde_json::to_string(&tokens).unwrap();
    let back: Vec<ExternalToken> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.len(), 3);
    for i in 0..tokens.len() {
        assert_eq!(back[i].name, tokens[i].name);
        assert_eq!(back[i].symbol_id, tokens[i].symbol_id);
    }
}

#[test]
fn serde_preserves_unicode_name() {
    let et = ext("日本語トークン", 88);
    let json = serde_json::to_string(&et).unwrap();
    let back: ExternalToken = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "日本語トークン");
}

#[test]
fn serde_roundtrip_grammar_with_externals() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    g.externals.push(ext("dedent", 51));

    let json = serde_json::to_string(&g).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.externals.len(), 2);
    assert_eq!(back.externals[0].name, "indent");
    assert_eq!(back.externals[1].symbol_id, SymbolId(51));
}

// ===========================================================================
// 5. Grammar registration
// ===========================================================================

#[test]
fn register_single_external() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "indent");
}

#[test]
fn register_multiple_externals() {
    let mut g = base_grammar();
    for (i, name) in ["indent", "dedent", "newline", "string_content"]
        .iter()
        .enumerate()
    {
        g.externals.push(ext(name, 50 + i as u16));
    }
    assert_eq!(g.externals.len(), 4);
}

#[test]
fn externals_preserved_after_add_rule() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    g.add_rule(make_rule(10, vec![Symbol::Terminal(SymbolId(1))]));
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn builder_registers_externals() {
    let g = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .external("INDENT")
        .external("DEDENT")
        .start("expr")
        .build();
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
}

// ===========================================================================
// 6. Validation with external tokens
// ===========================================================================

#[test]
fn validate_passes_with_registered_external_in_rule() {
    let mut g = base_grammar();
    let ext_id = SymbolId(50);
    g.externals.push(ext("indent", 50));
    g.add_rule(make_rule(10, vec![Symbol::External(ext_id)]));

    assert!(g.validate().is_ok(), "validation should pass");
}

#[test]
fn validate_fails_unresolved_external_ref() {
    let mut g = base_grammar();
    // Reference an external that is NOT registered
    g.add_rule(make_rule(10, vec![Symbol::External(SymbolId(999))]));

    match g.validate() {
        Err(GrammarError::UnresolvedExternalSymbol(SymbolId(999))) => {}
        other => panic!("expected UnresolvedExternalSymbol(999), got {other:?}"),
    }
}

#[test]
fn validate_passes_external_not_referenced_in_rules() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    // No rule references the external — that's fine
    assert!(g.validate().is_ok());
}

#[test]
fn validate_multiple_externals_all_referenced() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    g.externals.push(ext("dedent", 51));

    g.add_rule(make_rule(
        10,
        vec![
            Symbol::External(SymbolId(50)),
            Symbol::External(SymbolId(51)),
        ],
    ));
    assert!(g.validate().is_ok());
}

#[test]
fn validate_external_in_optional_wrapper() {
    let mut g = base_grammar();
    g.externals.push(ext("newline", 60));

    g.add_rule(make_rule(
        10,
        vec![Symbol::Optional(Box::new(Symbol::External(SymbolId(60))))],
    ));
    assert!(g.validate().is_ok());
}

#[test]
fn validate_external_in_repeat() {
    let mut g = base_grammar();
    g.externals.push(ext("comment", 70));

    g.add_rule(make_rule(
        10,
        vec![Symbol::Repeat(Box::new(Symbol::External(SymbolId(70))))],
    ));
    assert!(g.validate().is_ok());
}

#[test]
fn validate_external_in_choice() {
    let mut g = base_grammar();
    g.externals.push(ext("heredoc", 80));

    g.add_rule(make_rule(
        10,
        vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::External(SymbolId(80)),
        ])],
    ));
    assert!(g.validate().is_ok());
}

#[test]
fn validate_external_in_sequence() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    g.externals.push(ext("dedent", 51));

    g.add_rule(make_rule(
        10,
        vec![Symbol::Sequence(vec![
            Symbol::External(SymbolId(50)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::External(SymbolId(51)),
        ])],
    ));
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 7. Normalization with external tokens
// ===========================================================================

#[test]
fn normalize_preserves_external_symbols_in_simple_rule() {
    let mut g = Grammar::new("norm_test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.externals.push(ext("indent", 50));
    g.add_rule(make_rule(
        10,
        vec![
            Symbol::External(SymbolId(50)),
            Symbol::Terminal(SymbolId(1)),
        ],
    ));

    let rules = g.normalize();
    let has_external = rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::External(SymbolId(50))))
    });
    assert!(has_external, "external symbol should survive normalization");
}

#[test]
fn normalize_expands_optional_around_external() {
    let mut g = Grammar::new("norm_opt".into());
    g.externals.push(ext("indent", 50));
    g.add_rule(make_rule(
        10,
        vec![Symbol::Optional(Box::new(Symbol::External(SymbolId(50))))],
    ));

    let rules = g.normalize();
    // After normalization, Optional(External(50)) becomes aux rule: aux -> External(50) | ε
    assert!(
        rules.len() >= 3,
        "expected aux rules from Optional expansion"
    );
    let has_ext = rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::External(SymbolId(50))))
    });
    assert!(has_ext, "external should appear in expanded aux rule");
}

#[test]
fn normalize_expands_repeat_around_external() {
    let mut g = Grammar::new("norm_rep".into());
    g.externals.push(ext("comment", 70));
    g.add_rule(make_rule(
        10,
        vec![Symbol::Repeat(Box::new(Symbol::External(SymbolId(70))))],
    ));

    let rules = g.normalize();
    let has_ext = rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::External(SymbolId(70))))
    });
    assert!(
        has_ext,
        "external should appear in expanded Repeat aux rule"
    );
}

// ===========================================================================
// 8. Symbol registry integration
// ===========================================================================

#[test]
fn build_registry_includes_externals() {
    let mut g = base_grammar();
    g.externals.push(ext("indent", 50));
    g.externals.push(ext("dedent", 51));

    let registry = g.build_registry();
    let info = registry.get_id("indent");
    assert!(info.is_some(), "registry should contain external 'indent'");
    let info2 = registry.get_id("dedent");
    assert!(info2.is_some(), "registry should contain external 'dedent'");
}

#[test]
fn get_or_build_registry_includes_externals() {
    let mut g = base_grammar();
    g.externals.push(ext("newline", 60));

    let registry = g.get_or_build_registry();
    assert!(
        registry.get_id("newline").is_some(),
        "registry should contain external 'newline'"
    );
}

// ===========================================================================
// 9. External token in rule references (Symbol::External)
// ===========================================================================

#[test]
fn symbol_external_variant_equality() {
    let a = Symbol::External(SymbolId(10));
    let b = Symbol::External(SymbolId(10));
    let c = Symbol::External(SymbolId(11));
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn symbol_external_hashing() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Symbol::External(SymbolId(5)));
    assert!(set.contains(&Symbol::External(SymbolId(5))));
    assert!(!set.contains(&Symbol::External(SymbolId(6))));
}

#[test]
fn symbol_external_ordering() {
    let a = Symbol::External(SymbolId(1));
    let b = Symbol::External(SymbolId(2));
    assert!(a < b);
}

// ===========================================================================
// 10. Full lifecycle: create → register → validate → normalize
// ===========================================================================

#[test]
fn full_lifecycle_python_like() {
    // 1. Create external tokens
    let indent = ext("INDENT", 100);
    let dedent = ext("DEDENT", 101);

    // 2. Register in grammar
    let mut g = Grammar::new("python_lite".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "pass".into(),
            pattern: TokenPattern::String("pass".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(SymbolId(10), "block".into());
    g.externals.push(indent);
    g.externals.push(dedent);

    g.add_rule(make_rule(
        10,
        vec![
            Symbol::External(SymbolId(100)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::External(SymbolId(101)),
        ],
    ));

    // 3. Validate
    assert!(g.validate().is_ok(), "grammar should validate");

    // 4. Normalize
    let rules = g.normalize();
    assert!(!rules.is_empty(), "should have rules after normalization");

    // Externals list untouched
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn full_lifecycle_with_builder() {
    // Build grammar with externals via builder API
    let mut g = GrammarBuilder::new("lifecycle")
        .token("PASS", "pass")
        .token("COLON", ":")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("NEWLINE", "NEWLINE")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .rule("block", vec!["INDENT", "PASS", "DEDENT"])
        .start("block")
        .build();

    // Validate
    assert!(g.validate().is_ok(), "builder grammar should validate");

    // Normalize
    let rules = g.normalize();
    assert!(!rules.is_empty());

    // Serde roundtrip
    let json = serde_json::to_string(&g).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.externals.len(), 3);
    assert_eq!(back.name, "lifecycle");
}
