#![allow(clippy::needless_range_loop)]

//! Property-based tests for `ChildWalker` in the adze runtime.
//!
//! Uses proptest to verify invariants of cursor-style child traversal
//! over randomly generated `ParsedNode` trees.

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

/// Collect all symbols via the walker and return them.
fn walk_symbols(node: &ParsedNode) -> Vec<u16> {
    let mut walker = node.walk();
    let mut out = Vec::new();
    if walker.goto_first_child() {
        out.push(walker.node().symbol());
        while walker.goto_next_sibling() {
            out.push(walker.node().symbol());
        }
    }
    out
}

/// Strategy: generate a Vec of unique symbol ids with length in 0..=max_len.
fn symbol_vec(max_len: usize) -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(1u16..500, 0..=max_len)
}

// ===================================================================
// 1. Empty walker: goto_first_child always returns false
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn empty_walker_goto_first_child_is_false(sym in 0u16..1000) {
        let node = leaf(sym, 0, 1);
        let mut walker = node.walk();
        prop_assert!(!walker.goto_first_child());
    }
}

// ===================================================================
// 2. Empty walker on branch with no children
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn empty_branch_walker_is_false(sym in 0u16..1000) {
        let node = branch(sym, 0, 10, vec![]);
        let mut walker = node.walk();
        prop_assert!(!walker.goto_first_child());
    }
}

// ===================================================================
// 3. Walker count matches child_count
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_count_matches_child_count(syms in symbol_vec(20)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let n = kids.len();
        let parent = branch(999, 0, n, kids);

        let walked = walk_symbols(&parent);
        prop_assert_eq!(walked.len(), n);
        prop_assert_eq!(walked.len(), parent.child_count());
    }
}

// ===================================================================
// 4. Walker order matches children order
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_order_matches_children_order(syms in symbol_vec(20)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let walked = walk_symbols(&parent);
        let direct: Vec<u16> = parent.children().iter().map(|c| c.symbol()).collect();
        prop_assert_eq!(walked, direct);
    }
}

// ===================================================================
// 5. Walker start_byte order matches children slice
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_start_bytes_match_children(syms in symbol_vec(15)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i * 3, i * 3 + 2))
            .collect();
        let total = if kids.is_empty() { 0 } else { kids.last().unwrap().end_byte };
        let parent = branch(999, 0, total, kids);

        let mut walker = parent.walk();
        let mut walker_starts = Vec::new();
        if walker.goto_first_child() {
            walker_starts.push(walker.node().start_byte());
            while walker.goto_next_sibling() {
                walker_starts.push(walker.node().start_byte());
            }
        }
        let direct_starts: Vec<usize> = parent.children().iter().map(|c| c.start_byte()).collect();
        prop_assert_eq!(walker_starts, direct_starts);
    }
}

// ===================================================================
// 6. Named vs unnamed children: walker preserves is_named flags
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_preserves_is_named_flags(flags in prop::collection::vec(any::<bool>(), 1..=15)) {
        let kids: Vec<ParsedNode> = flags.iter().enumerate().map(|(i, &named)| {
            if named {
                leaf(i as u16 + 1, i, i + 1)
            } else {
                anon_leaf(i as u16 + 1, i, i + 1)
            }
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        let mut walked_flags = Vec::new();
        if walker.goto_first_child() {
            walked_flags.push(walker.node().is_named());
            while walker.goto_next_sibling() {
                walked_flags.push(walker.node().is_named());
            }
        }
        prop_assert_eq!(walked_flags, flags);
    }
}

// ===================================================================
// 7. Named child count via walker matches filter
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn named_child_count_via_walker(flags in prop::collection::vec(any::<bool>(), 0..=15)) {
        let kids: Vec<ParsedNode> = flags.iter().enumerate().map(|(i, &named)| {
            if named { leaf(i as u16, i, i + 1) } else { anon_leaf(i as u16, i, i + 1) }
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);
        let expected = flags.iter().filter(|&&f| f).count();

        let mut walker = parent.walk();
        let mut named_count = 0usize;
        if walker.goto_first_child() {
            if walker.node().is_named() { named_count += 1; }
            while walker.goto_next_sibling() {
                if walker.node().is_named() { named_count += 1; }
            }
        }
        prop_assert_eq!(named_count, expected);
    }
}

// ===================================================================
// 8. Walker reset (goto_first_child) returns to first child
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_reset_returns_to_first(syms in prop::collection::vec(1u16..500, 2..=10)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        walker.goto_first_child();
        // Advance partway
        walker.goto_next_sibling();
        let mid_sym = walker.node().symbol();
        // Reset
        walker.goto_first_child();
        prop_assert_eq!(walker.node().symbol(), syms[0]);
        // Confirm mid was different (unless only 2 children with same sym)
        if syms[0] != syms[1] {
            prop_assert_ne!(mid_sym, syms[0]);
        }
    }
}

// ===================================================================
// 9. Walker reset after full traversal yields same sequence
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_reset_full_traversal_same_sequence(syms in prop::collection::vec(1u16..500, 1..=15)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let first_pass = walk_symbols(&parent);
        let second_pass = walk_symbols(&parent);
        prop_assert_eq!(first_pass, second_pass);
    }
}

// ===================================================================
// 10. Multiple resets always start at index 0
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn multiple_resets_always_start_at_zero(
        syms in prop::collection::vec(1u16..500, 2..=8),
        resets in 2usize..=6,
    ) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        for _ in 0..resets {
            prop_assert!(walker.goto_first_child());
            prop_assert_eq!(walker.node().symbol(), syms[0]);
            // Advance to end
            while walker.goto_next_sibling() {}
        }
    }
}

// ===================================================================
// 11. Field-based child access: field_ids preserved through walker
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_preserves_field_ids(fids in prop::collection::vec(1u16..1000, 1..=10)) {
        let kids: Vec<ParsedNode> = fids.iter().enumerate()
            .map(|(i, &fid)| field_leaf(i as u16 + 1, i, i + 1, fid))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        let mut walked_fids = Vec::new();
        if walker.goto_first_child() {
            walked_fids.push(walker.node().field_id);
            while walker.goto_next_sibling() {
                walked_fids.push(walker.node().field_id);
            }
        }
        let expected: Vec<Option<u16>> = fids.into_iter().map(Some).collect();
        prop_assert_eq!(walked_fids, expected);
    }
}

// ===================================================================
// 12. Mixed field/no-field children
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_mixed_field_none(
        has_field in prop::collection::vec(any::<bool>(), 1..=12),
        fid_base in 1u16..500,
    ) {
        let kids: Vec<ParsedNode> = has_field.iter().enumerate().map(|(i, &hf)| {
            if hf {
                field_leaf(i as u16 + 1, i, i + 1, fid_base + i as u16)
            } else {
                leaf(i as u16 + 1, i, i + 1)
            }
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        let mut walked_fids = Vec::new();
        if walker.goto_first_child() {
            walked_fids.push(walker.node().field_id);
            while walker.goto_next_sibling() {
                walked_fids.push(walker.node().field_id);
            }
        }
        let expected: Vec<Option<u16>> = has_field.iter().enumerate().map(|(i, &hf)| {
            if hf { Some(fid_base + i as u16) } else { None }
        }).collect();
        prop_assert_eq!(walked_fids, expected);
    }
}

// ===================================================================
// 13. Walker field_id agrees with direct child() access
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_field_id_matches_direct_child(fids in prop::collection::vec(prop::option::of(1u16..1000), 1..=10)) {
        let kids: Vec<ParsedNode> = fids.iter().enumerate().map(|(i, &fid)| {
            make_node(
                i as u16 + 1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, false, false, true, fid,
            )
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        if walker.goto_first_child() {
            let mut idx = 0;
            prop_assert_eq!(walker.node().field_id, parent.child(idx).unwrap().field_id);
            idx += 1;
            while walker.goto_next_sibling() {
                prop_assert_eq!(walker.node().field_id, parent.child(idx).unwrap().field_id);
                idx += 1;
            }
            prop_assert_eq!(idx, parent.child_count());
        }
    }
}

// ===================================================================
// 14. Multiple walkers on same node see identical sequences
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn multiple_walkers_same_node(syms in prop::collection::vec(1u16..500, 0..=15)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let seq1 = walk_symbols(&parent);
        let seq2 = walk_symbols(&parent);
        let seq3 = walk_symbols(&parent);
        prop_assert_eq!(&seq1, &seq2);
        prop_assert_eq!(&seq2, &seq3);
    }
}

// ===================================================================
// 15. Two walkers can be interleaved independently
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn two_walkers_interleaved(syms in prop::collection::vec(1u16..500, 3..=10)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut w1 = parent.walk();
        let mut w2 = parent.walk();

        w1.goto_first_child();
        w2.goto_first_child();
        // Advance w1 twice
        w1.goto_next_sibling();
        w1.goto_next_sibling();
        // w2 should still be at first child
        prop_assert_eq!(w2.node().symbol(), syms[0]);
        prop_assert_eq!(w1.node().symbol(), syms[2]);
    }
}

// ===================================================================
// 16. goto_next_sibling at end stays false repeatedly
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn goto_next_sibling_at_end_stays_false(syms in prop::collection::vec(1u16..500, 1..=10)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        walker.goto_first_child();
        while walker.goto_next_sibling() {}
        // At end - repeated calls should stay false
        prop_assert!(!walker.goto_next_sibling());
        prop_assert!(!walker.goto_next_sibling());
        prop_assert!(!walker.goto_next_sibling());
        // Still at last child
        prop_assert_eq!(walker.node().symbol(), *syms.last().unwrap());
    }
}

// ===================================================================
// 17. Walker end_byte order is non-decreasing
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_end_bytes_non_decreasing(count in 1usize..=15) {
        let kids: Vec<ParsedNode> = (0..count)
            .map(|i| leaf(i as u16 + 1, i * 2, i * 2 + 2))
            .collect();
        let total = kids.last().unwrap().end_byte;
        let parent = branch(999, 0, total, kids);

        let mut walker = parent.walk();
        let mut ends = Vec::new();
        if walker.goto_first_child() {
            ends.push(walker.node().end_byte());
            while walker.goto_next_sibling() {
                ends.push(walker.node().end_byte());
            }
        }
        for i in 1..ends.len() {
            prop_assert!(ends[i] >= ends[i - 1]);
        }
    }
}

// ===================================================================
// 18. Walker on single child: no sibling
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn single_child_no_sibling(sym in 1u16..500) {
        let parent = branch(999, 0, 5, vec![leaf(sym, 0, 5)]);
        let mut walker = parent.walk();
        prop_assert!(walker.goto_first_child());
        prop_assert_eq!(walker.node().symbol(), sym);
        prop_assert!(!walker.goto_next_sibling());
    }
}

// ===================================================================
// 19. Walker cloned node independent of advancement
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn cloned_node_independent_of_advancement(syms in prop::collection::vec(1u16..500, 2..=8)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        walker.goto_first_child();
        let first_clone = walker.node().clone();
        walker.goto_next_sibling();
        // Clone still has original values
        prop_assert_eq!(first_clone.symbol(), syms[0]);
        prop_assert_eq!(walker.node().symbol(), syms[1]);
    }
}

// ===================================================================
// 20. Walker collects all is_error flags correctly
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_collects_error_flags(err_flags in prop::collection::vec(any::<bool>(), 1..=12)) {
        let kids: Vec<ParsedNode> = err_flags.iter().enumerate().map(|(i, &is_err)| {
            make_node(
                i as u16 + 1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, is_err, false, true, None,
            )
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        let mut walked_errs = Vec::new();
        if walker.goto_first_child() {
            walked_errs.push(walker.node().is_error());
            while walker.goto_next_sibling() {
                walked_errs.push(walker.node().is_error());
            }
        }
        prop_assert_eq!(walked_errs, err_flags);
    }
}

// ===================================================================
// 21. Walker collects is_extra flags correctly
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_collects_extra_flags(extra_flags in prop::collection::vec(any::<bool>(), 1..=12)) {
        let kids: Vec<ParsedNode> = extra_flags.iter().enumerate().map(|(i, &is_extra)| {
            make_node(
                i as u16 + 1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                is_extra, false, false, !is_extra, None,
            )
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        let mut walked_extras = Vec::new();
        if walker.goto_first_child() {
            walked_extras.push(walker.node().is_extra());
            while walker.goto_next_sibling() {
                walked_extras.push(walker.node().is_extra());
            }
        }
        prop_assert_eq!(walked_extras, extra_flags);
    }
}

// ===================================================================
// 22. Walker collects is_missing flags correctly
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_collects_missing_flags(miss_flags in prop::collection::vec(any::<bool>(), 1..=12)) {
        let kids: Vec<ParsedNode> = miss_flags.iter().enumerate().map(|(i, &is_miss)| {
            make_node(
                i as u16 + 1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, false, is_miss, true, None,
            )
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let mut walker = parent.walk();
        let mut walked = Vec::new();
        if walker.goto_first_child() {
            walked.push(walker.node().is_missing());
            while walker.goto_next_sibling() {
                walked.push(walker.node().is_missing());
            }
        }
        prop_assert_eq!(walked, miss_flags);
    }
}

// ===================================================================
// 23. Walker on nested node: inner walk is independent
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn nested_walk_independent(
        outer_syms in prop::collection::vec(1u16..100, 2..=5),
        inner_syms in prop::collection::vec(100u16..200, 1..=5),
    ) {
        let inner_kids: Vec<ParsedNode> = inner_syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let inner = branch(50, 0, inner_kids.len(), inner_kids);

        let mut outer_kids: Vec<ParsedNode> = outer_syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i + inner_syms.len(), i + inner_syms.len() + 1))
            .collect();
        outer_kids.insert(0, inner);
        let root = branch(999, 0, 100, outer_kids);

        // Walk outer
        let mut outer_walker = root.walk();
        outer_walker.goto_first_child();
        // First child is the inner branch node
        let inner_node = outer_walker.node();
        prop_assert_eq!(inner_node.symbol(), 50);

        // Walk inner independently
        let inner_walked = walk_symbols(inner_node);
        prop_assert_eq!(inner_walked, inner_syms);

        // Outer walker still at first child
        prop_assert_eq!(outer_walker.node().symbol(), 50);
    }
}

// ===================================================================
// 24. Walker byte ranges: each child's start_byte == child(i).start_byte
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_byte_ranges_match_indexed_access(count in 1usize..=15) {
        let kids: Vec<ParsedNode> = (0..count)
            .map(|i| leaf(i as u16 + 1, i * 4, i * 4 + 3))
            .collect();
        let parent = branch(999, 0, count * 4, kids);

        let mut walker = parent.walk();
        if walker.goto_first_child() {
            let mut idx = 0;
            prop_assert_eq!(walker.node().start_byte(), parent.child(idx).unwrap().start_byte());
            prop_assert_eq!(walker.node().end_byte(), parent.child(idx).unwrap().end_byte());
            idx += 1;
            while walker.goto_next_sibling() {
                prop_assert_eq!(walker.node().start_byte(), parent.child(idx).unwrap().start_byte());
                prop_assert_eq!(walker.node().end_byte(), parent.child(idx).unwrap().end_byte());
                idx += 1;
            }
            prop_assert_eq!(idx, count);
        }
    }
}

// ===================================================================
// 25. Walker Point data matches direct child access
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_points_match_direct_access(count in 1usize..=10) {
        let kids: Vec<ParsedNode> = (0..count).map(|i| {
            make_node(
                i as u16 + 1, vec![], i * 5, i * 5 + 4,
                pt(i as u32, 0), pt(i as u32, 4),
                false, false, false, true, None,
            )
        }).collect();
        let parent = branch(999, 0, count * 5, kids);

        let mut walker = parent.walk();
        if walker.goto_first_child() {
            let mut idx = 0;
            prop_assert_eq!(walker.node().start_point(), parent.child(idx).unwrap().start_point());
            prop_assert_eq!(walker.node().end_point(), parent.child(idx).unwrap().end_point());
            idx += 1;
            while walker.goto_next_sibling() {
                prop_assert_eq!(walker.node().start_point(), parent.child(idx).unwrap().start_point());
                prop_assert_eq!(walker.node().end_point(), parent.child(idx).unwrap().end_point());
                idx += 1;
            }
        }
    }
}

// ===================================================================
// 26. Walker has_error reflects children
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn has_error_reflects_any_error_child(err_flags in prop::collection::vec(any::<bool>(), 1..=10)) {
        let kids: Vec<ParsedNode> = err_flags.iter().enumerate().map(|(i, &is_err)| {
            make_node(
                i as u16 + 1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, is_err, false, true, None,
            )
        }).collect();
        let parent = branch(999, 0, kids.len(), kids);

        let any_err = err_flags.iter().any(|&e| e);
        prop_assert_eq!(parent.has_error(), any_err);
    }
}

// ===================================================================
// 27. Walker debug output contains "ParsedNode"
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn walker_node_debug_contains_parsed_node(sym in 1u16..500) {
        let parent = branch(999, 0, 5, vec![leaf(sym, 0, 5)]);
        let mut walker = parent.walk();
        walker.goto_first_child();
        let dbg = format!("{:?}", walker.node());
        prop_assert!(dbg.contains("ParsedNode"));
    }
}

// ===================================================================
// 28. Walker traversal count equals children().len()
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn walker_traversal_count_equals_children_len(n in 0usize..=20) {
        let kids: Vec<ParsedNode> = (0..n)
            .map(|i| leaf(i as u16 + 1, i, i + 1))
            .collect();
        let parent = branch(999, 0, n, kids);

        let mut walker = parent.walk();
        let mut count = 0;
        if walker.goto_first_child() {
            count += 1;
            while walker.goto_next_sibling() {
                count += 1;
            }
        }
        prop_assert_eq!(count, parent.children().len());
    }
}

// ===================================================================
// 29. Multiple independent walkers on same node have same length
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn three_walkers_same_count(syms in prop::collection::vec(1u16..500, 0..=15)) {
        let kids: Vec<ParsedNode> = syms.iter().enumerate()
            .map(|(i, &s)| leaf(s, i, i + 1))
            .collect();
        let parent = branch(999, 0, kids.len(), kids);

        let c1 = walk_symbols(&parent).len();
        let c2 = walk_symbols(&parent).len();
        let c3 = walk_symbols(&parent).len();
        prop_assert_eq!(c1, c2);
        prop_assert_eq!(c2, c3);
    }
}

// ===================================================================
// 30. Walker with all fields set: combined flags round-trip
// ===================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn walker_combined_flags_roundtrip(
        is_named_flags in prop::collection::vec(any::<bool>(), 1..=8),
        is_error_flags in prop::collection::vec(any::<bool>(), 1..=8),
    ) {
        let len = is_named_flags.len().min(is_error_flags.len());
        let kids: Vec<ParsedNode> = (0..len).map(|i| {
            make_node(
                i as u16 + 1, vec![], i, i + 1,
                pt(0, i as u32), pt(0, (i + 1) as u32),
                false, is_error_flags[i], false, is_named_flags[i], None,
            )
        }).collect();
        let parent = branch(999, 0, len, kids);

        let mut walker = parent.walk();
        let mut idx = 0;
        if walker.goto_first_child() {
            prop_assert_eq!(walker.node().is_named(), is_named_flags[idx]);
            prop_assert_eq!(walker.node().is_error(), is_error_flags[idx]);
            idx += 1;
            while walker.goto_next_sibling() {
                prop_assert_eq!(walker.node().is_named(), is_named_flags[idx]);
                prop_assert_eq!(walker.node().is_error(), is_error_flags[idx]);
                idx += 1;
            }
        }
        prop_assert_eq!(idx, len);
    }
}
