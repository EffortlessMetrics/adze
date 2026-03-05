//! Comprehensive tests for precedence and associativity handling in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ============================================================================
// 1. Precedence construction and defaults (8 tests)
// ============================================================================

#[test]
fn test_precedence_new_with_level_and_associativity() {
    let prec = Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    assert_eq!(prec.level, 5);
    assert_eq!(prec.associativity, Associativity::Left);
    assert!(prec.symbols.is_empty());
}

#[test]
fn test_precedence_zero_level() {
    let prec = Precedence {
        level: 0,
        associativity: Associativity::None,
        symbols: Vec::new(),
    };
    assert_eq!(prec.level, 0);
    assert_eq!(prec.associativity, Associativity::None);
}

#[test]
fn test_precedence_negative_level() {
    let prec = Precedence {
        level: -3,
        associativity: Associativity::Right,
        symbols: Vec::new(),
    };
    assert_eq!(prec.level, -3);
    assert_eq!(prec.associativity, Associativity::Right);
}

#[test]
fn test_precedence_with_symbols() {
    let prec = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    assert_eq!(prec.symbols.len(), 2);
    assert_eq!(prec.symbols[0], SymbolId(1));
    assert_eq!(prec.symbols[1], SymbolId(2));
}

#[test]
fn test_precedence_clone_is_equal() {
    let prec = Precedence {
        level: 10,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(5)],
    };
    let cloned = prec.clone();
    assert_eq!(prec, cloned);
}

#[test]
fn test_precedence_debug_format() {
    let prec = Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: Vec::new(),
    };
    let debug = format!("{:?}", prec);
    assert!(debug.contains("level: 2"));
    assert!(debug.contains("Right"));
}

#[test]
fn test_precedence_different_levels_not_equal() {
    let prec_a = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    let prec_b = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    assert_ne!(prec_a, prec_b);
}

#[test]
fn test_precedence_different_assoc_not_equal() {
    let prec_a = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    let prec_b = Precedence {
        level: 1,
        associativity: Associativity::Right,
        symbols: Vec::new(),
    };
    assert_ne!(prec_a, prec_b);
}

// ============================================================================
// 2. Associativity variants (5 tests)
// ============================================================================

#[test]
fn test_associativity_left() {
    let assoc = Associativity::Left;
    assert_eq!(assoc, Associativity::Left);
    assert_ne!(assoc, Associativity::Right);
    assert_ne!(assoc, Associativity::None);
}

#[test]
fn test_associativity_right() {
    let assoc = Associativity::Right;
    assert_eq!(assoc, Associativity::Right);
    assert_ne!(assoc, Associativity::Left);
}

#[test]
fn test_associativity_none() {
    let assoc = Associativity::None;
    assert_eq!(assoc, Associativity::None);
    assert_ne!(assoc, Associativity::Left);
    assert_ne!(assoc, Associativity::Right);
}

#[test]
fn test_associativity_copy_semantics() {
    let assoc = Associativity::Left;
    let copied = assoc;
    assert_eq!(assoc, copied);
}

#[test]
fn test_associativity_debug_format() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

// ============================================================================
// 3. PrecedenceEntry with SymbolId (via Precedence struct) (5 tests)
// ============================================================================

#[test]
fn test_precedence_single_symbol() {
    let prec = Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    };
    assert_eq!(prec.symbols.len(), 1);
    assert_eq!(prec.symbols[0], SymbolId(10));
}

#[test]
fn test_precedence_multiple_symbols() {
    let prec = Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
    };
    assert_eq!(prec.symbols.len(), 3);
}

#[test]
fn test_precedence_symbol_id_ordering() {
    let a = SymbolId(5);
    let b = SymbolId(10);
    assert!(a < b);
}

#[test]
fn test_symbol_id_display() {
    let sid = SymbolId(42);
    assert_eq!(format!("{sid}"), "Symbol(42)");
}

#[test]
fn test_symbol_id_copy_semantics() {
    let sid = SymbolId(7);
    let copied = sid;
    assert_eq!(sid, copied);
}

// ============================================================================
// 4. Grammar precedence entries (8 tests)
// ============================================================================

#[test]
fn test_grammar_default_has_empty_precedences() {
    let grammar = Grammar::default();
    assert!(grammar.precedences.is_empty());
}

#[test]
fn test_grammar_new_has_empty_precedences() {
    let grammar = Grammar::new("test".to_string());
    assert!(grammar.precedences.is_empty());
}

#[test]
fn test_grammar_add_precedence_directly() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    assert_eq!(grammar.precedences.len(), 1);
    assert_eq!(grammar.precedences[0].level, 1);
}

#[test]
fn test_grammar_multiple_precedence_levels() {
    let mut grammar = Grammar::new("calc".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(2)],
    });
    assert_eq!(grammar.precedences.len(), 2);
    assert!(grammar.precedences[0].level < grammar.precedences[1].level);
}

#[test]
fn test_grammar_precedence_with_different_associativities() {
    let mut grammar = Grammar::new("mixed".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    });
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: Vec::new(),
    });
    grammar.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::None,
        symbols: Vec::new(),
    });
    assert_eq!(grammar.precedences[0].associativity, Associativity::Left);
    assert_eq!(grammar.precedences[1].associativity, Associativity::Right);
    assert_eq!(grammar.precedences[2].associativity, Associativity::None);
}

#[test]
fn test_grammar_default_has_empty_conflicts() {
    let grammar = Grammar::default();
    assert!(grammar.conflicts.is_empty());
}

#[test]
fn test_grammar_precedence_preserves_symbol_order() {
    let mut grammar = Grammar::new("ordered".to_string());
    let symbols = vec![SymbolId(5), SymbolId(3), SymbolId(7)];
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: symbols.clone(),
    });
    assert_eq!(grammar.precedences[0].symbols, symbols);
}

#[test]
fn test_grammar_precedence_clone_equality() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    let cloned = grammar.clone();
    assert_eq!(grammar.precedences, cloned.precedences);
}

// ============================================================================
// 5. Builder with precedence rules (8 tests)
// ============================================================================

#[test]
fn test_builder_rule_with_precedence_left() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let add_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(add_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(add_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_builder_rule_with_precedence_right() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let pow_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(pow_rule.precedence, Some(PrecedenceKind::Static(3)));
    assert_eq!(pow_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_builder_rule_without_precedence_has_none() {
    let grammar = GrammarBuilder::new("simple")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    assert!(rules[0].precedence.is_none());
    assert!(rules[0].associativity.is_none());
}

#[test]
fn test_builder_multiple_precedence_levels() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 2);
}

#[test]
fn test_builder_precedence_declaration() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUMBER"])
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .start("expr")
        .build();

    assert_eq!(grammar.precedences.len(), 2);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[0].associativity, Associativity::Left);
    assert_eq!(grammar.precedences[1].level, 2);
}

#[test]
fn test_builder_precedence_declaration_with_multiple_symbols() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["NUMBER"])
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .start("expr")
        .build();

    assert_eq!(grammar.precedences.len(), 1);
    assert_eq!(grammar.precedences[0].symbols.len(), 2);
}

#[test]
fn test_builder_javascript_like_has_precedence_rules() {
    let grammar = GrammarBuilder::javascript_like();
    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expression")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    // + - * / all have precedence
    assert_eq!(prec_rules.len(), 4);
}

#[test]
fn test_builder_mixed_prec_and_no_prec_rules() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let with_prec = rules.iter().filter(|r| r.precedence.is_some()).count();
    let without_prec = rules.iter().filter(|r| r.precedence.is_none()).count();
    assert_eq!(with_prec, 1);
    assert_eq!(without_prec, 2);
}

// ============================================================================
// 6. Conflict declarations (8 tests)
// ============================================================================

#[test]
fn test_conflict_declaration_glr() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(conflict.symbols.len(), 2);
    assert_eq!(conflict.resolution, ConflictResolution::GLR);
}

#[test]
fn test_conflict_declaration_precedence_resolution() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(5)),
    };
    assert_eq!(
        conflict.resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
}

#[test]
fn test_conflict_declaration_associativity_resolution() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    };
    assert_eq!(
        conflict.resolution,
        ConflictResolution::Associativity(Associativity::Left)
    );
}

#[test]
fn test_conflict_declaration_dynamic_precedence() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(2)),
    };
    if let ConflictResolution::Precedence(PrecedenceKind::Dynamic(level)) = conflict.resolution {
        assert_eq!(level, 2);
    } else {
        panic!("Expected dynamic precedence resolution");
    }
}

#[test]
fn test_conflict_resolution_equality() {
    assert_eq!(ConflictResolution::GLR, ConflictResolution::GLR);
    assert_ne!(
        ConflictResolution::GLR,
        ConflictResolution::Associativity(Associativity::Left)
    );
}

#[test]
fn test_conflict_declaration_clone() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
        resolution: ConflictResolution::GLR,
    };
    let cloned = conflict.clone();
    assert_eq!(conflict, cloned);
}

#[test]
fn test_conflict_declaration_debug_format() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::GLR,
    };
    let debug = format!("{:?}", conflict);
    assert!(debug.contains("GLR"));
    assert!(debug.contains("SymbolId"));
}

#[test]
fn test_grammar_with_conflict_declarations() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3), SymbolId(4)],
        resolution: ConflictResolution::Associativity(Associativity::Right),
    });
    assert_eq!(grammar.conflicts.len(), 2);
}

// ============================================================================
// 7. Precedence serialization roundtrip (5 tests)
// ============================================================================

#[test]
fn test_associativity_serde_roundtrip() {
    for assoc in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        let json = serde_json::to_string(&assoc).unwrap();
        let deserialized: Associativity = serde_json::from_str(&json).unwrap();
        assert_eq!(assoc, deserialized);
    }
}

#[test]
fn test_precedence_serde_roundtrip() {
    let prec = Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let json = serde_json::to_string(&prec).unwrap();
    let deserialized: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(prec, deserialized);
}

#[test]
fn test_precedence_kind_serde_roundtrip() {
    let kinds = [PrecedenceKind::Static(3), PrecedenceKind::Dynamic(-1)];
    for kind in kinds {
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }
}

#[test]
fn test_conflict_declaration_serde_roundtrip() {
    let conflict = ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(20)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(2)),
    };
    let json = serde_json::to_string(&conflict).unwrap();
    let deserialized: ConflictDeclaration = serde_json::from_str(&json).unwrap();
    assert_eq!(conflict, deserialized);
}

#[test]
fn test_grammar_precedences_survive_serde_roundtrip() {
    let mut grammar = Grammar::new("serde_test".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2), SymbolId(3)],
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });

    let json = serde_json::to_string(&grammar).unwrap();
    let deserialized: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(grammar.precedences, deserialized.precedences);
    assert_eq!(grammar.conflicts, deserialized.conflicts);
}

// ============================================================================
// 8. Edge cases (8 tests)
// ============================================================================

#[test]
fn test_precedence_i16_max() {
    let prec = Precedence {
        level: i16::MAX,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    assert_eq!(prec.level, i16::MAX);
}

#[test]
fn test_precedence_i16_min() {
    let prec = Precedence {
        level: i16::MIN,
        associativity: Associativity::Right,
        symbols: Vec::new(),
    };
    assert_eq!(prec.level, i16::MIN);
}

#[test]
fn test_precedence_kind_static_negative() {
    let kind = PrecedenceKind::Static(-100);
    if let PrecedenceKind::Static(val) = kind {
        assert_eq!(val, -100);
    } else {
        panic!("Expected Static");
    }
}

#[test]
fn test_precedence_kind_dynamic_zero() {
    let kind = PrecedenceKind::Dynamic(0);
    assert_eq!(kind, PrecedenceKind::Dynamic(0));
    assert_ne!(kind, PrecedenceKind::Static(0));
}

#[test]
fn test_many_precedence_entries() {
    let mut grammar = Grammar::new("many_prec".to_string());
    for i in 0..100 {
        grammar.precedences.push(Precedence {
            level: i,
            associativity: Associativity::Left,
            symbols: vec![SymbolId(i as u16)],
        });
    }
    assert_eq!(grammar.precedences.len(), 100);
    assert_eq!(grammar.precedences[99].level, 99);
}

#[test]
fn test_precedence_empty_symbols_list() {
    let prec = Precedence {
        level: 1,
        associativity: Associativity::None,
        symbols: Vec::new(),
    };
    assert!(prec.symbols.is_empty());
}

#[test]
fn test_builder_rule_with_precedence_none_assoc() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("==", "==")
        .rule_with_precedence("expr", vec!["expr", "==", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let eq_rule = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::None))
        .unwrap();
    assert_eq!(eq_rule.precedence, Some(PrecedenceKind::Static(1)));
}

#[test]
fn test_builder_precedence_i16_extremes() {
    let grammar = GrammarBuilder::new("extremes")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence(
            "expr",
            vec!["expr", "+", "expr"],
            i16::MIN,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "*", "expr"],
            i16::MAX,
            Associativity::Left,
        )
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let rules = &grammar.rules[&expr_id];
    let prec_values: Vec<i16> = rules
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    assert!(prec_values.contains(&i16::MIN));
    assert!(prec_values.contains(&i16::MAX));
}
