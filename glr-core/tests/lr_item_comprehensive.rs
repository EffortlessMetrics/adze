// Comprehensive tests for LRItem, ItemSet, ItemSetCollection, Action, GLRError, etc.
use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};

// ---------------------------------------------------------------------------
// LRItem tests
// ---------------------------------------------------------------------------

#[test]
fn lr_item_new_basic() {
    let item = LRItem::new(RuleId(0), 0, SymbolId(1));
    assert_eq!(item.rule_id, RuleId(0));
    assert_eq!(item.position, 0);
    assert_eq!(item.lookahead, SymbolId(1));
}

#[test]
fn lr_item_equality() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 0, SymbolId(1));
    assert_eq!(a, b);
}

#[test]
fn lr_item_inequality_rule() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(1), 0, SymbolId(1));
    assert_ne!(a, b);
}

#[test]
fn lr_item_inequality_position() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 1, SymbolId(1));
    assert_ne!(a, b);
}

#[test]
fn lr_item_inequality_lookahead() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 0, SymbolId(2));
    assert_ne!(a, b);
}

#[test]
fn lr_item_clone() {
    let item = LRItem::new(RuleId(5), 3, SymbolId(7));
    let cloned = item.clone();
    assert_eq!(item, cloned);
}

#[test]
fn lr_item_debug() {
    let item = LRItem::new(RuleId(0), 0, SymbolId(1));
    let debug = format!("{:?}", item);
    assert!(debug.contains("LRItem"));
}

#[test]
fn lr_item_in_btreeset() {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    set.insert(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.insert(LRItem::new(RuleId(0), 0, SymbolId(1)));
    set.insert(LRItem::new(RuleId(0), 0, SymbolId(0)));
    assert_eq!(set.len(), 2);
}

#[test]
fn lr_item_is_reduce_item_empty_rule() {
    let g = GrammarBuilder::new("test")
        .token("tok", "x")
        .rule("start", vec![])
        .start("start")
        .build();
    let start_rule_id = RuleId(0);
    let item = LRItem::new(start_rule_id, 0, SymbolId(0));
    assert!(item.is_reduce_item(&g));
}

#[test]
fn lr_item_next_symbol_at_end() {
    let g = GrammarBuilder::new("test")
        .token("tok", "x")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let start_rule_id = RuleId(0);
    let item = LRItem::new(start_rule_id, 1, SymbolId(0));
    assert!(item.next_symbol(&g).is_none());
}

#[test]
fn lr_item_next_symbol_at_start() {
    let g = GrammarBuilder::new("test")
        .token("tok", "x")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let start_rule_id = RuleId(0);
    let item = LRItem::new(start_rule_id, 0, SymbolId(0));
    assert!(item.next_symbol(&g).is_some());
}

// ---------------------------------------------------------------------------
// ItemSet tests
// ---------------------------------------------------------------------------

#[test]
fn item_set_new_empty() {
    let iset = ItemSet::new(StateId(0));
    assert_eq!(iset.id, StateId(0));
    assert!(iset.items.is_empty());
}

#[test]
fn item_set_add_item() {
    let mut iset = ItemSet::new(StateId(0));
    iset.add_item(LRItem::new(RuleId(0), 0, SymbolId(1)));
    assert_eq!(iset.items.len(), 1);
}

#[test]
fn item_set_add_duplicate() {
    let mut iset = ItemSet::new(StateId(0));
    iset.add_item(LRItem::new(RuleId(0), 0, SymbolId(1)));
    iset.add_item(LRItem::new(RuleId(0), 0, SymbolId(1)));
    assert_eq!(iset.items.len(), 1);
}

#[test]
fn item_set_add_multiple() {
    let mut iset = ItemSet::new(StateId(0));
    iset.add_item(LRItem::new(RuleId(0), 0, SymbolId(1)));
    iset.add_item(LRItem::new(RuleId(0), 1, SymbolId(1)));
    iset.add_item(LRItem::new(RuleId(1), 0, SymbolId(2)));
    assert_eq!(iset.items.len(), 3);
}

#[test]
fn item_set_state_id() {
    let iset = ItemSet::new(StateId(42));
    assert_eq!(iset.id, StateId(42));
}

#[test]
fn item_set_debug() {
    let iset = ItemSet::new(StateId(0));
    let debug = format!("{:?}", iset);
    assert!(debug.contains("ItemSet"));
}

// ---------------------------------------------------------------------------
// ItemSetCollection tests
// ---------------------------------------------------------------------------

#[test]
fn item_set_collection_can_hold_sets() {
    let set1 = ItemSet::new(StateId(0));
    let set2 = ItemSet::new(StateId(1));
    let collection = ItemSetCollection {
        sets: vec![set1, set2],
        goto_table: indexmap::IndexMap::new(),
        symbol_is_terminal: indexmap::IndexMap::new(),
    };
    assert_eq!(collection.sets.len(), 2);
}

#[test]
fn item_set_collection_empty() {
    let collection = ItemSetCollection {
        sets: vec![],
        goto_table: indexmap::IndexMap::new(),
        symbol_is_terminal: indexmap::IndexMap::new(),
    };
    assert!(collection.sets.is_empty());
}

// ---------------------------------------------------------------------------
// Action enum tests
// ---------------------------------------------------------------------------

#[test]
fn action_shift() {
    let action = Action::Shift(StateId(5));
    assert!(matches!(action, Action::Shift(StateId(5))));
}

#[test]
fn action_reduce() {
    let action = Action::Reduce(RuleId(3));
    assert!(matches!(action, Action::Reduce(RuleId(3))));
}

#[test]
fn action_accept() {
    let action = Action::Accept;
    assert!(matches!(action, Action::Accept));
}

#[test]
fn action_error() {
    let action = Action::Error;
    assert!(matches!(action, Action::Error));
}

#[test]
fn action_recover() {
    let action = Action::Recover;
    assert!(matches!(action, Action::Recover));
}

#[test]
fn action_fork() {
    let actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))];
    let fork = Action::Fork(actions.clone());
    if let Action::Fork(inner) = fork {
        assert_eq!(inner.len(), 2);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn action_clone() {
    let a = Action::Shift(StateId(10));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn action_equality() {
    assert_eq!(Action::Accept, Action::Accept);
    assert_eq!(Action::Error, Action::Error);
    assert_eq!(Action::Shift(StateId(0)), Action::Shift(StateId(0)));
    assert_eq!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn action_inequality() {
    assert_ne!(Action::Accept, Action::Error);
    assert_ne!(Action::Shift(StateId(0)), Action::Shift(StateId(1)));
    assert_ne!(Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1)));
    assert_ne!(Action::Shift(StateId(0)), Action::Reduce(RuleId(0)));
}

#[test]
fn action_debug() {
    let a = Action::Shift(StateId(42));
    let debug = format!("{:?}", a);
    assert!(debug.contains("Shift"));
}

// ---------------------------------------------------------------------------
// GLRError tests
// ---------------------------------------------------------------------------

#[test]
fn glr_error_complex_symbols() {
    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "test".to_string(),
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("ComplexSymbolsNotNormalized"));
}

#[test]
fn glr_error_conflict_resolution() {
    let err = GLRError::ConflictResolution("test conflict".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("ConflictResolution"));
}

// ---------------------------------------------------------------------------
// ConflictType tests
// ---------------------------------------------------------------------------

#[test]
fn conflict_type_shift_reduce_debug() {
    let ct = ConflictType::ShiftReduce;
    let debug = format!("{:?}", ct);
    assert!(debug.contains("ShiftReduce"));
}

#[test]
fn conflict_type_reduce_reduce_debug() {
    let ct = ConflictType::ReduceReduce;
    let debug = format!("{:?}", ct);
    assert!(debug.contains("ReduceReduce"));
}

// ---------------------------------------------------------------------------
// ParseTable default and construction tests
// ---------------------------------------------------------------------------

#[test]
fn parse_table_default_empty() {
    let pt = ParseTable::default();
    assert!(pt.action_table.is_empty());
    assert!(pt.goto_table.is_empty());
    assert_eq!(pt.state_count, 0);
    assert_eq!(pt.symbol_count, 0);
}

#[test]
fn parse_table_default_eof_symbol() {
    let pt = ParseTable::default();
    assert_eq!(pt.eof_symbol, SymbolId(0));
}

#[test]
fn parse_table_default_start_symbol() {
    let pt = ParseTable::default();
    assert_eq!(pt.start_symbol, SymbolId(0));
}

#[test]
fn parse_table_default_initial_state() {
    let pt = ParseTable::default();
    assert_eq!(pt.initial_state, StateId(0));
}

// Counters require perf-counters feature — tested via feature-specific test files

// ---------------------------------------------------------------------------
// SymbolMetadata tests
// ---------------------------------------------------------------------------

#[test]
fn symbol_metadata_default_fields() {
    let md = SymbolMetadata {
        name: "test".to_string(),
        is_terminal: true,
        is_named: true,
        is_visible: true,
        is_extra: false,
        is_supertype: false,
        is_fragile: false,
        symbol_id: SymbolId(1),
    };
    assert_eq!(md.name, "test");
    assert!(md.is_terminal);
}

#[test]
fn symbol_metadata_error_symbol() {
    let md = SymbolMetadata {
        name: "ERROR".to_string(),
        is_terminal: false,
        is_named: false,
        is_visible: true,
        is_extra: false,
        is_supertype: false,
        is_fragile: false,
        symbol_id: SymbolId(0),
    };
    assert_eq!(md.name, "ERROR");
}

#[test]
fn symbol_metadata_extra_symbol() {
    let md = SymbolMetadata {
        name: "comment".to_string(),
        is_terminal: true,
        is_named: true,
        is_visible: false,
        is_extra: true,
        is_supertype: false,
        is_fragile: false,
        symbol_id: SymbolId(5),
    };
    assert!(md.is_extra);
    assert!(!md.is_visible);
}

// ---------------------------------------------------------------------------
// ParseRule tests
// ---------------------------------------------------------------------------

#[test]
fn parse_rule_basic() {
    let pr = ParseRule {
        lhs: SymbolId(1),
        rhs_len: 3,
    };
    assert_eq!(pr.lhs, SymbolId(1));
    assert_eq!(pr.rhs_len, 3);
}

#[test]
fn parse_rule_clone() {
    let pr = ParseRule {
        lhs: SymbolId(5),
        rhs_len: 2,
    };
    let cloned = pr.clone();
    assert_eq!(cloned.lhs, SymbolId(5));
    assert_eq!(cloned.rhs_len, 2);
}

#[test]
fn parse_rule_debug() {
    let pr = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 1,
    };
    let debug = format!("{:?}", pr);
    assert!(debug.contains("ParseRule"));
}

// ---------------------------------------------------------------------------
// FirstFollowSets computation tests
// ---------------------------------------------------------------------------

#[test]
fn first_follow_simple_grammar() {
    let mut g = GrammarBuilder::new("simple")
        .token("num", "[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
}

#[test]
fn first_follow_two_token_grammar() {
    let mut g = GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let start_sym = g.start_symbol().unwrap();
    let first = ff.first(start_sym);
    assert!(first.is_some());
}

#[test]
fn first_follow_nullable_detection() {
    let mut g = GrammarBuilder::new("nullable")
        .token("tok", "x")
        .rule("start", vec![])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let start_sym = g.start_symbol().unwrap();
    assert!(ff.is_nullable(start_sym));
}

#[test]
fn first_follow_non_nullable() {
    let mut g = GrammarBuilder::new("nonnull")
        .token("tok", "x")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let start_sym = g.start_symbol().unwrap();
    assert!(!ff.is_nullable(start_sym));
}

// ---------------------------------------------------------------------------
// build_lr1_automaton tests
// ---------------------------------------------------------------------------

#[test]
fn build_lr1_simple_grammar() {
    let mut g = GrammarBuilder::new("simple")
        .token("num", "[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let result = build_lr1_automaton_res(&g, &ff);
    assert!(result.is_ok());
}

#[test]
fn build_lr1_two_rules() {
    let mut g = GrammarBuilder::new("two_rules")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let result = build_lr1_automaton_res(&g, &ff);
    assert!(result.is_ok());
}

#[test]
fn build_lr1_sequence() {
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(pt.action_table.len() >= 2);
}

#[test]
fn sanity_check_simple_table() {
    let mut g = GrammarBuilder::new("simple")
        .token("num", "[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton_res(&g, &ff).unwrap();
    let result = sanity_check_tables(&pt);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// LexMode tests
// ---------------------------------------------------------------------------

#[test]
fn lex_mode_debug() {
    let lm = LexMode {
        lex_state: 0,
        external_lex_state: 0,
    };
    let debug = format!("{:?}", lm);
    assert!(debug.contains("LexMode"));
}

#[test]
fn lex_mode_clone() {
    let lm = LexMode {
        lex_state: 42,
        external_lex_state: 0,
    };
    let cloned = lm;
    assert_eq!(cloned.lex_state, 42);
}
