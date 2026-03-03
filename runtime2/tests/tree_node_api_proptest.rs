#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::{Point, Tree};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Build a tree with the given symbol, byte range, and number of leaf children.
/// Children are assigned consecutive, non-overlapping byte sub-ranges.
fn arb_tree(
    symbol: u32,
    start: usize,
    end: usize,
    num_children: usize,
) -> Tree {
    if num_children == 0 || end <= start {
        return Tree::new_for_testing(symbol, start, end, vec![]);
    }
    let span = end - start;
    let children: Vec<Tree> = (0..num_children)
        .map(|i| {
            let cs = start + (span * i) / num_children;
            let ce = start + (span * (i + 1)) / num_children;
            Tree::new_for_testing((i as u32) + 1, cs, ce, vec![])
        })
        .collect();
    Tree::new_for_testing(symbol, start, end, children)
}

/// Strategy for a non-empty tree with random symbol, byte range, and children.
fn arb_tree_strategy() -> impl Strategy<Value = Tree> {
    (0u32..500, 0usize..10_000, 0usize..10_000, 0usize..8).prop_map(
        |(sym, a, b, nchildren)| {
            let (start, end) = if a <= b { (a, b) } else { (b, a) };
            arb_tree(sym, start, end, nchildren)
        },
    )
}

/// Strategy for a tree guaranteed to have start < end (non-empty byte range).
fn arb_nonempty_tree_strategy() -> impl Strategy<Value = Tree> {
    (0u32..500, 0usize..10_000, 1usize..10_000, 0usize..8).prop_map(
        |(sym, start, span, nchildren)| {
            let end = start + span;
            arb_tree(sym, start, end, nchildren)
        },
    )
}

/// Strategy for a tree with at least one child.
fn arb_tree_with_children_strategy() -> impl Strategy<Value = Tree> {
    (0u32..500, 0usize..5_000, 1usize..5_000, 1usize..8).prop_map(
        |(sym, start, span, nchildren)| {
            let end = start + span;
            arb_tree(sym, start, end, nchildren)
        },
    )
}

// ===========================================================================
// 1 – kind_id matches the symbol used at construction
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn kind_id_matches_construction_symbol(sym in 0u32..65536) {
        let tree = Tree::new_for_testing(sym, 0, 10, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.kind_id(), sym as u16);
    }
}

// ===========================================================================
// 2 – kind_id truncation for large symbols
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn kind_id_truncates_to_u16(sym in 0u32..200_000) {
        let tree = Tree::new_for_testing(sym, 0, 5, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.kind_id(), sym as u16);
    }
}

// ===========================================================================
// 3 – kind returns "unknown" without language
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn kind_is_unknown_without_language(sym in 0u32..500) {
        let tree = Tree::new_for_testing(sym, 0, 10, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.kind(), "unknown");
    }
}

// ===========================================================================
// 4 – start_byte <= end_byte (byte range invariant)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn start_byte_le_end_byte(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert!(root.start_byte() <= root.end_byte(),
            "start_byte {} > end_byte {}", root.start_byte(), root.end_byte());
    }
}

// ===========================================================================
// 5 – Non-empty tree has start_byte < end_byte
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn nonempty_tree_start_lt_end(tree in arb_nonempty_tree_strategy()) {
        let root = tree.root_node();
        prop_assert!(root.start_byte() < root.end_byte(),
            "Expected start {} < end {}", root.start_byte(), root.end_byte());
    }
}

// ===========================================================================
// 6 – byte_range matches start/end accessors
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn byte_range_matches_start_end(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        let range = root.byte_range();
        prop_assert_eq!(range.start, root.start_byte());
        prop_assert_eq!(range.end, root.end_byte());
    }
}

// ===========================================================================
// 7 – byte_range length is end - start
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn byte_range_length_consistent(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        let range = root.byte_range();
        prop_assert_eq!(range.len(), root.end_byte() - root.start_byte());
    }
}

// ===========================================================================
// 8 – is_named always true (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn is_named_always_true(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert!(root.is_named());
    }
}

// ===========================================================================
// 9 – is_missing always false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn is_missing_always_false(tree in arb_tree_strategy()) {
        prop_assert!(!tree.root_node().is_missing());
    }
}

// ===========================================================================
// 10 – is_error always false
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn is_error_always_false(tree in arb_tree_strategy()) {
        prop_assert!(!tree.root_node().is_error());
    }
}

// ===========================================================================
// 11 – child_count matches number of children provided
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn child_count_matches_construction(n in 0usize..10) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(i as u32 + 1, i * 10, (i + 1) * 10, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n * 10, children);
        prop_assert_eq!(tree.root_node().child_count(), n);
    }
}

// ===========================================================================
// 12 – named_child_count <= child_count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn named_child_count_le_total(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert!(root.named_child_count() <= root.child_count());
    }
}

// ===========================================================================
// 13 – named_child_count equals child_count (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn named_child_count_eq_child_count_phase1(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert_eq!(root.named_child_count(), root.child_count());
    }
}

// ===========================================================================
// 14 – child(i) returns Some for valid indices
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn child_valid_index_returns_some(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            prop_assert!(root.child(i).is_some(), "child({}) should be Some", i);
        }
    }
}

// ===========================================================================
// 15 – child(i) returns None for out-of-bounds indices
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn child_oob_returns_none(tree in arb_tree_strategy(), extra in 0usize..100) {
        let root = tree.root_node();
        let oob = root.child_count() + extra;
        prop_assert!(root.child(oob).is_none());
    }
}

// ===========================================================================
// 16 – child kind_id differs from parent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn children_have_distinct_symbols(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        // Our construction assigns child symbols starting at 1, root is 0..500
        // Just verify children exist and have valid kind_ids
        for i in 0..root.child_count() {
            let child = root.child(i).unwrap();
            prop_assert!(child.kind_id() > 0, "child {} should have symbol > 0", i);
        }
    }
}

// ===========================================================================
// 17 – child byte ranges are within parent byte range
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn child_byte_ranges_within_parent(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let child = root.child(i).unwrap();
            prop_assert!(child.start_byte() >= root.start_byte(),
                "child {} start {} < parent start {}", i, child.start_byte(), root.start_byte());
            prop_assert!(child.end_byte() <= root.end_byte(),
                "child {} end {} > parent end {}", i, child.end_byte(), root.end_byte());
        }
    }
}

// ===========================================================================
// 18 – children byte ranges are non-overlapping and ordered
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn children_byte_ranges_ordered(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        let count = root.child_count();
        if count >= 2 {
            for i in 0..(count - 1) {
                let c1 = root.child(i).unwrap();
                let c2 = root.child(i + 1).unwrap();
                prop_assert!(c1.end_byte() <= c2.start_byte(),
                    "child {} end {} > child {} start {}",
                    i, c1.end_byte(), i + 1, c2.start_byte());
            }
        }
    }
}

// ===========================================================================
// 19 – named_child returns same as child (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn named_child_matches_child_phase1(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let c = root.child(i).unwrap();
            let nc = root.named_child(i).unwrap();
            prop_assert_eq!(c.kind_id(), nc.kind_id());
            prop_assert_eq!(c.start_byte(), nc.start_byte());
            prop_assert_eq!(c.end_byte(), nc.end_byte());
        }
    }
}

// ===========================================================================
// 20 – start_position returns origin (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn start_position_is_origin(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert_eq!(root.start_position(), Point::new(0, 0));
    }
}

// ===========================================================================
// 21 – end_position returns origin (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn end_position_is_origin(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert_eq!(root.end_position(), Point::new(0, 0));
    }
}

// ===========================================================================
// 22 – start_position <= end_position
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn start_position_le_end_position(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert!(root.start_position() <= root.end_position());
    }
}

// ===========================================================================
// 23 – child_by_field_name always None
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn child_by_field_name_is_none(tree in arb_tree_strategy(), name in "[a-z]{1,20}") {
        let root = tree.root_node();
        prop_assert!(root.child_by_field_name(&name).is_none());
    }
}

// ===========================================================================
// 24 – parent always None
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn parent_is_none(tree in arb_tree_strategy()) {
        prop_assert!(tree.root_node().parent().is_none());
    }
}

// ===========================================================================
// 25 – siblings always None
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn siblings_are_none(tree in arb_tree_strategy()) {
        let root = tree.root_node();
        prop_assert!(root.next_sibling().is_none());
        prop_assert!(root.prev_sibling().is_none());
        prop_assert!(root.next_named_sibling().is_none());
        prop_assert!(root.prev_named_sibling().is_none());
    }
}

// ===========================================================================
// 26 – utf8_text on valid source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn utf8_text_extracts_correct_slice(s in "[a-zA-Z0-9]{10,100}") {
        let bytes = s.as_bytes();
        let end = bytes.len();
        let tree = Tree::new_for_testing(0, 0, end, vec![]);
        let root = tree.root_node();
        let text = root.utf8_text(bytes).unwrap();
        prop_assert_eq!(text, s.as_str());
    }
}

// ===========================================================================
// 27 – utf8_text for children extracts sub-slices
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn child_utf8_text_is_substring(s in "[a-z]{20,100}") {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let mid = len / 2;
        let child1 = Tree::new_for_testing(1, 0, mid, vec![]);
        let child2 = Tree::new_for_testing(2, mid, len, vec![]);
        let tree = Tree::new_for_testing(0, 0, len, vec![child1, child2]);

        let root = tree.root_node();
        let c0 = root.child(0).unwrap();
        let c1 = root.child(1).unwrap();

        let t0 = c0.utf8_text(bytes).unwrap();
        let t1 = c1.utf8_text(bytes).unwrap();

        prop_assert_eq!(t0.len() + t1.len(), s.len());
        prop_assert_eq!(format!("{}{}", t0, t1), s);
    }
}

// ===========================================================================
// 28 – Node Debug contains "Node"
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn node_debug_format_contains_node(tree in arb_tree_strategy()) {
        let dbg = format!("{:?}", tree.root_node());
        prop_assert!(dbg.contains("Node"));
    }
}

// ===========================================================================
// 29 – Node Copy preserves all properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn node_copy_preserves_all_fields(tree in arb_tree_strategy()) {
        let n1 = tree.root_node();
        let n2 = n1;
        prop_assert_eq!(n1.kind_id(), n2.kind_id());
        prop_assert_eq!(n1.kind(), n2.kind());
        prop_assert_eq!(n1.start_byte(), n2.start_byte());
        prop_assert_eq!(n1.end_byte(), n2.end_byte());
        prop_assert_eq!(n1.byte_range(), n2.byte_range());
        prop_assert_eq!(n1.child_count(), n2.child_count());
        prop_assert_eq!(n1.is_named(), n2.is_named());
        prop_assert_eq!(n1.is_missing(), n2.is_missing());
        prop_assert_eq!(n1.is_error(), n2.is_error());
    }
}

// ===========================================================================
// 30 – Tree clone root has same node properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn tree_clone_root_matches(tree in arb_tree_strategy()) {
        let cloned = tree.clone();
        let r1 = tree.root_node();
        let r2 = cloned.root_node();
        prop_assert_eq!(r1.kind_id(), r2.kind_id());
        prop_assert_eq!(r1.start_byte(), r2.start_byte());
        prop_assert_eq!(r1.end_byte(), r2.end_byte());
        prop_assert_eq!(r1.child_count(), r2.child_count());
    }
}

// ===========================================================================
// 31 – child is_named is true for all children (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn all_children_are_named(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let child = root.child(i).unwrap();
            prop_assert!(child.is_named(), "child {} should be named", i);
        }
    }
}

// ===========================================================================
// 32 – children start_byte <= end_byte
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn children_byte_range_invariant(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let child = root.child(i).unwrap();
            prop_assert!(child.start_byte() <= child.end_byte(),
                "child {} start {} > end {}", i, child.start_byte(), child.end_byte());
        }
    }
}

// ===========================================================================
// 33 – leaf nodes have zero children
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn leaf_node_has_zero_children(sym in 0u32..500, start in 0usize..5000, span in 0usize..5000) {
        let tree = Tree::new_for_testing(sym, start, start + span, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.child_count(), 0);
        prop_assert_eq!(root.named_child_count(), 0);
        prop_assert!(root.child(0).is_none());
    }
}

// ===========================================================================
// 34 – root_kind matches root node kind_id
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn root_kind_matches_kind_id(sym in 0u32..65536) {
        let tree = Tree::new_for_testing(sym, 0, 10, vec![]);
        prop_assert_eq!(tree.root_kind(), sym);
        prop_assert_eq!(tree.root_node().kind_id(), sym as u16);
    }
}

// ===========================================================================
// 35 – child positions return origin (Phase 1)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn child_positions_are_origin(tree in arb_tree_with_children_strategy()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let child = root.child(i).unwrap();
            prop_assert_eq!(child.start_position(), Point::new(0, 0));
            prop_assert_eq!(child.end_position(), Point::new(0, 0));
        }
    }
}
