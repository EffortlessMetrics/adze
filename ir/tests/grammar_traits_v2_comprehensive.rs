//! Comprehensive tests for Grammar Clone/Debug/PartialEq, constructor patterns,
//! and Serialize/Deserialize roundtrip.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a minimal grammar with one token and one rule.
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("NUM", r"\d+")
        .rule("start", vec!["NUM"])
        .start("start")
        .build()
}

/// Build a medium-complexity arithmetic grammar.
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build a grammar that exercises many fields.
fn rich_grammar() -> Grammar {
    GrammarBuilder::new("rich")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("WS", r"\s+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .extra("WS")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .inline("expr")
        .supertype("expr")
        .precedence(1, Associativity::Left, vec!["expr"])
        .start("expr")
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Grammar Default (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_default_name_is_empty() {
    let grammar = Grammar::default();
    assert!(grammar.name.is_empty());
}

#[test]
fn test_default_rules_empty() {
    let grammar = Grammar::default();
    assert!(grammar.rules.is_empty());
}

#[test]
fn test_default_tokens_empty() {
    let grammar = Grammar::default();
    assert!(grammar.tokens.is_empty());
}

#[test]
fn test_default_precedences_empty() {
    let grammar = Grammar::default();
    assert!(grammar.precedences.is_empty());
}

#[test]
fn test_default_conflicts_extras_externals_empty() {
    let grammar = Grammar::default();
    assert!(grammar.conflicts.is_empty());
    assert!(grammar.extras.is_empty());
    assert!(grammar.externals.is_empty());
}

#[test]
fn test_default_fields_supertypes_inline_empty() {
    let grammar = Grammar::default();
    assert!(grammar.fields.is_empty());
    assert!(grammar.supertypes.is_empty());
    assert!(grammar.inline_rules.is_empty());
}

#[test]
fn test_default_alias_sequences_and_production_ids_empty() {
    let grammar = Grammar::default();
    assert!(grammar.alias_sequences.is_empty());
    assert!(grammar.production_ids.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

#[test]
fn test_default_rule_names_and_registry() {
    let grammar = Grammar::default();
    assert!(grammar.rule_names.is_empty());
    assert!(grammar.symbol_registry.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Grammar Clone equality (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_clone_default_grammar() {
    let original = Grammar::default();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_minimal_grammar() {
    let original = minimal_grammar();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_arithmetic_grammar() {
    let original = arithmetic_grammar();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_rich_grammar() {
    let original = rich_grammar();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_python_like_grammar() {
    let original = GrammarBuilder::python_like();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_javascript_like_grammar() {
    let original = GrammarBuilder::javascript_like();
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_clone_preserves_name() {
    let original = Grammar::new("hello".to_string());
    let cloned = original.clone();
    assert_eq!(cloned.name, "hello");
}

#[test]
fn test_clone_preserves_all_fields() {
    let original = rich_grammar();
    let cloned = original.clone();
    assert_eq!(original.name, cloned.name);
    assert_eq!(original.rules, cloned.rules);
    assert_eq!(original.tokens, cloned.tokens);
    assert_eq!(original.precedences, cloned.precedences);
    assert_eq!(original.conflicts, cloned.conflicts);
    assert_eq!(original.externals, cloned.externals);
    assert_eq!(original.extras, cloned.extras);
    assert_eq!(original.fields, cloned.fields);
    assert_eq!(original.supertypes, cloned.supertypes);
    assert_eq!(original.inline_rules, cloned.inline_rules);
    assert_eq!(original.alias_sequences, cloned.alias_sequences);
    assert_eq!(original.production_ids, cloned.production_ids);
    assert_eq!(
        original.max_alias_sequence_length,
        cloned.max_alias_sequence_length
    );
    assert_eq!(original.rule_names, cloned.rule_names);
    assert_eq!(original.symbol_registry, cloned.symbol_registry);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Grammar Debug output (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_debug_default_contains_grammar() {
    let grammar = Grammar::default();
    let dbg = format!("{grammar:?}");
    assert!(dbg.contains("Grammar"));
}

#[test]
fn test_debug_contains_name() {
    let grammar = Grammar::new("my_lang".to_string());
    let dbg = format!("{grammar:?}");
    assert!(dbg.contains("my_lang"));
}

#[test]
fn test_debug_minimal_contains_tokens() {
    let grammar = minimal_grammar();
    let dbg = format!("{grammar:?}");
    assert!(dbg.contains("tokens"));
}

#[test]
fn test_debug_minimal_contains_rules() {
    let grammar = minimal_grammar();
    let dbg = format!("{grammar:?}");
    assert!(dbg.contains("rules"));
}

#[test]
fn test_debug_rich_contains_externals() {
    let grammar = rich_grammar();
    let dbg = format!("{grammar:?}");
    assert!(dbg.contains("externals"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Grammar PartialEq (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_eq_two_defaults() {
    let a = Grammar::default();
    let b = Grammar::default();
    assert_eq!(a, b);
}

#[test]
fn test_eq_two_new_same_name() {
    let a = Grammar::new("lang".to_string());
    let b = Grammar::new("lang".to_string());
    assert_eq!(a, b);
}

#[test]
fn test_ne_different_names() {
    let a = Grammar::new("alpha".to_string());
    let b = Grammar::new("beta".to_string());
    assert_ne!(a, b);
}

#[test]
fn test_ne_default_vs_named() {
    let a = Grammar::default();
    let b = Grammar::new("named".to_string());
    assert_ne!(a, b);
}

#[test]
fn test_ne_minimal_vs_arithmetic() {
    let a = minimal_grammar();
    let b = arithmetic_grammar();
    assert_ne!(a, b);
}

#[test]
fn test_eq_reflexive() {
    let grammar = arithmetic_grammar();
    assert_eq!(grammar, grammar);
}

#[test]
fn test_eq_symmetric() {
    let a = minimal_grammar();
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn test_eq_transitive() {
    let a = minimal_grammar();
    let b = a.clone();
    let c = b.clone();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Serialize / Deserialize roundtrip (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_serde_roundtrip_default() {
    let original = Grammar::default();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_roundtrip_minimal() {
    let original = minimal_grammar();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_roundtrip_arithmetic() {
    let original = arithmetic_grammar();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_roundtrip_rich() {
    let original = rich_grammar();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_roundtrip_python_like() {
    let original = GrammarBuilder::python_like();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_roundtrip_javascript_like() {
    let original = GrammarBuilder::javascript_like();
    let json = serde_json::to_string(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_roundtrip_pretty_json() {
    let original = arithmetic_grammar();
    let json = serde_json::to_string_pretty(&original).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_serde_json_contains_name() {
    let grammar = Grammar::new("check_name".to_string());
    let json = serde_json::to_string(&grammar).unwrap();
    assert!(json.contains("check_name"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Grammar with all fields populated (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_rich_grammar_has_externals() {
    let grammar = rich_grammar();
    assert!(!grammar.externals.is_empty());
}

#[test]
fn test_rich_grammar_has_extras() {
    let grammar = rich_grammar();
    assert!(!grammar.extras.is_empty());
}

#[test]
fn test_rich_grammar_has_inline_rules() {
    let grammar = rich_grammar();
    assert!(!grammar.inline_rules.is_empty());
}

#[test]
fn test_rich_grammar_has_supertypes() {
    let grammar = rich_grammar();
    assert!(!grammar.supertypes.is_empty());
}

#[test]
fn test_rich_grammar_has_precedences() {
    let grammar = rich_grammar();
    assert!(!grammar.precedences.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Grammar modification after clone (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_clone_then_change_name() {
    let original = minimal_grammar();
    let mut cloned = original.clone();
    cloned.name = "modified".to_string();
    assert_ne!(original, cloned);
    assert_eq!(original.name, "minimal");
    assert_eq!(cloned.name, "modified");
}

#[test]
fn test_clone_then_add_rule() {
    let original = minimal_grammar();
    let mut cloned = original.clone();
    cloned.add_rule(Rule {
        lhs: SymbolId(99),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(99),
    });
    assert_ne!(original, cloned);
    assert!(cloned.rules.len() > original.rules.len());
}

#[test]
fn test_clone_then_clear_tokens() {
    let original = minimal_grammar();
    let mut cloned = original.clone();
    cloned.tokens.clear();
    assert_ne!(original, cloned);
    assert!(cloned.tokens.is_empty());
    assert!(!original.tokens.is_empty());
}

#[test]
fn test_clone_then_add_external() {
    let original = minimal_grammar();
    let mut cloned = original.clone();
    cloned.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(200),
    });
    assert_ne!(original, cloned);
    assert!(original.externals.is_empty());
}

#[test]
fn test_clone_then_set_max_alias_length() {
    let original = Grammar::default();
    let mut cloned = original.clone();
    cloned.max_alias_sequence_length = 42;
    assert_ne!(original, cloned);
    assert_eq!(original.max_alias_sequence_length, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_new_vs_default_differ_by_name() {
    let default_grammar = Grammar::default();
    let named = Grammar::new("something".to_string());
    assert_ne!(default_grammar, named);
}

#[test]
fn test_grammar_new_empty_string_equals_default() {
    let default_grammar = Grammar::default();
    let empty_named = Grammar::new(String::new());
    assert_eq!(default_grammar, empty_named);
}

#[test]
fn test_grammar_with_unicode_name() {
    let grammar = Grammar::new("日本語文法".to_string());
    let cloned = grammar.clone();
    assert_eq!(grammar, cloned);
    let json = serde_json::to_string(&grammar).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar, restored);
}

#[test]
fn test_grammar_with_empty_rule_vec() {
    let mut grammar = Grammar::default();
    grammar.rules.insert(SymbolId(1), Vec::new());
    let cloned = grammar.clone();
    assert_eq!(grammar, cloned);
}

#[test]
fn test_grammar_alias_sequences_populated() {
    let mut grammar = Grammar::default();
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![
                Some("alias_a".to_string()),
                None,
                Some("alias_b".to_string()),
            ],
        },
    );
    grammar.max_alias_sequence_length = 3;

    let cloned = grammar.clone();
    assert_eq!(grammar, cloned);

    let json = serde_json::to_string(&grammar).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar, restored);
}

#[test]
fn test_grammar_production_ids_populated() {
    let mut grammar = Grammar::default();
    grammar.production_ids.insert(RuleId(0), ProductionId(10));
    grammar.production_ids.insert(RuleId(1), ProductionId(20));

    let cloned = grammar.clone();
    assert_eq!(grammar, cloned);
}

#[test]
fn test_grammar_fields_populated() {
    let mut grammar = Grammar::default();
    // Fields must be in lexicographic order per grammar invariant
    grammar.fields.insert(FieldId(0), "alpha".to_string());
    grammar.fields.insert(FieldId(1), "beta".to_string());
    grammar.fields.insert(FieldId(2), "gamma".to_string());

    let json = serde_json::to_string(&grammar).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar, restored);
}

#[test]
fn test_grammar_conflict_declarations() {
    let mut grammar = Grammar::default();
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(5)),
    });

    let cloned = grammar.clone();
    assert_eq!(grammar, cloned);

    let json = serde_json::to_string(&grammar).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar, restored);
}
