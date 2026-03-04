//! Comprehensive tests for error types and error handling across adze-glr-core.

use adze_glr_core::driver::GlrError as DriverGlrError;
use adze_glr_core::parse_forest::ParseError;
use adze_glr_core::{GLRError, GlrError, GlrResult, TableError};
use adze_ir::{GrammarError, SymbolId};
use std::error::Error;

// ---------------------------------------------------------------------------
// 1. GLRError variant construction
// ---------------------------------------------------------------------------

#[test]
fn glr_error_grammar_error_variant() {
    let err = GLRError::GrammarError(GrammarError::InvalidFieldOrdering);
    assert!(matches!(
        err,
        GLRError::GrammarError(GrammarError::InvalidFieldOrdering)
    ));
}

#[test]
fn glr_error_conflict_resolution_variant() {
    let err = GLRError::ConflictResolution("shift/reduce on token X".into());
    match &err {
        GLRError::ConflictResolution(msg) => assert!(msg.contains("shift/reduce")),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn glr_error_state_machine_variant() {
    let err = GLRError::StateMachine("too many states".into());
    match &err {
        GLRError::StateMachine(msg) => assert_eq!(msg, "too many states"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn glr_error_table_validation_variant() {
    let err = GLRError::TableValidation(TableError::EofIsError);
    assert!(matches!(
        err,
        GLRError::TableValidation(TableError::EofIsError)
    ));
}

#[test]
fn glr_error_complex_symbols_not_normalized() {
    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "FIRST set computation".into(),
    };
    match &err {
        GLRError::ComplexSymbolsNotNormalized { operation } => {
            assert_eq!(operation, "FIRST set computation");
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn glr_error_expected_simple_symbol() {
    let err = GLRError::ExpectedSimpleSymbol {
        expected: "Terminal".into(),
    };
    assert!(matches!(
        err,
        GLRError::ExpectedSimpleSymbol { expected: _ }
    ));
}

#[test]
fn glr_error_invalid_symbol_state() {
    let err = GLRError::InvalidSymbolState {
        operation: "closure computation".into(),
    };
    match &err {
        GLRError::InvalidSymbolState { operation } => {
            assert_eq!(operation, "closure computation");
        }
        _ => panic!("wrong variant"),
    }
}

// ---------------------------------------------------------------------------
// 2. Display / Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn glr_error_display_grammar_error() {
    let err = GLRError::GrammarError(GrammarError::UnresolvedSymbol(SymbolId(42)));
    let msg = err.to_string();
    assert!(msg.contains("Grammar error"), "got: {msg}");
    assert!(msg.contains("42"), "got: {msg}");
}

#[test]
fn glr_error_display_conflict_resolution() {
    let err = GLRError::ConflictResolution("ambiguous".into());
    assert_eq!(err.to_string(), "Conflict resolution failed: ambiguous");
}

#[test]
fn glr_error_display_state_machine() {
    let err = GLRError::StateMachine("overflow".into());
    assert_eq!(err.to_string(), "State machine generation failed: overflow");
}

#[test]
fn glr_error_display_table_validation() {
    let err = GLRError::TableValidation(TableError::EofMissingFromIndex);
    let msg = err.to_string();
    assert!(msg.contains("Table validation failed"), "got: {msg}");
    assert!(msg.contains("EOF not present"), "got: {msg}");
}

#[test]
fn glr_error_display_complex_symbols() {
    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "LR(1)".into(),
    };
    assert!(err.to_string().contains("normalized"), "{}", err);
}

#[test]
fn glr_error_display_expected_simple() {
    let err = GLRError::ExpectedSimpleSymbol {
        expected: "Terminal".into(),
    };
    assert!(err.to_string().contains("Terminal"), "{}", err);
}

#[test]
fn glr_error_display_invalid_symbol_state() {
    let err = GLRError::InvalidSymbolState {
        operation: "goto".into(),
    };
    assert!(err.to_string().contains("goto"), "{}", err);
}

#[test]
fn glr_error_debug_includes_variant_name() {
    let err = GLRError::StateMachine("dbg".into());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("StateMachine"), "got: {dbg}");
}

// ---------------------------------------------------------------------------
// 3. Error conversion (From impls)
// ---------------------------------------------------------------------------

#[test]
fn glr_error_from_grammar_error() {
    let ge = GrammarError::InvalidFieldOrdering;
    let err: GLRError = ge.into();
    assert!(matches!(
        err,
        GLRError::GrammarError(GrammarError::InvalidFieldOrdering)
    ));
}

#[test]
fn glr_error_from_grammar_error_unresolved_symbol() {
    let ge = GrammarError::UnresolvedSymbol(SymbolId(99));
    let err: GLRError = GLRError::from(ge);
    match &err {
        GLRError::GrammarError(GrammarError::UnresolvedSymbol(id)) => {
            assert_eq!(id.0, 99);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn glr_error_from_grammar_error_external_symbol() {
    let ge = GrammarError::UnresolvedExternalSymbol(SymbolId(7));
    let err: GLRError = ge.into();
    assert!(matches!(
        err,
        GLRError::GrammarError(GrammarError::UnresolvedExternalSymbol(_))
    ));
}

// ---------------------------------------------------------------------------
// 4. std::error::Error impl
// ---------------------------------------------------------------------------

#[test]
fn glr_error_is_std_error() {
    let err: Box<dyn Error> = Box::new(GLRError::StateMachine("test".into()));
    assert!(err.to_string().contains("test"));
}

#[test]
fn glr_error_source_for_grammar_error() {
    let err = GLRError::GrammarError(GrammarError::InvalidFieldOrdering);
    // GrammarError is wrapped via #[from], so source() should return the inner error.
    let source = err.source();
    assert!(
        source.is_some(),
        "GrammarError variant should have a source"
    );
    assert!(
        source.unwrap().to_string().contains("field ordering"),
        "source: {}",
        source.unwrap()
    );
}

#[test]
fn glr_error_source_for_leaf_variants_is_none() {
    let err = GLRError::ConflictResolution("x".into());
    assert!(err.source().is_none());

    let err = GLRError::StateMachine("y".into());
    assert!(err.source().is_none());

    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "z".into(),
    };
    assert!(err.source().is_none());
}

// ---------------------------------------------------------------------------
// 5. Error propagation patterns (the ? operator)
// ---------------------------------------------------------------------------

fn fallible_glr() -> Result<(), GLRError> {
    Err(GLRError::StateMachine("propagated".into()))
}

#[test]
fn glr_error_propagation_with_question_mark() {
    let result = fallible_glr();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, GLRError::StateMachine(_)));
}

fn fallible_grammar_to_glr() -> Result<(), GLRError> {
    // GrammarError auto-converts via From
    Err(GrammarError::ConflictError("auto-convert".into()))?
}

#[test]
fn grammar_error_propagates_into_glr_error() {
    let result = fallible_grammar_to_glr();
    let err = result.unwrap_err();
    match err {
        GLRError::GrammarError(GrammarError::ConflictError(msg)) => {
            assert_eq!(msg, "auto-convert");
        }
        _ => panic!("expected GrammarError variant, got: {err}"),
    }
}

fn fallible_glr_result() -> GlrResult<u32> {
    Err(GLRError::StateMachine("via alias".into()))
}

#[test]
fn glr_result_alias_works() {
    let r = fallible_glr_result();
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("via alias"));
}

// ---------------------------------------------------------------------------
// 6. TableError variants
// ---------------------------------------------------------------------------

#[test]
fn table_error_eof_is_error() {
    let err = TableError::EofIsError;
    assert_eq!(err.to_string(), "EOF symbol collides with ERROR");
}

#[test]
fn table_error_eof_not_sentinel() {
    let err = TableError::EofNotSentinel {
        eof: 3,
        token_count: 10,
        external_count: 2,
    };
    let msg = err.to_string();
    assert!(msg.contains("3"), "got: {msg}");
    assert!(msg.contains("10"), "got: {msg}");
    assert!(msg.contains("2"), "got: {msg}");
}

#[test]
fn table_error_eof_missing_from_index() {
    let err = TableError::EofMissingFromIndex;
    assert!(err.to_string().contains("EOF not present"));
}

#[test]
fn table_error_eof_parity_mismatch() {
    let err = TableError::EofParityMismatch(7);
    assert!(err.to_string().contains("7"), "{}", err);
}

#[test]
fn table_error_is_std_error() {
    let err: Box<dyn Error> = Box::new(TableError::EofIsError);
    assert!(!err.to_string().is_empty());
}

#[test]
fn table_error_debug_format() {
    let err = TableError::EofParityMismatch(42);
    let dbg = format!("{err:?}");
    assert!(dbg.contains("EofParityMismatch"), "got: {dbg}");
    assert!(dbg.contains("42"), "got: {dbg}");
}

// ---------------------------------------------------------------------------
// 7. ParseError variants (parse_forest)
// ---------------------------------------------------------------------------

#[test]
fn parse_error_incomplete() {
    let err = ParseError::Incomplete;
    assert_eq!(err.to_string(), "Incomplete parse");
}

#[test]
fn parse_error_failed() {
    let err = ParseError::Failed("unexpected token '}'".into());
    assert!(err.to_string().contains("unexpected token"));
}

#[test]
fn parse_error_unknown() {
    let err = ParseError::Unknown;
    assert_eq!(err.to_string(), "Unknown error");
}

#[test]
fn parse_error_is_std_error() {
    let err: Box<dyn Error> = Box::new(ParseError::Unknown);
    assert!(!err.to_string().is_empty());
}

#[test]
fn parse_error_clone() {
    let err = ParseError::Failed("cloneable".into());
    let cloned = err.clone();
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn parse_error_debug_format() {
    let err = ParseError::Incomplete;
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Incomplete"), "got: {dbg}");
}

// ---------------------------------------------------------------------------
// 8. driver::GlrError variants
// ---------------------------------------------------------------------------

#[test]
fn driver_glr_error_lex() {
    let err = DriverGlrError::Lex("unexpected byte 0xFF".into());
    assert!(err.to_string().contains("lexer error"));
    assert!(err.to_string().contains("0xFF"));
}

#[test]
fn driver_glr_error_parse() {
    let err = DriverGlrError::Parse("no viable alternative".into());
    assert!(err.to_string().contains("parse error"));
}

#[test]
fn driver_glr_error_other() {
    let err = DriverGlrError::Other("miscellaneous".into());
    assert_eq!(err.to_string(), "miscellaneous");
}

#[test]
fn driver_glr_error_is_std_error() {
    let err: Box<dyn Error> = Box::new(DriverGlrError::Parse("boxed".into()));
    assert!(err.to_string().contains("boxed"));
}

#[test]
fn driver_glr_error_debug() {
    let err = DriverGlrError::Lex("dbg".into());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Lex"), "got: {dbg}");
}

// ---------------------------------------------------------------------------
// 9. GlrError alias identity
// ---------------------------------------------------------------------------

#[test]
fn glr_error_alias_is_same_type() {
    // `GlrError` is a type alias for `GLRError`
    let err: GlrError = GLRError::StateMachine("alias check".into());
    let _: GLRError = err;
}

// ---------------------------------------------------------------------------
// 10. GrammarError variants (from adze-ir, used via From)
// ---------------------------------------------------------------------------

#[test]
fn grammar_error_invalid_precedence() {
    let ge = GrammarError::InvalidPrecedence("negative level".into());
    let err: GLRError = ge.into();
    let msg = err.to_string();
    assert!(msg.contains("Grammar error"), "got: {msg}");
}

#[test]
fn grammar_error_conflict() {
    let ge = GrammarError::ConflictError("reduce/reduce".into());
    let msg = ge.to_string();
    assert!(msg.contains("reduce/reduce"));
}

// ---------------------------------------------------------------------------
// 11. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn glr_error_empty_message() {
    let err = GLRError::StateMachine(String::new());
    assert_eq!(err.to_string(), "State machine generation failed: ");
}

#[test]
fn glr_error_very_long_message() {
    let long = "x".repeat(10_000);
    let err = GLRError::ConflictResolution(long.clone());
    assert_eq!(
        err.to_string().len(),
        "Conflict resolution failed: ".len() + 10_000
    );
}

#[test]
fn table_error_eof_not_sentinel_boundary_values() {
    let err = TableError::EofNotSentinel {
        eof: u16::MAX,
        token_count: u32::MAX,
        external_count: u32::MAX,
    };
    let msg = err.to_string();
    assert!(msg.contains(&u16::MAX.to_string()), "got: {msg}");
}

#[test]
fn parse_error_failed_with_unicode() {
    let err = ParseError::Failed("unexpected '→' at position 42".into());
    assert!(err.to_string().contains('→'));
}

#[test]
fn driver_error_with_newlines() {
    let err = DriverGlrError::Parse("line1\nline2\nline3".into());
    assert!(err.to_string().contains("line1\nline2"));
}

#[test]
fn multiple_errors_in_vec() {
    let errors: Vec<Box<dyn Error>> = vec![
        Box::new(GLRError::StateMachine("a".into())),
        Box::new(ParseError::Incomplete),
        Box::new(DriverGlrError::Lex("b".into())),
        Box::new(TableError::EofIsError),
    ];
    assert_eq!(errors.len(), 4);
    for e in &errors {
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn glr_error_table_validation_wraps_all_table_error_variants() {
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
    for te in variants {
        let glr = GLRError::TableValidation(te);
        assert!(glr.to_string().contains("Table validation failed"));
    }
}
