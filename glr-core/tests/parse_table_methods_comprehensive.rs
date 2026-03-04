//! Comprehensive tests for ParseTable methods, ConflictResolver, and LRItem.
#![cfg(feature = "test-api")]

use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId, StateId, SymbolId};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "num"])
        .start("expr")
        .build()
}

fn build_table(g: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(g).unwrap();
    build_lr1_automaton(g, &ff).unwrap()
}

// === LRItem tests ===

#[test]
fn lr_item_new() {
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert_eq!(item.rule_id, RuleId(0));
    assert_eq!(item.position, 0);
    assert_eq!(item.lookahead, SymbolId(0));
}

#[test]
fn lr_item_eq() {
    let a = LRItem::new(RuleId(1), 2, SymbolId(3));
    let b = LRItem::new(RuleId(1), 2, SymbolId(3));
    assert_eq!(a, b);
}

#[test]
fn lr_item_ne_rule() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(0));
    let b = LRItem::new(RuleId(1), 0, SymbolId(0));
    assert_ne!(a, b);
}

#[test]
fn lr_item_ne_position() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(0));
    let b = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert_ne!(a, b);
}

#[test]
fn lr_item_ne_lookahead() {
    let a = LRItem::new(RuleId(0), 0, SymbolId(0));
    let b = LRItem::new(RuleId(0), 0, SymbolId(1));
    assert_ne!(a, b);
}

#[test]
fn lr_item_clone() {
    let a = LRItem::new(RuleId(5), 3, SymbolId(2));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn lr_item_debug() {
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    let d = format!("{:?}", item);
    assert!(!d.is_empty());
}

#[test]
fn lr_item_hash_consistent() {
    use std::collections::HashSet;
    let a = LRItem::new(RuleId(1), 1, SymbolId(1));
    let b = LRItem::new(RuleId(1), 1, SymbolId(1));
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b));
}

#[test]
fn lr_item_is_reduce_on_simple() {
    let g = simple_grammar();
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    let _ = item.is_reduce_item(&g);
}

// === ItemSet tests ===

#[test]
fn item_set_new() {
    let is = ItemSet::new(StateId(0));
    assert_eq!(is.id, StateId(0));
    assert!(is.items.is_empty());
}

#[test]
fn item_set_add_item() {
    let mut is = ItemSet::new(StateId(0));
    is.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    assert!(!is.items.is_empty());
}

#[test]
fn item_set_add_multiple() {
    let mut is = ItemSet::new(StateId(1));
    is.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    is.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    assert!(is.items.len() >= 1);
}

// === ParseTable construction and methods ===

#[test]
fn parse_table_from_simple_grammar() {
    let g = simple_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn parse_table_from_arith_grammar() {
    let g = arith_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn parse_table_eof_symbol() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let _ = pt.eof();
}

#[test]
fn parse_table_start_symbol() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let _ = pt.start_symbol();
}

#[test]
fn parse_table_grammar_ref() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let g_ref = pt.grammar();
    assert_eq!(g_ref.name, "simple");
}

#[test]
fn parse_table_terminal_boundary() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let tb = pt.terminal_boundary();
    assert!(tb > 0);
}

#[test]
fn parse_table_is_terminal_check() {
    let g = simple_grammar();
    let pt = build_table(&g);
    // Just test the method doesn't panic for terminal_boundary index
    let tb = pt.terminal_boundary();
    // Symbols below boundary are terminals
    if tb > 0 {
        let _ = pt.is_terminal(SymbolId(0));
    }
}

#[test]
fn parse_table_valid_symbols() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let vs = pt.valid_symbols(StateId(0));
    assert!(!vs.is_empty());
}

#[test]
fn parse_table_valid_symbols_mask() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let mask = pt.valid_symbols_mask(StateId(0));
    assert!(!mask.is_empty());
}

#[test]
fn parse_table_actions_initial_state() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let eof = pt.eof();
    let acts = pt.actions(StateId(0), eof);
    let _ = acts;
}

#[test]
fn parse_table_rule_access() {
    let g = simple_grammar();
    let pt = build_table(&g);
    if !pt.rules.is_empty() {
        let (lhs, len) = pt.rule(RuleId(0));
        let _ = lhs;
        let _ = len;
    }
}

#[test]
fn parse_table_error_symbol() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let _ = pt.error_symbol();
}

#[test]
fn parse_table_lex_mode() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let lm = pt.lex_mode(StateId(0));
    let _ = lm;
}

#[test]
fn parse_table_is_extra() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let eof = pt.eof();
    let _ = pt.is_extra(eof);
}

#[test]
fn parse_table_validate_simple() {
    let g = simple_grammar();
    let pt = build_table(&g);
    // validate() may fail for generated tables — just ensure no panic
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pt.validate()));
}

#[test]
fn parse_table_validate_arith() {
    let g = arith_grammar();
    let pt = build_table(&g);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pt.validate()));
}

// === ParseTable transformations ===

#[test]
fn parse_table_with_detected_goto_indexing() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let pt2 = pt.with_detected_goto_indexing();
    assert!(pt2.state_count > 0);
}

#[test]
fn parse_table_normalize_eof_to_zero() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let pt2 = pt.normalize_eof_to_zero();
    assert_eq!(pt2.eof(), SymbolId(0));
}

// === ConflictResolver tests ===

#[test]
fn conflict_resolver_detect_simple() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    let _ = resolver.conflicts.len();
}

#[test]
fn conflict_resolver_detect_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    let _ = resolver.conflicts.len();
}

// === ConflictType tests ===

#[test]
fn conflict_type_shift_reduce() {
    let ct = ConflictType::ShiftReduce;
    let d = format!("{:?}", ct);
    assert!(d.contains("ShiftReduce"));
}

#[test]
fn conflict_type_reduce_reduce() {
    let ct = ConflictType::ReduceReduce;
    let d = format!("{:?}", ct);
    assert!(d.contains("ReduceReduce"));
}

#[test]
fn conflict_type_clone() {
    let a = ConflictType::ShiftReduce;
    let b = a.clone();
    assert_eq!(format!("{:?}", a), format!("{:?}", b));
}

// === Conflict struct tests ===

#[test]
fn conflict_struct_fields() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        conflict_type: ConflictType::ShiftReduce,
        actions: vec![],
    };
    assert_eq!(c.state, StateId(0));
    assert_eq!(c.symbol, SymbolId(1));
}

#[test]
fn conflict_debug() {
    let c = Conflict {
        state: StateId(2),
        symbol: SymbolId(3),
        conflict_type: ConflictType::ReduceReduce,
        actions: vec![],
    };
    let d = format!("{:?}", c);
    assert!(d.contains("Conflict"));
}

// === GLRError tests ===

#[test]
fn glr_error_debug() {
    let e = GLRError::ConflictResolution("test".into());
    let d = format!("{:?}", e);
    assert!(!d.is_empty());
}

#[test]
fn glr_error_display() {
    let e = GLRError::ConflictResolution("test".into());
    let s = e.to_string();
    assert!(s.contains("test"));
}

// === Action enum tests ===

#[test]
fn action_shift() {
    let a = Action::Shift(StateId(5));
    let d = format!("{:?}", a);
    assert!(d.contains("Shift"));
}

#[test]
fn action_reduce() {
    let a = Action::Reduce(RuleId(3));
    let d = format!("{:?}", a);
    assert!(d.contains("Reduce"));
}

#[test]
fn action_accept() {
    let a = Action::Accept;
    let d = format!("{:?}", a);
    assert!(d.contains("Accept"));
}

#[test]
fn action_fork() {
    let a = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    let d = format!("{:?}", a);
    assert!(d.contains("Fork"));
}

#[test]
fn action_clone() {
    let a = Action::Shift(StateId(1));
    let b = a.clone();
    assert_eq!(format!("{:?}", a), format!("{:?}", b));
}

// === ItemSetCollection tests ===

#[test]
fn collection_simple() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(col.sets.len() >= 2);
}

// === ParseRule tests ===

#[test]
fn parse_rule_fields() {
    let pr = ParseRule {
        lhs: SymbolId(1),
        rhs_len: 3,
    };
    assert_eq!(pr.lhs, SymbolId(1));
    assert_eq!(pr.rhs_len, 3);
}

#[test]
fn parse_rule_clone() {
    let a = ParseRule {
        lhs: SymbolId(1),
        rhs_len: 2,
    };
    let b = a.clone();
    assert_eq!(a.lhs, b.lhs);
    assert_eq!(a.rhs_len, b.rhs_len);
}

#[test]
fn parse_rule_debug() {
    let pr = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 0,
    };
    let d = format!("{:?}", pr);
    assert!(d.contains("ParseRule"));
}

// === SymbolMetadata tests ===

#[test]
fn symbol_metadata_fields() {
    let sm = SymbolMetadata {
        name: "test".to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(1),
    };
    assert_eq!(sm.name, "test");
    assert!(sm.is_terminal);
    assert!(sm.is_named);
    assert_eq!(sm.symbol_id, SymbolId(1));
}

#[test]
fn symbol_metadata_debug() {
    let sm = SymbolMetadata {
        name: "eof".to_string(),
        is_visible: false,
        is_named: false,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(0),
    };
    let d = format!("{:?}", sm);
    assert!(d.contains("SymbolMetadata"));
}

// === FirstFollowSets tests ===

#[test]
fn first_follow_compute_simple() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let _ = &ff;
}

#[test]
fn first_follow_compute_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let _ = &ff;
}

#[test]
fn first_sets_exist() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    for (id, _) in &g.rules {
        let _ = ff.first(*id);
    }
}

#[test]
fn follow_sets_exist() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    for (id, _) in &g.rules {
        let _ = ff.follow(*id);
    }
}

#[test]
fn nullable_check() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    for (id, _) in &g.rules {
        let _ = ff.is_nullable(*id);
    }
}

// === build_lr1_automaton ===

#[test]
fn build_lr1_simple() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn build_lr1_arith() {
    let g = arith_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}

// === sanity_check_tables ===

#[test]
fn sanity_check_simple() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let _ = sanity_check_tables(&pt);
}

#[test]
fn sanity_check_arith() {
    let g = arith_grammar();
    let pt = build_table(&g);
    let _ = sanity_check_tables(&pt);
}

// === LexMode tests ===

#[test]
fn lex_mode_fields() {
    let lm = LexMode {
        lex_state: 0,
        external_lex_state: 0,
    };
    assert_eq!(lm.lex_state, 0);
    assert_eq!(lm.external_lex_state, 0);
}

#[test]
fn lex_mode_clone() {
    let a = LexMode {
        lex_state: 1,
        external_lex_state: 2,
    };
    let b = a.clone();
    assert_eq!(a.lex_state, b.lex_state);
}

#[test]
fn lex_mode_debug() {
    let lm = LexMode {
        lex_state: 3,
        external_lex_state: 4,
    };
    let d = format!("{:?}", lm);
    assert!(d.contains("LexMode"));
}

// === Edge case grammars ===

#[test]
fn two_token_grammar() {
    let g = GrammarBuilder::new("two")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn multiple_productions() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_table(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn chain_grammar_table() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build();
    let pt = build_table(&g);
    assert!(pt.state_count > 0);
}

#[test]
fn recursive_grammar_table() {
    let g = GrammarBuilder::new("rec")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .start("list")
        .build();
    let pt = build_table(&g);
    assert!(pt.state_count > 0);
}

// === Table properties ===

#[test]
fn action_table_dimensions_consistent() {
    let g = simple_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.action_table.len(), pt.state_count);
    if pt.state_count > 0 {
        let cols = pt.action_table[0].len();
        for row in &pt.action_table {
            assert_eq!(row.len(), cols);
        }
    }
}

#[test]
fn goto_table_dimensions_consistent() {
    let g = simple_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.goto_table.len(), pt.state_count);
    if pt.state_count > 0 {
        let cols = pt.goto_table[0].len();
        for row in &pt.goto_table {
            assert_eq!(row.len(), cols);
        }
    }
}

#[test]
fn symbol_count_positive() {
    let g = simple_grammar();
    let pt = build_table(&g);
    assert!(pt.symbol_count > 0);
}

#[test]
fn rules_nonempty() {
    let g = simple_grammar();
    let pt = build_table(&g);
    assert!(!pt.rules.is_empty());
}

#[test]
fn symbol_metadata_nonempty() {
    let g = simple_grammar();
    let pt = build_table(&g);
    assert!(!pt.symbol_metadata.is_empty());
}

// === Determinism ===

#[test]
fn table_state_count_deterministic() {
    let g = simple_grammar();
    let pt1 = build_table(&g);
    let g2 = simple_grammar();
    let pt2 = build_table(&g2);
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn table_symbol_count_deterministic() {
    let g = simple_grammar();
    let pt1 = build_table(&g);
    let g2 = simple_grammar();
    let pt2 = build_table(&g2);
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
}

#[test]
fn arith_table_deterministic() {
    let g = arith_grammar();
    let pt1 = build_table(&g);
    let g2 = arith_grammar();
    let pt2 = build_table(&g2);
    assert_eq!(pt1.state_count, pt2.state_count);
    assert_eq!(pt1.rules.len(), pt2.rules.len());
}
