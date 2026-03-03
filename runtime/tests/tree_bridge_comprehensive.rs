#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the tree_bridge module.
//!
//! The tree bridge converts between `parser_v4::Tree` (arena-allocated) and
//! `ForestNode` (GLR parse forest) representations.
//!
//! Since `Tree` fields are `pub(crate)`, we construct forests via the public
//! API, convert to trees with `forest_to_v4_tree`, inspect via the `Node` API,
//! then roundtrip back via `v4_tree_to_forest`.

use adze::glr_incremental::{ForestNode, ForkAlternative};
use adze::subtree::{Subtree, SubtreeNode};
use adze::tree_bridge::{forest_to_v4_tree, v4_tree_to_forest};
use adze_ir::{RuleId, SymbolId};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf_subtree(sym: u16) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: 0..0,
        },
        vec![],
    ))
}

fn branch_subtree(sym: u16, children: Vec<Arc<Subtree>>) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: 0..0,
        },
        children,
    ))
}

fn error_subtree(sym: u16) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: true,
            byte_range: 0..0,
        },
        vec![],
    ))
}

/// A forest leaf with no alternatives (only cached_subtree).
fn leaf_forest(sym: u16) -> Arc<ForestNode> {
    let subtree = leaf_subtree(sym);
    Arc::new(ForestNode {
        symbol: SymbolId(sym),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    })
}

/// A forest leaf with a single alternative.
fn leaf_forest_with_alt(sym: u16) -> Arc<ForestNode> {
    let subtree = leaf_subtree(sym);
    Arc::new(ForestNode {
        symbol: SymbolId(sym),
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: vec![],
            subtree: Arc::clone(&subtree),
        }],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    })
}

/// A forest branch with one alternative containing the given children.
fn branch_forest(sym: u16, children: Vec<Arc<ForestNode>>) -> Arc<ForestNode> {
    let child_subtrees: Vec<Arc<Subtree>> = children
        .iter()
        .map(|c| {
            c.cached_subtree
                .clone()
                .unwrap_or_else(|| leaf_subtree(c.symbol.0))
        })
        .collect();
    let subtree = branch_subtree(sym, child_subtrees);
    Arc::new(ForestNode {
        symbol: SymbolId(sym),
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: children.clone(),
            subtree: Arc::clone(&subtree),
        }],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    })
}

// ===========================================================================
// forest_to_v4_tree tests
// ===========================================================================

#[test]
fn forest_leaf_no_alternatives_becomes_v4_leaf() {
    let forest = ForestNode {
        symbol: SymbolId(42),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();

    assert_eq!(root.symbol(), 42);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn forest_leaf_with_alt_becomes_v4_leaf() {
    let forest_node = leaf_forest_with_alt(7);

    let tree = forest_to_v4_tree(&forest_node);
    let root = tree.root_node();

    assert_eq!(root.symbol(), 7);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn forest_branch_one_child_becomes_v4_branch() {
    let child = leaf_forest_with_alt(5);
    let root_forest = branch_forest(10, vec![child]);

    let tree = forest_to_v4_tree(&root_forest);
    let root = tree.root_node();

    assert_eq!(root.symbol(), 10);
    assert_eq!(root.child_count(), 1);

    let child_node = root.child(0).unwrap();
    assert_eq!(child_node.symbol(), 5);
    assert_eq!(child_node.child_count(), 0);
}

#[test]
fn forest_branch_multiple_children() {
    let children: Vec<_> = (1..=4).map(|s| leaf_forest_with_alt(s)).collect();
    let root_forest = branch_forest(50, children);

    let tree = forest_to_v4_tree(&root_forest);
    let root = tree.root_node();

    assert_eq!(root.child_count(), 4);
    for i in 0..4 {
        let c = root.child(i).unwrap();
        assert_eq!(c.symbol(), (i + 1) as u16);
    }
}

#[test]
fn forest_nested_branch_converts_recursively() {
    let leaf = leaf_forest_with_alt(1);
    let inner = branch_forest(2, vec![leaf]);
    let outer = branch_forest(3, vec![inner]);

    let tree = forest_to_v4_tree(&outer);
    let r = tree.root_node();

    assert_eq!(r.symbol(), 3);
    let mid = r.child(0).unwrap();
    assert_eq!(mid.symbol(), 2);
    let l = mid.child(0).unwrap();
    assert_eq!(l.symbol(), 1);
    assert_eq!(l.child_count(), 0);
}

#[test]
fn forest_empty_alternatives_cached_subtree_with_children_creates_branch() {
    // No alternatives, but cached_subtree has children → branch node
    let leaf_sub = leaf_subtree(9);
    let parent_sub = branch_subtree(11, vec![leaf_sub]);

    let forest = ForestNode {
        symbol: SymbolId(11),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(parent_sub),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();

    assert_eq!(root.symbol(), 11);
    // Branch kind but no children traversable (empty alternatives)
    assert_eq!(root.child_count(), 0);
}

#[test]
fn forest_no_cached_subtree_no_alternatives_is_leaf() {
    let forest = ForestNode {
        symbol: SymbolId(77),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();

    assert_eq!(root.symbol(), 77);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn forest_error_subtree_increments_error_count() {
    let err = error_subtree(99);
    let forest = ForestNode {
        symbol: SymbolId(99),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(err),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 1);
}

#[test]
fn forest_non_error_subtree_zero_error_count() {
    let sub = leaf_subtree(10);
    let forest = ForestNode {
        symbol: SymbolId(10),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(sub),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 0);
}

#[test]
fn forest_no_cached_subtree_zero_error_count() {
    let forest = ForestNode {
        symbol: SymbolId(1),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 0);
}

#[test]
fn forest_selects_first_alternative_for_v4() {
    let child_a = leaf_forest_with_alt(1);
    let child_b = leaf_forest_with_alt(2);

    let sub_a = branch_subtree(100, vec![leaf_subtree(1)]);
    let sub_b = branch_subtree(100, vec![leaf_subtree(2)]);

    let forest = ForestNode {
        symbol: SymbolId(100),
        alternatives: vec![
            ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![Arc::clone(&child_a)],
                subtree: Arc::clone(&sub_a),
            },
            ForkAlternative {
                fork_id: 1,
                rule_id: Some(RuleId(42)),
                children: vec![Arc::clone(&child_b)],
                subtree: sub_b,
            },
        ],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(sub_a),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();

    assert_eq!(root.child_count(), 1);
    // First alternative has child_a (symbol 1)
    assert_eq!(root.child(0).unwrap().symbol(), 1);
}

#[test]
fn forest_two_alternatives_different_arities() {
    let c1 = leaf_forest_with_alt(10);
    let c2 = leaf_forest_with_alt(20);
    let c3 = leaf_forest_with_alt(30);

    let sub1 = branch_subtree(1, vec![leaf_subtree(10), leaf_subtree(20)]);
    let sub2 = branch_subtree(1, vec![leaf_subtree(30)]);

    let forest = ForestNode {
        symbol: SymbolId(1),
        alternatives: vec![
            ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![Arc::clone(&c1), Arc::clone(&c2)],
                subtree: sub1.clone(),
            },
            ForkAlternative {
                fork_id: 1,
                rule_id: Some(RuleId(7)),
                children: vec![Arc::clone(&c3)],
                subtree: sub2,
            },
        ],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(sub1),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();

    // First alternative has 2 children (symbols 10, 20)
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().symbol(), 10);
    assert_eq!(root.child(1).unwrap().symbol(), 20);
}

#[test]
fn forest_deeply_nested_chain_converts() {
    // Build a 6-deep chain: sym6 -> sym5 -> ... -> sym1
    let mut node = leaf_forest_with_alt(1);
    for sym in 2..=6u16 {
        node = branch_forest(sym, vec![node]);
    }

    let tree = forest_to_v4_tree(&node);
    let mut current = tree.root_node();

    for sym in (1..=6u16).rev() {
        assert_eq!(current.symbol(), sym);
        if sym > 1 {
            current = current.child(0).unwrap();
        }
    }
}

#[test]
fn forest_wide_tree_ten_children() {
    let children: Vec<_> = (0..10).map(|i| leaf_forest_with_alt(i)).collect();
    let root_forest = branch_forest(200, children);

    let tree = forest_to_v4_tree(&root_forest);
    let root = tree.root_node();

    assert_eq!(root.symbol(), 200);
    assert_eq!(root.child_count(), 10);
    for i in 0..10 {
        assert_eq!(root.child(i).unwrap().symbol(), i as u16);
    }
}

#[test]
fn forest_symbol_id_zero() {
    let forest = ForestNode {
        symbol: SymbolId(0),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(leaf_subtree(0)),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.root_node().symbol(), 0);
}

#[test]
fn forest_large_symbol_id() {
    let sym: u16 = 1000;
    let forest_node = leaf_forest_with_alt(sym);

    let tree = forest_to_v4_tree(&forest_node);
    assert_eq!(tree.root_node().symbol(), sym);
}

#[test]
fn forest_mixed_leaf_and_branch_children() {
    // leaf(1), branch(3 -> leaf(2))
    let leaf_child = leaf_forest_with_alt(1);
    let inner_leaf = leaf_forest_with_alt(2);
    let branch_child = branch_forest(3, vec![inner_leaf]);

    let root_forest = branch_forest(4, vec![leaf_child, branch_child]);

    let tree = forest_to_v4_tree(&root_forest);
    let root = tree.root_node();

    assert_eq!(root.child_count(), 2);
    let c0 = root.child(0).unwrap();
    assert_eq!(c0.symbol(), 1);
    assert_eq!(c0.child_count(), 0);

    let c1 = root.child(1).unwrap();
    assert_eq!(c1.symbol(), 3);
    assert_eq!(c1.child_count(), 1);
    assert_eq!(c1.child(0).unwrap().symbol(), 2);
}

// ===========================================================================
// Roundtrip tests: forest -> v4 -> forest
// ===========================================================================

#[test]
fn roundtrip_single_leaf() {
    let original = leaf_forest_with_alt(55);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.symbol, SymbolId(55));
    assert_eq!(roundtripped.alternatives.len(), 1);
    assert!(roundtripped.alternatives[0].children.is_empty());
    assert!(roundtripped.cached_subtree.is_some());
    assert_eq!(roundtripped.cached_subtree.as_ref().unwrap().symbol(), 55);
}

#[test]
fn roundtrip_preserves_root_symbol() {
    let child1 = leaf_forest_with_alt(10);
    let child2 = leaf_forest_with_alt(20);
    let original = branch_forest(30, vec![child1, child2]);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.symbol, SymbolId(30));
}

#[test]
fn roundtrip_preserves_child_count() {
    let children: Vec<_> = (1..=5).map(|s| leaf_forest_with_alt(s)).collect();
    let original = branch_forest(99, children);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.alternatives.len(), 1);
    assert_eq!(roundtripped.alternatives[0].children.len(), 5);
}

#[test]
fn roundtrip_preserves_child_symbols() {
    let child1 = leaf_forest_with_alt(10);
    let child2 = leaf_forest_with_alt(20);
    let original = branch_forest(30, vec![child1, child2]);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    let children = &roundtripped.alternatives[0].children;
    assert_eq!(children[0].symbol, SymbolId(10));
    assert_eq!(children[1].symbol, SymbolId(20));
}

#[test]
fn roundtrip_preserves_nesting_depth() {
    let leaf = leaf_forest_with_alt(1);
    let inner = branch_forest(2, vec![leaf]);
    let outer = branch_forest(3, vec![inner]);

    let tree = forest_to_v4_tree(&outer);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.symbol, SymbolId(3));
    let mid = &roundtripped.alternatives[0].children[0];
    assert_eq!(mid.symbol, SymbolId(2));
    let leaf = &mid.alternatives[0].children[0];
    assert_eq!(leaf.symbol, SymbolId(1));
}

#[test]
fn roundtrip_deep_chain() {
    let mut node = leaf_forest_with_alt(0);
    for sym in 1..=8u16 {
        node = branch_forest(sym, vec![node]);
    }

    let tree = forest_to_v4_tree(&node);
    let roundtripped = v4_tree_to_forest(&tree);

    let mut current = &*roundtripped;
    for sym in (0..=8u16).rev() {
        assert_eq!(current.symbol, SymbolId(sym));
        if sym > 0 {
            current = &current.alternatives[0].children[0];
        }
    }
}

#[test]
fn roundtrip_wide_tree() {
    let children: Vec<_> = (0..10).map(|i| leaf_forest_with_alt(i)).collect();
    let original = branch_forest(200, children);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.symbol, SymbolId(200));
    let rt_children = &roundtripped.alternatives[0].children;
    assert_eq!(rt_children.len(), 10);
    for i in 0..10 {
        assert_eq!(rt_children[i].symbol, SymbolId(i as u16));
    }
}

#[test]
fn roundtrip_mixed_arity() {
    // root(20) -> [left(10) -> [a(1), b(2)], right(11) -> [c(3)]]
    let a = leaf_forest_with_alt(1);
    let b = leaf_forest_with_alt(2);
    let c = leaf_forest_with_alt(3);
    let left = branch_forest(10, vec![a, b]);
    let right = branch_forest(11, vec![c]);
    let original = branch_forest(20, vec![left, right]);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.symbol, SymbolId(20));
    let rt_children = &roundtripped.alternatives[0].children;
    assert_eq!(rt_children.len(), 2);

    let rt_left = &rt_children[0];
    assert_eq!(rt_left.symbol, SymbolId(10));
    assert_eq!(rt_left.alternatives[0].children.len(), 2);

    let rt_right = &rt_children[1];
    assert_eq!(rt_right.symbol, SymbolId(11));
    assert_eq!(rt_right.alternatives[0].children.len(), 1);
}

// ===========================================================================
// ForestNode structural properties
// ===========================================================================

#[test]
fn roundtripped_forest_has_single_alternative() {
    let leaf = leaf_forest_with_alt(5);
    let original = branch_forest(10, vec![leaf]);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    // v4_tree_to_forest always produces exactly one alternative per node
    assert_eq!(roundtripped.alternatives.len(), 1);
    assert_eq!(roundtripped.alternatives[0].fork_id, 0);
    assert!(roundtripped.alternatives[0].rule_id.is_none());
}

#[test]
fn roundtripped_forest_cached_subtree_matches_structure() {
    let c1 = leaf_forest_with_alt(7);
    let c2 = leaf_forest_with_alt(8);
    let original = branch_forest(20, vec![c1, c2]);

    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);

    let subtree = roundtripped.cached_subtree.as_ref().unwrap();
    assert_eq!(subtree.symbol(), 20);
    assert_eq!(subtree.children.len(), 2);
    assert_eq!(subtree.children[0].subtree.symbol(), 7);
    assert_eq!(subtree.children[1].subtree.symbol(), 8);
}

#[test]
fn roundtripped_forest_alternative_subtree_shares_cached() {
    let leaf = leaf_forest_with_alt(5);

    let tree = forest_to_v4_tree(&leaf);
    let roundtripped = v4_tree_to_forest(&tree);

    let cached = roundtripped.cached_subtree.as_ref().unwrap();
    let alt_sub = &roundtripped.alternatives[0].subtree;
    assert!(Arc::ptr_eq(cached, alt_sub));
}

#[test]
fn roundtripped_leaf_cached_subtree_has_no_children() {
    let leaf = leaf_forest_with_alt(33);

    let tree = forest_to_v4_tree(&leaf);
    let roundtripped = v4_tree_to_forest(&tree);

    let subtree = roundtripped.cached_subtree.as_ref().unwrap();
    assert!(subtree.children.is_empty());
}

#[test]
fn roundtripped_forest_ranges_are_zero() {
    let leaf = leaf_forest_with_alt(1);

    let tree = forest_to_v4_tree(&leaf);
    let roundtripped = v4_tree_to_forest(&tree);

    assert_eq!(roundtripped.byte_range, 0..0);
    assert_eq!(roundtripped.token_range, 0..0);
}

// ===========================================================================
// Ambiguity / multiple-alternative edge cases
// ===========================================================================

#[test]
fn forest_three_alternatives_first_selected() {
    let ca = leaf_forest_with_alt(1);
    let cb = leaf_forest_with_alt(2);
    let cc = leaf_forest_with_alt(3);

    let sa = branch_subtree(100, vec![leaf_subtree(1)]);
    let sb = branch_subtree(100, vec![leaf_subtree(2)]);
    let sc = branch_subtree(100, vec![leaf_subtree(3)]);

    let forest = ForestNode {
        symbol: SymbolId(100),
        alternatives: vec![
            ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![ca],
                subtree: sa.clone(),
            },
            ForkAlternative {
                fork_id: 1,
                rule_id: None,
                children: vec![cb],
                subtree: sb,
            },
            ForkAlternative {
                fork_id: 2,
                rule_id: None,
                children: vec![cc],
                subtree: sc,
            },
        ],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(sa),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    assert_eq!(root.child(0).unwrap().symbol(), 1);
}
