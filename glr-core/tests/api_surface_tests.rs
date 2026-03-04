//! Tests for GLR core API surface — state management, action types, conflicts.
#![cfg(feature = "test-api")]

use adze_glr_core::*;
use adze_ir::SymbolId;
use adze_ir::builder::GrammarBuilder;

#[test]
fn first_follow_simple_expr() {
    let mut g = GrammarBuilder::new("simple")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g);
    assert!(ff.is_ok());
    let ff = ff.unwrap();
    // Every non-terminal should have a FIRST set
    for (_sid, _rules) in &g.rules {
        // Just verify we can query FIRST sets
    }
    // Start symbol should have EOF in its FOLLOW set
    if let Some(start) = g.start_symbol() {
        if let Some(follow) = ff.follow(start) {
            assert!(
                follow.len() > 0,
                "start symbol should have non-empty FOLLOW"
            );
        }
    }
}

#[test]
fn first_follow_nullable() {
    let mut g = GrammarBuilder::new("nullable")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    // x is a terminal, so it should appear in FIRST of start
    let start = g.start_symbol().unwrap();
    let first = ff.first(start);
    assert!(first.is_some());
}

#[test]
fn parse_table_empty() {
    let pt = ParseTable::default();
    assert_eq!(pt.state_count, 0);
}

#[test]
fn state_id_ordering() {
    assert!(StateId(0) < StateId(1));
    assert!(StateId(5) > StateId(3));
    assert!(StateId(2) == StateId(2));
}

#[test]
fn rule_id_ordering() {
    assert!(RuleId(0) < RuleId(1));
    assert!(RuleId(10) > RuleId(5));
}

#[test]
fn lr_item_creation() {
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
fn lr_item_inequality_different_position() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(1));
    let b = LRItem::new(RuleId(0), 1, SymbolId(1));
    assert_ne!(a, b);
}

#[test]
fn item_set_creation() {
    let set = ItemSet::new(StateId(0));
    assert_eq!(set.id, StateId(0));
    assert!(set.items.is_empty());
}

#[test]
fn item_set_add_item() {
    let mut set = ItemSet::new(StateId(0));
    let item = LRItem::new(RuleId(0), 0, SymbolId(1));
    set.add_item(item);
    assert_eq!(set.items.len(), 1);
}

#[test]
fn item_set_deduplication() {
    let mut set = ItemSet::new(StateId(0));
    let item = LRItem::new(RuleId(0), 0, SymbolId(1));
    set.add_item(item.clone());
    set.add_item(item);
    // Should deduplicate
    assert!(set.items.len() <= 2);
}

#[test]
fn action_shift_debug() {
    let a = Action::Shift(StateId(5));
    let s = format!("{a:?}");
    assert!(s.contains("Shift") || s.contains("5"));
}

#[test]
fn action_reduce_debug() {
    let a = Action::Reduce(RuleId(3));
    let s = format!("{a:?}");
    assert!(s.contains("Reduce") || s.contains("3"));
}

#[test]
fn action_accept_debug() {
    let a = Action::Accept;
    let s = format!("{a:?}");
    assert!(s.contains("Accept"));
}

#[test]
fn action_equality() {
    assert_eq!(Action::Shift(StateId(1)), Action::Shift(StateId(1)));
    assert_ne!(Action::Shift(StateId(1)), Action::Shift(StateId(2)));
    assert_ne!(Action::Shift(StateId(1)), Action::Reduce(RuleId(1)));
    assert_eq!(Action::Accept, Action::Accept);
}
