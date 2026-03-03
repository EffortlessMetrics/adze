#![allow(clippy::needless_range_loop)]

//! Property-based tests for `ParsedNode` metadata in the adze runtime.
//!
//! Covers: kind/type, byte range, position, children access, is_error/is_missing
//! flags, field_id, symbol, and metadata determinism.

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

// ---------------------------------------------------------------------------
// 1. ParsedNode kind/type
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// kind() returns specific known fallback strings for symbols 0-10.
    #[test]
    fn kind_specific_fallback_values(sym in 0u16..11) {
        let expected = match sym {
            0 => "end",
            1 => "*",
            2 => "_2",
            3 => "_6",
            4 => "-",
            5 => "Expression",
            6 => "Whitespace__whitespace",
            7 => "Whitespace",
            8 => "Expression_Sub_1",
            9 => "Expression_Sub",
            10 => "rule_10",
            _ => unreachable!(),
        };
        let node = leaf(sym, 0, 1);
        prop_assert_eq!(node.kind(), expected);
    }

    /// kind() is deterministic: calling it twice yields the same result.
    #[test]
    fn kind_deterministic(sym in 0u16..200) {
        let node = leaf(sym, 0, 1);
        let k1 = node.kind();
        let k2 = node.kind();
        prop_assert_eq!(k1, k2);
    }

    /// kind() returns a non-empty string for any symbol.
    #[test]
    fn kind_never_empty(sym in 0u16..500) {
        let node = leaf(sym, 0, 1);
        prop_assert!(!node.kind().is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2. ParsedNode byte range
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Zero-length byte range (start == end) is valid.
    #[test]
    fn zero_length_byte_range(pos in 0usize..1000) {
        let node = make_node(
            1, vec![], pos, pos,
            pt(0, pos as u32), pt(0, pos as u32),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.start_byte(), pos);
        prop_assert_eq!(node.end_byte(), pos);
        prop_assert_eq!(node.end_byte() - node.start_byte(), 0);
    }

    /// utf8_text length matches end_byte - start_byte for valid ASCII.
    #[test]
    fn utf8_text_len_matches_byte_span(start in 0usize..50, len in 1usize..50) {
        let end = start + len;
        let source = "x".repeat(end + 5);
        let node = leaf(1, start, end);
        let text = node.utf8_text(source.as_bytes()).unwrap();
        prop_assert_eq!(text.len(), node.end_byte() - node.start_byte());
    }

    /// Parent byte range can encompass children byte ranges.
    #[test]
    fn parent_range_encompasses_children(n in 1usize..6) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i * 3, i * 3 + 2)).collect();
        let parent_start = 0;
        let parent_end = n * 3;
        let parent = make_node(
            0, children, parent_start, parent_end,
            pt(0, 0), pt(0, parent_end as u32),
            false, false, false, true, None,
        );
        for i in 0..n {
            let child = parent.child(i).unwrap();
            prop_assert!(child.start_byte() >= parent.start_byte());
            prop_assert!(child.end_byte() <= parent.end_byte());
        }
    }
}

// ---------------------------------------------------------------------------
// 3. ParsedNode position (start_position, end_position)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Multi-line positions: start row < end row.
    #[test]
    fn multiline_position(
        start_row in 0u32..100,
        start_col in 0u32..80,
        end_row_offset in 1u32..20,
        end_col in 0u32..80,
    ) {
        let end_row = start_row + end_row_offset;
        let node = make_node(
            1, vec![], 0, 100,
            pt(start_row, start_col), pt(end_row, end_col),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.start_point().row, start_row);
        prop_assert_eq!(node.start_point().column, start_col);
        prop_assert_eq!(node.end_point().row, end_row);
        prop_assert_eq!(node.end_point().column, end_col);
        prop_assert!(node.start_point().row < node.end_point().row);
    }

    /// Position with zero column is valid.
    #[test]
    fn position_zero_column(row in 0u32..100) {
        let node = make_node(
            1, vec![], 0, 10,
            pt(row, 0), pt(row, 10),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.start_point().column, 0);
    }

    /// Start and end point on same row maintain column ordering.
    #[test]
    fn same_row_column_ordering(row in 0u32..100, col in 0u32..100, len in 1u32..100) {
        let node = make_node(
            1, vec![], col as usize, (col + len) as usize,
            pt(row, col), pt(row, col + len),
            false, false, false, true, None,
        );
        prop_assert_eq!(node.start_point().row, node.end_point().row);
        prop_assert!(node.start_point().column < node.end_point().column);
    }
}

// ---------------------------------------------------------------------------
// 4. ParsedNode children access
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// children() slice elements match child(i) for all valid i.
    #[test]
    fn children_slice_matches_child_index(n in 1usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(i as u16, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        let slice = node.children();
        for i in 0..n {
            let by_index = node.child(i).unwrap();
            prop_assert_eq!(slice[i].symbol(), by_index.symbol());
            prop_assert_eq!(slice[i].start_byte(), by_index.start_byte());
            prop_assert_eq!(slice[i].end_byte(), by_index.end_byte());
        }
    }

    /// Grandchild access through nested child() calls.
    #[test]
    fn grandchild_access(
        parent_sym in 0u16..50,
        child_sym in 50u16..100,
        grandchild_sym in 100u16..150,
    ) {
        let grandchild = leaf(grandchild_sym, 0, 1);
        let child = make_node(
            child_sym, vec![grandchild], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        let parent = make_node(
            parent_sym, vec![child], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        let gc = parent.child(0).unwrap().child(0).unwrap();
        prop_assert_eq!(gc.symbol(), grandchild_sym);
    }

    /// Walker node() returns the correct child at each step.
    #[test]
    fn walker_node_matches_child(n in 2usize..8) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(i as u16, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        let mut walker = node.walk();
        prop_assert!(walker.goto_first_child());
        prop_assert_eq!(walker.node().symbol(), node.child(0).unwrap().symbol());
        for i in 1..n {
            prop_assert!(walker.goto_next_sibling());
            prop_assert_eq!(walker.node().symbol(), node.child(i).unwrap().symbol());
        }
    }

    /// Walker goto_next_sibling returns false when at last child.
    #[test]
    fn walker_next_sibling_false_at_end(n in 1usize..6) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        let mut walker = node.walk();
        prop_assert!(walker.goto_first_child());
        for _ in 1..n {
            prop_assert!(walker.goto_next_sibling());
        }
        // Should now be at the last child
        prop_assert!(!walker.goto_next_sibling());
    }
}

// ---------------------------------------------------------------------------
// 5. ParsedNode is_error/is_missing flags
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// has_error propagates from grandchild (deep descendant).
    #[test]
    fn has_error_propagates_from_grandchild(gc_err in any::<bool>()) {
        let grandchild = make_node(
            2, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, gc_err, false, true, None,
        );
        let child = make_node(
            1, vec![grandchild], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        let parent = make_node(
            0, vec![child], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        prop_assert_eq!(parent.has_error(), gc_err);
    }

    /// is_missing and is_error are independent flags.
    #[test]
    fn error_and_missing_independent(
        is_error in any::<bool>(),
        is_missing in any::<bool>(),
    ) {
        let node = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, is_error, is_missing, true, None,
        );
        prop_assert_eq!(node.is_error(), is_error);
        prop_assert_eq!(node.is_missing(), is_missing);
    }

    /// Error node with children still reports has_error.
    #[test]
    fn error_node_with_children_has_error(n in 1usize..4) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, true, false, true, None,
        );
        prop_assert!(node.is_error());
        prop_assert!(node.has_error());
    }

    /// Non-error tree: has_error is false when no descendant is an error.
    #[test]
    fn no_error_tree_has_no_error(n in 1usize..5) {
        let children: Vec<ParsedNode> = (0..n).map(|i| leaf(1, i, i + 1)).collect();
        let node = make_node(
            0, children, 0, n,
            pt(0, 0), pt(0, n as u32),
            false, false, false, true, None,
        );
        prop_assert!(!node.has_error());
    }

    /// has_error true if any one of multiple children is error.
    #[test]
    fn has_error_any_child(err_idx in 0usize..4) {
        let children: Vec<ParsedNode> = (0..4).map(|i| {
            make_node(
                1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, i == err_idx, false, true, None,
            )
        }).collect();
        let parent = make_node(
            0, children, 0, 4,
            pt(0, 0), pt(0, 4),
            false, false, false, true, None,
        );
        prop_assert!(parent.has_error());
    }
}

// ---------------------------------------------------------------------------
// 6. ParsedNode field_id
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Leaf with no field_id has None.
    #[test]
    fn leaf_field_id_is_none(sym in 0u16..100) {
        let node = leaf(sym, 0, 1);
        prop_assert_eq!(node.field_id, None);
    }

    /// field_id Some value is preserved.
    #[test]
    fn field_id_some_preserved(fid in 1u16..500) {
        let node = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, Some(fid),
        );
        prop_assert_eq!(node.field_id, Some(fid));
    }

    /// Distinct field_ids on sibling children are distinguishable.
    #[test]
    fn distinct_sibling_field_ids(
        fid_a in 1u16..100,
        fid_b in 100u16..200,
    ) {
        let child_a = make_node(
            1, vec![], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, Some(fid_a),
        );
        let child_b = make_node(
            2, vec![], 1, 2,
            pt(0, 1), pt(0, 2),
            false, false, false, true, Some(fid_b),
        );
        let parent = make_node(
            0, vec![child_a, child_b], 0, 2,
            pt(0, 0), pt(0, 2),
            false, false, false, true, None,
        );
        prop_assert_ne!(
            parent.child(0).unwrap().field_id,
            parent.child(1).unwrap().field_id
        );
    }
}

// ---------------------------------------------------------------------------
// 7. ParsedNode symbol
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Symbol boundary: 0 is valid.
    #[test]
    fn symbol_zero_is_valid(_dummy in 0u8..1) {
        let node = leaf(0, 0, 1);
        prop_assert_eq!(node.symbol(), 0);
    }

    /// Symbol high value is preserved.
    #[test]
    fn symbol_high_value(sym in 1000u16..u16::MAX) {
        let node = leaf(sym, 0, 1);
        prop_assert_eq!(node.symbol(), sym);
    }

    /// Symbol preserved when accessed through parent's child().
    #[test]
    fn symbol_preserved_through_child_access(
        parent_sym in 0u16..50,
        child_sym in 50u16..100,
    ) {
        let child = leaf(child_sym, 0, 1);
        let parent = make_node(
            parent_sym, vec![child], 0, 1,
            pt(0, 0), pt(0, 1),
            false, false, false, true, None,
        );
        prop_assert_eq!(parent.symbol(), parent_sym);
        prop_assert_eq!(parent.child(0).unwrap().symbol(), child_sym);
    }
}

// ---------------------------------------------------------------------------
// 8. ParsedNode metadata determinism
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// All accessors are idempotent: calling them twice yields the same result.
    #[test]
    fn accessors_idempotent(
        sym in 0u16..100,
        start in 0usize..500,
        len in 1usize..100,
        row in 0u32..100,
        col in 0u32..100,
        is_extra in any::<bool>(),
        is_error in any::<bool>(),
        is_missing in any::<bool>(),
        is_named in any::<bool>(),
        fid in proptest::option::of(0u16..50),
    ) {
        let end = start + len;
        let node = make_node(
            sym, vec![leaf(1, start, start + 1)], start, end,
            pt(row, col), pt(row, col + len as u32),
            is_extra, is_error, is_missing, is_named, fid,
        );
        // Call each accessor twice
        prop_assert_eq!(node.symbol(), node.symbol());
        prop_assert_eq!(node.start_byte(), node.start_byte());
        prop_assert_eq!(node.end_byte(), node.end_byte());
        prop_assert_eq!(node.start_point(), node.start_point());
        prop_assert_eq!(node.end_point(), node.end_point());
        prop_assert_eq!(node.is_extra(), node.is_extra());
        prop_assert_eq!(node.is_error(), node.is_error());
        prop_assert_eq!(node.is_missing(), node.is_missing());
        prop_assert_eq!(node.is_named(), node.is_named());
        prop_assert_eq!(node.has_error(), node.has_error());
        prop_assert_eq!(node.child_count(), node.child_count());
        prop_assert_eq!(node.kind(), node.kind());
        prop_assert_eq!(node.field_id, node.field_id);
    }

    /// Two independently constructed identical nodes produce the same metadata.
    #[test]
    fn independent_identical_construction(
        sym in 0u16..100,
        start in 0usize..200,
        len in 1usize..50,
        row in 0u32..50,
        col in 0u32..50,
    ) {
        let end = start + len;
        let a = make_node(
            sym, vec![], start, end,
            pt(row, col), pt(row, col + len as u32),
            false, false, false, true, Some(42),
        );
        let b = make_node(
            sym, vec![], start, end,
            pt(row, col), pt(row, col + len as u32),
            false, false, false, true, Some(42),
        );
        prop_assert_eq!(a.symbol(), b.symbol());
        prop_assert_eq!(a.start_byte(), b.start_byte());
        prop_assert_eq!(a.end_byte(), b.end_byte());
        prop_assert_eq!(a.start_point(), b.start_point());
        prop_assert_eq!(a.end_point(), b.end_point());
        prop_assert_eq!(a.is_extra(), b.is_extra());
        prop_assert_eq!(a.is_error(), b.is_error());
        prop_assert_eq!(a.is_missing(), b.is_missing());
        prop_assert_eq!(a.is_named(), b.is_named());
        prop_assert_eq!(a.has_error(), b.has_error());
        prop_assert_eq!(a.child_count(), b.child_count());
        prop_assert_eq!(a.kind(), b.kind());
        prop_assert_eq!(a.field_id, b.field_id);
    }

    /// Metadata is stable across clone: original and clone always agree.
    #[test]
    fn metadata_stable_across_clone(
        sym in 0u16..100,
        start in 0usize..200,
        len in 1usize..50,
        fid in proptest::option::of(0u16..50),
    ) {
        let end = start + len;
        let node = make_node(
            sym, vec![leaf(1, start, start + 1)], start, end,
            pt(0, start as u32), pt(0, end as u32),
            false, false, false, true, fid,
        );
        let cloned = node.clone();
        prop_assert_eq!(node.kind(), cloned.kind());
        prop_assert_eq!(node.has_error(), cloned.has_error());
        prop_assert_eq!(node.field_id, cloned.field_id);
        // Children metadata matches
        prop_assert_eq!(
            node.child(0).unwrap().symbol(),
            cloned.child(0).unwrap().symbol()
        );
    }
}
