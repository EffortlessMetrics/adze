//! Tests for the parse forest error types.

use adze_glr_core::parse_forest::{ErrorMeta, ParseError};

#[test]
fn error_meta_default_values() {
    let meta = ErrorMeta {
        missing: false,
        is_error: false,
        cost: 0,
    };
    assert!(!meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 0);
}

#[test]
fn error_meta_missing_terminal() {
    let meta = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 1,
    };
    assert!(meta.missing);
    assert_eq!(meta.cost, 1);
}

#[test]
fn error_meta_error_node() {
    let meta = ErrorMeta {
        missing: false,
        is_error: true,
        cost: 5,
    };
    assert!(meta.is_error);
    assert_eq!(meta.cost, 5);
}

#[test]
fn error_meta_debug() {
    let meta = ErrorMeta {
        missing: true,
        is_error: true,
        cost: 3,
    };
    let debug = format!("{meta:?}");
    assert!(debug.contains("ErrorMeta"));
}

#[test]
fn error_meta_clone() {
    let meta = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 10,
    };
    let cloned = meta;
    assert_eq!(meta.missing, cloned.missing);
    assert_eq!(meta.cost, cloned.cost);
}

#[test]
fn parse_error_incomplete() {
    let err = ParseError::Incomplete;
    let msg = format!("{err}");
    assert!(msg.contains("Incomplete"));
}

#[test]
fn parse_error_failed() {
    let err = ParseError::Failed("unexpected token".into());
    let msg = format!("{err}");
    assert!(msg.contains("unexpected token"));
}

#[test]
fn parse_error_unknown() {
    let err = ParseError::Unknown;
    let msg = format!("{err}");
    assert!(msg.contains("Unknown"));
}

#[test]
fn parse_error_debug() {
    let err = ParseError::Failed("test".into());
    let debug = format!("{err:?}");
    assert!(debug.contains("Failed"));
}

#[test]
fn parse_error_clone() {
    let err = ParseError::Incomplete;
    let cloned = err.clone();
    let msg1 = format!("{err}");
    let msg2 = format!("{cloned}");
    assert_eq!(msg1, msg2);
}
