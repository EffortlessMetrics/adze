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

    // -----------------------------------------------------------------------
    // 36. Construction: empty name
    // -----------------------------------------------------------------------

    #[test]
    fn ext_empty_name_construction(id in arb_symbol_id()) {
        let et = ExternalToken { name: String::new(), symbol_id: id };
        prop_assert!(et.name.is_empty());
        prop_assert_eq!(et.symbol_id, id);
    }

    // -----------------------------------------------------------------------
    // 37. Construction: zero symbol ID
    // -----------------------------------------------------------------------

    #[test]
    fn ext_zero_symbol_id(name in arb_name()) {
        let et = ExternalToken { name: name.clone(), symbol_id: SymbolId(0) };
        prop_assert_eq!(et.symbol_id.0, 0);
        prop_assert_eq!(&et.name, &name);
    }

    // -----------------------------------------------------------------------
    // 38. PartialEq: reflexivity
    // -----------------------------------------------------------------------

    #[test]
    fn ext_eq_reflexive(et in arb_external_token()) {
        prop_assert_eq!(&et, &et);
    }

    // -----------------------------------------------------------------------
    // 39. PartialEq: symmetry
    // -----------------------------------------------------------------------

    #[test]
    fn ext_eq_symmetric(et in arb_external_token()) {
        let other = et.clone();
        prop_assert_eq!(&et, &other);
        prop_assert_eq!(&other, &et);
    }

    // -----------------------------------------------------------------------
    // 40. PartialEq: transitivity
    // -----------------------------------------------------------------------

    #[test]
    fn ext_eq_transitive(et in arb_external_token()) {
        let b = et.clone();
        let c = b.clone();
        prop_assert_eq!(&et, &b);
        prop_assert_eq!(&b, &c);
        prop_assert_eq!(&et, &c);
    }

    // -----------------------------------------------------------------------
    // 41. PartialEq: different name, same ID → not equal
    // -----------------------------------------------------------------------

    #[test]
    fn ext_ne_different_name(id in arb_symbol_id()) {
        let a = ExternalToken { name: "alpha".into(), symbol_id: id };
        let b = ExternalToken { name: "beta".into(), symbol_id: id };
        prop_assert_ne!(&a, &b);
    }

    // -----------------------------------------------------------------------
    // 42. PartialEq: same name, different ID → not equal
    // -----------------------------------------------------------------------

    #[test]
    fn ext_ne_different_id(name in arb_name(), a in 0u16..250, b in 250u16..500) {
        let ea = ExternalToken { name: name.clone(), symbol_id: SymbolId(a) };
        let eb = ExternalToken { name: name.clone(), symbol_id: SymbolId(b) };
        prop_assert_ne!(&ea, &eb);
    }

    // -----------------------------------------------------------------------
    // 43. Serialization: postcard roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn ext_postcard_roundtrip(et in arb_external_token()) {
        let bytes = postcard::to_stdvec(&et).unwrap();
        let back: ExternalToken = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(&back, &et);
    }

    // -----------------------------------------------------------------------
    // 44. Serialization: Vec<ExternalToken> roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn ext_vec_json_roundtrip(
        tokens in prop::collection::vec(arb_external_token(), 0..10)
    ) {
        let json = serde_json::to_string(&tokens).unwrap();
        let back: Vec<ExternalToken> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.len(), tokens.len());
        for i in 0..tokens.len() {
            prop_assert_eq!(&back[i], &tokens[i]);
        }
    }

    // -----------------------------------------------------------------------
    // 45. Serialization: deserialization from hand-crafted JSON
    // -----------------------------------------------------------------------

    #[test]
    fn ext_from_manual_json(id in 0u16..500, name in arb_name()) {
        let json = format!(r#"{{"name":"{}","symbol_id":{}}}"#, name, id);
        let et: ExternalToken = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&et.name, &name);
        prop_assert_eq!(et.symbol_id.0, id);
    }

    // -----------------------------------------------------------------------
    // 46. Determinism: repeated serialization produces identical output
    // -----------------------------------------------------------------------

    #[test]
    fn ext_serialization_deterministic(et in arb_external_token()) {
        let json1 = serde_json::to_string(&et).unwrap();
        let json2 = serde_json::to_string(&et).unwrap();
        prop_assert_eq!(&json1, &json2);
    }

    // -----------------------------------------------------------------------
    // 47. Determinism: repeated grammar build with externals produces same IDs
    // -----------------------------------------------------------------------

    #[test]
    fn builder_deterministic_ids(n in 1usize..6) {
        let names: Vec<String> = (0..n).map(|i| format!("ext{i}")).collect();
        let build = || {
            let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
            for name in &names {
                b = b.external(name);
            }
            b.rule("start", vec!["ID"]).start("start").build()
        };
        let g1 = build();
        let g2 = build();
        for i in 0..n {
            prop_assert_eq!(g1.externals[i].symbol_id, g2.externals[i].symbol_id);
            prop_assert_eq!(&g1.externals[i].name, &g2.externals[i].name);
        }
    }

    // -----------------------------------------------------------------------
    // 48. Determinism: grammar JSON serialization is deterministic
    // -----------------------------------------------------------------------

    #[test]
    fn grammar_serde_deterministic_with_externals(n in 1usize..5) {
        let externals: Vec<ExternalToken> = (0..n)
            .map(|i| ExternalToken {
                name: format!("ext{i}"),
                symbol_id: SymbolId(100 + i as u16),
            })
            .collect();
        let g = grammar_with_externals(externals);
        let json1 = serde_json::to_string(&g).unwrap();
        let json2 = serde_json::to_string(&g).unwrap();
        prop_assert_eq!(&json1, &json2);
    }

    // -----------------------------------------------------------------------
    // 49. Multiple externals: duplicate symbol IDs are preserved
    // -----------------------------------------------------------------------

    #[test]
    fn grammar_allows_duplicate_external_ids(id in 100u16..500) {
        let mut g = grammar_with_externals(vec![]);
        g.externals.push(ExternalToken { name: "a".into(), symbol_id: SymbolId(id) });
        g.externals.push(ExternalToken { name: "b".into(), symbol_id: SymbolId(id) });
        prop_assert_eq!(g.externals.len(), 2);
        prop_assert_eq!(g.externals[0].symbol_id, g.externals[1].symbol_id);
    }

    // -----------------------------------------------------------------------
    // 50. Symbol IDs: external IDs don't collide with builder token IDs
    // -----------------------------------------------------------------------

    #[test]
    fn builder_external_ids_not_collide_with_tokens(n in 1usize..5) {
        let mut b = GrammarBuilder::new("test");
        for i in 0..n {
            b = b.token(&format!("TOK{i}"), &format!("t{i}"));
        }
        b = b.external("scanner");
        let g = b.rule("start", vec!["TOK0"]).start("start").build();
        let tok_ids: std::collections::HashSet<_> = g.tokens.keys().collect();
        for ext in &g.externals {
            prop_assert!(!tok_ids.contains(&ext.symbol_id), "external ID collides with token");
        }
    }

    // -----------------------------------------------------------------------
    // 51. Grammar default: empty externals
    // -----------------------------------------------------------------------

    #[test]
    fn grammar_default_has_empty_externals(_dummy in 0..1u8) {
        let g = Grammar::default();
        prop_assert!(g.externals.is_empty());
    }

    // -----------------------------------------------------------------------
    // 52. Grammar new: empty externals
    // -----------------------------------------------------------------------

    #[test]
    fn grammar_new_has_empty_externals(name in "[a-z]{1,10}") {
        let g = Grammar::new(name);
        prop_assert!(g.externals.is_empty());
    }

    // -----------------------------------------------------------------------
    // 53. Validation: nested Symbol::External in Choice
    // -----------------------------------------------------------------------

    #[test]
    fn validate_external_in_choice(ext_id_raw in 100u16..400) {
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
                rhs: vec![Symbol::Choice(vec![
                    Symbol::Terminal(tok_id),
                    Symbol::External(ext_id),
                ])],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 54. Validation: nested Symbol::External in Sequence
    // -----------------------------------------------------------------------

    #[test]
    fn validate_external_in_sequence(ext_id_raw in 100u16..400) {
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
                rhs: vec![Symbol::Sequence(vec![
                    Symbol::Terminal(tok_id),
                    Symbol::External(ext_id),
                ])],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 55. Validation: Symbol::External inside Optional
    // -----------------------------------------------------------------------

    #[test]
    fn validate_external_in_optional(ext_id_raw in 100u16..400) {
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
                    Symbol::Terminal(tok_id),
                    Symbol::Optional(Box::new(Symbol::External(ext_id))),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 56. Validation: Symbol::External inside Repeat
    // -----------------------------------------------------------------------

    #[test]
    fn validate_external_in_repeat(ext_id_raw in 100u16..400) {
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
                    Symbol::Terminal(tok_id),
                    Symbol::Repeat(Box::new(Symbol::External(ext_id))),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 57. Validation: Symbol::External inside RepeatOne
    // -----------------------------------------------------------------------

    #[test]
    fn validate_external_in_repeat_one(ext_id_raw in 100u16..400) {
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
                    Symbol::Terminal(tok_id),
                    Symbol::RepeatOne(Box::new(Symbol::External(ext_id))),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 58. Validation: unregistered external in nested position fails
    // -----------------------------------------------------------------------

    #[test]
    fn validate_unregistered_external_in_optional_fails(id in 600u16..1000) {
        let tok_id = SymbolId(1);
        let mut g = Grammar::new("test".into());
        g.tokens.insert(tok_id, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.rules.insert(
            SymbolId(0),
            vec![Rule {
                lhs: SymbolId(0),
                rhs: vec![Symbol::Optional(Box::new(Symbol::External(SymbolId(id))))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );
        prop_assert!(g.validate().is_err());
    }

    // -----------------------------------------------------------------------
    // 59. Multiple externals: symbol IDs are all distinct from builder
    // -----------------------------------------------------------------------

    #[test]
    fn builder_multiple_externals_all_distinct(n in 2usize..10) {
        let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
        for i in 0..n {
            b = b.external(&format!("ext{i}"));
        }
        let g = b.rule("start", vec!["ID"]).start("start").build();
        let mut ids: Vec<u16> = g.externals.iter().map(|e| e.symbol_id.0).collect();
        ids.sort();
        ids.dedup();
        prop_assert_eq!(ids.len(), n, "all external IDs must be distinct");
    }

    // -----------------------------------------------------------------------
    // 60. Determinism: registry build is deterministic
    // -----------------------------------------------------------------------

    #[test]
    fn registry_build_deterministic(n in 1usize..5) {
        let names: Vec<String> = (0..n).map(|i| format!("ext{i}")).collect();
        let build_reg = || {
            let mut b = GrammarBuilder::new("test").token("ID", "[a-z]+");
            for name in &names {
                b = b.external(name);
            }
            let mut g = b.rule("start", vec!["ID"]).start("start").build();
            let reg = g.get_or_build_registry();
            reg.iter().map(|(n, m)| (n.to_string(), m.id)).collect::<Vec<_>>()
        };
        let r1 = build_reg();
        let r2 = build_reg();
        prop_assert_eq!(r1, r2, "registry must be deterministic across builds");
    }
}
