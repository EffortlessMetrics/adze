//! Comprehensive tests for precedence and associativity in the IR crate.
//!
//! Covers PrecedenceKind, Associativity, Rule precedence composition,
//! ConflictResolution, PrecedenceDeclaration, GrammarBuilder precedence,
//! and property-based ordering tests.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ===========================================================================
// 1. PrecedenceKind::Static construction and comparison
// ===========================================================================

#[test]
fn static_zero_equals_itself() {
    assert_eq!(PrecedenceKind::Static(0), PrecedenceKind::Static(0));
}

#[test]
fn static_positive_equals_itself() {
    assert_eq!(PrecedenceKind::Static(42), PrecedenceKind::Static(42));
}

#[test]
fn static_negative_equals_itself() {
    assert_eq!(PrecedenceKind::Static(-7), PrecedenceKind::Static(-7));
}

#[test]
fn static_different_values_not_equal() {
    assert_ne!(PrecedenceKind::Static(1), PrecedenceKind::Static(2));
}

#[test]
fn static_i16_max() {
    let pk = PrecedenceKind::Static(i16::MAX);
    assert_eq!(pk, PrecedenceKind::Static(32767));
}

#[test]
fn static_i16_min() {
    let pk = PrecedenceKind::Static(i16::MIN);
    assert_eq!(pk, PrecedenceKind::Static(-32768));
}

#[test]
fn static_positive_and_negative_not_equal() {
    assert_ne!(PrecedenceKind::Static(5), PrecedenceKind::Static(-5));
}

// ===========================================================================
// 2. PrecedenceKind::Dynamic construction and comparison
// ===========================================================================

#[test]
fn dynamic_zero_equals_itself() {
    assert_eq!(PrecedenceKind::Dynamic(0), PrecedenceKind::Dynamic(0));
}

#[test]
fn dynamic_positive_equals_itself() {
    assert_eq!(PrecedenceKind::Dynamic(99), PrecedenceKind::Dynamic(99));
}

#[test]
fn dynamic_negative_equals_itself() {
    assert_eq!(PrecedenceKind::Dynamic(-10), PrecedenceKind::Dynamic(-10));
}

#[test]
fn dynamic_different_values_not_equal() {
    assert_ne!(PrecedenceKind::Dynamic(3), PrecedenceKind::Dynamic(4));
}

#[test]
fn dynamic_i16_max() {
    assert_eq!(
        PrecedenceKind::Dynamic(i16::MAX),
        PrecedenceKind::Dynamic(i16::MAX)
    );
}

#[test]
fn dynamic_i16_min() {
    assert_eq!(
        PrecedenceKind::Dynamic(i16::MIN),
        PrecedenceKind::Dynamic(i16::MIN)
    );
}

// ===========================================================================
// 3. Static vs Dynamic cross-variant comparison
// ===========================================================================

#[test]
fn static_and_dynamic_same_level_differ() {
    assert_ne!(PrecedenceKind::Static(1), PrecedenceKind::Dynamic(1));
}

#[test]
fn static_and_dynamic_zero_differ() {
    assert_ne!(PrecedenceKind::Static(0), PrecedenceKind::Dynamic(0));
}

#[test]
fn static_and_dynamic_negative_differ() {
    assert_ne!(PrecedenceKind::Static(-5), PrecedenceKind::Dynamic(-5));
}

// ===========================================================================
// 4. PrecedenceKind clone and copy semantics
// ===========================================================================

#[test]
fn static_clone_equals_original() {
    let pk = PrecedenceKind::Static(10);
    let cloned = pk;
    assert_eq!(pk, cloned);
}

#[test]
fn dynamic_clone_equals_original() {
    let pk = PrecedenceKind::Dynamic(-3);
    let cloned = pk;
    assert_eq!(pk, cloned);
}

// ===========================================================================
// 5. PrecedenceKind Debug formatting
// ===========================================================================

#[test]
fn static_debug_format() {
    let s = format!("{:?}", PrecedenceKind::Static(5));
    assert!(s.contains("Static"));
    assert!(s.contains("5"));
}

#[test]
fn dynamic_debug_format() {
    let s = format!("{:?}", PrecedenceKind::Dynamic(-2));
    assert!(s.contains("Dynamic"));
    assert!(s.contains("-2"));
}

// ===========================================================================
// 6. Associativity enum variants
// ===========================================================================

#[test]
fn associativity_left_equals_itself() {
    assert_eq!(Associativity::Left, Associativity::Left);
}

#[test]
fn associativity_right_equals_itself() {
    assert_eq!(Associativity::Right, Associativity::Right);
}

#[test]
fn associativity_none_equals_itself() {
    assert_eq!(Associativity::None, Associativity::None);
}

#[test]
fn associativity_left_ne_right() {
    assert_ne!(Associativity::Left, Associativity::Right);
}

#[test]
fn associativity_left_ne_none() {
    assert_ne!(Associativity::Left, Associativity::None);
}

#[test]
fn associativity_right_ne_none() {
    assert_ne!(Associativity::Right, Associativity::None);
}

#[test]
fn associativity_clone_preserves_variant() {
    let a = Associativity::Right;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn associativity_debug_left() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
}

#[test]
fn associativity_debug_right() {
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
}

#[test]
fn associativity_debug_none() {
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

// ===========================================================================
// 7. Associativity serde roundtrip
// ===========================================================================

#[test]
fn associativity_serde_roundtrip_left() {
    let json = serde_json::to_string(&Associativity::Left).unwrap();
    let back: Associativity = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Associativity::Left);
}

#[test]
fn associativity_serde_roundtrip_right() {
    let json = serde_json::to_string(&Associativity::Right).unwrap();
    let back: Associativity = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Associativity::Right);
}

#[test]
fn associativity_serde_roundtrip_none() {
    let json = serde_json::to_string(&Associativity::None).unwrap();
    let back: Associativity = serde_json::from_str(&json).unwrap();
    assert_eq!(back, Associativity::None);
}

// ===========================================================================
// 8. PrecedenceKind serde roundtrip
// ===========================================================================

#[test]
fn precedence_kind_serde_static() {
    let pk = PrecedenceKind::Static(7);
    let json = serde_json::to_string(&pk).unwrap();
    let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
    assert_eq!(back, pk);
}

#[test]
fn precedence_kind_serde_dynamic() {
    let pk = PrecedenceKind::Dynamic(-4);
    let json = serde_json::to_string(&pk).unwrap();
    let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
    assert_eq!(back, pk);
}

#[test]
fn precedence_kind_serde_boundary_values() {
    for val in [i16::MIN, -1, 0, 1, i16::MAX] {
        for pk in [PrecedenceKind::Static(val), PrecedenceKind::Dynamic(val)] {
            let json = serde_json::to_string(&pk).unwrap();
            let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
            assert_eq!(back, pk);
        }
    }
}

// ===========================================================================
// 9. Rule precedence composition
// ===========================================================================

fn make_rule(prec: Option<PrecedenceKind>, assoc: Option<Associativity>) -> Rule {
    Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: prec,
        associativity: assoc,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

#[test]
fn rule_no_precedence_no_associativity() {
    let r = make_rule(None, None);
    assert!(r.precedence.is_none());
    assert!(r.associativity.is_none());
}

#[test]
fn rule_static_precedence_left_assoc() {
    let r = make_rule(Some(PrecedenceKind::Static(3)), Some(Associativity::Left));
    assert_eq!(r.precedence, Some(PrecedenceKind::Static(3)));
    assert_eq!(r.associativity, Some(Associativity::Left));
}

#[test]
fn rule_dynamic_precedence_right_assoc() {
    let r = make_rule(Some(PrecedenceKind::Dynamic(2)), Some(Associativity::Right));
    assert_eq!(r.precedence, Some(PrecedenceKind::Dynamic(2)));
    assert_eq!(r.associativity, Some(Associativity::Right));
}

#[test]
fn rule_precedence_without_associativity() {
    let r = make_rule(Some(PrecedenceKind::Static(1)), None);
    assert!(r.precedence.is_some());
    assert!(r.associativity.is_none());
}

#[test]
fn rule_associativity_without_precedence() {
    let r = make_rule(None, Some(Associativity::None));
    assert!(r.precedence.is_none());
    assert_eq!(r.associativity, Some(Associativity::None));
}

#[test]
fn rule_equality_with_same_precedence() {
    let r1 = make_rule(Some(PrecedenceKind::Static(5)), Some(Associativity::Left));
    let r2 = make_rule(Some(PrecedenceKind::Static(5)), Some(Associativity::Left));
    assert_eq!(r1, r2);
}

#[test]
fn rule_inequality_different_precedence() {
    let r1 = make_rule(Some(PrecedenceKind::Static(1)), Some(Associativity::Left));
    let r2 = make_rule(Some(PrecedenceKind::Static(2)), Some(Associativity::Left));
    assert_ne!(r1, r2);
}

#[test]
fn rule_inequality_different_associativity() {
    let r1 = make_rule(Some(PrecedenceKind::Static(1)), Some(Associativity::Left));
    let r2 = make_rule(Some(PrecedenceKind::Static(1)), Some(Associativity::Right));
    assert_ne!(r1, r2);
}

#[test]
fn rule_serde_roundtrip_with_precedence() {
    let r = make_rule(
        Some(PrecedenceKind::Dynamic(10)),
        Some(Associativity::Right),
    );
    let json = serde_json::to_string(&r).unwrap();
    let back: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.precedence, r.precedence);
    assert_eq!(back.associativity, r.associativity);
}

// ===========================================================================
// 10. ConflictResolution variants
// ===========================================================================

#[test]
fn conflict_resolution_precedence_static() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(2));
    assert_eq!(
        cr,
        ConflictResolution::Precedence(PrecedenceKind::Static(2))
    );
}

#[test]
fn conflict_resolution_precedence_dynamic() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Dynamic(-1));
    assert_eq!(
        cr,
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(-1))
    );
}

#[test]
fn conflict_resolution_associativity_left() {
    let cr = ConflictResolution::Associativity(Associativity::Left);
    assert_eq!(cr, ConflictResolution::Associativity(Associativity::Left));
}

#[test]
fn conflict_resolution_associativity_right() {
    let cr = ConflictResolution::Associativity(Associativity::Right);
    assert_eq!(cr, ConflictResolution::Associativity(Associativity::Right));
}

#[test]
fn conflict_resolution_associativity_none() {
    let cr = ConflictResolution::Associativity(Associativity::None);
    assert_eq!(cr, ConflictResolution::Associativity(Associativity::None));
}

#[test]
fn conflict_resolution_glr() {
    let cr = ConflictResolution::GLR;
    assert_eq!(cr, ConflictResolution::GLR);
}

#[test]
fn conflict_resolution_variants_are_distinct() {
    let prec = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    let assoc = ConflictResolution::Associativity(Associativity::Left);
    let glr = ConflictResolution::GLR;
    assert_ne!(prec, assoc);
    assert_ne!(prec, glr);
    assert_ne!(assoc, glr);
}

#[test]
fn conflict_resolution_serde_roundtrip_precedence() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(3));
    let json = serde_json::to_string(&cr).unwrap();
    let back: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(back, cr);
}

#[test]
fn conflict_resolution_serde_roundtrip_associativity() {
    let cr = ConflictResolution::Associativity(Associativity::Right);
    let json = serde_json::to_string(&cr).unwrap();
    let back: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(back, cr);
}

#[test]
fn conflict_resolution_serde_roundtrip_glr() {
    let json = serde_json::to_string(&ConflictResolution::GLR).unwrap();
    let back: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(back, ConflictResolution::GLR);
}

#[test]
fn conflict_resolution_debug_glr() {
    let s = format!("{:?}", ConflictResolution::GLR);
    assert!(s.contains("GLR"));
}

// ===========================================================================
// 11. PrecedenceDeclaration (Precedence struct) handling
// ===========================================================================

#[test]
fn precedence_declaration_basic() {
    let p = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10), SymbolId(11)],
    };
    assert_eq!(p.level, 1);
    assert_eq!(p.associativity, Associativity::Left);
    assert_eq!(p.symbols.len(), 2);
}

#[test]
fn precedence_declaration_empty_symbols() {
    let p = Precedence {
        level: 0,
        associativity: Associativity::None,
        symbols: vec![],
    };
    assert!(p.symbols.is_empty());
}

#[test]
fn precedence_declaration_negative_level() {
    let p = Precedence {
        level: -5,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    };
    assert_eq!(p.level, -5);
}

#[test]
fn precedence_declaration_equality() {
    let p1 = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    let p2 = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    assert_eq!(p1, p2);
}

#[test]
fn precedence_declaration_inequality_different_level() {
    let p1 = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    let p2 = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    assert_ne!(p1, p2);
}

#[test]
fn precedence_declaration_inequality_different_assoc() {
    let p1 = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    };
    let p2 = Precedence {
        level: 1,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    };
    assert_ne!(p1, p2);
}

#[test]
fn precedence_declaration_serde_roundtrip() {
    let p = Precedence {
        level: 3,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(5), SymbolId(6), SymbolId(7)],
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(back, p);
}

#[test]
fn precedence_declaration_clone() {
    let p = Precedence {
        level: 4,
        associativity: Associativity::None,
        symbols: vec![SymbolId(100)],
    };
    let cloned = p.clone();
    assert_eq!(p, cloned);
}

// ===========================================================================
// 12. ConflictDeclaration with precedence/associativity resolution
// ===========================================================================

#[test]
fn conflict_declaration_with_precedence_resolution() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(5)),
    };
    assert_eq!(cd.symbols.len(), 2);
    assert_eq!(
        cd.resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
}

#[test]
fn conflict_declaration_with_associativity_resolution() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(3)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    };
    assert_eq!(
        cd.resolution,
        ConflictResolution::Associativity(Associativity::Left)
    );
}

#[test]
fn conflict_declaration_with_glr_resolution() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(cd.resolution, ConflictResolution::GLR);
    assert_eq!(cd.symbols.len(), 3);
}

#[test]
fn conflict_declaration_serde_roundtrip() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(20)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(2)),
    };
    let json = serde_json::to_string(&cd).unwrap();
    let back: ConflictDeclaration = serde_json::from_str(&json).unwrap();
    assert_eq!(back, cd);
}

// ===========================================================================
// 13. GrammarBuilder precedence declarations
// ===========================================================================

#[test]
fn builder_single_precedence_level() {
    let grammar = GrammarBuilder::new("test")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .build();
    assert_eq!(grammar.precedences.len(), 1);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[0].associativity, Associativity::Left);
}

#[test]
fn builder_multiple_precedence_levels() {
    let grammar = GrammarBuilder::new("test")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();
    assert_eq!(grammar.precedences.len(), 2);
    assert!(grammar.precedences[0].level < grammar.precedences[1].level);
}

#[test]
fn builder_precedence_with_multiple_symbols() {
    let grammar = GrammarBuilder::new("test")
        .token("+", "+")
        .token("-", "-")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .build();
    assert_eq!(grammar.precedences[0].symbols.len(), 2);
}

#[test]
fn builder_rule_with_precedence_sets_static() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();

    let rules = &grammar.rules[&expr_id];
    let add_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();

    assert_eq!(add_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(add_rule.associativity, Some(Associativity::Left));
}

#[test]
fn builder_multiple_rules_different_precedence() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();

    let rules = &grammar.rules[&expr_id];
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 2);

    // Verify different precedence levels exist
    let levels: Vec<i16> = prec_rules
        .iter()
        .map(|r| match r.precedence.unwrap() {
            PrecedenceKind::Static(v) => v,
            PrecedenceKind::Dynamic(v) => v,
        })
        .collect();
    assert!(levels.contains(&1));
    assert!(levels.contains(&2));
}

#[test]
fn builder_right_associative_rule() {
    let grammar = GrammarBuilder::new("assign")
        .token("ID", r"[a-z]+")
        .token("=", "=")
        .rule_with_precedence("assign", vec!["ID", "=", "assign"], 1, Associativity::Right)
        .rule("assign", vec!["ID"])
        .start("assign")
        .build();

    let assign_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "assign")
        .map(|(id, _)| *id)
        .unwrap();

    let rules = &grammar.rules[&assign_id];
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn builder_non_associative_rule() {
    let grammar = GrammarBuilder::new("cmp")
        .token("NUM", r"\d+")
        .token("<", "<")
        .rule_with_precedence("cmp", vec!["cmp", "<", "cmp"], 1, Associativity::None)
        .rule("cmp", vec!["NUM"])
        .start("cmp")
        .build();

    let cmp_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "cmp")
        .map(|(id, _)| *id)
        .unwrap();

    let rules = &grammar.rules[&cmp_id];
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::None));
}

#[test]
fn builder_precedence_declaration_preserved_in_grammar() {
    let grammar = GrammarBuilder::new("test")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .precedence(2, Associativity::Left, vec!["*", "/"])
        .build();

    assert_eq!(grammar.precedences.len(), 2);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[0].symbols.len(), 2);
    assert_eq!(grammar.precedences[1].level, 2);
    assert_eq!(grammar.precedences[1].symbols.len(), 2);
}

// ===========================================================================
// 14. Grammar-level precedence integration
// ===========================================================================

#[test]
fn grammar_direct_precedences_push() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    });
    grammar.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(20)],
    });
    assert_eq!(grammar.precedences.len(), 2);
}

#[test]
fn grammar_conflicts_with_mixed_resolution() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3), SymbolId(4)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(5), SymbolId(6)],
        resolution: ConflictResolution::GLR,
    });
    assert_eq!(grammar.conflicts.len(), 3);
}

// ===========================================================================
// 15. Property tests for precedence ordering
// ===========================================================================

#[test]
fn precedence_levels_form_total_order_on_i16() {
    // Verify that static precedence values maintain i16 ordering
    let low = PrecedenceKind::Static(-10);
    let mid = PrecedenceKind::Static(0);
    let high = PrecedenceKind::Static(10);

    if let (PrecedenceKind::Static(l), PrecedenceKind::Static(m), PrecedenceKind::Static(h)) =
        (low, mid, high)
    {
        assert!(l < m);
        assert!(m < h);
        assert!(l < h); // transitivity
    }
}

#[test]
fn dynamic_precedence_levels_maintain_order() {
    let vals: Vec<i16> = vec![-100, -1, 0, 1, 100];
    for window in vals.windows(2) {
        let a = window[0];
        let b = window[1];
        assert!(a < b, "{a} should be less than {b}");
    }
}

#[test]
fn all_associativity_variants_are_exhaustive() {
    // Ensure we can match all 3 variants
    let variants = [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ];
    for v in &variants {
        match v {
            Associativity::Left => {}
            Associativity::Right => {}
            Associativity::None => {}
        }
    }
    assert_eq!(variants.len(), 3);
}

#[test]
fn all_conflict_resolution_variants_are_exhaustive() {
    let variants = [
        ConflictResolution::Precedence(PrecedenceKind::Static(0)),
        ConflictResolution::Associativity(Associativity::Left),
        ConflictResolution::GLR,
    ];
    for v in &variants {
        match v {
            ConflictResolution::Precedence(_) => {}
            ConflictResolution::Associativity(_) => {}
            ConflictResolution::GLR => {}
        }
    }
    assert_eq!(variants.len(), 3);
}

#[test]
fn precedence_range_sweep_static() {
    // Verify a sweep of values all roundtrip correctly
    for val in (-10i16..=10).step_by(1) {
        let pk = PrecedenceKind::Static(val);
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pk, "Failed roundtrip for Static({val})");
    }
}

#[test]
fn precedence_range_sweep_dynamic() {
    for val in (-10i16..=10).step_by(1) {
        let pk = PrecedenceKind::Dynamic(val);
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pk, "Failed roundtrip for Dynamic({val})");
    }
}

#[test]
fn rule_precedence_none_vs_some_differ() {
    let r_none = make_rule(None, None);
    let r_some = make_rule(Some(PrecedenceKind::Static(0)), None);
    assert_ne!(r_none, r_some);
}

#[test]
fn javascript_like_grammar_has_precedence_rules() {
    let grammar = GrammarBuilder::javascript_like();

    // Find expression rules with precedence
    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expression")
        .map(|(id, _)| *id)
        .unwrap();

    let rules = &grammar.rules[&expr_id];
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    // + - * / all have precedence
    assert!(prec_rules.len() >= 4);

    // All should be left-associative
    for r in &prec_rules {
        assert_eq!(r.associativity, Some(Associativity::Left));
    }
}

#[test]
fn javascript_like_mul_higher_than_add() {
    let grammar = GrammarBuilder::javascript_like();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expression")
        .map(|(id, _)| *id)
        .unwrap();

    let rules = &grammar.rules[&expr_id];

    // Collect precedence levels
    let levels: Vec<i16> = rules
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => None,
        })
        .collect();

    let min_level = *levels.iter().min().unwrap();
    let max_level = *levels.iter().max().unwrap();
    assert!(
        max_level > min_level,
        "Expected different precedence levels, got min={min_level} max={max_level}"
    );
}

#[test]
fn grammar_serde_roundtrip_preserves_precedences() {
    let grammar = GrammarBuilder::new("prec_test")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();

    let json = serde_json::to_string(&grammar).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.precedences.len(), 2);
    assert_eq!(back.precedences[0].level, 1);
    assert_eq!(back.precedences[1].level, 2);
}

#[test]
fn grammar_serde_roundtrip_preserves_conflicts() {
    let mut grammar = Grammar::new("conflict_test".to_string());
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });

    let json = serde_json::to_string(&grammar).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.conflicts.len(), 1);
    assert_eq!(back.conflicts[0].resolution, ConflictResolution::GLR);
}

#[test]
fn rule_with_all_combinations_of_prec_and_assoc() {
    let precs: Vec<Option<PrecedenceKind>> = vec![
        None,
        Some(PrecedenceKind::Static(0)),
        Some(PrecedenceKind::Static(5)),
        Some(PrecedenceKind::Dynamic(-1)),
    ];
    let assocs: Vec<Option<Associativity>> = vec![
        None,
        Some(Associativity::Left),
        Some(Associativity::Right),
        Some(Associativity::None),
    ];

    let mut count = 0;
    for p in &precs {
        for a in &assocs {
            let r = make_rule(*p, *a);
            assert_eq!(r.precedence, *p);
            assert_eq!(r.associativity, *a);
            count += 1;
        }
    }
    assert_eq!(count, 16);
}

#[test]
fn builder_zero_precedence_level() {
    let grammar = GrammarBuilder::new("test")
        .token("+", "+")
        .precedence(0, Associativity::None, vec!["+"])
        .build();
    assert_eq!(grammar.precedences[0].level, 0);
}

#[test]
fn builder_negative_precedence_level() {
    let grammar = GrammarBuilder::new("test")
        .token("+", "+")
        .precedence(-5, Associativity::Left, vec!["+"])
        .build();
    assert_eq!(grammar.precedences[0].level, -5);
}
