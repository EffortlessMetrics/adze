#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the ParsedNode API in the adze runtime crate.
//!
//! Covers: construction, field access, Spanned wrapper, ChildWalker iteration,
//! named vs anonymous children, field-based access, node kind identification,
//! byte ranges, position tracking, error node detection, text extraction,
//! and integration with parsing results.

use adze::Spanned;
use adze::pure_parser::{ParseResult, ParsedNode, Point};
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Construct a `ParsedNode` without naming the `pub(crate)` `language` field.
/// Zero-initialises then overwrites every public field.
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

/// Named leaf node on row 0.
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

/// Anonymous (unnamed) leaf node on row 0.
fn anon_leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
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

/// Branch node with children.
fn branch(symbol: u16, start: usize, end: usize, children: Vec<ParsedNode>) -> ParsedNode {
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
}

/// Leaf with a field_id set.
fn field_leaf(symbol: u16, start: usize, end: usize, fid: u16) -> ParsedNode {
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
        Some(fid),
    )
}

/// Error leaf.
fn error_leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(
        symbol,
        vec![],
        start,
        end,
        pt(0, start as u32),
        pt(0, end as u32),
        false,
        true,
        false,
        true,
        None,
    )
}

// ===================================================================
// 1. ParsedNode construction and field access
// ===================================================================

#[test]
fn construct_leaf_and_read_symbol() {
    let n = leaf(42, 0, 5);
    assert_eq!(n.symbol(), 42);
}

#[test]
fn construct_branch_and_read_children_count() {
    let n = branch(
        10,
        0,
        10,
        vec![leaf(1, 0, 3), leaf(2, 3, 6), leaf(3, 6, 10)],
    );
    assert_eq!(n.child_count(), 3);
}

#[test]
fn field_id_defaults_to_none() {
    let n = leaf(1, 0, 1);
    assert_eq!(n.field_id, None);
}

#[test]
fn field_id_can_be_set() {
    let n = field_leaf(1, 0, 3, 7);
    assert_eq!(n.field_id, Some(7));
}

// ===================================================================
// 2. Spanned wrapper with start/end positions
// ===================================================================

#[test]
fn spanned_deref_to_inner_value() {
    let s = Spanned {
        value: 99,
        span: (0, 2),
    };
    assert_eq!(*s, 99);
}

#[test]
fn spanned_indexes_source_str() {
    let source = "hello world";
    let s = Spanned {
        value: (),
        span: (6, 11),
    };
    assert_eq!(&source[s], "world");
}

#[test]
fn spanned_empty_span_yields_empty_str() {
    let source = "abc";
    let s = Spanned {
        value: (),
        span: (1, 1),
    };
    assert_eq!(&source[s], "");
}

#[test]
fn spanned_clone_preserves_span() {
    let s = Spanned {
        value: "hi",
        span: (3, 7),
    };
    let c = s.clone();
    assert_eq!(c.span, (3, 7));
    assert_eq!(c.value, "hi");
}

// ===================================================================
// 3. ChildWalker iteration over children
// ===================================================================

#[test]
fn walker_empty_node_goto_first_returns_false() {
    let n = leaf(1, 0, 1);
    let mut w = n.walk();
    assert!(!w.goto_first_child());
}

#[test]
fn walker_traverses_all_children() {
    let n = branch(10, 0, 9, vec![leaf(1, 0, 3), leaf(2, 3, 6), leaf(3, 6, 9)]);
    let mut w = n.walk();
    let mut symbols = vec![];
    if w.goto_first_child() {
        symbols.push(w.node().symbol());
        while w.goto_next_sibling() {
            symbols.push(w.node().symbol());
        }
    }
    assert_eq!(symbols, vec![1, 2, 3]);
}

#[test]
fn walker_goto_next_sibling_at_end_returns_false() {
    let n = branch(10, 0, 3, vec![leaf(1, 0, 3)]);
    let mut w = n.walk();
    assert!(w.goto_first_child());
    assert!(!w.goto_next_sibling());
}

#[test]
fn walker_goto_first_child_resets_index() {
    let n = branch(10, 0, 6, vec![leaf(1, 0, 3), leaf(2, 3, 6)]);
    let mut w = n.walk();
    w.goto_first_child();
    w.goto_next_sibling(); // at child index 1
    w.goto_first_child(); // reset to 0
    assert_eq!(w.node().symbol(), 1);
}

// ===================================================================
// 4. Named vs anonymous child access
// ===================================================================

#[test]
fn named_and_anonymous_children_mixed() {
    let children = vec![
        leaf(1, 0, 1),      // named
        anon_leaf(2, 1, 2), // anonymous
        leaf(3, 2, 3),      // named
    ];
    let parent = branch(10, 0, 3, children);

    assert!(parent.child(0).unwrap().is_named());
    assert!(!parent.child(1).unwrap().is_named());
    assert!(parent.child(2).unwrap().is_named());
}

#[test]
fn filter_named_children_from_slice() {
    let children = vec![
        leaf(1, 0, 1),
        anon_leaf(2, 1, 2),
        anon_leaf(3, 2, 3),
        leaf(4, 3, 4),
    ];
    let parent = branch(10, 0, 4, children);
    let named: Vec<_> = parent.children().iter().filter(|c| c.is_named()).collect();
    assert_eq!(named.len(), 2);
    assert_eq!(named[0].symbol(), 1);
    assert_eq!(named[1].symbol(), 4);
}

#[test]
fn all_anonymous_children() {
    let children = vec![anon_leaf(1, 0, 1), anon_leaf(2, 1, 2)];
    let parent = branch(10, 0, 2, children);
    let named: Vec<_> = parent.children().iter().filter(|c| c.is_named()).collect();
    assert!(named.is_empty());
}

// ===================================================================
// 5. Field-based child access
// ===================================================================

#[test]
fn children_with_field_ids() {
    let children = vec![
        field_leaf(1, 0, 3, 10),
        leaf(2, 3, 4),
        field_leaf(3, 4, 7, 20),
    ];
    let parent = branch(100, 0, 7, children);
    assert_eq!(parent.child(0).unwrap().field_id, Some(10));
    assert_eq!(parent.child(1).unwrap().field_id, None);
    assert_eq!(parent.child(2).unwrap().field_id, Some(20));
}

#[test]
fn find_child_by_field_id() {
    let children = vec![
        field_leaf(1, 0, 2, 5),
        leaf(2, 2, 4),
        field_leaf(3, 4, 6, 8),
    ];
    let parent = branch(100, 0, 6, children);
    let found = parent.children().iter().find(|c| c.field_id == Some(8));
    assert!(found.is_some());
    assert_eq!(found.unwrap().symbol(), 3);
}

#[test]
fn no_child_matches_field_id() {
    let parent = branch(10, 0, 3, vec![leaf(1, 0, 1), leaf(2, 1, 3)]);
    let found = parent.children().iter().find(|c| c.field_id == Some(99));
    assert!(found.is_none());
}

// ===================================================================
// 6. Node kind identification
// ===================================================================

#[test]
fn kind_fallback_symbol_zero_is_end() {
    let n = leaf(0, 0, 1);
    assert_eq!(n.kind(), "end");
}

#[test]
fn kind_fallback_large_symbol_is_unknown() {
    let n = leaf(999, 0, 1);
    assert_eq!(n.kind(), "unknown");
}

#[test]
fn kind_fallback_symbol_five_is_expression() {
    let n = leaf(5, 0, 1);
    assert_eq!(n.kind(), "Expression");
}

// ===================================================================
// 7. Byte ranges and position tracking
// ===================================================================

#[test]
fn byte_range_accessors() {
    let n = leaf(1, 10, 25);
    assert_eq!(n.start_byte(), 10);
    assert_eq!(n.end_byte(), 25);
}

#[test]
fn point_accessors() {
    let n = make_node(
        1,
        vec![],
        0,
        20,
        pt(2, 5),
        pt(4, 10),
        false,
        false,
        false,
        true,
        None,
    );
    assert_eq!(n.start_point(), pt(2, 5));
    assert_eq!(n.end_point(), pt(4, 10));
}

#[test]
fn multiline_node_positions() {
    let n = make_node(
        1,
        vec![],
        0,
        30,
        pt(0, 0),
        pt(3, 8),
        false,
        false,
        false,
        true,
        None,
    );
    assert_eq!(n.start_point().row, 0);
    assert_eq!(n.end_point().row, 3);
    assert_eq!(n.end_point().column, 8);
}

#[test]
fn zero_length_node_has_equal_start_end() {
    let n = leaf(1, 5, 5);
    assert_eq!(n.start_byte(), n.end_byte());
    assert_eq!(n.start_point(), n.end_point());
}

// ===================================================================
// 8. Error node detection
// ===================================================================

#[test]
fn is_error_on_error_node() {
    let n = error_leaf(99, 0, 1);
    assert!(n.is_error());
}

#[test]
fn is_error_on_normal_node() {
    let n = leaf(1, 0, 1);
    assert!(!n.is_error());
}

#[test]
fn has_error_propagates_to_parent() {
    let err = error_leaf(99, 2, 3);
    let parent = branch(10, 0, 5, vec![leaf(1, 0, 2), err]);
    assert!(!parent.is_error());
    assert!(parent.has_error());
}

#[test]
fn has_error_deeply_nested() {
    let err = error_leaf(99, 4, 5);
    let inner = branch(5, 3, 5, vec![err]);
    let mid = branch(3, 0, 5, vec![leaf(1, 0, 3), inner]);
    let root = branch(1, 0, 5, vec![mid]);
    assert!(root.has_error());
}

#[test]
fn has_error_clean_tree_returns_false() {
    let root = branch(
        10,
        0,
        6,
        vec![
            leaf(1, 0, 2),
            branch(5, 2, 6, vec![leaf(2, 2, 4), leaf(3, 4, 6)]),
        ],
    );
    assert!(!root.has_error());
}

#[test]
fn is_missing_flag() {
    let n = make_node(
        1,
        vec![],
        0,
        0,
        pt(0, 0),
        pt(0, 0),
        false,
        false,
        true,
        true,
        None,
    );
    assert!(n.is_missing());
    assert!(!n.is_error());
}

// ===================================================================
// 9. Node text extraction
// ===================================================================

#[test]
fn utf8_text_full_source() {
    let source = b"hello";
    let n = leaf(1, 0, 5);
    assert_eq!(n.utf8_text(source).unwrap(), "hello");
}

#[test]
fn utf8_text_substring() {
    let source = b"abcdef";
    let n = leaf(1, 2, 5);
    assert_eq!(n.utf8_text(source).unwrap(), "cde");
}

#[test]
fn utf8_text_empty_range() {
    let source = b"abc";
    let n = leaf(1, 1, 1);
    assert_eq!(n.utf8_text(source).unwrap(), "");
}

#[test]
fn utf8_text_out_of_bounds_returns_err() {
    let source = b"hi";
    let n = leaf(1, 0, 100);
    assert!(n.utf8_text(source).is_err());
}

#[test]
fn utf8_text_unicode_multibyte() {
    let source = "日本語".as_bytes(); // 9 bytes (3 chars × 3 bytes)
    let n = leaf(1, 0, source.len());
    assert_eq!(n.utf8_text(source).unwrap(), "日本語");
}

#[test]
fn utf8_text_child_extraction() {
    let source = b"1+2*3";
    let root = branch(
        10,
        0,
        5,
        vec![
            leaf(1, 0, 1),
            leaf(2, 1, 2),
            leaf(3, 2, 3),
            leaf(4, 3, 4),
            leaf(5, 4, 5),
        ],
    );
    assert_eq!(root.child(0).unwrap().utf8_text(source).unwrap(), "1");
    assert_eq!(root.child(1).unwrap().utf8_text(source).unwrap(), "+");
    assert_eq!(root.child(3).unwrap().utf8_text(source).unwrap(), "*");
}

// ===================================================================
// 10. Integration with parsing results
// ===================================================================

#[test]
fn parse_result_with_root_and_no_errors() {
    let root = leaf(1, 0, 5);
    let result = ParseResult {
        root: Some(root),
        errors: vec![],
    };
    assert!(result.root.is_some());
    assert!(result.errors.is_empty());
    assert_eq!(result.root.unwrap().symbol(), 1);
}

#[test]
fn parse_result_with_no_root() {
    let result = ParseResult {
        root: None,
        errors: vec![],
    };
    assert!(result.root.is_none());
}

#[test]
fn clone_preserves_all_fields() {
    let n = make_node(
        42,
        vec![leaf(1, 0, 2), leaf(2, 2, 4)],
        0,
        4,
        pt(0, 0),
        pt(0, 4),
        true,
        false,
        false,
        true,
        Some(7),
    );
    let c = n.clone();
    assert_eq!(c.symbol(), n.symbol());
    assert_eq!(c.start_byte(), n.start_byte());
    assert_eq!(c.end_byte(), n.end_byte());
    assert_eq!(c.start_point(), n.start_point());
    assert_eq!(c.end_point(), n.end_point());
    assert_eq!(c.is_extra(), n.is_extra());
    assert_eq!(c.is_error(), n.is_error());
    assert_eq!(c.is_missing(), n.is_missing());
    assert_eq!(c.is_named(), n.is_named());
    assert_eq!(c.field_id, n.field_id);
    assert_eq!(c.child_count(), n.child_count());
}

#[test]
fn is_extra_flag_works() {
    let n = make_node(
        1,
        vec![],
        0,
        1,
        pt(0, 0),
        pt(0, 1),
        true,
        false,
        false,
        true,
        None,
    );
    assert!(n.is_extra());
}

#[test]
fn point_default_is_zero() {
    let p = Point::default();
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_equality_and_inequality() {
    let a = pt(1, 5);
    let b = pt(1, 5);
    let c = pt(2, 0);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn debug_format_contains_struct_name() {
    let n = leaf(7, 0, 3);
    let dbg = format!("{:?}", n);
    assert!(dbg.contains("ParsedNode"));
}
