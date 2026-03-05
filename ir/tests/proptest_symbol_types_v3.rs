//! Proptest-based tests for `Symbol` and `SymbolId` types.

use adze_ir::{Symbol, SymbolId};
use proptest::prelude::*;
use std::collections::{BTreeSet, HashSet};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn terminal_strategy() -> impl Strategy<Value = Symbol> {
    any::<u16>().prop_map(|v| Symbol::Terminal(SymbolId(v)))
}

fn nonterminal_strategy() -> impl Strategy<Value = Symbol> {
    any::<u16>().prop_map(|v| Symbol::NonTerminal(SymbolId(v)))
}

/// Flat symbol (no recursion) for fast tests.
fn flat_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        terminal_strategy(),
        nonterminal_strategy(),
        any::<u16>().prop_map(|v| Symbol::External(SymbolId(v))),
        Just(Symbol::Epsilon),
    ]
}

// ---------------------------------------------------------------------------
// 1. SymbolId proptest — construction, ordering, equality, hash (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbolid_roundtrip_value(v in any::<u16>()) {
        let id = SymbolId(v);
        prop_assert_eq!(id.0, v);
    }

    #[test]
    fn symbolid_equality_reflexive(v in any::<u16>()) {
        let id = SymbolId(v);
        prop_assert_eq!(id, id);
    }

    #[test]
    fn symbolid_equality_same_value(v in any::<u16>()) {
        prop_assert_eq!(SymbolId(v), SymbolId(v));
    }

    #[test]
    fn symbolid_inequality_different_values(a in any::<u16>(), b in any::<u16>()) {
        prop_assume!(a != b);
        prop_assert_ne!(SymbolId(a), SymbolId(b));
    }

    #[test]
    fn symbolid_ordering_consistent_with_u16(a in any::<u16>(), b in any::<u16>()) {
        prop_assert_eq!(SymbolId(a).cmp(&SymbolId(b)), a.cmp(&b));
    }

    #[test]
    fn symbolid_partial_ord_agrees_with_ord(a in any::<u16>(), b in any::<u16>()) {
        prop_assert_eq!(
            SymbolId(a).partial_cmp(&SymbolId(b)),
            Some(SymbolId(a).cmp(&SymbolId(b)))
        );
    }

    #[test]
    fn symbolid_hash_deterministic(v in any::<u16>()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        SymbolId(v).hash(&mut h1);
        SymbolId(v).hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn symbolid_hash_equal_values_equal_hash(v in any::<u16>()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let id_a = SymbolId(v);
        let id_b = SymbolId(v);
        let hash = |id: &SymbolId| {
            let mut h = DefaultHasher::new();
            id.hash(&mut h);
            h.finish()
        };
        prop_assert_eq!(hash(&id_a), hash(&id_b));
    }
}

// ---------------------------------------------------------------------------
// 2. Symbol variant proptest — terminal, nonterminal, kind, id extraction (8)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_terminal_holds_id(v in any::<u16>()) {
        let sym = Symbol::Terminal(SymbolId(v));
        match sym {
            Symbol::Terminal(id) => prop_assert_eq!(id.0, v),
            _ => prop_assert!(false, "expected Terminal"),
        }
    }

    #[test]
    fn symbol_nonterminal_holds_id(v in any::<u16>()) {
        let sym = Symbol::NonTerminal(SymbolId(v));
        match sym {
            Symbol::NonTerminal(id) => prop_assert_eq!(id.0, v),
            _ => prop_assert!(false, "expected NonTerminal"),
        }
    }

    #[test]
    fn symbol_external_holds_id(v in any::<u16>()) {
        let sym = Symbol::External(SymbolId(v));
        match sym {
            Symbol::External(id) => prop_assert_eq!(id.0, v),
            _ => prop_assert!(false, "expected External"),
        }
    }

    #[test]
    fn symbol_terminal_is_not_epsilon(v in any::<u16>()) {
        let sym = Symbol::Terminal(SymbolId(v));
        prop_assert!(!matches!(sym, Symbol::Epsilon));
    }

    #[test]
    fn symbol_nonterminal_is_not_terminal(v in any::<u16>()) {
        let sym = Symbol::NonTerminal(SymbolId(v));
        prop_assert!(!matches!(sym, Symbol::Terminal(_)));
    }

    #[test]
    fn symbol_terminal_not_nonterminal(v in any::<u16>()) {
        let sym = Symbol::Terminal(SymbolId(v));
        prop_assert!(!matches!(sym, Symbol::NonTerminal(_)));
    }

    #[test]
    fn symbol_clone_equals_original(sym in flat_symbol_strategy()) {
        let cloned = sym.clone();
        prop_assert_eq!(sym, cloned);
    }

    #[test]
    fn symbol_debug_not_empty(sym in flat_symbol_strategy()) {
        let dbg = format!("{:?}", sym);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3. Symbol predicates via pattern matching (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn predicate_terminal_matches(v in any::<u16>()) {
        let sym = Symbol::Terminal(SymbolId(v));
        prop_assert!(matches!(sym, Symbol::Terminal(_)));
        prop_assert!(!matches!(sym, Symbol::NonTerminal(_)));
        prop_assert!(!matches!(sym, Symbol::Epsilon));
    }

    #[test]
    fn predicate_nonterminal_matches(v in any::<u16>()) {
        let sym = Symbol::NonTerminal(SymbolId(v));
        prop_assert!(matches!(sym, Symbol::NonTerminal(_)));
        prop_assert!(!matches!(sym, Symbol::Terminal(_)));
        prop_assert!(!matches!(sym, Symbol::Epsilon));
    }

    #[test]
    fn predicate_epsilon_matches(_v in any::<u16>()) {
        let sym = Symbol::Epsilon;
        prop_assert!(matches!(sym, Symbol::Epsilon));
        prop_assert!(!matches!(sym, Symbol::Terminal(_)));
        prop_assert!(!matches!(sym, Symbol::NonTerminal(_)));
    }

    #[test]
    fn predicate_external_matches(v in any::<u16>()) {
        let sym = Symbol::External(SymbolId(v));
        prop_assert!(matches!(sym, Symbol::External(_)));
        prop_assert!(!matches!(sym, Symbol::Terminal(_)));
        prop_assert!(!matches!(sym, Symbol::Epsilon));
    }

    #[test]
    fn predicate_exactly_one_leaf_variant(sym in flat_symbol_strategy()) {
        let is_t = matches!(sym, Symbol::Terminal(_));
        let is_nt = matches!(sym, Symbol::NonTerminal(_));
        let is_ext = matches!(sym, Symbol::External(_));
        let is_eps = matches!(sym, Symbol::Epsilon);
        // Exactly one must be true for flat symbols.
        let count = [is_t, is_nt, is_ext, is_eps].iter().filter(|&&b| b).count();
        prop_assert_eq!(count, 1);
    }
}

// ---------------------------------------------------------------------------
// 4. Symbol serialization proptest — JSON roundtrip (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_terminal_roundtrip(v in any::<u16>()) {
        let sym = Symbol::Terminal(SymbolId(v));
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn serde_nonterminal_roundtrip(v in any::<u16>()) {
        let sym = Symbol::NonTerminal(SymbolId(v));
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn serde_epsilon_roundtrip(_v in any::<u16>()) {
        let sym = Symbol::Epsilon;
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }

    #[test]
    fn serde_symbolid_roundtrip(v in any::<u16>()) {
        let id = SymbolId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: SymbolId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn serde_flat_symbol_roundtrip(sym in flat_symbol_strategy()) {
        let json = serde_json::to_string(&sym).unwrap();
        let back: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, back);
    }
}

// ---------------------------------------------------------------------------
// 5. SymbolId arithmetic proptest (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbolid_zero_is_minimum(v in any::<u16>()) {
        prop_assert!(SymbolId(0) <= SymbolId(v));
    }

    #[test]
    fn symbolid_max_is_maximum(v in any::<u16>()) {
        prop_assert!(SymbolId(v) <= SymbolId(u16::MAX));
    }

    #[test]
    fn symbolid_consecutive_ordering(v in 0u16..u16::MAX) {
        prop_assert!(SymbolId(v) < SymbolId(v + 1));
    }

    #[test]
    fn symbolid_wrapping_inner(v in any::<u16>()) {
        let wrapped = v.wrapping_add(1);
        let id = SymbolId(wrapped);
        prop_assert_eq!(id.0, wrapped);
    }

    #[test]
    fn symbolid_btree_insertion_order(vals in prop::collection::vec(any::<u16>(), 1..50)) {
        let set: BTreeSet<SymbolId> = vals.iter().copied().map(SymbolId).collect();
        let sorted: Vec<_> = set.into_iter().collect();
        for window in sorted.windows(2) {
            prop_assert!(window[0] < window[1]);
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Regular Symbol tests (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn symbol_terminal_equality() {
    assert_eq!(Symbol::Terminal(SymbolId(0)), Symbol::Terminal(SymbolId(0)));
}

#[test]
fn symbol_terminal_inequality() {
    assert_ne!(Symbol::Terminal(SymbolId(0)), Symbol::Terminal(SymbolId(1)));
}

#[test]
fn symbol_different_variants_not_equal() {
    assert_ne!(
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(0))
    );
}

#[test]
fn symbol_epsilon_unique() {
    assert_eq!(Symbol::Epsilon, Symbol::Epsilon);
}

#[test]
fn symbol_epsilon_not_terminal() {
    assert_ne!(Symbol::Epsilon, Symbol::Terminal(SymbolId(0)));
}

#[test]
fn symbol_optional_wraps_terminal() {
    let inner = Symbol::Terminal(SymbolId(42));
    let opt = Symbol::Optional(Box::new(inner.clone()));
    if let Symbol::Optional(boxed) = opt {
        assert_eq!(*boxed, Symbol::Terminal(SymbolId(42)));
    } else {
        panic!("expected Optional");
    }
}

#[test]
fn symbol_repeat_wraps_nonterminal() {
    let inner = Symbol::NonTerminal(SymbolId(7));
    let rep = Symbol::Repeat(Box::new(inner.clone()));
    if let Symbol::Repeat(boxed) = rep {
        assert_eq!(*boxed, Symbol::NonTerminal(SymbolId(7)));
    } else {
        panic!("expected Repeat");
    }
}

#[test]
fn symbol_sequence_preserves_order() {
    let seq = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Epsilon,
    ]);
    if let Symbol::Sequence(items) = seq {
        assert_eq!(items.len(), 3);
        assert!(matches!(items[0], Symbol::Terminal(SymbolId(1))));
        assert!(matches!(items[1], Symbol::NonTerminal(SymbolId(2))));
        assert!(matches!(items[2], Symbol::Epsilon));
    } else {
        panic!("expected Sequence");
    }
}

// ---------------------------------------------------------------------------
// 7. Regular SymbolId tests (5 tests)
// ---------------------------------------------------------------------------

#[test]
fn symbolid_zero() {
    assert_eq!(SymbolId(0).0, 0);
}

#[test]
fn symbolid_max() {
    assert_eq!(SymbolId(u16::MAX).0, u16::MAX);
}

#[test]
fn symbolid_display_format() {
    assert_eq!(format!("{}", SymbolId(42)), "Symbol(42)");
}

#[test]
fn symbolid_display_zero() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
}

#[test]
fn symbolid_hashset_dedup() {
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    assert_eq!(set.len(), 2);
}

// ---------------------------------------------------------------------------
// 8. Edge cases (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn edge_empty_choice() {
    let sym = Symbol::Choice(vec![]);
    if let Symbol::Choice(items) = sym {
        assert!(items.is_empty());
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn edge_empty_sequence() {
    let sym = Symbol::Sequence(vec![]);
    if let Symbol::Sequence(items) = sym {
        assert!(items.is_empty());
    } else {
        panic!("expected Sequence");
    }
}

#[test]
fn edge_nested_optional() {
    let inner = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(0))));
    let outer = Symbol::Optional(Box::new(inner));
    assert!(matches!(outer, Symbol::Optional(_)));
}

#[test]
fn edge_deep_repeat_one() {
    let sym = Symbol::RepeatOne(Box::new(Symbol::RepeatOne(Box::new(Symbol::Epsilon))));
    if let Symbol::RepeatOne(a) = sym {
        if let Symbol::RepeatOne(b) = *a {
            assert_eq!(*b, Symbol::Epsilon);
        } else {
            panic!("expected inner RepeatOne");
        }
    } else {
        panic!("expected outer RepeatOne");
    }
}

#[test]
fn edge_symbolid_boundary_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(u16::MAX - 1) < SymbolId(u16::MAX));
}

#[test]
fn edge_choice_with_all_variants() {
    let sym = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(1)),
        Symbol::External(SymbolId(2)),
        Symbol::Epsilon,
    ]);
    if let Symbol::Choice(items) = sym {
        assert_eq!(items.len(), 4);
    } else {
        panic!("expected Choice");
    }
}
