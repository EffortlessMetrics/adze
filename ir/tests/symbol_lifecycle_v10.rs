//! Comprehensive tests for Symbol creation, ID assignment, and lifecycle in adze-ir.

use std::collections::{HashMap, HashSet};

use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, Symbol, SymbolId};

// ═══════════════════════════════════════════════════════════════════════════
// Group 1: SymbolId creation from u16
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_creation_from_zero() {
    let id = SymbolId(0);
    assert_eq!(id.0, 0);
}

#[test]
fn test_symbol_id_creation_from_one() {
    let id = SymbolId(1);
    assert_eq!(id.0, 1);
}

#[test]
fn test_symbol_id_creation_from_arbitrary_value() {
    let id = SymbolId(42);
    assert_eq!(id.0, 42);
}

#[test]
fn test_symbol_id_creation_from_u16_max() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, u16::MAX);
}

#[test]
fn test_symbol_id_creation_from_midrange() {
    let id = SymbolId(32768);
    assert_eq!(id.0, 32768);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 2: SymbolId equality and ordering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_equality_same_value() {
    assert_eq!(SymbolId(5), SymbolId(5));
}

#[test]
fn test_symbol_id_inequality_different_values() {
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn test_symbol_id_ordering_less_than() {
    assert!(SymbolId(1) < SymbolId(2));
}

#[test]
fn test_symbol_id_ordering_greater_than() {
    assert!(SymbolId(100) > SymbolId(10));
}

#[test]
fn test_symbol_id_ordering_equal() {
    assert!(SymbolId(7) <= SymbolId(7));
    assert!(SymbolId(7) >= SymbolId(7));
}

#[test]
fn test_symbol_id_ord_consistency_with_partial_ord() {
    let a = SymbolId(3);
    let b = SymbolId(5);
    assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

#[test]
fn test_symbol_id_zero_is_smallest() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(0) < SymbolId(u16::MAX));
}

#[test]
fn test_symbol_id_max_is_largest() {
    assert!(SymbolId(u16::MAX) > SymbolId(0));
    assert!(SymbolId(u16::MAX) > SymbolId(u16::MAX - 1));
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 3: SymbolId hashing (use in HashMap/HashSet)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_in_hashset() {
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_symbol_id_in_hashmap_as_key() {
    let mut map = HashMap::new();
    map.insert(SymbolId(10), "terminal");
    map.insert(SymbolId(20), "nonterminal");
    assert_eq!(map[&SymbolId(10)], "terminal");
    assert_eq!(map[&SymbolId(20)], "nonterminal");
}

#[test]
fn test_symbol_id_hashset_contains() {
    let set: HashSet<SymbolId> = [SymbolId(0), SymbolId(5), SymbolId(10)]
        .into_iter()
        .collect();
    assert!(set.contains(&SymbolId(5)));
    assert!(!set.contains(&SymbolId(6)));
}

#[test]
fn test_symbol_id_hashmap_overwrite() {
    let mut map = HashMap::new();
    map.insert(SymbolId(1), "first");
    map.insert(SymbolId(1), "second");
    assert_eq!(map.len(), 1);
    assert_eq!(map[&SymbolId(1)], "second");
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 4: SymbolId Copy semantics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_copy_semantics() {
    let a = SymbolId(42);
    let b = a; // Copy, not move
    assert_eq!(a, b);
    assert_eq!(a.0, 42);
}

#[test]
fn test_symbol_id_copy_into_function() {
    fn consume(id: SymbolId) -> u16 {
        id.0
    }
    let id = SymbolId(99);
    let val = consume(id);
    // id is still usable because SymbolId is Copy
    assert_eq!(val, id.0);
}

#[test]
fn test_symbol_id_copy_in_vec() {
    let id = SymbolId(7);
    let v = [id, id, id];
    assert_eq!(v.len(), 3);
    assert_eq!(v[0], id);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 5: SymbolId Debug formatting
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_debug_format() {
    let id = SymbolId(42);
    let debug = format!("{:?}", id);
    assert!(
        debug.contains("42"),
        "Debug output should contain the ID value"
    );
}

#[test]
fn test_symbol_id_display_format() {
    let id = SymbolId(5);
    let display = format!("{}", id);
    assert_eq!(display, "Symbol(5)");
}

#[test]
fn test_symbol_id_display_zero() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
}

#[test]
fn test_symbol_id_display_max() {
    assert_eq!(
        format!("{}", SymbolId(u16::MAX)),
        format!("Symbol({})", u16::MAX)
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 6: Tokens get assigned SymbolIds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_token_gets_symbol_id() {
    let grammar = GrammarBuilder::new("sl_v10_tok1")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    assert!(!grammar.tokens.is_empty());
    let first_token_id = *grammar.tokens.keys().next().unwrap();
    assert!(first_token_id.0 > 0, "Token IDs start after EOF (0)");
}

#[test]
fn test_multiple_tokens_get_distinct_ids() {
    let grammar = GrammarBuilder::new("sl_v10_tok2")
        .token("num", r"\d+")
        .token("op", r"\+")
        .rule("expr", vec!["num", "op", "num"])
        .start("expr")
        .build();
    let token_ids: Vec<SymbolId> = grammar.tokens.keys().copied().collect();
    assert_eq!(token_ids.len(), 2);
    assert_ne!(token_ids[0], token_ids[1]);
}

#[test]
fn test_token_id_accessible_via_keys() {
    let grammar = GrammarBuilder::new("sl_v10_tok3")
        .token("lit", "x")
        .rule("root", vec!["lit"])
        .start("root")
        .build();
    let ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    assert_eq!(ids.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 7: Non-terminals get SymbolIds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_nonterminal_gets_symbol_id() {
    let grammar = GrammarBuilder::new("sl_v10_nt1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let start_id = grammar.find_symbol_by_name("start");
    assert!(start_id.is_some());
}

#[test]
fn test_multiple_nonterminals_get_distinct_ids() {
    let grammar = GrammarBuilder::new("sl_v10_nt2")
        .token("x", "x")
        .rule("alpha", vec!["x"])
        .rule("beta", vec!["alpha"])
        .start("alpha")
        .build();
    let alpha_id = grammar.find_symbol_by_name("alpha").unwrap();
    let beta_id = grammar.find_symbol_by_name("beta").unwrap();
    assert_ne!(alpha_id, beta_id);
}

#[test]
fn test_nonterminal_in_rule_names() {
    let grammar = GrammarBuilder::new("sl_v10_nt3")
        .token("t", "t")
        .rule("root", vec!["t"])
        .start("root")
        .build();
    let root_id = grammar.find_symbol_by_name("root").unwrap();
    assert_eq!(grammar.rule_names[&root_id], "root");
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 8: Start symbol has valid SymbolId
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_start_symbol_is_first_rule() {
    let grammar = GrammarBuilder::new("sl_v10_start1")
        .token("a", "a")
        .rule("other", vec!["a"])
        .rule("main", vec!["other"])
        .start("main")
        .build();
    let first_rule_id = *grammar.rules.keys().next().unwrap();
    assert_eq!(grammar.rule_names[&first_rule_id], "main");
}

#[test]
fn test_start_symbol_returns_some() {
    let grammar = GrammarBuilder::new("sl_v10_start2")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn test_start_symbol_matches_builder_start() {
    let grammar = GrammarBuilder::new("sl_v10_start3")
        .token("v", "v")
        .rule("entry", vec!["v"])
        .rule("other", vec!["v"])
        .start("entry")
        .build();
    let entry_id = grammar.find_symbol_by_name("entry").unwrap();
    let first_rule_id = *grammar.rules.keys().next().unwrap();
    assert_eq!(first_rule_id, entry_id);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 9: Different symbols have different IDs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_different_tokens_have_different_ids() {
    let grammar = GrammarBuilder::new("sl_v10_diff1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    assert_eq!(ids.len(), 3);
}

#[test]
fn test_token_and_nonterminal_different_ids() {
    let grammar = GrammarBuilder::new("sl_v10_diff2")
        .token("tok", "t")
        .rule("nt", vec!["tok"])
        .start("nt")
        .build();
    let tok_id = *grammar.tokens.keys().next().unwrap();
    let nt_id = grammar.find_symbol_by_name("nt").unwrap();
    assert_ne!(tok_id, nt_id);
}

#[test]
fn test_all_symbols_unique_in_complex_grammar() {
    let grammar = GrammarBuilder::new("sl_v10_diff3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("expr", vec!["a"])
        .rule("stmt", vec!["expr", "b"])
        .rule("prog", vec!["stmt", "c"])
        .start("prog")
        .build();
    let mut all_ids = HashSet::new();
    for id in grammar.tokens.keys() {
        assert!(all_ids.insert(*id), "Duplicate token ID found");
    }
    for id in grammar.rules.keys() {
        assert!(all_ids.insert(*id), "Duplicate rule ID found");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 10: ID assignment is deterministic
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_deterministic_id_assignment_simple() {
    let build = || {
        GrammarBuilder::new("sl_v10_det1")
            .token("x", "x")
            .rule("root", vec!["x"])
            .start("root")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ids1: Vec<SymbolId> = g1.tokens.keys().copied().collect();
    let ids2: Vec<SymbolId> = g2.tokens.keys().copied().collect();
    assert_eq!(ids1, ids2);
}

#[test]
fn test_deterministic_id_assignment_multiple_symbols() {
    let build = || {
        GrammarBuilder::new("sl_v10_det2")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("expr", vec!["a", "b"])
            .rule("stmt", vec!["expr", "c"])
            .start("stmt")
            .build()
    };
    let g1 = build();
    let g2 = build();
    for name in ["expr", "stmt"] {
        assert_eq!(g1.find_symbol_by_name(name), g2.find_symbol_by_name(name));
    }
}

#[test]
fn test_deterministic_rule_names() {
    let build = || {
        GrammarBuilder::new("sl_v10_det3")
            .token("t", "t")
            .rule("alpha", vec!["t"])
            .rule("beta", vec!["alpha"])
            .start("alpha")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let names1: Vec<_> = g1.rule_names.values().collect();
    let names2: Vec<_> = g2.rule_names.values().collect();
    assert_eq!(names1, names2);
}

#[test]
fn test_deterministic_token_ids() {
    let build = || {
        GrammarBuilder::new("sl_v10_det4")
            .token("num", r"\d+")
            .token("str", r#""[^"]*""#)
            .rule("lit", vec!["num"])
            .start("lit")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let tok_ids1: Vec<SymbolId> = g1.tokens.keys().copied().collect();
    let tok_ids2: Vec<SymbolId> = g2.tokens.keys().copied().collect();
    assert_eq!(tok_ids1, tok_ids2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 11: Grammar with 1 symbol
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_single_token() {
    let grammar = GrammarBuilder::new("sl_v10_single1")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    assert_eq!(grammar.tokens.len(), 1);
    assert_eq!(grammar.rules.len(), 1);
}

#[test]
fn test_grammar_single_nonterminal_has_id() {
    let grammar = GrammarBuilder::new("sl_v10_single2")
        .token("x", "x")
        .rule("item", vec!["x"])
        .start("item")
        .build();
    assert!(grammar.find_symbol_by_name("item").is_some());
}

#[test]
fn test_grammar_single_rule_symbol_in_rhs() {
    let grammar = GrammarBuilder::new("sl_v10_single3")
        .token("v", "v")
        .rule("top", vec!["v"])
        .start("top")
        .build();
    let top_id = grammar.find_symbol_by_name("top").unwrap();
    let rules = grammar.get_rules_for_symbol(top_id).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 12: Grammar with 10 symbols
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_ten_tokens() {
    let mut builder = GrammarBuilder::new("sl_v10_ten1");
    for i in 0..10 {
        builder = builder.token(&format!("t{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["t0"]).start("root");
    let grammar = builder.build();
    assert_eq!(grammar.tokens.len(), 10);
}

#[test]
fn test_grammar_ten_tokens_all_unique_ids() {
    let mut builder = GrammarBuilder::new("sl_v10_ten2");
    for i in 0..10 {
        builder = builder.token(&format!("t{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["t0"]).start("root");
    let grammar = builder.build();
    let ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    assert_eq!(ids.len(), 10);
}

#[test]
fn test_grammar_ten_mixed_symbols() {
    let mut builder = GrammarBuilder::new("sl_v10_ten3");
    for i in 0..5 {
        builder = builder.token(&format!("t{i}"), &format!("t{i}"));
    }
    for i in 0..5 {
        builder = builder.rule(&format!("r{i}"), vec!["t0"]);
    }
    builder = builder.start("r0");
    let grammar = builder.build();
    assert_eq!(grammar.tokens.len(), 5);
    assert_eq!(grammar.rules.len(), 5);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 13: Grammar with 50 symbols
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_fifty_tokens() {
    let mut builder = GrammarBuilder::new("sl_v10_fifty1");
    for i in 0..50 {
        builder = builder.token(&format!("tok{i}"), &format!("tok{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    let grammar = builder.build();
    assert_eq!(grammar.tokens.len(), 50);
}

#[test]
fn test_grammar_fifty_tokens_all_unique() {
    let mut builder = GrammarBuilder::new("sl_v10_fifty2");
    for i in 0..50 {
        builder = builder.token(&format!("tok{i}"), &format!("tok{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    let grammar = builder.build();
    let ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    assert_eq!(ids.len(), 50);
}

#[test]
fn test_grammar_fifty_mixed_all_have_ids() {
    let mut builder = GrammarBuilder::new("sl_v10_fifty3");
    for i in 0..25 {
        builder = builder.token(&format!("tok{i}"), &format!("tok{i}"));
    }
    for i in 0..25 {
        builder = builder.rule(&format!("nt{i}"), vec!["tok0"]);
    }
    builder = builder.start("nt0");
    let grammar = builder.build();
    let mut all_ids = HashSet::new();
    for id in grammar.tokens.keys() {
        all_ids.insert(*id);
    }
    for id in grammar.rules.keys() {
        all_ids.insert(*id);
    }
    assert_eq!(all_ids.len(), 50);
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 14: Same grammar built twice → same ID assignment
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_same_grammar_twice_same_token_ids() {
    let build = || {
        GrammarBuilder::new("sl_v10_twice1")
            .token("a", "a")
            .token("b", "b")
            .rule("root", vec!["a", "b"])
            .start("root")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let ids1: Vec<_> = g1.tokens.keys().copied().collect();
    let ids2: Vec<_> = g2.tokens.keys().copied().collect();
    assert_eq!(ids1, ids2);
}

#[test]
fn test_same_grammar_twice_same_rule_ids() {
    let build = || {
        GrammarBuilder::new("sl_v10_twice2")
            .token("x", "x")
            .rule("alpha", vec!["x"])
            .rule("beta", vec!["alpha"])
            .start("alpha")
            .build()
    };
    let g1 = build();
    let g2 = build();
    let rule_ids1: Vec<_> = g1.rules.keys().copied().collect();
    let rule_ids2: Vec<_> = g2.rules.keys().copied().collect();
    assert_eq!(rule_ids1, rule_ids2);
}

#[test]
fn test_same_grammar_twice_same_rule_names_map() {
    let build = || {
        GrammarBuilder::new("sl_v10_twice3")
            .token("v", "v")
            .rule("foo", vec!["v"])
            .rule("bar", vec!["foo"])
            .start("foo")
            .build()
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(g1.rule_names.len(), g2.rule_names.len());
    for (id, name) in &g1.rule_names {
        assert_eq!(g2.rule_names.get(id), Some(name));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 15: SymbolId max value (u16::MAX)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_max_value_creation() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, 65535);
}

#[test]
fn test_symbol_id_max_value_in_collection() {
    let mut set = HashSet::new();
    set.insert(SymbolId(u16::MAX));
    assert!(set.contains(&SymbolId(u16::MAX)));
}

#[test]
fn test_symbol_id_max_ordering() {
    assert!(SymbolId(u16::MAX) > SymbolId(u16::MAX - 1));
    assert!(SymbolId(u16::MAX) >= SymbolId(u16::MAX));
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 16: SymbolId zero
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_zero_value() {
    let id = SymbolId(0);
    assert_eq!(id.0, 0);
}

#[test]
fn test_symbol_id_zero_is_valid() {
    let id = SymbolId(0);
    let set: HashSet<SymbolId> = [id].into_iter().collect();
    assert!(set.contains(&SymbolId(0)));
}

#[test]
fn test_symbol_id_zero_equality() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn test_symbol_id_zero_reserved_for_eof_in_builder() {
    // GrammarBuilder starts next_symbol_id at 1, reserving 0 for EOF
    let grammar = GrammarBuilder::new("sl_v10_zero1")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let first_token_id = *grammar.tokens.keys().next().unwrap();
    assert!(first_token_id.0 >= 1, "Symbol 0 is reserved for EOF");
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 17: SymbolId arithmetic / comparison
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_id_sequential_values_comparison() {
    assert!(SymbolId(1) < SymbolId(2));
    assert!(SymbolId(2) < SymbolId(3));
}

#[test]
fn test_symbol_id_inner_value_arithmetic() {
    let a = SymbolId(10);
    let b = SymbolId(20);
    assert_eq!(b.0 - a.0, 10);
}

#[test]
fn test_symbol_id_sorting() {
    let mut ids = vec![
        SymbolId(5),
        SymbolId(1),
        SymbolId(3),
        SymbolId(2),
        SymbolId(4),
    ];
    ids.sort();
    let expected: Vec<SymbolId> = (1..=5).map(SymbolId).collect();
    assert_eq!(ids, expected);
}

#[test]
fn test_symbol_id_min_max() {
    let ids = [SymbolId(10), SymbolId(3), SymbolId(7)];
    assert_eq!(*ids.iter().min().unwrap(), SymbolId(3));
    assert_eq!(*ids.iter().max().unwrap(), SymbolId(10));
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 18: Symbol IDs contiguous after build
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_builder_assigns_contiguous_ids() {
    let grammar = GrammarBuilder::new("sl_v10_contig1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let mut all_ids: Vec<u16> = grammar
        .tokens
        .keys()
        .chain(grammar.rules.keys())
        .map(|id| id.0)
        .collect();
    all_ids.sort();
    // IDs should be sequential (starting from 1, since 0 is EOF)
    for window in all_ids.windows(2) {
        assert_eq!(window[1] - window[0], 1, "IDs should be contiguous");
    }
}

#[test]
fn test_contiguous_ids_five_tokens() {
    let mut builder = GrammarBuilder::new("sl_v10_contig2");
    for i in 0..5 {
        builder = builder.token(&format!("t{i}"), &format!("{i}"));
    }
    builder = builder.rule("root", vec!["t0"]).start("root");
    let grammar = builder.build();
    let mut ids: Vec<u16> = grammar
        .tokens
        .keys()
        .chain(grammar.rules.keys())
        .map(|id| id.0)
        .collect();
    ids.sort();
    for window in ids.windows(2) {
        assert_eq!(window[1] - window[0], 1);
    }
}

#[test]
fn test_contiguous_ids_mixed_symbols() {
    let grammar = GrammarBuilder::new("sl_v10_contig3")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("stmt", vec!["expr", "b"])
        .start("expr")
        .build();
    let mut ids: Vec<u16> = grammar
        .tokens
        .keys()
        .chain(grammar.rules.keys())
        .map(|id| id.0)
        .collect();
    ids.sort();
    ids.dedup();
    for window in ids.windows(2) {
        assert_eq!(window[1] - window[0], 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 19: Token IDs distinct from non-terminal IDs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_token_ids_disjoint_from_nonterminal_ids() {
    let grammar = GrammarBuilder::new("sl_v10_disjoint1")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("stmt", vec!["b"])
        .start("expr")
        .build();
    let token_ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    let rule_ids: HashSet<SymbolId> = grammar.rules.keys().copied().collect();
    let intersection: Vec<_> = token_ids.intersection(&rule_ids).collect();
    assert!(
        intersection.is_empty(),
        "Token and rule IDs must not overlap"
    );
}

#[test]
fn test_many_tokens_disjoint_from_nonterminals() {
    let mut builder = GrammarBuilder::new("sl_v10_disjoint2");
    for i in 0..10 {
        builder = builder.token(&format!("t{i}"), &format!("{i}"));
    }
    for i in 0..10 {
        builder = builder.rule(&format!("r{i}"), vec!["t0"]);
    }
    builder = builder.start("r0");
    let grammar = builder.build();
    let token_ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    let rule_ids: HashSet<SymbolId> = grammar.rules.keys().copied().collect();
    assert!(token_ids.is_disjoint(&rule_ids));
}

#[test]
fn test_token_and_rule_id_namespaces_separate() {
    let grammar = GrammarBuilder::new("sl_v10_disjoint3")
        .token("lit", "l")
        .rule("top", vec!["lit"])
        .start("top")
        .build();
    let tok_id = *grammar.tokens.keys().next().unwrap();
    let rule_id = *grammar.rules.keys().next().unwrap();
    assert_ne!(
        tok_id, rule_id,
        "Token and non-terminal should have distinct IDs"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Group 20: Clone preserves all symbol IDs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_grammar_clone_preserves_token_ids() {
    let grammar = GrammarBuilder::new("sl_v10_clone1")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a", "b"])
        .start("root")
        .build();
    let cloned = grammar.clone();
    let orig_ids: Vec<SymbolId> = grammar.tokens.keys().copied().collect();
    let clone_ids: Vec<SymbolId> = cloned.tokens.keys().copied().collect();
    assert_eq!(orig_ids, clone_ids);
}

#[test]
fn test_grammar_clone_preserves_rule_ids() {
    let grammar = GrammarBuilder::new("sl_v10_clone2")
        .token("x", "x")
        .rule("foo", vec!["x"])
        .rule("bar", vec!["foo"])
        .start("foo")
        .build();
    let cloned = grammar.clone();
    let orig_ids: Vec<SymbolId> = grammar.rules.keys().copied().collect();
    let clone_ids: Vec<SymbolId> = cloned.rules.keys().copied().collect();
    assert_eq!(orig_ids, clone_ids);
}

#[test]
fn test_grammar_clone_preserves_rule_names() {
    let grammar = GrammarBuilder::new("sl_v10_clone3")
        .token("v", "v")
        .rule("alpha", vec!["v"])
        .rule("beta", vec!["alpha"])
        .start("alpha")
        .build();
    let cloned = grammar.clone();
    assert_eq!(grammar.rule_names, cloned.rule_names);
}

#[test]
fn test_grammar_clone_preserves_find_symbol_by_name() {
    let grammar = GrammarBuilder::new("sl_v10_clone4")
        .token("q", "q")
        .rule("target", vec!["q"])
        .start("target")
        .build();
    let cloned = grammar.clone();
    assert_eq!(
        grammar.find_symbol_by_name("target"),
        cloned.find_symbol_by_name("target")
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional tests to reach 80+ total
// ═══════════════════════════════════════════════════════════════════════════

// --- RuleId basic tests ---

#[test]
fn test_rule_id_creation() {
    let id = RuleId(0);
    assert_eq!(id.0, 0);
}

#[test]
fn test_rule_id_equality() {
    assert_eq!(RuleId(3), RuleId(3));
    assert_ne!(RuleId(1), RuleId(2));
}

#[test]
fn test_rule_id_ordering() {
    assert!(RuleId(1) < RuleId(2));
    let mut ids = vec![RuleId(5), RuleId(1), RuleId(3)];
    ids.sort();
    assert_eq!(ids, vec![RuleId(1), RuleId(3), RuleId(5)]);
}

#[test]
fn test_rule_id_copy_semantics() {
    let a = RuleId(10);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn test_rule_id_debug_format() {
    let debug = format!("{:?}", RuleId(7));
    assert!(debug.contains("7"));
}

#[test]
fn test_rule_id_display_format() {
    assert_eq!(format!("{}", RuleId(42)), "Rule(42)");
}

#[test]
fn test_rule_id_in_hashmap() {
    let mut map = HashMap::new();
    map.insert(RuleId(1), "first");
    map.insert(RuleId(2), "second");
    assert_eq!(map[&RuleId(1)], "first");
}

// --- Symbol enum variant tests ---

#[test]
fn test_symbol_terminal_variant() {
    let sym = Symbol::Terminal(SymbolId(5));
    match sym {
        Symbol::Terminal(id) => assert_eq!(id, SymbolId(5)),
        _ => panic!("Expected Terminal variant"),
    }
}

#[test]
fn test_symbol_nonterminal_variant() {
    let sym = Symbol::NonTerminal(SymbolId(10));
    match sym {
        Symbol::NonTerminal(id) => assert_eq!(id, SymbolId(10)),
        _ => panic!("Expected NonTerminal variant"),
    }
}

#[test]
fn test_symbol_external_variant() {
    let sym = Symbol::External(SymbolId(20));
    match sym {
        Symbol::External(id) => assert_eq!(id, SymbolId(20)),
        _ => panic!("Expected External variant"),
    }
}

#[test]
fn test_symbol_epsilon_variant() {
    let sym = Symbol::Epsilon;
    assert!(matches!(sym, Symbol::Epsilon));
}

#[test]
fn test_symbol_equality() {
    assert_eq!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1)));
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

// --- Registry integration via Grammar ---

#[test]
fn test_grammar_build_registry() {
    let mut grammar = GrammarBuilder::new("sl_v10_reg1")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let registry = grammar.get_or_build_registry();
    assert!(!registry.is_empty());
}

#[test]
fn test_grammar_registry_contains_tokens() {
    let mut grammar = GrammarBuilder::new("sl_v10_reg2")
        .token("lit", "x")
        .rule("root", vec!["lit"])
        .start("root")
        .build();
    let registry = grammar.get_or_build_registry();
    assert!(registry.get_id("lit").is_some());
}

#[test]
fn test_grammar_registry_contains_nonterminals() {
    let mut grammar = GrammarBuilder::new("sl_v10_reg3")
        .token("a", "a")
        .rule("top", vec!["a"])
        .start("top")
        .build();
    let registry = grammar.get_or_build_registry();
    assert!(registry.get_id("top").is_some());
}

#[test]
fn test_grammar_registry_deterministic() {
    let build = || {
        let mut g = GrammarBuilder::new("sl_v10_reg4")
            .token("a", "a")
            .token("b", "b")
            .rule("root", vec!["a", "b"])
            .start("root")
            .build();
        g.get_or_build_registry().clone()
    };
    let reg1 = build();
    let reg2 = build();
    assert_eq!(reg1, reg2);
}

#[test]
fn test_grammar_registry_eof_is_zero() {
    let mut grammar = GrammarBuilder::new("sl_v10_reg5")
        .token("z", "z")
        .rule("root", vec!["z"])
        .start("root")
        .build();
    let registry = grammar.get_or_build_registry();
    assert_eq!(registry.get_id("end"), Some(SymbolId(0)));
}

// --- Grammar name and structure checks ---

#[test]
fn test_grammar_name_preserved() {
    let grammar = GrammarBuilder::new("sl_v10_name1").build();
    assert_eq!(grammar.name, "sl_v10_name1");
}

#[test]
fn test_empty_grammar_no_start_symbol() {
    let grammar = GrammarBuilder::new("sl_v10_empty1").build();
    assert!(grammar.start_symbol().is_none());
}

#[test]
fn test_grammar_with_empty_production() {
    let grammar = GrammarBuilder::new("sl_v10_eps1")
        .token("a", "a")
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .start("opt")
        .build();
    let opt_id = grammar.find_symbol_by_name("opt").unwrap();
    let rules = grammar.get_rules_for_symbol(opt_id).unwrap();
    assert_eq!(rules.len(), 2);
    assert!(rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

// --- Edge cases for ID management ---

#[test]
fn test_reusing_same_token_name_returns_same_id() {
    // When a token name is used in a rule's RHS, it should reference the same ID
    let grammar = GrammarBuilder::new("sl_v10_reuse1")
        .token("a", "a")
        .rule("first", vec!["a"])
        .rule("second", vec!["a"])
        .start("first")
        .build();
    let first_id = grammar.find_symbol_by_name("first").unwrap();
    let second_id = grammar.find_symbol_by_name("second").unwrap();
    let first_rules = grammar.get_rules_for_symbol(first_id).unwrap();
    let second_rules = grammar.get_rules_for_symbol(second_id).unwrap();
    // Both rules reference the same token "a" — same SymbolId in RHS
    assert_eq!(first_rules[0].rhs, second_rules[0].rhs);
}

#[test]
fn test_symbol_id_used_as_btree_key() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert(SymbolId(3), "c");
    map.insert(SymbolId(1), "a");
    map.insert(SymbolId(2), "b");
    let keys: Vec<SymbolId> = map.keys().copied().collect();
    assert_eq!(keys, vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
}

#[test]
fn test_symbol_id_vec_dedup() {
    let mut ids = vec![
        SymbolId(1),
        SymbolId(1),
        SymbolId(2),
        SymbolId(2),
        SymbolId(3),
    ];
    ids.dedup();
    assert_eq!(ids, vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
}

#[test]
fn test_grammar_all_rules_iterator() {
    let grammar = GrammarBuilder::new("sl_v10_iter1")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("x", vec!["b"])
        .rule("y", vec!["x"])
        .start("x")
        .build();
    let count = grammar.all_rules().count();
    assert_eq!(count, 3);
}

#[test]
fn test_grammar_find_nonexistent_symbol() {
    let grammar = GrammarBuilder::new("sl_v10_miss1")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    assert!(grammar.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn test_grammar_rules_for_nonexistent_symbol() {
    let grammar = GrammarBuilder::new("sl_v10_miss2")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    assert!(grammar.get_rules_for_symbol(SymbolId(999)).is_none());
}

#[test]
fn test_symbol_clone_equality() {
    let sym = Symbol::Terminal(SymbolId(42));
    let cloned = sym.clone();
    assert_eq!(sym, cloned);
}

#[test]
fn test_symbol_nonterminal_clone_equality() {
    let sym = Symbol::NonTerminal(SymbolId(7));
    let cloned = sym.clone();
    assert_eq!(sym, cloned);
}

#[test]
fn test_grammar_external_tokens_get_ids() {
    let grammar = GrammarBuilder::new("sl_v10_ext1")
        .token("a", "a")
        .external("scanner")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    assert_eq!(grammar.externals.len(), 1);
    let ext_id = grammar.externals[0].symbol_id;
    // External should have a non-zero ID
    assert!(ext_id.0 > 0);
}

#[test]
fn test_grammar_external_ids_distinct_from_tokens() {
    let grammar = GrammarBuilder::new("sl_v10_ext2")
        .token("a", "a")
        .token("b", "b")
        .external("ext1")
        .external("ext2")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let token_ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
    let ext_ids: HashSet<SymbolId> = grammar.externals.iter().map(|e| e.symbol_id).collect();
    assert!(token_ids.is_disjoint(&ext_ids));
}

#[test]
fn test_grammar_extras_have_valid_ids() {
    let grammar = GrammarBuilder::new("sl_v10_extra1")
        .token("ws", r"\s+")
        .token("a", "a")
        .extra("ws")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    assert_eq!(grammar.extras.len(), 1);
    let ws_id = grammar.extras[0];
    assert!(grammar.tokens.contains_key(&ws_id));
}
