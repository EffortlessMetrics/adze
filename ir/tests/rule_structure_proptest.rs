#![allow(clippy::needless_range_loop)]

//! Property-based tests for Rule structure in adze-ir.

use adze_ir::{
    AliasSequence, Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol,
    SymbolId,
};
use proptest::prelude::*;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_rule(lhs: u16, rhs: Vec<Symbol>) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

fn term(id: u16) -> Symbol {
    Symbol::Terminal(SymbolId(id))
}

fn nonterm(id: u16) -> Symbol {
    Symbol::NonTerminal(SymbolId(id))
}

// ── proptest strategies ──────────────────────────────────────────────────────

fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    (1u16..500).prop_map(SymbolId)
}

fn field_id_strategy() -> impl Strategy<Value = FieldId> {
    (0u16..100).prop_map(FieldId)
}

fn production_id_strategy() -> impl Strategy<Value = ProductionId> {
    (0u16..1000).prop_map(ProductionId)
}

fn precedence_kind_strategy() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        (-100i16..100).prop_map(PrecedenceKind::Static),
        (-100i16..100).prop_map(PrecedenceKind::Dynamic),
    ]
}

fn associativity_strategy() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

fn leaf_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        symbol_id_strategy().prop_map(Symbol::Terminal),
        symbol_id_strategy().prop_map(Symbol::NonTerminal),
        symbol_id_strategy().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

fn symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        4 => leaf_symbol_strategy(),
        1 => leaf_symbol_strategy().prop_map(|s| Symbol::Optional(Box::new(s))),
        1 => leaf_symbol_strategy().prop_map(|s| Symbol::Repeat(Box::new(s))),
        1 => leaf_symbol_strategy().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
        1 => prop::collection::vec(leaf_symbol_strategy(), 1..=4).prop_map(Symbol::Choice),
        1 => prop::collection::vec(leaf_symbol_strategy(), 1..=4).prop_map(Symbol::Sequence),
    ]
}

fn rhs_strategy(max_len: usize) -> impl Strategy<Value = Vec<Symbol>> {
    prop::collection::vec(symbol_strategy(), 0..=max_len)
}

fn field_mapping_strategy(rhs_len: usize) -> impl Strategy<Value = Vec<(FieldId, usize)>> {
    if rhs_len == 0 {
        return Just(vec![]).boxed();
    }
    prop::collection::vec((field_id_strategy(), 0..rhs_len), 0..=rhs_len.min(5)).boxed()
}

fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        symbol_id_strategy(),
        rhs_strategy(6),
        proptest::option::of(precedence_kind_strategy()),
        proptest::option::of(associativity_strategy()),
        production_id_strategy(),
    )
        .prop_flat_map(|(lhs, rhs, prec, assoc, prod_id)| {
            let rhs_len = rhs.len();
            field_mapping_strategy(rhs_len).prop_map(move |fields| Rule {
                lhs,
                rhs: rhs.clone(),
                precedence: prec,
                associativity: assoc,
                fields,
                production_id: prod_id,
            })
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // ── 1. Rule creation ─────────────────────────────────────────────────

    #[test]
    fn rule_creation_lhs_preserved(lhs in 1u16..500) {
        let rule = make_rule(lhs, vec![term(1)]);
        prop_assert_eq!(rule.lhs, SymbolId(lhs));
    }

    #[test]
    fn rule_creation_rhs_length(rhs in rhs_strategy(10)) {
        let len = rhs.len();
        let rule = make_rule(1, rhs);
        prop_assert_eq!(rule.rhs.len(), len);
    }

    #[test]
    fn rule_creation_defaults_no_precedence(lhs in 1u16..100) {
        let rule = make_rule(lhs, vec![]);
        prop_assert!(rule.precedence.is_none());
        prop_assert!(rule.associativity.is_none());
        prop_assert!(rule.fields.is_empty());
    }

    #[test]
    fn rule_creation_production_id(prod in production_id_strategy()) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: prod,
        };
        prop_assert_eq!(rule.production_id, prod);
    }

    // ── 2. Rule name access (via Grammar.rule_names) ─────────────────────

    #[test]
    fn rule_name_lookup_via_grammar(id in 1u16..200) {
        let mut g = Grammar::new("test".into());
        let name = format!("rule_{id}");
        g.rule_names.insert(SymbolId(id), name.clone());
        let found = g.find_symbol_by_name(&name);
        prop_assert_eq!(found, Some(SymbolId(id)));
    }

    #[test]
    fn rule_name_missing_returns_none(id in 1u16..200) {
        let g = Grammar::new("test".into());
        let found = g.find_symbol_by_name(&format!("nonexistent_{id}"));
        prop_assert!(found.is_none());
    }

    #[test]
    fn rule_name_distinguishes_ids(a in 1u16..100, b in 100u16..200) {
        let mut g = Grammar::new("test".into());
        g.rule_names.insert(SymbolId(a), format!("alpha{a}"));
        g.rule_names.insert(SymbolId(b), format!("beta{b}"));
        prop_assert_eq!(g.find_symbol_by_name(&format!("alpha{a}")), Some(SymbolId(a)));
        prop_assert_eq!(g.find_symbol_by_name(&format!("beta{b}")), Some(SymbolId(b)));
    }

    // ── 3. Rule symbols ──────────────────────────────────────────────────

    #[test]
    fn rule_rhs_terminals_preserved(ids in prop::collection::vec(1u16..300, 1..=8)) {
        let rhs: Vec<Symbol> = ids.iter().map(|&i| term(i)).collect();
        let rule = make_rule(1, rhs.clone());
        for i in 0..ids.len() {
            prop_assert_eq!(&rule.rhs[i], &rhs[i]);
        }
    }

    #[test]
    fn rule_rhs_nonterminals_preserved(ids in prop::collection::vec(1u16..300, 1..=8)) {
        let rhs: Vec<Symbol> = ids.iter().map(|&i| nonterm(i)).collect();
        let rule = make_rule(1, rhs.clone());
        for i in 0..ids.len() {
            prop_assert_eq!(&rule.rhs[i], &rhs[i]);
        }
    }

    #[test]
    fn rule_rhs_mixed_symbols(n in 1usize..6) {
        let rhs: Vec<Symbol> = (0..n)
            .map(|i| if i % 2 == 0 { term(i as u16 + 1) } else { nonterm(i as u16 + 1) })
            .collect();
        let rule = make_rule(1, rhs.clone());
        prop_assert_eq!(rule.rhs.len(), n);
        for i in 0..n {
            prop_assert_eq!(&rule.rhs[i], &rhs[i]);
        }
    }

    #[test]
    fn rule_rhs_optional_wrapper(id in symbol_id_strategy()) {
        let sym = Symbol::Optional(Box::new(term(id.0)));
        let rule = make_rule(1, vec![sym.clone()]);
        prop_assert_eq!(&rule.rhs[0], &sym);
    }

    #[test]
    fn rule_rhs_repeat_wrapper(id in symbol_id_strategy()) {
        let sym = Symbol::Repeat(Box::new(nonterm(id.0)));
        let rule = make_rule(1, vec![sym.clone()]);
        prop_assert_eq!(&rule.rhs[0], &sym);
    }

    // ── 4. Rule fields ───────────────────────────────────────────────────

    #[test]
    fn rule_fields_count_preserved(
        n_rhs in 1usize..8,
        n_fields in 0usize..5,
    ) {
        let rhs: Vec<Symbol> = (0..n_rhs).map(|i| term(i as u16 + 1)).collect();
        let fields: Vec<(FieldId, usize)> = (0..n_fields.min(n_rhs))
            .map(|i| (FieldId(i as u16), i))
            .collect();
        let rule = Rule {
            lhs: SymbolId(1),
            rhs,
            precedence: None,
            associativity: None,
            fields: fields.clone(),
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.fields.len(), fields.len());
    }

    #[test]
    fn rule_fields_position_values(field_ids in prop::collection::vec(0u16..200, 1..=5)) {
        let rhs_len = field_ids.len();
        let rhs: Vec<Symbol> = (0..rhs_len).map(|i| term(i as u16 + 1)).collect();
        let fields: Vec<(FieldId, usize)> = field_ids
            .iter()
            .enumerate()
            .map(|(pos, &fid)| (FieldId(fid), pos))
            .collect();
        let rule = Rule {
            lhs: SymbolId(1),
            rhs,
            precedence: None,
            associativity: None,
            fields: fields.clone(),
            production_id: ProductionId(0),
        };
        for i in 0..fields.len() {
            prop_assert_eq!(rule.fields[i].0, fields[i].0);
            prop_assert_eq!(rule.fields[i].1, fields[i].1);
        }
    }

    #[test]
    fn rule_empty_fields_for_empty_rhs(lhs in symbol_id_strategy()) {
        let rule = Rule {
            lhs,
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert!(rule.fields.is_empty());
        prop_assert!(rule.rhs.is_empty());
    }

    // ── 5. Rule serde roundtrip ──────────────────────────────────────────

    #[test]
    fn rule_json_roundtrip(rule in rule_strategy()) {
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn rule_postcard_roundtrip(rule in rule_strategy()) {
        let bytes = postcard::to_stdvec(&rule).unwrap();
        let decoded: Rule = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn rule_json_pretty_roundtrip(rule in rule_strategy()) {
        let json = serde_json::to_string_pretty(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn rule_serde_preserves_lhs(lhs in symbol_id_strategy()) {
        let rule = Rule {
            lhs,
            rhs: vec![term(1)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.lhs, lhs);
    }

    #[test]
    fn rule_serde_preserves_fields(
        n_rhs in 1usize..6,
        n_fields in 1usize..4,
    ) {
        let rhs: Vec<Symbol> = (0..n_rhs).map(|i| term(i as u16 + 1)).collect();
        let fields: Vec<(FieldId, usize)> = (0..n_fields.min(n_rhs))
            .map(|i| (FieldId(i as u16 + 10), i))
            .collect();
        let rule = Rule {
            lhs: SymbolId(1),
            rhs,
            precedence: None,
            associativity: None,
            fields: fields.clone(),
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.fields, &fields);
    }

    // ── 6. Rule with precedence ──────────────────────────────────────────

    #[test]
    fn rule_static_precedence_preserved(level in -100i16..100) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.precedence, Some(PrecedenceKind::Static(level)));
    }

    #[test]
    fn rule_dynamic_precedence_preserved(level in -100i16..100) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Dynamic(level)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(level)));
    }

    #[test]
    fn rule_precedence_roundtrip(prec in precedence_kind_strategy()) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2), nonterm(3)],
            precedence: Some(prec),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.precedence, Some(prec));
    }

    #[test]
    fn rule_associativity_roundtrip(assoc in associativity_strategy()) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: Some(assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.associativity, Some(assoc));
    }

    #[test]
    fn rule_precedence_and_assoc_combined(
        prec in precedence_kind_strategy(),
        assoc in associativity_strategy(),
    ) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![nonterm(2), term(3), nonterm(4)],
            precedence: Some(prec),
            associativity: Some(assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.precedence, Some(prec));
        prop_assert_eq!(rule.associativity, Some(assoc));
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    // ── 7. Rule with alias (via AliasSequence on Grammar) ────────────────

    #[test]
    fn alias_sequence_length_preserved(n in 1usize..10) {
        let aliases: Vec<Option<String>> = (0..n)
            .map(|i| if i % 2 == 0 { Some(format!("alias_{i}")) } else { None })
            .collect();
        let seq = AliasSequence { aliases: aliases.clone() };
        prop_assert_eq!(seq.aliases.len(), n);
        for i in 0..n {
            prop_assert_eq!(&seq.aliases[i], &aliases[i]);
        }
    }

    #[test]
    fn alias_sequence_roundtrip(n in 1usize..8) {
        let aliases: Vec<Option<String>> = (0..n)
            .map(|i| if i % 3 == 0 { Some(format!("a{i}")) } else { None })
            .collect();
        let seq = AliasSequence { aliases };
        let json = serde_json::to_string(&seq).unwrap();
        let decoded: AliasSequence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&seq, &decoded);
    }

    #[test]
    fn rule_with_alias_in_grammar(prod_id in 0u16..50) {
        let mut g = Grammar::new("alias_test".into());
        g.rule_names.insert(SymbolId(1), "expr".into());
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(10)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        };
        g.add_rule(rule);
        g.alias_sequences.insert(
            ProductionId(prod_id),
            AliasSequence { aliases: vec![Some("expression".into())] },
        );
        prop_assert!(g.alias_sequences.contains_key(&ProductionId(prod_id)));
        let seq = &g.alias_sequences[&ProductionId(prod_id)];
        prop_assert_eq!(seq.aliases[0].as_deref(), Some("expression"));
    }

    // ── 8. Rule equality ─────────────────────────────────────────────────

    #[test]
    fn rule_eq_reflexive(rule in rule_strategy()) {
        prop_assert_eq!(&rule, &rule);
    }

    #[test]
    fn rule_eq_symmetric(rule in rule_strategy()) {
        let cloned = rule.clone();
        prop_assert_eq!(&rule, &cloned);
        prop_assert_eq!(&cloned, &rule);
    }

    #[test]
    fn rule_ne_different_lhs(a in 1u16..250, b in 250u16..500) {
        let r1 = make_rule(a, vec![term(1)]);
        let r2 = make_rule(b, vec![term(1)]);
        prop_assert_ne!(&r1, &r2);
    }

    #[test]
    fn rule_ne_different_rhs(a in 1u16..250, b in 250u16..500) {
        let r1 = make_rule(1, vec![term(a)]);
        let r2 = make_rule(1, vec![term(b)]);
        prop_assert_ne!(&r1, &r2);
    }

    #[test]
    fn rule_ne_different_precedence(a in -100i16..0, b in 1i16..100) {
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Static(a)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Static(b)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_ne!(&r1, &r2);
    }

    #[test]
    fn rule_ne_different_production_id(a in 0u16..250, b in 250u16..500) {
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(a),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(b),
        };
        prop_assert_ne!(&r1, &r2);
    }

    #[test]
    fn rule_clone_then_mutate_lhs(rule in rule_strategy()) {
        let mut cloned = rule.clone();
        cloned.lhs = SymbolId(9999);
        // Original should be unchanged
        prop_assert_ne!(rule.lhs, SymbolId(9999));
        prop_assert_eq!(cloned.lhs, SymbolId(9999));
    }

    // ── 9. Rule alternatives (multiple rules same LHS) ───────────────────

    #[test]
    fn grammar_multiple_rules_same_lhs(n in 2usize..8) {
        let mut g = Grammar::new("alt_test".into());
        for i in 0..n {
            g.add_rule(Rule {
                lhs: SymbolId(1),
                rhs: vec![term(i as u16 + 10)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let rules = g.get_rules_for_symbol(SymbolId(1)).unwrap();
        prop_assert_eq!(rules.len(), n);
    }

    #[test]
    fn grammar_alternatives_order_preserved(n in 2usize..6) {
        let mut g = Grammar::new("order_test".into());
        for i in 0..n {
            g.add_rule(Rule {
                lhs: SymbolId(5),
                rhs: vec![term(i as u16 + 100)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let rules = g.get_rules_for_symbol(SymbolId(5)).unwrap();
        for i in 0..n {
            prop_assert_eq!(&rules[i].rhs[0], &term(i as u16 + 100));
        }
    }

    #[test]
    fn grammar_get_rules_unknown_symbol_returns_none(id in 500u16..1000) {
        let g = Grammar::new("empty".into());
        prop_assert!(g.get_rules_for_symbol(SymbolId(id)).is_none());
    }

    #[test]
    fn grammar_all_rules_counts_across_symbols(
        n_a in 1usize..4,
        n_b in 1usize..4,
    ) {
        let mut g = Grammar::new("count_test".into());
        for i in 0..n_a {
            g.add_rule(Rule {
                lhs: SymbolId(1),
                rhs: vec![term(i as u16 + 10)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        for i in 0..n_b {
            g.add_rule(Rule {
                lhs: SymbolId(2),
                rhs: vec![nonterm(i as u16 + 20)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId((n_a + i) as u16),
            });
        }
        prop_assert_eq!(g.all_rules().count(), n_a + n_b);
    }

    // ── 10. Clone/Debug/PartialEq extended ───────────────────────────────

    #[test]
    fn rule_debug_contains_lhs_value(lhs in 1u16..500) {
        let rule = make_rule(lhs, vec![term(1)]);
        let dbg = format!("{:?}", rule);
        let expected = format!("SymbolId({})", lhs);
        prop_assert!(dbg.contains(&expected));
    }

    #[test]
    fn rule_debug_contains_production_id(pid in 0u16..500) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        };
        let dbg = format!("{:?}", rule);
        let expected = format!("ProductionId({})", pid);
        prop_assert!(dbg.contains(&expected));
    }

    #[test]
    fn rule_clone_independence_rhs(rule in rule_strategy()) {
        let mut cloned = rule.clone();
        cloned.rhs.push(Symbol::Epsilon);
        prop_assert_ne!(rule.rhs.len(), cloned.rhs.len());
    }

    #[test]
    fn rule_clone_independence_fields(rule in rule_strategy()) {
        let mut cloned = rule.clone();
        cloned.fields.push((FieldId(9999), 0));
        prop_assert_ne!(rule.fields.len(), cloned.fields.len());
    }

    #[test]
    fn rule_eq_transitive(rule in rule_strategy()) {
        let a = rule.clone();
        let b = a.clone();
        let c = b.clone();
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &c);
        prop_assert_eq!(&a, &c);
    }

    #[test]
    fn rule_ne_different_associativity(
        a_assoc in associativity_strategy(),
        b_assoc in associativity_strategy(),
    ) {
        prop_assume!(a_assoc != b_assoc);
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: Some(a_assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: Some(b_assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_ne!(&r1, &r2);
    }

    // ── 11. Determinism (construction → inspection roundtrip) ────────────

    #[test]
    fn rule_construct_inspect_all_fields(
        lhs in symbol_id_strategy(),
        prod in production_id_strategy(),
        prec in precedence_kind_strategy(),
        assoc in associativity_strategy(),
    ) {
        let rhs = vec![term(10), nonterm(20), term(30)];
        let fields = vec![(FieldId(0), 0), (FieldId(1), 2)];
        let rule = Rule {
            lhs,
            rhs: rhs.clone(),
            precedence: Some(prec),
            associativity: Some(assoc),
            fields: fields.clone(),
            production_id: prod,
        };
        prop_assert_eq!(rule.lhs, lhs);
        prop_assert_eq!(&rule.rhs, &rhs);
        prop_assert_eq!(rule.precedence, Some(prec));
        prop_assert_eq!(rule.associativity, Some(assoc));
        prop_assert_eq!(&rule.fields, &fields);
        prop_assert_eq!(rule.production_id, prod);
    }

    #[test]
    fn rule_rhs_order_deterministic(ids in prop::collection::vec(1u16..300, 2..=8)) {
        let rhs1: Vec<Symbol> = ids.iter().map(|&i| term(i)).collect();
        let rhs2: Vec<Symbol> = ids.iter().map(|&i| term(i)).collect();
        let r1 = make_rule(1, rhs1);
        let r2 = make_rule(1, rhs2);
        prop_assert_eq!(&r1.rhs, &r2.rhs);
    }

    #[test]
    fn rule_field_mapping_order_deterministic(n in 1usize..6) {
        let fields: Vec<(FieldId, usize)> = (0..n).map(|i| (FieldId(i as u16), i)).collect();
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: (0..n).map(|i| term(i as u16 + 1)).collect(),
            precedence: None,
            associativity: None,
            fields: fields.clone(),
            production_id: ProductionId(0),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: (0..n).map(|i| term(i as u16 + 1)).collect(),
            precedence: None,
            associativity: None,
            fields: fields.clone(),
            production_id: ProductionId(0),
        };
        prop_assert_eq!(&r1, &r2);
        for i in 0..n {
            prop_assert_eq!(r1.fields[i], r2.fields[i]);
        }
    }

    // ── 12. Additional symbol variants ───────────────────────────────────

    #[test]
    fn rule_rhs_repeat_one_preserved(id in symbol_id_strategy()) {
        let sym = Symbol::RepeatOne(Box::new(term(id.0)));
        let rule = make_rule(1, vec![sym.clone()]);
        prop_assert_eq!(&rule.rhs[0], &sym);
    }

    #[test]
    fn rule_rhs_external_symbol_preserved(id in symbol_id_strategy()) {
        let sym = Symbol::External(id);
        let rule = make_rule(1, vec![sym.clone()]);
        prop_assert_eq!(&rule.rhs[0], &sym);
    }

    #[test]
    fn rule_rhs_choice_symbol_preserved(
        ids in prop::collection::vec(1u16..100, 2..=4),
    ) {
        let choices: Vec<Symbol> = ids.iter().map(|&i| term(i)).collect();
        let sym = Symbol::Choice(choices);
        let rule = make_rule(1, vec![sym.clone()]);
        prop_assert_eq!(&rule.rhs[0], &sym);
    }

    #[test]
    fn rule_rhs_sequence_symbol_preserved(
        ids in prop::collection::vec(1u16..100, 2..=4),
    ) {
        let seq: Vec<Symbol> = ids.iter().map(|&i| nonterm(i)).collect();
        let sym = Symbol::Sequence(seq);
        let rule = make_rule(1, vec![sym.clone()]);
        prop_assert_eq!(&rule.rhs[0], &sym);
    }

    #[test]
    fn rule_empty_rhs_is_valid(lhs in symbol_id_strategy()) {
        let rule = make_rule(lhs.0, vec![]);
        prop_assert!(rule.rhs.is_empty());
        prop_assert_eq!(rule.lhs, lhs);
    }

    #[test]
    fn rule_rhs_epsilon_symbol(lhs in symbol_id_strategy()) {
        let rule = make_rule(lhs.0, vec![Symbol::Epsilon]);
        prop_assert_eq!(rule.rhs.len(), 1);
        prop_assert_eq!(&rule.rhs[0], &Symbol::Epsilon);
    }

    // ── 13. Precedence discrimination ────────────────────────────────────

    #[test]
    fn rule_static_vs_dynamic_differ(level in -100i16..100) {
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Dynamic(level)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_ne!(&r1, &r2);
    }

    #[test]
    fn rule_none_precedence_vs_some_differ(level in -100i16..100) {
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: Some(PrecedenceKind::Static(level)),
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_ne!(&r1, &r2);
    }

    // ── 14. Associativity discrimination ─────────────────────────────────

    #[test]
    fn rule_assoc_some_none_vs_absent_differ(_dummy in 0u8..1) {
        let r1 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: Some(Associativity::None),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let r2 = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_ne!(&r1, &r2);
    }

    // ── 15. Production ID edge cases ─────────────────────────────────────

    #[test]
    fn rule_production_id_zero_valid(_dummy in 0u8..1) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.production_id, ProductionId(0));
    }

    #[test]
    fn rule_production_id_max_value(_dummy in 0u8..1) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(u16::MAX),
        };
        prop_assert_eq!(rule.production_id, ProductionId(u16::MAX));
    }

    // ── 16. Clone full rule roundtrip ────────────────────────────────────

    #[test]
    fn rule_clone_equals_original(rule in rule_strategy()) {
        let cloned = rule.clone();
        prop_assert_eq!(cloned.lhs, rule.lhs);
        prop_assert_eq!(&cloned.rhs, &rule.rhs);
        prop_assert_eq!(cloned.precedence, rule.precedence);
        prop_assert_eq!(cloned.associativity, rule.associativity);
        prop_assert_eq!(&cloned.fields, &rule.fields);
        prop_assert_eq!(cloned.production_id, rule.production_id);
    }
}
