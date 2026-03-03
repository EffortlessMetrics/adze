//! Comprehensive tests for the precedence and associativity system in the IR crate.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// 1. Rule creation with various precedence levels
// ---------------------------------------------------------------------------

#[test]
fn rule_with_no_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert!(rule.precedence.is_none());
    assert!(rule.associativity.is_none());
}

#[test]
fn rule_with_static_precedence_zero() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(0)),
        associativity: Some(Associativity::Left),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(0)));
}

#[test]
fn rule_with_positive_static_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Right),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(5)));
}

#[test]
fn rule_with_negative_static_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(-3)),
        associativity: Some(Associativity::None),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(-3)));
}

#[test]
fn rule_with_dynamic_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Dynamic(10)),
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(10)));
}

#[test]
fn rule_with_negative_dynamic_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Dynamic(-1)),
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(-1)));
}

#[test]
fn rule_with_i16_max_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(i16::MAX)),
        associativity: Some(Associativity::Left),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(i16::MAX)));
}

#[test]
fn rule_with_i16_min_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(i16::MIN)),
        associativity: Some(Associativity::Right),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(i16::MIN)));
}

// ---------------------------------------------------------------------------
// 2. Associativity comparisons and ordering
// ---------------------------------------------------------------------------

#[test]
fn associativity_variants_are_distinct() {
    assert_ne!(Associativity::Left, Associativity::Right);
    assert_ne!(Associativity::Left, Associativity::None);
    assert_ne!(Associativity::Right, Associativity::None);
}

#[test]
fn associativity_equality() {
    assert_eq!(Associativity::Left, Associativity::Left);
    assert_eq!(Associativity::Right, Associativity::Right);
    assert_eq!(Associativity::None, Associativity::None);
}

#[test]
fn associativity_clone() {
    let a = Associativity::Left;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn associativity_debug_format() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

#[test]
fn precedence_kind_equality() {
    assert_eq!(PrecedenceKind::Static(1), PrecedenceKind::Static(1));
    assert_ne!(PrecedenceKind::Static(1), PrecedenceKind::Static(2));
    assert_ne!(PrecedenceKind::Static(1), PrecedenceKind::Dynamic(1));
    assert_eq!(PrecedenceKind::Dynamic(3), PrecedenceKind::Dynamic(3));
}

// ---------------------------------------------------------------------------
// 3. Rules with and without field maps
// ---------------------------------------------------------------------------

#[test]
fn rule_with_empty_fields() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert!(rule.fields.is_empty());
}

#[test]
fn rule_with_single_field() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.fields.len(), 1);
    assert_eq!(rule.fields[0], (FieldId(0), 0));
}

#[test]
fn rule_with_multiple_fields_and_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(4)),
        ],
        precedence: Some(PrecedenceKind::Static(3)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(1),
    };
    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(3)));
}

// ---------------------------------------------------------------------------
// 4. Precedence interaction with grammar normalization
// ---------------------------------------------------------------------------

#[test]
fn normalization_preserves_rule_precedence() {
    let mut grammar = GrammarBuilder::new("norm_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    grammar.normalize();

    // After normalization, the precedence-annotated rule should still exist
    let prec_rules: Vec<_> = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert!(
        !prec_rules.is_empty(),
        "precedence rules must survive normalization"
    );
    assert_eq!(prec_rules[0].precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(prec_rules[0].associativity, Some(Associativity::Left));
}

#[test]
fn normalization_of_optional_does_not_add_precedence() {
    let mut grammar = Grammar::new("opt_prec".to_string());

    let lhs = SymbolId(1);
    let inner = SymbolId(2);
    grammar.tokens.insert(
        inner,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(inner)))],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Right),
        fields: Vec::new(),
        production_id: ProductionId(0),
    });

    grammar.normalize();

    // The original rule's precedence should be preserved on the rewritten rule
    let original_rules: Vec<_> = grammar
        .get_rules_for_symbol(lhs)
        .unwrap()
        .iter()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert!(!original_rules.is_empty());

    // The generated auxiliary rules should NOT have precedence
    for rule in grammar.all_rules() {
        if rule.lhs != lhs {
            assert!(
                rule.precedence.is_none(),
                "auxiliary rules should not inherit precedence"
            );
        }
    }
}

#[test]
fn normalization_of_repeat_preserves_parent_precedence() {
    let mut grammar = Grammar::new("rep_prec".to_string());

    let lhs = SymbolId(1);
    let inner = SymbolId(2);
    grammar.tokens.insert(
        inner,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(inner)))],
        precedence: Some(PrecedenceKind::Dynamic(7)),
        associativity: Some(Associativity::None),
        fields: Vec::new(),
        production_id: ProductionId(0),
    });

    grammar.normalize();

    let parent_rule = grammar
        .get_rules_for_symbol(lhs)
        .unwrap()
        .iter()
        .find(|r| r.precedence.is_some())
        .expect("parent rule must retain precedence");
    assert_eq!(parent_rule.precedence, Some(PrecedenceKind::Dynamic(7)));
    assert_eq!(parent_rule.associativity, Some(Associativity::None));
}

// ---------------------------------------------------------------------------
// 5. Multiple rules with same LHS but different precedence
// ---------------------------------------------------------------------------

#[test]
fn multiple_rules_same_lhs_different_precedences() {
    let grammar = GrammarBuilder::new("multi_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let expr_rules = grammar.get_rules_for_symbol(expr_id).unwrap();

    // Should have 4 rules total
    assert_eq!(expr_rules.len(), 4);

    // Collect precedence levels
    let prec_levels: Vec<Option<PrecedenceKind>> =
        expr_rules.iter().map(|r| r.precedence).collect();

    // Three rules with precedence, one without
    let with_prec = prec_levels.iter().filter(|p| p.is_some()).count();
    let without_prec = prec_levels.iter().filter(|p| p.is_none()).count();
    assert_eq!(with_prec, 3);
    assert_eq!(without_prec, 1);
}

#[test]
fn rules_with_mixed_associativity_same_lhs() {
    let grammar = GrammarBuilder::new("mixed_assoc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let expr_rules = grammar.get_rules_for_symbol(expr_id).unwrap();

    let add_rule = expr_rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .unwrap();
    let pow_rule = expr_rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(3)))
        .unwrap();

    assert_eq!(add_rule.associativity, Some(Associativity::Left));
    assert_eq!(pow_rule.associativity, Some(Associativity::Right));
}

#[test]
fn rules_with_non_associativity() {
    let grammar = GrammarBuilder::new("non_assoc")
        .token("NUM", r"\d+")
        .token("<", "<")
        .rule_with_precedence("expr", vec!["expr", "<", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    let cmp_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(cmp_rule.associativity, Some(Associativity::None));
}

// ---------------------------------------------------------------------------
// 6. Builder API creating rules with precedence
// ---------------------------------------------------------------------------

#[test]
fn builder_rule_with_precedence_sets_static() {
    let grammar = GrammarBuilder::new("builder_prec")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("start", vec!["a", "b"], 10, Associativity::Left)
        .start("start")
        .build();

    let s_id = grammar.find_symbol_by_name("start").unwrap();
    let rules = grammar.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(10)));
    assert_eq!(rules[0].associativity, Some(Associativity::Left));
}

#[test]
fn builder_plain_rule_has_no_precedence() {
    let grammar = GrammarBuilder::new("no_prec")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let s_id = grammar.find_symbol_by_name("start").unwrap();
    let rules = grammar.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(rules[0].precedence, None);
    assert_eq!(rules[0].associativity, None);
}

#[test]
fn builder_precedence_declaration() {
    let grammar = GrammarBuilder::new("prec_decl")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUM"])
        .precedence(1, Associativity::Left, vec!["expr"])
        .start("expr")
        .build();

    assert_eq!(grammar.precedences.len(), 1);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[0].associativity, Associativity::Left);
    assert_eq!(grammar.precedences[0].symbols.len(), 1);
}

#[test]
fn builder_multiple_precedence_declarations() {
    let grammar = GrammarBuilder::new("multi_decl")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUM"])
        .precedence(1, Associativity::Left, vec!["expr"])
        .precedence(2, Associativity::Left, vec!["expr"])
        .precedence(3, Associativity::Right, vec!["expr"])
        .start("expr")
        .build();

    assert_eq!(grammar.precedences.len(), 3);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[1].level, 2);
    assert_eq!(grammar.precedences[2].level, 3);
    assert_eq!(grammar.precedences[2].associativity, Associativity::Right);
}

#[test]
fn builder_javascript_like_has_four_precedence_rules() {
    let grammar = GrammarBuilder::javascript_like();

    let prec_rules: Vec<_> = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .collect();
    // +, -, *, / each have a precedence rule
    assert_eq!(prec_rules.len(), 4);

    // + and - should be precedence 1
    let prec1: Vec<_> = prec_rules
        .iter()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .collect();
    assert_eq!(prec1.len(), 2);

    // * and / should be precedence 2
    let prec2: Vec<_> = prec_rules
        .iter()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .collect();
    assert_eq!(prec2.len(), 2);
}

// ---------------------------------------------------------------------------
// 7. Serialization of precedence/associativity fields
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_rule_with_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2)), Symbol::Terminal(SymbolId(3))],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };

    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.precedence, rule.precedence);
    assert_eq!(deserialized.associativity, rule.associativity);
    assert_eq!(deserialized.fields, rule.fields);
}

#[test]
fn serde_roundtrip_rule_without_precedence() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(3),
    };

    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.precedence, None);
    assert_eq!(deserialized.associativity, None);
}

#[test]
fn serde_roundtrip_dynamic_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Dynamic(42)),
        associativity: Some(Associativity::Right),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };

    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.precedence, Some(PrecedenceKind::Dynamic(42)));
    assert_eq!(deserialized.associativity, Some(Associativity::Right));
}

#[test]
fn serde_roundtrip_precedence_declaration() {
    let prec = Precedence {
        level: 7,
        associativity: Associativity::None,
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
    };

    let json = serde_json::to_string(&prec).unwrap();
    let deserialized: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.level, 7);
    assert_eq!(deserialized.associativity, Associativity::None);
    assert_eq!(deserialized.symbols.len(), 3);
}

#[test]
fn serde_roundtrip_associativity_all_variants() {
    for variant in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: Associativity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn serde_roundtrip_precedence_kind_all_variants() {
    let variants = [
        PrecedenceKind::Static(0),
        PrecedenceKind::Static(100),
        PrecedenceKind::Static(-100),
        PrecedenceKind::Dynamic(0),
        PrecedenceKind::Dynamic(50),
        PrecedenceKind::Dynamic(-50),
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, variant);
    }
}

#[test]
fn serde_roundtrip_full_grammar_with_precedence() {
    let grammar = GrammarBuilder::new("serde_test")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .precedence(1, Associativity::Left, vec!["expr"])
        .precedence(2, Associativity::Left, vec!["expr"])
        .start("expr")
        .build();

    let json = serde_json::to_string(&grammar).unwrap();
    let deserialized: Grammar = serde_json::from_str(&json).unwrap();

    // Verify precedences survived roundtrip
    assert_eq!(deserialized.precedences.len(), grammar.precedences.len());
    for (orig, deser) in grammar
        .precedences
        .iter()
        .zip(deserialized.precedences.iter())
    {
        assert_eq!(orig.level, deser.level);
        assert_eq!(orig.associativity, deser.associativity);
    }

    // Verify rule-level precedences survived roundtrip
    let orig_prec_count = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .count();
    let deser_prec_count = deserialized
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .count();
    assert_eq!(orig_prec_count, deser_prec_count);
}

// ---------------------------------------------------------------------------
// Additional edge cases
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_by_precedence() {
    let resolution = ConflictResolution::Precedence(PrecedenceKind::Static(5));
    assert_eq!(
        resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
    assert_ne!(
        resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(3))
    );
}

#[test]
fn conflict_resolution_by_associativity() {
    let resolution = ConflictResolution::Associativity(Associativity::Left);
    assert_eq!(
        resolution,
        ConflictResolution::Associativity(Associativity::Left)
    );
    assert_ne!(
        resolution,
        ConflictResolution::Associativity(Associativity::Right)
    );
}

#[test]
fn precedence_struct_with_multiple_symbols() {
    let prec = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
    };
    assert_eq!(prec.level, 2);
    assert_eq!(prec.symbols.len(), 3);
    assert_eq!(prec.associativity, Associativity::Left);
}

#[test]
fn grammar_add_rule_preserves_precedence() {
    let mut grammar = Grammar::new("manual".to_string());

    grammar.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(10)),
        associativity: Some(Associativity::Right),
        fields: Vec::new(),
        production_id: ProductionId(0),
    });

    let rules = grammar.get_rules_for_symbol(SymbolId(1)).unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(10)));
    assert_eq!(rules[0].associativity, Some(Associativity::Right));
}
