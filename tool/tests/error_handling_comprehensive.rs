#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for error types, conversions, and handling in the tool crate.

use adze_tool::ToolResult;
use adze_tool::error::ToolError;

// ── Variant construction & Display ──────────────────────────────────────────

#[test]
fn multiple_word_rules_display() {
    let err = ToolError::MultipleWordRules;
    let msg = err.to_string();
    assert!(msg.contains("word rule"), "got: {msg}");
}

#[test]
fn multiple_precedence_attributes_display() {
    let err = ToolError::MultiplePrecedenceAttributes;
    let msg = err.to_string();
    assert!(msg.contains("prec"), "got: {msg}");
}

#[test]
fn expected_string_literal_display() {
    let err = ToolError::ExpectedStringLiteral {
        context: "leaf token".into(),
        actual: "42".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("string literal"), "got: {msg}");
    assert!(msg.contains("leaf token"), "got: {msg}");
    assert!(msg.contains("42"), "got: {msg}");
}

#[test]
fn expected_integer_literal_display() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "not_a_number".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("integer literal"), "got: {msg}");
    assert!(msg.contains("not_a_number"), "got: {msg}");
}

#[test]
fn expected_path_type_display() {
    let err = ToolError::ExpectedPathType {
        actual: "&[u8]".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("path") || msg.contains("unit"), "got: {msg}");
    assert!(msg.contains("&[u8]"), "got: {msg}");
}

#[test]
fn expected_single_segment_path_display() {
    let err = ToolError::ExpectedSingleSegmentPath {
        actual: "std::vec::Vec".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("single segment"), "got: {msg}");
    assert!(msg.contains("std::vec::Vec"), "got: {msg}");
}

#[test]
fn nested_option_type_display() {
    let err = ToolError::NestedOptionType;
    let msg = err.to_string();
    assert!(msg.contains("Option<Option"), "got: {msg}");
}

#[test]
fn struct_has_no_fields_display() {
    let err = ToolError::StructHasNoFields {
        name: "EmptyNode".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("EmptyNode"), "got: {msg}");
    assert!(
        msg.contains("no non-skipped fields") || msg.contains("no fields"),
        "got: {msg}"
    );
}

#[test]
fn invalid_production_display() {
    let err = ToolError::InvalidProduction {
        details: "empty RHS".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid production"), "got: {msg}");
    assert!(msg.contains("empty RHS"), "got: {msg}");
}

#[test]
fn grammar_validation_display() {
    let err = ToolError::GrammarValidation {
        reason: "missing start symbol".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("grammar validation"), "got: {msg}");
    assert!(msg.contains("missing start symbol"), "got: {msg}");
}

#[test]
fn other_variant_display() {
    let err = ToolError::Other("custom message".into());
    assert_eq!(err.to_string(), "custom message");
}

// ── Helper constructors ─────────────────────────────────────────────────────

#[test]
fn helper_string_too_long() {
    let err = ToolError::string_too_long("extraction", 65536);
    let msg = err.to_string();
    assert!(msg.contains("extraction"), "got: {msg}");
    assert!(msg.contains("65536"), "got: {msg}");
}

#[test]
fn helper_complex_symbols_not_normalized() {
    let err = ToolError::complex_symbols_not_normalized("FIRST set computation");
    let msg = err.to_string();
    assert!(msg.contains("normalized"), "got: {msg}");
    assert!(msg.contains("FIRST set computation"), "got: {msg}");
}

#[test]
fn helper_expected_symbol_type() {
    let err = ToolError::expected_symbol_type("Terminal");
    let msg = err.to_string();
    assert!(msg.contains("Terminal"), "got: {msg}");
}

#[test]
fn helper_expected_action_type() {
    let err = ToolError::expected_action_type("Shift");
    let msg = err.to_string();
    assert!(msg.contains("Shift"), "got: {msg}");
}

#[test]
fn helper_expected_error_type() {
    let err = ToolError::expected_error_type("Recoverable");
    let msg = err.to_string();
    assert!(msg.contains("Recoverable"), "got: {msg}");
}

#[test]
fn helper_grammar_validation() {
    let err = ToolError::grammar_validation("conflicting precedence");
    let msg = err.to_string();
    assert!(msg.contains("conflicting precedence"), "got: {msg}");
}

// ── From conversions ────────────────────────────────────────────────────────

#[test]
fn from_string() {
    let err: ToolError = String::from("something went wrong").into();
    match &err {
        ToolError::Other(s) => assert_eq!(s, "something went wrong"),
        other => panic!("expected Other, got: {other:?}"),
    }
}

#[test]
fn from_str_ref() {
    let err: ToolError = "a &str error".into();
    match &err {
        ToolError::Other(s) => assert_eq!(s, "a &str error"),
        other => panic!("expected Other, got: {other:?}"),
    }
}

#[test]
fn from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err: ToolError = io_err.into();
    match &err {
        ToolError::Io(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        other => panic!("expected Io, got: {other:?}"),
    }
}

#[test]
fn from_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let err: ToolError = json_err.into();
    match &err {
        ToolError::Json(_) => {}
        other => panic!("expected Json, got: {other:?}"),
    }
}

#[test]
fn from_ir_error() {
    let ir_err = adze_ir::IrError::InvalidSymbol("bad_sym".into());
    let err: ToolError = ir_err.into();
    match &err {
        ToolError::Ir(_) => {
            assert!(err.to_string().contains("bad_sym"), "got: {err}");
        }
        other => panic!("expected Ir, got: {other:?}"),
    }
}

#[test]
fn from_glr_error() {
    let glr_err = adze_glr_core::GLRError::StateMachine("state overflow".into());
    let err: ToolError = glr_err.into();
    match &err {
        ToolError::Glr(_) => {
            assert!(err.to_string().contains("state overflow"), "got: {err}");
        }
        other => panic!("expected Glr, got: {other:?}"),
    }
}

#[test]
fn from_tablegen_error() {
    let tg_err = adze_tablegen::TableGenError::EmptyGrammar;
    let err: ToolError = tg_err.into();
    match &err {
        ToolError::TableGen(_) => {
            assert!(err.to_string().contains("empty grammar"), "got: {err}");
        }
        other => panic!("expected TableGen, got: {other:?}"),
    }
}

#[test]
fn from_syn_error() {
    let syn_err = syn::Error::new(proc_macro2::Span::call_site(), "bad syntax");
    let err: ToolError = syn_err.into();
    match &err {
        ToolError::SynError { .. } => {
            assert!(err.to_string().contains("bad syntax"), "got: {err}");
        }
        other => panic!("expected SynError, got: {other:?}"),
    }
}

// ── std::error::Error trait ─────────────────────────────────────────────────

#[test]
fn implements_std_error() {
    let err = ToolError::MultipleWordRules;
    let _dyn: &dyn std::error::Error = &err;
}

#[test]
fn io_error_display_is_transparent() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
    let original_msg = io_err.to_string();
    let err: ToolError = io_err.into();
    // transparent delegates Display to the inner error
    assert_eq!(err.to_string(), original_msg);
}

#[test]
fn json_error_display_is_transparent() {
    let json_err = serde_json::from_str::<serde_json::Value>("}{").unwrap_err();
    let original_msg = json_err.to_string();
    let err: ToolError = json_err.into();
    assert_eq!(err.to_string(), original_msg);
}

// ── Debug ───────────────────────────────────────────────────────────────────

#[test]
fn debug_includes_variant_name() {
    let err = ToolError::NestedOptionType;
    let dbg = format!("{err:?}");
    assert!(dbg.contains("NestedOptionType"), "got: {dbg}");
}

#[test]
fn debug_includes_field_values() {
    let err = ToolError::StringTooLong {
        operation: "compress".into(),
        length: 999,
    };
    let dbg = format!("{err:?}");
    assert!(dbg.contains("compress"), "got: {dbg}");
    assert!(dbg.contains("999"), "got: {dbg}");
}

// ── Result type alias ───────────────────────────────────────────────────────

#[test]
fn result_ok_variant() {
    let r: ToolResult<i32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[test]
fn result_err_variant() {
    let r: ToolResult<i32> = Err(ToolError::MultipleWordRules);
    assert!(r.is_err());
}

#[test]
fn result_question_mark_propagation() {
    fn inner() -> ToolResult<()> {
        let _: serde_json::Value = serde_json::from_str("{}")?;
        Ok(())
    }
    assert!(inner().is_ok());
}

#[test]
fn result_question_mark_propagates_json_err() {
    fn inner() -> ToolResult<()> {
        let _: serde_json::Value = serde_json::from_str("bad")?;
        Ok(())
    }
    let err = inner().unwrap_err();
    assert!(matches!(err, ToolError::Json(_)));
}
