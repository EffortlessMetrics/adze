//! Comprehensive tests for the GLR parser stack module (`StackNode`).

use adze_glr_core::stack::StackNode;

// ---------------------------------------------------------------------------
// 1. Stack creation and initial state
// ---------------------------------------------------------------------------

#[test]
fn new_stack_is_empty() {
    let stack = StackNode::new();
    assert!(stack.is_empty());
    assert_eq!(stack.depth(), 0);
    assert_eq!(stack.top(), None);
    assert_eq!(stack.to_vec(), Vec::<u16>::new());
}

#[test]
fn default_is_same_as_new() {
    let a = StackNode::new();
    let b = StackNode::default();
    assert_eq!(a.depth(), b.depth());
    assert_eq!(a.is_empty(), b.is_empty());
    assert_eq!(a.top(), b.top());
    assert_eq!(a.to_vec(), b.to_vec());
}

#[test]
fn with_state_creates_non_empty_stack() {
    let stack = StackNode::with_state(42);
    assert!(!stack.is_empty());
    assert_eq!(stack.depth(), 1);
    assert_eq!(stack.top(), Some(42));
    assert_eq!(stack.to_vec(), vec![42]);
}

#[test]
fn with_state_zero_is_empty() {
    // State 0 is the BASE_EMPTY_STATE sentinel, so treated as empty.
    let stack = StackNode::with_state(0);
    assert!(stack.is_empty());
    assert_eq!(stack.depth(), 0);
    assert_eq!(stack.top(), None);
}

// ---------------------------------------------------------------------------
// 2. Push / pop operations
// ---------------------------------------------------------------------------

#[test]
fn push_then_pop_single() {
    let mut stack = StackNode::new();
    stack.push(10, None);
    assert_eq!(stack.depth(), 1);
    assert_eq!(stack.top(), Some(10));
    assert_eq!(stack.pop(), Some((10, None)));
    assert!(stack.is_empty());
}

#[test]
fn push_pop_preserves_lifo_order() {
    let mut stack = StackNode::new();
    for i in 1..=5 {
        stack.push(i, None);
    }
    for i in (1..=5).rev() {
        assert_eq!(stack.pop(), Some((i, None)));
    }
    assert!(stack.is_empty());
}

#[test]
fn push_with_symbol_roundtrips() {
    let mut stack = StackNode::new();
    stack.push(1, Some(100));
    stack.push(2, Some(200));
    stack.push(3, None);

    assert_eq!(stack.pop(), Some((3, None)));
    assert_eq!(stack.pop(), Some((2, Some(200))));
    assert_eq!(stack.pop(), Some((1, Some(100))));
    assert_eq!(stack.pop(), None);
}

#[test]
fn pop_on_empty_returns_none() {
    let mut stack = StackNode::new();
    assert_eq!(stack.pop(), None);
}

#[test]
fn pop_on_empty_repeated_returns_none() {
    let mut stack = StackNode::new();
    assert_eq!(stack.pop(), None);
    assert_eq!(stack.pop(), None);
    assert_eq!(stack.pop(), None);
}

#[test]
fn pop_drains_with_state_initial() {
    let mut stack = StackNode::with_state(5);
    stack.push(10, None);
    assert_eq!(stack.pop(), Some((10, None)));
    assert_eq!(stack.pop(), Some((5, None)));
    assert_eq!(stack.pop(), None);
}

#[test]
fn push_after_full_drain() {
    let mut stack = StackNode::new();
    stack.push(1, None);
    stack.pop();
    assert!(stack.is_empty());
    stack.push(2, None);
    assert_eq!(stack.depth(), 1);
    assert_eq!(stack.top(), Some(2));
}

// ---------------------------------------------------------------------------
// 3. Fork operations
// ---------------------------------------------------------------------------

#[test]
fn fork_produces_equal_stack() {
    let mut orig = StackNode::with_state(1);
    orig.push(2, Some(50));
    orig.push(3, None);

    let forked = orig.fork();
    assert_eq!(orig.to_vec(), forked.to_vec());
    assert_eq!(orig.depth(), forked.depth());
    assert_eq!(orig.top(), forked.top());
}

#[test]
fn fork_is_independent_push() {
    let mut a = StackNode::with_state(1);
    a.push(2, None);

    let mut b = a.fork();
    b.push(99, None);

    assert_eq!(a.to_vec(), vec![1, 2]);
    assert_eq!(b.to_vec(), vec![1, 2, 99]);
}

#[test]
fn fork_is_independent_pop() {
    let mut a = StackNode::with_state(1);
    a.push(2, None);
    a.push(3, None);

    let mut b = a.fork();
    b.pop();

    assert_eq!(a.to_vec(), vec![1, 2, 3]);
    assert_eq!(b.to_vec(), vec![1, 2]);
}

#[test]
fn multiple_forks_are_independent() {
    let mut root = StackNode::with_state(1);
    root.push(2, None);

    let mut f1 = root.fork();
    let mut f2 = root.fork();
    let mut f3 = root.fork();

    f1.push(10, None);
    f2.push(20, None);
    f3.push(30, None);

    assert_eq!(f1.to_vec(), vec![1, 2, 10]);
    assert_eq!(f2.to_vec(), vec![1, 2, 20]);
    assert_eq!(f3.to_vec(), vec![1, 2, 30]);
    // Original unchanged
    assert_eq!(root.to_vec(), vec![1, 2]);
}

#[test]
fn fork_of_fork() {
    let mut a = StackNode::with_state(1);
    a.push(2, None);

    let mut b = a.fork();
    b.push(3, None);

    let mut c = b.fork();
    c.push(4, None);

    assert_eq!(a.to_vec(), vec![1, 2]);
    assert_eq!(b.to_vec(), vec![1, 2, 3]);
    assert_eq!(c.to_vec(), vec![1, 2, 3, 4]);
}

#[test]
fn fork_of_empty_stack() {
    let empty = StackNode::new();
    let forked = empty.fork();
    assert!(forked.is_empty());
    assert_eq!(forked.depth(), 0);
}

// ---------------------------------------------------------------------------
// 4. Stack state queries
// ---------------------------------------------------------------------------

#[test]
fn top_returns_most_recent_push() {
    let mut stack = StackNode::new();
    stack.push(5, None);
    assert_eq!(stack.top(), Some(5));
    stack.push(10, None);
    assert_eq!(stack.top(), Some(10));
}

#[test]
fn top_returns_state_not_symbol() {
    let mut stack = StackNode::new();
    stack.push(7, Some(200));
    assert_eq!(stack.top(), Some(7));
}

#[test]
fn last_is_alias_for_top() {
    let mut stack = StackNode::with_state(3);
    stack.push(8, None);
    assert_eq!(stack.last(), stack.top());
}

#[test]
fn depth_counts_states_correctly() {
    let mut stack = StackNode::with_state(1);
    assert_eq!(stack.depth(), 1);
    stack.push(2, None);
    assert_eq!(stack.depth(), 2);
    stack.push(3, Some(50));
    assert_eq!(stack.depth(), 3);
    stack.pop();
    assert_eq!(stack.depth(), 2);
}

#[test]
fn to_vec_returns_bottom_to_top_order() {
    let mut stack = StackNode::with_state(1);
    stack.push(2, None);
    stack.push(3, None);
    assert_eq!(stack.to_vec(), vec![1, 2, 3]);
}

#[test]
fn to_vec_excludes_symbols() {
    let mut stack = StackNode::new();
    stack.push(10, Some(100));
    stack.push(20, Some(200));
    // Only states should appear
    assert_eq!(stack.to_vec(), vec![10, 20]);
}

#[test]
fn can_merge_with_identical_stacks() {
    let mut a = StackNode::with_state(1);
    a.push(2, None);
    let b = a.fork();
    assert!(a.can_merge_with(&b));
}

#[test]
fn can_merge_with_same_depth_and_top() {
    let mut a = StackNode::new();
    a.push(5, None);
    a.push(10, None);

    let mut b = StackNode::new();
    b.push(99, None);
    b.push(10, None);

    // Same depth (2) and same top (10) ⇒ can merge
    assert!(a.can_merge_with(&b));
}

#[test]
fn cannot_merge_different_depth() {
    let mut a = StackNode::new();
    a.push(1, None);

    let mut b = StackNode::new();
    b.push(1, None);
    b.push(2, None);

    assert!(!a.can_merge_with(&b));
}

#[test]
fn cannot_merge_different_top() {
    let mut a = StackNode::new();
    a.push(1, None);

    let mut b = StackNode::new();
    b.push(2, None);

    assert!(!a.can_merge_with(&b));
}

// ---------------------------------------------------------------------------
// 5. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn deep_stack_push_pop_roundtrip() {
    let mut stack = StackNode::new();
    let n = 100;
    for i in 0..n {
        stack.push(i, None);
    }
    assert_eq!(stack.depth(), n as usize);

    for i in (0..n).rev() {
        assert_eq!(stack.pop(), Some((i, None)));
    }
    assert!(stack.is_empty());
}

#[test]
fn deep_stack_to_vec() {
    let mut stack = StackNode::new();
    let expected: Vec<u16> = (1..=50).collect();
    for &s in &expected {
        stack.push(s, None);
    }
    assert_eq!(stack.to_vec(), expected);
}

#[test]
fn deep_stack_fork_independence() {
    let mut stack = StackNode::new();
    for i in 1..=20 {
        stack.push(i, None);
    }
    let mut forked = stack.fork();
    forked.push(999, None);

    assert_eq!(stack.depth(), 20);
    assert_eq!(forked.depth(), 21);
    assert_eq!(forked.top(), Some(999));
    assert_eq!(stack.top(), Some(20));
}

#[test]
fn assert_well_formed_on_valid_stack() {
    let mut stack = StackNode::with_state(1);
    stack.push(2, Some(10));
    stack.push(3, None);
    // Should not panic
    stack.assert_well_formed();
}

#[test]
fn assert_well_formed_on_empty() {
    let stack = StackNode::new();
    stack.assert_well_formed();
}

#[test]
fn interleaved_push_pop() {
    let mut stack = StackNode::new();
    stack.push(1, None);
    stack.push(2, None);
    assert_eq!(stack.pop(), Some((2, None)));
    stack.push(3, None);
    stack.push(4, None);
    assert_eq!(stack.pop(), Some((4, None)));
    assert_eq!(stack.pop(), Some((3, None)));
    assert_eq!(stack.pop(), Some((1, None)));
    assert!(stack.is_empty());
}

#[test]
fn with_state_pop_returns_initial_state() {
    let mut stack = StackNode::with_state(7);
    assert_eq!(stack.pop(), Some((7, None)));
    assert!(stack.is_empty());
    assert_eq!(stack.pop(), None);
}
