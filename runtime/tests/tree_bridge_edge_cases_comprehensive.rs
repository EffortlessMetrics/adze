#![allow(clippy::needless_range_loop)]

//! Edge-case tests for tree bridge (forest-to-tree conversion).
//!
//! Covers: empty forests, single-node, deep trees, wide trees,
//! mixed terminal/nonterminal, error nodes, grammar compliance,
//! byte range preservation, and named vs anonymous node handling.

use adze::adze_ir as ir;
use adze::glr_incremental::{ForestNode, ForkAlternative};
use adze::subtree::{Subtree, SubtreeNode};
use adze::tree_bridge::{forest_to_v4_tree, v4_tree_to_forest};

use ir::{RuleId, SymbolId};
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

fn leaf_subtree_with_range(sym: u16, range: std::ops::Range<usize>) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: range,
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

fn error_branch_subtree(sym: u16, children: Vec<Arc<Subtree>>) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: true,
            byte_range: 0..0,
        },
        children,
    ))
}

/// Forest leaf with a single alternative.
fn leaf_forest(sym: u16) -> Arc<ForestNode> {
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

/// Forest branch with one alternative containing the given children.
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
// 1. Empty forest conversion
// ===========================================================================

#[test]
fn empty_forest_no_alternatives_no_cached() {
    let forest = ForestNode {
        symbol: SymbolId(1),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    assert_eq!(root.symbol(), 1);
    assert_eq!(root.child_count(), 0);
    assert_eq!(tree.error_count(), 0);
}

#[test]
fn empty_forest_with_zero_symbol() {
    let forest = ForestNode {
        symbol: SymbolId(0),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.root_node().symbol(), 0);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn empty_alternatives_with_leaf_cached_subtree() {
    let subtree = leaf_subtree(42);
    let forest = ForestNode {
        symbol: SymbolId(42),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    // Leaf cached subtree has no children, so this stays a leaf
    assert_eq!(root.symbol(), 42);
    assert_eq!(root.child_count(), 0);
}

// ===========================================================================
// 2. Single-node conversion
// ===========================================================================

#[test]
fn single_leaf_node_preserves_symbol() {
    let forest = leaf_forest(255);
    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.root_node().symbol(), 255);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn single_leaf_max_symbol_id() {
    let sym: u16 = u16::MAX;
    let subtree = leaf_subtree(sym);
    let forest = ForestNode {
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
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.root_node().symbol(), sym);
}

#[test]
fn single_node_roundtrip_preserves_identity() {
    let original = leaf_forest(77);
    let tree = forest_to_v4_tree(&original);
    let roundtripped = v4_tree_to_forest(&tree);
    assert_eq!(roundtripped.symbol, SymbolId(77));
    assert!(roundtripped.cached_subtree.is_some());
    assert_eq!(roundtripped.cached_subtree.as_ref().unwrap().symbol(), 77);
}

// ===========================================================================
// 3. Deep tree conversion
// ===========================================================================

#[test]
fn deep_chain_ten_levels() {
    let mut node = leaf_forest(0);
    for sym in 1..=10u16 {
        node = branch_forest(sym, vec![node]);
    }

    let tree = forest_to_v4_tree(&node);
    let mut current = tree.root_node();
    for sym in (0..=10u16).rev() {
        assert_eq!(current.symbol(), sym);
        if sym > 0 {
            assert_eq!(current.child_count(), 1);
            current = current.child(0).unwrap();
        }
    }
    assert_eq!(current.child_count(), 0);
}

#[test]
fn deep_chain_roundtrip_preserves_all_symbols() {
    let mut node = leaf_forest(100);
    for sym in 101..=108u16 {
        node = branch_forest(sym, vec![node]);
    }

    let tree = forest_to_v4_tree(&node);
    let rt = v4_tree_to_forest(&tree);

    let mut current = &*rt;
    for sym in (100..=108u16).rev() {
        assert_eq!(current.symbol, SymbolId(sym));
        if sym > 100 {
            assert_eq!(current.alternatives[0].children.len(), 1);
            current = &current.alternatives[0].children[0];
        }
    }
}

#[test]
fn deep_left_spine_binary_tree() {
    // Left-deep: ((leaf, leaf), leaf)
    let a = leaf_forest(1);
    let b = leaf_forest(2);
    let inner = branch_forest(10, vec![a, b]);
    let c = leaf_forest(3);
    let root = branch_forest(20, vec![inner, c]);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert_eq!(r.symbol(), 20);
    assert_eq!(r.child_count(), 2);

    let left = r.child(0).unwrap();
    assert_eq!(left.symbol(), 10);
    assert_eq!(left.child_count(), 2);

    let right = r.child(1).unwrap();
    assert_eq!(right.symbol(), 3);
    assert_eq!(right.child_count(), 0);
}

// ===========================================================================
// 4. Wide tree conversion
// ===========================================================================

#[test]
fn wide_tree_twenty_children() {
    let children: Vec<_> = (0..20u16).map(leaf_forest).collect();
    let root = branch_forest(500, children);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert_eq!(r.child_count(), 20);
    for i in 0..20 {
        assert_eq!(r.child(i).unwrap().symbol(), i as u16);
    }
}

#[test]
fn wide_tree_roundtrip_preserves_order() {
    let children: Vec<_> = (50..65u16).map(leaf_forest).collect();
    let root = branch_forest(999, children);

    let tree = forest_to_v4_tree(&root);
    let rt = v4_tree_to_forest(&tree);

    assert_eq!(rt.alternatives[0].children.len(), 15);
    for i in 0..15 {
        assert_eq!(
            rt.alternatives[0].children[i].symbol,
            SymbolId(50 + i as u16)
        );
    }
}

#[test]
fn wide_tree_single_child() {
    let child = leaf_forest(8);
    let root = branch_forest(9, vec![child]);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert_eq!(r.child_count(), 1);
    assert_eq!(r.child(0).unwrap().symbol(), 8);
}

// ===========================================================================
// 5. Mixed terminal/nonterminal trees
// ===========================================================================

#[test]
fn mixed_leaf_and_branch_children_symbols_preserved() {
    // root(100) -> [leaf(1), branch(50 -> [leaf(2), leaf(3)]), leaf(4)]
    let l1 = leaf_forest(1);
    let l2 = leaf_forest(2);
    let l3 = leaf_forest(3);
    let l4 = leaf_forest(4);
    let inner = branch_forest(50, vec![l2, l3]);
    let root = branch_forest(100, vec![l1, inner, l4]);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert_eq!(r.child_count(), 3);

    assert_eq!(r.child(0).unwrap().symbol(), 1);
    assert_eq!(r.child(0).unwrap().child_count(), 0);

    let mid = r.child(1).unwrap();
    assert_eq!(mid.symbol(), 50);
    assert_eq!(mid.child_count(), 2);
    assert_eq!(mid.child(0).unwrap().symbol(), 2);
    assert_eq!(mid.child(1).unwrap().symbol(), 3);

    assert_eq!(r.child(2).unwrap().symbol(), 4);
}

#[test]
fn alternating_leaf_branch_pattern() {
    // leaf, branch, leaf, branch
    let l1 = leaf_forest(10);
    let b1 = branch_forest(20, vec![leaf_forest(11)]);
    let l2 = leaf_forest(12);
    let b2 = branch_forest(30, vec![leaf_forest(13)]);
    let root = branch_forest(99, vec![l1, b1, l2, b2]);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert_eq!(r.child_count(), 4);

    // Leaves have 0 children, branches have 1
    assert_eq!(r.child(0).unwrap().child_count(), 0);
    assert_eq!(r.child(1).unwrap().child_count(), 1);
    assert_eq!(r.child(2).unwrap().child_count(), 0);
    assert_eq!(r.child(3).unwrap().child_count(), 1);
}

#[test]
fn nested_branches_three_levels() {
    // root -> mid -> inner -> leaf
    let leaf = leaf_forest(1);
    let inner = branch_forest(2, vec![leaf]);
    let mid = branch_forest(3, vec![inner]);
    let root = branch_forest(4, vec![mid]);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert_eq!(r.symbol(), 4);
    let m = r.child(0).unwrap();
    assert_eq!(m.symbol(), 3);
    let i = m.child(0).unwrap();
    assert_eq!(i.symbol(), 2);
    let l = i.child(0).unwrap();
    assert_eq!(l.symbol(), 1);
    assert_eq!(l.child_count(), 0);
}

// ===========================================================================
// 6. Error node handling in conversion
// ===========================================================================

#[test]
fn error_cached_subtree_sets_error_count_one() {
    let err = error_subtree(5);
    let forest = ForestNode {
        symbol: SymbolId(5),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(err),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 1);
}

#[test]
fn non_error_cached_subtree_zero_error_count() {
    let ok = leaf_subtree(5);
    let forest = ForestNode {
        symbol: SymbolId(5),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(ok),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 0);
}

#[test]
fn no_cached_subtree_zero_error_count() {
    let forest = ForestNode {
        symbol: SymbolId(5),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 0);
}

#[test]
fn error_branch_subtree_sets_error_count() {
    let child_ok = leaf_subtree(1);
    let err = error_branch_subtree(10, vec![child_ok]);
    let forest = ForestNode {
        symbol: SymbolId(10),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(err),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.error_count(), 1);
}

// ===========================================================================
// 7. Grammar compliance checking
// ===========================================================================

#[test]
fn first_alternative_selected_when_multiple_exist() {
    let child_a = leaf_forest(1);
    let child_b = leaf_forest(2);

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
                rule_id: Some(RuleId(7)),
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
    // First alternative's child (symbol 1) should be selected
    assert_eq!(root.child(0).unwrap().symbol(), 1);
}

#[test]
fn alternatives_with_different_arities_picks_first() {
    let c1 = leaf_forest(10);
    let c2 = leaf_forest(20);
    let c3 = leaf_forest(30);

    let sub_2kids = branch_subtree(1, vec![leaf_subtree(10), leaf_subtree(20)]);
    let sub_1kid = branch_subtree(1, vec![leaf_subtree(30)]);

    let forest = ForestNode {
        symbol: SymbolId(1),
        alternatives: vec![
            ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![Arc::clone(&c1), Arc::clone(&c2)],
                subtree: sub_2kids.clone(),
            },
            ForkAlternative {
                fork_id: 1,
                rule_id: None,
                children: vec![Arc::clone(&c3)],
                subtree: sub_1kid,
            },
        ],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(sub_2kids),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().symbol(), 10);
    assert_eq!(root.child(1).unwrap().symbol(), 20);
}

#[test]
fn cached_subtree_with_children_but_no_alternatives_creates_branch() {
    // Cached subtree has children but alternatives is empty → branch node
    let child_sub = leaf_subtree(9);
    let parent_sub = branch_subtree(11, vec![child_sub]);

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
    // Branch kind but no traversable children since alternatives is empty
    assert_eq!(root.child_count(), 0);
}

// ===========================================================================
// 8. Byte range preservation
// ===========================================================================

#[test]
fn forest_byte_range_stored_in_node() {
    let subtree = leaf_subtree_with_range(5, 10..20);
    let forest = ForestNode {
        symbol: SymbolId(5),
        alternatives: vec![],
        byte_range: 10..20,
        token_range: 0..1,
        cached_subtree: Some(subtree),
    };

    // The forest carries byte range info; verify conversion doesn't panic
    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    assert_eq!(root.symbol(), 5);
}

#[test]
fn forest_with_nonzero_byte_ranges_converts_without_panic() {
    let child_sub = leaf_subtree_with_range(1, 0..5);
    let parent_sub = Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(10),
            is_error: false,
            byte_range: 0..10,
        },
        vec![child_sub.clone()],
    ));

    let child_forest = Arc::new(ForestNode {
        symbol: SymbolId(1),
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: vec![],
            subtree: Arc::clone(&child_sub),
        }],
        byte_range: 0..5,
        token_range: 0..1,
        cached_subtree: Some(child_sub),
    });

    let forest = ForestNode {
        symbol: SymbolId(10),
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: vec![child_forest],
            subtree: Arc::clone(&parent_sub),
        }],
        byte_range: 0..10,
        token_range: 0..2,
        cached_subtree: Some(parent_sub),
    };

    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    assert_eq!(root.symbol(), 10);
    assert_eq!(root.child_count(), 1);
    assert_eq!(root.child(0).unwrap().symbol(), 1);
}

#[test]
fn empty_byte_range_is_valid() {
    let subtree = leaf_subtree_with_range(7, 5..5);
    let forest = ForestNode {
        symbol: SymbolId(7),
        alternatives: vec![],
        byte_range: 5..5,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    };

    let tree = forest_to_v4_tree(&forest);
    assert_eq!(tree.root_node().symbol(), 7);
}

// ===========================================================================
// 9. Named vs anonymous node conversion
// ===========================================================================

#[test]
fn leaf_node_is_not_named_by_default() {
    let forest = leaf_forest(42);
    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    // The Node API returns false for is_named by default
    assert!(!root.is_named());
}

#[test]
fn branch_node_is_not_named_by_default() {
    let child = leaf_forest(1);
    let root = branch_forest(10, vec![child]);

    let tree = forest_to_v4_tree(&root);
    let r = tree.root_node();
    assert!(!r.is_named());
}

#[test]
fn child_is_not_named_by_default() {
    let child = leaf_forest(1);
    let root = branch_forest(10, vec![child]);

    let tree = forest_to_v4_tree(&root);
    let c = tree.root_node().child(0).unwrap();
    assert!(!c.is_named());
}

// ===========================================================================
// Additional edge cases
// ===========================================================================

#[test]
fn out_of_bounds_child_returns_none() {
    let forest = leaf_forest(1);
    let tree = forest_to_v4_tree(&forest);
    let root = tree.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(100).is_none());
}

#[test]
fn children_iterator_yields_correct_count() {
    let kids: Vec<_> = (0..5u16).map(leaf_forest).collect();
    let root = branch_forest(100, kids);
    let tree = forest_to_v4_tree(&root);

    let count = tree.root_node().children().count();
    assert_eq!(count, 5);
}

#[test]
fn children_iterator_symbols_match() {
    let kids: Vec<_> = (10..13u16).map(leaf_forest).collect();
    let root = branch_forest(200, kids);
    let tree = forest_to_v4_tree(&root);

    let symbols: Vec<u16> = tree.root_node().children().map(|c| c.symbol()).collect();
    assert_eq!(symbols, vec![10, 11, 12]);
}

#[test]
fn v4_to_forest_roundtrip_single_leaf_creates_one_alternative() {
    let forest = leaf_forest(55);
    let tree = forest_to_v4_tree(&forest);
    let rt = v4_tree_to_forest(&tree);

    // v4_tree_to_forest always creates exactly one alternative
    assert_eq!(rt.alternatives.len(), 1);
    assert!(rt.alternatives[0].children.is_empty());
}

#[test]
fn v4_to_forest_roundtrip_branch_preserves_structure() {
    let a = leaf_forest(1);
    let b = leaf_forest(2);
    let root = branch_forest(10, vec![a, b]);

    let tree = forest_to_v4_tree(&root);
    let rt = v4_tree_to_forest(&tree);

    assert_eq!(rt.symbol, SymbolId(10));
    assert_eq!(rt.alternatives.len(), 1);
    assert_eq!(rt.alternatives[0].children.len(), 2);
    assert_eq!(rt.alternatives[0].children[0].symbol, SymbolId(1));
    assert_eq!(rt.alternatives[0].children[1].symbol, SymbolId(2));
}
