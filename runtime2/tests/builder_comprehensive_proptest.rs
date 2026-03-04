#![cfg(feature = "glr-core")]
#![allow(clippy::needless_range_loop)]

//! Comprehensive property-based and unit tests for the builder module
//! (forest-to-tree conversion).
//!
//! Covers: `builder.rs` forest_to_tree pipeline, TreeNode construction,
//! Tree/Node invariants, TreeCursor navigation, clone independence,
//! new_for_testing shapes, performance characteristics, and edge cases.

use proptest::prelude::*;
use std::time::Instant;

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::language::SymbolMetadata;
use adze_runtime::tree::TreeCursor;
use adze_runtime::{Language, Node, Parser, Point, Token, Tree};

// ---------------------------------------------------------------------------
// Grammar helpers
// ---------------------------------------------------------------------------

/// start → a  (Symbols: 0=EOF, 1=a, 2=start)
fn lang_single() -> Language {
    let mut g = Grammar::new("single".into());
    let a = SymbolId(1);
    g.tokens.insert(
        a,
        IrToken {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let start = SymbolId(2);
    g.rule_names.insert(start, "start".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tbl = build_lr1_automaton(&g, &ff)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let tbl: &'static _ = Box::leak(Box::new(tbl));
    Language::builder()
        .parse_table(tbl)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut t = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                if b == b'a' {
                    t.push(Token {
                        kind: 1,
                        start: i as u32,
                        end: (i + 1) as u32,
                    });
                }
            }
            t.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(t.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

/// start → a b  (Symbols: 0=EOF, 1=a, 2=b, 3=start)
fn lang_two() -> Language {
    let mut g = Grammar::new("two".into());
    for (id, name, pat) in [(1, "a", "a"), (2, "b", "b")] {
        g.tokens.insert(
            SymbolId(id),
            IrToken {
                name: name.into(),
                pattern: TokenPattern::String(pat.into()),
                fragile: false,
            },
        );
    }
    let start = SymbolId(3);
    g.rule_names.insert(start, "start".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tbl = build_lr1_automaton(&g, &ff)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let tbl: &'static _ = Box::leak(Box::new(tbl));
    Language::builder()
        .parse_table(tbl)
        .symbol_names(vec!["EOF".into(), "a".into(), "b".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut t = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                let k = match b {
                    b'a' => 1,
                    b'b' => 2,
                    _ => continue,
                };
                t.push(Token {
                    kind: k,
                    start: i as u32,
                    end: (i + 1) as u32,
                });
            }
            t.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(t.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

/// Chain grammar: start → mid, mid → a  (Symbols: 0=EOF, 1=a, 2=mid, 3=start)
fn lang_chain() -> Language {
    let mut g = Grammar::new("chain".into());
    let a = SymbolId(1);
    g.tokens.insert(
        a,
        IrToken {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let mid = SymbolId(2);
    g.rule_names.insert(mid, "mid".into());
    g.rules.insert(
        mid,
        vec![Rule {
            lhs: mid,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let start = SymbolId(3);
    g.rule_names.insert(start, "start".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::NonTerminal(mid)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tbl = build_lr1_automaton(&g, &ff)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let tbl: &'static _ = Box::leak(Box::new(tbl));
    Language::builder()
        .parse_table(tbl)
        .symbol_names(vec!["EOF".into(), "a".into(), "mid".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut t = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                if b == b'a' {
                    t.push(Token {
                        kind: 1,
                        start: i as u32,
                        end: (i + 1) as u32,
                    });
                }
            }
            t.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(t.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tree analysis helpers
// ---------------------------------------------------------------------------

fn count_nodes(n: Node<'_>) -> usize {
    let mut total = 1;
    for i in 0..n.child_count() {
        total += count_nodes(n.child(i).unwrap());
    }
    total
}

fn tree_depth(n: Node<'_>) -> usize {
    let mut d = 0;
    for i in 0..n.child_count() {
        d = d.max(1 + tree_depth(n.child(i).unwrap()));
    }
    d
}

fn collect_shape(n: Node<'_>, out: &mut Vec<(u16, usize, usize, usize)>) {
    out.push((n.kind_id(), n.start_byte(), n.end_byte(), n.child_count()));
    for i in 0..n.child_count() {
        collect_shape(n.child(i).unwrap(), out);
    }
}

fn assert_valid_ranges(n: Node<'_>) {
    assert!(
        n.start_byte() <= n.end_byte(),
        "start {} > end {}",
        n.start_byte(),
        n.end_byte()
    );
    for i in 0..n.child_count() {
        let c = n.child(i).unwrap();
        assert!(c.start_byte() >= n.start_byte());
        assert!(c.end_byte() <= n.end_byte());
        assert_valid_ranges(c);
    }
}

fn parse_with(lang: Language, input: &[u8]) -> Tree {
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    p.parse(input, None).unwrap()
}

/// Walk the tree via TreeCursor and count nodes visited.
fn cursor_count_nodes(tree: &Tree) -> usize {
    let mut cursor = TreeCursor::new(tree);
    let mut count = 1; // root
    if cursor.goto_first_child() {
        loop {
            count += cursor_count_subtree(&mut cursor);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
    count
}

fn cursor_count_subtree(cursor: &mut TreeCursor<'_>) -> usize {
    let mut count = 1;
    if cursor.goto_first_child() {
        loop {
            count += cursor_count_subtree(cursor);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
    count
}

// ===========================================================================
// 1 – new_for_testing: leaf node construction
// ===========================================================================

#[test]
fn for_testing_leaf_has_correct_symbol() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn for_testing_leaf_has_correct_byte_range() {
    let tree = Tree::new_for_testing(1, 5, 15, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 15);
}

#[test]
fn for_testing_leaf_has_zero_children() {
    let tree = Tree::new_for_testing(1, 0, 1, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn for_testing_leaf_kind_id_matches_symbol() {
    let tree = Tree::new_for_testing(99, 0, 1, vec![]);
    assert_eq!(tree.root_node().kind_id(), 99);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn for_testing_leaf_preserves_arbitrary_symbol_and_range(
        sym in 0u32..5000,
        start in 0usize..1000,
        len in 0usize..1000,
    ) {
        let end = start + len;
        let tree = Tree::new_for_testing(sym, start, end, vec![]);
        prop_assert_eq!(tree.root_kind(), sym);
        prop_assert_eq!(tree.root_node().start_byte(), start);
        prop_assert_eq!(tree.root_node().end_byte(), end);
    }
}

// ===========================================================================
// 2 – new_for_testing: parent-child construction
// ===========================================================================

#[test]
fn for_testing_one_child_is_accessible() {
    let child = Tree::new_for_testing(1, 0, 5, vec![]);
    let tree = Tree::new_for_testing(0, 0, 5, vec![child]);
    assert_eq!(tree.root_node().child_count(), 1);
    let c = tree.root_node().child(0).unwrap();
    assert_eq!(c.kind_id(), 1);
    assert_eq!(c.start_byte(), 0);
    assert_eq!(c.end_byte(), 5);
}

#[test]
fn for_testing_multiple_children_ordered() {
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let c3 = Tree::new_for_testing(3, 6, 9, vec![]);
    let tree = Tree::new_for_testing(0, 0, 9, vec![c1, c2, c3]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    for i in 0..3 {
        assert_eq!(root.child(i).unwrap().kind_id(), (i + 1) as u16);
    }
}

#[test]
fn for_testing_nested_grandchild() {
    let grandchild = Tree::new_for_testing(2, 0, 1, vec![]);
    let child = Tree::new_for_testing(1, 0, 1, vec![grandchild]);
    let tree = Tree::new_for_testing(0, 0, 1, vec![child]);
    let root = tree.root_node();
    let c = root.child(0).unwrap();
    assert_eq!(c.child_count(), 1);
    let gc = c.child(0).unwrap();
    assert_eq!(gc.kind_id(), 2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn for_testing_child_count_matches_input(n in 1usize..25) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i * 2, (i + 1) * 2, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n * 2, children);
        prop_assert_eq!(tree.root_node().child_count(), n);
    }

    #[test]
    fn for_testing_all_children_accessible_by_index(n in 1usize..20) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing((i + 1) as u32, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n, children);
        for i in 0..n {
            let child = tree.root_node().child(i);
            prop_assert!(child.is_some(), "child {} should exist", i);
            prop_assert_eq!(child.unwrap().kind_id(), (i + 1) as u16);
        }
    }
}

// ===========================================================================
// 3 – new_for_testing: deep chain shapes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn deep_chain_depth_equals_levels_minus_one(depth in 1usize..40) {
        let mut tree = Tree::new_for_testing(1, 0, 10, vec![]);
        for i in 1..depth {
            tree = Tree::new_for_testing((i + 1) as u32, 0, 10, vec![tree]);
        }
        prop_assert_eq!(tree_depth(tree.root_node()), depth - 1);
    }

    #[test]
    fn deep_chain_node_count_equals_depth(depth in 1usize..40) {
        let mut tree = Tree::new_for_testing(1, 0, 10, vec![]);
        for i in 1..depth {
            tree = Tree::new_for_testing((i + 1) as u32, 0, 10, vec![tree]);
        }
        prop_assert_eq!(count_nodes(tree.root_node()), depth);
    }
}

#[test]
fn deep_chain_50_levels_doesnt_panic() {
    let mut tree = Tree::new_for_testing(1, 0, 1, vec![]);
    for i in 1..50 {
        tree = Tree::new_for_testing((i + 1) as u32, 0, 1, vec![tree]);
    }
    assert_eq!(tree_depth(tree.root_node()), 49);
}

// ===========================================================================
// 4 – new_for_testing: wide tree shapes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn wide_tree_has_depth_one(n in 2usize..50) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n, children);
        prop_assert_eq!(tree_depth(tree.root_node()), 1);
    }

    #[test]
    fn wide_tree_node_count_is_n_plus_one(n in 1usize..50) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n, children);
        prop_assert_eq!(count_nodes(tree.root_node()), n + 1);
    }
}

// ===========================================================================
// 5 – Stub tree invariants
// ===========================================================================

#[test]
fn stub_tree_root_kind_zero() {
    assert_eq!(Tree::new_stub().root_kind(), 0);
}

#[test]
fn stub_tree_root_kind_id_zero() {
    assert_eq!(Tree::new_stub().root_node().kind_id(), 0);
}

#[test]
fn stub_tree_zero_byte_range() {
    let s = Tree::new_stub();
    assert_eq!(s.root_node().start_byte(), 0);
    assert_eq!(s.root_node().end_byte(), 0);
    assert_eq!(s.root_node().byte_range(), 0..0);
}

#[test]
fn stub_tree_no_children() {
    assert_eq!(Tree::new_stub().root_node().child_count(), 0);
}

#[test]
fn stub_tree_no_language() {
    assert!(Tree::new_stub().language().is_none());
}

#[test]
fn stub_tree_no_source() {
    assert!(Tree::new_stub().source_bytes().is_none());
}

#[test]
fn stub_tree_kind_is_unknown() {
    assert_eq!(Tree::new_stub().root_node().kind(), "unknown");
}

#[test]
fn stub_tree_child_out_of_bounds() {
    assert!(Tree::new_stub().root_node().child(0).is_none());
}

// ===========================================================================
// 6 – Clone independence
// ===========================================================================

#[test]
fn clone_stub_preserves_all_properties() {
    let orig = Tree::new_stub();
    let cloned = orig.clone();
    assert_eq!(orig.root_kind(), cloned.root_kind());
    assert_eq!(
        orig.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(orig.root_node().end_byte(), cloned.root_node().end_byte());
}

#[test]
fn clone_for_testing_tree_shape_identical() {
    let c1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let c2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2]);
    let cloned = tree.clone();
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    collect_shape(tree.root_node(), &mut s1);
    collect_shape(cloned.root_node(), &mut s2);
    assert_eq!(s1, s2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn clone_preserves_node_count(n in 1usize..20) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n, children);
        let cloned = tree.clone();
        prop_assert_eq!(
            count_nodes(tree.root_node()),
            count_nodes(cloned.root_node())
        );
    }

    #[test]
    fn clone_deep_chain_preserves_depth(depth in 2usize..25) {
        let mut tree = Tree::new_for_testing(1, 0, 5, vec![]);
        for i in 1..depth {
            tree = Tree::new_for_testing((i + 1) as u32, 0, 5, vec![tree]);
        }
        let cloned = tree.clone();
        prop_assert_eq!(
            tree_depth(tree.root_node()),
            tree_depth(cloned.root_node())
        );
    }
}

// ===========================================================================
// 7 – Forest-to-tree pipeline via Parser (single token grammar)
// ===========================================================================

#[test]
fn pipeline_single_produces_tree_with_valid_ranges() {
    let tree = parse_with(lang_single(), b"a");
    assert_valid_ranges(tree.root_node());
}

#[test]
fn pipeline_single_has_language() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.language().is_some());
}

#[test]
fn pipeline_single_stores_source() {
    let tree = parse_with(lang_single(), b"a");
    assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
}

#[test]
fn pipeline_single_root_has_at_least_one_child() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.root_node().child_count() > 0);
}

#[test]
fn pipeline_single_root_kind_is_not_unknown() {
    let tree = parse_with(lang_single(), b"a");
    assert_ne!(tree.root_node().kind(), "unknown");
}

// ===========================================================================
// 8 – Forest-to-tree pipeline via Parser (two token grammar)
// ===========================================================================

#[test]
fn pipeline_two_token_root_has_two_children() {
    let tree = parse_with(lang_two(), b"ab");
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn pipeline_two_token_children_ranges() {
    let tree = parse_with(lang_two(), b"ab");
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    let c1 = root.child(1).unwrap();
    assert_eq!(c0.start_byte(), 0);
    assert_eq!(c0.end_byte(), 1);
    assert_eq!(c1.start_byte(), 1);
    assert_eq!(c1.end_byte(), 2);
}

#[test]
fn pipeline_two_token_children_non_overlapping() {
    let tree = parse_with(lang_two(), b"ab");
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    let c1 = root.child(1).unwrap();
    assert!(c0.end_byte() <= c1.start_byte());
}

#[test]
fn pipeline_two_token_root_spans_input() {
    let tree = parse_with(lang_two(), b"ab");
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 2);
}

// ===========================================================================
// 9 – Forest-to-tree pipeline (chain grammar)
// ===========================================================================

#[test]
fn pipeline_chain_has_nontrivial_depth() {
    let tree = parse_with(lang_chain(), b"a");
    assert!(tree_depth(tree.root_node()) >= 1);
}

#[test]
fn pipeline_chain_has_at_least_two_nodes() {
    let tree = parse_with(lang_chain(), b"a");
    assert!(count_nodes(tree.root_node()) >= 2);
}

#[test]
fn pipeline_chain_root_kind_name_resolved() {
    let tree = parse_with(lang_chain(), b"a");
    assert_ne!(tree.root_node().kind(), "unknown");
}

// ===========================================================================
// 10 – Node API on built trees
// ===========================================================================

#[test]
fn node_is_named_true() {
    let tree = parse_with(lang_two(), b"ab");
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_error_false() {
    let tree = parse_with(lang_two(), b"ab");
    assert!(!tree.root_node().is_error());
}

#[test]
fn node_is_missing_false() {
    let tree = parse_with(lang_two(), b"ab");
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_out_of_bounds_child_none() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn node_named_child_count_equals_child_count() {
    let tree = parse_with(lang_two(), b"ab");
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn node_named_child_same_as_child() {
    let tree = parse_with(lang_two(), b"ab");
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let c = root.child(i).unwrap();
        let nc = root.named_child(i).unwrap();
        assert_eq!(c.kind_id(), nc.kind_id());
        assert_eq!(c.start_byte(), nc.start_byte());
    }
}

#[test]
fn node_child_by_field_name_returns_none() {
    let tree = parse_with(lang_two(), b"ab");
    assert!(tree.root_node().child_by_field_name("anything").is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn node_next_sibling_returns_none() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.root_node().next_sibling().is_none());
}

#[test]
fn node_prev_sibling_returns_none() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.root_node().prev_sibling().is_none());
}

#[test]
fn node_start_position_is_zero() {
    let tree = parse_with(lang_single(), b"a");
    assert_eq!(tree.root_node().start_position(), Point::new(0, 0));
}

#[test]
fn node_end_position_is_zero() {
    let tree = parse_with(lang_single(), b"a");
    assert_eq!(tree.root_node().end_position(), Point::new(0, 0));
}

#[test]
fn node_utf8_text_root() {
    let tree = parse_with(lang_two(), b"ab");
    let text = tree.root_node().utf8_text(b"ab").unwrap();
    assert_eq!(text, "ab");
}

#[test]
fn node_utf8_text_child() {
    let tree = parse_with(lang_two(), b"ab");
    let c0 = tree.root_node().child(0).unwrap();
    let text = c0.utf8_text(b"ab").unwrap();
    assert_eq!(text, "a");
}

// ===========================================================================
// 11 – TreeCursor on built trees
// ===========================================================================

#[test]
fn cursor_starts_at_root_depth_zero() {
    let tree = parse_with(lang_two(), b"ab");
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child_increases_depth() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_goto_parent_decreases_depth() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_root_has_no_parent() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_sibling_navigation() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    let first_kind = cursor.node().kind_id();
    assert!(cursor.goto_next_sibling());
    let second_kind = cursor.node().kind_id();
    assert_ne!(first_kind, second_kind);
}

#[test]
fn cursor_no_sibling_past_last_child() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling(); // move to second child
    assert!(!cursor.goto_next_sibling()); // no third child
}

#[test]
fn cursor_leaf_has_no_children() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_reset_returns_to_root() {
    let tree = parse_with(lang_two(), b"ab");
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.reset(&tree);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), tree.root_node().kind_id());
}

#[test]
fn cursor_depth_tracks_chain_grammar() {
    let tree = parse_with(lang_chain(), b"a");
    let mut cursor = TreeCursor::new(&tree);
    let mut max_depth = 0;
    if cursor.goto_first_child() {
        max_depth = max_depth.max(cursor.depth());
        while cursor.goto_first_child() {
            max_depth = max_depth.max(cursor.depth());
        }
    }
    assert!(max_depth >= 1);
}

// ===========================================================================
// 12 – TreeCursor on new_for_testing trees
// ===========================================================================

#[test]
fn cursor_for_testing_tree_walks_all_children() {
    let c1 = Tree::new_for_testing(1, 0, 2, vec![]);
    let c2 = Tree::new_for_testing(2, 2, 4, vec![]);
    let c3 = Tree::new_for_testing(3, 4, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![c1, c2, c3]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn cursor_for_testing_nested_depth() {
    let gc = Tree::new_for_testing(3, 0, 1, vec![]);
    let child = Tree::new_for_testing(2, 0, 1, vec![gc]);
    let tree = Tree::new_for_testing(1, 0, 1, vec![child]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child(); // depth 1
    cursor.goto_first_child(); // depth 2
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().kind_id(), 3);
}

// ===========================================================================
// 13 – Cursor walk count matches manual count
// ===========================================================================

#[test]
fn cursor_walk_count_matches_node_count_parsed_single() {
    let tree = parse_with(lang_single(), b"a");
    assert_eq!(cursor_count_nodes(&tree), count_nodes(tree.root_node()));
}

#[test]
fn cursor_walk_count_matches_node_count_parsed_two() {
    let tree = parse_with(lang_two(), b"ab");
    assert_eq!(cursor_count_nodes(&tree), count_nodes(tree.root_node()));
}

#[test]
fn cursor_walk_count_matches_node_count_parsed_chain() {
    let tree = parse_with(lang_chain(), b"a");
    assert_eq!(cursor_count_nodes(&tree), count_nodes(tree.root_node()));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn cursor_walk_count_matches_for_testing(n in 1usize..15) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n, children);
        prop_assert_eq!(cursor_count_nodes(&tree), count_nodes(tree.root_node()));
    }
}

// ===========================================================================
// 14 – Range invariant property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn pipeline_ranges_valid_all_grammars(choice in 0usize..3) {
        let tree = match choice {
            0 => parse_with(lang_single(), b"a"),
            1 => parse_with(lang_two(), b"ab"),
            _ => parse_with(lang_chain(), b"a"),
        };
        assert_valid_ranges(tree.root_node());
    }
}

// ===========================================================================
// 15 – Determinism: identical parses yield identical shapes
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn deterministic_shape_across_two_parses(choice in 0usize..3) {
        let (l1, l2, input): (Language, Language, &[u8]) = match choice {
            0 => (lang_single(), lang_single(), b"a" as &[u8]),
            1 => (lang_two(), lang_two(), b"ab"),
            _ => (lang_chain(), lang_chain(), b"a"),
        };
        let t1 = parse_with(l1, input);
        let t2 = parse_with(l2, input);
        let mut s1 = Vec::new();
        let mut s2 = Vec::new();
        collect_shape(t1.root_node(), &mut s1);
        collect_shape(t2.root_node(), &mut s2);
        prop_assert_eq!(s1, s2);
    }
}

#[test]
fn deterministic_root_kind_across_ten_parses() {
    let kinds: Vec<u32> = (0..10)
        .map(|_| parse_with(lang_two(), b"ab").root_kind())
        .collect();
    for k in &kinds[1..] {
        assert_eq!(kinds[0], *k);
    }
}

// ===========================================================================
// 16 – Clone of parsed tree identical shape
// ===========================================================================

#[test]
fn clone_parsed_tree_identical_shape() {
    let tree = parse_with(lang_two(), b"ab");
    let cloned = tree.clone();
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    collect_shape(tree.root_node(), &mut s1);
    collect_shape(cloned.root_node(), &mut s2);
    assert_eq!(s1, s2);
}

#[test]
fn clone_parsed_tree_preserves_source() {
    let tree = parse_with(lang_single(), b"a");
    let cloned = tree.clone();
    assert_eq!(tree.source_bytes(), cloned.source_bytes());
}

// ===========================================================================
// 17 – Performance: bounded conversion time
// ===========================================================================

#[test]
fn conversion_completes_quickly_single() {
    let start = Instant::now();
    let _tree = parse_with(lang_single(), b"a");
    assert!(start.elapsed().as_secs() < 5);
}

#[test]
fn conversion_completes_quickly_two() {
    let start = Instant::now();
    let _tree = parse_with(lang_two(), b"ab");
    assert!(start.elapsed().as_secs() < 5);
}

#[test]
fn conversion_completes_quickly_chain() {
    let start = Instant::now();
    let _tree = parse_with(lang_chain(), b"a");
    assert!(start.elapsed().as_secs() < 5);
}

// ===========================================================================
// 18 – Node count >= depth + 1 (pigeonhole)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn node_count_ge_depth_plus_one(choice in 0usize..3) {
        let tree = match choice {
            0 => parse_with(lang_single(), b"a"),
            1 => parse_with(lang_two(), b"ab"),
            _ => parse_with(lang_chain(), b"a"),
        };
        let nc = count_nodes(tree.root_node());
        let d = tree_depth(tree.root_node());
        prop_assert!(nc > d);
    }
}

// ===========================================================================
// 19 – Debug formatting doesn't panic
// ===========================================================================

#[test]
fn debug_tree_parsed() {
    let tree = parse_with(lang_single(), b"a");
    let s = format!("{:?}", tree);
    assert!(!s.is_empty());
}

#[test]
fn debug_tree_stub() {
    let s = format!("{:?}", Tree::new_stub());
    assert!(!s.is_empty());
}

#[test]
fn debug_node() {
    let tree = parse_with(lang_two(), b"ab");
    let s = format!("{:?}", tree.root_node());
    assert!(s.contains("Node"));
}

#[test]
fn debug_node_child() {
    let tree = parse_with(lang_two(), b"ab");
    let c = tree.root_node().child(0).unwrap();
    let s = format!("{:?}", c);
    assert!(s.contains("Node"));
}

// ===========================================================================
// 20 – Parse with old tree (incremental fallback)
// ===========================================================================

#[test]
fn parse_with_old_tree_returns_same_kind() {
    let lang = lang_single();
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    let t1 = p.parse(b"a", None).unwrap();
    let t2 = p.parse(b"a", Some(&t1)).unwrap();
    assert_eq!(t1.root_kind(), t2.root_kind());
}

#[test]
fn parse_utf8_works() {
    let lang = lang_single();
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    let tree = p.parse_utf8("a", None).unwrap();
    assert!(tree.root_node().start_byte() <= tree.root_node().end_byte());
}

// ===========================================================================
// 21 – Point type tests
// ===========================================================================

#[test]
fn point_new_and_fields() {
    let p = Point::new(3, 7);
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn point_display() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{}", p), "1:1"); // 1-indexed display
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(2, 1));
}

#[test]
fn point_ordering() {
    assert!(Point::new(0, 0) < Point::new(0, 1));
    assert!(Point::new(0, 9) < Point::new(1, 0));
}

#[test]
fn point_clone() {
    let p = Point::new(5, 10);
    let p2 = p;
    assert_eq!(p, p2);
}

// ===========================================================================
// 22 – Language API on built languages
// ===========================================================================

#[test]
fn language_symbol_name_lookup() {
    let lang = lang_two();
    assert_eq!(lang.symbol_name(0), Some("EOF"));
    assert_eq!(lang.symbol_name(1), Some("a"));
    assert_eq!(lang.symbol_name(2), Some("b"));
    assert_eq!(lang.symbol_name(3), Some("start"));
}

#[test]
fn language_symbol_name_out_of_bounds() {
    let lang = lang_single();
    assert_eq!(lang.symbol_name(999), None);
}

#[test]
fn language_is_terminal() {
    let lang = lang_two();
    assert!(lang.is_terminal(1));
    assert!(lang.is_terminal(2));
    assert!(!lang.is_terminal(3)); // start is non-terminal
}

#[test]
fn language_is_visible() {
    let lang = lang_two();
    assert!(!lang.is_visible(0)); // EOF not visible
    assert!(lang.is_visible(1)); // "a" visible
}

#[test]
fn language_symbol_for_name() {
    let lang = lang_two();
    // "a" is visible=true, so is_named=true
    assert_eq!(lang.symbol_for_name("a", true), Some(1));
    // "start" is visible=true
    assert_eq!(lang.symbol_for_name("start", true), Some(3));
    // No symbol named "nope"
    assert_eq!(lang.symbol_for_name("nope", true), None);
}

// ===========================================================================
// 23 – Mixed deep and wide tree property
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn mixed_tree_shape_valid_ranges(width in 1usize..8, depth in 1usize..6) {
        // Build a tree that is `depth` deep with `width` children at each level.
        fn build_level(width: usize, depth: usize, base_start: usize) -> Tree {
            if depth == 0 {
                return Tree::new_for_testing(1, base_start, base_start + 1, vec![]);
            }
            let children: Vec<Tree> = (0..width)
                .map(|i| build_level(1, depth - 1, base_start + i))
                .collect();
            let end = base_start + width;
            Tree::new_for_testing(0, base_start, end, children)
        }
        let tree = build_level(width, depth, 0);
        assert_valid_ranges(tree.root_node());
    }
}
