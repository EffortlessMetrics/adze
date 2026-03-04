#![cfg(feature = "test-api")]
// Comprehensive tests for GLRError, GlrError, TableError, and error handling paths.

use adze_glr_core::{GLRError, GlrError, TableError};
use adze_ir::GrammarError;
use std::error::Error;

// ===== GLRError/GlrError alias =====

#[test]
fn glr_error_alias_same_type() {
    // GlrError is a type alias for GLRError
    let e: GlrError = GLRError::ConflictResolution("test".to_string());
    let _ = format!("{}", e);
}

// ===== GLRError variants =====

#[test]
fn conflict_resolution_display() {
    let e = GLRError::ConflictResolution("shift/reduce conflict".to_string());
    let s = format!("{}", e);
    assert!(s.contains("Conflict resolution failed"));
    assert!(s.contains("shift/reduce conflict"));
}

#[test]
fn conflict_resolution_debug() {
    let e = GLRError::ConflictResolution("test".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("ConflictResolution"));
}

#[test]
fn state_machine_display() {
    let e = GLRError::StateMachine("invalid transition".to_string());
    let s = format!("{}", e);
    assert!(s.contains("State machine generation failed"));
}

#[test]
fn state_machine_debug() {
    let e = GLRError::StateMachine("msg".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("StateMachine"));
}

#[test]
fn complex_symbols_not_normalized_display() {
    let e = GLRError::ComplexSymbolsNotNormalized {
        operation: "FIRST set computation".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("Complex symbols must be normalized"));
    assert!(s.contains("FIRST set computation"));
}

#[test]
fn complex_symbols_not_normalized_debug() {
    let e = GLRError::ComplexSymbolsNotNormalized {
        operation: "test".to_string(),
    };
    let s = format!("{:?}", e);
    assert!(s.contains("ComplexSymbolsNotNormalized"));
}

#[test]
fn expected_simple_symbol_display() {
    let e = GLRError::ExpectedSimpleSymbol {
        expected: "terminal".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("Expected"));
    assert!(s.contains("terminal"));
}

#[test]
fn expected_simple_symbol_debug() {
    let e = GLRError::ExpectedSimpleSymbol {
        expected: "test".to_string(),
    };
    let s = format!("{:?}", e);
    assert!(s.contains("ExpectedSimpleSymbol"));
}

#[test]
fn invalid_symbol_state_display() {
    let e = GLRError::InvalidSymbolState {
        operation: "reduction".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("Invalid symbol state"));
    assert!(s.contains("reduction"));
}

#[test]
fn invalid_symbol_state_debug() {
    let e = GLRError::InvalidSymbolState {
        operation: "test".to_string(),
    };
    let s = format!("{:?}", e);
    assert!(s.contains("InvalidSymbolState"));
}

#[test]
fn table_validation_display() {
    let e = GLRError::TableValidation(TableError::EofIsError);
    let s = format!("{}", e);
    assert!(s.contains("Table validation failed"));
}

#[test]
fn table_validation_debug() {
    let e = GLRError::TableValidation(TableError::EofIsError);
    let s = format!("{:?}", e);
    assert!(s.contains("TableValidation"));
}

#[test]
fn grammar_error_display() {
    let e = GLRError::GrammarError(GrammarError::InvalidFieldOrdering);
    let s = format!("{}", e);
    assert!(s.contains("Grammar error"));
}

#[test]
fn grammar_error_from_conversion() {
    let ge = GrammarError::InvalidFieldOrdering;
    let e: GLRError = ge.into();
    match e {
        GLRError::GrammarError(_) => {}
        _ => panic!("Expected GrammarError variant"),
    }
}

// ===== TableError variants =====

#[test]
fn eof_is_error_display() {
    let e = TableError::EofIsError;
    let s = format!("{}", e);
    assert!(s.contains("EOF symbol collides with ERROR"));
}

#[test]
fn eof_is_error_debug() {
    let e = TableError::EofIsError;
    let s = format!("{:?}", e);
    assert!(s.contains("EofIsError"));
}

#[test]
fn eof_not_sentinel_display() {
    let e = TableError::EofNotSentinel {
        eof: 5,
        token_count: 10,
        external_count: 2,
    };
    let s = format!("{}", e);
    assert!(s.contains("EOF symbol must be >= token_count"));
    assert!(s.contains("5"));
}

#[test]
fn eof_not_sentinel_debug() {
    let e = TableError::EofNotSentinel {
        eof: 5,
        token_count: 10,
        external_count: 2,
    };
    let s = format!("{:?}", e);
    assert!(s.contains("EofNotSentinel"));
}

#[test]
fn eof_missing_from_index_display() {
    let e = TableError::EofMissingFromIndex;
    let s = format!("{}", e);
    assert!(s.contains("EOF not present in symbol_to_index"));
}

#[test]
fn eof_parity_mismatch_display() {
    let e = TableError::EofParityMismatch(42);
    let s = format!("{}", e);
    assert!(s.contains("EOF column parity mismatch"));
    assert!(s.contains("42"));
}

#[test]
fn eof_parity_mismatch_debug() {
    let e = TableError::EofParityMismatch(0);
    let s = format!("{:?}", e);
    assert!(s.contains("EofParityMismatch"));
}

// ===== Error trait =====

#[test]
fn glr_error_implements_error() {
    let e = GLRError::ConflictResolution("test".to_string());
    let _: &dyn Error = &e;
}

#[test]
fn table_error_implements_error() {
    let e = TableError::EofIsError;
    let _: &dyn Error = &e;
}

// ===== Edge cases =====

#[test]
fn empty_string_conflict_resolution() {
    let e = GLRError::ConflictResolution(String::new());
    let s = format!("{}", e);
    assert!(s.contains("Conflict resolution failed"));
}

#[test]
fn empty_string_state_machine() {
    let e = GLRError::StateMachine(String::new());
    let s = format!("{}", e);
    assert!(s.contains("State machine generation failed"));
}

#[test]
fn long_message_conflict() {
    let msg = "x".repeat(5000);
    let e = GLRError::ConflictResolution(msg);
    let s = format!("{}", e);
    assert!(s.len() > 5000);
}

#[test]
fn unicode_in_operation() {
    let e = GLRError::ComplexSymbolsNotNormalized {
        operation: "正規化".to_string(),
    };
    let s = format!("{}", e);
    assert!(s.contains("正規化"));
}

#[test]
fn eof_not_sentinel_zero_counts() {
    let e = TableError::EofNotSentinel {
        eof: 0,
        token_count: 0,
        external_count: 0,
    };
    let s = format!("{}", e);
    assert!(s.contains("0"));
}

#[test]
fn eof_parity_mismatch_max_state() {
    let e = TableError::EofParityMismatch(u16::MAX);
    let s = format!("{}", e);
    assert!(s.contains(&format!("{}", u16::MAX)));
}

// ===== All variants non-empty display =====

#[test]
fn all_glr_variants_nonempty_display() {
    let variants: Vec<GLRError> = vec![
        GLRError::GrammarError(GrammarError::InvalidFieldOrdering),
        GLRError::ConflictResolution("a".to_string()),
        GLRError::StateMachine("b".to_string()),
        GLRError::TableValidation(TableError::EofIsError),
        GLRError::ComplexSymbolsNotNormalized {
            operation: "c".to_string(),
        },
        GLRError::ExpectedSimpleSymbol {
            expected: "d".to_string(),
        },
        GLRError::InvalidSymbolState {
            operation: "e".to_string(),
        },
    ];
    for e in &variants {
        assert!(
            !format!("{}", e).is_empty(),
            "Display should not be empty for {:?}",
            e
        );
    }
}

#[test]
fn all_table_variants_nonempty_display() {
    let variants: Vec<TableError> = vec![
        TableError::EofIsError,
        TableError::EofNotSentinel {
            eof: 1,
            token_count: 2,
            external_count: 3,
        },
        TableError::EofMissingFromIndex,
        TableError::EofParityMismatch(0),
    ];
    for e in &variants {
        assert!(!format!("{}", e).is_empty());
    }
}
