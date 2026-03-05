//! Tests for the symbol registry and name resolution system.
//!
//! 64 tests across 8 categories (8 each):
//!   1. registry_add_*      — adding symbols to registry
//!   2. registry_lookup_*   — looking up by name/ID
//!   3. registry_token_*    — token registry operations
//!   4. registry_rule_*     — rule name registry operations
//!   5. registry_field_*    — field registry operations
//!   6. registry_unique_*   — uniqueness enforcement
//!   7. registry_capacity_* — capacity and scaling
//!   8. registry_serialize_* — serialize/deserialize registry

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, SymbolId, SymbolMetadata, SymbolRegistry};

/// Helper: terminal metadata (visible, unnamed, not hidden, terminal).
fn terminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    }
}

/// Helper: non-terminal metadata (visible, named, not hidden, not terminal).
fn nonterminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    }
}

/// Helper: hidden metadata (invisible, unnamed, hidden, terminal).
fn hidden_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: false,
        named: false,
        hidden: true,
        terminal: true,
    }
}

// ═══════════════════════════════════════════════════════════════════
// 1. registry_add_* — adding symbols to registry
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_add_new_creates_with_end_symbol() {
    let reg = SymbolRegistry::new();
    assert_eq!(
        reg.len(),
        1,
        "new registry should contain only the 'end' symbol"
    );
    assert!(reg.get_id("end").is_some());
    assert_eq!(reg.get_id("end").unwrap(), SymbolId(0));
}

#[test]
fn registry_add_single_terminal() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("plus", terminal_meta());
    assert_eq!(id, SymbolId(1));
    assert_eq!(reg.len(), 2);
}

#[test]
fn registry_add_single_nonterminal() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("expression", nonterminal_meta());
    assert_eq!(id, SymbolId(1));
    assert!(reg.get_metadata(id).unwrap().named);
}

#[test]
fn registry_add_multiple_symbols_sequential_ids() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("alpha", terminal_meta());
    let b = reg.register("beta", terminal_meta());
    let c = reg.register("gamma", terminal_meta());
    assert_eq!(a, SymbolId(1));
    assert_eq!(b, SymbolId(2));
    assert_eq!(c, SymbolId(3));
    assert_eq!(reg.len(), 4);
}

#[test]
fn registry_add_preserves_insertion_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("zebra", terminal_meta());
    reg.register("apple", terminal_meta());
    reg.register("mango", terminal_meta());

    let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    assert_eq!(names, vec!["end", "zebra", "apple", "mango"]);
}

#[test]
fn registry_add_hidden_symbol() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("_whitespace", hidden_meta());
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.hidden);
    assert!(!meta.visible);
}

#[test]
fn registry_add_returns_correct_metadata() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };
    let id = reg.register("_internal", meta);
    let retrieved = reg.get_metadata(id).unwrap();
    assert_eq!(retrieved, meta);
}

#[test]
fn registry_add_end_symbol_metadata_is_correct() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0)).unwrap();
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 2. registry_lookup_* — looking up by name/ID
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_lookup_by_name_returns_correct_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("stmt", nonterminal_meta());
    assert_eq!(reg.get_id("stmt"), Some(id));
}

#[test]
fn registry_lookup_by_id_returns_correct_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("factor", nonterminal_meta());
    assert_eq!(reg.get_name(id), Some("factor"));
}

#[test]
fn registry_lookup_missing_name_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

#[test]
fn registry_lookup_missing_id_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(999)), None);
}

#[test]
fn registry_lookup_contains_id_true_for_existing() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("item", terminal_meta());
    assert!(reg.contains_id(id));
}

#[test]
fn registry_lookup_contains_id_false_for_absent() {
    let reg = SymbolRegistry::new();
    assert!(!reg.contains_id(SymbolId(42)));
}

#[test]
fn registry_lookup_end_symbol_by_name() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn registry_lookup_metadata_for_missing_returns_none() {
    let reg = SymbolRegistry::new();
    assert!(reg.get_metadata(SymbolId(500)).is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 3. registry_token_* — token registry operations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_token_builder_registers_tokens() {
    let grammar = GrammarBuilder::new("tok_test")
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUMBER", "PLUS", "NUMBER"])
        .start("expr")
        .build();

    assert!(!grammar.tokens.is_empty());
    assert_eq!(grammar.tokens.len(), 2);
}

#[test]
fn registry_token_names_preserved_in_grammar() {
    let grammar = GrammarBuilder::new("tok_names")
        .token("IDENT", r"[a-z]+")
        .rule("prog", vec!["IDENT"])
        .start("prog")
        .build();

    let has_ident = grammar.tokens.values().any(|t| t.name == "IDENT");
    assert!(has_ident);
}

#[test]
fn registry_token_in_built_registry() {
    let mut grammar = GrammarBuilder::new("tok_reg")
        .token("STAR", r"\*")
        .rule("items", vec!["STAR"])
        .start("items")
        .build();

    let reg = grammar.get_or_build_registry();
    assert!(reg.get_id("STAR").is_some());
}

#[test]
fn registry_token_marked_terminal_in_registry() {
    let mut grammar = GrammarBuilder::new("tok_term")
        .token("SEMI", ";")
        .rule("stmts", vec!["SEMI"])
        .start("stmts")
        .build();

    let reg = grammar.get_or_build_registry();
    let id = reg.get_id("SEMI").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.terminal);
}

#[test]
fn registry_token_multiple_tokens_all_present() {
    let grammar = GrammarBuilder::new("multi_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B", "C"])
        .start("root")
        .build();

    let names: Vec<&str> = grammar.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"A"));
    assert!(names.contains(&"B"));
    assert!(names.contains(&"C"));
}

#[test]
fn registry_token_extras_marked_hidden() {
    let mut grammar = GrammarBuilder::new("extras")
        .token("WS", r"\s+")
        .token("NUM", r"\d+")
        .extra("WS")
        .rule("root", vec!["NUM"])
        .start("root")
        .build();

    let reg = grammar.get_or_build_registry();
    let ws_id = reg.get_id("WS").unwrap();
    let meta = reg.get_metadata(ws_id).unwrap();
    assert!(meta.hidden);
}

#[test]
fn registry_token_distinct_ids_for_each() {
    let mut grammar = GrammarBuilder::new("dist_tok")
        .token("X", "x")
        .token("Y", "y")
        .rule("r", vec!["X", "Y"])
        .start("r")
        .build();

    let reg = grammar.get_or_build_registry();
    let x_id = reg.get_id("X").unwrap();
    let y_id = reg.get_id("Y").unwrap();
    assert_ne!(x_id, y_id);
}

#[test]
fn registry_token_pattern_does_not_affect_registry_name() {
    let mut grammar = GrammarBuilder::new("pat_test")
        .token("DOT", r"\.")
        .rule("root", vec!["DOT"])
        .start("root")
        .build();

    let reg = grammar.get_or_build_registry();
    assert!(reg.get_id("DOT").is_some());
    // The pattern "\\." should not appear as a name.
    assert!(reg.get_id(r"\.").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 4. registry_rule_* — rule name registry operations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_rule_names_contain_start_symbol() {
    let grammar = GrammarBuilder::new("rule_start")
        .token("ID", r"[a-z]+")
        .rule("program", vec!["ID"])
        .start("program")
        .build();

    let has_program = grammar.rule_names.values().any(|n| n == "program");
    assert!(has_program);
}

#[test]
fn registry_rule_names_contain_all_nonterminals() {
    let grammar = GrammarBuilder::new("multi_rule")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["term", "PLUS", "term"])
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    let names: Vec<&str> = grammar.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"expr"));
    assert!(names.contains(&"term"));
}

#[test]
fn registry_rule_name_in_built_registry_is_named() {
    let mut grammar = GrammarBuilder::new("rule_named")
        .token("TOK", "t")
        .rule("root", vec!["TOK"])
        .start("root")
        .build();

    let reg = grammar.get_or_build_registry();
    let root_id = reg.get_id("root").unwrap();
    let meta = reg.get_metadata(root_id).unwrap();
    assert!(meta.named, "non-terminals should be named");
}

#[test]
fn registry_rule_name_not_terminal() {
    let mut grammar = GrammarBuilder::new("rule_nonterm")
        .token("TOK", "t")
        .rule("node", vec!["TOK"])
        .start("node")
        .build();

    let reg = grammar.get_or_build_registry();
    let node_id = reg.get_id("node").unwrap();
    let meta = reg.get_metadata(node_id).unwrap();
    assert!(!meta.terminal, "rules should not be terminals");
}

#[test]
fn registry_rule_start_symbol_resolves() {
    let grammar = GrammarBuilder::new("start_res")
        .token("TOK", "t")
        .rule("entry", vec!["TOK"])
        .start("entry")
        .build();

    assert!(grammar.start_symbol().is_some());
}

#[test]
fn registry_rule_names_map_symbol_ids() {
    let grammar = GrammarBuilder::new("id_map")
        .token("TOK", "t")
        .rule("foo", vec!["TOK"])
        .start("foo")
        .build();

    for (sym_id, name) in &grammar.rule_names {
        assert!(
            !name.is_empty(),
            "rule name for {:?} should not be empty",
            sym_id
        );
    }
}

#[test]
fn registry_rule_hidden_rule_starts_with_underscore() {
    let mut grammar = GrammarBuilder::new("hidden_rule")
        .token("TOK", "t")
        .rule("_hidden", vec!["TOK"])
        .rule("visible", vec!["_hidden"])
        .start("visible")
        .build();

    let reg = grammar.get_or_build_registry();
    if let Some(id) = reg.get_id("_hidden") {
        let meta = reg.get_metadata(id).unwrap();
        assert!(meta.hidden, "underscore-prefixed rules should be hidden");
        assert!(!meta.visible);
    }
}

#[test]
fn registry_rule_registry_contains_all_rule_names() {
    let mut grammar = GrammarBuilder::new("all_rules")
        .token("A", "a")
        .token("B", "b")
        .rule("first", vec!["A"])
        .rule("second", vec!["B"])
        .rule("third", vec!["first", "second"])
        .start("third")
        .build();

    let reg = grammar.build_registry();
    for name in grammar.rule_names.values() {
        assert!(
            reg.get_id(name).is_some(),
            "rule '{}' should be in registry",
            name
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// 5. registry_field_* — field registry operations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_field_empty_by_default() {
    let grammar = GrammarBuilder::new("no_fields")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();

    assert!(grammar.fields.is_empty());
}

#[test]
fn registry_field_insert_and_retrieve() {
    let mut grammar = Grammar::new("field_test".to_string());
    grammar.fields.insert(FieldId(0), "name".to_string());
    assert_eq!(grammar.fields.get(&FieldId(0)), Some(&"name".to_string()));
}

#[test]
fn registry_field_multiple_ids_are_distinct() {
    let mut grammar = Grammar::new("multi_field".to_string());
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());

    assert_ne!(
        grammar.fields.get(&FieldId(0)),
        grammar.fields.get(&FieldId(1))
    );
}

#[test]
fn registry_field_id_is_copy() {
    let id = FieldId(5);
    let copy = id; // Copy, not move
    assert_eq!(id, copy);
}

#[test]
fn registry_field_preserves_insertion_order() {
    let mut grammar = Grammar::new("order_test".to_string());
    grammar.fields.insert(FieldId(0), "zebra".to_string());
    grammar.fields.insert(FieldId(1), "alpha".to_string());
    grammar.fields.insert(FieldId(2), "middle".to_string());

    let names: Vec<&str> = grammar.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["zebra", "alpha", "middle"]);
}

#[test]
fn registry_field_overwrite_same_id() {
    let mut grammar = Grammar::new("overwrite".to_string());
    grammar.fields.insert(FieldId(0), "old".to_string());
    grammar.fields.insert(FieldId(0), "new".to_string());
    assert_eq!(grammar.fields.get(&FieldId(0)), Some(&"new".to_string()));
    assert_eq!(grammar.fields.len(), 1);
}

#[test]
fn registry_field_id_hash_equality() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(FieldId(10), "value");
    assert_eq!(map.get(&FieldId(10)), Some(&"value"));
}

#[test]
fn registry_field_display_format() {
    let id = FieldId(42);
    let display = format!("{}", id);
    assert!(!display.is_empty(), "FieldId should have a Display impl");
}

// ═══════════════════════════════════════════════════════════════════
// 6. registry_unique_* — uniqueness enforcement
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_unique_duplicate_name_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let first = reg.register("dup", terminal_meta());
    let second = reg.register("dup", terminal_meta());
    assert_eq!(first, second);
}

#[test]
fn registry_unique_duplicate_does_not_increase_len() {
    let mut reg = SymbolRegistry::new();
    reg.register("sym", terminal_meta());
    let len_before = reg.len();
    reg.register("sym", terminal_meta());
    assert_eq!(reg.len(), len_before);
}

#[test]
fn registry_unique_duplicate_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("change_me", terminal_meta());

    let new_meta = nonterminal_meta();
    reg.register("change_me", new_meta);

    let retrieved = reg.get_metadata(id).unwrap();
    assert_eq!(retrieved, new_meta);
}

#[test]
fn registry_unique_case_sensitive_names() {
    let mut reg = SymbolRegistry::new();
    let lower = reg.register("token", terminal_meta());
    let upper = reg.register("Token", terminal_meta());
    assert_ne!(lower, upper, "names are case-sensitive");
}

#[test]
fn registry_unique_ids_never_reused() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("a", terminal_meta());
    let b = reg.register("b", terminal_meta());
    let c = reg.register("c", terminal_meta());

    let ids = [SymbolId(0), a, b, c];
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "IDs at {} and {} should differ", i, j);
        }
    }
}

#[test]
fn registry_unique_empty_name_allowed() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("", terminal_meta());
    assert_eq!(reg.get_name(id), Some(""));
}

#[test]
fn registry_unique_whitespace_name_distinct() {
    let mut reg = SymbolRegistry::new();
    let space = reg.register(" ", terminal_meta());
    let tab = reg.register("\t", terminal_meta());
    assert_ne!(space, tab);
}

#[test]
fn registry_unique_reregister_end_returns_zero() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("end", terminal_meta());
    assert_eq!(id, SymbolId(0), "re-registering 'end' should return ID 0");
}

// ═══════════════════════════════════════════════════════════════════
// 7. registry_capacity_* — capacity and scaling
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_capacity_hundred_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0..100 {
        reg.register(&format!("sym_{}", i), terminal_meta());
    }
    // 100 new + 1 "end"
    assert_eq!(reg.len(), 101);
}

#[test]
fn registry_capacity_all_lookups_valid_after_bulk_insert() {
    let mut reg = SymbolRegistry::new();
    for i in 0..50 {
        reg.register(&format!("s{}", i), terminal_meta());
    }
    for i in 0..50 {
        let name = format!("s{}", i);
        assert!(reg.get_id(&name).is_some(), "lookup failed for {}", name);
    }
}

#[test]
fn registry_capacity_reverse_lookup_after_bulk_insert() {
    let mut reg = SymbolRegistry::new();
    let mut ids = Vec::new();
    for i in 0..30 {
        ids.push(reg.register(&format!("r{}", i), nonterminal_meta()));
    }
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(reg.get_name(*id), Some(format!("r{}", i).as_str()));
    }
}

#[test]
fn registry_capacity_iter_count_matches_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..25 {
        reg.register(&format!("item_{}", i), terminal_meta());
    }
    assert_eq!(reg.iter().count(), reg.len());
}

#[test]
fn registry_capacity_to_index_map_size() {
    let mut reg = SymbolRegistry::new();
    for i in 0..20 {
        reg.register(&format!("n{}", i), terminal_meta());
    }
    let index_map = reg.to_index_map();
    assert_eq!(index_map.len(), reg.len());
}

#[test]
fn registry_capacity_to_symbol_map_roundtrip() {
    let mut reg = SymbolRegistry::new();
    for i in 0..15 {
        reg.register(&format!("x{}", i), terminal_meta());
    }
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();

    for (&sym_id, &idx) in &idx_map {
        assert_eq!(sym_map.get(&idx), Some(&sym_id));
    }
}

#[test]
fn registry_capacity_is_empty_false_after_new() {
    let reg = SymbolRegistry::new();
    assert!(!reg.is_empty(), "new registry has the 'end' symbol");
}

#[test]
fn registry_capacity_grammar_build_registry_populates() {
    let mut grammar = GrammarBuilder::new("scale")
        .token("T1", "a")
        .token("T2", "b")
        .token("T3", "c")
        .rule("r1", vec!["T1"])
        .rule("r2", vec!["T2"])
        .rule("r3", vec!["T3", "r1", "r2"])
        .start("r3")
        .build();

    let reg = grammar.get_or_build_registry();
    // At minimum: end + 3 tokens + 3 rules = 7
    assert!(
        reg.len() >= 7,
        "registry should have at least 7 symbols, got {}",
        reg.len()
    );
}

// ═══════════════════════════════════════════════════════════════════
// 8. registry_serialize_* — serialize/deserialize registry
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_serialize_empty_roundtrip() {
    let reg = SymbolRegistry::new();
    let json = serde_json::to_string(&reg).expect("serialize");
    let restored: SymbolRegistry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reg, restored);
}

#[test]
fn registry_serialize_with_symbols_roundtrip() {
    let mut reg = SymbolRegistry::new();
    reg.register("alpha", terminal_meta());
    reg.register("beta", nonterminal_meta());

    let json = serde_json::to_string(&reg).expect("serialize");
    let restored: SymbolRegistry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reg, restored);
}

#[test]
fn registry_serialize_preserves_ids() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("sym", terminal_meta());

    let json = serde_json::to_string(&reg).expect("serialize");
    let restored: SymbolRegistry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.get_id("sym"), Some(id));
}

#[test]
fn registry_serialize_preserves_metadata() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };
    let id = reg.register("special", meta);

    let json = serde_json::to_string(&reg).expect("serialize");
    let restored: SymbolRegistry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.get_metadata(id), Some(meta));
}

#[test]
fn registry_serialize_preserves_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("zz", terminal_meta());
    reg.register("aa", terminal_meta());
    reg.register("mm", terminal_meta());

    let json = serde_json::to_string(&reg).expect("serialize");
    let restored: SymbolRegistry = serde_json::from_str(&json).expect("deserialize");

    let orig_names: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    let rest_names: Vec<&str> = restored.iter().map(|(n, _)| n).collect();
    assert_eq!(orig_names, rest_names);
}

#[test]
fn registry_serialize_json_contains_symbol_names() {
    let mut reg = SymbolRegistry::new();
    reg.register("foobar", terminal_meta());

    let json = serde_json::to_string(&reg).expect("serialize");
    assert!(
        json.contains("foobar"),
        "JSON should contain the symbol name"
    );
}

#[test]
fn registry_serialize_grammar_with_registry_roundtrip() {
    let mut grammar = GrammarBuilder::new("serde_grammar")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    // Force registry construction
    let _ = grammar.get_or_build_registry();

    let json = serde_json::to_string(&grammar).expect("serialize grammar");
    let restored: Grammar = serde_json::from_str(&json).expect("deserialize grammar");

    assert_eq!(grammar.name, restored.name);
    assert_eq!(grammar.tokens.len(), restored.tokens.len());
    assert_eq!(grammar.rule_names.len(), restored.rule_names.len());
}

#[test]
fn registry_serialize_large_registry_roundtrip() {
    let mut reg = SymbolRegistry::new();
    for i in 0..200 {
        let meta = if i % 2 == 0 {
            terminal_meta()
        } else {
            nonterminal_meta()
        };
        reg.register(&format!("sym_{}", i), meta);
    }

    let json = serde_json::to_string(&reg).expect("serialize");
    let restored: SymbolRegistry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reg.len(), restored.len());
    assert_eq!(reg, restored);
}
