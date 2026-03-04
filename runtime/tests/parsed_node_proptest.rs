#![allow(clippy::needless_range_loop)]

//! Property-based tests for `ParsedNode` in the adze runtime.
//!
//! Uses proptest to verify invariants of `ParsedNode` construction,
//! accessors, cloning, comparison, and tree structure over randomly
//! generated nodes and trees.

use adze::pure_parser::{ParsedNode, Point};
use proptest::prelude::*;
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

#[allow(clippy::too_many_arguments)]
fn make_node(
    symbol: u16,
    children: Vec<ParsedNode>,
    start: usize,
    end: usize,
    start_pt: Point,
    end_pt: Point,
    is_extra: bool,
    is_error: bool,
    is_missing: bool,
    is_named: bool,
    field_id: Option<u16>,
) -> ParsedNode {
    let mut uninit = MaybeUninit::<ParsedNode>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1);
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end);
        std::ptr::addr_of_mut!((*ptr).start_point).write(start_pt);
        std::ptr::addr_of_mut!((*ptr).end_point).write(end_pt);
        std::ptr::addr_of_mut!((*ptr).is_extra).write(is_extra);
        std::ptr::addr_of_mut!((*ptr).is_error).write(is_error);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(is_missing);
        std::ptr::addr_of_mut!((*ptr).is_named).write(is_named);
        std::ptr::addr_of_mut!((*ptr).field_id).write(field_id);
        uninit.assume_init()
    }
}

fn leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol,
        vec![],
        start,
        end,
        pt(0, start as u32),
        pt(0, end as u32),
        false,
        false,
        false,
        true,
        None,
    )
}

fn leaf_unnamed(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol,
        vec![],
        start,
        end,
        pt(0, start as u32),
        pt(0, end as u32),
        false,
        false,
        false,
        false,
        None,
    )
}

/// Strategy: generate a leaf ParsedNode with random properties.
fn arb_leaf() -> impl Strategy<Value = ParsedNode> {
    (
        0u16..100,
        0usize..1000,
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        proptest::option::of(0u16..50),
    )
        .prop_map(
            |(symbol, start, is_extra, is_error, is_missing, is_named, field_id)| {
                let end = start + 1;
                make_node(
                    symbol,
                    vec![],
                    start,
                    end,
                    pt(0, start as u32),
                    pt(0, end as u32),
                    is_extra,
                    is_error,
                    is_missing,
                    is_named,
                    field_id,
                )
            },
        )
}

/// Strategy: generate a parent node with 1-5 leaf children.
fn arb_parent() -> impl Strategy<Value = ParsedNode> {
    (0u16..100, proptest::collection::vec(arb_leaf(), 1..=5)).prop_map(|(symbol, children)| {
        let start = children.first().map_or(0, |c| c.start_byte);
        let end = children.last().map_or(0, |c| c.end_byte);
        make_node(
            symbol,
            children,
            start,
            end,
            pt(0, start as u32),
            pt(0, end as u32),
            false,
            false,
            false,
            true,
            None,
        )
    })
}

// ---------------------------------------------------------------------------
// Tests: kind
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// kind() returns a fallback string when no language is set.
    #[test]
    fn kind_fallback_for_known_symbols(sym in 0u16..11) {
        let node = leaf(sym, 0, 1);
        let kind = node.kind();
        // Known fallback symbols 0..=10 should not be "unknown"
        prop_assert!(!kind.is_empty());
    }

    /// kind() returns "unknown" for symbol ids outside the fallback table.
    #[test]
    fn kind_unknown_for_large_symbols(sym in 11u16..1000) {
        let node = leaf(sym, 0, 1);
        prop_assert_eq!(node.kind(), "unknown");
    }
}

// ---------------------------------------------------------------------------
// Tests: children count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// child_count matches the number of children in the Vec.
    #[test]
    fn child_count_matches_vec_len(n in 0usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.child_count(), n);
    }

    /// children() slice length equals child_count().
    #[test]
    fn children_slice_len_eq_child_count(n in 0usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.children().len(), node.child_count());
    }

    /// child(i) returns Some for valid indices.
    #[test]
    fn child_valid_index_returns_some(n in 1usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        for i in 0..n {
            prop_assert!(node.child(i).is_some());
        }
    }

    /// child(n) returns None for out-of-bounds index.
    #[test]
    fn child_oob_returns_none(n in 0usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        prop_assert!(node.child(n).is_none());
        prop_assert!(node.child(n + 100).is_none());
    }
}

// ---------------------------------------------------------------------------
// Tests: byte range
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// start_byte <= end_byte when constructed that way.
    #[test]
    fn byte_range_start_le_end(start in 0usize..1000, len in 0usize..500) {
        let end = start + len;
        let node = leaf(1, start, end);
        prop_assert!(node.start_byte() <= node.end_byte());
    }

    /// Byte range accessors return the values passed to the constructor.
    #[test]
    fn byte_range_roundtrip(start in 0usize..1000, len in 1usize..500) {
        let end = start + len;
        let node = leaf(1, start, end);
        prop_assert_eq!(node.start_byte(), start);
        prop_assert_eq!(node.end_byte(), end);
    }

    /// utf8_text returns the correct substring.
    #[test]
    fn utf8_text_correct_slice(start in 0usize..50, len in 1usize..50) {
        let end = start + len;
        let source = "a".repeat(end + 10);
        let node = leaf(1, start, end);
        let text = node.utf8_text(source.as_bytes()).unwrap();
        prop_assert_eq!(text.len(), len);
    }

    /// utf8_text returns error for out-of-bounds range.
    #[test]
    fn utf8_text_oob(start in 0usize..10, len in 1usize..10) {
        let end = start + len;
        // Source too short
        let source = "x".repeat(start);
        let node = leaf(1, start, end);
        prop_assert!(node.utf8_text(source.as_bytes()).is_err());
    }
}

// ---------------------------------------------------------------------------
// Tests: named vs unnamed
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// is_named accessor returns the value from construction.
    #[test]
    fn is_named_reflects_construction(named in any::<bool>()) {
        let node = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, named, None,
        );
        prop_assert_eq!(node.is_named(), named);
    }

    /// Named leaf created via helper is named.
    #[test]
    fn leaf_helper_is_named(sym in 0u16..50) {
        let node = leaf(sym, 0, 1);
        prop_assert!(node.is_named());
    }

    /// Unnamed leaf created via helper is not named.
    #[test]
    fn leaf_unnamed_helper_not_named(sym in 0u16..50) {
        let node = leaf_unnamed(sym, 0, 1);
        prop_assert!(!node.is_named());
    }
}

// ---------------------------------------------------------------------------
// Tests: field access
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// field_id roundtrips through construction.
    #[test]
    fn field_id_roundtrip(fid in proptest::option::of(0u16..100)) {
        let node = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, fid,
        );
        prop_assert_eq!(node.field_id, fid);
    }

    /// Children preserve their field_ids.
    #[test]
    fn children_preserve_field_ids(
        fids in proptest::collection::vec(proptest::option::of(0u16..50), 1..=5)
    ) {
        let children: Vec<ParsedNode> = fids.iter().enumerate().map(|(i, &fid)| {
            make_node(
                1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, false, false, true, fid,
            )
        }).collect();
        let node = make_node(
            0, children, 0, fids.len(),
            pt(0, 0), pt(0, fids.len() as u32),
            false, false, false, true, None,
        );
        for i in 0..fids.len() {
            prop_assert_eq!(node.child(i).unwrap().field_id, fids[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: various tree shapes
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Leaf node has zero children.
    #[test]
    fn leaf_has_no_children(sym in 0u16..100, start in 0usize..500) {
        let node = leaf(sym, start, start + 1);
        prop_assert_eq!(node.child_count(), 0);
        prop_assert!(node.children().is_empty());
    }

    /// A deep linear chain preserves depth and symbols.
    #[test]
    fn deep_linear_chain(depth in 1usize..10) {
        let mut node = leaf(0, 0, 1);
        for d in 1..=depth {
            node = make_node(
                d as u16, vec![node], 0, 1,
                pt(0, 0), pt(0, 1),
                false, false, false, true, None,
            );
        }
        // Walk down the chain
        let mut current = &node;
        for d in (1..=depth).rev() {
            prop_assert_eq!(current.symbol(), d as u16);
            prop_assert_eq!(current.child_count(), 1);
            current = current.child(0).unwrap();
        }
        prop_assert_eq!(current.symbol(), 0);
        prop_assert_eq!(current.child_count(), 0);
    }

    /// A wide node has the correct child count.
    #[test]
    fn wide_node_child_count(width in 0usize..20) {
        let children: Vec<ParsedNode> = (0..width).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, width,
            pt(0, 0), pt(0, width as u32),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.child_count(), width);
    }

    /// Binary tree structure: each internal node has exactly 2 children.
    #[test]
    fn binary_tree_structure(sym in 0u16..50) {
        let left = leaf(sym, 0, 2);
        let right = leaf(sym + 1, 2, 4);
        let parent = make_node(
            sym + 2, vec![left, right], 0, 4,
            pt(0, 0), pt(0, 4),
            false, false, false, true, None,
        );
        prop_assert_eq!(parent.child_count(), 2);
        prop_assert_eq!(parent.child(0).unwrap().symbol(), sym);
        prop_assert_eq!(parent.child(1).unwrap().symbol(), sym + 1);
    }

    /// Mixed named/unnamed children.
    #[test]
    fn mixed_named_unnamed_children(n in 1usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| {
            if i % 2 == 0 {
                leaf(i as u16, i, i + 1)
            } else {
                leaf_unnamed(i as u16, i, i + 1)
            }
        }).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        for i in 0..n {
            let child = node.child(i).unwrap();
            prop_assert_eq!(child.is_named(), i % 2 == 0);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: comparison (field-by-field, since no PartialEq)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Two nodes built with identical arguments have matching accessors.
    #[test]
    fn identical_construction_same_accessors(
        sym in 0u16..100,
        start in 0usize..500,
        len in 1usize..100,
        is_named in any::<bool>(),
        is_extra in any::<bool>(),
    ) {
        let end = start + len;
        let a = make_node(
            sym, vec![], start, end,
            pt(0, start as u32), pt(0, end as u32),
            is_extra, false, false, is_named, None,
        );
        let b = make_node(
            sym, vec![], start, end,
            pt(0, start as u32), pt(0, end as u32),
            is_extra, false, false, is_named, None,
        );
        prop_assert_eq!(a.symbol(), b.symbol());
        prop_assert_eq!(a.start_byte(), b.start_byte());
        prop_assert_eq!(a.end_byte(), b.end_byte());
        prop_assert_eq!(a.is_named(), b.is_named());
        prop_assert_eq!(a.is_extra(), b.is_extra());
        prop_assert_eq!(a.child_count(), b.child_count());
        prop_assert_eq!(a.kind(), b.kind());
    }

    /// Nodes with different symbols have different symbol().
    #[test]
    fn different_symbols_differ(s1 in 11u16..100, s2 in 11u16..100) {
        prop_assume!(s1 != s2);
        let a = leaf(s1, 0, 1);
        let b = leaf(s2, 0, 1);
        prop_assert_ne!(a.symbol(), b.symbol());
    }

    /// Nodes with different byte ranges differ in start/end.
    #[test]
    fn different_ranges_differ(
        start1 in 0usize..500,
        start2 in 0usize..500,
    ) {
        prop_assume!(start1 != start2);
        let a = leaf(1, start1, start1 + 1);
        let b = leaf(1, start2, start2 + 1);
        prop_assert_ne!(a.start_byte(), b.start_byte());
    }
}

// ---------------------------------------------------------------------------
// Tests: clone
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Cloning a leaf preserves all accessor values.
    #[test]
    fn clone_leaf_preserves_values(
        sym in 0u16..100,
        start in 0usize..500,
        len in 1usize..100,
        is_named in any::<bool>(),
        fid in proptest::option::of(0u16..50),
    ) {
        let end = start + len;
        let node = make_node(
            sym, vec![], start, end,
            pt(0, start as u32), pt(0, end as u32),
            false, false, false, is_named, fid,
        );
        let cloned = node.clone();
        prop_assert_eq!(cloned.symbol(), node.symbol());
        prop_assert_eq!(cloned.start_byte(), node.start_byte());
        prop_assert_eq!(cloned.end_byte(), node.end_byte());
        prop_assert_eq!(cloned.start_point(), node.start_point());
        prop_assert_eq!(cloned.end_point(), node.end_point());
        prop_assert_eq!(cloned.is_named(), node.is_named());
        prop_assert_eq!(cloned.is_extra(), node.is_extra());
        prop_assert_eq!(cloned.is_error(), node.is_error());
        prop_assert_eq!(cloned.is_missing(), node.is_missing());
        prop_assert_eq!(cloned.child_count(), node.child_count());
        prop_assert_eq!(cloned.field_id, node.field_id);
    }

    /// Cloning a parent preserves children.
    #[test]
    fn clone_parent_preserves_children(n in 1usize..6) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(i as u16, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        let cloned = node.clone();
        prop_assert_eq!(cloned.child_count(), node.child_count());
        for i in 0..n {
            let orig = node.child(i).unwrap();
            let copy = cloned.child(i).unwrap();
            prop_assert_eq!(orig.symbol(), copy.symbol());
            prop_assert_eq!(orig.start_byte(), copy.start_byte());
            prop_assert_eq!(orig.end_byte(), copy.end_byte());
        }
    }

    /// Clone is independent: mutating original children doesn't affect clone.
    #[test]
    fn clone_is_independent(sym in 0u16..50) {
        let mut node = make_node(
            sym, vec![leaf(1, 0, 1)], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        let cloned = node.clone();
        // Mutate the original
        node.children.push(leaf(2, 1, 2));
        prop_assert_eq!(cloned.child_count(), 1);
        prop_assert_eq!(node.child_count(), 2);
    }

    /// Deep clone preserves nested structure.
    #[test]
    fn deep_clone_preserves_nesting(depth in 1usize..6) {
        let mut node = leaf(0, 0, 1);
        for d in 1..=depth {
            node = make_node(
                d as u16, vec![node], 0, 1,
                pt(0, 0), pt(0, 1),
                false, false, false, true, None,
            );
        }
        let cloned = node.clone();
        let mut orig = &node;
        let mut copy = &cloned;
        for _ in 0..depth {
            prop_assert_eq!(orig.symbol(), copy.symbol());
            prop_assert_eq!(orig.child_count(), copy.child_count());
            orig = orig.child(0).unwrap();
            copy = copy.child(0).unwrap();
        }
        prop_assert_eq!(orig.symbol(), copy.symbol());
    }
}

// ---------------------------------------------------------------------------
// Tests: additional properties (points, flags, walker, symbol)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// start_point and end_point roundtrip.
    #[test]
    fn points_roundtrip(row in 0u32..1000, col in 0u32..200, len in 1u32..100) {
        let node = make_node(
            1, vec![], col as usize, (col + len) as usize,
            pt(row, col), pt(row, col + len),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.start_point(), pt(row, col));
        prop_assert_eq!(node.end_point(), pt(row, col + len));
    }

    /// is_error, is_extra, is_missing roundtrip.
    #[test]
    fn error_flags_roundtrip(
        is_extra in any::<bool>(),
        is_error in any::<bool>(),
        is_missing in any::<bool>(),
    ) {
        let node = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            is_extra, is_error, is_missing, true, None,
        );
        prop_assert_eq!(node.is_extra(), is_extra);
        prop_assert_eq!(node.is_error(), is_error);
        prop_assert_eq!(node.is_missing(), is_missing);
    }

    /// has_error is true when node itself is an error.
    #[test]
    fn has_error_when_self_is_error(is_err in any::<bool>()) {
        let node = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, is_err, false, true, None,
        );
        if is_err {
            prop_assert!(node.has_error());
        }
    }

    /// has_error propagates from children.
    #[test]
    fn has_error_propagates_from_child(child_err in any::<bool>()) {
        let child = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, child_err, false, true, None,
        );
        let parent = make_node(
            0, vec![child], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        prop_assert_eq!(parent.has_error(), child_err);
    }

    /// walk() goto_first_child returns true iff children exist.
    #[test]
    fn walker_first_child_iff_children(n in 0usize..5) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n.max(1),
            pt(0, 0), pt(0, n.max(1) as u32),
            false, false, false, true, None,
        );
        let mut walker = node.walk();
        prop_assert_eq!(walker.goto_first_child(), n > 0);
    }

    /// walk() traverses all children via goto_next_sibling.
    #[test]
    fn walker_visits_all_children(n in 1usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(i as u16, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        let mut walker = node.walk();
        prop_assert!(walker.goto_first_child());
        let mut visited = 1;
        while walker.goto_next_sibling() {
            visited += 1;
        }
        prop_assert_eq!(visited, n);
    }

    /// symbol() accessor returns the value set at construction.
    #[test]
    fn symbol_roundtrip(sym in 0u16..1000) {
        let node = leaf(sym, 0, 1);
        prop_assert_eq!(node.symbol(), sym);
    }

    /// arb_parent strategy produces valid parent nodes.
    #[test]
    fn arb_parent_has_children(node in arb_parent()) {
        prop_assert!(node.child_count() >= 1);
        prop_assert!(node.child_count() <= 5);
    }
}
