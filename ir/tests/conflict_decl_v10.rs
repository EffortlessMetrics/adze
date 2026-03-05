//! Comprehensive tests for conflict declarations in adze-ir Grammar.
//!
//! Prefix: `cd_v10_` for unique grammar names.

use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, Precedence, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern, builder::GrammarBuilder,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal valid grammar with one token + one rule for the given symbol.
fn minimal_grammar(name: &str, sym: SymbolId) -> Grammar {
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

/// Build a grammar with two tokens and two rules.
fn two_symbol_grammar(name: &str, s0: SymbolId, s1: SymbolId) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    for (sym, tok_name, pat) in [(s0, "a", "a"), (s1, "b", "b")] {
        g.tokens.insert(
            sym,
            Token {
                name: tok_name.to_string(),
                pattern: TokenPattern::String(pat.to_string()),
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
    }
    g
}

fn glr_conflict(symbols: Vec<SymbolId>) -> ConflictDeclaration {
    ConflictDeclaration {
        symbols,
        resolution: ConflictResolution::GLR,
    }
}

fn prec_conflict(symbols: Vec<SymbolId>, level: i16) -> ConflictDeclaration {
    ConflictDeclaration {
        symbols,
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(level)),
    }
}

fn assoc_conflict(symbols: Vec<SymbolId>, assoc: Associativity) -> ConflictDeclaration {
    ConflictDeclaration {
        symbols,
        resolution: ConflictResolution::Associativity(assoc),
    }
}

// ===========================================================================
// 1. No conflicts → conflicts empty
// ===========================================================================

#[test]
fn cd_v10_no_conflicts_empty() {
    let g = Grammar::new("cd_v10_empty".to_string());
    assert!(g.conflicts.is_empty());
}

#[test]
fn cd_v10_default_grammar_no_conflicts() {
    let g = Grammar::default();
    assert!(g.conflicts.is_empty());
}

#[test]
fn cd_v10_builder_no_conflicts() {
    let g = GrammarBuilder::new("cd_v10_builder_empty")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert!(g.conflicts.is_empty());
}

// ===========================================================================
// 2. One conflict pair → conflicts has 1
// ===========================================================================

#[test]
fn cd_v10_one_conflict_pair() {
    let mut g = Grammar::new("cd_v10_one".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(1), SymbolId(2)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_one_conflict_symbols_match() {
    let mut g = Grammar::new("cd_v10_one_syms".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(10), SymbolId(20)]));
    assert_eq!(g.conflicts[0].symbols, vec![SymbolId(10), SymbolId(20)]);
}

#[test]
fn cd_v10_one_conflict_resolution_is_glr() {
    let mut g = Grammar::new("cd_v10_one_glr".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert_eq!(g.conflicts[0].resolution, ConflictResolution::GLR);
}

// ===========================================================================
// 3. Multiple conflict declarations → all present
// ===========================================================================

#[test]
fn cd_v10_multiple_conflicts_count() {
    let mut g = Grammar::new("cd_v10_multi".to_string());
    for i in 0..7 {
        g.conflicts
            .push(glr_conflict(vec![SymbolId(i), SymbolId(i + 1)]));
    }
    assert_eq!(g.conflicts.len(), 7);
}

#[test]
fn cd_v10_multiple_conflicts_all_distinct() {
    let mut g = Grammar::new("cd_v10_multi_dist".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    g.conflicts
        .push(prec_conflict(vec![SymbolId(2), SymbolId(3)], 5));
    g.conflicts.push(assoc_conflict(
        vec![SymbolId(4), SymbolId(5)],
        Associativity::Left,
    ));
    assert_eq!(g.conflicts.len(), 3);
    assert_ne!(g.conflicts[0], g.conflicts[1]);
    assert_ne!(g.conflicts[1], g.conflicts[2]);
}

#[test]
fn cd_v10_multiple_conflicts_order_preserved() {
    let mut g = Grammar::new("cd_v10_multi_ord".to_string());
    let decls: Vec<ConflictDeclaration> = (0..5)
        .map(|i| glr_conflict(vec![SymbolId(i), SymbolId(i + 10)]))
        .collect();
    for d in &decls {
        g.conflicts.push(d.clone());
    }
    for (i, d) in decls.iter().enumerate() {
        assert_eq!(&g.conflicts[i], d);
    }
}

// ===========================================================================
// 4. Conflict between two non-terminals
// ===========================================================================

#[test]
fn cd_v10_two_nonterminals_conflict() {
    let mut g = two_symbol_grammar("cd_v10_2nt", SymbolId(0), SymbolId(1));
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert_eq!(g.conflicts[0].symbols.len(), 2);
    assert!(g.conflicts[0].symbols.contains(&SymbolId(0)));
    assert!(g.conflicts[0].symbols.contains(&SymbolId(1)));
}

#[test]
fn cd_v10_two_nonterminals_with_precedence_resolution() {
    let mut g = two_symbol_grammar("cd_v10_2nt_prec", SymbolId(0), SymbolId(1));
    g.conflicts
        .push(prec_conflict(vec![SymbolId(0), SymbolId(1)], 3));
    match &g.conflicts[0].resolution {
        ConflictResolution::Precedence(PrecedenceKind::Static(3)) => {}
        other => panic!("Expected Static(3), got {other:?}"),
    }
}

// ===========================================================================
// 5. Conflict between three non-terminals
// ===========================================================================

#[test]
fn cd_v10_three_nonterminals_conflict() {
    let mut g = Grammar::new("cd_v10_3nt".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1), SymbolId(2)]));
    assert_eq!(g.conflicts[0].symbols.len(), 3);
}

#[test]
fn cd_v10_three_nonterminals_all_ids_present() {
    let mut g = Grammar::new("cd_v10_3nt_ids".to_string());
    let ids = vec![SymbolId(10), SymbolId(20), SymbolId(30)];
    g.conflicts.push(glr_conflict(ids.clone()));
    assert_eq!(g.conflicts[0].symbols, ids);
}

#[test]
fn cd_v10_three_nonterminals_with_assoc() {
    let mut g = Grammar::new("cd_v10_3nt_assoc".to_string());
    g.conflicts.push(assoc_conflict(
        vec![SymbolId(0), SymbolId(1), SymbolId(2)],
        Associativity::Right,
    ));
    assert_eq!(
        g.conflicts[0].resolution,
        ConflictResolution::Associativity(Associativity::Right)
    );
}

// ===========================================================================
// 6. Clone preserves conflicts
// ===========================================================================

#[test]
fn cd_v10_clone_preserves_conflicts() {
    let mut g = Grammar::new("cd_v10_clone".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(1), SymbolId(2)]));
    g.conflicts
        .push(prec_conflict(vec![SymbolId(3), SymbolId(4)], 10));
    let cloned = g.clone();
    assert_eq!(cloned.conflicts.len(), 2);
    assert_eq!(cloned.conflicts, g.conflicts);
}

#[test]
fn cd_v10_clone_is_independent() {
    let mut g = Grammar::new("cd_v10_clone_ind".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(1)]));
    let mut cloned = g.clone();
    cloned
        .conflicts
        .push(glr_conflict(vec![SymbolId(2), SymbolId(3)]));
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(cloned.conflicts.len(), 2);
}

#[test]
fn cd_v10_clone_conflict_declaration_field_equality() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(7), SymbolId(8), SymbolId(9)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(5)),
    };
    let cloned = decl.clone();
    assert_eq!(decl.symbols, cloned.symbols);
    assert_eq!(decl.resolution, cloned.resolution);
}

// ===========================================================================
// 7. Debug includes conflict info
// ===========================================================================

#[test]
fn cd_v10_debug_grammar_contains_conflicts() {
    let mut g = Grammar::new("cd_v10_dbg".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(42)]));
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("conflicts"),
        "Debug should mention 'conflicts': {dbg}"
    );
}

#[test]
fn cd_v10_debug_conflict_contains_glr() {
    let decl = glr_conflict(vec![SymbolId(1)]);
    let dbg = format!("{decl:?}");
    assert!(dbg.contains("GLR"), "Debug should contain 'GLR': {dbg}");
}

#[test]
fn cd_v10_debug_conflict_contains_symbol_id() {
    let decl = glr_conflict(vec![SymbolId(99)]);
    let dbg = format!("{decl:?}");
    assert!(
        dbg.contains("99"),
        "Debug should contain symbol id '99': {dbg}"
    );
}

#[test]
fn cd_v10_debug_conflict_resolution_precedence() {
    let decl = prec_conflict(vec![SymbolId(0)], 7);
    let dbg = format!("{decl:?}");
    assert!(
        dbg.contains("Precedence"),
        "Debug should contain 'Precedence': {dbg}"
    );
    assert!(
        dbg.contains("Static"),
        "Debug should contain 'Static': {dbg}"
    );
}

// ===========================================================================
// 8. Serde roundtrip preserves conflicts
// ===========================================================================

#[test]
fn cd_v10_serde_roundtrip_single_conflict() {
    let mut g = Grammar::new("cd_v10_serde1".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(1), SymbolId(2)]));
    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.conflicts.len(), 1);
    assert_eq!(
        restored.conflicts[0].symbols,
        vec![SymbolId(1), SymbolId(2)]
    );
    assert_eq!(restored.conflicts[0].resolution, ConflictResolution::GLR);
}

#[test]
fn cd_v10_serde_roundtrip_multiple_conflicts() {
    let mut g = Grammar::new("cd_v10_serde_m".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts
        .push(prec_conflict(vec![SymbolId(1), SymbolId(2)], -3));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(3)], Associativity::None));
    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.conflicts.len(), 3);
    assert_eq!(restored.conflicts[0].resolution, ConflictResolution::GLR);
    assert_eq!(
        restored.conflicts[1].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(-3))
    );
    assert_eq!(
        restored.conflicts[2].resolution,
        ConflictResolution::Associativity(Associativity::None)
    );
}

#[test]
fn cd_v10_serde_roundtrip_empty_conflicts() {
    let g = Grammar::new("cd_v10_serde_e".to_string());
    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert!(restored.conflicts.is_empty());
}

#[test]
fn cd_v10_serde_roundtrip_conflict_declaration_standalone() {
    let original = ConflictDeclaration {
        symbols: vec![SymbolId(100), SymbolId(200)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(42)),
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: ConflictDeclaration = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ===========================================================================
// 9. Normalize preserves conflicts
// ===========================================================================

#[test]
fn cd_v10_normalize_preserves_conflicts() {
    let mut g = minimal_grammar("cd_v10_norm", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let _ = g.normalize();
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.conflicts[0].resolution, ConflictResolution::GLR);
}

#[test]
fn cd_v10_normalize_preserves_multiple_conflicts() {
    let mut g = minimal_grammar("cd_v10_norm_m", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(0)], 5));
    let _ = g.normalize();
    assert_eq!(g.conflicts.len(), 2);
}

#[test]
fn cd_v10_normalize_preserves_conflict_symbols() {
    let mut g = minimal_grammar("cd_v10_norm_s", SymbolId(0));
    let syms = vec![SymbolId(0), SymbolId(100)];
    g.conflicts.push(glr_conflict(syms.clone()));
    let _ = g.normalize();
    assert_eq!(g.conflicts[0].symbols, syms);
}

#[test]
fn cd_v10_normalize_preserves_conflict_resolution_type() {
    let mut g = minimal_grammar("cd_v10_norm_r", SymbolId(0));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(0)], Associativity::Left));
    let _ = g.normalize();
    assert_eq!(
        g.conflicts[0].resolution,
        ConflictResolution::Associativity(Associativity::Left)
    );
}

// ===========================================================================
// 10. Optimize preserves conflicts
// ===========================================================================

#[test]
fn cd_v10_optimize_preserves_conflicts() {
    let mut g = minimal_grammar("cd_v10_opt", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.optimize();
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_optimize_preserves_multiple_conflicts() {
    let mut g = minimal_grammar("cd_v10_opt_m", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(0)], 2));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(0)], Associativity::Right));
    g.optimize();
    assert_eq!(g.conflicts.len(), 3);
}

#[test]
fn cd_v10_optimize_preserves_conflict_data() {
    let mut g = minimal_grammar("cd_v10_opt_d", SymbolId(0));
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(0), SymbolId(50)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(9)),
    };
    g.conflicts.push(decl.clone());
    g.optimize();
    assert_eq!(g.conflicts[0], decl);
}

// ===========================================================================
// 11. Validate with conflicts → Ok
// ===========================================================================

#[test]
fn cd_v10_validate_ok_with_glr_conflict() {
    let mut g = minimal_grammar("cd_v10_val", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert!(g.validate().is_ok());
}

#[test]
fn cd_v10_validate_ok_with_prec_conflict() {
    let mut g = minimal_grammar("cd_v10_val_p", SymbolId(0));
    g.conflicts.push(prec_conflict(vec![SymbolId(0)], 1));
    assert!(g.validate().is_ok());
}

#[test]
fn cd_v10_validate_ok_with_assoc_conflict() {
    let mut g = minimal_grammar("cd_v10_val_a", SymbolId(0));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(0)], Associativity::Left));
    assert!(g.validate().is_ok());
}

#[test]
fn cd_v10_validate_ok_with_many_conflicts() {
    let mut g = minimal_grammar("cd_v10_val_many", SymbolId(0));
    for i in 0..10 {
        g.conflicts
            .push(glr_conflict(vec![SymbolId(0), SymbolId(i)]));
    }
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 12. Conflict with same name/symbol twice → behavior
// ===========================================================================

#[test]
fn cd_v10_duplicate_symbol_in_single_conflict() {
    let mut g = Grammar::new("cd_v10_dup".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(5), SymbolId(5)]));
    assert_eq!(g.conflicts[0].symbols.len(), 2);
    assert!(g.conflicts[0].symbols.iter().all(|s| *s == SymbolId(5)));
}

#[test]
fn cd_v10_same_conflict_added_twice() {
    let mut g = Grammar::new("cd_v10_dup2".to_string());
    let decl = glr_conflict(vec![SymbolId(1), SymbolId(2)]);
    g.conflicts.push(decl.clone());
    g.conflicts.push(decl);
    assert_eq!(g.conflicts.len(), 2);
    assert_eq!(g.conflicts[0], g.conflicts[1]);
}

#[test]
fn cd_v10_triple_duplicate_symbol() {
    let decl = glr_conflict(vec![SymbolId(3), SymbolId(3), SymbolId(3)]);
    assert_eq!(decl.symbols.len(), 3);
}

// ===========================================================================
// 13. Grammar with conflicts + precedence
// ===========================================================================

#[test]
fn cd_v10_conflicts_and_precedence_coexist() {
    let mut g = Grammar::new("cd_v10_cp".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    });
    g.conflicts
        .push(glr_conflict(vec![SymbolId(10), SymbolId(11)]));
    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_multiple_precedences_and_conflicts() {
    let mut g = Grammar::new("cd_v10_mpc".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(0)],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    });
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(2)], 3));
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.conflicts.len(), 2);
}

#[test]
fn cd_v10_precedence_symbol_in_conflict() {
    let mut g = Grammar::new("cd_v10_ps_in_c".to_string());
    let sym = SymbolId(42);
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::None,
        symbols: vec![sym],
    });
    g.conflicts.push(glr_conflict(vec![sym]));
    assert!(g.conflicts[0].symbols.contains(&sym));
    assert!(g.precedences[0].symbols.contains(&sym));
}

// ===========================================================================
// 14. Grammar with conflicts + inline
// ===========================================================================

#[test]
fn cd_v10_conflicts_and_inline_coexist() {
    let mut g = minimal_grammar("cd_v10_ci", SymbolId(0));
    g.inline_rules.push(SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_inline_symbol_in_conflict() {
    let mut g = Grammar::new("cd_v10_isc".to_string());
    let sym = SymbolId(7);
    g.inline_rules.push(sym);
    g.conflicts.push(glr_conflict(vec![sym, SymbolId(8)]));
    assert!(g.inline_rules.contains(&sym));
    assert!(g.conflicts[0].symbols.contains(&sym));
}

#[test]
fn cd_v10_builder_inline_then_add_conflict() {
    let mut g = GrammarBuilder::new("cd_v10_bi")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .inline("s")
        .start("s")
        .build();
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert!(!g.inline_rules.is_empty());
    assert_eq!(g.conflicts.len(), 1);
}

// ===========================================================================
// 15. Conflicts are deterministic
// ===========================================================================

#[test]
fn cd_v10_deterministic_insertion_order() {
    let build = || {
        let mut g = Grammar::new("cd_v10_det".to_string());
        g.conflicts.push(glr_conflict(vec![SymbolId(1)]));
        g.conflicts.push(prec_conflict(vec![SymbolId(2)], 5));
        g.conflicts
            .push(assoc_conflict(vec![SymbolId(3)], Associativity::Left));
        g
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(g1.conflicts, g2.conflicts);
}

#[test]
fn cd_v10_deterministic_serde() {
    let mut g = Grammar::new("cd_v10_det_s".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    let json1 = serde_json::to_string(&g).unwrap();
    let json2 = serde_json::to_string(&g).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn cd_v10_deterministic_debug() {
    let decl = glr_conflict(vec![SymbolId(5), SymbolId(6)]);
    let dbg1 = format!("{decl:?}");
    let dbg2 = format!("{decl:?}");
    assert_eq!(dbg1, dbg2);
}

// ===========================================================================
// 16. Different conflict sets → different grammars
// ===========================================================================

#[test]
fn cd_v10_different_conflicts_not_equal() {
    let mut g1 = Grammar::new("cd_v10_ne1".to_string());
    g1.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let mut g2 = Grammar::new("cd_v10_ne1".to_string());
    g2.conflicts.push(glr_conflict(vec![SymbolId(1)]));
    assert_ne!(g1, g2);
}

#[test]
fn cd_v10_conflict_vs_no_conflict_not_equal() {
    let g1 = Grammar::new("cd_v10_ne2".to_string());
    let mut g2 = Grammar::new("cd_v10_ne2".to_string());
    g2.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_ne!(g1, g2);
}

#[test]
fn cd_v10_different_resolution_not_equal() {
    let mut g1 = Grammar::new("cd_v10_ne3".to_string());
    g1.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let mut g2 = Grammar::new("cd_v10_ne3".to_string());
    g2.conflicts.push(prec_conflict(vec![SymbolId(0)], 1));
    assert_ne!(g1, g2);
}

#[test]
fn cd_v10_same_conflicts_equal() {
    let build = || {
        let mut g = Grammar::new("cd_v10_eq".to_string());
        g.conflicts
            .push(glr_conflict(vec![SymbolId(5), SymbolId(6)]));
        g
    };
    assert_eq!(build(), build());
}

// ===========================================================================
// 17. Conflict count matches declarations
// ===========================================================================

#[test]
fn cd_v10_count_zero() {
    let g = Grammar::new("cd_v10_c0".to_string());
    assert_eq!(g.conflicts.len(), 0);
}

#[test]
fn cd_v10_count_one() {
    let mut g = Grammar::new("cd_v10_c1".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_count_ten() {
    let mut g = Grammar::new("cd_v10_c10".to_string());
    for i in 0..10 {
        g.conflicts.push(glr_conflict(vec![SymbolId(i)]));
    }
    assert_eq!(g.conflicts.len(), 10);
}

#[test]
fn cd_v10_count_after_extend() {
    let mut g = Grammar::new("cd_v10_cext".to_string());
    let decls: Vec<ConflictDeclaration> = (0..4).map(|i| glr_conflict(vec![SymbolId(i)])).collect();
    g.conflicts.extend(decls);
    assert_eq!(g.conflicts.len(), 4);
}

// ===========================================================================
// 18. Conflict names reference valid symbols
// ===========================================================================

#[test]
fn cd_v10_conflict_references_token_symbol() {
    let mut g = minimal_grammar("cd_v10_ref_tok", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert!(g.tokens.contains_key(&g.conflicts[0].symbols[0]));
}

#[test]
fn cd_v10_conflict_references_rule_lhs() {
    let mut g = minimal_grammar("cd_v10_ref_rule", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert!(g.rules.contains_key(&g.conflicts[0].symbols[0]));
}

#[test]
fn cd_v10_conflict_with_nonexistent_symbol_still_stores() {
    let mut g = Grammar::new("cd_v10_ref_ne".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(9999)]));
    assert_eq!(g.conflicts[0].symbols[0], SymbolId(9999));
}

#[test]
fn cd_v10_conflict_symbols_subset_of_grammar_tokens() {
    let mut g = two_symbol_grammar("cd_v10_ref_sub", SymbolId(0), SymbolId(1));
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    for sym in &g.conflicts[0].symbols {
        assert!(g.tokens.contains_key(sym));
    }
}

// ===========================================================================
// 19. Grammar with 5 conflict groups
// ===========================================================================

#[test]
fn cd_v10_five_conflict_groups() {
    let mut g = Grammar::new("cd_v10_5grp".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    g.conflicts
        .push(prec_conflict(vec![SymbolId(2), SymbolId(3)], 1));
    g.conflicts.push(assoc_conflict(
        vec![SymbolId(4), SymbolId(5)],
        Associativity::Left,
    ));
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(6), SymbolId(7)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(2)),
    });
    g.conflicts.push(assoc_conflict(
        vec![SymbolId(8), SymbolId(9)],
        Associativity::Right,
    ));
    assert_eq!(g.conflicts.len(), 5);
}

#[test]
fn cd_v10_five_groups_all_resolutions_different() {
    let mut g = Grammar::new("cd_v10_5grp_diff".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(1)], 1));
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(1)),
    });
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(3)], Associativity::Left));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(4)], Associativity::Right));

    // Verify all five are distinct
    for i in 0..5 {
        for j in (i + 1)..5 {
            assert_ne!(
                g.conflicts[i], g.conflicts[j],
                "conflicts[{i}] and conflicts[{j}] should differ"
            );
        }
    }
}

#[test]
fn cd_v10_five_groups_serde_roundtrip() {
    let mut g = Grammar::new("cd_v10_5grp_serde".to_string());
    for i in 0..5 {
        g.conflicts
            .push(glr_conflict(vec![SymbolId(i), SymbolId(i + 5)]));
    }
    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.conflicts.len(), 5);
    for i in 0..5 {
        assert_eq!(restored.conflicts[i], g.conflicts[i]);
    }
}

// ===========================================================================
// 20. Conflict interaction with alternatives (Choice symbol)
// ===========================================================================

#[test]
fn cd_v10_conflict_with_choice_rule() {
    let mut g = two_symbol_grammar("cd_v10_choice", SymbolId(0), SymbolId(1));
    // Add a rule that uses Choice
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_with_optional_symbol() {
    let mut g = minimal_grammar("cd_v10_opt_sym", SymbolId(0));
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(0))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_with_repeat_symbol() {
    let mut g = minimal_grammar("cd_v10_rep_sym", SymbolId(0));
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(0))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.conflicts.len(), 1);
}

// ===========================================================================
// Additional edge cases and coverage
// ===========================================================================

#[test]
fn cd_v10_conflict_declaration_equality() {
    let a = glr_conflict(vec![SymbolId(1), SymbolId(2)]);
    let b = glr_conflict(vec![SymbolId(1), SymbolId(2)]);
    assert_eq!(a, b);
}

#[test]
fn cd_v10_conflict_declaration_inequality_symbols() {
    let a = glr_conflict(vec![SymbolId(1), SymbolId(2)]);
    let b = glr_conflict(vec![SymbolId(1), SymbolId(3)]);
    assert_ne!(a, b);
}

#[test]
fn cd_v10_conflict_declaration_inequality_resolution() {
    let a = glr_conflict(vec![SymbolId(1)]);
    let b = prec_conflict(vec![SymbolId(1)], 0);
    assert_ne!(a, b);
}

#[test]
fn cd_v10_conflict_declaration_inequality_symbol_order() {
    let a = glr_conflict(vec![SymbolId(1), SymbolId(2)]);
    let b = glr_conflict(vec![SymbolId(2), SymbolId(1)]);
    assert_ne!(a, b);
}

#[test]
fn cd_v10_empty_symbol_list() {
    let decl = glr_conflict(vec![]);
    assert!(decl.symbols.is_empty());
}

#[test]
fn cd_v10_single_symbol_conflict() {
    let decl = glr_conflict(vec![SymbolId(0)]);
    assert_eq!(decl.symbols.len(), 1);
}

#[test]
fn cd_v10_large_symbol_ids_in_conflict() {
    let decl = glr_conflict(vec![SymbolId(u16::MAX), SymbolId(u16::MAX - 1)]);
    assert_eq!(decl.symbols[0], SymbolId(u16::MAX));
    assert_eq!(decl.symbols[1], SymbolId(u16::MAX - 1));
}

#[test]
fn cd_v10_many_symbols_in_one_conflict() {
    let ids: Vec<SymbolId> = (0..50).map(SymbolId).collect();
    let decl = glr_conflict(ids.clone());
    assert_eq!(decl.symbols.len(), 50);
    assert_eq!(decl.symbols, ids);
}

#[test]
fn cd_v10_normalize_then_validate_with_conflicts() {
    let mut g = minimal_grammar("cd_v10_nv", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let _ = g.normalize();
    assert!(g.validate().is_ok());
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_optimize_then_validate_with_conflicts() {
    let mut g = minimal_grammar("cd_v10_ov", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.optimize();
    assert!(g.validate().is_ok());
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_normalize_optimize_validate_chain() {
    let mut g = minimal_grammar("cd_v10_nov", SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(0)], 1));
    let _ = g.normalize();
    g.optimize();
    assert!(g.validate().is_ok());
    assert_eq!(g.conflicts.len(), 2);
}

#[test]
fn cd_v10_builder_grammar_add_conflicts_validate() {
    let mut g = GrammarBuilder::new("cd_v10_bgv")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert!(g.validate().is_ok());
}

#[test]
fn cd_v10_conflict_with_sequence_rule() {
    let mut g = two_symbol_grammar("cd_v10_seq", SymbolId(0), SymbolId(1));
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_with_epsilon_rule() {
    let mut g = minimal_grammar("cd_v10_eps", SymbolId(0));
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_with_extras() {
    let mut g = minimal_grammar("cd_v10_ext", SymbolId(0));
    g.extras.push(SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert!(!g.extras.is_empty());
    assert!(!g.conflicts.is_empty());
}

#[test]
fn cd_v10_conflict_with_externals() {
    let mut g = Grammar::new("cd_v10_extern".to_string());
    g.externals.push(adze_ir::ExternalToken {
        name: "ext".to_string(),
        symbol_id: SymbolId(10),
    });
    g.conflicts.push(glr_conflict(vec![SymbolId(10)]));
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_with_supertypes() {
    let mut g = Grammar::new("cd_v10_super".to_string());
    g.supertypes.push(SymbolId(0));
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert!(!g.supertypes.is_empty());
    assert!(!g.conflicts.is_empty());
}

#[test]
fn cd_v10_conflict_resolution_dynamic_precedence() {
    let decl = ConflictDeclaration {
        symbols: vec![SymbolId(0), SymbolId(1)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(99)),
    };
    match &decl.resolution {
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(99)) => {}
        other => panic!("Expected Dynamic(99), got {other:?}"),
    }
}

#[test]
fn cd_v10_conflict_assoc_none() {
    let decl = assoc_conflict(vec![SymbolId(0)], Associativity::None);
    assert_eq!(
        decl.resolution,
        ConflictResolution::Associativity(Associativity::None)
    );
}

#[test]
fn cd_v10_grammar_partial_eq_with_conflicts() {
    let mut g1 = Grammar::new("cd_v10_peq".to_string());
    g1.conflicts.push(glr_conflict(vec![SymbolId(1)]));
    let g2 = g1.clone();
    assert_eq!(g1, g2);
}

#[test]
fn cd_v10_grammar_default_conflicts_empty() {
    let g: Grammar = Default::default();
    assert!(g.conflicts.is_empty());
}

#[test]
fn cd_v10_serde_json_contains_conflicts_key() {
    let mut g = Grammar::new("cd_v10_json_key".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let json = serde_json::to_string(&g).unwrap();
    assert!(
        json.contains("\"conflicts\""),
        "JSON should contain 'conflicts' key"
    );
}

#[test]
fn cd_v10_serde_json_contains_glr() {
    let mut g = Grammar::new("cd_v10_json_glr".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("GLR"), "JSON should contain 'GLR'");
}

#[test]
fn cd_v10_serde_pretty_roundtrip() {
    let mut g = Grammar::new("cd_v10_pretty".to_string());
    g.conflicts
        .push(glr_conflict(vec![SymbolId(5), SymbolId(6)]));
    let json = serde_json::to_string_pretty(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.conflicts, restored.conflicts);
}

#[test]
fn cd_v10_conflict_vec_iter() {
    let mut g = Grammar::new("cd_v10_iter".to_string());
    for i in 0..3 {
        g.conflicts.push(glr_conflict(vec![SymbolId(i)]));
    }
    let collected: Vec<_> = g.conflicts.iter().collect();
    assert_eq!(collected.len(), 3);
}

#[test]
fn cd_v10_conflict_vec_retain() {
    let mut g = Grammar::new("cd_v10_retain".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(1)], 1));
    g.conflicts.push(glr_conflict(vec![SymbolId(2)]));
    g.conflicts
        .retain(|c| c.resolution == ConflictResolution::GLR);
    assert_eq!(g.conflicts.len(), 2);
}

#[test]
fn cd_v10_builder_precedence_then_add_conflict() {
    let mut g = GrammarBuilder::new("cd_v10_bpc")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .precedence(1, Associativity::Left, vec!["A"])
        .start("s")
        .build();
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert!(!g.precedences.is_empty());
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_symbol_id_zero() {
    let decl = glr_conflict(vec![SymbolId(0)]);
    assert_eq!(decl.symbols[0].0, 0);
}

#[test]
fn cd_v10_conflict_many_groups_count() {
    let mut g = Grammar::new("cd_v10_many_grp".to_string());
    for i in 0..20 {
        g.conflicts
            .push(glr_conflict(vec![SymbolId(i * 2), SymbolId(i * 2 + 1)]));
    }
    assert_eq!(g.conflicts.len(), 20);
}

#[test]
fn cd_v10_conflict_mixed_resolution_types() {
    let mut g = Grammar::new("cd_v10_mixed".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(1)], 10));
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(-5)),
    });
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(3)], Associativity::Left));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(4)], Associativity::Right));
    g.conflicts
        .push(assoc_conflict(vec![SymbolId(5)], Associativity::None));
    assert_eq!(g.conflicts.len(), 6);
}

#[test]
fn cd_v10_conflict_with_repeat_one_symbol() {
    let mut g = minimal_grammar("cd_v10_rep1", SymbolId(0));
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(0))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn cd_v10_conflict_with_nonterminal_symbol_in_rule() {
    let mut g = minimal_grammar("cd_v10_nt_rule", SymbolId(0));
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::NonTerminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.conflicts
        .push(glr_conflict(vec![SymbolId(0), SymbolId(1)]));
    assert_eq!(g.conflicts[0].symbols.len(), 2);
}

#[test]
fn cd_v10_conflict_cleared_then_readded() {
    let mut g = Grammar::new("cd_v10_clear".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    assert_eq!(g.conflicts.len(), 1);
    g.conflicts.clear();
    assert!(g.conflicts.is_empty());
    g.conflicts.push(prec_conflict(vec![SymbolId(1)], 3));
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(
        g.conflicts[0].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(3))
    );
}

#[test]
fn cd_v10_conflict_index_access() {
    let mut g = Grammar::new("cd_v10_idx".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(10)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(20)], 2));
    assert_eq!(g.conflicts[0].symbols[0], SymbolId(10));
    assert_eq!(g.conflicts[1].symbols[0], SymbolId(20));
}

#[test]
fn cd_v10_conflict_last() {
    let mut g = Grammar::new("cd_v10_last".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(1)], 7));
    let last = g.conflicts.last().unwrap();
    assert_eq!(last.symbols[0], SymbolId(1));
    assert_eq!(
        last.resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(7))
    );
}

#[test]
fn cd_v10_conflict_first() {
    let mut g = Grammar::new("cd_v10_first".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(99)]));
    g.conflicts.push(prec_conflict(vec![SymbolId(100)], 1));
    let first = g.conflicts.first().unwrap();
    assert_eq!(first.symbols[0], SymbolId(99));
    assert_eq!(first.resolution, ConflictResolution::GLR);
}

#[test]
fn cd_v10_conflict_contains_check() {
    let mut g = Grammar::new("cd_v10_contains".to_string());
    let target = glr_conflict(vec![SymbolId(42)]);
    g.conflicts.push(target.clone());
    g.conflicts.push(prec_conflict(vec![SymbolId(43)], 1));
    assert!(g.conflicts.contains(&target));
}

#[test]
fn cd_v10_conflict_not_contains_check() {
    let mut g = Grammar::new("cd_v10_ncontains".to_string());
    g.conflicts.push(glr_conflict(vec![SymbolId(0)]));
    let missing = glr_conflict(vec![SymbolId(999)]);
    assert!(!g.conflicts.contains(&missing));
}
