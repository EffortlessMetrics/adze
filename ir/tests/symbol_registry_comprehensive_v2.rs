//! Comprehensive tests for SymbolRegistry API.

use adze_ir::symbol_registry::{SymbolInfo, SymbolRegistry};
use adze_ir::{SymbolId, SymbolMetadata};

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

// === Construction ===

#[test]
fn new_has_end_symbol() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn new_is_not_empty() {
    let reg = SymbolRegistry::new();
    assert!(!reg.is_empty());
}

#[test]
fn new_len_is_one() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.len(), 1);
}

#[test]
fn default_same_as_new() {
    let a = SymbolRegistry::new();
    let b = SymbolRegistry::default();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.get_id("end"), b.get_id("end"));
}

// === Register ===

#[test]
fn register_returns_new_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("num", terminal_meta());
    assert_eq!(id, SymbolId(1));
}

#[test]
fn register_increments() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("a", terminal_meta());
    let b = reg.register("b", terminal_meta());
    assert_eq!(a, SymbolId(1));
    assert_eq!(b, SymbolId(2));
}

#[test]
fn register_duplicate_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("tok", terminal_meta());
    let id2 = reg.register("tok", terminal_meta());
    assert_eq!(id1, id2);
}

#[test]
fn register_duplicate_no_extra_len() {
    let mut reg = SymbolRegistry::new();
    reg.register("x", terminal_meta());
    reg.register("x", terminal_meta());
    assert_eq!(reg.len(), 2); // "end" + "x"
}

#[test]
fn register_many() {
    let mut reg = SymbolRegistry::new();
    for i in 0..100 {
        let name = format!("sym{i}");
        reg.register(&name, terminal_meta());
    }
    assert_eq!(reg.len(), 101); // 100 + "end"
}

// === get_id ===

#[test]
fn get_id_existing() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("foo", terminal_meta());
    assert_eq!(reg.get_id("foo"), Some(id));
}

#[test]
fn get_id_missing() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

#[test]
fn get_id_end() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

// === get_name ===

#[test]
fn get_name_existing() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("bar", terminal_meta());
    assert_eq!(reg.get_name(id), Some("bar"));
}

#[test]
fn get_name_end() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn get_name_missing() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(999)), None);
}

// === get_metadata ===

#[test]
fn get_metadata_existing() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    let meta = reg.get_metadata(id);
    assert!(meta.is_some());
    assert!(meta.unwrap().terminal);
}

#[test]
fn get_metadata_nonterminal() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("rule", nonterminal_meta());
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
    assert!(meta.named);
}

#[test]
fn get_metadata_missing() {
    let reg = SymbolRegistry::new();
    assert!(reg.get_metadata(SymbolId(999)).is_none());
}

#[test]
fn get_metadata_end() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0));
    assert!(meta.is_some());
}

// === contains_id ===

#[test]
fn contains_id_end() {
    let reg = SymbolRegistry::new();
    assert!(reg.contains_id(SymbolId(0)));
}

#[test]
fn contains_id_registered() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("x", terminal_meta());
    assert!(reg.contains_id(id));
}

#[test]
fn contains_id_missing() {
    let reg = SymbolRegistry::new();
    assert!(!reg.contains_id(SymbolId(999)));
}

// === iter ===

#[test]
fn iter_includes_end() {
    let reg = SymbolRegistry::new();
    let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    assert!(names.contains(&"end"));
}

#[test]
fn iter_includes_registered() {
    let mut reg = SymbolRegistry::new();
    reg.register("alpha", terminal_meta());
    reg.register("beta", nonterminal_meta());
    let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn iter_count_matches_len() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    reg.register("b", terminal_meta());
    assert_eq!(reg.iter().count(), reg.len());
}

#[test]
fn iter_preserves_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("first", terminal_meta());
    reg.register("second", terminal_meta());
    reg.register("third", terminal_meta());
    let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    assert_eq!(names[0], "end");
    assert_eq!(names[1], "first");
    assert_eq!(names[2], "second");
    assert_eq!(names[3], "third");
}

// === to_index_map / to_symbol_map ===

#[test]
fn index_map_contains_all() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    reg.register("b", terminal_meta());
    let map = reg.to_index_map();
    assert_eq!(map.len(), reg.len());
}

#[test]
fn symbol_map_contains_all() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    let map = reg.to_symbol_map();
    assert_eq!(map.len(), reg.len());
}

#[test]
fn index_map_and_symbol_map_inverse() {
    let mut reg = SymbolRegistry::new();
    reg.register("x", terminal_meta());
    reg.register("y", terminal_meta());
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();
    for (&sym, &idx) in &idx_map {
        assert_eq!(sym_map[&idx], sym);
    }
}

// === Determinism ===

#[test]
fn deterministic_ids() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    for name in ["a", "b", "c", "d"] {
        r1.register(name, terminal_meta());
        r2.register(name, terminal_meta());
    }
    for name in ["a", "b", "c", "d"] {
        assert_eq!(r1.get_id(name), r2.get_id(name));
    }
}

#[test]
fn deterministic_order() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    for name in ["x", "y", "z"] {
        r1.register(name, terminal_meta());
        r2.register(name, terminal_meta());
    }
    let n1: Vec<&str> = r1.iter().map(|(n, _)| n).collect();
    let n2: Vec<&str> = r2.iter().map(|(n, _)| n).collect();
    assert_eq!(n1, n2);
}

// === Clone/PartialEq ===

#[test]
fn clone_eq() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal_meta());
    let cloned = reg.clone();
    assert_eq!(reg, cloned);
}

#[test]
fn ne_different_symbols() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    r1.register("a", terminal_meta());
    r2.register("b", terminal_meta());
    assert_ne!(r1, r2);
}

// === Debug ===

#[test]
fn debug_not_empty() {
    let reg = SymbolRegistry::new();
    let d = format!("{:?}", reg);
    assert!(!d.is_empty());
}

// === SymbolInfo ===

#[test]
fn symbol_info_from_iter() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("test", terminal_meta());
    let info: Vec<SymbolInfo> = reg.iter().map(|(_, info)| info).collect();
    assert!(info.iter().any(|i| i.id == id));
}

#[test]
fn symbol_info_metadata() {
    let mut reg = SymbolRegistry::new();
    reg.register("nt", nonterminal_meta());
    for (name, info) in reg.iter() {
        if name == "nt" {
            assert!(!info.metadata.terminal);
            assert!(info.metadata.named);
        }
    }
}

// === Edge cases ===

#[test]
fn register_empty_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("", terminal_meta());
    assert_eq!(reg.get_id(""), Some(id));
    assert_eq!(reg.get_name(id), Some(""));
}

#[test]
fn register_unicode_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("αβγ", terminal_meta());
    assert_eq!(reg.get_id("αβγ"), Some(id));
}

#[test]
fn register_long_name() {
    let mut reg = SymbolRegistry::new();
    let name = "x".repeat(1000);
    let id = reg.register(&name, terminal_meta());
    assert_eq!(reg.get_name(id), Some(name.as_str()));
}

#[test]
fn register_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    // Re-register with different metadata
    let id2 = reg.register("tok", nonterminal_meta());
    assert_eq!(id, id2);
    let meta = reg.get_metadata(id).unwrap();
    // Should have updated metadata
    assert!(!meta.terminal); // nonterminal_meta
}

// === Serde roundtrip ===

#[test]
fn serde_roundtrip() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    reg.register("b", nonterminal_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let reg2: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, reg2);
}
