use adze_ir::symbol_registry::*;
use adze_ir::*;

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

// 1. Basic construction
#[test]
fn test_new_registry_has_eof() {
    let reg = SymbolRegistry::new();
    // EOF "end" is pre-registered, so length == 1
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());
    assert!(reg.contains_id(SymbolId(0)));
}

// 2. Default trait delegates to new()
#[test]
fn test_default_matches_new() {
    let from_new = SymbolRegistry::new();
    let from_default = SymbolRegistry::default();
    assert_eq!(from_new.len(), from_default.len());
    assert_eq!(from_new.get_id("end"), from_default.get_id("end"));
}

// 3. EOF is SymbolId(0) — verified via contains_id + metadata rather than get_id
#[test]
fn test_eof_symbol_zero_metadata() {
    let reg = SymbolRegistry::new();
    let eof_id = SymbolId(0);
    assert!(reg.contains_id(eof_id));
    assert_eq!(reg.get_name(eof_id), Some("end"));
    let meta = reg.get_metadata(eof_id).expect("EOF should have metadata");
    assert!(meta.terminal, "EOF should be terminal");
    assert!(!meta.named, "EOF should not be named");
}

// 4. Register and lookup a symbol
#[test]
fn test_register_and_lookup() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("number", terminal_meta());
    assert_eq!(id, SymbolId(1)); // 0 is EOF
    assert_eq!(reg.get_id("number"), Some(id));
    assert_eq!(reg.get_name(id), Some("number"));
    assert!(reg.contains_id(id));
    assert_eq!(reg.len(), 2);
}

// 5. Lookup of nonexistent symbol returns None
#[test]
fn test_lookup_nonexistent() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
    assert_eq!(reg.get_name(SymbolId(999)), None);
    assert_eq!(reg.get_metadata(SymbolId(999)), None);
    assert!(!reg.contains_id(SymbolId(999)));
}

// 6. Duplicate registration returns same ID but updates metadata
#[test]
fn test_duplicate_registration_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("tok", terminal_meta());
    let updated_meta = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };
    let id2 = reg.register("tok", updated_meta);

    assert_eq!(id1, id2, "re-registering must return the same ID");
    assert_eq!(reg.len(), 2, "count must not increase on duplicate");
    let meta = reg.get_metadata(id1).unwrap();
    assert_eq!(meta, updated_meta, "metadata should be updated");
}

// 7. Deterministic ordering across independent registries
#[test]
fn test_deterministic_ordering() {
    let names = ["alpha", "beta", "gamma", "delta", "epsilon"];
    let mut reg_a = SymbolRegistry::new();
    let mut reg_b = SymbolRegistry::new();

    for &n in &names {
        reg_a.register(n, terminal_meta());
        reg_b.register(n, terminal_meta());
    }

    for &n in &names {
        assert_eq!(reg_a.get_id(n), reg_b.get_id(n));
    }

    // Verify sequential IDs starting after EOF(0)
    for (i, &n) in names.iter().enumerate() {
        assert_eq!(reg_a.get_id(n), Some(SymbolId((i + 1) as u16)));
    }
}

// 8. Iteration preserves insertion order
#[test]
fn test_iter_preserves_insertion_order() {
    let mut reg = SymbolRegistry::new();
    let names = ["expr", "term", "factor"];
    for &n in &names {
        reg.register(n, nonterminal_meta());
    }

    let collected: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    // "end" (EOF) is first, then insertion order
    assert_eq!(collected, vec!["end", "expr", "term", "factor"]);
}

// 9. SymbolInfo from iter carries correct id and metadata
#[test]
fn test_iter_symbol_info() {
    let mut reg = SymbolRegistry::new();
    let meta = nonterminal_meta();
    let id = reg.register("stmt", meta);

    let info = reg
        .iter()
        .find(|(name, _)| *name == "stmt")
        .map(|(_, info)| info)
        .expect("stmt must be in iter");

    assert_eq!(info.id, id);
    assert_eq!(info.metadata, meta);
}

// 10. Large registry (200 symbols)
#[test]
fn test_large_registry() {
    let mut reg = SymbolRegistry::new();
    for i in 0..200u16 {
        let name = format!("sym_{i}");
        let id = reg.register(&name, terminal_meta());
        assert_eq!(id, SymbolId(i + 1)); // +1 because EOF is 0
    }
    assert_eq!(reg.len(), 201); // 200 + EOF

    // Spot-check random lookups
    assert_eq!(reg.get_id("sym_0"), Some(SymbolId(1)));
    assert_eq!(reg.get_id("sym_199"), Some(SymbolId(200)));
    assert_eq!(reg.get_name(SymbolId(100)), Some("sym_99"));
}

// 11. to_index_map and to_symbol_map are inverses
#[test]
fn test_index_map_and_symbol_map_inverse() {
    let mut reg = SymbolRegistry::new();
    for name in ["a", "b", "c"] {
        reg.register(name, terminal_meta());
    }

    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();

    for (&sym_id, &idx) in &idx_map {
        assert_eq!(sym_map.get(&idx), Some(&sym_id));
    }
    for (&idx, &sym_id) in &sym_map {
        assert_eq!(idx_map.get(&sym_id), Some(&idx));
    }
}

// 12. Error symbol handling
#[test]
fn test_error_symbol_handling() {
    let mut reg = SymbolRegistry::new();
    let error_meta = SymbolMetadata {
        visible: false,
        named: false,
        hidden: true,
        terminal: true,
    };
    let id = reg.register("ERROR", error_meta);
    assert!(reg.contains_id(id));
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.hidden);
    assert!(!meta.named);
    assert_eq!(reg.get_name(id), Some("ERROR"));
}

// 13. Mixed terminal and nonterminal symbols
#[test]
fn test_mixed_terminal_nonterminal() {
    let mut reg = SymbolRegistry::new();
    let t_id = reg.register("+", terminal_meta());
    let nt_id = reg.register("expression", nonterminal_meta());

    let t_meta = reg.get_metadata(t_id).unwrap();
    let nt_meta = reg.get_metadata(nt_id).unwrap();

    assert!(t_meta.terminal);
    assert!(!nt_meta.terminal);
    assert!(!t_meta.named);
    assert!(nt_meta.named);
}
