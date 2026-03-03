//! Comprehensive tests for ExternalToken, extras, supertypes, conflicts,
//! inline_rules, and their interactions with Grammar methods.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_rule(lhs: SymbolId, rhs: Vec<Symbol>) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

fn simple_grammar_with_token(id: u16, name: &str) -> Grammar {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(id),
        Token {
            name: name.into(),
            pattern: TokenPattern::String(name.into()),
            fragile: false,
        },
    );
    g
}

// ===========================================================================
// 1. ExternalToken construction, fields, Debug, Clone, serde
// ===========================================================================

#[test]
fn external_token_construction() {
    let et = ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(42),
    };
    assert_eq!(et.name, "indent");
    assert_eq!(et.symbol_id, SymbolId(42));
}

#[test]
fn external_token_clone() {
    let et = ExternalToken {
        name: "dedent".into(),
        symbol_id: SymbolId(7),
    };
    let cloned = et.clone();
    assert_eq!(cloned.name, et.name);
    assert_eq!(cloned.symbol_id, et.symbol_id);
}

#[test]
fn external_token_debug() {
    let et = ExternalToken {
        name: "newline".into(),
        symbol_id: SymbolId(1),
    };
    let dbg = format!("{et:?}");
    assert!(dbg.contains("newline"));
    assert!(dbg.contains("ExternalToken"));
}

#[test]
fn external_token_serde_roundtrip() {
    let et = ExternalToken {
        name: "string_content".into(),
        symbol_id: SymbolId(99),
    };
    let json = serde_json::to_string(&et).unwrap();
    let deserialized: ExternalToken = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "string_content");
    assert_eq!(deserialized.symbol_id, SymbolId(99));
}

#[test]
fn external_token_serde_json_shape() {
    let et = ExternalToken {
        name: "heredoc".into(),
        symbol_id: SymbolId(5),
    };
    let val: serde_json::Value = serde_json::to_value(&et).unwrap();
    assert_eq!(val["name"], "heredoc");
    assert_eq!(val["symbol_id"], 5);
}

// ===========================================================================
// 2. Grammar.externals manipulation
// ===========================================================================

#[test]
fn grammar_externals_starts_empty() {
    let g = Grammar::new("empty".into());
    assert!(g.externals.is_empty());
}

#[test]
fn grammar_add_external_directly() {
    let mut g = Grammar::new("test".into());
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "dedent".into(),
        symbol_id: SymbolId(11),
    });
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.externals[0].name, "indent");
    assert_eq!(g.externals[1].symbol_id, SymbolId(11));
}

#[test]
fn grammar_builder_external() {
    let g = GrammarBuilder::new("python")
        .token("ID", "[a-z]+")
        .external("indent")
        .external("dedent")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.externals[0].name, "indent");
    assert_eq!(g.externals[1].name, "dedent");
}

#[test]
fn external_symbol_validates_when_referenced() {
    let mut g = Grammar::new("test".into());
    let ext_id = SymbolId(100);
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: ext_id,
    });
    // Add a rule and a token so the rule's terminal resolves
    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let rule = make_rule(
        SymbolId(0),
        vec![Symbol::Terminal(tok_id), Symbol::External(ext_id)],
    );
    g.rules.insert(SymbolId(0), vec![rule]);
    // Validation passes because the external is registered
    assert!(g.validate().is_ok());
}

#[test]
fn external_symbol_validation_fails_unresolved() {
    let mut g = Grammar::new("test".into());
    let tok_id = SymbolId(1);
    g.tokens.insert(
        tok_id,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    // Reference an external that doesn't exist
    let rule = make_rule(
        SymbolId(0),
        vec![Symbol::Terminal(tok_id), Symbol::External(SymbolId(999))],
    );
    g.rules.insert(SymbolId(0), vec![rule]);
    let err = g.validate().unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("external"),
        "expected external error, got: {msg}"
    );
}

#[test]
fn build_registry_includes_externals() {
    let mut g = GrammarBuilder::new("lang")
        .token("ID", "[a-z]+")
        .external("indent")
        .external("dedent")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let registry = g.get_or_build_registry();
    // Externals should be registered in the symbol registry
    let names: Vec<_> = registry.iter().map(|(name, _)| name.to_string()).collect();
    assert!(names.contains(&"indent".to_string()));
    assert!(names.contains(&"dedent".to_string()));
}

// ===========================================================================
// 3. Grammar.extras manipulation
// ===========================================================================

#[test]
fn grammar_extras_starts_empty() {
    let g = Grammar::new("empty".into());
    assert!(g.extras.is_empty());
}

#[test]
fn grammar_add_extras_directly() {
    let mut g = simple_grammar_with_token(1, "WS");
    g.extras.push(SymbolId(1));
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.extras[0], SymbolId(1));
}

#[test]
fn grammar_builder_extra() {
    let g = GrammarBuilder::new("lang")
        .token("WS", "\\s+")
        .token("COMMENT", "//[^\n]*")
        .token("ID", "[a-z]+")
        .extra("WS")
        .extra("COMMENT")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn extras_marked_hidden_in_registry() {
    let mut g = GrammarBuilder::new("lang")
        .token("WS", "\\s+")
        .token("ID", "[a-z]+")
        .extra("WS")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let registry = g.get_or_build_registry();
    // The WS token should be hidden because it's in extras
    for (name, info) in registry.iter() {
        if name == "WS" {
            assert!(info.metadata.hidden, "extras should be marked hidden");
        }
    }
}

#[test]
fn multiple_extras_all_tracked() {
    let mut g = Grammar::new("test".into());
    for i in 0..5 {
        g.extras.push(SymbolId(i));
    }
    assert_eq!(g.extras.len(), 5);
    for i in 0..5 {
        assert!(g.extras.contains(&SymbolId(i)));
    }
}

// ===========================================================================
// 4. Grammar.supertypes manipulation
// ===========================================================================

#[test]
fn grammar_supertypes_starts_empty() {
    let g = Grammar::new("empty".into());
    assert!(g.supertypes.is_empty());
}

#[test]
fn grammar_add_supertypes() {
    let mut g = Grammar::new("test".into());
    g.supertypes.push(SymbolId(10));
    g.supertypes.push(SymbolId(20));
    assert_eq!(g.supertypes.len(), 2);
    assert_eq!(g.supertypes[0], SymbolId(10));
    assert_eq!(g.supertypes[1], SymbolId(20));
}

#[test]
fn supertypes_preserved_through_serde() {
    let mut g = Grammar::new("test".into());
    g.supertypes.push(SymbolId(5));
    g.supertypes.push(SymbolId(15));
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g2.supertypes, vec![SymbolId(5), SymbolId(15)]);
}

#[test]
fn supertypes_clone() {
    let mut g = Grammar::new("test".into());
    g.supertypes.push(SymbolId(3));
    let g2 = g.clone();
    assert_eq!(g2.supertypes, g.supertypes);
}

// ===========================================================================
// 5. Grammar.conflicts manipulation
// ===========================================================================

#[test]
fn grammar_conflicts_starts_empty() {
    let g = Grammar::new("empty".into());
    assert!(g.conflicts.is_empty());
}

#[test]
fn grammar_add_conflict_glr() {
    let mut g = Grammar::new("test".into());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.conflicts[0].resolution, ConflictResolution::GLR);
    assert_eq!(g.conflicts[0].symbols.len(), 2);
}

#[test]
fn grammar_add_conflict_precedence() {
    let mut g = Grammar::new("test".into());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(5), SymbolId(6), SymbolId(7)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(3)),
    });
    assert_eq!(g.conflicts[0].symbols.len(), 3);
    assert_eq!(
        g.conflicts[0].resolution,
        ConflictResolution::Precedence(PrecedenceKind::Static(3))
    );
}

#[test]
fn grammar_add_conflict_associativity() {
    let mut g = Grammar::new("test".into());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    assert_eq!(
        g.conflicts[0].resolution,
        ConflictResolution::Associativity(Associativity::Left)
    );
}

#[test]
fn conflict_declaration_serde_roundtrip() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    };
    let json = serde_json::to_string(&cd).unwrap();
    let cd2: ConflictDeclaration = serde_json::from_str(&json).unwrap();
    assert_eq!(cd2.symbols, vec![SymbolId(1), SymbolId(2)]);
}

#[test]
fn conflict_resolution_variants_debug() {
    let glr = ConflictResolution::GLR;
    let prec = ConflictResolution::Precedence(PrecedenceKind::Dynamic(2));
    let assoc = ConflictResolution::Associativity(Associativity::Right);
    assert!(format!("{glr:?}").contains("GLR"));
    assert!(format!("{prec:?}").contains("Dynamic"));
    assert!(format!("{assoc:?}").contains("Right"));
}

// ===========================================================================
// 6. Grammar.inline_rules manipulation
// ===========================================================================

#[test]
fn grammar_inline_rules_starts_empty() {
    let g = Grammar::new("empty".into());
    assert!(g.inline_rules.is_empty());
}

#[test]
fn grammar_add_inline_rules() {
    let mut g = Grammar::new("test".into());
    g.inline_rules.push(SymbolId(3));
    g.inline_rules.push(SymbolId(4));
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn inline_rules_preserved_through_serde() {
    let mut g = Grammar::new("test".into());
    g.inline_rules.push(SymbolId(7));
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g2.inline_rules, vec![SymbolId(7)]);
}

// ===========================================================================
// 7. Interactions between features and Grammar methods
// ===========================================================================

#[test]
fn grammar_with_all_features_serde_roundtrip() {
    let mut g = Grammar::new("full".into());
    // tokens
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "ID".into(),
            pattern: TokenPattern::Regex("[a-z]+".into()),
            fragile: false,
        },
    );
    // rules
    g.add_rule(make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]));
    g.rule_names.insert(SymbolId(0), "start".into());
    // externals
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(100),
    });
    // extras
    g.extras.push(SymbolId(1));
    // supertypes
    g.supertypes.push(SymbolId(0));
    // conflicts
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(0)],
        resolution: ConflictResolution::GLR,
    });
    // inline_rules
    g.inline_rules.push(SymbolId(0));

    let json = serde_json::to_string_pretty(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g2.name, "full");
    assert_eq!(g2.externals.len(), 1);
    assert_eq!(g2.extras.len(), 1);
    assert_eq!(g2.supertypes.len(), 1);
    assert_eq!(g2.conflicts.len(), 1);
    assert_eq!(g2.inline_rules.len(), 1);
    assert_eq!(g2.rules.len(), 1);
}

#[test]
fn grammar_default_has_empty_collections() {
    let g = Grammar::default();
    assert!(g.name.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.conflicts.is_empty());
    assert!(g.inline_rules.is_empty());
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn grammar_clone_preserves_externals_and_extras() {
    let g = GrammarBuilder::new("lang")
        .token("WS", "\\s+")
        .token("ID", "[a-z]+")
        .external("indent")
        .extra("WS")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.externals.len(), g.externals.len());
    assert_eq!(g2.extras.len(), g.extras.len());
    assert_eq!(g2.externals[0].name, g.externals[0].name);
}

#[test]
fn validate_passes_with_externals_extras_conflicts() {
    let mut g = Grammar::new("test".into());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let ext_id = SymbolId(50);
    g.externals.push(ExternalToken {
        name: "ext".into(),
        symbol_id: ext_id,
    });
    g.extras.push(tok);
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![tok],
        resolution: ConflictResolution::GLR,
    });
    g.add_rule(make_rule(
        SymbolId(0),
        vec![Symbol::Terminal(tok), Symbol::External(ext_id)],
    ));
    assert!(g.validate().is_ok());
}

#[test]
fn builder_external_creates_unique_symbol_ids() {
    let g = GrammarBuilder::new("lang")
        .token("ID", "[a-z]+")
        .external("indent")
        .external("dedent")
        .external("newline")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let ids: Vec<_> = g.externals.iter().map(|e| e.symbol_id).collect();
    // All IDs should be distinct
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "external symbol IDs must be unique");
        }
    }
}

#[test]
fn find_symbol_by_name_finds_externals_registered_by_builder() {
    let g = GrammarBuilder::new("lang")
        .token("ID", "[a-z]+")
        .external("indent")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    // The builder registers externals in rule_names, so they are findable
    let found = g.find_symbol_by_name("indent");
    assert!(
        found.is_some(),
        "builder-registered externals should be in rule_names"
    );
}

#[test]
fn check_empty_terminals_ignores_externals() {
    let mut g = Grammar::new("test".into());
    g.externals.push(ExternalToken {
        name: "".into(), // empty name, but externals aren't terminals
        symbol_id: SymbolId(10),
    });
    // No tokens => no empty terminal errors
    assert!(g.check_empty_terminals().is_ok());
}

#[test]
fn multiple_conflict_resolutions_coexist() {
    let mut g = Grammar::new("test".into());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(4)],
        resolution: ConflictResolution::Associativity(Associativity::None),
    });
    assert_eq!(g.conflicts.len(), 3);
}
