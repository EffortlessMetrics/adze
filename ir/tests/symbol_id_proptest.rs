#![allow(clippy::needless_range_loop)]

use adze_ir::{FieldId, ProductionId, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------
// 1. SymbolId equality — reflexive, symmetric, transitive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_id_eq_reflexive(v in any::<u16>()) {
        let id = SymbolId(v);
        prop_assert_eq!(id, id);
    }

    #[test]
    fn symbol_id_eq_symmetric(a in any::<u16>(), b in any::<u16>()) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        prop_assert_eq!(x == y, y == x);
    }

    #[test]
    fn symbol_id_eq_transitive(v in any::<u16>()) {
        let a = SymbolId(v);
        let b = SymbolId(v);
        let c = SymbolId(v);
        prop_assert_eq!(a, b);
        prop_assert_eq!(b, c);
        prop_assert_eq!(a, c);
    }

    #[test]
    fn symbol_id_ne_for_different_values(a in any::<u16>(), b in any::<u16>()) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        prop_assert_eq!(x == y, a == b);
    }

    // -----------------------------------------------------------------------
    // 2. SymbolId ordering — total order properties
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_ord_antisymmetric(a in any::<u16>(), b in any::<u16>()) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        if x <= y && y <= x {
            prop_assert_eq!(x, y);
        }
    }

    #[test]
    fn symbol_id_ord_transitive(a in any::<u16>(), b in any::<u16>(), c in any::<u16>()) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        let z = SymbolId(c);
        if x <= y && y <= z {
            prop_assert!(x <= z);
        }
    }

    #[test]
    fn symbol_id_ord_total(a in any::<u16>(), b in any::<u16>()) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        prop_assert!(x <= y || y <= x);
    }

    #[test]
    fn symbol_id_ord_consistent_with_inner(a in any::<u16>(), b in any::<u16>()) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        prop_assert_eq!(x.cmp(&y), a.cmp(&b));
    }

    // -----------------------------------------------------------------------
    // 3. SymbolId hashing — equal IDs → equal hashes
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_equal_implies_equal_hash(v in any::<u16>()) {
        let a = SymbolId(v);
        let b = SymbolId(v);
        prop_assert_eq!(hash_of(&a), hash_of(&b));
    }

    // -----------------------------------------------------------------------
    // 4. Serde roundtrip (JSON)
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_json_roundtrip(v in any::<u16>()) {
        let id = SymbolId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: SymbolId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn rule_id_json_roundtrip(v in any::<u16>()) {
        let id = RuleId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: RuleId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn state_id_json_roundtrip(v in any::<u16>()) {
        let id = StateId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: StateId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn field_id_json_roundtrip(v in any::<u16>()) {
        let id = FieldId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: FieldId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    #[test]
    fn production_id_json_roundtrip(v in any::<u16>()) {
        let id = ProductionId(v);
        let json = serde_json::to_string(&id).unwrap();
        let back: ProductionId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, back);
    }

    // -----------------------------------------------------------------------
    // 5. Copy semantics
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_copy_semantics(v in any::<u16>()) {
        let a = SymbolId(v);
        let b = a; // Copy
        let c = a; // still usable after copy
        prop_assert_eq!(a, b);
        prop_assert_eq!(a, c);
    }

    #[test]
    fn all_ids_copy_semantics(v in any::<u16>()) {
        let s = SymbolId(v);  let s2 = s; prop_assert_eq!(s, s2);
        let r = RuleId(v);    let r2 = r; prop_assert_eq!(r, r2);
        let t = StateId(v);   let t2 = t; prop_assert_eq!(t, t2);
        let f = FieldId(v);   let f2 = f; prop_assert_eq!(f, f2);
        let p = ProductionId(v); let p2 = p; prop_assert_eq!(p, p2);
    }

    // -----------------------------------------------------------------------
    // 6. Arithmetic / conversion properties
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_inner_value_preserved(v in any::<u16>()) {
        prop_assert_eq!(SymbolId(v).0, v);
    }

    #[test]
    fn symbol_id_distinct_from_rule_id_by_type(v in any::<u16>()) {
        // Same inner value but different types should serialize differently
        let sid_json = serde_json::to_string(&SymbolId(v)).unwrap();
        let rid_json = serde_json::to_string(&RuleId(v)).unwrap();
        // Both serialize their inner u16 the same way, but deserialization
        // into the wrong type must still produce the same numeric value
        let back_as_symbol: SymbolId = serde_json::from_str(&rid_json).unwrap();
        prop_assert_eq!(back_as_symbol.0, v);
    }

    // -----------------------------------------------------------------------
    // 7. Collection behavior — HashSet, BTreeSet, BTreeMap
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_hashset_dedup(vals in prop::collection::vec(any::<u16>(), 0..64)) {
        let ids: Vec<SymbolId> = vals.iter().map(|&v| SymbolId(v)).collect();
        let set: HashSet<SymbolId> = ids.iter().copied().collect();
        let raw_set: HashSet<u16> = vals.iter().copied().collect();
        prop_assert_eq!(set.len(), raw_set.len());
    }

    #[test]
    fn symbol_id_btreeset_sorted(vals in prop::collection::vec(any::<u16>(), 1..64)) {
        let set: BTreeSet<SymbolId> = vals.iter().map(|&v| SymbolId(v)).collect();
        let sorted: Vec<SymbolId> = set.iter().copied().collect();
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1] < sorted[i]);
        }
    }

    #[test]
    fn symbol_id_btreemap_lookup(vals in prop::collection::vec(any::<u16>(), 1..32)) {
        let map: BTreeMap<SymbolId, u16> = vals.iter().map(|&v| (SymbolId(v), v)).collect();
        for &v in &vals {
            prop_assert!(map.contains_key(&SymbolId(v)));
            prop_assert_eq!(*map.get(&SymbolId(v)).unwrap(), v);
        }
    }

    #[test]
    fn symbol_id_hashmap_lookup(vals in prop::collection::vec(any::<u16>(), 1..32)) {
        let map: HashMap<SymbolId, u16> = vals.iter().map(|&v| (SymbolId(v), v)).collect();
        for &v in &vals {
            prop_assert_eq!(*map.get(&SymbolId(v)).unwrap(), v);
        }
    }

    // -----------------------------------------------------------------------
    // 9. Display / Debug formatting
    // -----------------------------------------------------------------------

    #[test]
    fn symbol_id_display_format(v in any::<u16>()) {
        let id = SymbolId(v);
        prop_assert_eq!(format!("{id}"), format!("Symbol({v})"));
    }

    #[test]
    fn rule_id_display_format(v in any::<u16>()) {
        prop_assert_eq!(format!("{}", RuleId(v)), format!("Rule({v})"));
    }

    #[test]
    fn state_id_display_format(v in any::<u16>()) {
        prop_assert_eq!(format!("{}", StateId(v)), format!("State({v})"));
    }

    #[test]
    fn field_id_display_format(v in any::<u16>()) {
        prop_assert_eq!(format!("{}", FieldId(v)), format!("Field({v})"));
    }

    #[test]
    fn production_id_display_format(v in any::<u16>()) {
        prop_assert_eq!(format!("{}", ProductionId(v)), format!("Production({v})"));
    }

    #[test]
    fn symbol_id_debug_contains_value(v in any::<u16>()) {
        let dbg = format!("{:?}", SymbolId(v));
        prop_assert!(dbg.contains(&v.to_string()));
    }

    // -----------------------------------------------------------------------
    // 10. Cross-type properties for all ID types
    // -----------------------------------------------------------------------

    #[test]
    fn rule_id_eq_and_hash(a in any::<u16>(), b in any::<u16>()) {
        let x = RuleId(a);
        let y = RuleId(b);
        if x == y {
            prop_assert_eq!(hash_of(&x), hash_of(&y));
        }
        prop_assert_eq!(x == y, a == b);
    }

    #[test]
    fn state_id_ord_consistent(a in any::<u16>(), b in any::<u16>()) {
        prop_assert_eq!(StateId(a).cmp(&StateId(b)), a.cmp(&b));
    }

    #[test]
    fn field_id_eq_and_hash(a in any::<u16>(), b in any::<u16>()) {
        let x = FieldId(a);
        let y = FieldId(b);
        if x == y {
            prop_assert_eq!(hash_of(&x), hash_of(&y));
        }
        prop_assert_eq!(x == y, a == b);
    }

    #[test]
    fn production_id_ord_consistent(a in any::<u16>(), b in any::<u16>()) {
        prop_assert_eq!(ProductionId(a).cmp(&ProductionId(b)), a.cmp(&b));
    }
}

// ---------------------------------------------------------------------------
// 8. Edge values (0, u16::MAX)
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_edge_zero() {
    let z = SymbolId(0);
    assert_eq!(z.0, 0);
    assert_eq!(format!("{z}"), "Symbol(0)");
    let rt: SymbolId = serde_json::from_str(&serde_json::to_string(&z).unwrap()).unwrap();
    assert_eq!(z, rt);
}

#[test]
fn symbol_id_edge_max() {
    let m = SymbolId(u16::MAX);
    assert_eq!(m.0, u16::MAX);
    assert_eq!(format!("{m}"), format!("Symbol({})", u16::MAX));
    let rt: SymbolId = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    assert_eq!(m, rt);
}

#[test]
fn all_ids_edge_zero_and_max() {
    for v in [0u16, u16::MAX] {
        let _ = SymbolId(v);
        let _ = RuleId(v);
        let _ = StateId(v);
        let _ = FieldId(v);
        let _ = ProductionId(v);
    }
}

#[test]
fn edge_ordering_min_lt_max() {
    assert!(SymbolId(0) < SymbolId(u16::MAX));
    assert!(RuleId(0) < RuleId(u16::MAX));
    assert!(StateId(0) < StateId(u16::MAX));
    assert!(ProductionId(0) < ProductionId(u16::MAX));
}

#[test]
fn edge_hash_distinct_for_zero_and_max() {
    // While not guaranteed in general, for simple wrappers this should hold
    assert_ne!(hash_of(&SymbolId(0)), hash_of(&SymbolId(u16::MAX)));
}
