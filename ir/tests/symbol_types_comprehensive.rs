//! Comprehensive tests for SymbolId, RuleId, StateId, FieldId, ProductionId,
//! SymbolMetadata, and SymbolRegistry types in the IR crate.

use adze_ir::{
    FieldId, ProductionId, RuleId, StateId, SymbolId, SymbolInfo, SymbolMetadata, SymbolRegistry,
};
use std::collections::{BTreeSet, HashMap, HashSet};

// ===========================================================================
// Helper: reusable metadata constructors
// ===========================================================================

fn terminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    }
}

fn nonterminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    }
}

fn hidden_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: false,
        named: false,
        hidden: true,
        terminal: true,
    }
}

// ===========================================================================
// 1. SymbolId — construction, equality, ordering, hashing, debug, display
// ===========================================================================

#[test]
fn symbol_id_construction_and_inner_access() {
    let id = SymbolId(42);
    assert_eq!(id.0, 42);
}

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_eq!(SymbolId(u16::MAX), SymbolId(u16::MAX));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(99));
    let mut ids = vec![SymbolId(5), SymbolId(1), SymbolId(3)];
    ids.sort();
    assert_eq!(ids, vec![SymbolId(1), SymbolId(3), SymbolId(5)]);
}

#[test]
fn symbol_id_in_btreeset() {
    let mut set = BTreeSet::new();
    set.insert(SymbolId(10));
    set.insert(SymbolId(2));
    set.insert(SymbolId(10));
    assert_eq!(set.len(), 2);
    assert_eq!(*set.iter().next().unwrap(), SymbolId(2));
}

#[test]
fn symbol_id_hashing() {
    let mut set = HashSet::new();
    set.insert(SymbolId(10));
    set.insert(SymbolId(20));
    set.insert(SymbolId(10)); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn symbol_id_as_hashmap_key() {
    let mut map = HashMap::new();
    map.insert(SymbolId(1), "plus");
    map.insert(SymbolId(2), "expr");
    assert_eq!(map.get(&SymbolId(1)), Some(&"plus"));
    assert_eq!(map.get(&SymbolId(99)), None);
}

#[test]
fn symbol_id_debug_format() {
    let dbg = format!("{:?}", SymbolId(7));
    assert!(dbg.contains("SymbolId"));
    assert!(dbg.contains("7"));
}

#[test]
fn symbol_id_display_format() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
    assert_eq!(format!("{}", SymbolId(u16::MAX)), "Symbol(65535)");
}

#[test]
fn symbol_id_copy_semantics() {
    let a = SymbolId(5);
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn symbol_id_boundary_values() {
    let zero = SymbolId(0);
    let max = SymbolId(u16::MAX);
    assert_ne!(zero, max);
    assert!(zero < max);
    // Serde roundtrip at boundaries
    for id in [zero, max] {
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(serde_json::from_str::<SymbolId>(&json).unwrap(), id);
    }
}

// ===========================================================================
// 2. RuleId — same coverage
// ===========================================================================

#[test]
fn rule_id_construction_equality_ordering() {
    assert_eq!(RuleId(0), RuleId(0));
    assert_ne!(RuleId(0), RuleId(1));
    assert!(RuleId(1) < RuleId(2));
    let mut v = vec![RuleId(3), RuleId(1), RuleId(2)];
    v.sort();
    assert_eq!(v, vec![RuleId(1), RuleId(2), RuleId(3)]);
}

#[test]
fn rule_id_hashing_and_display() {
    let mut set = HashSet::new();
    set.insert(RuleId(5));
    set.insert(RuleId(5));
    assert_eq!(set.len(), 1);
    assert_eq!(format!("{}", RuleId(10)), "Rule(10)");
    assert!(format!("{:?}", RuleId(10)).contains("RuleId"));
}

#[test]
fn rule_id_serde_roundtrip() {
    let id = RuleId(u16::MAX);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(serde_json::from_str::<RuleId>(&json).unwrap(), id);
}

// ===========================================================================
// 3. StateId — same coverage
// ===========================================================================

#[test]
fn state_id_construction_equality_ordering() {
    assert_eq!(StateId(0), StateId(0));
    assert_ne!(StateId(0), StateId(1));
    assert!(StateId(10) > StateId(9));
    let mut v = vec![StateId(30), StateId(10), StateId(20)];
    v.sort();
    assert_eq!(v, vec![StateId(10), StateId(20), StateId(30)]);
}

#[test]
fn state_id_hashing_and_display() {
    let mut set = HashSet::new();
    set.insert(StateId(255));
    set.insert(StateId(255));
    assert_eq!(set.len(), 1);
    assert_eq!(format!("{}", StateId(255)), "State(255)");
    assert!(format!("{:?}", StateId(0)).contains("StateId"));
}

#[test]
fn state_id_serde_roundtrip() {
    let id = StateId(0);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(serde_json::from_str::<StateId>(&json).unwrap(), id);
}

// ===========================================================================
// 4. FieldId — same coverage (note: NO PartialOrd/Ord)
// ===========================================================================

#[test]
fn field_id_construction_and_equality() {
    assert_eq!(FieldId(0), FieldId(0));
    assert_ne!(FieldId(0), FieldId(1));
}

#[test]
fn field_id_hashing_and_display() {
    let mut set = HashSet::new();
    set.insert(FieldId(3));
    set.insert(FieldId(3));
    assert_eq!(set.len(), 1);
    assert_eq!(format!("{}", FieldId(3)), "Field(3)");
    assert!(format!("{:?}", FieldId(3)).contains("FieldId"));
}

#[test]
fn field_id_serde_roundtrip() {
    let id = FieldId(55);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(serde_json::from_str::<FieldId>(&json).unwrap(), id);
}

// ===========================================================================
// 5. ProductionId — same coverage
// ===========================================================================

#[test]
fn production_id_construction_equality_ordering() {
    assert_eq!(ProductionId(0), ProductionId(0));
    assert_ne!(ProductionId(0), ProductionId(1));
    assert!(ProductionId(1) < ProductionId(2));
    let mut v = vec![ProductionId(9), ProductionId(1), ProductionId(5)];
    v.sort();
    assert_eq!(v, vec![ProductionId(1), ProductionId(5), ProductionId(9)]);
}

#[test]
fn production_id_hashing_and_display() {
    let mut set = HashSet::new();
    set.insert(ProductionId(7));
    set.insert(ProductionId(7));
    assert_eq!(set.len(), 1);
    assert_eq!(format!("{}", ProductionId(7)), "Production(7)");
    assert!(format!("{:?}", ProductionId(7)).contains("ProductionId"));
}

#[test]
fn production_id_serde_roundtrip() {
    let id = ProductionId(200);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(serde_json::from_str::<ProductionId>(&json).unwrap(), id);
}

// ===========================================================================
// 6. SymbolMetadata — construction, field access, equality, clone, serde
// ===========================================================================

#[test]
fn symbol_metadata_field_access() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    assert!(meta.visible);
    assert!(meta.named);
    assert!(!meta.hidden);
    assert!(!meta.terminal);
}

#[test]
fn symbol_metadata_equality() {
    let a = terminal_meta();
    let b = terminal_meta();
    assert_eq!(a, b);
    assert_ne!(terminal_meta(), nonterminal_meta());
}

#[test]
fn symbol_metadata_all_false() {
    let meta = SymbolMetadata {
        visible: false,
        named: false,
        hidden: false,
        terminal: false,
    };
    assert!(!meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(!meta.terminal);
}

#[test]
fn symbol_metadata_all_true() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: true,
        terminal: true,
    };
    assert!(meta.visible && meta.named && meta.hidden && meta.terminal);
}

#[test]
fn symbol_metadata_clone_is_independent() {
    let a = hidden_meta();
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn symbol_metadata_debug_format() {
    let dbg = format!("{:?}", terminal_meta());
    assert!(dbg.contains("SymbolMetadata"));
    assert!(dbg.contains("visible"));
}

#[test]
fn symbol_metadata_serde_roundtrip() {
    let meta = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: true,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let deser: SymbolMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta, deser);
}

// ===========================================================================
// 7. SymbolRegistry — new(), pre-registered "end", CRUD, iteration
// ===========================================================================

#[test]
fn registry_new_preregisters_end() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());
}

#[test]
fn registry_default_equals_new() {
    let a = SymbolRegistry::new();
    let b = SymbolRegistry::default();
    assert_eq!(a, b);
}

#[test]
fn registry_register_returns_incremental_ids() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("number", terminal_meta());
    let id2 = reg.register("expr", nonterminal_meta());
    assert_eq!(id1, SymbolId(1));
    assert_eq!(id2, SymbolId(2));
    assert_eq!(reg.len(), 3); // end + number + expr
}

#[test]
fn registry_get_id_and_get_name_roundtrip() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("plus", terminal_meta());
    assert_eq!(reg.get_id("plus"), Some(id));
    assert_eq!(reg.get_name(id), Some("plus"));
}

#[test]
fn registry_get_metadata() {
    let mut reg = SymbolRegistry::new();
    let meta = nonterminal_meta();
    let id = reg.register("stmt", meta);
    assert_eq!(reg.get_metadata(id), Some(meta));
}

#[test]
fn registry_contains_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    assert!(reg.contains_id(id));
    assert!(reg.contains_id(SymbolId(0))); // "end"
    assert!(!reg.contains_id(SymbolId(999)));
}

#[test]
fn registry_get_nonexistent_name_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

#[test]
fn registry_get_nonexistent_id_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(999)), None);
    assert_eq!(reg.get_metadata(SymbolId(999)), None);
}

#[test]
fn registry_duplicate_name_returns_same_id_and_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("ident", terminal_meta());
    let id2 = reg.register("ident", nonterminal_meta());
    assert_eq!(id1, id2);
    assert_eq!(reg.len(), 2); // end + ident (not duplicated)
    // Metadata should be updated to the second registration
    assert_eq!(reg.get_metadata(id1), Some(nonterminal_meta()));
}

#[test]
fn registry_iter_yields_all_symbols_in_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("alpha", terminal_meta());
    reg.register("beta", nonterminal_meta());

    let items: Vec<(&str, SymbolInfo)> = reg.iter().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].0, "end");
    assert_eq!(items[0].1.id, SymbolId(0));
    assert_eq!(items[1].0, "alpha");
    assert_eq!(items[1].1.id, SymbolId(1));
    assert_eq!(items[2].0, "beta");
    assert_eq!(items[2].1.id, SymbolId(2));
}

#[test]
fn registry_to_index_map() {
    let mut reg = SymbolRegistry::new();
    reg.register("x", terminal_meta());
    reg.register("y", terminal_meta());
    let index_map = reg.to_index_map();
    // "end"=SymbolId(0) -> index 0, "x"=SymbolId(1) -> index 1, "y"=SymbolId(2) -> index 2
    assert_eq!(index_map[&SymbolId(0)], 0);
    assert_eq!(index_map[&SymbolId(1)], 1);
    assert_eq!(index_map[&SymbolId(2)], 2);
}

#[test]
fn registry_to_symbol_map() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    let sym_map = reg.to_symbol_map();
    assert_eq!(sym_map[&0], SymbolId(0));
    assert_eq!(sym_map[&1], SymbolId(1));
}

#[test]
fn registry_serde_roundtrip() {
    let mut reg = SymbolRegistry::new();
    reg.register("foo", terminal_meta());
    reg.register("bar", nonterminal_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let deser: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, deser);
}

#[test]
fn registry_deterministic_id_assignment() {
    let mut reg1 = SymbolRegistry::new();
    let mut reg2 = SymbolRegistry::new();
    for name in ["number", "plus", "minus", "expr"] {
        let meta = if name == "expr" {
            nonterminal_meta()
        } else {
            terminal_meta()
        };
        reg1.register(name, meta);
        reg2.register(name, meta);
    }
    for name in ["number", "plus", "minus", "expr"] {
        assert_eq!(reg1.get_id(name), reg2.get_id(name));
    }
}

#[test]
fn registry_many_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0u16..100 {
        let name = format!("sym_{i}");
        reg.register(&name, terminal_meta());
    }
    assert_eq!(reg.len(), 101); // 100 + "end"
    assert_eq!(reg.get_id("sym_0"), Some(SymbolId(1)));
    assert_eq!(reg.get_id("sym_99"), Some(SymbolId(100)));
}

#[test]
fn registry_end_metadata() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0)).unwrap();
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(meta.terminal);
}
