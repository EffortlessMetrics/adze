#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for precedence and associativity ordering in adze-ir.
//! Focuses on level comparison, ordering semantics, grammar-level declarations,
//! dynamic precedence values, conflict resolution integration, formatting,
//! trait derivations, and rule integration.

use adze_ir::*;

// ---------------------------------------------------------------------------
// 1. Precedence level comparison
// ---------------------------------------------------------------------------

#[test]
fn precedence_levels_ordered_by_value() {
    let low = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    };
    let high = Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(11)],
    };
    assert!(low.level < high.level);
}

#[test]
fn precedence_levels_equal_when_same_value() {
    let a = Precedence {
        level: 3,
        associativity: Associativity::Right,
        symbols: vec![],
    };
    let b = Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    assert_eq!(a.level, b.level);
}

#[test]
fn negative_precedence_lower_than_positive() {
    let neg = Precedence {
        level: -2,
        associativity: Associativity::None,
        symbols: vec![],
    };
    let pos = Precedence {
        level: 2,
        associativity: Associativity::None,
        symbols: vec![],
    };
    assert!(neg.level < pos.level);
}

#[test]
fn precedence_level_boundaries() {
    let min_prec = Precedence {
        level: i16::MIN,
        associativity: Associativity::Left,
        symbols: vec![],
    };
    let max_prec = Precedence {
        level: i16::MAX,
        associativity: Associativity::Left,
        symbols: vec![],
    };
    assert!(min_prec.level < max_prec.level);
    assert_eq!(max_prec.level.wrapping_sub(min_prec.level), -1);
}

// ---------------------------------------------------------------------------
// 2. Associativity enum variants
// ---------------------------------------------------------------------------

#[test]
fn associativity_left_variant() {
    let a = Associativity::Left;
    assert_eq!(a, Associativity::Left);
    assert_ne!(a, Associativity::Right);
    assert_ne!(a, Associativity::None);
}

#[test]
fn associativity_right_variant() {
    let a = Associativity::Right;
    assert_eq!(a, Associativity::Right);
    assert_ne!(a, Associativity::Left);
}

#[test]
fn associativity_none_variant() {
    let a = Associativity::None;
    assert_eq!(a, Associativity::None);
    assert_ne!(a, Associativity::Left);
    assert_ne!(a, Associativity::Right);
}

#[test]
fn all_associativity_variants_are_distinct() {
    let variants = [Associativity::Left, Associativity::Right, Associativity::None];
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

// ---------------------------------------------------------------------------
// 3. Precedence declarations in grammar context
// ---------------------------------------------------------------------------

#[test]
fn grammar_starts_with_no_precedences() {
    let g = Grammar::new("empty".to_string());
    assert!(g.precedences.is_empty());
}

#[test]
fn grammar_add_single_precedence_declaration() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(0)],
    });
    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.precedences[0].level, 1);
}

#[test]
fn grammar_precedence_declarations_preserve_insertion_order() {
    let mut g = Grammar::new("test".to_string());
    for level in [3, 1, 2] {
        g.precedences.push(Precedence {
            level,
            associativity: Associativity::Left,
            symbols: vec![],
        });
    }
    assert_eq!(g.precedences[0].level, 3);
    assert_eq!(g.precedences[1].level, 1);
    assert_eq!(g.precedences[2].level, 2);
}

#[test]
fn grammar_precedence_symbols_reference_tokens() {
    let mut g = Grammar::new("arith".to_string());
    let plus = SymbolId(1);
    let star = SymbolId(2);
    g.tokens.insert(
        plus,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        star,
        Token {
            name: "*".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![star],
    });
    assert!(g.precedences[0].symbols.contains(&plus));
    assert!(g.precedences[1].symbols.contains(&star));
    assert!(g.precedences[0].level < g.precedences[1].level);
}

// ---------------------------------------------------------------------------
// 4. Multiple precedence levels ordering
// ---------------------------------------------------------------------------

#[test]
fn sort_precedences_by_level() {
    let mut precs = vec![
        Precedence { level: 5, associativity: Associativity::Left, symbols: vec![] },
        Precedence { level: 1, associativity: Associativity::Right, symbols: vec![] },
        Precedence { level: 3, associativity: Associativity::None, symbols: vec![] },
    ];
    precs.sort_by_key(|p| p.level);
    assert_eq!(precs[0].level, 1);
    assert_eq!(precs[1].level, 3);
    assert_eq!(precs[2].level, 5);
}

#[test]
fn many_precedence_levels_ascending_order() {
    let levels: Vec<i16> = (-10..=10).collect();
    let precs: Vec<Precedence> = levels
        .iter()
        .map(|&l| Precedence {
            level: l,
            associativity: Associativity::Left,
            symbols: vec![],
        })
        .collect();
    for i in 1..precs.len() {
        assert!(precs[i - 1].level < precs[i].level);
    }
}

#[test]
fn precedence_levels_with_mixed_associativity() {
    let mut g = Grammar::new("mixed".to_string());
    let assocs = [Associativity::Left, Associativity::Right, Associativity::None];
    for (i, assoc) in assocs.iter().enumerate() {
        g.precedences.push(Precedence {
            level: (i + 1) as i16,
            associativity: *assoc,
            symbols: vec![SymbolId(i as u16)],
        });
    }
    assert_eq!(g.precedences.len(), 3);
    assert_eq!(g.precedences[0].associativity, Associativity::Left);
    assert_eq!(g.precedences[1].associativity, Associativity::Right);
    assert_eq!(g.precedences[2].associativity, Associativity::None);
}

// ---------------------------------------------------------------------------
// 5. Dynamic precedence values
// ---------------------------------------------------------------------------

#[test]
fn dynamic_precedence_kind_distinct_from_static() {
    let s = PrecedenceKind::Static(5);
    let d = PrecedenceKind::Dynamic(5);
    assert_ne!(s, d);
}

#[test]
fn dynamic_precedence_values_compare() {
    let low = PrecedenceKind::Dynamic(1);
    let high = PrecedenceKind::Dynamic(10);
    match (low, high) {
        (PrecedenceKind::Dynamic(a), PrecedenceKind::Dynamic(b)) => assert!(a < b),
        _ => panic!("expected Dynamic variants"),
    }
}

#[test]
fn rule_with_dynamic_precedence() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: Some(PrecedenceKind::Dynamic(3)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(3)));
}

#[test]
fn dynamic_precedence_negative_values() {
    let pk = PrecedenceKind::Dynamic(-5);
    match pk {
        PrecedenceKind::Dynamic(v) => assert_eq!(v, -5),
        _ => panic!("expected Dynamic"),
    }
}

// ---------------------------------------------------------------------------
// 6. Precedence with conflict resolution
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_by_precedence() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(2));
    assert_eq!(cr, ConflictResolution::Precedence(PrecedenceKind::Static(2)));
}

#[test]
fn conflict_resolution_by_associativity() {
    let cr = ConflictResolution::Associativity(Associativity::Right);
    assert_eq!(cr, ConflictResolution::Associativity(Associativity::Right));
}

#[test]
fn conflict_resolution_glr_differs_from_precedence() {
    let glr = ConflictResolution::GLR;
    let prec = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    assert_ne!(glr, prec);
}

#[test]
fn conflict_declaration_with_precedence_resolution() {
    let mut g = Grammar::new("conflict_test".to_string());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(3)),
    });
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(
        g.conflicts[0].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(3))
    );
}

#[test]
fn conflict_declaration_with_associativity_resolution() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(5), SymbolId(6)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    };
    match &decl.resolution {
        ConflictResolution::Associativity(a) => assert_eq!(*a, Associativity::Left),
        _ => panic!("expected Associativity resolution"),
    }
}

// ---------------------------------------------------------------------------
// 7. Display/Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn precedence_kind_debug_static() {
    let pk = PrecedenceKind::Static(7);
    let dbg = format!("{pk:?}");
    assert!(dbg.contains("Static"));
    assert!(dbg.contains("7"));
}

#[test]
fn precedence_kind_debug_dynamic() {
    let pk = PrecedenceKind::Dynamic(-2);
    let dbg = format!("{pk:?}");
    assert!(dbg.contains("Dynamic"));
    assert!(dbg.contains("-2"));
}

#[test]
fn associativity_debug_formatting() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

#[test]
fn precedence_struct_debug_includes_fields() {
    let p = Precedence {
        level: 4,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(10), SymbolId(20)],
    };
    let dbg = format!("{p:?}");
    assert!(dbg.contains("level: 4"));
    assert!(dbg.contains("Right"));
    assert!(dbg.contains("10"));
    assert!(dbg.contains("20"));
}

#[test]
fn conflict_resolution_debug_all_variants() {
    let variants = [
        ConflictResolution::Precedence(PrecedenceKind::Static(1)),
        ConflictResolution::Associativity(Associativity::Left),
        ConflictResolution::GLR,
    ];
    let debugs: Vec<String> = variants.iter().map(|v| format!("{v:?}")).collect();
    assert!(debugs[0].contains("Precedence"));
    assert!(debugs[1].contains("Associativity"));
    assert!(debugs[2].contains("GLR"));
}

// ---------------------------------------------------------------------------
// 8. Clone/Eq behavior
// ---------------------------------------------------------------------------

#[test]
fn precedence_kind_clone_preserves_equality() {
    let original = PrecedenceKind::Static(42);
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn associativity_clone_preserves_equality() {
    let original = Associativity::Right;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn precedence_struct_clone_is_independent() {
    let original = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let mut cloned = original.clone();
    cloned.level = 99;
    cloned.symbols.push(SymbolId(3));
    assert_eq!(original.level, 2);
    assert_eq!(original.symbols.len(), 2);
    assert_eq!(cloned.level, 99);
    assert_eq!(cloned.symbols.len(), 3);
}

#[test]
fn conflict_resolution_eq_same_variant() {
    let a = ConflictResolution::GLR;
    let b = ConflictResolution::GLR;
    assert_eq!(a, b);
}

#[test]
fn conflict_resolution_ne_different_precedence_values() {
    let a = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    let b = ConflictResolution::Precedence(PrecedenceKind::Static(2));
    assert_ne!(a, b);
}

// ---------------------------------------------------------------------------
// 9. Integration with rules
// ---------------------------------------------------------------------------

#[test]
fn rule_with_static_precedence_and_associativity() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(2)));
    assert_eq!(rule.associativity, Some(Associativity::Left));
}

#[test]
fn rule_without_precedence_or_associativity() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert!(rule.precedence.is_none());
    assert!(rule.associativity.is_none());
}

#[test]
fn multiple_rules_with_different_precedence_levels() {
    let rules = vec![
        Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        },
        Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: Some(PrecedenceKind::Static(2)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(1),
        },
        Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(3))],
            precedence: Some(PrecedenceKind::Static(3)),
            associativity: Some(Associativity::Right),
            fields: vec![],
            production_id: ProductionId(2),
        },
    ];

    // Verify we can sort rules by precedence level
    let mut sorted: Vec<_> = rules.iter().collect();
    sorted.sort_by_key(|r| match r.precedence {
        Some(PrecedenceKind::Static(v)) => v,
        _ => 0,
    });
    assert_eq!(sorted[0].production_id, ProductionId(0));
    assert_eq!(sorted[1].production_id, ProductionId(1));
    assert_eq!(sorted[2].production_id, ProductionId(2));
}

#[test]
fn grammar_rules_reference_precedence_declarations() {
    let mut g = Grammar::new("expr".to_string());
    let expr = SymbolId(0);
    let plus = SymbolId(1);
    let star = SymbolId(2);

    g.tokens.insert(
        plus,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        star,
        Token {
            name: "*".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(expr, "expr".to_string());

    // Precedence: + at level 1 (left), * at level 2 (left)
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![star],
    });

    // Rules using those precedence levels
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(plus),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(star),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    });

    let rules = g.get_rules_for_symbol(expr).unwrap();
    assert_eq!(rules.len(), 2);

    // Verify the mul rule has higher precedence
    let add_rule = &rules[0];
    let mul_rule = &rules[1];
    match (add_rule.precedence, mul_rule.precedence) {
        (Some(PrecedenceKind::Static(a)), Some(PrecedenceKind::Static(b))) => {
            assert!(a < b, "addition should have lower precedence than multiplication");
        }
        _ => panic!("expected static precedence on both rules"),
    }
}

#[test]
fn precedence_serde_roundtrip() {
    let prec = Precedence {
        level: 7,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(3), SymbolId(4)],
    };
    let json = serde_json::to_string(&prec).unwrap();
    let restored: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.level, 7);
    assert_eq!(restored.associativity, Associativity::Right);
    assert_eq!(restored.symbols, vec![SymbolId(3), SymbolId(4)]);
}

#[test]
fn precedence_kind_serde_roundtrip() {
    for pk in [
        PrecedenceKind::Static(0),
        PrecedenceKind::Static(-100),
        PrecedenceKind::Dynamic(50),
        PrecedenceKind::Dynamic(i16::MIN),
    ] {
        let json = serde_json::to_string(&pk).unwrap();
        let restored: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(pk, restored);
    }
}

#[test]
fn associativity_serde_roundtrip() {
    for assoc in [Associativity::Left, Associativity::Right, Associativity::None] {
        let json = serde_json::to_string(&assoc).unwrap();
        let restored: Associativity = serde_json::from_str(&json).unwrap();
        assert_eq!(assoc, restored);
    }
}
