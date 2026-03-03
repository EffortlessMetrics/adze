#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the Precedence type, PrecedenceKind, Associativity,
//! and their interactions with Grammar rules, normalization, serialization,
//! and conflict resolution in the IR crate.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// 1. PrecedenceKind construction and equality
// ---------------------------------------------------------------------------

#[test]
fn static_precedence_zero() {
    let pk = PrecedenceKind::Static(0);
    assert_eq!(pk, PrecedenceKind::Static(0));
}

#[test]
fn static_precedence_positive() {
    assert_eq!(PrecedenceKind::Static(5), PrecedenceKind::Static(5));
}

#[test]
fn static_precedence_negative() {
    assert_eq!(PrecedenceKind::Static(-3), PrecedenceKind::Static(-3));
}

#[test]
fn dynamic_precedence_positive() {
    assert_eq!(PrecedenceKind::Dynamic(10), PrecedenceKind::Dynamic(10));
}

#[test]
fn dynamic_precedence_negative() {
    assert_eq!(PrecedenceKind::Dynamic(-1), PrecedenceKind::Dynamic(-1));
}

#[test]
fn static_and_dynamic_same_level_are_not_equal() {
    assert_ne!(PrecedenceKind::Static(1), PrecedenceKind::Dynamic(1));
}

#[test]
fn precedence_kind_i16_boundaries() {
    assert_eq!(
        PrecedenceKind::Static(i16::MAX),
        PrecedenceKind::Static(i16::MAX)
    );
    assert_eq!(
        PrecedenceKind::Static(i16::MIN),
        PrecedenceKind::Static(i16::MIN)
    );
    assert_eq!(
        PrecedenceKind::Dynamic(i16::MAX),
        PrecedenceKind::Dynamic(i16::MAX)
    );
    assert_eq!(
        PrecedenceKind::Dynamic(i16::MIN),
        PrecedenceKind::Dynamic(i16::MIN)
    );
    assert_ne!(
        PrecedenceKind::Static(i16::MIN),
        PrecedenceKind::Static(i16::MAX)
    );
}

#[test]
fn precedence_kind_copy_semantics() {
    let a = PrecedenceKind::Static(7);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn precedence_kind_debug_contains_variant_name() {
    let s = format!("{:?}", PrecedenceKind::Static(3));
    assert!(s.contains("Static"));
    assert!(s.contains("3"));
    let d = format!("{:?}", PrecedenceKind::Dynamic(-2));
    assert!(d.contains("Dynamic"));
}

// ---------------------------------------------------------------------------
// 2. Associativity construction, equality, and traits
// ---------------------------------------------------------------------------

#[test]
fn associativity_all_variants_distinct() {
    let variants = [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ];
    for i in 0..variants.len() {
        for j in 0..variants.len() {
            if i == j {
                assert_eq!(variants[i], variants[j]);
            } else {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }
}

#[test]
fn associativity_copy_semantics() {
    let a = Associativity::Right;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn associativity_debug_format() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

// ---------------------------------------------------------------------------
// 3. Precedence declaration struct
// ---------------------------------------------------------------------------

#[test]
fn precedence_declaration_with_empty_symbols() {
    let prec = Precedence {
        level: 0,
        associativity: Associativity::None,
        symbols: vec![],
    };
    assert_eq!(prec.level, 0);
    assert!(prec.symbols.is_empty());
}

#[test]
fn precedence_declaration_with_multiple_symbols() {
    let prec = Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
    };
    assert_eq!(prec.level, 5);
    assert_eq!(prec.symbols.len(), 3);
    assert_eq!(prec.associativity, Associativity::Left);
}

#[test]
fn precedence_declaration_negative_level() {
    let prec = Precedence {
        level: -10,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(42)],
    };
    assert_eq!(prec.level, -10);
}

#[test]
fn precedence_declaration_clone() {
    let prec = Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    let cloned = prec.clone();
    assert_eq!(cloned.level, 3);
    assert_eq!(cloned.associativity, Associativity::Left);
    assert_eq!(cloned.symbols, vec![SymbolId(1)]);
}

// ---------------------------------------------------------------------------
// 4. Rule-level precedence and associativity
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
fn rule_with_static_precedence_and_left_assoc() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(2)),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(rule.associativity, Some(Associativity::Left));
}

#[test]
fn rule_with_dynamic_precedence_no_associativity() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Dynamic(10)),
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(10)));
    assert!(rule.associativity.is_none());
}

#[test]
fn rule_precedence_equality_via_clone() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(99)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(5),
    };
    let cloned = rule.clone();
    assert_eq!(rule.precedence, cloned.precedence);
    assert_eq!(rule.associativity, cloned.associativity);
    assert_eq!(rule.fields, cloned.fields);
}

// ---------------------------------------------------------------------------
// 5. Grammar add_rule preserves precedence
// ---------------------------------------------------------------------------

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

#[test]
fn grammar_multiple_rules_same_lhs_different_precedence() {
    let mut grammar = Grammar::new("multi".to_string());
    let lhs = SymbolId(1);
    for level in [1i16, 2, 3] {
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId(10 + level as u16))],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: Some(Associativity::Left),
            fields: Vec::new(),
            production_id: ProductionId(level as u16),
        });
    }

    let rules = grammar.get_rules_for_symbol(lhs).unwrap();
    assert_eq!(rules.len(), 3);
    for i in 0..rules.len() {
        assert_eq!(
            rules[i].precedence,
            Some(PrecedenceKind::Static((i + 1) as i16))
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Builder API — rule-level precedence
// ---------------------------------------------------------------------------

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
fn builder_mixed_prec_and_plain_rules() {
    let grammar = GrammarBuilder::new("mix")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 2);

    let with_prec = rules.iter().filter(|r| r.precedence.is_some()).count();
    let without_prec = rules.iter().filter(|r| r.precedence.is_none()).count();
    assert_eq!(with_prec, 1);
    assert_eq!(without_prec, 1);
}

// ---------------------------------------------------------------------------
// 7. Builder API — precedence declarations on grammar
// ---------------------------------------------------------------------------

#[test]
fn builder_single_precedence_declaration() {
    let grammar = GrammarBuilder::new("pd")
        .token("NUM", r"\d+")
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
fn builder_multiple_precedence_declarations_ordering() {
    let grammar = GrammarBuilder::new("multi_decl")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .precedence(1, Associativity::Left, vec!["expr"])
        .precedence(2, Associativity::Left, vec!["expr"])
        .precedence(3, Associativity::Right, vec!["expr"])
        .start("expr")
        .build();

    assert_eq!(grammar.precedences.len(), 3);
    for i in 0..grammar.precedences.len() {
        assert_eq!(grammar.precedences[i].level, (i + 1) as i16);
    }
    assert_eq!(grammar.precedences[2].associativity, Associativity::Right);
}

#[test]
fn builder_precedence_declaration_with_multiple_symbols() {
    let grammar = GrammarBuilder::new("multi_sym")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("expr", vec!["NUM"])
        .rule("term", vec!["ID"])
        .precedence(1, Associativity::Left, vec!["expr", "term"])
        .start("expr")
        .build();

    assert_eq!(grammar.precedences[0].symbols.len(), 2);
}

// ---------------------------------------------------------------------------
// 8. JavaScript-like grammar precedence hierarchy
// ---------------------------------------------------------------------------

#[test]
fn javascript_like_four_precedence_rules() {
    let grammar = GrammarBuilder::javascript_like();

    let prec_rules: Vec<_> = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert_eq!(prec_rules.len(), 4);
}

#[test]
fn javascript_like_additive_vs_multiplicative() {
    let grammar = GrammarBuilder::javascript_like();
    let prec_rules: Vec<_> = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .collect();

    let prec1: Vec<_> = prec_rules
        .iter()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .collect();
    let prec2: Vec<_> = prec_rules
        .iter()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .collect();

    assert_eq!(prec1.len(), 2, "+/- at level 1");
    assert_eq!(prec2.len(), 2, "* / at level 2");
}

#[test]
fn javascript_like_all_left_associative() {
    let grammar = GrammarBuilder::javascript_like();
    for rule in grammar.all_rules() {
        if rule.precedence.is_some() {
            assert_eq!(rule.associativity, Some(Associativity::Left));
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Normalization preserves / does not inherit precedence
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

    let prec_rules: Vec<_> = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert!(!prec_rules.is_empty());
    assert_eq!(prec_rules[0].precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(prec_rules[0].associativity, Some(Associativity::Left));
}

#[test]
fn normalization_optional_aux_has_no_precedence() {
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

    // Parent keeps precedence
    let parent_rules: Vec<_> = grammar
        .get_rules_for_symbol(lhs)
        .unwrap()
        .iter()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert!(!parent_rules.is_empty());

    // Auxiliary rules do not gain precedence
    for rule in grammar.all_rules() {
        if rule.lhs != lhs {
            assert!(rule.precedence.is_none(), "aux rule must not inherit prec");
        }
    }
}

#[test]
fn normalization_repeat_aux_has_no_precedence() {
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
        .expect("parent must retain precedence");
    assert_eq!(parent_rule.precedence, Some(PrecedenceKind::Dynamic(7)));
}

#[test]
fn normalization_repeat_one_aux_has_no_precedence() {
    let mut grammar = Grammar::new("rep1".to_string());
    let lhs = SymbolId(1);
    let inner = SymbolId(2);
    grammar.tokens.insert(
        inner,
        Token {
            name: "y".to_string(),
            pattern: TokenPattern::String("y".to_string()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(inner)))],
        precedence: Some(PrecedenceKind::Static(9)),
        associativity: Some(Associativity::Left),
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    grammar.normalize();

    for rule in grammar.all_rules() {
        if rule.lhs != lhs {
            assert!(rule.precedence.is_none());
        }
    }
    let kept = grammar
        .get_rules_for_symbol(lhs)
        .unwrap()
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(9)));
    assert!(kept);
}

// ---------------------------------------------------------------------------
// 10. ConflictResolution interactions with precedence
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_precedence_static() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Static(5));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
    assert_ne!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(3))
    );
}

#[test]
fn conflict_resolution_precedence_dynamic() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Dynamic(2));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(2))
    );
    assert_ne!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(2))
    );
}

#[test]
fn conflict_resolution_associativity() {
    let res = ConflictResolution::Associativity(Associativity::Left);
    assert_eq!(res, ConflictResolution::Associativity(Associativity::Left));
    assert_ne!(res, ConflictResolution::Associativity(Associativity::Right));
    assert_ne!(res, ConflictResolution::GLR);
}

#[test]
fn conflict_resolution_glr_distinct_from_precedence() {
    assert_ne!(
        ConflictResolution::GLR,
        ConflictResolution::Precedence(PrecedenceKind::Static(0))
    );
    assert_ne!(
        ConflictResolution::GLR,
        ConflictResolution::Associativity(Associativity::None)
    );
}

// ---------------------------------------------------------------------------
// 11. Serialization roundtrips
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_precedence_kind_all_variants() {
    let variants = [
        PrecedenceKind::Static(0),
        PrecedenceKind::Static(100),
        PrecedenceKind::Static(-100),
        PrecedenceKind::Static(i16::MAX),
        PrecedenceKind::Static(i16::MIN),
        PrecedenceKind::Dynamic(0),
        PrecedenceKind::Dynamic(50),
        PrecedenceKind::Dynamic(-50),
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let de: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(de, variant);
    }
}

#[test]
fn serde_roundtrip_associativity_all_variants() {
    for variant in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let de: Associativity = serde_json::from_str(&json).unwrap();
        assert_eq!(de, variant);
    }
}

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
    let de: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(de.precedence, rule.precedence);
    assert_eq!(de.associativity, rule.associativity);
    assert_eq!(de.fields, rule.fields);
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
    let de: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(de.precedence, None);
    assert_eq!(de.associativity, None);
}

#[test]
fn serde_roundtrip_precedence_declaration() {
    let prec = Precedence {
        level: 7,
        associativity: Associativity::None,
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
    };
    let json = serde_json::to_string(&prec).unwrap();
    let de: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(de.level, 7);
    assert_eq!(de.associativity, Associativity::None);
    assert_eq!(de.symbols.len(), 3);
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
    let de: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(de.precedences.len(), grammar.precedences.len());
    for (orig, des) in grammar.precedences.iter().zip(de.precedences.iter()) {
        assert_eq!(orig.level, des.level);
        assert_eq!(orig.associativity, des.associativity);
    }

    let orig_count = grammar
        .all_rules()
        .filter(|r| r.precedence.is_some())
        .count();
    let de_count = de.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(orig_count, de_count);
}

#[test]
fn serde_roundtrip_conflict_resolution_precedence() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Dynamic(42));
    let json = serde_json::to_string(&res).unwrap();
    let de: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(de, res);
}

// ---------------------------------------------------------------------------
// 12. Grammar-level precedence list interactions
// ---------------------------------------------------------------------------

#[test]
fn grammar_precedences_default_empty() {
    let grammar = Grammar::new("empty".to_string());
    assert!(grammar.precedences.is_empty());
}

#[test]
fn grammar_precedences_pushed_directly() {
    let mut grammar = Grammar::new("direct".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    });
    assert_eq!(grammar.precedences.len(), 2);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[1].level, 2);
}

// ---------------------------------------------------------------------------
// 13. Mixed associativity in an expression grammar
// ---------------------------------------------------------------------------

#[test]
fn mixed_left_right_none_associativity() {
    let grammar = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .token("<", "<")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "<", "expr"], 2, Associativity::None)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 4);

    let add = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .unwrap();
    let cmp = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .unwrap();
    let pow = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(3)))
        .unwrap();

    assert_eq!(add.associativity, Some(Associativity::Left));
    assert_eq!(cmp.associativity, Some(Associativity::None));
    assert_eq!(pow.associativity, Some(Associativity::Right));
}

// ---------------------------------------------------------------------------
// 14. ConflictDeclaration with precedence resolution
// ---------------------------------------------------------------------------

#[test]
fn conflict_declaration_with_precedence_resolution() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(5)),
    };
    assert_eq!(decl.symbols.len(), 2);
    assert_eq!(
        decl.resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
}

#[test]
fn conflict_declaration_with_associativity_resolution() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(3)],
        resolution: ConflictResolution::Associativity(Associativity::Right),
    };
    assert_eq!(
        decl.resolution,
        ConflictResolution::Associativity(Associativity::Right)
    );
}

// ---------------------------------------------------------------------------
// 15. Edge cases and stress
// ---------------------------------------------------------------------------

#[test]
fn many_precedence_levels_on_same_lhs() {
    let mut builder = GrammarBuilder::new("many")
        .token("NUM", r"\d+")
        .token("a", "a");

    for level in -10i16..=10 {
        let tok_name = format!("op{}", (level + 20) as u8 as char);
        builder = builder.token(&tok_name, &tok_name);
        builder = builder.rule_with_precedence(
            "expr",
            vec!["expr", &tok_name, "expr"],
            level,
            Associativity::Left,
        );
    }
    builder = builder.rule("expr", vec!["NUM"]).start("expr");
    let grammar = builder.build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();

    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 21); // -10..=10
}

#[test]
fn precedence_kind_pattern_matching() {
    let pk = PrecedenceKind::Static(42);
    match pk {
        PrecedenceKind::Static(n) => assert_eq!(n, 42),
        PrecedenceKind::Dynamic(_) => panic!("expected Static"),
    }

    let pk2 = PrecedenceKind::Dynamic(-5);
    match pk2 {
        PrecedenceKind::Dynamic(n) => assert_eq!(n, -5),
        PrecedenceKind::Static(_) => panic!("expected Dynamic"),
    }
}
