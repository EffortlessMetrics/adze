//! Comprehensive tests for the Token type.

use adze_runtime::Token;

#[test]
fn token_fields() {
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
fn token_zero_length() {
    let t = Token {
        kind: 0,
        start: 10,
        end: 10,
    };
    assert_eq!(t.start, t.end);
}

#[test]
fn token_debug() {
    let t = Token {
        kind: 42,
        start: 1,
        end: 3,
    };
    let debug = format!("{:?}", t);
    assert!(debug.contains("42"));
}

#[test]
fn token_clone() {
    let t = Token {
        kind: 5,
        start: 0,
        end: 10,
    };
    let t2 = t;
    assert_eq!(t.kind, t2.kind);
    assert_eq!(t.start, t2.start);
    assert_eq!(t.end, t2.end);
}

#[test]
fn token_large_offsets() {
    let t = Token {
        kind: u32::MAX,
        start: u32::MAX - 1,
        end: u32::MAX,
    };
    assert_eq!(t.kind, u32::MAX);
}

#[test]
fn token_vec() {
    let tokens: Vec<Token> = (0..100)
        .map(|i| Token {
            kind: i % 5,
            start: i * 2,
            end: i * 2 + 1,
        })
        .collect();
    assert_eq!(tokens.len(), 100);
    assert_eq!(tokens[50].kind, 0);
    assert_eq!(tokens[50].start, 100);
}

#[test]
fn token_is_copy() {
    let t1 = Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    let t2 = t1; // Copy
    let t3 = t1; // Still valid because Copy
    assert_eq!(t2.kind, t3.kind);
}
