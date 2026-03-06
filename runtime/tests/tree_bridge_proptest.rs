#![allow(clippy::needless_range_loop)]

//! Property-based tests for tree bridge (forest-to-tree conversion).
//!
//! Uses `proptest` to verify invariants of `forest_to_v4_tree` and
//! `v4_tree_to_forest` over randomly generated forest structures.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::glr_incremental::{ForestNode, ForkAlternative};
use adze::subtree::{Subtree, SubtreeNode};
use adze::tree_bridge::{forest_to_v4_tree, v4_tree_to_forest};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;
use proptest::prelude::*;
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

fn leaf_subtree_ranged(sym: u16, range: std::ops::Range<usize>) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: range,
        },
        vec![],
    ))
}

fn error_leaf_subtree(sym: u16) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: true,
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

/// Build a ForestNode leaf with a single alternative.
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

/// Build a ForestNode leaf with a byte range.
fn leaf_forest_ranged(sym: u16, range: std::ops::Range<usize>) -> Arc<ForestNode> {
    let subtree = leaf_subtree_ranged(sym, range.clone());
    Arc::new(ForestNode {
        symbol: SymbolId(sym),
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: vec![],
            subtree: Arc::clone(&subtree),
        }],
        byte_range: range,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    })
}

/// Build a ForestNode branch with children, single alternative.
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

/// Build a ForestNode with an error cached_subtree.
fn error_forest(sym: u16) -> Arc<ForestNode> {
    let subtree = error_leaf_subtree(sym);
    Arc::new(ForestNode {
        symbol: SymbolId(sym),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    })
}

/// Build a ForestNode with no cached_subtree and no alternatives.
fn bare_forest(sym: u16) -> ForestNode {
    ForestNode {
        symbol: SymbolId(sym),
        alternatives: vec![],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: None,
    }
}

// ---------------------------------------------------------------------------
// Recursive tree counting helpers
// ---------------------------------------------------------------------------

/// Count nodes in a v4 tree by walking via the Node API.
fn count_v4_nodes(node: adze::node::Node<'_>) -> usize {
    let mut total = 1;
    for i in 0..node.child_count() {
        total += count_v4_nodes(node.child(i).unwrap());
    }
    total
}

/// Count nodes in a ForestNode (first-alternative children only).
fn count_forest_nodes(forest: &ForestNode) -> usize {
    let children = forest
        .alternatives
        .first()
        .map(|a| a.children.as_slice())
        .unwrap_or(&[]);
    let mut total = 1;
    for child in children {
        total += count_forest_nodes(child);
    }
    total
}

/// Depth of a ForestNode (first-alternative).
fn forest_depth(forest: &ForestNode) -> usize {
    let children = forest
        .alternatives
        .first()
        .map(|a| a.children.as_slice())
        .unwrap_or(&[]);
    if children.is_empty() {
        1
    } else {
        1 + children.iter().map(|c| forest_depth(c)).max().unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

/// Generate a random symbol id (0..500).
fn arb_symbol() -> impl Strategy<Value = u16> {
    0u16..500
}

/// Generate a random leaf ForestNode.
fn arb_leaf() -> impl Strategy<Value = Arc<ForestNode>> {
    arb_symbol().prop_map(leaf_forest)
}

/// Recursively generate a random ForestNode tree up to `depth`.
fn arb_forest(depth: u32) -> BoxedStrategy<Arc<ForestNode>> {
    if depth == 0 {
        arb_leaf().boxed()
    } else {
        prop_oneof![
            3 => arb_leaf(),
            2 => (arb_symbol(), prop::collection::vec(arb_forest(depth - 1), 1..=4))
                .prop_map(|(sym, children)| branch_forest(sym, children)),
        ]
        .boxed()
    }
}

/// Generate a ForestNode tree with moderate depth.
fn arb_forest_tree() -> BoxedStrategy<Arc<ForestNode>> {
    arb_forest(4)
}

// ===========================================================================
// 1. Conversion produces a valid tree (root symbol readable)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn conversion_produces_valid_tree(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let root = tree.root_node();
        // We can read the root symbol without panic
        let _ = root.symbol();
        let _ = root.child_count();
    }
}

// ===========================================================================
// 2. Root symbol preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn root_symbol_preserved_after_conversion(sym in arb_symbol()) {
        let forest = leaf_forest(sym);
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.root_node().symbol(), sym);
    }
}

// ===========================================================================
// 3. Root symbol preserved for branch
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn branch_root_symbol_preserved(
        sym in arb_symbol(),
        children in prop::collection::vec(arb_leaf(), 1..=5),
    ) {
        let forest = branch_forest(sym, children);
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.root_node().symbol(), sym);
    }
}

// ===========================================================================
// 4. Leaf has zero children
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn leaf_conversion_has_zero_children(sym in arb_symbol()) {
        let forest = leaf_forest(sym);
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.root_node().child_count(), 0);
    }
}

// ===========================================================================
// 5. Child count matches alternative children count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn child_count_matches_alternative(
        sym in arb_symbol(),
        children in prop::collection::vec(arb_leaf(), 1..=8),
    ) {
        let n = children.len();
        let forest = branch_forest(sym, children);
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.root_node().child_count(), n);
    }
}

// ===========================================================================
// 6. Node count: tree has >= forest node count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn node_count_preserved_or_increased(forest in arb_forest_tree()) {
        let forest_count = count_forest_nodes(&forest);
        let tree = forest_to_v4_tree(&forest);
        let tree_count = count_v4_nodes(tree.root_node());
        // The tree should have at least as many nodes as the forest
        // (bridge may create additional structure nodes)
        prop_assert!(
            tree_count >= forest_count,
            "tree_count={} < forest_count={}",
            tree_count,
            forest_count
        );
    }
}

// ===========================================================================
// 7. Roundtrip preserves root symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_preserves_root_symbol(forest in arb_forest_tree()) {
        let original_sym = forest.symbol;
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert_eq!(roundtripped.symbol, original_sym);
    }
}

// ===========================================================================
// 8. Roundtrip always produces single alternative per node
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_produces_single_alternative(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert_eq!(roundtripped.alternatives.len(), 1);
    }
}

// ===========================================================================
// 9. Roundtrip cached subtree is always Some
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_cached_subtree_is_some(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert!(roundtripped.cached_subtree.is_some());
    }
}

// ===========================================================================
// 10. Roundtrip cached subtree symbol matches root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_cached_subtree_symbol_matches(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        let cached = roundtripped.cached_subtree.as_ref().unwrap();
        prop_assert_eq!(cached.symbol(), roundtripped.symbol.0);
    }
}

// ===========================================================================
// 11. Roundtrip preserves child count at root
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_preserves_child_count(
        sym in arb_symbol(),
        children in prop::collection::vec(arb_leaf(), 1..=6),
    ) {
        let n = children.len();
        let forest = branch_forest(sym, children);
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert_eq!(roundtripped.alternatives[0].children.len(), n);
    }
}

// ===========================================================================
// 12. Roundtrip preserves child symbols
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_preserves_child_symbols(
        root_sym in arb_symbol(),
        child_syms in prop::collection::vec(arb_symbol(), 1..=6),
    ) {
        let children: Vec<_> = child_syms.iter().copied().map(leaf_forest).collect();
        let forest = branch_forest(root_sym, children);
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        let rt_children = &roundtripped.alternatives[0].children;
        for i in 0..child_syms.len() {
            prop_assert_eq!(rt_children[i].symbol, SymbolId(child_syms[i]));
        }
    }
}

// ===========================================================================
// 13. Error node tracked in error_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn error_node_counted(sym in arb_symbol()) {
        let forest = error_forest(sym);
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.error_count(), 1);
    }
}

// ===========================================================================
// 14. Non-error node has zero error_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn non_error_has_zero_error_count(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.error_count(), 0);
    }
}

// ===========================================================================
// 15. Bare forest (no cached, no alternatives) becomes leaf
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn bare_forest_becomes_leaf(sym in arb_symbol()) {
        let forest = bare_forest(sym);
        let tree = forest_to_v4_tree(&forest);
        let root = tree.root_node();
        prop_assert_eq!(root.symbol(), sym);
        prop_assert_eq!(root.child_count(), 0);
    }
}

// ===========================================================================
// 16. Empty forest (leaf, no children) conversion is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn empty_forest_conversion_idempotent(sym in arb_symbol()) {
        let forest = leaf_forest(sym);
        let tree1 = forest_to_v4_tree(&forest);
        let rt1 = v4_tree_to_forest(&tree1);
        let tree2 = forest_to_v4_tree(&rt1);
        let rt2 = v4_tree_to_forest(&tree2);
        prop_assert_eq!(rt1.symbol, rt2.symbol);
        prop_assert_eq!(rt1.alternatives.len(), rt2.alternatives.len());
    }
}

// ===========================================================================
// 17. Deep forest conversion does not panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn deep_forest_conversion_no_panic(depth in 5u16..30) {
        let mut node = leaf_forest(0);
        for sym in 1..=depth {
            node = branch_forest(sym, vec![node]);
        }
        let tree = forest_to_v4_tree(&node);
        let root = tree.root_node();
        prop_assert_eq!(root.symbol(), depth);
    }
}

// ===========================================================================
// 18. Deep forest roundtrip preserves depth
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn deep_forest_roundtrip_preserves_depth(depth in 2u16..20) {
        let mut node = leaf_forest(0);
        for sym in 1..=depth {
            node = branch_forest(sym, vec![node]);
        }
        let original_depth = forest_depth(&node);
        let tree = forest_to_v4_tree(&node);
        let roundtripped = v4_tree_to_forest(&tree);
        let rt_depth = forest_depth(&roundtripped);
        prop_assert_eq!(original_depth, rt_depth);
    }
}

// ===========================================================================
// 19. Bridge is deterministic: same input → same output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn bridge_is_deterministic(forest in arb_forest_tree()) {
        let tree1 = forest_to_v4_tree(&forest);
        let tree2 = forest_to_v4_tree(&forest);
        let root1 = tree1.root_node();
        let root2 = tree2.root_node();
        prop_assert_eq!(root1.symbol(), root2.symbol());
        prop_assert_eq!(root1.child_count(), root2.child_count());
        prop_assert_eq!(tree1.error_count(), tree2.error_count());
    }
}

// ===========================================================================
// 20. Roundtrip deterministic: same input → same roundtrip output
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn roundtrip_is_deterministic(forest in arb_forest_tree()) {
        let tree1 = forest_to_v4_tree(&forest);
        let rt1 = v4_tree_to_forest(&tree1);
        let tree2 = forest_to_v4_tree(&forest);
        let rt2 = v4_tree_to_forest(&tree2);
        prop_assert_eq!(rt1.symbol, rt2.symbol);
        prop_assert_eq!(rt1.alternatives.len(), rt2.alternatives.len());
        if !rt1.alternatives.is_empty() && !rt2.alternatives.is_empty() {
            prop_assert_eq!(
                rt1.alternatives[0].children.len(),
                rt2.alternatives[0].children.len()
            );
        }
    }
}

// ===========================================================================
// 21. Byte ranges preserved through ForestNode fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn byte_ranges_present_on_forest(
        sym in arb_symbol(),
        start in 0usize..1000,
        len in 1usize..500,
    ) {
        let end = start + len;
        let forest = leaf_forest_ranged(sym, start..end);
        // The ForestNode should carry the byte range we set
        prop_assert_eq!(forest.byte_range.clone(), start..end);
        // After conversion the tree is created (no panic)
        let tree = forest_to_v4_tree(&forest);
        let _ = tree.root_node().symbol();
    }
}

// ===========================================================================
// 22. Wide tree child symbols all distinct when generated distinctly
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn wide_tree_child_symbols_preserved(
        root_sym in 500u16..600,
        width in 2usize..10,
    ) {
        let children: Vec<_> = (0..width as u16).map(leaf_forest).collect();
        let forest = branch_forest(root_sym, children);
        let tree = forest_to_v4_tree(&forest);
        let root = tree.root_node();
        prop_assert_eq!(root.child_count(), width);
        for i in 0..width {
            prop_assert_eq!(root.child(i).unwrap().symbol(), i as u16);
        }
    }
}

// ===========================================================================
// 23. Symbol ID zero is a valid symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn symbol_zero_valid(children_count in 0usize..5) {
        let children: Vec<_> = (1..=children_count as u16).map(leaf_forest).collect();
        let forest = if children.is_empty() {
            leaf_forest(0)
        } else {
            branch_forest(0, children)
        };
        let tree = forest_to_v4_tree(&forest);
        prop_assert_eq!(tree.root_node().symbol(), 0);
    }
}

// ===========================================================================
// 24. Roundtrip fork_id always 0 and rule_id always None
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_fork_metadata(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert_eq!(roundtripped.alternatives[0].fork_id, 0);
        prop_assert!(roundtripped.alternatives[0].rule_id.is_none());
    }
}

// ===========================================================================
// 25. Roundtrip byte_range and token_range are 0..0
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_ranges_are_zeroed(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert_eq!(roundtripped.byte_range.clone(), 0..0);
        prop_assert_eq!(roundtripped.token_range.clone(), 0..0);
    }
}

// ===========================================================================
// 26. Alternative subtree shares cached_subtree pointer after roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn roundtrip_alt_subtree_shares_cached(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        let cached = roundtripped.cached_subtree.as_ref().unwrap();
        let alt_sub = &roundtripped.alternatives[0].subtree;
        prop_assert!(Arc::ptr_eq(cached, alt_sub));
    }
}

// ===========================================================================
// 27. Nested branch children are accessible after conversion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn nested_children_accessible(
        root_sym in arb_symbol(),
        mid_sym in arb_symbol(),
        leaf_sym in arb_symbol(),
    ) {
        let leaf = leaf_forest(leaf_sym);
        let mid = branch_forest(mid_sym, vec![leaf]);
        let root = branch_forest(root_sym, vec![mid]);
        let tree = forest_to_v4_tree(&root);
        let root_node = tree.root_node();
        prop_assert_eq!(root_node.symbol(), root_sym);
        let mid_node = root_node.child(0).unwrap();
        prop_assert_eq!(mid_node.symbol(), mid_sym);
        let leaf_node = mid_node.child(0).unwrap();
        prop_assert_eq!(leaf_node.symbol(), leaf_sym);
        prop_assert_eq!(leaf_node.child_count(), 0);
    }
}

// ===========================================================================
// 28. Double roundtrip preserves root symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn double_roundtrip_preserves_root(forest in arb_forest_tree()) {
        let original_sym = forest.symbol;
        let tree1 = forest_to_v4_tree(&forest);
        let rt1 = v4_tree_to_forest(&tree1);
        let tree2 = forest_to_v4_tree(&rt1);
        let rt2 = v4_tree_to_forest(&tree2);
        prop_assert_eq!(rt2.symbol, original_sym);
    }
}

// ===========================================================================
// 29. Double roundtrip preserves child count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn double_roundtrip_preserves_child_count(
        sym in arb_symbol(),
        children in prop::collection::vec(arb_leaf(), 1..=6),
    ) {
        let n = children.len();
        let forest = branch_forest(sym, children);
        let tree1 = forest_to_v4_tree(&forest);
        let rt1 = v4_tree_to_forest(&tree1);
        let tree2 = forest_to_v4_tree(&rt1);
        let rt2 = v4_tree_to_forest(&tree2);
        prop_assert_eq!(rt2.alternatives[0].children.len(), n);
    }
}

// ===========================================================================
// 30. Leaf roundtrip cached subtree has no children
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn leaf_roundtrip_cached_subtree_no_children(sym in arb_symbol()) {
        let forest = leaf_forest(sym);
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        let cached = roundtripped.cached_subtree.as_ref().unwrap();
        prop_assert!(cached.children.is_empty());
    }
}

// ===========================================================================
// 31. Branch roundtrip cached subtree children match count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn branch_roundtrip_cached_children_count(
        sym in arb_symbol(),
        child_syms in prop::collection::vec(arb_symbol(), 1..=6),
    ) {
        let children: Vec<_> = child_syms.iter().copied().map(leaf_forest).collect();
        let forest = branch_forest(sym, children);
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        let cached = roundtripped.cached_subtree.as_ref().unwrap();
        prop_assert_eq!(cached.children.len(), child_syms.len());
    }
}

// ===========================================================================
// 32. Conversion tree error_count is 0 for generated (non-error) forests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn generated_forest_error_count_zero(forest in arb_forest_tree()) {
        let tree = forest_to_v4_tree(&forest);
        // Our strategy never generates error nodes
        prop_assert_eq!(tree.error_count(), 0);
    }
}

// ===========================================================================
// 33. Bare forest roundtrip: bare → tree → forest still valid
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn bare_forest_roundtrip(sym in arb_symbol()) {
        let forest = bare_forest(sym);
        let tree = forest_to_v4_tree(&forest);
        let roundtripped = v4_tree_to_forest(&tree);
        prop_assert_eq!(roundtripped.symbol, SymbolId(sym));
        prop_assert_eq!(roundtripped.alternatives.len(), 1);
        prop_assert!(roundtripped.cached_subtree.is_some());
    }
}
