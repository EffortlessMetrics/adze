use rust_sitter_glr_core::stack::{GlrStack, StackNode};

#[test]
fn vec_impl_peek_matches_slice() {
    let v = vec![1u16, 2, 3];
    // Trait method:
    let t = <Vec<u16> as GlrStack>::peek(&v);
    // Slice method:
    let s = v.as_slice().last().copied();
    assert_eq!(t, s);
    assert_eq!(t, Some(3));
}

#[test]
fn stacknode_top_falls_back_to_state() {
    // Empty head -> returns state
    let s = StackNode {
        state: 42,
        symbol: None,
        head: Vec::new(),
        tail: None,
    };
    assert_eq!(s.top(), Some(42));
}

#[test]
fn stacknode_top_reads_head() {
    let s = StackNode {
        state: 5,
        symbol: None,
        head: vec![7, 11], // 7 is state, 11 is symbol (or NO_SYM)
        tail: None,
    };
    assert_eq!(s.top(), Some(7)); // top() returns the state, not the symbol
    // GlrStack impl calls .top() for peek()
    assert_eq!(<StackNode as GlrStack>::peek(&s), Some(7));
    // Length delegates to depth()
    assert!(<StackNode as GlrStack>::len(&s) >= 2);
}

#[test]
fn vec_glrstack_operations() {
    let mut v = vec![10u16, 20];

    // Test push
    <Vec<u16> as GlrStack>::push(&mut v, 30);
    assert_eq!(v, vec![10, 20, 30]);

    // Test peek
    assert_eq!(<Vec<u16> as GlrStack>::peek(&v), Some(30));

    // Test pop
    assert_eq!(<Vec<u16> as GlrStack>::pop(&mut v), Some(30));
    assert_eq!(v, vec![10, 20]);

    // Test len and is_empty
    assert_eq!(<Vec<u16> as GlrStack>::len(&v), 2);
    assert!(!<Vec<u16> as GlrStack>::is_empty(&v));

    v.clear();
    assert!(<Vec<u16> as GlrStack>::is_empty(&v));
}
