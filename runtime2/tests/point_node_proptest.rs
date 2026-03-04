#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::{Point, Tree};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_point() -> impl Strategy<Value = Point> {
    (any::<usize>(), any::<usize>()).prop_map(|(row, column)| Point::new(row, column))
}

fn arb_small_point() -> impl Strategy<Value = Point> {
    (0usize..10_000, 0usize..10_000).prop_map(|(row, column)| Point::new(row, column))
}

// ---------------------------------------------------------------------------
// 1 – Point construction and field access
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn point_new_roundtrip(row in any::<usize>(), col in any::<usize>()) {
        let p = Point::new(row, col);
        prop_assert_eq!(p.row, row);
        prop_assert_eq!(p.column, col);
    }

    #[test]
    fn point_zero_is_origin(_ in 0..1u8) {
        let p = Point::new(0, 0);
        prop_assert_eq!(p.row, 0);
        prop_assert_eq!(p.column, 0);
    }
}

// ---------------------------------------------------------------------------
// 2 – Point Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn point_copy_is_independent(row in any::<usize>(), col in any::<usize>()) {
        let p1 = Point::new(row, col);
        let p2 = p1; // Copy
        prop_assert_eq!(p1, p2);
        prop_assert_eq!(p1.row, p2.row);
        prop_assert_eq!(p1.column, p2.column);
    }

    #[test]
    fn point_clone_equals_original(row in any::<usize>(), col in any::<usize>()) {
        let p = Point::new(row, col);
        #[allow(clippy::clone_on_copy)]
        let cloned = p.clone();
        prop_assert_eq!(p, cloned);
    }
}

// ---------------------------------------------------------------------------
// 3 – Point PartialEq / Eq
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn point_eq_reflexive(row in any::<usize>(), col in any::<usize>()) {
        let p = Point::new(row, col);
        prop_assert_eq!(p, p);
    }

    #[test]
    fn point_eq_symmetric(r1 in any::<usize>(), c1 in any::<usize>(),
                          r2 in any::<usize>(), c2 in any::<usize>()) {
        let a = Point::new(r1, c1);
        let b = Point::new(r2, c2);
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn point_eq_same_fields(row in any::<usize>(), col in any::<usize>()) {
        let a = Point::new(row, col);
        let b = Point::new(row, col);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn point_ne_different_row(row in any::<usize>(), col in any::<usize>()) {
        let a = Point::new(row, col);
        let b = Point::new(row.wrapping_add(1), col);
        if row != usize::MAX {
            prop_assert_ne!(a, b);
        }
    }

    #[test]
    fn point_ne_different_column(row in any::<usize>(), col in any::<usize>()) {
        let a = Point::new(row, col);
        let b = Point::new(row, col.wrapping_add(1));
        if col != usize::MAX {
            prop_assert_ne!(a, b);
        }
    }
}

// ---------------------------------------------------------------------------
// 4 – Point Ord / PartialOrd
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn point_ord_reflexive(row in any::<usize>(), col in any::<usize>()) {
        let p = Point::new(row, col);
        prop_assert!(p <= p);
        prop_assert!(p >= p);
    }

    #[test]
    fn point_ord_row_dominant(r1 in 0usize..1000, r2 in 0usize..1000,
                              c1 in any::<usize>(), c2 in any::<usize>()) {
        let a = Point::new(r1, c1);
        let b = Point::new(r2, c2);
        if r1 < r2 {
            prop_assert!(a < b, "row-dominant: {:?} should be < {:?}", a, b);
        } else if r1 > r2 {
            prop_assert!(a > b, "row-dominant: {:?} should be > {:?}", a, b);
        }
    }

    #[test]
    fn point_ord_column_tiebreak(row in any::<usize>(),
                                 c1 in 0usize..1000, c2 in 0usize..1000) {
        let a = Point::new(row, c1);
        let b = Point::new(row, c2);
        prop_assert_eq!(a.cmp(&b), c1.cmp(&c2));
    }

    #[test]
    fn point_ord_antisymmetric(p1 in arb_point(), p2 in arb_point()) {
        if p1 < p2 {
            prop_assert!(p2 > p1);
        } else if p1 > p2 {
            prop_assert!(p2 < p1);
        } else {
            prop_assert_eq!(p1, p2);
        }
    }

    #[test]
    fn point_ord_transitive(a in arb_small_point(), b in arb_small_point(), c in arb_small_point()) {
        if a <= b && b <= c {
            prop_assert!(a <= c);
        }
        if a >= b && b >= c {
            prop_assert!(a >= c);
        }
    }
}

// ---------------------------------------------------------------------------
// 5 – Point Debug / Display
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn point_debug_contains_fields(row in 0usize..10_000, col in 0usize..10_000) {
        let p = Point::new(row, col);
        let dbg = format!("{:?}", p);
        prop_assert!(dbg.contains(&row.to_string()));
        prop_assert!(dbg.contains(&col.to_string()));
    }

    #[test]
    fn point_display_one_indexed(row in 0usize..10_000, col in 0usize..10_000) {
        let p = Point::new(row, col);
        let display = format!("{}", p);
        let expected = format!("{}:{}", row + 1, col + 1);
        prop_assert_eq!(display, expected);
    }
}

// ---------------------------------------------------------------------------
// 6 – Node from stub tree: basic properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn stub_node_kind_is_unknown(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert_eq!(root.kind(), "unknown");
    }

    #[test]
    fn stub_node_kind_id_is_zero(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert_eq!(root.kind_id(), 0);
    }

    #[test]
    fn stub_node_byte_range_is_empty(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert_eq!(root.start_byte(), 0);
        prop_assert_eq!(root.end_byte(), 0);
        prop_assert_eq!(root.byte_range(), 0..0);
    }

    #[test]
    fn stub_node_positions_are_origin(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert_eq!(root.start_position(), Point::new(0, 0));
        prop_assert_eq!(root.end_position(), Point::new(0, 0));
    }

    #[test]
    fn stub_node_flags(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert!(root.is_named());
        prop_assert!(!root.is_missing());
        prop_assert!(!root.is_error());
    }
}

// ---------------------------------------------------------------------------
// 7 – Node child access
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn stub_node_has_no_children(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert_eq!(root.child_count(), 0);
        prop_assert_eq!(root.named_child_count(), 0);
    }

    #[test]
    fn stub_node_child_out_of_bounds(idx in 0usize..100) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert!(root.child(idx).is_none());
        prop_assert!(root.named_child(idx).is_none());
    }
}

// ---------------------------------------------------------------------------
// 8 – Node navigation stubs
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn stub_node_parent_is_none(_ in 0..1u8) {
        let tree = Tree::new_stub();
        prop_assert!(tree.root_node().parent().is_none());
    }

    #[test]
    fn stub_node_siblings_are_none(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert!(root.next_sibling().is_none());
        prop_assert!(root.prev_sibling().is_none());
        prop_assert!(root.next_named_sibling().is_none());
        prop_assert!(root.prev_named_sibling().is_none());
    }

    #[test]
    fn stub_node_child_by_field_name_is_none(name in "[a-z]{1,20}") {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert!(root.child_by_field_name(&name).is_none());
    }
}

// ---------------------------------------------------------------------------
// 9 – Node Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn node_copy_preserves_identity(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let n1 = tree.root_node();
        let n2 = n1; // Copy
        prop_assert_eq!(n1.kind(), n2.kind());
        prop_assert_eq!(n1.kind_id(), n2.kind_id());
        prop_assert_eq!(n1.start_byte(), n2.start_byte());
        prop_assert_eq!(n1.end_byte(), n2.end_byte());
        prop_assert_eq!(n1.child_count(), n2.child_count());
    }
}

// ---------------------------------------------------------------------------
// 10 – Node Debug format
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn node_debug_is_non_empty(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        let dbg = format!("{:?}", root);
        prop_assert!(!dbg.is_empty());
        prop_assert!(dbg.contains("Node"));
    }
}

// ---------------------------------------------------------------------------
// 11 – Node utf8_text
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn stub_node_utf8_text_empty_source(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        let text = root.utf8_text(b"").unwrap();
        prop_assert_eq!(text, "");
    }

    #[test]
    fn stub_node_utf8_text_with_source(s in "[a-zA-Z0-9 ]{0,64}") {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        // Stub root has byte_range 0..0, so text is always ""
        let text = root.utf8_text(s.as_bytes()).unwrap();
        prop_assert_eq!(text, "");
    }
}

// ---------------------------------------------------------------------------
// 12 – Tree clone independence
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn tree_clone_produces_equal_root(_ in 0..1u8) {
        let t1 = Tree::new_stub();
        let t2 = t1.clone();
        let r1 = t1.root_node();
        let r2 = t2.root_node();
        prop_assert_eq!(r1.kind(), r2.kind());
        prop_assert_eq!(r1.start_byte(), r2.start_byte());
        prop_assert_eq!(r1.end_byte(), r2.end_byte());
        prop_assert_eq!(r1.child_count(), r2.child_count());
    }
}

// ---------------------------------------------------------------------------
// 13 – Tree stub metadata
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn tree_stub_has_no_language(_ in 0..1u8) {
        let tree = Tree::new_stub();
        prop_assert!(tree.language().is_none());
    }

    #[test]
    fn tree_stub_has_no_source_bytes(_ in 0..1u8) {
        let tree = Tree::new_stub();
        prop_assert!(tree.source_bytes().is_none());
    }

    #[test]
    fn tree_stub_root_kind_is_zero(_ in 0..1u8) {
        let tree = Tree::new_stub();
        prop_assert_eq!(tree.root_kind(), 0);
    }
}

// ---------------------------------------------------------------------------
// 14 – Point max value boundaries
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn point_max_values(_ in 0..1u8) {
        let p = Point::new(usize::MAX, usize::MAX);
        prop_assert_eq!(p.row, usize::MAX);
        prop_assert_eq!(p.column, usize::MAX);
    }

    #[test]
    fn point_max_row_orders_last(col in any::<usize>()) {
        let low = Point::new(0, col);
        let high = Point::new(usize::MAX, 0);
        prop_assert!(low < high);
    }
}

// ---------------------------------------------------------------------------
// 15 – Point ordering consistency with PartialOrd
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn point_partial_ord_matches_ord(p1 in arb_point(), p2 in arb_point()) {
        prop_assert_eq!(p1.partial_cmp(&p2), Some(p1.cmp(&p2)));
    }
}
