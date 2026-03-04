#![allow(clippy::needless_range_loop)]

use adze_runtime::Token;

// === 1. Token creation with kind/start/end ===

#[test]
fn create_token_basic() {
    let t = Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    assert_eq!(t.kind, 1);
    assert_eq!(t.start, 0);
    assert_eq!(t.end, 5);
}

#[test]
fn create_token_different_values() {
    let t = Token {
        kind: 42,
        start: 100,
        end: 200,
    };
    assert_eq!(t.kind, 42);
    assert_eq!(t.start, 100);
    assert_eq!(t.end, 200);
}

#[test]
fn create_token_all_zeros() {
    let t = Token {
        kind: 0,
        start: 0,
        end: 0,
    };
    assert_eq!(t.kind, 0);
    assert_eq!(t.start, 0);
    assert_eq!(t.end, 0);
}

// === 2. Token Copy semantics ===

#[test]
fn copy_token_preserves_fields() {
    let t1 = Token {
        kind: 7,
        start: 10,
        end: 20,
    };
    let t2 = t1; // Copy
    // Both are usable after copy
    assert_eq!(t1.kind, t2.kind);
    assert_eq!(t1.start, t2.start);
    assert_eq!(t1.end, t2.end);
}

#[test]
fn copy_token_is_independent() {
    let t1 = Token {
        kind: 3,
        start: 5,
        end: 15,
    };
    let mut t2 = t1;
    t2.kind = 99;
    // Original unchanged after mutating the copy
    assert_eq!(t1.kind, 3);
    assert_eq!(t2.kind, 99);
}

#[test]
fn copy_into_function_arg() {
    fn consume(t: Token) -> u32 {
        t.kind
    }
    let t = Token {
        kind: 11,
        start: 0,
        end: 1,
    };
    let k = consume(t);
    // t still accessible because Token is Copy
    assert_eq!(k, 11);
    assert_eq!(t.kind, 11);
}

// === 3. Token Clone behavior ===

#[test]
fn clone_equals_original() {
    let t = Token {
        kind: 5,
        start: 50,
        end: 100,
    };
    let cloned = t;
    assert_eq!(t, cloned);
}

#[test]
fn clone_is_independent() {
    let t = Token {
        kind: 8,
        start: 30,
        end: 40,
    };
    let mut cloned = t;
    cloned.start = 999;
    assert_eq!(t.start, 30);
    assert_eq!(cloned.start, 999);
}

#[test]
fn clone_multiple_times() {
    let t = Token {
        kind: 2,
        start: 1,
        end: 3,
    };
    let c1 = t;
    let c2 = t;
    let c3 = c1;
    assert_eq!(t, c1);
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}

// === 4. Token Debug display ===

#[test]
fn debug_format_contains_kind() {
    let t = Token {
        kind: 42,
        start: 0,
        end: 10,
    };
    let dbg = format!("{:?}", t);
    assert!(dbg.contains("42"), "Debug should contain kind value: {dbg}");
}

#[test]
fn debug_format_contains_start_end() {
    let t = Token {
        kind: 1,
        start: 100,
        end: 200,
    };
    let dbg = format!("{:?}", t);
    assert!(dbg.contains("100"), "Debug should contain start: {dbg}");
    assert!(dbg.contains("200"), "Debug should contain end: {dbg}");
}

#[test]
fn debug_format_contains_struct_name() {
    let t = Token {
        kind: 0,
        start: 0,
        end: 0,
    };
    let dbg = format!("{:?}", t);
    assert!(
        dbg.contains("Token"),
        "Debug should contain struct name: {dbg}"
    );
}

#[test]
fn debug_format_is_nonempty() {
    let t = Token {
        kind: 1,
        start: 2,
        end: 3,
    };
    let dbg = format!("{:?}", t);
    assert!(!dbg.is_empty());
}

// === 5. Token PartialEq/Eq ===

#[test]
fn equal_tokens_are_eq() {
    let a = Token {
        kind: 5,
        start: 10,
        end: 20,
    };
    let b = Token {
        kind: 5,
        start: 10,
        end: 20,
    };
    assert_eq!(a, b);
}

#[test]
fn different_kind_not_eq() {
    let a = Token {
        kind: 1,
        start: 10,
        end: 20,
    };
    let b = Token {
        kind: 2,
        start: 10,
        end: 20,
    };
    assert_ne!(a, b);
}

#[test]
fn different_start_not_eq() {
    let a = Token {
        kind: 1,
        start: 10,
        end: 20,
    };
    let b = Token {
        kind: 1,
        start: 11,
        end: 20,
    };
    assert_ne!(a, b);
}

#[test]
fn different_end_not_eq() {
    let a = Token {
        kind: 1,
        start: 10,
        end: 20,
    };
    let b = Token {
        kind: 1,
        start: 10,
        end: 21,
    };
    assert_ne!(a, b);
}

#[test]
fn eq_is_reflexive() {
    let t = Token {
        kind: 9,
        start: 0,
        end: 50,
    };
    assert_eq!(t, t);
}

#[test]
fn eq_is_symmetric() {
    let a = Token {
        kind: 4,
        start: 3,
        end: 7,
    };
    let b = Token {
        kind: 4,
        start: 3,
        end: 7,
    };
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn eq_is_transitive() {
    let a = Token {
        kind: 6,
        start: 1,
        end: 2,
    };
    let b = Token {
        kind: 6,
        start: 1,
        end: 2,
    };
    let c = Token {
        kind: 6,
        start: 1,
        end: 2,
    };
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

// === 6. Token field access ===

#[test]
fn field_access_kind() {
    let t = Token {
        kind: 255,
        start: 0,
        end: 1,
    };
    let k: u32 = t.kind;
    assert_eq!(k, 255);
}

#[test]
fn field_access_start() {
    let t = Token {
        kind: 0,
        start: 1024,
        end: 2048,
    };
    let s: u32 = t.start;
    assert_eq!(s, 1024);
}

#[test]
fn field_access_end() {
    let t = Token {
        kind: 0,
        start: 0,
        end: 65535,
    };
    let e: u32 = t.end;
    assert_eq!(e, 65535);
}

#[test]
fn token_span_length() {
    let t = Token {
        kind: 1,
        start: 10,
        end: 30,
    };
    assert_eq!(t.end - t.start, 20);
}

// === 7. Token zero-length (start == end) ===

#[test]
fn zero_length_token_at_origin() {
    let t = Token {
        kind: 1,
        start: 0,
        end: 0,
    };
    assert_eq!(t.start, t.end);
    assert_eq!(t.end - t.start, 0);
}

#[test]
fn zero_length_token_at_offset() {
    let t = Token {
        kind: 5,
        start: 500,
        end: 500,
    };
    assert_eq!(t.start, t.end);
}

#[test]
fn zero_length_tokens_different_positions_not_eq() {
    let a = Token {
        kind: 1,
        start: 0,
        end: 0,
    };
    let b = Token {
        kind: 1,
        start: 10,
        end: 10,
    };
    assert_ne!(a, b);
}

#[test]
fn zero_length_tokens_same_position_eq() {
    let a = Token {
        kind: 3,
        start: 42,
        end: 42,
    };
    let b = Token {
        kind: 3,
        start: 42,
        end: 42,
    };
    assert_eq!(a, b);
}

// === 8. Token with large values (u32 max boundary) ===

#[test]
fn token_max_kind() {
    let t = Token {
        kind: u32::MAX,
        start: 0,
        end: 1,
    };
    assert_eq!(t.kind, u32::MAX);
}

#[test]
fn token_max_start_and_end() {
    let t = Token {
        kind: 0,
        start: u32::MAX,
        end: u32::MAX,
    };
    assert_eq!(t.start, u32::MAX);
    assert_eq!(t.end, u32::MAX);
}

#[test]
fn token_all_fields_max() {
    let t = Token {
        kind: u32::MAX,
        start: u32::MAX,
        end: u32::MAX,
    };
    assert_eq!(t.kind, u32::MAX);
    assert_eq!(t.start, u32::MAX);
    assert_eq!(t.end, u32::MAX);
}

#[test]
fn token_max_boundary_debug() {
    let t = Token {
        kind: u32::MAX,
        start: u32::MAX,
        end: u32::MAX,
    };
    let dbg = format!("{:?}", t);
    assert!(
        dbg.contains(&u32::MAX.to_string()),
        "Debug should show max value: {dbg}"
    );
}

#[test]
fn token_max_boundary_eq() {
    let a = Token {
        kind: u32::MAX,
        start: u32::MAX,
        end: u32::MAX,
    };
    let b = Token {
        kind: u32::MAX,
        start: u32::MAX,
        end: u32::MAX,
    };
    assert_eq!(a, b);
}

#[test]
fn token_near_max_boundary() {
    let t = Token {
        kind: u32::MAX - 1,
        start: u32::MAX - 1,
        end: u32::MAX,
    };
    assert_eq!(t.kind, u32::MAX - 1);
    assert_eq!(t.end - t.start, 1);
}
