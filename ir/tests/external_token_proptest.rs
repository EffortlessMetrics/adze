#![allow(clippy::needless_range_loop)]

//! Property-based tests for ExternalToken handling in the IR crate.
//!
//! Covers construction, serde roundtrips, grammar integration, validation,
//! normalization interaction, builder API, and registry behaviour.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, Grammar, GrammarValidator, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..500).prop_map(SymbolId)
}

fn arb_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_map(String::from)
}

fn arb_external_token() -> impl Strategy<Value = ExternalToken> {
    (arb_name(), arb_symbol_id()).prop_map(|(name, symbol_id)| ExternalToken { name, symbol_id })
}

/// Build a minimal valid grammar containing a token, a rule, and the given externals.
fn grammar_with_externals(externals: Vec<ExternalToken>) -> Grammar {
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
    g.rules.insert(
        SymbolId(0),
        vec![Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.externals = externals;
    g
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    // -----------------------------------------------------------------------
    // 1–3. ExternalToken field preservation
    // -----------------------------------------------------------------------

    #[test]
    fn ext_name_preserved(name in arb_name()) {
        let et = ExternalToken { name: name.clone(), symbol_id: SymbolId(0) };
        prop_assert_eq!(&et.name, &name);
    }

    #[test]
    fn ext_symbol_id_preserved(id in 0u16..=u16::MAX) {
        let et = ExternalToken { name: "tok".into(), symbol_id: SymbolId(id) };
        prop_assert_eq!(et.symbol_id, SymbolId(id));
    }

    #[test]
    fn ext_clone_equals_original(et in arb_external_token()) {
        let cloned = et.clone();
        prop_assert_eq!(&cloned.name, &et.name);
        prop_assert_eq!(cloned.symbol_id, et.symbol_id);
    }

    // -----------------------------------------------------------------------
    // 4–6. Serde roundtrips
    // -----------------------------------------------------------------------

    #[test]
    fn ext_json_roundtrip(et in arb_external_token()) {
        let json = serde_json::to_string(&et).unwrap();
        let back: ExternalToken = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.name, &et.name);
        prop_assert_eq!(back.symbol_id, et.symbol_id);
    }

    #[test]
    fn ext_json_value_shape(et in arb_external_token()) {
        let val: serde_json::Value = serde_json::to_value(&et).unwrap();
        prop_assert_eq!(val["name"].as_str().unwrap(), &et.name);
        prop_assert_eq!(val["symbol_id"].as_u64().unwrap(), et.symbol_id.0 as u64);
    }

    #[test]
    fn ext_pretty_json_roundtrip(et in arb_external_token()) {
        let json = serde_json::to_string_pretty(&et).unwrap();
        let back: ExternalToken = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.name, &et.name);
        prop_assert_eq!(back.symbol_id, et.symbol_id);
    }

    // -----------------------------------------------------------------------
    // 7–8. Debug formatting
    // -----------------------------------------------------------------------

    #[test]
    fn ext_debug_contains_name(et in arb_external_token()) {
        let dbg = format!("{et:?}");
        prop_assert!(dbg.contains("ExternalToken"));
        prop_assert!(dbg.contains(&et.name));
    }

    #[test]
    fn ext_debug_contains_id(id in 0u16..1000) {
        let et = ExternalToken { name: "x".into(), symbol_id: SymbolId(id) };
        let dbg = format!("{et:?}");
        prop_assert!(dbg.contains(&id.to_string()));
    }

    // -----------------------------------------------------------------------
    // 9–11. Grammar.externals collection operations
    // -----------------------------------------------------------------------

    #[test]
    fn grammar_externals_length_matches(n in 0usize..20) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("ext_{i}"),
                symbol_id: SymbolId(100 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        prop_assert_eq!(g.externals.len(), n);
    }

    #[test]
    fn grammar_externals_order_preserved(n in 1usize..10) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("ext_{i}"),
                symbol_id: SymbolId(200 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        for i in 0..n {
            prop_assert_eq!(&g.externals[i].name, &format!("ext_{i}"));
            prop_assert_eq!(g.externals[i].symbol_id, SymbolId(200 + i as u16));
        }
    }

    #[test]
    fn grammar_clone_preserves_externals(n in 0usize..10) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("e{i}"),
                symbol_id: SymbolId(300 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let g2 = g.clone();
        prop_assert_eq!(g2.externals.len(), g.externals.len());
        for i in 0..n {
            prop_assert_eq!(&g2.externals[i].name, &g.externals[i].name);
            prop_assert_eq!(g2.externals[i].symbol_id, g.externals[i].symbol_id);
        }
    }

    // -----------------------------------------------------------------------
    // 12–14. Grammar serde with externals
    // -----------------------------------------------------------------------

    #[test]
    fn grammar_serde_preserves_externals(n in 0usize..8) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("tok{i}"),
                symbol_id: SymbolId(400 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let json = serde_json::to_string(&g).unwrap();
        let g2: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(g2.externals.len(), n);
        for i in 0..n {
            prop_assert_eq!(&g2.externals[i].name, &g.externals[i].name);
            prop_assert_eq!(g2.externals[i].symbol_id, g.externals[i].symbol_id);
        }
    }

    #[test]
    fn grammar_serde_external_ids_survive(ids in prop::collection::vec(0u16..500, 1..8)) {
        let externals: Vec<ExternalToken> = ids
            .iter()
            .enumerate()
            .map(|(i, &id)| ExternalToken {
                name: format!("e{i}"),
                symbol_id: SymbolId(id),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let json = serde_json::to_string(&g).unwrap();
        let g2: Grammar = serde_json::from_str(&json).unwrap();
        for (i, &id) in ids.iter().enumerate() {
            prop_assert_eq!(g2.externals[i].symbol_id, SymbolId(id));
        }
    }

    #[test]
    fn grammar_serde_external_names_survive(
        names in prop::collection::vec("[a-z]{1,6}", 1..6)
    ) {
        let externals: Vec<ExternalToken> = names
            .iter()
            .enumerate()
            .map(|(i, name)| ExternalToken {
                name: name.clone(),
                symbol_id: SymbolId(500 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let json = serde_json::to_string(&g).unwrap();
        let g2: Grammar = serde_json::from_str(&json).unwrap();
        for (i, name) in names.iter().enumerate() {
            prop_assert_eq!(&g2.externals[i].name, name);
        }
    }

    // -----------------------------------------------------------------------
    // 15–17. Validation – resolved vs unresolved externals
    // -----------------------------------------------------------------------

    #[test]
    fn validate_passes_when_external_registered(id in 100u16..500) {
        let mut g = Grammar::new("test".into());
        let tok_id = SymbolId(1);
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        let ext_id = SymbolId(id);
        g.externals.push(ExternalToken {
            name: "ext".into(),
            symbol_id: ext_id,
        });
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![Symbol::Terminal(tok_id), Symbol::External(ext_id)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn validate_fails_for_unresolved_external(id in 600u16..1000) {
        let mut g = Grammar::new("test".into());
        let tok_id = SymbolId(1);
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![Symbol::Terminal(tok_id), Symbol::External(SymbolId(id))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        let err = g.validate().unwrap_err();
        let msg = format!("{err}");
        prop_assert!(msg.contains("external"), "expected external error: {msg}");
    }

    #[test]
    fn validate_external_error_contains_id(id in 600u16..1000) {
        let mut g = Grammar::new("test".into());
        let tok_id = SymbolId(1);
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![Symbol::External(SymbolId(id))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        let err = g.validate().unwrap_err();
        let msg = format!("{err:?}");
        prop_assert!(msg.contains(&id.to_string()), "error should mention id: {msg}");
    }

    // -----------------------------------------------------------------------
    // 18–20. Builder API for externals
    // -----------------------------------------------------------------------

    #[test]
    fn builder_external_count(n in 1usize..8) {
        let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
        for i in 0..n {
            b = b.external(&format!("ext{i}"));
        }
        let g = b.rule("start", vec!["ID"]).start("start").build();
        prop_assert_eq!(g.externals.len(), n);
    }

    #[test]
    fn builder_external_unique_ids(n in 2usize..8) {
        let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
        for i in 0..n {
            b = b.external(&format!("ext{i}"));
        }
        let g = b.rule("start", vec!["ID"]).start("start").build();
        let ids: Vec<_> = g.externals.iter().map(|e| e.symbol_id).collect();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                prop_assert_ne!(ids[i], ids[j], "external IDs must be unique");
            }
        }
    }

    #[test]
    fn builder_external_names_match(n in 1usize..6) {
        let names: Vec<String> = (0..n).map(|i| format!("scanner{i}")).collect();
        let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
        for name in &names {
            b = b.external(name);
        }
        let g = b.rule("start", vec!["ID"]).start("start").build();
        for (i, name) in names.iter().enumerate() {
            prop_assert_eq!(&g.externals[i].name, name);
        }
    }

    // -----------------------------------------------------------------------
    // 21–22. Normalization leaves externals untouched
    // -----------------------------------------------------------------------

    #[test]
    fn normalization_preserves_external_list(n in 1usize..6) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("ext{i}"),
                symbol_id: SymbolId(100 + i as u16),
            })
            .collect();
        let mut g = grammar_with_externals(externals.clone());
        g.normalize();
        prop_assert_eq!(g.externals.len(), n);
        for i in 0..n {
            prop_assert_eq!(&g.externals[i].name, &externals[i].name);
            prop_assert_eq!(g.externals[i].symbol_id, externals[i].symbol_id);
        }
    }

    #[test]
    fn normalization_does_not_rewrite_external_symbols(id in 100u16..500) {
        let ext_id = SymbolId(id);
        let mut g = Grammar::new("test".into());
        let tok_id = SymbolId(1);
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.externals.push(ExternalToken {
            name: "ext".into(),
            symbol_id: ext_id,
        });
        // Rule referencing the external symbol
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![Symbol::Terminal(tok_id), Symbol::External(ext_id)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        g.normalize();
        // External symbol reference is preserved in the rule RHS
        let all_rhs: Vec<&Symbol> = g.all_rules().flat_map(|r| r.rhs.iter()).collect();
        let has_ext = all_rhs.iter().any(|s| matches!(s, Symbol::External(sid) if *sid == ext_id));
        prop_assert!(has_ext, "Symbol::External({id}) must survive normalization");
    }

    // -----------------------------------------------------------------------
    // 23–24. Registry includes externals
    // -----------------------------------------------------------------------

    #[test]
    fn registry_contains_external_names(n in 1usize..6) {
        let names: Vec<String> = (0..n).map(|i| format!("ext{i}")).collect();
        let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
        for name in &names {
            b = b.external(name);
        }
        let mut g = b.rule("start", vec!["ID"]).start("start").build();
        let registry = g.get_or_build_registry();
        let reg_names: Vec<String> = registry.iter().map(|(n, _)| n.to_string()).collect();
        for name in &names {
            prop_assert!(reg_names.contains(name), "registry missing {name}");
        }
    }

    #[test]
    fn registry_externals_are_terminal(n in 1usize..5) {
        let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
        for i in 0..n {
            b = b.external(&format!("ext{i}"));
        }
        let mut g = b.rule("start", vec!["ID"]).start("start").build();
        let registry = g.get_or_build_registry();
        for i in 0..n {
            let name = format!("ext{i}");
            let info = registry.iter().find(|(n, _)| *n == name);
            prop_assert!(info.is_some(), "expected {name} in registry");
            let (_, meta) = info.unwrap();
            prop_assert!(meta.metadata.terminal, "{name} should be terminal");
        }
    }

    // -----------------------------------------------------------------------
    // 25–27. GrammarValidator external_tokens stat & duplicate detection
    // -----------------------------------------------------------------------

    #[test]
    fn validator_counts_external_tokens(n in 0usize..10) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("ext{i}"),
                symbol_id: SymbolId(100 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let mut validator = GrammarValidator::new();
        let result = validator.validate(&g);
        prop_assert_eq!(result.stats.external_tokens, n);
    }

    #[test]
    fn validator_detects_duplicate_external_names(name in arb_name()) {
        let mut g = grammar_with_externals(vec![]);
        g.externals.push(ExternalToken { name: name.clone(), symbol_id: SymbolId(100) });
        g.externals.push(ExternalToken { name: name.clone(), symbol_id: SymbolId(101) });
        let mut validator = GrammarValidator::new();
        let result = validator.validate(&g);
        let has_conflict = result.errors.iter().any(|e| {
            let msg = format!("{e}");
            msg.contains("conflict") || msg.contains("External")
        });
        prop_assert!(has_conflict, "duplicate external names should produce a conflict error");
    }

    #[test]
    fn validator_no_conflict_for_unique_names(n in 1usize..6) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("unique{i}"),
                symbol_id: SymbolId(100 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let mut validator = GrammarValidator::new();
        let result = validator.validate(&g);
        let has_ext_conflict = result.errors.iter().any(|e| {
            let msg = format!("{e}");
            msg.contains("External tokens")
        });
        prop_assert!(!has_ext_conflict, "unique names must not trigger conflict");
    }

    // -----------------------------------------------------------------------
    // 28. Symbol::External equality and hashing
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_external_eq_and_hash(id in 0u16..500) {
        let s1 = Symbol::External(SymbolId(id));
        let s2 = Symbol::External(SymbolId(id));
        prop_assert_eq!(&s1, &s2);
        let mut set = std::collections::HashSet::new();
        set.insert(s1.clone());
        prop_assert!(set.contains(&s2));
    }

    // -----------------------------------------------------------------------
    // 29. Symbol::External ordering
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_external_ord(a in 0u16..500, b in 0u16..500) {
        let sa = Symbol::External(SymbolId(a));
        let sb = Symbol::External(SymbolId(b));
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }

    // -----------------------------------------------------------------------
    // 30. check_empty_terminals ignores externals
    // -----------------------------------------------------------------------

    #[test]
    fn check_empty_terminals_ignores_externals(n in 0usize..10) {
        let mut g = Grammar::new("test".into());
        for i in 0..n {
            g.externals.push(ExternalToken {
                name: String::new(),
                symbol_id: SymbolId(100 + i as u16),
            });
        }
        prop_assert!(g.check_empty_terminals().is_ok());
    }

    // -----------------------------------------------------------------------
    // 31. find_symbol_by_name discovers builder-registered externals
    // -----------------------------------------------------------------------

    #[test]
    fn find_symbol_by_name_finds_external(name in "[a-z]{2,8}") {
        let g = GrammarBuilder::new("test")
            .token("ID", "[a-z]+")
            .external(&name)
            .rule("start", vec!["ID"])
            .start("start")
            .build();
        let found = g.find_symbol_by_name(&name);
        prop_assert!(found.is_some(), "external '{name}' should be findable");
    }

    // -----------------------------------------------------------------------
    // 32–33. Normalization with complex symbols alongside externals
    // -----------------------------------------------------------------------

    #[test]
    fn normalization_with_optional_and_external(ext_id_raw in 100u16..400) {
        let ext_id = SymbolId(ext_id_raw);
        let tok_id = SymbolId(1);
        let mut g = Grammar::new("test".into());
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.externals.push(ExternalToken {
            name: "ext".into(),
            symbol_id: ext_id,
        });
        // Rule with Optional wrapping a terminal, followed by External
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![
                    Symbol::Optional(Box::new(Symbol::Terminal(tok_id))),
                    Symbol::External(ext_id),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        g.normalize();
        // After normalization, External reference must still appear
        let all_syms: Vec<&Symbol> = g.all_rules().flat_map(|r| r.rhs.iter()).collect();
        let ext_present = all_syms.iter().any(|s| matches!(s, Symbol::External(id) if *id == ext_id));
        prop_assert!(ext_present, "External({ext_id_raw}) must survive optional normalization");
    }

    #[test]
    fn normalization_with_repeat_and_external(ext_id_raw in 100u16..400) {
        let ext_id = SymbolId(ext_id_raw);
        let tok_id = SymbolId(1);
        let mut g = Grammar::new("test".into());
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.externals.push(ExternalToken {
            name: "ext".into(),
            symbol_id: ext_id,
        });
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![
                    Symbol::Repeat(Box::new(Symbol::Terminal(tok_id))),
                    Symbol::External(ext_id),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        g.normalize();
        let all_syms: Vec<&Symbol> = g.all_rules().flat_map(|r| r.rhs.iter()).collect();
        let ext_present = all_syms.iter().any(|s| matches!(s, Symbol::External(id) if *id == ext_id));
        prop_assert!(ext_present, "External({ext_id_raw}) must survive repeat normalization");
    }

    // -----------------------------------------------------------------------
    // 34–35. Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn external_with_max_symbol_id(name in arb_name()) {
        let et = ExternalToken { name: name.clone(), symbol_id: SymbolId(u16::MAX) };
        let json = serde_json::to_string(&et).unwrap();
        let back: ExternalToken = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.name, &name);
        prop_assert_eq!(back.symbol_id, SymbolId(u16::MAX));
    }

    #[test]
    fn grammar_with_only_externals_validates(n in 1usize..6) {
        // Grammar with no rules referencing externals should still validate
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("ext{i}"),
                symbol_id: SymbolId(100 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        // validate() should succeed – externals exist but aren't referenced
        prop_assert!(g.validate().is_ok());
    }
}
