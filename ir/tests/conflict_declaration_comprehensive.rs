#![allow(clippy::needless_range_loop)]

use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, Precedence, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// ConflictResolution variant construction & equality
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_precedence_static() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Static(3));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(3))
    );
}

#[test]
fn conflict_resolution_precedence_dynamic() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Dynamic(-2));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(-2))
    );
}

#[test]
fn conflict_resolution_associativity_left() {
    let res = ConflictResolution::Associativity(Associativity::Left);
    assert_eq!(res, ConflictResolution::Associativity(Associativity::Left));
}

#[test]
fn conflict_resolution_associativity_right() {
    let res = ConflictResolution::Associativity(Associativity::Right);
    assert_eq!(res, ConflictResolution::Associativity(Associativity::Right));
}

#[test]
fn conflict_resolution_associativity_none() {
    let res = ConflictResolution::Associativity(Associativity::None);
    assert_eq!(res, ConflictResolution::Associativity(Associativity::None));
}

#[test]
fn conflict_resolution_glr() {
    let res = ConflictResolution::GLR;
    assert_eq!(res, ConflictResolution::GLR);
}

#[test]
fn conflict_resolution_variants_not_equal() {
    let prec = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    let assoc = ConflictResolution::Associativity(Associativity::Left);
    let glr = ConflictResolution::GLR;

    assert_ne!(prec, assoc);
    assert_ne!(prec, glr);
    assert_ne!(assoc, glr);
}

#[test]
fn conflict_resolution_different_precedence_levels_not_equal() {
    let a = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    let b = ConflictResolution::Precedence(PrecedenceKind::Static(2));
    assert_ne!(a, b);
}

#[test]
fn conflict_resolution_static_vs_dynamic_not_equal() {
    let s = ConflictResolution::Precedence(PrecedenceKind::Static(5));
    let d = ConflictResolution::Precedence(PrecedenceKind::Dynamic(5));
    assert_ne!(s, d);
}

// ---------------------------------------------------------------------------
// ConflictResolution clone & debug
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_clone_preserves_equality() {
    let original = ConflictResolution::Precedence(PrecedenceKind::Dynamic(7));
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn conflict_resolution_debug_contains_variant_name() {
    let glr = ConflictResolution::GLR;
    let dbg = format!("{:?}", glr);
    assert!(
        dbg.contains("GLR"),
        "Debug output should contain 'GLR': {dbg}"
    );
}

// ---------------------------------------------------------------------------
// ConflictResolution serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_serde_roundtrip_glr() {
    let original = ConflictResolution::GLR;
    let json = serde_json::to_string(&original).unwrap();
    let restored: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn conflict_resolution_serde_roundtrip_precedence() {
    let original = ConflictResolution::Precedence(PrecedenceKind::Static(-10));
    let json = serde_json::to_string(&original).unwrap();
    let restored: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn conflict_resolution_serde_roundtrip_associativity() {
    let original = ConflictResolution::Associativity(Associativity::Right);
    let json = serde_json::to_string(&original).unwrap();
    let restored: ConflictResolution = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ---------------------------------------------------------------------------
// ConflictDeclaration construction
// ---------------------------------------------------------------------------

#[test]
fn conflict_declaration_empty_symbols() {
    let decl = ConflictDeclaration {
        symbols: vec![],
        resolution: ConflictResolution::GLR,
    };
    assert!(decl.symbols.is_empty());
}

#[test]
fn conflict_declaration_single_symbol() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(5)],
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(decl.symbols.len(), 1);
    assert_eq!(decl.symbols[0], SymbolId(5));
}

#[test]
fn conflict_declaration_two_symbols() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    };
    assert_eq!(decl.symbols.len(), 2);
    assert!(decl.symbols.contains(&SymbolId(1)));
    assert!(decl.symbols.contains(&SymbolId(2)));
}

#[test]
fn conflict_declaration_many_symbols() {
    let ids: Vec<SymbolId> = (0..10).map(SymbolId).collect();
    let decl = ConflictDeclaration {
        symbols: ids.clone(),
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(decl.symbols.len(), 10);
    for i in 0..10 {
        assert_eq!(decl.symbols[i], SymbolId(i as u16));
    }
}

#[test]
fn conflict_declaration_with_precedence_resolution() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(3), SymbolId(4)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(10)),
    };
    match &decl.resolution {
        ConflictResolution::Precedence(PrecedenceKind::Static(10)) => {}
        other => panic!("Expected Static(10), got {other:?}"),
    }
}

#[test]
fn conflict_declaration_with_dynamic_precedence() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(0), SymbolId(1)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(3)),
    };
    match &decl.resolution {
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(3)) => {}
        other => panic!("Expected Dynamic(3), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// ConflictDeclaration clone & debug
// ---------------------------------------------------------------------------

#[test]
fn conflict_declaration_clone() {
    let original = ConflictDeclaration {
        symbols: vec![SymbolId(7), SymbolId(8)],
        resolution: ConflictResolution::GLR,
    };
    let cloned = original.clone();
    assert_eq!(cloned.symbols, original.symbols);
    assert_eq!(cloned.resolution, original.resolution);
}

#[test]
fn conflict_declaration_debug_output() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::GLR,
    };
    let dbg = format!("{:?}", decl);
    assert!(dbg.contains("ConflictDeclaration"), "{dbg}");
    assert!(dbg.contains("GLR"), "{dbg}");
}

// ---------------------------------------------------------------------------
// ConflictDeclaration serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn conflict_declaration_serde_roundtrip() {
    let original = ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(20), SymbolId(30)],
        resolution: ConflictResolution::Associativity(Associativity::None),
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: ConflictDeclaration = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.symbols.len(), 3);
    assert_eq!(restored.resolution, original.resolution);
    for i in 0..3 {
        assert_eq!(restored.symbols[i], original.symbols[i]);
    }
}

// ---------------------------------------------------------------------------
// Grammar integration: adding conflicts
// ---------------------------------------------------------------------------

#[test]
fn grammar_starts_with_no_conflicts() {
    let grammar = Grammar::new("empty".to_string());
    assert!(grammar.conflicts.is_empty());
}

#[test]
fn grammar_add_single_conflict() {
    let mut grammar = Grammar::new("g".to_string());
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    assert_eq!(grammar.conflicts.len(), 1);
}

#[test]
fn grammar_add_multiple_conflicts() {
    let mut grammar = Grammar::new("g".to_string());
    for i in 0..5 {
        grammar.conflicts.push(ConflictDeclaration {
            symbols: vec![SymbolId(i), SymbolId(i + 1)],
            resolution: ConflictResolution::GLR,
        });
    }
    assert_eq!(grammar.conflicts.len(), 5);
}

#[test]
fn grammar_conflicts_with_mixed_resolutions() {
    let mut grammar = Grammar::new("mixed".to_string());
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3), SymbolId(4)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(5)),
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(5), SymbolId(6)],
        resolution: ConflictResolution::Associativity(Associativity::Right),
    });

    assert_eq!(grammar.conflicts.len(), 3);
    assert_eq!(grammar.conflicts[0].resolution, ConflictResolution::GLR);
    assert_eq!(
        grammar.conflicts[1].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
    assert_eq!(
        grammar.conflicts[2].resolution,
        ConflictResolution::Associativity(Associativity::Right)
    );
}

// ---------------------------------------------------------------------------
// Grammar with conflicts survives validation
// ---------------------------------------------------------------------------

/// Helper: build a minimal valid grammar with a token for the given id.
fn grammar_with_token(name: &str, sym: SymbolId) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    g.tokens.insert(
        sym,
        Token {
            name: "tok".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: sym,
        rhs: vec![Symbol::Terminal(sym)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

#[test]
fn grammar_with_glr_conflict_validates() {
    let mut g = grammar_with_token("v", SymbolId(0));
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(0)],
        resolution: ConflictResolution::GLR,
    });
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_with_precedence_conflict_validates() {
    let mut g = grammar_with_token("v", SymbolId(0));
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(0)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Grammar serde round-trip preserves conflicts
// ---------------------------------------------------------------------------

#[test]
fn grammar_serde_roundtrip_preserves_conflicts() {
    let mut g = Grammar::new("serde_test".to_string());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(4)),
    });

    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.conflicts.len(), 2);
    assert_eq!(
        restored.conflicts[0].symbols,
        vec![SymbolId(1), SymbolId(2)]
    );
    assert_eq!(restored.conflicts[0].resolution, ConflictResolution::GLR);
    assert_eq!(restored.conflicts[1].symbols, vec![SymbolId(3)]);
    assert_eq!(
        restored.conflicts[1].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(4))
    );
}

// ---------------------------------------------------------------------------
// Conflict + Precedence interaction
// ---------------------------------------------------------------------------

#[test]
fn conflict_and_precedence_coexist_in_grammar() {
    let mut g = Grammar::new("prec_conflict".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(10), SymbolId(11)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });

    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.precedences[0].level, 1);
}

// ---------------------------------------------------------------------------
// Negative & boundary precedence values
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_negative_precedence() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Static(-100));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(-100))
    );
}

#[test]
fn conflict_resolution_zero_precedence() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Static(0));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(0))
    );
}

#[test]
fn conflict_resolution_max_precedence() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Static(i16::MAX));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(i16::MAX))
    );
}

#[test]
fn conflict_resolution_min_precedence() {
    let res = ConflictResolution::Precedence(PrecedenceKind::Static(i16::MIN));
    assert_eq!(
        res,
        ConflictResolution::Precedence(PrecedenceKind::Static(i16::MIN))
    );
}

// ---------------------------------------------------------------------------
// Duplicate symbols in a declaration
// ---------------------------------------------------------------------------

#[test]
fn conflict_declaration_allows_duplicate_symbols() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(5), SymbolId(5), SymbolId(5)],
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(decl.symbols.len(), 3);
    assert!(decl.symbols.iter().all(|s| *s == SymbolId(5)));
}

// ---------------------------------------------------------------------------
// Conflict declarations survive normalize
// ---------------------------------------------------------------------------

#[test]
fn conflicts_preserved_after_normalize() {
    let mut g = Grammar::new("norm".to_string());
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(0))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(0)],
        resolution: ConflictResolution::GLR,
    });

    let _ = g.normalize();

    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.conflicts[0].resolution, ConflictResolution::GLR);
}

// ---------------------------------------------------------------------------
// Large symbol id values
// ---------------------------------------------------------------------------

#[test]
fn conflict_declaration_with_large_symbol_ids() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(u16::MAX), SymbolId(u16::MAX - 1)],
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(decl.symbols[0], SymbolId(u16::MAX));
    assert_eq!(decl.symbols[1], SymbolId(u16::MAX - 1));
}

// ---------------------------------------------------------------------------
// Pattern matching exhaustiveness
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_match_all_variants() {
    let variants = [
        ConflictResolution::GLR,
        ConflictResolution::Precedence(PrecedenceKind::Static(0)),
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(0)),
        ConflictResolution::Associativity(Associativity::Left),
        ConflictResolution::Associativity(Associativity::Right),
        ConflictResolution::Associativity(Associativity::None),
    ];

    for variant in &variants {
        match variant {
            ConflictResolution::GLR => {}
            ConflictResolution::Precedence(PrecedenceKind::Static(_)) => {}
            ConflictResolution::Precedence(PrecedenceKind::Dynamic(_)) => {}
            ConflictResolution::Associativity(Associativity::Left) => {}
            ConflictResolution::Associativity(Associativity::Right) => {}
            ConflictResolution::Associativity(Associativity::None) => {}
        }
    }
    assert_eq!(variants.len(), 6);
}
