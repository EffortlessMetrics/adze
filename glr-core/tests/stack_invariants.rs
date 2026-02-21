// Stack invariant tests to ensure correctness of the persistent stack implementation
use adze_glr_core::stack::StackNode;

#[test]
fn depth_equals_states_len() {
    let mut s = StackNode::new();
    s.push(1, None);
    s.push(2, Some(7));
    s.push(3, None);

    let depth = s.depth();
    let vec_len = s.to_vec().len();

    assert_eq!(depth, vec_len, "depth() should equal to_vec().len()");
    assert_eq!(depth, 3, "Should have 3 states");
}

#[test]
fn top_is_last_state_in_to_vec() {
    let mut s = StackNode::new();
    s.push(1, None);
    s.push(2, Some(9));
    s.push(3, None);

    let vec = s.to_vec();
    let top = s.top();
    let last = vec.last().copied();

    assert_eq!(top, last, "top() should equal to_vec().last()");
    assert_eq!(top, Some(3), "Top should be 3");
}

#[test]
fn push_pop_round_trip() {
    let mut s = StackNode::new();

    // Push various state/symbol combinations
    s.push(1, None);
    s.push(2, Some(100));
    s.push(3, Some(200));
    s.push(4, None);
    s.push(5, Some(300));

    // Pop and verify we get back what we pushed
    assert_eq!(s.pop(), Some((5, Some(300))));
    assert_eq!(s.pop(), Some((4, None)));
    assert_eq!(s.pop(), Some((3, Some(200))));
    assert_eq!(s.pop(), Some((2, Some(100))));
    assert_eq!(s.pop(), Some((1, None)));
    assert_eq!(s.pop(), None);
}

#[test]
fn fork_preserves_invariants() {
    let mut s1 = StackNode::new();
    s1.push(1, None);
    s1.push(2, Some(10));

    let mut s2 = s1.fork();

    // Both stacks should have the same state
    assert_eq!(s1.depth(), s2.depth());
    assert_eq!(s1.to_vec(), s2.to_vec());
    assert_eq!(s1.top(), s2.top());

    // Diverge the forks
    s1.push(3, None);
    s2.push(4, Some(20));

    // Now they should differ
    assert_ne!(s1.to_vec(), s2.to_vec());
    assert_ne!(s1.top(), s2.top());

    // But both should maintain invariants
    assert_eq!(s1.depth(), s1.to_vec().len());
    assert_eq!(s2.depth(), s2.to_vec().len());
}

#[test]
fn empty_stack_invariants() {
    let s = StackNode::new();

    assert!(s.is_empty());
    assert_eq!(s.depth(), 0);
    assert_eq!(s.to_vec().len(), 0);
    assert_eq!(s.top(), None);
}

#[test]
fn with_state_constructor() {
    let s = StackNode::with_state(42);

    assert!(!s.is_empty());
    assert_eq!(s.depth(), 1);
    assert_eq!(s.to_vec(), vec![42]);
    assert_eq!(s.top(), Some(42));
}

#[test]
fn large_stack_maintains_invariants() {
    let mut s = StackNode::new();

    // Push many states to trigger spilling
    for i in 0..20 {
        s.push(i, if i % 2 == 0 { None } else { Some(i * 100) });
    }

    assert_eq!(s.depth(), 20);
    assert_eq!(s.to_vec().len(), 20);
    assert_eq!(s.top(), Some(19));

    // Pop half
    for _ in 0..10 {
        s.pop();
    }

    assert_eq!(s.depth(), 10);
    assert_eq!(s.to_vec().len(), 10);
    assert_eq!(s.top(), Some(9));
}

#[test]
fn well_formedness_check() {
    let mut s = StackNode::new();

    // This should not panic
    s.assert_well_formed();

    s.push(1, None);
    s.assert_well_formed();

    s.push(2, Some(100));
    s.assert_well_formed();

    // Push enough to cause spilling
    for i in 3..15 {
        s.push(i, None);
        s.assert_well_formed();
    }
}
