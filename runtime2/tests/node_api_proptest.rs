#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::{Point, Tree};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Build a flat tree with `n` leaf children spanning `[start, end)`.
fn make_tree(symbol: u32, start: usize, end: usize, n: usize) -> Tree {
    if n == 0 || end <= start {
        return Tree::new_for_testing(symbol, start, end, vec![]);
    }
    let span = end - start;
    let children: Vec<Tree> = (0..n)
        .map(|i| {
            let cs = start + (span * i) / n;
            let ce = start + (span * (i + 1)) / n;
            Tree::new_for_testing(i as u32 + 100, cs, ce, vec![])
        })
        .collect();
    Tree::new_for_testing(symbol, start, end, children)
}

/// Arbitrary tree (may have zero-length range or zero children).
fn any_tree() -> impl Strategy<Value = Tree> {
    (0u32..1000, 0usize..8000, 0usize..8000, 0usize..10).prop_map(|(sym, a, b, n)| {
        let (s, e) = if a <= b { (a, b) } else { (b, a) };
        make_tree(sym, s, e, n)
    })
}

/// Tree guaranteed to have at least one child and a non-empty byte range.
fn tree_with_kids() -> impl Strategy<Value = Tree> {
    (0u32..1000, 0usize..4000, 1usize..4000, 1usize..10)
        .prop_map(|(sym, start, span, n)| make_tree(sym, start, start + span, n))
}

/// Build a two-level tree: root -> children -> grandchildren.
fn two_level_tree(sym: u32, start: usize, end: usize, n_children: usize, n_grand: usize) -> Tree {
    if n_children == 0 || end <= start {
        return Tree::new_for_testing(sym, start, end, vec![]);
    }
    let span = end - start;
    let children: Vec<Tree> = (0..n_children)
        .map(|i| {
            let cs = start + (span * i) / n_children;
            let ce = start + (span * (i + 1)) / n_children;
            make_tree(i as u32 + 10, cs, ce, n_grand)
        })
        .collect();
    Tree::new_for_testing(sym, start, end, children)
}

fn nested_tree() -> impl Strategy<Value = Tree> {
    (0u32..500, 0usize..2000, 1usize..2000, 1usize..6, 0usize..4)
        .prop_map(|(sym, start, span, nc, ng)| two_level_tree(sym, start, start + span, nc, ng))
}

// ===========================================================================
// 1 – kind() returns "unknown" for any symbol when no language is set
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn kind_without_language_is_unknown(sym in 0u32..100_000) {
        let tree = Tree::new_for_testing(sym, 0, 1, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.kind(), "unknown");
    }
}

// ===========================================================================
// 2 – kind_id() round-trips the symbol used at construction
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn kind_id_roundtrips_symbol(sym in 0u32..65536) {
        let tree = Tree::new_for_testing(sym, 0, 5, vec![]);
        prop_assert_eq!(tree.root_node().kind_id(), sym as u16);
    }
}

// ===========================================================================
// 3 – start_byte() equals the value passed at construction
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn start_byte_matches_construction(start in 0usize..50_000) {
        let end = start + 10;
        let tree = Tree::new_for_testing(0, start, end, vec![]);
        prop_assert_eq!(tree.root_node().start_byte(), start);
    }
}

// ===========================================================================
// 4 – end_byte() equals the value passed at construction
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn end_byte_matches_construction(start in 0usize..50_000, span in 0usize..10_000) {
        let end = start + span;
        let tree = Tree::new_for_testing(0, start, end, vec![]);
        prop_assert_eq!(tree.root_node().end_byte(), end);
    }
}

// ===========================================================================
// 5 – byte_range() is consistent with start_byte/end_byte
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn byte_range_accessors_agree(tree in any_tree()) {
        let n = tree.root_node();
        let r = n.byte_range();
        prop_assert_eq!(r.start, n.start_byte());
        prop_assert_eq!(r.end, n.end_byte());
    }
}

// ===========================================================================
// 6 – start_byte <= end_byte always holds
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn start_le_end(tree in any_tree()) {
        let n = tree.root_node();
        prop_assert!(n.start_byte() <= n.end_byte());
    }
}

// ===========================================================================
// 7 – is_named() always true in Phase 1
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn is_named_invariant(tree in any_tree()) {
        prop_assert!(tree.root_node().is_named());
    }
}

// ===========================================================================
// 8 – is_named is true for every child node as well
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn children_are_named(tree in tree_with_kids()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            prop_assert!(root.child(i).unwrap().is_named());
        }
    }
}

// ===========================================================================
// 9 – child_count() for a leaf node is zero
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn leaf_has_zero_children(sym in 0u32..500, start in 0usize..5000, span in 0usize..5000) {
        let tree = Tree::new_for_testing(sym, start, start + span, vec![]);
        prop_assert_eq!(tree.root_node().child_count(), 0);
    }
}

// ===========================================================================
// 10 – child_count() matches the number of children supplied
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn child_count_matches_input(n in 0usize..15) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(i as u32 + 1, i * 5, (i + 1) * 5, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n * 5, children);
        prop_assert_eq!(tree.root_node().child_count(), n);
    }
}

// ===========================================================================
// 11 – child(i) returns Some for every valid index
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn child_some_for_valid(tree in tree_with_kids()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            prop_assert!(root.child(i).is_some());
        }
    }
}

// ===========================================================================
// 12 – child(child_count()) returns None (one past the end)
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn child_none_at_count(tree in any_tree()) {
        let root = tree.root_node();
        prop_assert!(root.child(root.child_count()).is_none());
    }
}

// ===========================================================================
// 13 – child(i) for large i returns None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn child_none_far_oob(tree in any_tree(), extra in 1usize..500) {
        let root = tree.root_node();
        prop_assert!(root.child(root.child_count() + extra).is_none());
    }
}

// ===========================================================================
// 14 – child byte ranges sit within parent range
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn child_ranges_inside_parent(tree in tree_with_kids()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let c = root.child(i).unwrap();
            prop_assert!(c.start_byte() >= root.start_byte());
            prop_assert!(c.end_byte() <= root.end_byte());
        }
    }
}

// ===========================================================================
// 15 – siblings are ordered: child(i).end_byte <= child(i+1).start_byte
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn children_non_overlapping(tree in tree_with_kids()) {
        let root = tree.root_node();
        for i in 1..root.child_count() {
            let prev = root.child(i - 1).unwrap();
            let curr = root.child(i).unwrap();
            prop_assert!(prev.end_byte() <= curr.start_byte());
        }
    }
}

// ===========================================================================
// 16 – no children: child(0) is None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn empty_children_child0_none(sym in 0u32..500) {
        let tree = Tree::new_for_testing(sym, 0, 10, vec![]);
        prop_assert!(tree.root_node().child(0).is_none());
    }
}

// ===========================================================================
// 17 – many children: tree with up to 50 children is well-formed
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn many_children_well_formed(n in 1usize..50) {
        let total = n * 10;
        let tree = make_tree(0, 0, total, n);
        let root = tree.root_node();
        prop_assert_eq!(root.child_count(), n);
        prop_assert_eq!(root.start_byte(), 0);
        prop_assert_eq!(root.end_byte(), total);
        // First child starts at root start
        prop_assert_eq!(root.child(0).unwrap().start_byte(), 0);
        // Last child ends at root end
        prop_assert_eq!(root.child(n - 1).unwrap().end_byte(), total);
    }
}

// ===========================================================================
// 18 – parent() returns None (links not stored)
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parent_always_none(tree in any_tree()) {
        prop_assert!(tree.root_node().parent().is_none());
    }
}

// ===========================================================================
// 19 – next_sibling() returns None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn next_sibling_none(tree in any_tree()) {
        prop_assert!(tree.root_node().next_sibling().is_none());
    }
}

// ===========================================================================
// 20 – prev_sibling() returns None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prev_sibling_none(tree in any_tree()) {
        prop_assert!(tree.root_node().prev_sibling().is_none());
    }
}

// ===========================================================================
// 21 – next_named_sibling() / prev_named_sibling() return None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn named_siblings_none(tree in any_tree()) {
        let n = tree.root_node();
        prop_assert!(n.next_named_sibling().is_none());
        prop_assert!(n.prev_named_sibling().is_none());
    }
}

// ===========================================================================
// 22 – parent/sibling of child nodes also returns None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn child_nav_returns_none(tree in tree_with_kids()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let c = root.child(i).unwrap();
            prop_assert!(c.parent().is_none());
            prop_assert!(c.next_sibling().is_none());
            prop_assert!(c.prev_sibling().is_none());
        }
    }
}

// ===========================================================================
// 23 – named_child_count equals child_count (Phase 1)
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn named_child_count_eq_child_count(tree in any_tree()) {
        let root = tree.root_node();
        prop_assert_eq!(root.named_child_count(), root.child_count());
    }
}

// ===========================================================================
// 24 – named_child(i) matches child(i) for every valid i
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn named_child_eq_child(tree in tree_with_kids()) {
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
// 25 – child_by_field_name always returns None
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn field_name_lookup_none(tree in any_tree(), name in "[a-z_]{1,15}") {
        prop_assert!(tree.root_node().child_by_field_name(&name).is_none());
    }
}

// ===========================================================================
// 26 – is_missing / is_error both false for children
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn children_not_missing_not_error(tree in tree_with_kids()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let c = root.child(i).unwrap();
            prop_assert!(!c.is_missing());
            prop_assert!(!c.is_error());
        }
    }
}

// ===========================================================================
// 27 – grandchildren are reachable and have valid ranges
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn grandchildren_valid_ranges(tree in nested_tree()) {
        let root = tree.root_node();
        for i in 0..root.child_count() {
            let child = root.child(i).unwrap();
            for j in 0..child.child_count() {
                let gc = child.child(j).unwrap();
                prop_assert!(gc.start_byte() <= gc.end_byte());
                prop_assert!(gc.start_byte() >= child.start_byte());
                prop_assert!(gc.end_byte() <= child.end_byte());
            }
        }
    }
}

// ===========================================================================
// 28 – start_position / end_position return origin (Phase 1 stub)
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn positions_are_origin(tree in any_tree()) {
        let n = tree.root_node();
        prop_assert_eq!(n.start_position(), Point::new(0, 0));
        prop_assert_eq!(n.end_position(), Point::new(0, 0));
    }
}

// ===========================================================================
// 29 – utf8_text round-trips source for root
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn utf8_text_roundtrip(s in "[a-zA-Z0-9 ]{1,200}") {
        let bytes = s.as_bytes();
        let tree = Tree::new_for_testing(0, 0, bytes.len(), vec![]);
        let text = tree.root_node().utf8_text(bytes).unwrap();
        prop_assert_eq!(text, s.as_str());
    }
}

// ===========================================================================
// 30 – utf8_text on a child extracts the correct sub-slice
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn child_utf8_text_subslice(s in "[a-z]{10,100}") {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let tree = make_tree(0, 0, len, 3);
        let root = tree.root_node();

        let mut concat = String::new();
        for i in 0..root.child_count() {
            let c = root.child(i).unwrap();
            concat.push_str(c.utf8_text(bytes).unwrap());
        }
        prop_assert_eq!(concat, s);
    }
}

// ===========================================================================
// 31 – Debug output contains "Node" and byte range info
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn debug_contains_node_and_range(tree in any_tree()) {
        let dbg = format!("{:?}", tree.root_node());
        prop_assert!(dbg.contains("Node"));
        prop_assert!(dbg.contains("range"));
    }
}

// ===========================================================================
// 32 – Node is Copy: copied node has identical properties
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn node_copy_identical(tree in any_tree()) {
        let a = tree.root_node();
        let b = a; // Copy
        prop_assert_eq!(a.kind_id(), b.kind_id());
        prop_assert_eq!(a.start_byte(), b.start_byte());
        prop_assert_eq!(a.end_byte(), b.end_byte());
        prop_assert_eq!(a.child_count(), b.child_count());
        prop_assert_eq!(a.is_named(), b.is_named());
    }
}

// ===========================================================================
// 33 – Tree::clone preserves all root node properties
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn clone_preserves_root(tree in any_tree()) {
        let cloned = tree.clone();
        let a = tree.root_node();
        let b = cloned.root_node();
        prop_assert_eq!(a.kind_id(), b.kind_id());
        prop_assert_eq!(a.start_byte(), b.start_byte());
        prop_assert_eq!(a.end_byte(), b.end_byte());
        prop_assert_eq!(a.child_count(), b.child_count());
    }
}

// ===========================================================================
// 34 – Tree::clone preserves children recursively
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn clone_preserves_children(tree in tree_with_kids()) {
        let cloned = tree.clone();
        let r1 = tree.root_node();
        let r2 = cloned.root_node();
        prop_assert_eq!(r1.child_count(), r2.child_count());
        for i in 0..r1.child_count() {
            let c1 = r1.child(i).unwrap();
            let c2 = r2.child(i).unwrap();
            prop_assert_eq!(c1.kind_id(), c2.kind_id());
            prop_assert_eq!(c1.start_byte(), c2.start_byte());
            prop_assert_eq!(c1.end_byte(), c2.end_byte());
            prop_assert_eq!(c1.child_count(), c2.child_count());
        }
    }
}

// ===========================================================================
// 35 – root_kind() agrees with root_node().kind_id() as u32
// ===========================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn root_kind_consistent(sym in 0u32..65536) {
        let tree = Tree::new_for_testing(sym, 0, 10, vec![]);
        prop_assert_eq!(tree.root_kind(), sym);
        prop_assert_eq!(tree.root_node().kind_id() as u32, sym);
    }
}
