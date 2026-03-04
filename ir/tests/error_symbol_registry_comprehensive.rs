// Wave 133: Comprehensive IR error types and symbol registry tests
use adze_ir::SymbolMetadata;
use adze_ir::error::IrError;
use adze_ir::symbol_registry::SymbolRegistry;

// =====================================================================
// IrError
// =====================================================================

#[test]
fn ir_error_invalid_symbol_display() {
    let err = IrError::InvalidSymbol("foo".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("foo"));
    assert!(msg.contains("invalid symbol"));
}

#[test]
fn ir_error_duplicate_rule_display() {
    let err = IrError::DuplicateRule("bar".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("bar"));
    assert!(msg.contains("duplicate rule"));
}

#[test]
fn ir_error_internal_display() {
    let err = IrError::Internal("something broke".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("something broke"));
}

#[test]
fn ir_error_debug() {
    let err = IrError::InvalidSymbol("test".to_string());
    let debug = format!("{:?}", err);
    assert!(!debug.is_empty());
}

#[test]
fn ir_error_is_error_trait() {
    let err: Box<dyn std::error::Error> = Box::new(IrError::Internal("test".to_string()));
    let _ = err.to_string();
}

// =====================================================================
// SymbolRegistry construction
// =====================================================================

#[test]
fn registry_new() {
    let reg = SymbolRegistry::new();
    // Registry pre-registers "end" (EOF symbol), so not empty
    assert!(!reg.is_empty());
    assert_eq!(reg.len(), 1);
}

// =====================================================================
// Registration
// =====================================================================

#[test]
fn register_single_symbol() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "foo",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert!(!reg.is_empty());
    // Registry starts with "end" (EOF) symbol, so 2 after adding one
    assert_eq!(reg.len(), 2);
    assert!(reg.contains_id(id));
}

#[test]
fn register_multiple_symbols() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register(
        "a",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let id2 = reg.register(
        "b",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let id3 = reg.register(
        "c",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    // 1 (end) + 3 = 4
    assert_eq!(reg.len(), 4);
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn register_same_name_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register(
        "foo",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let id2 = reg.register(
        "foo",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert_eq!(id1, id2);
}

// =====================================================================
// Lookup by name
// =====================================================================

#[test]
fn get_id_by_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "test",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert_eq!(reg.get_id("test"), Some(id));
}

#[test]
fn get_id_nonexistent() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

// =====================================================================
// Lookup by id
// =====================================================================

#[test]
fn get_name_by_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "hello",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert_eq!(reg.get_name(id), Some("hello"));
}

#[test]
fn get_name_nonexistent_id() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(adze_ir::SymbolId(9999)), None);
}

// =====================================================================
// Metadata
// =====================================================================

#[test]
fn get_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "sym",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let meta = reg.get_metadata(id);
    assert!(meta.is_some());
}

#[test]
fn get_metadata_nonexistent() {
    let reg = SymbolRegistry::new();
    assert!(reg.get_metadata(adze_ir::SymbolId(9999)).is_none());
}

// =====================================================================
// Contains
// =====================================================================

#[test]
fn contains_id_true() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "present",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert!(reg.contains_id(id));
}

#[test]
fn contains_id_false() {
    let reg = SymbolRegistry::new();
    assert!(!reg.contains_id(adze_ir::SymbolId(9999)));
}

// =====================================================================
// Iteration
// =====================================================================

#[test]
fn iter_empty() {
    let reg = SymbolRegistry::new();
    // Has "end" pre-registered
    assert_eq!(reg.iter().count(), 1);
}

#[test]
fn iter_all_symbols() {
    let mut reg = SymbolRegistry::new();
    reg.register(
        "a",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    reg.register(
        "b",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    reg.register(
        "c",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let names: Vec<String> = reg.iter().map(|(n, _)| n.to_string()).collect();
    // 3 registered + 1 "end" = 4
    assert_eq!(names.len(), 4);
    assert!(names.contains(&"a".to_string()));
    assert!(names.contains(&"b".to_string()));
    assert!(names.contains(&"c".to_string()));
}

// =====================================================================
// to_index_map and to_symbol_map
// =====================================================================

#[test]
fn to_index_map() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register(
        "x",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let id2 = reg.register(
        "y",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let map = reg.to_index_map();
    assert!(map.contains_key(&id1));
    assert!(map.contains_key(&id2));
    // 2 registered + 1 "end" = 3
    assert_eq!(map.len(), 3);
}

#[test]
fn to_symbol_map() {
    let mut reg = SymbolRegistry::new();
    reg.register(
        "x",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    reg.register(
        "y",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    let map = reg.to_symbol_map();
    assert_eq!(map.len(), 3); // 2 + end
}

#[test]
fn to_index_map_empty() {
    let reg = SymbolRegistry::new();
    let map = reg.to_index_map();
    // Has "end" pre-registered
    assert_eq!(map.len(), 1);
}

// =====================================================================
// Many symbols
// =====================================================================

#[test]
fn register_many_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0..100 {
        reg.register(
            &format!("sym_{}", i),
            SymbolMetadata {
                visible: true,
                named: true,
                hidden: false,
                terminal: false,
            },
        );
    }
    // 100 registered + 1 "end" = 101
    assert_eq!(reg.len(), 101);
}

// =====================================================================
// Name with special characters
// =====================================================================

#[test]
fn register_underscore_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "_start",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert_eq!(reg.get_name(id), Some("_start"));
}

#[test]
fn register_long_name() {
    let mut reg = SymbolRegistry::new();
    let long_name = "a".repeat(256);
    let id = reg.register(
        &long_name,
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert_eq!(reg.get_name(id), Some(long_name.as_str()));
}

#[test]
fn register_unicode_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(
        "αβγ",
        SymbolMetadata {
            visible: true,
            named: true,
            hidden: false,
            terminal: false,
        },
    );
    assert_eq!(reg.get_name(id), Some("αβγ"));
}
