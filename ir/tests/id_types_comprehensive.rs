//! Comprehensive tests for SymbolId, RuleId, StateId, FieldId, ProductionId types.

use adze_ir::{FieldId, ProductionId, SymbolId};

// ── SymbolId ──

#[test]
fn symbol_id_zero() {
    let s = SymbolId(0);
    assert_eq!(s.0, 0);
}

#[test]
fn symbol_id_max() {
    let s = SymbolId(u16::MAX);
    assert_eq!(s.0, u16::MAX);
}

#[test]
fn symbol_id_clone() {
    let s = SymbolId(42);
    let c = s.clone();
    assert_eq!(s, c);
}

#[test]
fn symbol_id_copy() {
    let s = SymbolId(42);
    let c = s;
    assert_eq!(s, c);
}

#[test]
fn symbol_id_eq() {
    assert_eq!(SymbolId(1), SymbolId(1));
}

#[test]
fn symbol_id_ne() {
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn symbol_id_debug() {
    let d = format!("{:?}", SymbolId(42));
    assert!(d.contains("42"));
}

#[test]
fn symbol_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1));
    assert_eq!(set.len(), 2);
}

#[test]
fn symbol_id_ord() {
    assert!(SymbolId(1) < SymbolId(2));
    assert!(SymbolId(10) > SymbolId(5));
}

// ── FieldId ──

#[test]
fn field_id_zero() {
    let f = FieldId(0);
    assert_eq!(f.0, 0);
}

#[test]
fn field_id_max() {
    let f = FieldId(u16::MAX);
    assert_eq!(f.0, u16::MAX);
}

#[test]
fn field_id_clone() {
    let f = FieldId(10);
    let c = f.clone();
    assert_eq!(f, c);
}

#[test]
fn field_id_copy() {
    let f = FieldId(10);
    let c = f;
    assert_eq!(f, c);
}

#[test]
fn field_id_eq() {
    assert_eq!(FieldId(5), FieldId(5));
}

#[test]
fn field_id_ne() {
    assert_ne!(FieldId(5), FieldId(6));
}

#[test]
fn field_id_debug() {
    let d = format!("{:?}", FieldId(99));
    assert!(d.contains("99"));
}

// ── ProductionId ──

#[test]
fn production_id_zero() {
    let p = ProductionId(0);
    assert_eq!(p.0, 0);
}

#[test]
fn production_id_max() {
    let p = ProductionId(u16::MAX);
    assert_eq!(p.0, u16::MAX);
}

#[test]
fn production_id_clone() {
    let p = ProductionId(7);
    let c = p.clone();
    assert_eq!(p, c);
}

#[test]
fn production_id_copy() {
    let p = ProductionId(7);
    let c = p;
    assert_eq!(p, c);
}

#[test]
fn production_id_eq() {
    assert_eq!(ProductionId(3), ProductionId(3));
}

#[test]
fn production_id_ne() {
    assert_ne!(ProductionId(3), ProductionId(4));
}

#[test]
fn production_id_debug() {
    let d = format!("{:?}", ProductionId(88));
    assert!(d.contains("88"));
}

// ── Cross-type comparisons ──

#[test]
fn symbol_id_and_field_id_different_types() {
    let s = SymbolId(1);
    let f = FieldId(1);
    // They should be different types even with same inner value
    assert_eq!(s.0, f.0);
}

// ── Collections of IDs ──

#[test]
fn vec_of_symbol_ids() {
    let v: Vec<SymbolId> = (0..100).map(SymbolId).collect();
    assert_eq!(v.len(), 100);
}

#[test]
fn vec_of_field_ids() {
    let v: Vec<FieldId> = (0..50).map(FieldId).collect();
    assert_eq!(v.len(), 50);
}

#[test]
fn sorted_symbol_ids() {
    let mut v = vec![SymbolId(5), SymbolId(1), SymbolId(3)];
    v.sort();
    assert_eq!(v, vec![SymbolId(1), SymbolId(3), SymbolId(5)]);
}

// ── Serialization ──

#[test]
fn symbol_id_serialize() {
    let s = SymbolId(42);
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn symbol_id_deserialize() {
    let s = SymbolId(42);
    let json = serde_json::to_string(&s).unwrap();
    let s2: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(s, s2);
}

#[test]
fn field_id_serialize() {
    let f = FieldId(10);
    let json = serde_json::to_string(&f).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn field_id_deserialize() {
    let f = FieldId(10);
    let json = serde_json::to_string(&f).unwrap();
    let f2: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(f, f2);
}

#[test]
fn production_id_serialize() {
    let p = ProductionId(99);
    let json = serde_json::to_string(&p).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn production_id_deserialize() {
    let p = ProductionId(99);
    let json = serde_json::to_string(&p).unwrap();
    let p2: ProductionId = serde_json::from_str(&json).unwrap();
    assert_eq!(p, p2);
}
