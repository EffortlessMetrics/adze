#![allow(clippy::needless_range_loop)]

//! Property-based tests for Symbol enum in adze-ir.

use adze_ir::{Symbol, SymbolId};
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

/// Deeply nested symbol strategy (depth 4-5).
fn deep_symbol_strategy() -> impl Strategy<Value = Symbol> {
    leaf_symbol_strategy().prop_recursive(5, 32, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..=3).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..=3).prop_map(Symbol::Sequence),
        ]
    })
}

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

/// Count the total number of nodes in a symbol tree.
fn node_count(sym: &Symbol) -> usize {
    match sym {
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon => 1,
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            1 + node_count(inner)
        }
        Symbol::Choice(children) | Symbol::Sequence(children) => {
            1 + children.iter().map(node_count).sum::<usize>()
        }
    }
}

/// Compute the depth of a symbol tree.
fn depth(sym: &Symbol) -> usize {
    match sym {
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon => 0,
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            1 + depth(inner)
        }
        Symbol::Choice(children) | Symbol::Sequence(children) => {
            1 + children.iter().map(depth).max().unwrap_or(0)
        }
    }
}

/// Classify whether a symbol is a leaf variant.
fn is_leaf(sym: &Symbol) -> bool {
    matches!(
        sym,
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon
    )
}

/// Classify whether a symbol is a terminal-like variant.
fn is_terminal_variant(sym: &Symbol) -> bool {
    matches!(sym, Symbol::Terminal(_))
}

/// Classify whether a symbol is a nonterminal-like variant.
fn is_nonterminal_variant(sym: &Symbol) -> bool {
    matches!(sym, Symbol::NonTerminal(_))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    // 1. Clone preserves equality
    #[test]
    fn clone_preserves_equality(sym in symbol_strategy()) {
        let cloned = sym.clone();
        prop_assert_eq!(&sym, &cloned);
    }

    // 2. Debug output is non-empty
    #[test]
    fn debug_output_is_nonempty(sym in symbol_strategy()) {
        let debug_str = format!("{:?}", sym);
        prop_assert!(!debug_str.is_empty());
    }

    // 3. Terminal and NonTerminal are mutually exclusive
    #[test]
    fn terminal_nonterminal_mutually_exclusive(sym in symbol_strategy()) {
        prop_assert!(!(is_terminal_variant(&sym) && is_nonterminal_variant(&sym)));
    }

    // 4. Epsilon is neither terminal nor nonterminal
    #[test]
    fn epsilon_is_not_terminal_or_nonterminal(_dummy in 0..1u8) {
        let eps = Symbol::Epsilon;
        prop_assert!(!is_terminal_variant(&eps));
        prop_assert!(!is_nonterminal_variant(&eps));
    }

    // 5. Serde JSON roundtrip preserves structure
    #[test]
    fn serde_json_roundtrip(sym in symbol_strategy()) {
        let json = serde_json::to_string(&sym).unwrap();
        let deserialized: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&sym, &deserialized);
    }

    // 6. Hash is consistent across clones
    #[test]
    fn hash_consistent_with_clone(sym in symbol_strategy()) {
        let cloned = sym.clone();
        prop_assert_eq!(hash_of(&sym), hash_of(&cloned));
    }

    // 7. Equal symbols have equal hashes
    #[test]
    fn equal_implies_equal_hash(a in symbol_strategy(), b in symbol_strategy()) {
        if a == b {
            prop_assert_eq!(hash_of(&a), hash_of(&b));
        }
    }

    // 8. Ordering is reflexive
    #[test]
    fn ord_reflexive(sym in symbol_strategy()) {
        prop_assert!(sym <= sym);
        prop_assert!(sym >= sym);
    }

    // 9. Ordering is antisymmetric
    #[test]
    fn ord_antisymmetric(a in symbol_strategy(), b in symbol_strategy()) {
        if a <= b && b <= a {
            prop_assert_eq!(a, b);
        }
    }

    // 10. Ordering is transitive (use same value to guarantee chain)
    #[test]
    fn ord_total(a in symbol_strategy(), b in symbol_strategy()) {
        prop_assert!(a <= b || b <= a);
    }

    // 11. Node count is always >= 1
    #[test]
    fn node_count_at_least_one(sym in symbol_strategy()) {
        prop_assert!(node_count(&sym) >= 1);
    }

    // 12. Leaf symbols have node count == 1
    #[test]
    fn leaf_node_count_is_one(sym in leaf_symbol_strategy()) {
        prop_assert_eq!(node_count(&sym), 1);
    }

    // 13. Leaf symbols have depth == 0
    #[test]
    fn leaf_depth_is_zero(sym in leaf_symbol_strategy()) {
        prop_assert_eq!(depth(&sym), 0);
    }

    // 14. Wrapping in Optional increases depth by 1
    #[test]
    fn optional_increases_depth(sym in symbol_strategy()) {
        let wrapped = Symbol::Optional(Box::new(sym.clone()));
        prop_assert_eq!(depth(&wrapped), depth(&sym) + 1);
    }

    // 15. Wrapping in Repeat increases depth by 1
    #[test]
    fn repeat_increases_depth(sym in symbol_strategy()) {
        let wrapped = Symbol::Repeat(Box::new(sym.clone()));
        prop_assert_eq!(depth(&wrapped), depth(&sym) + 1);
    }

    // 16. Wrapping in RepeatOne increases depth by 1
    #[test]
    fn repeat_one_increases_depth(sym in symbol_strategy()) {
        let wrapped = Symbol::RepeatOne(Box::new(sym.clone()));
        prop_assert_eq!(depth(&wrapped), depth(&sym) + 1);
    }

    // 17. Wrapping in Optional increases node count by 1
    #[test]
    fn optional_increases_node_count(sym in symbol_strategy()) {
        let wrapped = Symbol::Optional(Box::new(sym.clone()));
        prop_assert_eq!(node_count(&wrapped), node_count(&sym) + 1);
    }

    // 18. Choice node count equals 1 + sum of children counts
    #[test]
    fn choice_node_count(children in prop::collection::vec(leaf_symbol_strategy(), 1..=5)) {
        let total_children: usize = children.iter().map(node_count).sum();
        let choice = Symbol::Choice(children);
        prop_assert_eq!(node_count(&choice), 1 + total_children);
    }

    // 19. Sequence node count equals 1 + sum of children counts
    #[test]
    fn sequence_node_count(children in prop::collection::vec(leaf_symbol_strategy(), 1..=5)) {
        let total_children: usize = children.iter().map(node_count).sum();
        let seq = Symbol::Sequence(children);
        prop_assert_eq!(node_count(&seq), 1 + total_children);
    }

    // 20. Deep symbols still roundtrip through serde
    #[test]
    fn deep_serde_roundtrip(sym in deep_symbol_strategy()) {
        let json = serde_json::to_string(&sym).unwrap();
        let deserialized: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&sym, &deserialized);
    }

    // 21. Debug output contains variant name
    #[test]
    fn debug_contains_variant_name(id in any::<u16>()) {
        let terminal = Symbol::Terminal(SymbolId(id));
        let nonterminal = Symbol::NonTerminal(SymbolId(id));
        let external = Symbol::External(SymbolId(id));
        let eps = Symbol::Epsilon;

        let t_dbg = format!("{terminal:?}");
        let nt_dbg = format!("{nonterminal:?}");
        let ext_dbg = format!("{external:?}");
        let eps_dbg = format!("{eps:?}");

        prop_assert!(t_dbg.contains("Terminal"));
        prop_assert!(nt_dbg.contains("NonTerminal"));
        prop_assert!(ext_dbg.contains("External"));
        prop_assert!(eps_dbg.contains("Epsilon"));
    }

    // 22. PartialEq is symmetric
    #[test]
    fn partial_eq_symmetric(a in symbol_strategy(), b in symbol_strategy()) {
        prop_assert_eq!(a == b, b == a);
    }

    // 23. Terminal with same id equals itself
    #[test]
    fn terminal_same_id_eq(id in any::<u16>()) {
        let a = Symbol::Terminal(SymbolId(id));
        let b = Symbol::Terminal(SymbolId(id));
        prop_assert_eq!(a, b);
    }

    // 24. Terminal and NonTerminal with same id are not equal
    #[test]
    fn terminal_ne_nonterminal_same_id(id in any::<u16>()) {
        let t = Symbol::Terminal(SymbolId(id));
        let nt = Symbol::NonTerminal(SymbolId(id));
        prop_assert_ne!(t, nt);
    }

    // 25. Wrapping preserves inner equality
    #[test]
    fn optional_wrapping_preserves_inner(sym in symbol_strategy()) {
        let wrapped = Symbol::Optional(Box::new(sym.clone()));
        if let Symbol::Optional(inner) = &wrapped {
            prop_assert_eq!(&sym, inner.as_ref());
        } else {
            prop_assert!(false, "Expected Optional");
        }
    }

    // 26. Repeat wrapping preserves inner equality
    #[test]
    fn repeat_wrapping_preserves_inner(sym in symbol_strategy()) {
        let wrapped = Symbol::Repeat(Box::new(sym.clone()));
        if let Symbol::Repeat(inner) = &wrapped {
            prop_assert_eq!(&sym, inner.as_ref());
        } else {
            prop_assert!(false, "Expected Repeat");
        }
    }

    // 27. RepeatOne wrapping preserves inner equality
    #[test]
    fn repeat_one_wrapping_preserves_inner(sym in symbol_strategy()) {
        let wrapped = Symbol::RepeatOne(Box::new(sym.clone()));
        if let Symbol::RepeatOne(inner) = &wrapped {
            prop_assert_eq!(&sym, inner.as_ref());
        } else {
            prop_assert!(false, "Expected RepeatOne");
        }
    }

    // 28. Choice preserves children
    #[test]
    fn choice_preserves_children(children in prop::collection::vec(symbol_strategy(), 1..=4)) {
        let choice = Symbol::Choice(children.clone());
        if let Symbol::Choice(inner) = &choice {
            prop_assert_eq!(inner.len(), children.len());
            for i in 0..children.len() {
                prop_assert_eq!(&inner[i], &children[i]);
            }
        } else {
            prop_assert!(false, "Expected Choice");
        }
    }

    // 29. Sequence preserves children
    #[test]
    fn sequence_preserves_children(children in prop::collection::vec(symbol_strategy(), 1..=4)) {
        let seq = Symbol::Sequence(children.clone());
        if let Symbol::Sequence(inner) = &seq {
            prop_assert_eq!(inner.len(), children.len());
            for i in 0..children.len() {
                prop_assert_eq!(&inner[i], &children[i]);
            }
        } else {
            prop_assert!(false, "Expected Sequence");
        }
    }

    // 30. Serde roundtrip preserves hash
    #[test]
    fn serde_roundtrip_preserves_hash(sym in symbol_strategy()) {
        let json = serde_json::to_string(&sym).unwrap();
        let deserialized: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(hash_of(&sym), hash_of(&deserialized));
    }

    // 31. Double clone equals original
    #[test]
    fn double_clone_eq(sym in symbol_strategy()) {
        let c1 = sym.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&sym, &c2);
    }

    // 32. is_leaf consistency with wrapper types
    #[test]
    fn wrapper_is_not_leaf(sym in symbol_strategy()) {
        let opt = Symbol::Optional(Box::new(sym.clone()));
        let rep = Symbol::Repeat(Box::new(sym.clone()));
        let rep1 = Symbol::RepeatOne(Box::new(sym));
        prop_assert!(!is_leaf(&opt));
        prop_assert!(!is_leaf(&rep));
        prop_assert!(!is_leaf(&rep1));
    }

    // 33. Nested Optional unwraps correctly
    #[test]
    fn nested_optional_depth(sym in leaf_symbol_strategy(), n in 1usize..=4) {
        let mut current = sym;
        for _ in 0..n {
            current = Symbol::Optional(Box::new(current));
        }
        prop_assert_eq!(depth(&current), n);
    }

    // 34. Serde pretty roundtrip
    #[test]
    fn serde_pretty_roundtrip(sym in symbol_strategy()) {
        let json = serde_json::to_string_pretty(&sym).unwrap();
        let deserialized: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&sym, &deserialized);
    }

    // 35. External variant is distinct from Terminal and NonTerminal
    #[test]
    fn external_distinct_from_terminal_and_nonterminal(id in any::<u16>()) {
        let t = Symbol::Terminal(SymbolId(id));
        let nt = Symbol::NonTerminal(SymbolId(id));
        let ext = Symbol::External(SymbolId(id));
        prop_assert_ne!(t, ext.clone());
        prop_assert_ne!(nt, ext);
    }
}
