#![allow(clippy::needless_range_loop)]

//! Property-based tests for the Rule type in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
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

/// Leaf-only symbol strategy (no recursive nesting).
fn leaf_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        symbol_id_strategy().prop_map(Symbol::Terminal),
        symbol_id_strategy().prop_map(Symbol::NonTerminal),
        symbol_id_strategy().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

/// Symbol strategy with one level of nesting.
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
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ── 1. Rule creation with various rhs lengths ────────────────────────

    #[test]
    fn rule_rhs_length_preserved(rhs in rhs_strategy(10)) {
        let len = rhs.len();
        let rule = make_rule(1, rhs);
        prop_assert_eq!(rule.rhs.len(), len);
    }

    #[test]
    fn rule_rhs_zero_length(_dummy in 0..1i32) {
        let rule = make_rule(1, vec![]);
        prop_assert!(rule.rhs.is_empty());
    }

    #[test]
    fn rule_rhs_single_terminal(id in 1u16..500) {
        let rule = make_rule(1, vec![term(id)]);
        prop_assert_eq!(rule.rhs.len(), 1);
        prop_assert_eq!(&rule.rhs[0], &Symbol::Terminal(SymbolId(id)));
    }

    #[test]
    fn rule_rhs_many_symbols(n in 1usize..20) {
        let rhs: Vec<Symbol> = (0..n).map(|i| term(i as u16 + 1)).collect();
        let rule = make_rule(1, rhs);
        prop_assert_eq!(rule.rhs.len(), n);
    }

    // ── 2. Rule serde roundtrip ──────────────────────────────────────────

    #[test]
    fn rule_json_roundtrip(rule in rule_strategy()) {
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn rule_json_roundtrip_pretty(rule in rule_strategy()) {
        let json = serde_json::to_string_pretty(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn rule_bincode_roundtrip(rule in rule_strategy()) {
        let bytes = bincode::serialize(&rule).unwrap();
        let decoded: Rule = bincode::deserialize(&bytes).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn rule_json_contains_lhs(lhs_id in 1u16..500) {
        let rule = make_rule(lhs_id, vec![term(1)]);
        let json = serde_json::to_string(&rule).unwrap();
        prop_assert!(json.contains(&lhs_id.to_string()));
    }

    // ── 3. Rule clone and equality ───────────────────────────────────────

    #[test]
    fn rule_clone_equals_original(rule in rule_strategy()) {
        let cloned = rule.clone();
        prop_assert_eq!(&rule, &cloned);
    }

    #[test]
    fn rule_clone_is_independent(rule in rule_strategy()) {
        let mut cloned = rule.clone();
        cloned.lhs = SymbolId(9999);
        prop_assert_ne!(rule.lhs, cloned.lhs);
    }

    #[test]
    fn rule_ne_different_lhs(id_a in 1u16..250, id_b in 250u16..500) {
        let a = make_rule(id_a, vec![term(1)]);
        let b = make_rule(id_b, vec![term(1)]);
        prop_assert_ne!(&a, &b);
    }

    #[test]
    fn rule_ne_different_rhs(id_a in 1u16..250, id_b in 250u16..500) {
        let a = make_rule(1, vec![term(id_a)]);
        let b = make_rule(1, vec![term(id_b)]);
        prop_assert_ne!(&a, &b);
    }

    // ── 4. Empty rhs rule ────────────────────────────────────────────────

    #[test]
    fn empty_rhs_serde_roundtrip(lhs in symbol_id_strategy()) {
        let rule = Rule {
            lhs,
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule, &decoded);
    }

    #[test]
    fn empty_rhs_clone(lhs in symbol_id_strategy()) {
        let rule = Rule {
            lhs,
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let cloned = rule.clone();
        prop_assert_eq!(&rule, &cloned);
        prop_assert!(cloned.rhs.is_empty());
    }

    // ── 5. Rule with all symbol types ────────────────────────────────────

    #[test]
    fn rule_preserves_terminal(id in symbol_id_strategy()) {
        let rule = make_rule(1, vec![Symbol::Terminal(id)]);
        prop_assert!(matches!(rule.rhs[0], Symbol::Terminal(sid) if sid == id));
    }

    #[test]
    fn rule_preserves_nonterminal(id in symbol_id_strategy()) {
        let rule = make_rule(1, vec![Symbol::NonTerminal(id)]);
        prop_assert!(matches!(rule.rhs[0], Symbol::NonTerminal(sid) if sid == id));
    }

    #[test]
    fn rule_preserves_external(id in symbol_id_strategy()) {
        let rule = make_rule(1, vec![Symbol::External(id)]);
        prop_assert!(matches!(rule.rhs[0], Symbol::External(sid) if sid == id));
    }

    #[test]
    fn rule_preserves_optional(id in symbol_id_strategy()) {
        let rule = make_rule(1, vec![Symbol::Optional(Box::new(term(id.0)))]);
        match &rule.rhs[0] {
            Symbol::Optional(inner) => prop_assert_eq!(inner.as_ref(), &term(id.0)),
            other => prop_assert!(false, "Expected Optional, got {:?}", other),
        }
    }

    #[test]
    fn rule_preserves_repeat(id in symbol_id_strategy()) {
        let rule = make_rule(1, vec![Symbol::Repeat(Box::new(nonterm(id.0)))]);
        match &rule.rhs[0] {
            Symbol::Repeat(inner) => prop_assert_eq!(inner.as_ref(), &nonterm(id.0)),
            other => prop_assert!(false, "Expected Repeat, got {:?}", other),
        }
    }

    #[test]
    fn rule_preserves_repeat_one(id in symbol_id_strategy()) {
        let rule = make_rule(1, vec![Symbol::RepeatOne(Box::new(term(id.0)))]);
        match &rule.rhs[0] {
            Symbol::RepeatOne(inner) => prop_assert_eq!(inner.as_ref(), &term(id.0)),
            other => prop_assert!(false, "Expected RepeatOne, got {:?}", other),
        }
    }

    #[test]
    fn rule_preserves_choice(ids in prop::collection::vec(1u16..100, 2..=5)) {
        let syms: Vec<Symbol> = ids.iter().map(|&i| term(i)).collect();
        let rule = make_rule(1, vec![Symbol::Choice(syms.clone())]);
        match &rule.rhs[0] {
            Symbol::Choice(inner) => prop_assert_eq!(inner, &syms),
            other => prop_assert!(false, "Expected Choice, got {:?}", other),
        }
    }

    #[test]
    fn rule_preserves_sequence(ids in prop::collection::vec(1u16..100, 2..=5)) {
        let syms: Vec<Symbol> = ids.iter().map(|&i| nonterm(i)).collect();
        let rule = make_rule(1, vec![Symbol::Sequence(syms.clone())]);
        match &rule.rhs[0] {
            Symbol::Sequence(inner) => prop_assert_eq!(inner, &syms),
            other => prop_assert!(false, "Expected Sequence, got {:?}", other),
        }
    }

    #[test]
    fn rule_preserves_epsilon(_dummy in 0..1i32) {
        let rule = make_rule(1, vec![Symbol::Epsilon]);
        prop_assert_eq!(&rule.rhs[0], &Symbol::Epsilon);
    }

    // ── 6. Rule field mappings ───────────────────────────────────────────

    #[test]
    fn rule_field_mappings_preserved(
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
        prop_assert_eq!(&rule.fields, &fields);
    }

    #[test]
    fn rule_field_mappings_roundtrip(
        n_rhs in 1usize..6,
        n_fields in 0usize..4,
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
            fields,
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rule.fields, &decoded.fields);
    }

    #[test]
    fn rule_field_id_values_preserved(field_ids in prop::collection::vec(0u16..200, 1..=5)) {
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

    // ── 7. Rule precedence setting ───────────────────────────────────────

    #[test]
    fn rule_static_precedence(level in -100i16..100) {
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
    fn rule_dynamic_precedence(level in -100i16..100) {
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
    fn rule_associativity_preserved(assoc in associativity_strategy()) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2)],
            precedence: None,
            associativity: Some(assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        prop_assert_eq!(rule.associativity, Some(assoc));
    }

    #[test]
    fn rule_precedence_and_associativity_roundtrip(
        prec in precedence_kind_strategy(),
        assoc in associativity_strategy(),
    ) {
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![term(2), nonterm(3)],
            precedence: Some(prec),
            associativity: Some(assoc),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let decoded: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.precedence, Some(prec));
        prop_assert_eq!(decoded.associativity, Some(assoc));
    }

    // ── 8. Rule in Grammar context ───────────────────────────────────────

    #[test]
    fn grammar_add_rule_retrieval(lhs in 1u16..100) {
        let mut g = Grammar::new("test".into());
        g.rule_names.insert(SymbolId(lhs), format!("rule_{lhs}"));
        let rule = make_rule(lhs, vec![Symbol::Epsilon]);
        g.add_rule(rule.clone());
        let retrieved = g.get_rules_for_symbol(SymbolId(lhs));
        prop_assert!(retrieved.is_some());
        prop_assert_eq!(&retrieved.unwrap()[0], &rule);
    }

    #[test]
    fn grammar_multiple_rules_same_lhs(n in 1usize..6) {
        let mut g = Grammar::new("test".into());
        g.rule_names.insert(SymbolId(1), "start".into());
        for i in 0..n {
            let r = Rule {
                lhs: SymbolId(1),
                rhs: vec![Symbol::Epsilon],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            };
            g.add_rule(r);
        }
        let rules = g.get_rules_for_symbol(SymbolId(1)).unwrap();
        prop_assert_eq!(rules.len(), n);
    }

    #[test]
    fn grammar_all_rules_iterator(n_lhs in 1usize..5, n_alt in 1usize..4) {
        let mut g = Grammar::new("test".into());
        let mut total = 0usize;
        for l in 0..n_lhs {
            let lhs = (l as u16) + 1;
            g.rule_names.insert(SymbolId(lhs), format!("r{lhs}"));
            for a in 0..n_alt {
                g.add_rule(Rule {
                    lhs: SymbolId(lhs),
                    rhs: vec![Symbol::Epsilon],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(total as u16),
                });
                total += 1;
            }
        }
        prop_assert_eq!(g.all_rules().count(), total);
    }

    #[test]
    fn grammar_roundtrip_with_rules(n in 1usize..4) {
        let mut g = Grammar::new("roundtrip".into());
        for i in 0..n {
            let lhs = (i as u16) + 1;
            let tok_id = lhs + 100;
            g.tokens.insert(
                SymbolId(tok_id),
                Token {
                    name: format!("T{tok_id}"),
                    pattern: TokenPattern::String(format!("t{tok_id}")),
                    fragile: false,
                },
            );
            g.rule_names.insert(SymbolId(lhs), format!("rule_{lhs}"));
            g.add_rule(Rule {
                lhs: SymbolId(lhs),
                rhs: vec![Symbol::Terminal(SymbolId(tok_id))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let json = serde_json::to_string(&g).unwrap();
        let decoded: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(g.all_rules().count(), decoded.all_rules().count());
        prop_assert_eq!(&g.name, &decoded.name);
    }

    #[test]
    fn builder_rule_with_precedence_preserved(
        level in -50i16..50,
        assoc in associativity_strategy(),
    ) {
        let g = GrammarBuilder::new("prec_test")
            .token("A", "a")
            .rule_with_precedence("expr", vec!["A"], level, assoc)
            .build();
        let rules: Vec<&Rule> = g.all_rules().collect();
        prop_assert!(!rules.is_empty());
        let r = rules[0];
        prop_assert_eq!(r.precedence, Some(PrecedenceKind::Static(level)));
        prop_assert_eq!(r.associativity, Some(assoc));
    }
}
