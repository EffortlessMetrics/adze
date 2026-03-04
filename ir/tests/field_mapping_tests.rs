//! Comprehensive tests for IR field mapping and production operations.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, FieldId, Grammar, GrammarError, PrecedenceKind, ProductionId,
    Rule, RuleId, Symbol, SymbolId,
};

// ---------------------------------------------------------------------------
// Helper: build an expression grammar with fields on rules
// ---------------------------------------------------------------------------
fn expr_grammar_with_fields() -> Grammar {
    let mut grammar = GrammarBuilder::new("expr")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    // Register fields in lexicographic order
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "operator".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());

    // Attach field mappings to the binary-operator rules
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    if let Some(rules) = grammar.rules.get_mut(&expr_id) {
        for rule in rules.iter_mut() {
            if rule.rhs.len() == 3 {
                // left=0, operator=1, right=2
                rule.fields = vec![
                    (FieldId(0), 0), // left  -> position 0
                    (FieldId(1), 1), // operator -> position 1
                    (FieldId(2), 2), // right -> position 2
                ];
            }
        }
    }

    grammar
}

// ===========================================================================
// FieldId basic properties
// ===========================================================================

#[test]
fn field_id_display_format() {
    assert_eq!(format!("{}", FieldId(0)), "Field(0)");
    assert_eq!(format!("{}", FieldId(42)), "Field(42)");
    assert_eq!(
        format!("{}", FieldId(u16::MAX)),
        format!("Field({})", u16::MAX)
    );
}

#[test]
fn field_id_equality_and_hash() {
    use std::collections::HashSet;

    let a = FieldId(1);
    let b = FieldId(1);
    let c = FieldId(2);

    assert_eq!(a, b);
    assert_ne!(a, c);

    let mut set = HashSet::new();
    set.insert(a);
    set.insert(b);
    set.insert(c);
    assert_eq!(set.len(), 2);
}

#[test]
fn field_id_clone_and_copy() {
    let original = FieldId(7);
    let cloned = original;
    assert_eq!(original, cloned);
}

// ===========================================================================
// ProductionId basic properties
// ===========================================================================

#[test]
fn production_id_display_format() {
    assert_eq!(format!("{}", ProductionId(0)), "Production(0)");
    assert_eq!(format!("{}", ProductionId(99)), "Production(99)");
}

#[test]
fn production_id_ordering() {
    let a = ProductionId(1);
    let b = ProductionId(2);
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, ProductionId(1));
}

#[test]
fn production_id_clone_and_copy() {
    let original = ProductionId(5);
    let cloned = original;
    assert_eq!(original, cloned);
}

// ===========================================================================
// Grammar.fields (IndexMap<FieldId, String>)
// ===========================================================================

#[test]
fn grammar_fields_empty_by_default() {
    let grammar = Grammar::new("empty".to_string());
    assert_eq!(grammar.fields.len(), 0);
}

#[test]
fn grammar_fields_insert_and_lookup() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.fields.insert(FieldId(0), "alpha".to_string());
    grammar.fields.insert(FieldId(1), "beta".to_string());

    assert_eq!(grammar.fields.get(&FieldId(0)).unwrap(), "alpha");
    assert_eq!(grammar.fields.get(&FieldId(1)).unwrap(), "beta");
    assert!(grammar.fields.get(&FieldId(99)).is_none());
}

#[test]
fn grammar_fields_lexicographic_order_validates() {
    let mut grammar = Grammar::new("ordered".to_string());
    grammar.fields.insert(FieldId(0), "aaa".to_string());
    grammar.fields.insert(FieldId(1), "bbb".to_string());
    grammar.fields.insert(FieldId(2), "ccc".to_string());

    assert!(grammar.validate().is_ok());
}

#[test]
fn grammar_fields_non_lexicographic_order_fails_validation() {
    let mut grammar = Grammar::new("unordered".to_string());
    grammar.fields.insert(FieldId(0), "zebra".to_string());
    grammar.fields.insert(FieldId(1), "alpha".to_string());

    let err = grammar.validate().unwrap_err();
    assert!(matches!(err, GrammarError::InvalidFieldOrdering));
}

#[test]
fn grammar_fields_single_entry_validates() {
    let mut grammar = Grammar::new("single".to_string());
    grammar.fields.insert(FieldId(0), "only_field".to_string());
    assert!(grammar.validate().is_ok());
}

// ===========================================================================
// Rule.fields (Vec<(FieldId, usize)>) – mapping fields to RHS positions
// ===========================================================================

#[test]
fn rule_fields_default_empty() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert!(rule.fields.is_empty());
}

#[test]
fn rule_fields_single_mapping() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.fields.len(), 1);
    assert_eq!(rule.fields[0], (FieldId(0), 0));
}

#[test]
fn rule_fields_multiple_positions() {
    let grammar = expr_grammar_with_fields();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();

    // Binary rules should have 3 field mappings
    for rule in rules.iter().filter(|r| r.rhs.len() == 3) {
        assert_eq!(rule.fields.len(), 3);
        let field_ids: Vec<FieldId> = rule.fields.iter().map(|(fid, _)| *fid).collect();
        assert_eq!(field_ids, vec![FieldId(0), FieldId(1), FieldId(2)]);

        let positions: Vec<usize> = rule.fields.iter().map(|(_, pos)| *pos).collect();
        assert_eq!(positions, vec![0, 1, 2]);
    }

    // Terminal-only rule should have no field mappings
    for rule in rules.iter().filter(|r| r.rhs.len() == 1) {
        assert!(rule.fields.is_empty());
    }
}

// ===========================================================================
// ProductionId assignment via builder
// ===========================================================================

#[test]
fn builder_assigns_unique_production_ids() {
    let grammar = GrammarBuilder::new("test")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();

    let s_id = grammar.find_symbol_by_name("s").unwrap();
    let rules = grammar.get_rules_for_symbol(s_id).unwrap();

    let prod_ids: Vec<u16> = rules.iter().map(|r| r.production_id.0).collect();
    // All production IDs must be distinct
    let mut deduped = prod_ids.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(prod_ids.len(), deduped.len());
}

#[test]
fn builder_production_ids_start_at_zero() {
    let grammar = GrammarBuilder::new("test")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();

    let root_id = grammar.find_symbol_by_name("root").unwrap();
    let rules = grammar.get_rules_for_symbol(root_id).unwrap();
    assert_eq!(rules[0].production_id, ProductionId(0));
}

#[test]
fn builder_production_ids_increment_across_symbols() {
    let grammar = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"]) // production 0
        .rule("y", vec!["B"]) // production 1
        .rule("y", vec!["A", "B"]) // production 2
        .start("x")
        .build();

    let mut all_prod_ids: Vec<u16> = grammar.all_rules().map(|r| r.production_id.0).collect();
    all_prod_ids.sort();
    assert_eq!(all_prod_ids, vec![0, 1, 2]);
}

// ===========================================================================
// Grammar.alias_sequences
// ===========================================================================

#[test]
fn alias_sequences_empty_by_default() {
    let grammar = Grammar::new("test".to_string());
    assert!(grammar.alias_sequences.is_empty());
}

#[test]
fn alias_sequences_insert_and_retrieve() {
    let mut grammar = Grammar::new("test".to_string());
    let seq = AliasSequence {
        aliases: vec![
            Some("identifier".to_string()),
            None,
            Some("value".to_string()),
        ],
    };
    grammar.alias_sequences.insert(ProductionId(0), seq);

    let retrieved = grammar.alias_sequences.get(&ProductionId(0)).unwrap();
    assert_eq!(retrieved.aliases.len(), 3);
    assert_eq!(retrieved.aliases[0].as_deref(), Some("identifier"));
    assert!(retrieved.aliases[1].is_none());
    assert_eq!(retrieved.aliases[2].as_deref(), Some("value"));
}

// ===========================================================================
// Grammar.production_ids (RuleId -> ProductionId mapping)
// ===========================================================================

#[test]
fn production_ids_map_empty_by_default() {
    let grammar = Grammar::new("test".to_string());
    assert!(grammar.production_ids.is_empty());
}

#[test]
fn production_ids_map_insert_and_lookup() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.production_ids.insert(RuleId(0), ProductionId(10));
    grammar.production_ids.insert(RuleId(1), ProductionId(20));

    assert_eq!(
        grammar.production_ids.get(&RuleId(0)),
        Some(&ProductionId(10))
    );
    assert_eq!(
        grammar.production_ids.get(&RuleId(1)),
        Some(&ProductionId(20))
    );
    assert!(grammar.production_ids.get(&RuleId(99)).is_none());
}

// ===========================================================================
// Rule equality with field mappings
// ===========================================================================

#[test]
fn rules_with_same_fields_are_equal() {
    let r1 = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    };
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

#[test]
fn rules_with_different_fields_are_not_equal() {
    let base = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };

    let mut different = base.clone();
    different.fields = vec![(FieldId(1), 0)]; // different field id
    assert_ne!(base, different);
}

#[test]
fn rules_with_different_production_ids_are_not_equal() {
    let r1 = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let mut r2 = r1.clone();
    r2.production_id = ProductionId(1);
    assert_ne!(r1, r2);
}

// ===========================================================================
// Integration: fields + precedence + production on the same rule
// ===========================================================================

#[test]
fn rule_with_fields_and_precedence() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![
            (FieldId(0), 0), // left
            (FieldId(1), 1), // op
            (FieldId(2), 2), // right
        ],
        production_id: ProductionId(42),
    };

    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(rule.associativity, Some(Associativity::Left));
    assert_eq!(rule.fields.len(), 3);
    assert_eq!(rule.production_id, ProductionId(42));
}

// ===========================================================================
// End-to-end: build grammar, attach fields, validate
// ===========================================================================

#[test]
fn end_to_end_grammar_with_fields_validates() {
    let grammar = expr_grammar_with_fields();
    assert!(grammar.validate().is_ok());
    assert_eq!(grammar.fields.len(), 3);

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 3); // two binary, one terminal
}

#[test]
fn field_names_match_registered_values() {
    let grammar = expr_grammar_with_fields();
    let names: Vec<&str> = grammar.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["left", "operator", "right"]);
}

// ===========================================================================
// Serialization round-trip for fields and production ids
// ===========================================================================

#[test]
fn field_id_serde_roundtrip() {
    let original = FieldId(123);
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn production_id_serde_roundtrip() {
    let original = ProductionId(456);
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ProductionId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn rule_with_fields_serde_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
        ],
        precedence: Some(PrecedenceKind::Dynamic(3)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(7),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, deserialized);
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn field_mapping_to_epsilon_rule_is_empty() {
    let grammar = GrammarBuilder::new("nullable")
        .rule("empty", vec![])
        .start("empty")
        .build();

    let empty_id = grammar.find_symbol_by_name("empty").unwrap();
    let rules = grammar.get_rules_for_symbol(empty_id).unwrap();
    // Epsilon rules should carry no field mappings
    for rule in rules {
        assert!(rule.fields.is_empty());
    }
}

#[test]
fn duplicate_field_ids_in_grammar_fields_overwrite() {
    let mut grammar = Grammar::new("dup".to_string());
    grammar.fields.insert(FieldId(0), "first".to_string());
    grammar.fields.insert(FieldId(0), "replaced".to_string());

    assert_eq!(grammar.fields.len(), 1);
    assert_eq!(grammar.fields.get(&FieldId(0)).unwrap(), "replaced");
}
