use adze_glr_core::GLRError;
use adze_ir::error::IrError;
use adze_tablegen::error::TableGenError;
use std::error::Error;

#[test]
fn glr_error_preserves_source_chain() {
    let glr_err = GLRError::ConflictResolution("test conflict issue".to_string());
    let tablegen_err: TableGenError = glr_err.into();

    // Check that we can access the error through the transparent wrapper
    match tablegen_err {
        TableGenError::Glr(ref e) => {
            assert_eq!(
                e.to_string(),
                "Conflict resolution failed: test conflict issue"
            );
        }
        _ => panic!("Expected Glr variant"),
    }
}

#[test]
fn ir_error_preserves_source_chain() {
    let ir_err = IrError::InvalidSymbol("bad_symbol".to_string());
    let tablegen_err: TableGenError = ir_err.into();

    // Check that we can access the error through the transparent wrapper
    match tablegen_err {
        TableGenError::Ir(ref e) => {
            assert_eq!(e.to_string(), "invalid symbol: bad_symbol");
        }
        _ => panic!("Expected Ir variant"),
    }
}

#[test]
fn error_chain_can_be_traversed() {
    let glr_err = GLRError::ConflictResolution("shift/reduce conflict".to_string());
    let tablegen_err: TableGenError = glr_err.into();

    // Verify the display chain
    let err_display = tablegen_err.to_string();
    assert!(err_display.contains("shift/reduce conflict"));

    // For transparent errors, source() returns None since the error IS the source
    assert!(tablegen_err.source().is_none());
}

#[test]
fn string_conversions_still_work() {
    let err1: TableGenError = "plain string error".into();
    assert!(matches!(err1, TableGenError::TableGeneration(_)));

    let err2: TableGenError = String::from("owned string error").into();
    assert!(matches!(err2, TableGenError::TableGeneration(_)));
}
