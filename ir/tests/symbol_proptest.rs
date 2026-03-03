#![allow(clippy::needless_range_loop)]

//! Property-based tests for Symbol enum variants in adze-ir.

use adze_ir::{
    Associativity, FieldId, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
};
use proptest::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    any::<u16>().prop_map(SymbolId)
}

/// Leaf-only symbols (no recursive nesting).
fn leaf_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        symbol_id_strategy().prop_map(Symbol::Terminal),
        symbol_id_strategy().prop_map(Symbol::NonTerminal),
        symbol_id_strategy().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

/// Recursive symbol strategy (up to depth 3).
fn symbol_strategy() -> impl Strategy<Value = Symbol> {
    leaf_symbol_strategy().prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..=4).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..=4).prop_map(Symbol::Sequence),
        ]
    })
}

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        symbol_id_strategy(),
        prop::collection::vec(symbol_strategy(), 0..=4),
        any::<u16>(),
    )
        .prop_map(|(lhs, rhs, prod)| Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod),
        })
}

/// Return the discriminant tag of a Symbol variant.
fn discriminant(sym: &Symbol) -> u8 {
    match sym {
        Symbol::Terminal(_) => 0,
        Symbol::NonTerminal(_) => 1,
        Symbol::External(_) => 2,
        Symbol::Optional(_) => 3,
        Symbol::Repeat(_) => 4,
        Symbol::RepeatOne(_) => 5,
        Symbol::Choice(_) => 6,
        Symbol::Sequence(_) => 7,
        Symbol::Epsilon => 8,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    // 1. Terminal constructible with random SymbolId
    #[test]
    fn terminal_constructible(id in any::<u16>()) {
        let s = Symbol::Terminal(SymbolId(id));
        prop_assert!(matches!(s, Symbol::Terminal(_)));
    }

    // 2. NonTerminal constructible with random SymbolId
    #[test]
    fn nonterminal_constructible(id in any::<u16>()) {
        let s = Symbol::NonTerminal(SymbolId(id));
        prop_assert!(matches!(s, Symbol::NonTerminal(_)));
    }

    // 3. External constructible with random SymbolId
    #[test]
    fn external_constructible(id in any::<u16>()) {
        let s = Symbol::External(SymbolId(id));
        prop_assert!(matches!(s, Symbol::External(_)));
    }

    // 4. Optional constructible around any symbol
    #[test]
    fn optional_constructible(inner in symbol_strategy()) {
        let s = Symbol::Optional(Box::new(inner));
        prop_assert!(matches!(s, Symbol::Optional(_)));
    }

    // 5. Repeat constructible around any symbol
    #[test]
    fn repeat_constructible(inner in symbol_strategy()) {
        let s = Symbol::Repeat(Box::new(inner));
        prop_assert!(matches!(s, Symbol::Repeat(_)));
    }

    // 6. RepeatOne constructible around any symbol
    #[test]
    fn repeat_one_constructible(inner in symbol_strategy()) {
        let s = Symbol::RepeatOne(Box::new(inner));
        prop_assert!(matches!(s, Symbol::RepeatOne(_)));
    }

    // 7. Choice constructible with random children
    #[test]
    fn choice_constructible(children in prop::collection::vec(symbol_strategy(), 1..=5)) {
        let s = Symbol::Choice(children);
        prop_assert!(matches!(s, Symbol::Choice(_)));
    }

    // 8. Sequence constructible with random children
    #[test]
    fn sequence_constructible(children in prop::collection::vec(symbol_strategy(), 1..=5)) {
        let s = Symbol::Sequence(children);
        prop_assert!(matches!(s, Symbol::Sequence(_)));
    }

    // 9. Epsilon is always Epsilon
    #[test]
    fn epsilon_constructible(_dummy in 0..1u8) {
        let s = Symbol::Epsilon;
        prop_assert!(matches!(s, Symbol::Epsilon));
    }

    // 10. Serde JSON roundtrip for Terminal
    #[test]
    fn serde_roundtrip_terminal(id in any::<u16>()) {
        let s = Symbol::Terminal(SymbolId(id));
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 11. Serde JSON roundtrip for NonTerminal
    #[test]
    fn serde_roundtrip_nonterminal(id in any::<u16>()) {
        let s = Symbol::NonTerminal(SymbolId(id));
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 12. Serde JSON roundtrip for External
    #[test]
    fn serde_roundtrip_external(id in any::<u16>()) {
        let s = Symbol::External(SymbolId(id));
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 13. Serde JSON roundtrip for Optional
    #[test]
    fn serde_roundtrip_optional(inner in symbol_strategy()) {
        let s = Symbol::Optional(Box::new(inner));
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 14. Serde JSON roundtrip for Repeat
    #[test]
    fn serde_roundtrip_repeat(inner in symbol_strategy()) {
        let s = Symbol::Repeat(Box::new(inner));
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 15. Serde JSON roundtrip for Epsilon
    #[test]
    fn serde_roundtrip_epsilon(_dummy in 0..1u8) {
        let s = Symbol::Epsilon;
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 16. Clone preserves equality for any variant
    #[test]
    fn clone_preserves_equality(sym in symbol_strategy()) {
        let cloned = sym.clone();
        prop_assert_eq!(&sym, &cloned);
    }

    // 17. Debug is non-empty for any variant
    #[test]
    fn debug_is_nonempty(sym in symbol_strategy()) {
        let dbg = format!("{:?}", sym);
        prop_assert!(!dbg.is_empty());
    }

    // 18. Debug contains variant name for leaf types
    #[test]
    fn debug_contains_variant_name(id in any::<u16>()) {
        assert!(format!("{:?}", Symbol::Terminal(SymbolId(id))).contains("Terminal"));
        assert!(format!("{:?}", Symbol::NonTerminal(SymbolId(id))).contains("NonTerminal"));
        assert!(format!("{:?}", Symbol::External(SymbolId(id))).contains("External"));
        assert!(format!("{:?}", Symbol::Epsilon).contains("Epsilon"));
    }

    // 19. Symbol usable in Rule RHS
    #[test]
    fn symbol_in_rule_rhs(rule in rule_strategy()) {
        // Rule constructed successfully; verify RHS symbols are accessible
        for sym in &rule.rhs {
            let _ = format!("{:?}", sym);
        }
        prop_assert!(rule.rhs.len() <= 4);
    }

    // 20. Rule with single symbol RHS roundtrips through serde
    #[test]
    fn rule_with_symbol_rhs_serde(sym in symbol_strategy(), lhs_id in any::<u16>()) {
        let rule = Rule {
            lhs: SymbolId(lhs_id),
            rhs: vec![sym],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let de: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, de);
    }

    // 21. Nested Optional(Repeat(Terminal)) roundtrip
    #[test]
    fn nested_optional_repeat_terminal(id in any::<u16>()) {
        let s = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(
            SymbolId(id),
        )))));
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 22. Nested RepeatOne(Choice([Terminal, NonTerminal]))
    #[test]
    fn nested_repeat_one_choice(a in any::<u16>(), b in any::<u16>()) {
        let s = Symbol::RepeatOne(Box::new(Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(a)),
            Symbol::NonTerminal(SymbolId(b)),
        ])));
        let cloned = s.clone();
        prop_assert_eq!(&s, &cloned);
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 23. Nested Sequence(Optional(External))
    #[test]
    fn nested_sequence_optional_external(id in any::<u16>()) {
        let s = Symbol::Sequence(vec![
            Symbol::Optional(Box::new(Symbol::External(SymbolId(id)))),
            Symbol::Epsilon,
        ]);
        let json = serde_json::to_string(&s).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, de);
    }

    // 24. Same-variant same-id symbols are equal
    #[test]
    fn symbol_equality_same_variant(id in any::<u16>()) {
        prop_assert_eq!(Symbol::Terminal(SymbolId(id)), Symbol::Terminal(SymbolId(id)));
        prop_assert_eq!(Symbol::NonTerminal(SymbolId(id)), Symbol::NonTerminal(SymbolId(id)));
        prop_assert_eq!(Symbol::External(SymbolId(id)), Symbol::External(SymbolId(id)));
    }

    // 25. PartialEq is symmetric
    #[test]
    fn partial_eq_symmetric(a in symbol_strategy(), b in symbol_strategy()) {
        prop_assert_eq!(a == b, b == a);
    }

    // 26. Terminal != NonTerminal even with same id
    #[test]
    fn terminal_ne_nonterminal(id in any::<u16>()) {
        prop_assert_ne!(Symbol::Terminal(SymbolId(id)), Symbol::NonTerminal(SymbolId(id)));
    }

    // 27. Terminal != External even with same id
    #[test]
    fn terminal_ne_external(id in any::<u16>()) {
        prop_assert_ne!(Symbol::Terminal(SymbolId(id)), Symbol::External(SymbolId(id)));
    }

    // 28. NonTerminal != External even with same id
    #[test]
    fn nonterminal_ne_external(id in any::<u16>()) {
        prop_assert_ne!(Symbol::NonTerminal(SymbolId(id)), Symbol::External(SymbolId(id)));
    }

    // 29. Epsilon != any id-bearing variant
    #[test]
    fn epsilon_ne_id_variants(id in any::<u16>()) {
        prop_assert_ne!(Symbol::Epsilon, Symbol::Terminal(SymbolId(id)));
        prop_assert_ne!(Symbol::Epsilon, Symbol::NonTerminal(SymbolId(id)));
        prop_assert_ne!(Symbol::Epsilon, Symbol::External(SymbolId(id)));
    }

    // 30. Different discriminants always unequal (discrimination)
    #[test]
    fn different_discriminants_unequal(a in symbol_strategy(), b in symbol_strategy()) {
        if discriminant(&a) != discriminant(&b) {
            prop_assert_ne!(a, b);
        }
    }

    // 31. Hash consistent with equality
    #[test]
    fn hash_consistent_with_eq(a in symbol_strategy(), b in symbol_strategy()) {
        if a == b {
            prop_assert_eq!(hash_of(&a), hash_of(&b));
        }
    }

    // 32. Hash stable across clones
    #[test]
    fn hash_stable_across_clone(sym in symbol_strategy()) {
        prop_assert_eq!(hash_of(&sym), hash_of(&sym.clone()));
    }

    // 33. Serde roundtrip preserves hash
    #[test]
    fn serde_roundtrip_preserves_hash(sym in symbol_strategy()) {
        let json = serde_json::to_string(&sym).unwrap();
        let de: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(hash_of(&sym), hash_of(&de));
    }

    // 34. Rule RHS with mixed variants roundtrips
    #[test]
    fn rule_mixed_rhs_roundtrip(
        a in any::<u16>(),
        b in any::<u16>(),
        c in any::<u16>(),
    ) {
        let rule = Rule {
            lhs: SymbolId(a),
            rhs: vec![
                Symbol::Terminal(SymbolId(a)),
                Symbol::NonTerminal(SymbolId(b)),
                Symbol::Optional(Box::new(Symbol::External(SymbolId(c)))),
                Symbol::Epsilon,
            ],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![(FieldId(0), 1)],
            production_id: ProductionId(0),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let de: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, de);
    }

    // 35. Double-wrapping Optional preserves inner structure
    #[test]
    fn double_optional_preserves_inner(sym in symbol_strategy()) {
        let inner = Symbol::Optional(Box::new(sym.clone()));
        let outer = Symbol::Optional(Box::new(inner.clone()));
        if let Symbol::Optional(o) = &outer {
            prop_assert_eq!(o.as_ref(), &inner);
            if let Symbol::Optional(i) = o.as_ref() {
                prop_assert_eq!(i.as_ref(), &sym);
            } else {
                prop_assert!(false, "expected inner Optional");
            }
        } else {
            prop_assert!(false, "expected outer Optional");
        }
    }
}
