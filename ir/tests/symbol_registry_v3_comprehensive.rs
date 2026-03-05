//! Comprehensive tests for SymbolRegistry management in adze-ir.
//!
//! Covers: name-to-ID mapping, token mapping, rule registration, symbol counts,
//! large grammars, serialization roundtrips, builder integration, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
// 1. Symbol name to ID mapping (8 tests)
// ===========================================================================

#[test]
fn test_name_to_id_eof_always_zero() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_name_to_id_first_user_symbol_is_one() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("number", terminal_meta());
    assert_eq!(id, SymbolId(1));
}

#[test]
fn test_name_to_id_sequential_assignment() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("alpha", terminal_meta());
    let b = reg.register("beta", terminal_meta());
    let c = reg.register("gamma", terminal_meta());
    assert_eq!(a.0 + 1, b.0);
    assert_eq!(b.0 + 1, c.0);
}

#[test]
fn test_name_to_id_duplicate_returns_same() {
    let mut reg = SymbolRegistry::new();
    let first = reg.register("tok", terminal_meta());
    let second = reg.register("tok", terminal_meta());
    assert_eq!(first, second);
}

#[test]
fn test_name_to_id_missing_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

#[test]
fn test_name_to_id_deterministic_across_instances() {
    let names = ["x", "y", "z"];
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    for n in &names {
        r1.register(n, terminal_meta());
        r2.register(n, terminal_meta());
    }
    for n in &names {
        assert_eq!(r1.get_id(n), r2.get_id(n));
    }
}

#[test]
fn test_name_to_id_reverse_lookup() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("identifier", nonterminal_meta());
    assert_eq!(reg.get_name(id), Some("identifier"));
}

#[test]
fn test_name_to_id_reverse_lookup_eof() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

// ===========================================================================
// 2. Token name to ID mapping (8 tests)
// ===========================================================================

#[test]
fn test_token_registered_in_grammar_registry() {
    let grammar = GrammarBuilder::new("tok_test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("NUMBER").is_some());
    assert!(reg.get_id("+").is_some());
}

#[test]
fn test_token_metadata_is_terminal() {
    let grammar = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let reg = grammar.build_registry();
    let id = reg.get_id("NUM").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.terminal);
}

#[test]
fn test_token_multiple_tokens_all_present() {
    let grammar = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("r", vec!["A", "B", "C"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    for name in ["A", "B", "C"] {
        assert!(reg.get_id(name).is_some(), "missing token {name}");
    }
}

#[test]
fn test_token_ids_are_distinct() {
    let grammar = GrammarBuilder::new("distinct")
        .token("X", "x")
        .token("Y", "y")
        .rule("r", vec!["X"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    let x = reg.get_id("X").unwrap();
    let y = reg.get_id("Y").unwrap();
    assert_ne!(x, y);
}

#[test]
fn test_token_fragile_token_registered() {
    let grammar = GrammarBuilder::new("fragile")
        .fragile_token("ERR", "error")
        .token("OK", "ok")
        .rule("r", vec!["OK"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("ERR").is_some());
}

#[test]
fn test_token_operator_tokens() {
    let grammar = GrammarBuilder::new("ops")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let reg = grammar.build_registry();
    for op in ["+", "-", "*", "/"] {
        assert!(reg.get_id(op).is_some(), "missing operator {op}");
    }
}

#[test]
fn test_token_hidden_extra_token() {
    let grammar = GrammarBuilder::new("extra")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .token("ID", r"[a-z]+")
        .rule("r", vec!["ID"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    let ws_id = reg.get_id("WS").unwrap();
    let meta = reg.get_metadata(ws_id).unwrap();
    assert!(meta.hidden);
}

#[test]
fn test_token_registry_contains_eof() {
    let grammar = GrammarBuilder::new("eof")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("end").is_some());
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

// ===========================================================================
// 3. Rule name registration (8 tests)
// ===========================================================================

#[test]
fn test_rule_name_present_in_registry() {
    let grammar = GrammarBuilder::new("rn")
        .token("A", "a")
        .rule("statement", vec!["A"])
        .start("statement")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("statement").is_some());
}

#[test]
fn test_rule_name_metadata_is_nonterminal() {
    let grammar = GrammarBuilder::new("rn")
        .token("A", "a")
        .rule("statement", vec!["A"])
        .start("statement")
        .build();
    let reg = grammar.build_registry();
    let id = reg.get_id("statement").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
    assert!(meta.named);
}

#[test]
fn test_rule_name_multiple_rules_same_lhs() {
    let grammar = GrammarBuilder::new("multi_rule")
        .token("A", "a")
        .token("B", "b")
        .rule("item", vec!["A"])
        .rule("item", vec!["B"])
        .start("item")
        .build();
    let reg = grammar.build_registry();
    // "item" appears exactly once in registry despite 2 productions
    let id = reg.get_id("item").unwrap();
    assert!(reg.get_name(id).is_some());
}

#[test]
fn test_rule_name_distinct_from_token() {
    let grammar = GrammarBuilder::new("sep")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let reg = grammar.build_registry();
    let tok_id = reg.get_id("NUM").unwrap();
    let rule_id = reg.get_id("expr").unwrap();
    assert_ne!(tok_id, rule_id);
}

#[test]
fn test_rule_name_start_symbol_registered() {
    let grammar = GrammarBuilder::new("start")
        .token("X", "x")
        .rule("program", vec!["X"])
        .start("program")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("program").is_some());
}

#[test]
fn test_rule_name_underscore_prefix_hidden() {
    let mut grammar = Grammar::new("hidden_test".to_string());
    let id = SymbolId(10);
    grammar.rule_names.insert(id, "_internal".to_string());
    let reg = grammar.build_registry();
    if let Some(rid) = reg.get_id("_internal") {
        let meta = reg.get_metadata(rid).unwrap();
        assert!(meta.hidden);
        assert!(!meta.visible);
    }
}

#[test]
fn test_rule_name_find_symbol_by_name() {
    let grammar = GrammarBuilder::new("find")
        .token("A", "a")
        .rule("target", vec!["A"])
        .start("target")
        .build();
    let found = grammar.find_symbol_by_name("target");
    assert!(found.is_some());
}

#[test]
fn test_rule_name_find_symbol_missing() {
    let grammar = GrammarBuilder::new("find")
        .token("A", "a")
        .rule("target", vec!["A"])
        .start("target")
        .build();
    assert!(grammar.find_symbol_by_name("missing").is_none());
}

// ===========================================================================
// 4. Symbol count consistency (5 tests)
// ===========================================================================

#[test]
fn test_count_new_registry_has_eof() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());
}

#[test]
fn test_count_after_registrations() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    reg.register("b", terminal_meta());
    reg.register("c", terminal_meta());
    // 1 (EOF) + 3
    assert_eq!(reg.len(), 4);
}

#[test]
fn test_count_duplicate_does_not_increase() {
    let mut reg = SymbolRegistry::new();
    reg.register("dup", terminal_meta());
    reg.register("dup", terminal_meta());
    assert_eq!(reg.len(), 2); // EOF + "dup"
}

#[test]
fn test_count_matches_grammar_tokens_plus_rules() {
    let grammar = GrammarBuilder::new("cnt")
        .token("A", "a")
        .token("B", "b")
        .rule("r1", vec!["A"])
        .rule("r2", vec!["B"])
        .start("r1")
        .build();
    let reg = grammar.build_registry();
    // EOF + 2 tokens + 2 nonterminals = 5
    assert_eq!(reg.len(), 5);
}

#[test]
fn test_count_contains_id_valid_and_invalid() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("valid", terminal_meta());
    assert!(reg.contains_id(id));
    assert!(!reg.contains_id(SymbolId(9999)));
}

// ===========================================================================
// 5. Grammar with many symbols (5 tests)
// ===========================================================================

#[test]
fn test_many_tokens_100() {
    let mut builder = GrammarBuilder::new("big");
    for i in 0..100 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    builder = builder.rule("start", vec!["T0"]).start("start");
    let grammar = builder.build();
    let reg = grammar.build_registry();
    // EOF + 100 tokens + "start" nonterminal
    assert_eq!(reg.len(), 102);
}

#[test]
fn test_many_rules_50() {
    let mut builder = GrammarBuilder::new("rules50");
    builder = builder.token("X", "x");
    for i in 0..50 {
        builder = builder.rule(&format!("rule{i}"), vec!["X"]);
    }
    builder = builder.start("rule0");
    let grammar = builder.build();
    let reg = grammar.build_registry();
    for i in 0..50 {
        assert!(reg.get_id(&format!("rule{i}")).is_some(), "missing rule{i}");
    }
}

#[test]
fn test_many_symbols_ids_unique() {
    let mut reg = SymbolRegistry::new();
    let mut ids = Vec::new();
    for i in 0..200 {
        ids.push(reg.register(&format!("sym{i}"), terminal_meta()));
    }
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 200); // all unique
}

#[test]
fn test_many_symbols_reverse_lookup_all() {
    let mut reg = SymbolRegistry::new();
    for i in 0..50 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    for i in 0..50 {
        let id = reg.get_id(&format!("s{i}")).unwrap();
        assert_eq!(reg.get_name(id), Some(format!("s{i}").as_str()));
    }
}

#[test]
fn test_many_symbols_index_map_roundtrip() {
    let mut reg = SymbolRegistry::new();
    for i in 0..30 {
        reg.register(&format!("m{i}"), terminal_meta());
    }
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();
    for (&sym_id, &idx) in &idx_map {
        assert_eq!(sym_map[&idx], sym_id);
    }
}

// ===========================================================================
// 6. Serialization roundtrip (5 tests)
// ===========================================================================

#[test]
fn test_serde_registry_roundtrip_json() {
    let mut reg = SymbolRegistry::new();
    reg.register("plus", terminal_meta());
    reg.register("expr", nonterminal_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let deserialized: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, deserialized);
}

#[test]
fn test_serde_registry_preserves_ids() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.get_id("tok"), Some(id));
}

#[test]
fn test_serde_registry_preserves_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("hidden_tok", hidden_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    let meta = back.get_metadata(id).unwrap();
    assert!(meta.hidden);
    assert!(!meta.visible);
}

#[test]
fn test_serde_grammar_roundtrip_with_registry() {
    let mut grammar = GrammarBuilder::new("serde_g")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    grammar.symbol_registry = Some(grammar.build_registry());
    let json = serde_json::to_string(&grammar).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar, back);
}

#[test]
fn test_serde_grammar_none_registry_roundtrip() {
    let grammar = GrammarBuilder::new("none_reg")
        .token("B", "b")
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert!(grammar.symbol_registry.is_none());
    let json = serde_json::to_string(&grammar).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert!(back.symbol_registry.is_none());
}

// ===========================================================================
// 7. Builder creates correct mappings (8 tests)
// ===========================================================================

#[test]
fn test_builder_basic_grammar() {
    let grammar = GrammarBuilder::new("basic")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert_eq!(grammar.name, "basic");
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
}

#[test]
fn test_builder_rule_names_populated() {
    let grammar = GrammarBuilder::new("rn")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert!(grammar.rule_names.values().any(|v| v == "root"));
}

#[test]
fn test_builder_start_symbol_first_in_rules() {
    let grammar = GrammarBuilder::new("order")
        .token("X", "x")
        .rule("second", vec!["X"])
        .rule("first", vec!["X"])
        .start("first")
        .build();
    let first_key = grammar.rules.keys().next().unwrap();
    let first_name = grammar.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "first");
}

#[test]
fn test_builder_extras_recorded() {
    let grammar = GrammarBuilder::new("ext")
        .token("WS", r"\s+")
        .extra("WS")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert!(!grammar.extras.is_empty());
}

#[test]
fn test_builder_externals_recorded() {
    let grammar = GrammarBuilder::new("ext_scan")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert!(!grammar.externals.is_empty());
    assert_eq!(grammar.externals[0].name, "INDENT");
}

#[test]
fn test_builder_python_like_registry() {
    let grammar = GrammarBuilder::python_like();
    let reg = grammar.build_registry();
    assert!(reg.get_id("def").is_some());
    assert!(reg.get_id("module").is_some());
}

#[test]
fn test_builder_javascript_like_registry() {
    let grammar = GrammarBuilder::javascript_like();
    let reg = grammar.build_registry();
    assert!(reg.get_id("function").is_some());
    assert!(reg.get_id("program").is_some());
    assert!(reg.get_id("NUMBER").is_some());
}

#[test]
fn test_builder_get_or_build_registry_caches() {
    let mut grammar = GrammarBuilder::new("cache")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert!(grammar.symbol_registry.is_none());
    let _ = grammar.get_or_build_registry();
    assert!(grammar.symbol_registry.is_some());
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn test_edge_default_registry_has_eof() {
    let reg = SymbolRegistry::default();
    assert_eq!(reg.len(), 1);
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_edge_empty_grammar_registry() {
    let grammar = Grammar::new("empty".to_string());
    let reg = grammar.build_registry();
    // Only EOF
    assert_eq!(reg.len(), 1);
}

#[test]
fn test_edge_single_token_grammar() {
    let grammar = GrammarBuilder::new("single")
        .token("ONLY", "only")
        .rule("r", vec!["ONLY"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("ONLY").is_some());
    assert!(reg.get_id("r").is_some());
}

#[test]
fn test_edge_symbol_id_display() {
    let id = SymbolId(42);
    assert_eq!(format!("{id}"), "Symbol(42)");
}

#[test]
fn test_edge_symbol_id_ordering() {
    let a = SymbolId(1);
    let b = SymbolId(2);
    assert!(a < b);
    assert!(b > a);
    assert_eq!(SymbolId(5), SymbolId(5));
}

#[test]
fn test_edge_symbol_id_hash_key() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(SymbolId(1), "one");
    map.insert(SymbolId(2), "two");
    assert_eq!(map[&SymbolId(1)], "one");
    assert_eq!(map[&SymbolId(2)], "two");
}

#[test]
fn test_edge_registry_iter_order_matches_insertion() {
    let mut reg = SymbolRegistry::new();
    let names = ["alpha", "beta", "gamma", "delta"];
    for n in &names {
        reg.register(n, terminal_meta());
    }
    let iter_names: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    // First element is "end" (EOF)
    assert_eq!(iter_names[0], "end");
    for (i, n) in names.iter().enumerate() {
        assert_eq!(iter_names[i + 1], *n);
    }
}

#[test]
fn test_edge_metadata_update_on_re_register() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("sym", terminal_meta());
    let updated = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };
    let id2 = reg.register("sym", updated);
    assert_eq!(id, id2);
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.visible);
    assert!(meta.hidden);
}
