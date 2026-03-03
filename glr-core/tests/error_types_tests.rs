//! Tests for GLR error types.

use adze_glr_core::{GLRError, TableError};

#[test]
fn glr_error_conflict_resolution_display() {
    let err = GLRError::ConflictResolution("test conflict".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("Conflict resolution failed"));
    assert!(msg.contains("test conflict"));
}

#[test]
fn glr_error_state_machine_display() {
    let err = GLRError::StateMachine("build failed".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("State machine generation failed"));
}

#[test]
fn glr_error_complex_symbols_display() {
    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "FIRST computation".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("normalized"));
    assert!(msg.contains("FIRST computation"));
}

#[test]
fn glr_error_expected_simple_display() {
    let err = GLRError::ExpectedSimpleSymbol {
        expected: "terminal".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("terminal"));
}

#[test]
fn glr_error_invalid_symbol_state() {
    let err = GLRError::InvalidSymbolState {
        operation: "lookup".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Invalid symbol state"));
}

#[test]
fn glr_error_debug_impl() {
    let err = GLRError::ConflictResolution("x".into());
    let debug = format!("{err:?}");
    assert!(debug.contains("ConflictResolution"));
}

#[test]
fn table_error_eof_is_error_display() {
    let err = TableError::EofIsError;
    let msg = format!("{err}");
    assert!(msg.contains("EOF"));
}

#[test]
fn table_error_eof_is_error_debug() {
    let err = TableError::EofIsError;
    let debug = format!("{err:?}");
    assert!(debug.contains("EofIsError"));
}

#[test]
fn table_error_eof_missing_from_index() {
    let err = TableError::EofMissingFromIndex;
    let msg = format!("{err}");
    assert!(msg.contains("EOF"));
}

#[test]
fn table_error_eof_not_sentinel() {
    let err = TableError::EofNotSentinel {
        eof: 1,
        token_count: 5,
        external_count: 0,
    };
    let msg = format!("{err}");
    assert!(msg.contains("EOF symbol must be"));
}

#[test]
fn glr_error_table_validation() {
    let err = GLRError::TableValidation(TableError::EofIsError);
    let msg = format!("{err}");
    assert!(msg.contains("Table validation failed"));
}
