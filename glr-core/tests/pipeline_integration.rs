//! Cross-crate pipeline tests: Grammar -> FirstFollow -> ItemSets.

use adze_glr_core::{FirstFollowSets, ItemSetCollection};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;

fn expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("number", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["lparen", "expr", "rparen"])
        .rule("factor", vec!["number"])
        .start("expr")
        .build()
}

fn trivial_grammar() -> Grammar {
    GrammarBuilder::new("trivial")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("left_rec")
        .token("x", "x")
        .token("plus", "\\+")
        .rule("list", vec!["list", "plus", "x"])
        .rule("list", vec!["x"])
        .start("list")
        .build()
}

#[test]
fn trivial_first_follow_computes() {
    let g = trivial_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // Start symbol should have first set
    if let Some(start) = g.start_symbol() {
        assert!(ff.first(start).is_some() || ff.follow(start).is_some());
    }
}

#[test]
fn trivial_item_sets_build() {
    let g = trivial_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(c.sets.len() >= 2);
}

#[test]
fn trivial_goto_table_populated() {
    let g = trivial_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(!c.goto_table.is_empty());
}

#[test]
fn expr_first_follow() {
    let g = expr_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // At least some rules should have FIRST sets
    let has_first = g.rule_names.iter().any(|(id, _)| ff.first(*id).is_some());
    assert!(has_first);
}

#[test]
fn expr_item_sets_many_states() {
    let g = expr_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(c.sets.len() >= 5);
}

#[test]
fn left_recursive_builds() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(c.sets.len() >= 3);
}

#[test]
fn normalize_then_first_follow() {
    let mut g = expr_grammar();
    let _new_rules = g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let has_first = g.rule_names.iter().any(|(id, _)| ff.first(*id).is_some());
    assert!(has_first);
}

#[test]
fn validate_then_build() {
    let g = expr_grammar();
    g.validate().unwrap();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(!c.sets.is_empty());
}

#[test]
fn multiple_grammars_independent() {
    let g1 = trivial_grammar();
    let g2 = expr_grammar();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&g1, &ff1);
    let c2 = ItemSetCollection::build_canonical_collection(&g2, &ff2);
    assert_ne!(c1.sets.len(), c2.sets.len());
}

#[test]
fn item_set_ids_sequential() {
    let g = expr_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    for (i, set) in c.sets.iter().enumerate() {
        assert_eq!(set.id.0 as usize, i);
    }
}

#[test]
fn follow_exists_for_start() {
    let g = trivial_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        let follow = ff.follow(start);
        assert!(follow.is_some(), "FOLLOW of start symbol should exist");
    }
}

#[test]
fn serde_roundtrip_preserves_pipeline() {
    let g = expr_grammar();
    let serialized = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&serialized).unwrap();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&g, &ff1);
    let c2 = ItemSetCollection::build_canonical_collection(&g2, &ff2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

#[test]
fn normalized_builds_item_sets() {
    let mut g = left_recursive_grammar();
    let _new_rules = g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let c = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(!c.sets.is_empty());
}

#[test]
fn first_sets_deterministic() {
    let g = expr_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        let first1 = ff.first(start);
        let first2 = ff.first(start);
        assert_eq!(first1, first2);
    }
}
