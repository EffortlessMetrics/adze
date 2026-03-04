//! Advanced tests for Symbol enum, SymbolMetadata, and SymbolId interactions.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, SymbolMetadata, Token,
    TokenPattern, builder::GrammarBuilder,
};

// ===========================================================================
// 1. SymbolId arithmetic and ordering
// ===========================================================================

#[test]
fn symbol_id_zero_is_smallest() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(0) <= SymbolId(0));
}

#[test]
fn symbol_id_max_is_largest() {
    assert!(SymbolId(u16::MAX) > SymbolId(u16::MAX - 1));
    assert!(SymbolId(u16::MAX) >= SymbolId(u16::MAX));
}

#[test]
fn symbol_id_ordering_transitive() {
    let a = SymbolId(10);
    let b = SymbolId(20);
    let c = SymbolId(30);
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}

#[test]
fn symbol_id_sort_vec() {
    let mut ids = vec![
        SymbolId(5),
        SymbolId(1),
        SymbolId(3),
        SymbolId(2),
        SymbolId(4),
    ];
    ids.sort();
    assert_eq!(
        ids,
        vec![
            SymbolId(1),
            SymbolId(2),
            SymbolId(3),
            SymbolId(4),
            SymbolId(5)
        ]
    );
}

#[test]
fn symbol_id_min_max() {
    let ids = [SymbolId(10), SymbolId(3), SymbolId(7)];
    assert_eq!(ids.iter().min(), Some(&SymbolId(3)));
    assert_eq!(ids.iter().max(), Some(&SymbolId(10)));
}

#[test]
fn symbol_id_consecutive_ordering() {
    for i in 0u16..100 {
        assert!(SymbolId(i) <= SymbolId(i));
        if i > 0 {
            assert!(SymbolId(i - 1) < SymbolId(i));
        }
    }
}

#[test]
fn symbol_id_reverse_sort() {
    let mut ids = vec![SymbolId(1), SymbolId(5), SymbolId(3)];
    ids.sort_by(|a, b| b.cmp(a));
    assert_eq!(ids, vec![SymbolId(5), SymbolId(3), SymbolId(1)]);
}

// ===========================================================================
// 2. SymbolId as HashMap / BTreeMap key
// ===========================================================================

#[test]
fn symbol_id_hashmap_insert_and_get() {
    let mut map = HashMap::new();
    map.insert(SymbolId(1), "one");
    map.insert(SymbolId(2), "two");
    assert_eq!(map.get(&SymbolId(1)), Some(&"one"));
    assert_eq!(map.get(&SymbolId(2)), Some(&"two"));
    assert_eq!(map.get(&SymbolId(3)), None);
}

#[test]
fn symbol_id_hashmap_overwrite() {
    let mut map = HashMap::new();
    map.insert(SymbolId(1), "first");
    map.insert(SymbolId(1), "second");
    assert_eq!(map.len(), 1);
    assert_eq!(map[&SymbolId(1)], "second");
}

#[test]
fn symbol_id_hashset_dedup() {
    let mut set = HashSet::new();
    set.insert(SymbolId(5));
    set.insert(SymbolId(5));
    set.insert(SymbolId(10));
    assert_eq!(set.len(), 2);
    assert!(set.contains(&SymbolId(5)));
}

#[test]
fn symbol_id_btreemap_ordered_iteration() {
    let mut map = BTreeMap::new();
    map.insert(SymbolId(30), "c");
    map.insert(SymbolId(10), "a");
    map.insert(SymbolId(20), "b");
    let keys: Vec<_> = map.keys().copied().collect();
    assert_eq!(keys, vec![SymbolId(10), SymbolId(20), SymbolId(30)]);
}

#[test]
fn symbol_id_btreeset_range() {
    let set: BTreeSet<_> = (0u16..10).map(SymbolId).collect();
    let range: Vec<_> = set.range(SymbolId(3)..SymbolId(7)).copied().collect();
    assert_eq!(
        range,
        vec![SymbolId(3), SymbolId(4), SymbolId(5), SymbolId(6)]
    );
}

#[test]
fn symbol_id_hashmap_many_entries() {
    let map: HashMap<SymbolId, u16> = (0u16..500).map(|i| (SymbolId(i), i * 2)).collect();
    assert_eq!(map.len(), 500);
    assert_eq!(map[&SymbolId(250)], 500);
}

// ===========================================================================
// 3. Symbol enum variant construction
// ===========================================================================

#[test]
fn symbol_terminal_construction() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(SymbolId(1))));
}

#[test]
fn symbol_nonterminal_construction() {
    let s = Symbol::NonTerminal(SymbolId(42));
    assert!(matches!(s, Symbol::NonTerminal(SymbolId(42))));
}

#[test]
fn symbol_external_construction() {
    let s = Symbol::External(SymbolId(100));
    assert!(matches!(s, Symbol::External(SymbolId(100))));
}

#[test]
fn symbol_optional_wraps_inner() {
    let inner = Symbol::Terminal(SymbolId(1));
    let opt = Symbol::Optional(Box::new(inner.clone()));
    assert_eq!(
        opt,
        Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))
    );
}

#[test]
fn symbol_repeat_wraps_inner() {
    let rep = Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(5))));
    if let Symbol::Repeat(inner) = &rep {
        assert_eq!(**inner, Symbol::NonTerminal(SymbolId(5)));
    } else {
        panic!("expected Repeat");
    }
}

#[test]
fn symbol_repeat_one_wraps_inner() {
    let rep = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(3))));
    if let Symbol::RepeatOne(inner) = &rep {
        assert_eq!(**inner, Symbol::Terminal(SymbolId(3)));
    } else {
        panic!("expected RepeatOne");
    }
}

#[test]
fn symbol_choice_multiple_variants() {
    let choice = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
        Symbol::NonTerminal(SymbolId(3)),
    ]);
    if let Symbol::Choice(alts) = &choice {
        assert_eq!(alts.len(), 3);
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn symbol_sequence_multiple() {
    let seq = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ]);
    if let Symbol::Sequence(parts) = &seq {
        assert_eq!(parts.len(), 2);
    } else {
        panic!("expected Sequence");
    }
}

#[test]
fn symbol_epsilon() {
    let e = Symbol::Epsilon;
    assert_eq!(e, Symbol::Epsilon);
}

#[test]
fn symbol_nested_optional_repeat() {
    let nested = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(
        SymbolId(1),
    )))));
    if let Symbol::Optional(inner) = &nested {
        assert!(matches!(**inner, Symbol::Repeat(_)));
    } else {
        panic!("expected Optional");
    }
}

#[test]
fn symbol_deeply_nested_choice_in_sequence() {
    let deep = Symbol::Sequence(vec![
        Symbol::Choice(vec![Symbol::Terminal(SymbolId(1)), Symbol::Epsilon]),
        Symbol::RepeatOne(Box::new(Symbol::External(SymbolId(99)))),
    ]);
    if let Symbol::Sequence(parts) = &deep {
        assert!(matches!(&parts[0], Symbol::Choice(_)));
        assert!(matches!(&parts[1], Symbol::RepeatOne(_)));
    } else {
        panic!("expected Sequence");
    }
}

// ===========================================================================
// 4. Symbol equality, clone, and hash
// ===========================================================================

#[test]
fn symbol_clone_equals_original() {
    let original = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Optional(Box::new(Symbol::NonTerminal(SymbolId(2)))),
    ]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn symbol_different_variants_not_equal() {
    let t = Symbol::Terminal(SymbolId(1));
    let nt = Symbol::NonTerminal(SymbolId(1));
    assert_ne!(t, nt);
}

#[test]
fn symbol_hash_consistent() {
    use std::hash::{Hash, Hasher};
    let s1 = Symbol::Terminal(SymbolId(42));
    let s2 = Symbol::Terminal(SymbolId(42));

    let hash_of = |s: &Symbol| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    };
    assert_eq!(hash_of(&s1), hash_of(&s2));
}

#[test]
fn symbol_in_hashset() {
    let mut set = HashSet::new();
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::NonTerminal(SymbolId(1)));
    assert_eq!(set.len(), 2);
}

#[test]
fn symbol_ordering_terminal_vs_nonterminal() {
    // Symbol derives Ord; just verify it doesn't panic
    let mut syms = vec![
        Symbol::NonTerminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(1)),
        Symbol::Epsilon,
    ];
    syms.sort();
    // Sorted order is deterministic (based on enum discriminant then inner value)
    assert_eq!(syms.len(), 3);
}

// ===========================================================================
// 5. SymbolMetadata construction and field access
// ===========================================================================

#[test]
fn symbol_metadata_visible_terminal() {
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(meta.terminal);
}

#[test]
fn symbol_metadata_hidden_nonterminal() {
    let meta = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };
    assert!(!meta.visible);
    assert!(meta.named);
    assert!(meta.hidden);
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
fn symbol_metadata_all_false() {
    let meta = SymbolMetadata {
        visible: false,
        named: false,
        hidden: false,
        terminal: false,
    };
    assert!(!meta.visible && !meta.named && !meta.hidden && !meta.terminal);
}

#[test]
fn symbol_metadata_equality() {
    let a = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let b = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let c = SymbolMetadata {
        visible: false,
        named: false,
        hidden: false,
        terminal: true,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn symbol_metadata_copy_semantics() {
    let a = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn symbol_metadata_clone() {
    let a = SymbolMetadata {
        visible: true,
        named: false,
        hidden: true,
        terminal: true,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

// ===========================================================================
// 6. SymbolMetadata serialization roundtrip
// ===========================================================================

#[test]
fn symbol_metadata_serde_roundtrip() {
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: true,
        terminal: true,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: SymbolMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta, back);
}

#[test]
fn symbol_id_serde_roundtrip() {
    let id = SymbolId(12345);
    let json = serde_json::to_string(&id).unwrap();
    let back: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

#[test]
fn symbol_enum_serde_roundtrip() {
    let sym = Symbol::Optional(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Epsilon,
    ])));
    let json = serde_json::to_string(&sym).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(sym, back);
}

// ===========================================================================
// 7. Grammar rule_names lookup
// ===========================================================================

#[test]
fn grammar_rule_names_insert_and_lookup() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.rule_names.insert(SymbolId(1), "expr".to_string());
    grammar.rule_names.insert(SymbolId(2), "stmt".to_string());
    assert_eq!(
        grammar.rule_names.get(&SymbolId(1)),
        Some(&"expr".to_string())
    );
    assert_eq!(
        grammar.rule_names.get(&SymbolId(2)),
        Some(&"stmt".to_string())
    );
    assert_eq!(grammar.rule_names.get(&SymbolId(3)), None);
}

#[test]
fn grammar_find_symbol_by_name() {
    let mut grammar = Grammar::new("test".to_string());
    grammar
        .rule_names
        .insert(SymbolId(10), "expression".to_string());
    grammar
        .rule_names
        .insert(SymbolId(20), "statement".to_string());
    assert_eq!(
        grammar.find_symbol_by_name("expression"),
        Some(SymbolId(10))
    );
    assert_eq!(grammar.find_symbol_by_name("statement"), Some(SymbolId(20)));
    assert_eq!(grammar.find_symbol_by_name("missing"), None);
}

#[test]
fn grammar_rule_names_iteration_order() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.rule_names.insert(SymbolId(3), "c".to_string());
    grammar.rule_names.insert(SymbolId(1), "a".to_string());
    grammar.rule_names.insert(SymbolId(2), "b".to_string());
    // IndexMap preserves insertion order
    let keys: Vec<_> = grammar.rule_names.keys().copied().collect();
    assert_eq!(keys, vec![SymbolId(3), SymbolId(1), SymbolId(2)]);
}

// ===========================================================================
// 8. Grammar tokens lookup
// ===========================================================================

#[test]
fn grammar_tokens_insert_and_lookup() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let tok = grammar.tokens.get(&SymbolId(1)).unwrap();
    assert_eq!(tok.name, "NUMBER");
    assert!(!tok.fragile);
}

#[test]
fn grammar_tokens_string_pattern() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(5),
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let tok = grammar.tokens.get(&SymbolId(5)).unwrap();
    assert!(matches!(&tok.pattern, TokenPattern::String(s) if s == "+"));
}

#[test]
fn grammar_tokens_fragile_flag() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(7),
        Token {
            name: "KEYWORD".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: true,
        },
    );
    assert!(grammar.tokens.get(&SymbolId(7)).unwrap().fragile);
}

#[test]
fn grammar_tokens_missing_key() {
    let grammar = Grammar::new("test".to_string());
    assert!(grammar.tokens.get(&SymbolId(999)).is_none());
}

// ===========================================================================
// 9. Symbol debug format
// ===========================================================================

#[test]
fn symbol_id_debug_format() {
    let id = SymbolId(42);
    let dbg = format!("{:?}", id);
    assert!(dbg.contains("42"), "debug should contain the inner value");
}

#[test]
fn symbol_id_display_format() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
    assert_eq!(
        format!("{}", SymbolId(u16::MAX)),
        format!("Symbol({})", u16::MAX)
    );
}

#[test]
fn symbol_terminal_debug_format() {
    let s = Symbol::Terminal(SymbolId(7));
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("7"));
}

#[test]
fn symbol_epsilon_debug_format() {
    let dbg = format!("{:?}", Symbol::Epsilon);
    assert!(dbg.contains("Epsilon"));
}

#[test]
fn symbol_metadata_debug_format() {
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let dbg = format!("{:?}", meta);
    assert!(dbg.contains("visible"));
    assert!(dbg.contains("true"));
}

// ===========================================================================
// 10. SymbolId edge cases (0, u16::MAX)
// ===========================================================================

#[test]
fn symbol_id_zero() {
    let id = SymbolId(0);
    assert_eq!(id.0, 0);
    assert_eq!(id, SymbolId(0));
}

#[test]
fn symbol_id_max_value() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, 65535);
}

#[test]
fn symbol_id_zero_in_hashmap() {
    let mut map = HashMap::new();
    map.insert(SymbolId(0), "eof");
    assert_eq!(map[&SymbolId(0)], "eof");
}

#[test]
fn symbol_id_max_in_btreeset() {
    let mut set = BTreeSet::new();
    set.insert(SymbolId(u16::MAX));
    set.insert(SymbolId(0));
    let first = *set.iter().next().unwrap();
    let last = *set.iter().next_back().unwrap();
    assert_eq!(first, SymbolId(0));
    assert_eq!(last, SymbolId(u16::MAX));
}

#[test]
fn symbol_id_adjacent_values_not_equal() {
    assert_ne!(SymbolId(0), SymbolId(1));
    assert_ne!(SymbolId(u16::MAX - 1), SymbolId(u16::MAX));
}

// ===========================================================================
// 11. Multiple SymbolIds in collections
// ===========================================================================

#[test]
fn multiple_symbol_ids_vec_contains() {
    let ids = vec![SymbolId(1), SymbolId(2), SymbolId(3)];
    assert!(ids.contains(&SymbolId(2)));
    assert!(!ids.contains(&SymbolId(4)));
}

#[test]
fn multiple_symbol_ids_dedup() {
    let mut ids = vec![
        SymbolId(1),
        SymbolId(2),
        SymbolId(1),
        SymbolId(3),
        SymbolId(2),
    ];
    ids.sort();
    ids.dedup();
    assert_eq!(ids, vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
}

#[test]
fn symbol_ids_as_rule_rhs() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(4)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.rhs.len(), 3);
    assert_eq!(rule.lhs, SymbolId(1));
}

#[test]
fn symbol_ids_in_grammar_extras() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.extras.push(SymbolId(100));
    grammar.extras.push(SymbolId(101));
    assert_eq!(grammar.extras.len(), 2);
    assert!(grammar.extras.contains(&SymbolId(100)));
}

#[test]
fn symbol_ids_in_grammar_supertypes() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.supertypes.push(SymbolId(50));
    assert_eq!(grammar.supertypes.len(), 1);
    assert_eq!(grammar.supertypes[0], SymbolId(50));
}

#[test]
fn symbol_ids_in_grammar_inline_rules() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.inline_rules.push(SymbolId(10));
    grammar.inline_rules.push(SymbolId(20));
    assert_eq!(grammar.inline_rules, vec![SymbolId(10), SymbolId(20)]);
}

// ===========================================================================
// 12. GrammarBuilder interactions with SymbolId
// ===========================================================================

#[test]
fn builder_assigns_symbol_ids() {
    let grammar = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .build();
    // Builder starts at SymbolId(1), so both NUM and expr should have IDs
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_rule_names_populated() {
    let grammar = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .build();
    // "expr" should be in rule_names
    let has_expr = grammar.rule_names.values().any(|v| v == "expr");
    assert!(has_expr, "rule_names should contain 'expr'");
}

#[test]
fn builder_find_symbol_by_name_works() {
    let grammar = GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .rule("value", vec!["NUM"])
        .build();
    assert!(grammar.find_symbol_by_name("value").is_some());
}

#[test]
fn builder_tokens_accessible() {
    let grammar = GrammarBuilder::new("test")
        .token("PLUS", "+")
        .token("MINUS", "-")
        .rule("op", vec!["PLUS"])
        .build();
    // At least one token should exist
    assert!(grammar.tokens.len() >= 1);
}

// ===========================================================================
// 13. Grammar add_rule and get_rules_for_symbol
// ===========================================================================

#[test]
fn grammar_add_and_get_rules() {
    let mut grammar = Grammar::new("test".to_string());
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule);
    let rules = grammar.get_rules_for_symbol(SymbolId(1)).unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn grammar_multiple_rules_same_lhs() {
    let mut grammar = Grammar::new("test".to_string());
    for i in 0..3 {
        grammar.add_rule(Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(10 + i))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
    }
    assert_eq!(grammar.get_rules_for_symbol(SymbolId(1)).unwrap().len(), 3);
}

#[test]
fn grammar_get_rules_for_missing_symbol() {
    let grammar = Grammar::new("test".to_string());
    assert!(grammar.get_rules_for_symbol(SymbolId(999)).is_none());
}

#[test]
fn grammar_all_rules_iterator() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(10))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(11))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    assert_eq!(grammar.all_rules().count(), 2);
}

// ===========================================================================
// 14. Symbol interactions with FieldId
// ===========================================================================

#[test]
fn rule_with_field_mappings() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.fields[0], (FieldId(0), 0));
    assert_eq!(rule.fields[1], (FieldId(1), 1));
}

// ===========================================================================
// 15. ExternalToken and Symbol::External
// ===========================================================================

#[test]
fn external_token_symbol_id_matches() {
    let ext = ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(200),
    };
    let sym = Symbol::External(ext.symbol_id);
    assert!(matches!(sym, Symbol::External(SymbolId(200))));
}

#[test]
fn grammar_externals_contain_symbol_ids() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.externals.push(ExternalToken {
        name: "newline".to_string(),
        symbol_id: SymbolId(300),
    });
    assert_eq!(grammar.externals[0].symbol_id, SymbolId(300));
}

// ===========================================================================
// 16. Cross-cutting: metadata + symbol + grammar interactions
// ===========================================================================

#[test]
fn build_registry_assigns_metadata_for_tokens() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let registry = grammar.build_registry();
    // Registry should have at least one entry
    assert!(registry.len() > 0);
}

#[test]
fn build_registry_hidden_for_extras() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "WS".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    grammar.extras.push(SymbolId(1));
    let registry = grammar.build_registry();
    // The WS token should be marked hidden because it's in extras
    let id = registry.get_id("WS");
    assert!(id.is_some());
    let meta = registry.get_metadata(id.unwrap()).unwrap();
    assert!(meta.hidden);
}

#[test]
fn symbol_metadata_terminal_for_tokens_nonterminal_for_rules() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "ID".to_string(),
            pattern: TokenPattern::Regex("[a-z]+".to_string()),
            fragile: false,
        },
    );
    grammar
        .rule_names
        .insert(SymbolId(2), "program".to_string());
    let registry = grammar.build_registry();

    if let Some(id) = registry.get_id("ID") {
        let meta = registry.get_metadata(id).unwrap();
        assert!(meta.terminal);
    }
    if let Some(id) = registry.get_id("program") {
        let meta = registry.get_metadata(id).unwrap();
        assert!(!meta.terminal);
    }
}

#[test]
fn symbol_metadata_underscore_prefix_hidden() {
    let mut grammar = Grammar::new("test".to_string());
    grammar
        .rule_names
        .insert(SymbolId(1), "_hidden_rule".to_string());
    let registry = grammar.build_registry();
    if let Some(id) = registry.get_id("_hidden_rule") {
        let meta = registry.get_metadata(id).unwrap();
        assert!(meta.hidden);
        assert!(!meta.visible);
    }
}

#[test]
fn grammar_serde_roundtrip_with_rule_names() {
    let mut grammar = Grammar::new("serde_test".to_string());
    grammar.rule_names.insert(SymbolId(1), "expr".to_string());
    grammar.rule_names.insert(SymbolId(2), "term".to_string());
    let json = serde_json::to_string(&grammar).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rule_names.get(&SymbolId(1)), Some(&"expr".to_string()));
    assert_eq!(back.rule_names.get(&SymbolId(2)), Some(&"term".to_string()));
}

#[test]
fn grammar_default_is_empty() {
    let grammar = Grammar::default();
    assert!(grammar.name.is_empty());
    assert!(grammar.rules.is_empty());
    assert!(grammar.tokens.is_empty());
    assert!(grammar.rule_names.is_empty());
}
