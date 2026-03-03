#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `ChildWalker` iteration in the adze runtime.
//!
//! Covers: empty walker, single/multiple child iteration, named vs anonymous
//! children, iteration order, error nodes, reset/reuse, field-based access,
//! and clone/debug of nodes obtained through the walker.

use adze::pure_parser::{ParsedNode, Point};
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

/// Named leaf on row 0.
fn leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol, vec![], start, end,
        pt(0, start as u32), pt(0, end as u32),
        false, false, false, true, None,
    )
}

/// Anonymous (unnamed) leaf on row 0.
fn anon_leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol, vec![], start, end,
        pt(0, start as u32), pt(0, end as u32),
        false, false, false, false, None,
    )
}

/// Branch node with children.
fn branch(symbol: u16, start: usize, end: usize, children: Vec<ParsedNode>) -> ParsedNode {
    make_node(
        symbol, children, start, end,
        pt(0, start as u32), pt(0, end as u32),
        false, false, false, true, None,
    )
}

/// Error leaf node.
fn error_leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol, vec![], start, end,
        pt(0, start as u32), pt(0, end as u32),
        false, true, false, true, None,
    )
}

/// Leaf with a field_id.
fn field_leaf(symbol: u16, start: usize, end: usize, fid: u16) -> ParsedNode {
    make_node(
        symbol, vec![], start, end,
        pt(0, start as u32), pt(0, end as u32),
        false, false, false, true, Some(fid),
    )
}

// ===================================================================
// Empty ChildWalker
// ===================================================================

#[test]
fn empty_walker_goto_first_child_returns_false() {
    let node = leaf(1, 0, 1);
    let mut walker = node.walk();
    assert!(!walker.goto_first_child());
}

#[test]
fn empty_walker_from_childless_branch() {
    let node = branch(10, 0, 5, vec![]);
    let mut walker = node.walk();
    assert!(!walker.goto_first_child());
}

// ===================================================================
// Single child iteration
// ===================================================================

#[test]
fn single_child_goto_first_child_returns_true() {
    let parent = branch(10, 0, 3, vec![leaf(1, 0, 3)]);
    let mut walker = parent.walk();
    assert!(walker.goto_first_child());
}

#[test]
fn single_child_node_returns_correct_child() {
    let parent = branch(10, 0, 3, vec![leaf(1, 0, 3)]);
    let mut walker = parent.walk();
    assert!(walker.goto_first_child());
    assert_eq!(walker.node().symbol(), 1);
    assert_eq!(walker.node().start_byte(), 0);
    assert_eq!(walker.node().end_byte(), 3);
}

#[test]
fn single_child_no_next_sibling() {
    let parent = branch(10, 0, 3, vec![leaf(1, 0, 3)]);
    let mut walker = parent.walk();
    assert!(walker.goto_first_child());
    assert!(!walker.goto_next_sibling());
}

#[test]
fn single_child_goto_next_sibling_repeated_stays_false() {
    let parent = branch(10, 0, 3, vec![leaf(1, 0, 3)]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    assert!(!walker.goto_next_sibling());
    assert!(!walker.goto_next_sibling());
    // Still points at the same child
    assert_eq!(walker.node().symbol(), 1);
}

// ===================================================================
// Multiple children iteration
// ===================================================================

#[test]
fn three_children_full_traversal() {
    let parent = branch(10, 0, 9, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
        leaf(3, 6, 9),
    ]);
    let mut walker = parent.walk();
    let mut symbols = Vec::new();
    if walker.goto_first_child() {
        symbols.push(walker.node().symbol());
        while walker.goto_next_sibling() {
            symbols.push(walker.node().symbol());
        }
    }
    assert_eq!(symbols, vec![1, 2, 3]);
}

#[test]
fn five_children_count_matches() {
    let kids: Vec<ParsedNode> = (0..5).map(|i| leaf(i as u16, i, i + 1)).collect();
    let parent = branch(100, 0, 5, kids);
    let mut walker = parent.walk();
    let mut count = 0;
    if walker.goto_first_child() {
        count += 1;
        while walker.goto_next_sibling() {
            count += 1;
        }
    }
    assert_eq!(count, 5);
}

#[test]
fn multiple_children_walker_matches_children_slice() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 2),
        leaf(2, 2, 4),
        leaf(3, 4, 6),
    ]);
    let children = parent.children();
    let mut walker = parent.walk();
    let mut idx = 0;
    if walker.goto_first_child() {
        assert_eq!(walker.node().symbol(), children[idx].symbol());
        idx += 1;
        while walker.goto_next_sibling() {
            assert_eq!(walker.node().symbol(), children[idx].symbol());
            idx += 1;
        }
    }
    assert_eq!(idx, children.len());
}

// ===================================================================
// Named vs anonymous children
// ===================================================================

#[test]
fn named_children_identified_through_walker() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 2),       // named
        anon_leaf(2, 2, 4),  // anonymous
        leaf(3, 4, 6),       // named
    ]);
    let mut walker = parent.walk();
    let mut named = Vec::new();
    if walker.goto_first_child() {
        named.push(walker.node().is_named());
        while walker.goto_next_sibling() {
            named.push(walker.node().is_named());
        }
    }
    assert_eq!(named, vec![true, false, true]);
}

#[test]
fn all_anonymous_children() {
    let parent = branch(10, 0, 6, vec![
        anon_leaf(1, 0, 2),
        anon_leaf(2, 2, 4),
        anon_leaf(3, 4, 6),
    ]);
    let mut walker = parent.walk();
    if walker.goto_first_child() {
        assert!(!walker.node().is_named());
        while walker.goto_next_sibling() {
            assert!(!walker.node().is_named());
        }
    }
}

#[test]
fn count_named_children_via_walker() {
    let parent = branch(10, 0, 8, vec![
        anon_leaf(1, 0, 2),
        leaf(2, 2, 4),
        anon_leaf(3, 4, 6),
        leaf(4, 6, 8),
    ]);
    let mut walker = parent.walk();
    let mut named_count = 0;
    if walker.goto_first_child() {
        if walker.node().is_named() { named_count += 1; }
        while walker.goto_next_sibling() {
            if walker.node().is_named() { named_count += 1; }
        }
    }
    assert_eq!(named_count, 2);
}

// ===================================================================
// Iteration order
// ===================================================================

#[test]
fn walker_visits_in_source_order() {
    let parent = branch(10, 0, 12, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
        leaf(3, 6, 9),
        leaf(4, 9, 12),
    ]);
    let mut walker = parent.walk();
    let mut starts = Vec::new();
    if walker.goto_first_child() {
        starts.push(walker.node().start_byte());
        while walker.goto_next_sibling() {
            starts.push(walker.node().start_byte());
        }
    }
    assert_eq!(starts, vec![0, 3, 6, 9]);
}

#[test]
fn walker_end_bytes_monotonically_increase() {
    let parent = branch(10, 0, 12, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 7),
        leaf(3, 7, 12),
    ]);
    let mut walker = parent.walk();
    let mut ends = Vec::new();
    if walker.goto_first_child() {
        ends.push(walker.node().end_byte());
        while walker.goto_next_sibling() {
            ends.push(walker.node().end_byte());
        }
    }
    for i in 1..ends.len() {
        assert!(ends[i] >= ends[i - 1], "end bytes should be non-decreasing");
    }
}

#[test]
fn walker_symbol_order_preserved() {
    let syms: Vec<u16> = vec![42, 7, 99, 3, 50];
    let kids: Vec<ParsedNode> = syms.iter().enumerate()
        .map(|(i, &s)| leaf(s, i, i + 1))
        .collect();
    let parent = branch(10, 0, 5, kids);
    let mut walker = parent.walk();
    let mut collected = Vec::new();
    if walker.goto_first_child() {
        collected.push(walker.node().symbol());
        while walker.goto_next_sibling() {
            collected.push(walker.node().symbol());
        }
    }
    assert_eq!(collected, syms);
}

// ===================================================================
// ChildWalker with error nodes
// ===================================================================

#[test]
fn walker_traverses_error_nodes() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 2),
        error_leaf(0xFFFF, 2, 4),
        leaf(3, 4, 6),
    ]);
    let mut walker = parent.walk();
    let mut errors = Vec::new();
    if walker.goto_first_child() {
        errors.push(walker.node().is_error());
        while walker.goto_next_sibling() {
            errors.push(walker.node().is_error());
        }
    }
    assert_eq!(errors, vec![false, true, false]);
}

#[test]
fn walker_error_node_has_error_flag() {
    let parent = branch(10, 0, 4, vec![
        error_leaf(5, 0, 2),
        leaf(6, 2, 4),
    ]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    assert!(walker.node().is_error());
    walker.goto_next_sibling();
    assert!(!walker.node().is_error());
}

#[test]
fn walker_has_error_propagates_from_child() {
    let parent = branch(10, 0, 4, vec![
        error_leaf(5, 0, 2),
        leaf(6, 2, 4),
    ]);
    assert!(parent.has_error());
}

#[test]
fn walker_all_error_children() {
    let parent = branch(10, 0, 4, vec![
        error_leaf(1, 0, 2),
        error_leaf(2, 2, 4),
    ]);
    let mut walker = parent.walk();
    if walker.goto_first_child() {
        assert!(walker.node().is_error());
        while walker.goto_next_sibling() {
            assert!(walker.node().is_error());
        }
    }
}

// ===================================================================
// ChildWalker reset/reuse
// ===================================================================

#[test]
fn walker_reset_via_goto_first_child() {
    let parent = branch(10, 0, 9, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
        leaf(3, 6, 9),
    ]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    walker.goto_next_sibling();
    assert_eq!(walker.node().symbol(), 2);
    // Reset
    walker.goto_first_child();
    assert_eq!(walker.node().symbol(), 1);
}

#[test]
fn walker_reset_after_full_traversal() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
    ]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    while walker.goto_next_sibling() {}
    assert_eq!(walker.node().symbol(), 2);
    // Reset and traverse again
    walker.goto_first_child();
    let mut symbols = vec![walker.node().symbol()];
    while walker.goto_next_sibling() {
        symbols.push(walker.node().symbol());
    }
    assert_eq!(symbols, vec![1, 2]);
}

#[test]
fn walker_multiple_resets() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 2),
        leaf(2, 2, 4),
        leaf(3, 4, 6),
    ]);
    let mut walker = parent.walk();
    for _ in 0..3 {
        assert!(walker.goto_first_child());
        assert_eq!(walker.node().symbol(), 1);
        walker.goto_next_sibling();
        assert_eq!(walker.node().symbol(), 2);
    }
}

#[test]
fn walker_reset_on_empty_stays_false() {
    let node = leaf(1, 0, 1);
    let mut walker = node.walk();
    assert!(!walker.goto_first_child());
    assert!(!walker.goto_first_child());
    assert!(!walker.goto_first_child());
}

// ===================================================================
// Field-based child access
// ===================================================================

#[test]
fn walker_children_have_field_ids() {
    let parent = branch(10, 0, 6, vec![
        field_leaf(1, 0, 2, 100),
        field_leaf(2, 2, 4, 200),
        field_leaf(3, 4, 6, 300),
    ]);
    let mut walker = parent.walk();
    let mut field_ids = Vec::new();
    if walker.goto_first_child() {
        field_ids.push(walker.node().field_id);
        while walker.goto_next_sibling() {
            field_ids.push(walker.node().field_id);
        }
    }
    assert_eq!(field_ids, vec![Some(100), Some(200), Some(300)]);
}

#[test]
fn walker_mixed_field_and_no_field_children() {
    let parent = branch(10, 0, 6, vec![
        field_leaf(1, 0, 2, 10),
        leaf(2, 2, 4),          // no field
        field_leaf(3, 4, 6, 30),
    ]);
    let mut walker = parent.walk();
    let mut fields = Vec::new();
    if walker.goto_first_child() {
        fields.push(walker.node().field_id);
        while walker.goto_next_sibling() {
            fields.push(walker.node().field_id);
        }
    }
    assert_eq!(fields, vec![Some(10), None, Some(30)]);
}

#[test]
fn walker_field_id_via_direct_child_access() {
    let parent = branch(10, 0, 4, vec![
        field_leaf(1, 0, 2, 42),
        leaf(2, 2, 4),
    ]);
    // Verify walker and direct child() agree on field_id
    let mut walker = parent.walk();
    walker.goto_first_child();
    assert_eq!(walker.node().field_id, parent.child(0).unwrap().field_id);
    walker.goto_next_sibling();
    assert_eq!(walker.node().field_id, parent.child(1).unwrap().field_id);
}

// ===================================================================
// ChildWalker clone/debug (on nodes from the walker)
// ===================================================================

#[test]
fn walker_node_is_cloneable() {
    let parent = branch(10, 0, 3, vec![leaf(1, 0, 3)]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    let cloned = walker.node().clone();
    assert_eq!(cloned.symbol(), walker.node().symbol());
    assert_eq!(cloned.start_byte(), walker.node().start_byte());
    assert_eq!(cloned.end_byte(), walker.node().end_byte());
}

#[test]
fn walker_node_debug_format() {
    let parent = branch(10, 0, 3, vec![leaf(7, 0, 3)]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    let dbg = format!("{:?}", walker.node());
    assert!(dbg.contains("ParsedNode"));
    assert!(dbg.contains("symbol: 7"));
}

#[test]
fn walker_cloned_node_independent_of_original() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
    ]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    let first = walker.node().clone();
    walker.goto_next_sibling();
    // After advancing, cloned node still refers to the first child
    assert_eq!(first.symbol(), 1);
    assert_eq!(walker.node().symbol(), 2);
}

#[test]
fn walker_debug_parent_shows_children() {
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
    ]);
    let dbg = format!("{:?}", parent);
    assert!(dbg.contains("children:"));
}

// ===================================================================
// Additional edge cases
// ===================================================================

#[test]
fn walker_on_deeply_nested_node() {
    let inner = branch(3, 2, 4, vec![leaf(4, 2, 4)]);
    let mid = branch(2, 0, 6, vec![leaf(1, 0, 2), inner, leaf(5, 4, 6)]);
    let root = branch(100, 0, 6, vec![mid]);
    // Walk root -> one child
    let mut walker = root.walk();
    assert!(walker.goto_first_child());
    assert_eq!(walker.node().symbol(), 2);
    assert!(!walker.goto_next_sibling());
    // Walk into the mid node's children
    let mid_node = walker.node();
    let mut mid_walker = mid_node.walk();
    assert!(mid_walker.goto_first_child());
    assert_eq!(mid_walker.node().symbol(), 1);
    assert!(mid_walker.goto_next_sibling());
    assert_eq!(mid_walker.node().symbol(), 3);
    assert!(mid_walker.goto_next_sibling());
    assert_eq!(mid_walker.node().symbol(), 5);
    assert!(!mid_walker.goto_next_sibling());
}

#[test]
fn walker_node_byte_ranges_consistent() {
    let parent = branch(10, 0, 9, vec![
        leaf(1, 0, 3),
        leaf(2, 3, 6),
        leaf(3, 6, 9),
    ]);
    let mut walker = parent.walk();
    if walker.goto_first_child() {
        let mut prev_end = walker.node().start_byte();
        assert_eq!(prev_end, 0);
        prev_end = walker.node().end_byte();
        while walker.goto_next_sibling() {
            assert_eq!(walker.node().start_byte(), prev_end);
            prev_end = walker.node().end_byte();
        }
        assert_eq!(prev_end, 9);
    }
}

#[test]
fn walker_extra_children_visible() {
    let extra_child = make_node(
        99, vec![], 2, 3,
        pt(0, 2), pt(0, 3),
        true, false, false, false, None,
    );
    let parent = branch(10, 0, 6, vec![
        leaf(1, 0, 2),
        extra_child,
        leaf(2, 3, 6),
    ]);
    let mut walker = parent.walk();
    let mut extras = Vec::new();
    if walker.goto_first_child() {
        extras.push(walker.node().is_extra());
        while walker.goto_next_sibling() {
            extras.push(walker.node().is_extra());
        }
    }
    assert_eq!(extras, vec![false, true, false]);
}

#[test]
fn walker_missing_node_visible() {
    let missing = make_node(
        50, vec![], 2, 2,
        pt(0, 2), pt(0, 2),
        false, false, true, true, None,
    );
    let parent = branch(10, 0, 4, vec![
        leaf(1, 0, 2),
        missing,
        leaf(2, 2, 4),
    ]);
    let mut walker = parent.walk();
    let mut missing_flags = Vec::new();
    if walker.goto_first_child() {
        missing_flags.push(walker.node().is_missing());
        while walker.goto_next_sibling() {
            missing_flags.push(walker.node().is_missing());
        }
    }
    assert_eq!(missing_flags, vec![false, true, false]);
}

#[test]
fn walker_points_track_correctly() {
    let child0 = make_node(1, vec![], 0, 5, pt(0, 0), pt(0, 5), false, false, false, true, None);
    let child1 = make_node(2, vec![], 6, 10, pt(1, 0), pt(1, 4), false, false, false, true, None);
    let parent = branch(10, 0, 10, vec![child0, child1]);
    let mut walker = parent.walk();
    walker.goto_first_child();
    assert_eq!(walker.node().start_point(), pt(0, 0));
    assert_eq!(walker.node().end_point(), pt(0, 5));
    walker.goto_next_sibling();
    assert_eq!(walker.node().start_point(), pt(1, 0));
    assert_eq!(walker.node().end_point(), pt(1, 4));
}
