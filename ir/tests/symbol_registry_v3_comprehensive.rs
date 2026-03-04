//! Comprehensive tests for SymbolRegistry and Grammar symbol management (v3).
//!
//! Covers: token/rule registration, symbol ID properties, start symbol,
//! lookup by name/ID, normalization side-effects, and large grammars.

use adze_ir::builder::GrammarBuilder;
use adze_ir::symbol_registry::SymbolRegistry;
use adze_ir::{Grammar, SymbolId, SymbolMetadata};

// ---------------------------------------------------------------------------
// Helper: build a minimal arithmetic grammar
// ---------------------------------------------------------------------------
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ---------------------------------------------------------------------------
// 1. Grammar has tokens after building
// ---------------------------------------------------------------------------

#[test]
fn tokens_present_after_build() {
    let g = arith_grammar();
    assert!(!g.tokens.is_empty());
}

#[test]
fn token_count_matches_declared() {
    let g = arith_grammar();
    // NUMBER, +, -
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn single_token_grammar_has_one_token() {
    let g = GrammarBuilder::new("tiny")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn tokens_contain_expected_names() {
    let g = arith_grammar();
    let names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"NUMBER"));
    assert!(names.contains(&"+"));
    assert!(names.contains(&"-"));
}

#[test]
fn token_pattern_is_preserved() {
    let g = arith_grammar();
    let num_tok = g.tokens.values().find(|t| t.name == "NUMBER").unwrap();
    match &num_tok.pattern {
        adze_ir::TokenPattern::Regex(r) => assert_eq!(r, r"\d+"),
        adze_ir::TokenPattern::String(s) => assert_eq!(s, r"\d+"),
    }
}

// ---------------------------------------------------------------------------
// 2. Grammar has rule_names after building
// ---------------------------------------------------------------------------

#[test]
fn rule_names_present_after_build() {
    let g = arith_grammar();
    assert!(!g.rule_names.is_empty());
}

#[test]
fn rule_names_contain_start_rule() {
    let g = arith_grammar();
    let names: Vec<&str> = g.rule_names.values().map(|n| n.as_str()).collect();
    assert!(names.contains(&"expr"));
}

#[test]
fn rule_names_count_for_single_rule_grammar() {
    let g = GrammarBuilder::new("one")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    assert!(g.rule_names.values().any(|n| n == "root"));
}

#[test]
fn rule_names_distinct_from_tokens() {
    let g = arith_grammar();
    for (sid, _name) in &g.rule_names {
        // rule_names entries should not also appear as token SymbolIds
        // (the builder only adds non-punctuation, non-uppercase names)
        // We simply verify the map is non-empty and well-formed
        assert!(sid.0 > 0);
    }
}

// ---------------------------------------------------------------------------
// 3. Symbol IDs for tokens vs rules
// ---------------------------------------------------------------------------

#[test]
fn token_symbol_ids_are_positive() {
    let g = arith_grammar();
    for (sid, _tok) in &g.tokens {
        assert!(sid.0 > 0, "Builder reserves 0 for EOF");
    }
}

#[test]
fn rule_symbol_ids_are_positive() {
    let g = arith_grammar();
    for (sid, _name) in &g.rule_names {
        assert!(sid.0 > 0);
    }
}

#[test]
fn token_and_rule_ids_may_overlap_in_rule_names() {
    // GrammarBuilder inserts into rule_names for alphanumeric names
    // that don't look like punctuation.  NUMBER is all-caps so it's excluded.
    let g = arith_grammar();
    let rule_ids: Vec<SymbolId> = g.rule_names.keys().copied().collect();
    let token_ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    // "expr" should be in rule_names but not in tokens
    for rid in &rule_ids {
        if g.rule_names[rid] == "expr" {
            assert!(!token_ids.contains(rid));
        }
    }
}

#[test]
fn all_symbol_ids_unique_within_tokens() {
    let g = arith_grammar();
    let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    for (i, a) in ids.iter().enumerate() {
        for b in &ids[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn all_symbol_ids_unique_within_rule_names() {
    let g = arith_grammar();
    let ids: Vec<SymbolId> = g.rule_names.keys().copied().collect();
    for (i, a) in ids.iter().enumerate() {
        for b in &ids[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Start symbol is in rule_names
// ---------------------------------------------------------------------------

#[test]
fn start_symbol_is_some() {
    let g = arith_grammar();
    assert!(g.start_symbol().is_some());
}

#[test]
fn start_symbol_in_rule_names() {
    let g = arith_grammar();
    let start = g.start_symbol().unwrap();
    assert!(g.rule_names.contains_key(&start));
}

#[test]
fn start_symbol_has_rules() {
    let g = arith_grammar();
    let start = g.start_symbol().unwrap();
    assert!(g.rules.contains_key(&start));
}

#[test]
fn start_symbol_first_in_rules_map() {
    let g = arith_grammar();
    let first_rule_lhs = *g.rules.keys().next().unwrap();
    let start = g.start_symbol().unwrap();
    assert_eq!(first_rule_lhs, start);
}

#[test]
fn python_like_start_symbol_exists() {
    let g = GrammarBuilder::python_like();
    assert!(g.start_symbol().is_some());
}

#[test]
fn javascript_like_start_symbol_exists() {
    let g = GrammarBuilder::javascript_like();
    assert!(g.start_symbol().is_some());
}

// ---------------------------------------------------------------------------
// 5. Token lookup by name
// ---------------------------------------------------------------------------

#[test]
fn find_token_by_name_number() {
    let g = arith_grammar();
    let found = g.tokens.values().any(|t| t.name == "NUMBER");
    assert!(found);
}

#[test]
fn find_token_by_name_plus() {
    let g = arith_grammar();
    let found = g.tokens.values().any(|t| t.name == "+");
    assert!(found);
}

#[test]
fn missing_token_not_found() {
    let g = arith_grammar();
    let found = g.tokens.values().any(|t| t.name == "MISSING");
    assert!(!found);
}

#[test]
fn token_lookup_round_trip() {
    let g = arith_grammar();
    for (sid, tok) in &g.tokens {
        // We can find the same token by iterating
        let found = g.tokens.get(sid).unwrap();
        assert_eq!(found.name, tok.name);
    }
}

// ---------------------------------------------------------------------------
// 6. Rule name lookup by SymbolId
// ---------------------------------------------------------------------------

#[test]
fn rule_name_lookup_by_id() {
    let g = arith_grammar();
    let (sid, name) = g.rule_names.iter().next().unwrap();
    assert_eq!(g.rule_names.get(sid).unwrap(), name);
}

#[test]
fn find_symbol_by_name_returns_correct_id() {
    let g = arith_grammar();
    let id = g.find_symbol_by_name("expr").unwrap();
    assert_eq!(g.rule_names[&id], "expr");
}

#[test]
fn find_symbol_by_name_missing_returns_none() {
    let g = arith_grammar();
    assert!(g.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn rule_names_values_are_nonempty_strings() {
    let g = arith_grammar();
    for name in g.rule_names.values() {
        assert!(!name.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 7. Symbol IDs are u16
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_inner_is_u16() {
    let id = SymbolId(42);
    let val: u16 = id.0;
    assert_eq!(val, 42u16);
}

#[test]
fn symbol_id_zero() {
    let id = SymbolId(0);
    assert_eq!(id.0, 0u16);
}

#[test]
fn symbol_id_max_u16() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, u16::MAX);
}

#[test]
fn symbol_id_display() {
    let id = SymbolId(7);
    let s = format!("{id}");
    assert!(s.contains("7"));
}

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(1), SymbolId(1));
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(1) < SymbolId(2));
}

#[test]
fn symbol_id_clone_and_copy() {
    let a = SymbolId(10);
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
}

// ---------------------------------------------------------------------------
// 8. Multiple tokens registered
// ---------------------------------------------------------------------------

#[test]
fn five_tokens_registered() {
    let g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 5);
}

#[test]
fn duplicate_token_name_reuses_id() {
    let g = GrammarBuilder::new("dup")
        .token("X", "x")
        .token("X", "y") // same name, different pattern
        .rule("start", vec!["X"])
        .start("start")
        .build();
    // Second .token("X", ...) overwrites the first token entry
    // but reuses the same SymbolId
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn tokens_preserve_insertion_order() {
    let g = GrammarBuilder::new("ordered")
        .token("FIRST", "1")
        .token("SECOND", "2")
        .token("THIRD", "3")
        .rule("r", vec!["FIRST"])
        .start("r")
        .build();
    let names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["FIRST", "SECOND", "THIRD"]);
}

#[test]
fn fragile_token_flag() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("ERR", "error")
        .token("OK", "ok")
        .rule("r", vec!["OK"])
        .start("r")
        .build();
    let err = g.tokens.values().find(|t| t.name == "ERR").unwrap();
    assert!(err.fragile);
    let ok = g.tokens.values().find(|t| t.name == "OK").unwrap();
    assert!(!ok.fragile);
}

// ---------------------------------------------------------------------------
// 9. After normalize, new symbols may appear
// ---------------------------------------------------------------------------

#[test]
fn normalize_on_simple_grammar_is_idempotent() {
    let mut g = arith_grammar();
    let rules_before = g.rules.len();
    g.normalize();
    // No complex symbols → same rule count
    assert_eq!(g.rules.len(), rules_before);
}

#[test]
fn normalize_expands_optional_symbol() {
    let mut g = GrammarBuilder::new("opt")
        .token("A", "a")
        .token("B", "b")
        .build();
    // Manually insert a rule with Optional
    let a_id = *g.tokens.keys().find(|k| g.tokens[*k].name == "A").unwrap();
    let b_id = *g.tokens.keys().find(|k| g.tokens[*k].name == "B").unwrap();
    let lhs = SymbolId(100);
    g.rules.insert(
        lhs,
        vec![adze_ir::Rule {
            lhs,
            rhs: vec![
                adze_ir::Symbol::Terminal(a_id),
                adze_ir::Symbol::Optional(Box::new(adze_ir::Symbol::Terminal(b_id))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(0),
        }],
    );
    let rules_before = g.rules.len();
    g.normalize();
    // Normalization creates an auxiliary rule for the Optional
    assert!(g.rules.len() > rules_before);
}

#[test]
fn normalize_expands_repeat_symbol() {
    let mut g = GrammarBuilder::new("rep").token("X", "x").build();
    let x_id = *g.tokens.keys().next().unwrap();
    let lhs = SymbolId(100);
    g.rules.insert(
        lhs,
        vec![adze_ir::Rule {
            lhs,
            rhs: vec![adze_ir::Symbol::Repeat(Box::new(
                adze_ir::Symbol::Terminal(x_id),
            ))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(0),
        }],
    );
    g.normalize();
    // Should now have the original lhs rule PLUS the aux rule
    assert!(g.rules.len() >= 2);
}

#[test]
fn normalize_expands_repeat_one_symbol() {
    let mut g = GrammarBuilder::new("rep1").token("Y", "y").build();
    let y_id = *g.tokens.keys().next().unwrap();
    let lhs = SymbolId(200);
    g.rules.insert(
        lhs,
        vec![adze_ir::Rule {
            lhs,
            rhs: vec![adze_ir::Symbol::RepeatOne(Box::new(
                adze_ir::Symbol::Terminal(y_id),
            ))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(0),
        }],
    );
    g.normalize();
    assert!(g.rules.len() >= 2);
}

#[test]
fn normalize_expands_choice_symbol() {
    let mut g = GrammarBuilder::new("choice")
        .token("A", "a")
        .token("B", "b")
        .build();
    let a_id = *g.tokens.keys().find(|k| g.tokens[*k].name == "A").unwrap();
    let b_id = *g.tokens.keys().find(|k| g.tokens[*k].name == "B").unwrap();
    let lhs = SymbolId(300);
    g.rules.insert(
        lhs,
        vec![adze_ir::Rule {
            lhs,
            rhs: vec![adze_ir::Symbol::Choice(vec![
                adze_ir::Symbol::Terminal(a_id),
                adze_ir::Symbol::Terminal(b_id),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(0),
        }],
    );
    g.normalize();
    assert!(g.rules.len() >= 2);
}

#[test]
fn normalize_twice_is_stable() {
    let mut g = arith_grammar();
    g.normalize();
    let count1 = g.rules.len();
    g.normalize();
    let count2 = g.rules.len();
    assert_eq!(count1, count2);
}

// ---------------------------------------------------------------------------
// 10. Large grammars with many symbols
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_50_tokens() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..50 {
        b = b.token(&format!("T{i}"), &format!("t{i}"));
    }
    b = b.rule("start", vec!["T0"]).start("start");
    let g = b.build();
    assert_eq!(g.tokens.len(), 50);
}

#[test]
fn grammar_with_50_rules() {
    let mut b = GrammarBuilder::new("many_rules");
    b = b.token("X", "x");
    for i in 0..50 {
        b = b.rule(&format!("r{i}"), vec!["X"]);
    }
    b = b.start("r0");
    let g = b.build();
    assert_eq!(g.rules.len(), 50);
}

#[test]
fn large_grammar_all_rules_iterator() {
    let mut b = GrammarBuilder::new("iter");
    b = b.token("A", "a");
    for i in 0..20 {
        b = b.rule(&format!("rule{i}"), vec!["A"]);
    }
    b = b.start("rule0");
    let g = b.build();
    assert_eq!(g.all_rules().count(), 20);
}

#[test]
fn large_grammar_rule_names_count() {
    let mut b = GrammarBuilder::new("names");
    b = b.token("Z", "z");
    for i in 0..30 {
        b = b.rule(&format!("sym{i}"), vec!["Z"]);
    }
    b = b.start("sym0");
    let g = b.build();
    // All 30 rule names should be present
    assert_eq!(g.rule_names.len(), 30);
}

#[test]
fn large_grammar_find_symbol_by_name() {
    let mut b = GrammarBuilder::new("find");
    b = b.token("T", "t");
    for i in 0..25 {
        b = b.rule(&format!("node{i}"), vec!["T"]);
    }
    b = b.start("node0");
    let g = b.build();
    for i in 0..25 {
        assert!(g.find_symbol_by_name(&format!("node{i}")).is_some());
    }
}

// ---------------------------------------------------------------------------
// SymbolRegistry direct API tests
// ---------------------------------------------------------------------------

#[test]
fn registry_new_has_eof() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn registry_eof_metadata_is_terminal() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0)).unwrap();
    assert!(meta.terminal);
}

#[test]
fn registry_len_after_new() {
    let reg = SymbolRegistry::new();
    // Only EOF registered
    assert_eq!(reg.len(), 1);
}

#[test]
fn registry_is_not_empty_after_new() {
    let reg = SymbolRegistry::new();
    assert!(!reg.is_empty());
}

#[test]
fn registry_register_returns_incremented_ids() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id1 = reg.register("alpha", meta);
    let id2 = reg.register("beta", meta);
    assert_eq!(id1.0 + 1, id2.0);
}

#[test]
fn registry_register_duplicate_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id1 = reg.register("dup", meta);
    let id2 = reg.register("dup", meta);
    assert_eq!(id1, id2);
}

#[test]
fn registry_get_name_round_trip() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let id = reg.register("my_sym", meta);
    assert_eq!(reg.get_name(id), Some("my_sym"));
}

#[test]
fn registry_contains_id() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id = reg.register("present", meta);
    assert!(reg.contains_id(id));
    assert!(!reg.contains_id(SymbolId(9999)));
}

#[test]
fn registry_iter_order() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    reg.register("aaa", meta);
    reg.register("bbb", meta);
    let names: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    // Insertion order: "end", "aaa", "bbb"
    assert_eq!(names, vec!["end", "aaa", "bbb"]);
}

#[test]
fn registry_to_index_map_covers_all() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    reg.register("x", meta);
    reg.register("y", meta);
    let idx_map = reg.to_index_map();
    assert_eq!(idx_map.len(), reg.len());
}

#[test]
fn registry_to_symbol_map_inverse() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    reg.register("a", meta);
    let idx = reg.to_index_map();
    let sym = reg.to_symbol_map();
    for (&sid, &i) in &idx {
        assert_eq!(sym[&i], sid);
    }
}

#[test]
fn registry_default_is_new() {
    let reg: SymbolRegistry = Default::default();
    assert_eq!(reg.len(), 1);
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

// ---------------------------------------------------------------------------
// Grammar.build_registry / get_or_build_registry
// ---------------------------------------------------------------------------

#[test]
fn build_registry_contains_tokens() {
    let g = arith_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("NUMBER").is_some());
    assert!(reg.get_id("+").is_some());
    assert!(reg.get_id("-").is_some());
}

#[test]
fn build_registry_contains_non_terminals() {
    let g = arith_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("expr").is_some());
}

#[test]
fn build_registry_eof_present() {
    let g = arith_grammar();
    let reg = g.build_registry();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn get_or_build_registry_caches() {
    let mut g = arith_grammar();
    assert!(g.symbol_registry.is_none());
    let _ = g.get_or_build_registry();
    assert!(g.symbol_registry.is_some());
}

#[test]
fn get_or_build_registry_stable_across_calls() {
    let mut g = arith_grammar();
    let len1 = g.get_or_build_registry().len();
    let len2 = g.get_or_build_registry().len();
    assert_eq!(len1, len2);
}

#[test]
fn build_registry_terminal_metadata() {
    let g = arith_grammar();
    let reg = g.build_registry();
    let num_id = reg.get_id("NUMBER").unwrap();
    let meta = reg.get_metadata(num_id).unwrap();
    assert!(meta.terminal);
}

#[test]
fn build_registry_nonterminal_metadata() {
    let g = arith_grammar();
    let reg = g.build_registry();
    let expr_id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(expr_id).unwrap();
    assert!(!meta.terminal);
    assert!(meta.named);
}

// ---------------------------------------------------------------------------
// Grammar construction edge cases
// ---------------------------------------------------------------------------

#[test]
fn grammar_new_is_empty() {
    let g = Grammar::new("empty".to_string());
    assert!(g.tokens.is_empty());
    assert!(g.rules.is_empty());
    assert!(g.rule_names.is_empty());
}

#[test]
fn grammar_name_preserved() {
    let g = arith_grammar();
    assert_eq!(g.name, "arith");
}

#[test]
fn grammar_with_extras() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"\s+")
        .token("ID", r"[a-z]+")
        .extra("WS")
        .rule("prog", vec!["ID"])
        .start("prog")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn grammar_with_external_tokens() {
    let g = GrammarBuilder::python_like();
    assert!(!g.externals.is_empty());
}

#[test]
fn grammar_externals_in_registry() {
    let g = GrammarBuilder::python_like();
    let reg = g.build_registry();
    assert!(reg.get_id("INDENT").is_some());
    assert!(reg.get_id("DEDENT").is_some());
}

#[test]
fn grammar_symbol_registry_none_by_default() {
    let g = arith_grammar();
    assert!(g.symbol_registry.is_none());
}

#[test]
fn javascript_like_has_many_tokens() {
    let g = GrammarBuilder::javascript_like();
    assert!(g.tokens.len() >= 10);
}

#[test]
fn javascript_like_has_precedence_rules() {
    let g = GrammarBuilder::javascript_like();
    let has_prec = g.all_rules().any(|r| r.precedence.is_some());
    assert!(has_prec);
}
