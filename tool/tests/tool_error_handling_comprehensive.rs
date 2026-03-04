//! Comprehensive tests for `ToolError` type and error handling in the adze-tool crate.

use adze_tool::error::ToolError;
use std::io;

// ============================================================
// 1. Construction of every variant
// ============================================================

#[test]
fn construct_multiple_word_rules() {
    let e = ToolError::MultipleWordRules;
    assert!(format!("{e}").contains("multiple word rules"));
}

#[test]
fn construct_multiple_precedence_attributes() {
    let e = ToolError::MultiplePrecedenceAttributes;
    assert!(format!("{e}").contains("prec"));
}

#[test]
fn construct_expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: "rule name".into(),
        actual: "42".into(),
    };
    let msg = format!("{e}");
    assert!(msg.contains("rule name"));
    assert!(msg.contains("42"));
}

#[test]
fn construct_expected_integer_literal() {
    let e = ToolError::ExpectedIntegerLiteral {
        actual: "abc".into(),
    };
    assert!(format!("{e}").contains("abc"));
}

#[test]
fn construct_expected_path_type() {
    let e = ToolError::ExpectedPathType {
        actual: "fn()".into(),
    };
    assert!(format!("{e}").contains("fn()"));
}

#[test]
fn construct_expected_single_segment_path() {
    let e = ToolError::ExpectedSingleSegmentPath {
        actual: "a::b::c".into(),
    };
    assert!(format!("{e}").contains("a::b::c"));
}

#[test]
fn construct_nested_option_type() {
    let e = ToolError::NestedOptionType;
    assert!(format!("{e}").contains("Option<Option<_>>"));
}

#[test]
fn construct_struct_has_no_fields() {
    let e = ToolError::StructHasNoFields { name: "Foo".into() };
    assert!(format!("{e}").contains("Foo"));
}

#[test]
fn construct_complex_symbols_not_normalized() {
    let e = ToolError::ComplexSymbolsNotNormalized {
        operation: "FIRST set".into(),
    };
    assert!(format!("{e}").contains("FIRST set"));
}

#[test]
fn construct_expected_symbol_type() {
    let e = ToolError::ExpectedSymbolType {
        expected: "terminal".into(),
    };
    assert!(format!("{e}").contains("terminal"));
}

#[test]
fn construct_expected_action_type() {
    let e = ToolError::ExpectedActionType {
        expected: "shift".into(),
    };
    assert!(format!("{e}").contains("shift"));
}

#[test]
fn construct_expected_error_type() {
    let e = ToolError::ExpectedErrorType {
        expected: "syntax".into(),
    };
    assert!(format!("{e}").contains("syntax"));
}

#[test]
fn construct_string_too_long() {
    let e = ToolError::StringTooLong {
        operation: "serialize".into(),
        length: 99999,
    };
    let msg = format!("{e}");
    assert!(msg.contains("serialize"));
    assert!(msg.contains("99999"));
}

#[test]
fn construct_invalid_production() {
    let e = ToolError::InvalidProduction {
        details: "empty RHS".into(),
    };
    assert!(format!("{e}").contains("empty RHS"));
}

#[test]
fn construct_grammar_validation() {
    let e = ToolError::GrammarValidation {
        reason: "no start symbol".into(),
    };
    assert!(format!("{e}").contains("no start symbol"));
}

#[test]
fn construct_other() {
    let e = ToolError::Other("custom msg".into());
    assert_eq!(format!("{e}"), "custom msg");
}

#[test]
fn construct_io_variant() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "gone");
    let e = ToolError::Io(io_err);
    assert!(format!("{e}").contains("gone"));
}

#[test]
fn construct_json_variant() {
    let j: std::result::Result<serde_json::Value, _> = serde_json::from_str("{bad");
    let e = ToolError::Json(j.unwrap_err());
    assert!(!format!("{e}").is_empty());
}

#[test]
fn construct_ir_variant() {
    let ir = adze_ir::IrError::InvalidSymbol("sym".into());
    let e = ToolError::Ir(ir);
    assert!(format!("{e}").contains("sym"));
}

#[test]
fn construct_glr_variant() {
    let glr = adze_glr_core::GLRError::ConflictResolution("ambiguous".into());
    let e = ToolError::Glr(glr);
    assert!(format!("{e}").contains("ambiguous"));
}

#[test]
fn construct_tablegen_variant() {
    let tg = adze_tablegen::TableGenError::EmptyGrammar;
    let e = ToolError::TableGen(tg);
    assert!(format!("{e}").contains("empty grammar"));
}

#[test]
fn construct_syn_error_variant() {
    let span = proc_macro2::Span::call_site();
    let syn_err = syn::Error::new(span, "bad token");
    let e = ToolError::SynError { syn_error: syn_err };
    assert!(format!("{e}").contains("bad token"));
}

// ============================================================
// 2. From conversions
// ============================================================

#[test]
fn from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let e: ToolError = io_err.into();
    assert!(matches!(e, ToolError::Io(_)));
    assert!(format!("{e}").contains("denied"));
}

#[test]
fn from_serde_json_error() {
    let j: std::result::Result<serde_json::Value, _> = serde_json::from_str("");
    let e: ToolError = j.unwrap_err().into();
    assert!(matches!(e, ToolError::Json(_)));
}

#[test]
fn from_ir_error() {
    let ir = adze_ir::IrError::DuplicateRule("r1".into());
    let e: ToolError = ir.into();
    assert!(matches!(e, ToolError::Ir(_)));
}

#[test]
fn from_glr_error() {
    let glr = adze_glr_core::GLRError::StateMachine("fail".into());
    let e: ToolError = glr.into();
    assert!(matches!(e, ToolError::Glr(_)));
}

#[test]
fn from_tablegen_error() {
    let tg = adze_tablegen::TableGenError::Compression("lzw fail".into());
    let e: ToolError = tg.into();
    assert!(matches!(e, ToolError::TableGen(_)));
}

#[test]
fn from_syn_error() {
    let span = proc_macro2::Span::call_site();
    let syn_err = syn::Error::new(span, "unexpected");
    let e: ToolError = syn_err.into();
    assert!(matches!(e, ToolError::SynError { .. }));
}

#[test]
fn from_string() {
    let e: ToolError = String::from("oops").into();
    assert!(matches!(e, ToolError::Other(ref s) if s == "oops"));
}

#[test]
fn from_str_ref() {
    let e: ToolError = "boom".into();
    assert!(matches!(e, ToolError::Other(ref s) if s == "boom"));
}

// ============================================================
// 3. Display output (exact message matching)
// ============================================================

#[test]
fn display_multiple_word_rules_exact() {
    let e = ToolError::MultipleWordRules;
    assert_eq!(
        e.to_string(),
        "multiple word rules specified - only one word rule is allowed per grammar"
    );
}

#[test]
fn display_multiple_precedence_attributes_exact() {
    let e = ToolError::MultiplePrecedenceAttributes;
    assert_eq!(
        e.to_string(),
        "only one of prec, prec_left, and prec_right can be specified"
    );
}

#[test]
fn display_nested_option_type_exact() {
    let e = ToolError::NestedOptionType;
    assert_eq!(e.to_string(), "Option<Option<_>> is not supported");
}

#[test]
fn display_expected_string_literal_format() {
    let e = ToolError::ExpectedStringLiteral {
        context: "token".into(),
        actual: "123".into(),
    };
    assert_eq!(e.to_string(), "expected string literal for token: 123");
}

#[test]
fn display_expected_integer_literal_format() {
    let e = ToolError::ExpectedIntegerLiteral {
        actual: "xyz".into(),
    };
    assert_eq!(
        e.to_string(),
        "expected integer literal for precedence: xyz"
    );
}

#[test]
fn display_expected_path_type_format() {
    let e = ToolError::ExpectedPathType {
        actual: "&str".into(),
    };
    assert_eq!(e.to_string(), "expected a path or unit type: &str");
}

#[test]
fn display_expected_single_segment_path_format() {
    let e = ToolError::ExpectedSingleSegmentPath {
        actual: "std::io".into(),
    };
    assert_eq!(e.to_string(), "expected a single segment path: std::io");
}

#[test]
fn display_struct_has_no_fields_format() {
    let e = ToolError::StructHasNoFields {
        name: "Empty".into(),
    };
    assert_eq!(e.to_string(), "struct Empty has no non-skipped fields");
}

#[test]
fn display_complex_symbols_not_normalized_format() {
    let e = ToolError::ComplexSymbolsNotNormalized {
        operation: "LR(1)".into(),
    };
    assert_eq!(
        e.to_string(),
        "complex symbols should be normalized before LR(1)"
    );
}

#[test]
fn display_expected_symbol_type_format() {
    let e = ToolError::ExpectedSymbolType {
        expected: "nonterminal".into(),
    };
    assert_eq!(e.to_string(), "expected nonterminal symbol");
}

#[test]
fn display_expected_action_type_format() {
    let e = ToolError::ExpectedActionType {
        expected: "reduce".into(),
    };
    assert_eq!(e.to_string(), "expected reduce action");
}

#[test]
fn display_expected_error_type_format() {
    let e = ToolError::ExpectedErrorType {
        expected: "parse".into(),
    };
    assert_eq!(e.to_string(), "expected parse error");
}

#[test]
fn display_string_too_long_format() {
    let e = ToolError::StringTooLong {
        operation: "emit".into(),
        length: 50000,
    };
    assert_eq!(
        e.to_string(),
        "string too long for emit: length 50000 exceeds maximum"
    );
}

#[test]
fn display_invalid_production_format() {
    let e = ToolError::InvalidProduction {
        details: "cycle detected".into(),
    };
    assert_eq!(e.to_string(), "invalid production rule: cycle detected");
}

#[test]
fn display_grammar_validation_format() {
    let e = ToolError::GrammarValidation {
        reason: "unreachable rule".into(),
    };
    assert_eq!(e.to_string(), "grammar validation failed: unreachable rule");
}

#[test]
fn display_other_format() {
    let e = ToolError::Other("hello world".into());
    assert_eq!(e.to_string(), "hello world");
}

// ============================================================
// 4. Debug output
// ============================================================

#[test]
fn debug_multiple_word_rules() {
    let e = ToolError::MultipleWordRules;
    let dbg = format!("{e:?}");
    assert!(dbg.contains("MultipleWordRules"));
}

#[test]
fn debug_expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: "ctx".into(),
        actual: "act".into(),
    };
    let dbg = format!("{e:?}");
    assert!(dbg.contains("ExpectedStringLiteral"));
    assert!(dbg.contains("ctx"));
    assert!(dbg.contains("act"));
}

#[test]
fn debug_other_variant() {
    let e = ToolError::Other("test".into());
    let dbg = format!("{e:?}");
    assert!(dbg.contains("Other"));
    assert!(dbg.contains("test"));
}

#[test]
fn debug_io_variant() {
    let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe");
    let e = ToolError::Io(io_err);
    let dbg = format!("{e:?}");
    assert!(dbg.contains("Io"));
}

#[test]
fn debug_json_variant() {
    let j: std::result::Result<serde_json::Value, _> = serde_json::from_str("{bad");
    let e = ToolError::Json(j.unwrap_err());
    let dbg = format!("{e:?}");
    assert!(dbg.contains("Json"));
}

#[test]
fn debug_syn_error_variant() {
    let span = proc_macro2::Span::call_site();
    let syn_err = syn::Error::new(span, "whoops");
    let e = ToolError::SynError { syn_error: syn_err };
    let dbg = format!("{e:?}");
    assert!(dbg.contains("SynError"));
}

#[test]
fn debug_nested_option_type() {
    let e = ToolError::NestedOptionType;
    assert!(format!("{e:?}").contains("NestedOptionType"));
}

#[test]
fn debug_string_too_long() {
    let e = ToolError::StringTooLong {
        operation: "op".into(),
        length: 1,
    };
    let dbg = format!("{e:?}");
    assert!(dbg.contains("StringTooLong"));
    assert!(dbg.contains("op"));
}

// ============================================================
// 5. Convenience constructors
// ============================================================

#[test]
fn convenience_string_too_long() {
    let e = ToolError::string_too_long("compress", 1024);
    assert!(matches!(
        e,
        ToolError::StringTooLong {
            ref operation,
            length: 1024,
        } if operation == "compress"
    ));
}

#[test]
fn convenience_complex_symbols_not_normalized() {
    let e = ToolError::complex_symbols_not_normalized("FOLLOW set");
    assert!(matches!(
        e,
        ToolError::ComplexSymbolsNotNormalized { ref operation } if operation == "FOLLOW set"
    ));
}

#[test]
fn convenience_expected_symbol_type() {
    let e = ToolError::expected_symbol_type("anonymous");
    assert!(matches!(
        e,
        ToolError::ExpectedSymbolType { ref expected } if expected == "anonymous"
    ));
}

#[test]
fn convenience_expected_action_type() {
    let e = ToolError::expected_action_type("accept");
    assert!(matches!(
        e,
        ToolError::ExpectedActionType { ref expected } if expected == "accept"
    ));
}

#[test]
fn convenience_expected_error_type() {
    let e = ToolError::expected_error_type("lex");
    assert!(matches!(
        e,
        ToolError::ExpectedErrorType { ref expected } if expected == "lex"
    ));
}

#[test]
fn convenience_grammar_validation() {
    let e = ToolError::grammar_validation("missing rule");
    assert!(matches!(
        e,
        ToolError::GrammarValidation { ref reason } if reason == "missing rule"
    ));
}

// ============================================================
// 6. Edge cases: empty messages
// ============================================================

#[test]
fn empty_other_message() {
    let e = ToolError::Other(String::new());
    assert_eq!(e.to_string(), "");
}

#[test]
fn empty_expected_string_literal_fields() {
    let e = ToolError::ExpectedStringLiteral {
        context: String::new(),
        actual: String::new(),
    };
    assert_eq!(e.to_string(), "expected string literal for : ");
}

#[test]
fn empty_struct_name() {
    let e = ToolError::StructHasNoFields {
        name: String::new(),
    };
    assert_eq!(e.to_string(), "struct  has no non-skipped fields");
}

#[test]
fn empty_grammar_validation_reason() {
    let e = ToolError::grammar_validation("");
    assert_eq!(e.to_string(), "grammar validation failed: ");
}

#[test]
fn empty_invalid_production_details() {
    let e = ToolError::InvalidProduction {
        details: String::new(),
    };
    assert_eq!(e.to_string(), "invalid production rule: ");
}

// ============================================================
// 7. Edge cases: very long messages
// ============================================================

#[test]
fn very_long_other_message() {
    let long = "x".repeat(10_000);
    let e = ToolError::Other(long.clone());
    assert_eq!(e.to_string(), long);
}

#[test]
fn very_long_context_and_actual() {
    let ctx = "c".repeat(5_000);
    let act = "a".repeat(5_000);
    let e = ToolError::ExpectedStringLiteral {
        context: ctx.clone(),
        actual: act.clone(),
    };
    let msg = e.to_string();
    assert!(msg.contains(&ctx));
    assert!(msg.contains(&act));
}

// ============================================================
// 8. Edge cases: special characters
// ============================================================

#[test]
fn special_chars_in_other() {
    let e = ToolError::Other("line1\nline2\ttab\r\0null".into());
    let msg = e.to_string();
    assert!(msg.contains('\n'));
    assert!(msg.contains('\t'));
    assert!(msg.contains('\0'));
}

#[test]
fn unicode_in_expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: "名前".into(),
        actual: "値".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("名前"));
    assert!(msg.contains("値"));
}

#[test]
fn emoji_in_other() {
    let e = ToolError::Other("error 🔥 occurred 💥".into());
    assert!(e.to_string().contains("🔥"));
}

#[test]
fn backslash_in_grammar_validation() {
    let e = ToolError::grammar_validation("path\\to\\file");
    assert!(e.to_string().contains("path\\to\\file"));
}

// ============================================================
// 9. Result type alias
// ============================================================

#[test]
fn result_type_ok() {
    let r: adze_tool::error::Result<i32> = Ok(42);
    assert!(r.is_ok());
}

#[test]
fn result_type_err() {
    let r: adze_tool::error::Result<i32> = Err(ToolError::NestedOptionType);
    assert!(r.is_err());
}

// ============================================================
// 10. Error trait – std::error::Error is implemented via thiserror
// ============================================================

#[test]
fn tool_error_is_std_error() {
    let e = ToolError::MultipleWordRules;
    let _: &dyn std::error::Error = &e;
}

#[test]
fn io_transparent_display_delegates() {
    let inner = io::Error::new(io::ErrorKind::NotFound, "missing");
    let expected_msg = inner.to_string();
    let e = ToolError::Io(inner);
    // transparent delegates Display to the inner error
    assert_eq!(e.to_string(), expected_msg);
}

#[test]
fn json_transparent_display_delegates() {
    let j: std::result::Result<serde_json::Value, _> = serde_json::from_str("{bad");
    let inner = j.unwrap_err();
    let expected_msg = inner.to_string();
    let e = ToolError::Json(inner);
    assert_eq!(e.to_string(), expected_msg);
}

#[test]
fn syn_transparent_display_delegates() {
    let span = proc_macro2::Span::call_site();
    let syn_err = syn::Error::new(span, "err");
    let expected_msg = syn_err.to_string();
    let e = ToolError::SynError { syn_error: syn_err };
    assert_eq!(e.to_string(), expected_msg);
}

#[test]
fn unit_variant_source_is_none() {
    let e = ToolError::MultipleWordRules;
    assert!(std::error::Error::source(&e).is_none());
}

#[test]
fn other_source_is_none() {
    let e = ToolError::Other("msg".into());
    assert!(std::error::Error::source(&e).is_none());
}

// ============================================================
// 11. Error classification: pattern-match variant correctness
// ============================================================

#[test]
fn classify_from_string_is_other() {
    let e: ToolError = "foo".into();
    assert!(matches!(e, ToolError::Other(_)));
}

#[test]
fn classify_from_io_is_io() {
    let e: ToolError = io::Error::other("x").into();
    assert!(matches!(e, ToolError::Io(_)));
}

#[test]
fn classify_from_ir_invalid_symbol_is_ir() {
    let e: ToolError = adze_ir::IrError::InvalidSymbol("s".into()).into();
    assert!(matches!(e, ToolError::Ir(_)));
}

#[test]
fn classify_from_ir_duplicate_rule_is_ir() {
    let e: ToolError = adze_ir::IrError::DuplicateRule("r".into()).into();
    assert!(matches!(e, ToolError::Ir(_)));
}

#[test]
fn classify_from_ir_internal_is_ir() {
    let e: ToolError = adze_ir::IrError::Internal("bug".into()).into();
    assert!(matches!(e, ToolError::Ir(_)));
}

#[test]
fn classify_from_glr_conflict_resolution() {
    let e: ToolError = adze_glr_core::GLRError::ConflictResolution("c".into()).into();
    assert!(matches!(e, ToolError::Glr(_)));
}

#[test]
fn classify_from_glr_state_machine() {
    let e: ToolError = adze_glr_core::GLRError::StateMachine("s".into()).into();
    assert!(matches!(e, ToolError::Glr(_)));
}

#[test]
fn classify_from_tablegen_empty_grammar() {
    let e: ToolError = adze_tablegen::TableGenError::EmptyGrammar.into();
    assert!(matches!(e, ToolError::TableGen(_)));
}

#[test]
fn classify_from_tablegen_invalid_input() {
    let e: ToolError = adze_tablegen::TableGenError::InvalidInput("bad").into();
    assert!(matches!(e, ToolError::TableGen(_)));
}

// ============================================================
// 12. Display preserves inner error messages through transparent
// ============================================================

#[test]
fn io_display_preserves_message() {
    let io_err = io::Error::new(io::ErrorKind::AlreadyExists, "file exists");
    let e: ToolError = io_err.into();
    assert!(e.to_string().contains("file exists"));
}

#[test]
fn ir_display_preserves_message() {
    let ir = adze_ir::IrError::InvalidSymbol("xyz".into());
    let e: ToolError = ir.into();
    assert!(e.to_string().contains("xyz"));
}

#[test]
fn glr_display_preserves_message() {
    let glr = adze_glr_core::GLRError::ConflictResolution("shift/reduce".into());
    let e: ToolError = glr.into();
    assert!(e.to_string().contains("shift/reduce"));
}

#[test]
fn tablegen_display_preserves_message() {
    let tg = adze_tablegen::TableGenError::Automaton("state overflow".into());
    let e: ToolError = tg.into();
    assert!(e.to_string().contains("state overflow"));
}

#[test]
fn syn_display_preserves_message() {
    let span = proc_macro2::Span::call_site();
    let syn_err = syn::Error::new(span, "unexpected token");
    let e: ToolError = syn_err.into();
    assert!(e.to_string().contains("unexpected token"));
}

// ============================================================
// 13. Question-mark operator (?-chain) works with Result alias
// ============================================================

fn returns_ok() -> adze_tool::error::Result<u32> {
    Ok(100)
}

fn returns_err() -> adze_tool::error::Result<u32> {
    Err(ToolError::NestedOptionType)
}

fn chain_ok() -> adze_tool::error::Result<u32> {
    let v = returns_ok()?;
    Ok(v + 1)
}

fn chain_err() -> adze_tool::error::Result<u32> {
    let _v = returns_err()?;
    Ok(0)
}

#[test]
fn question_mark_ok_chain() {
    assert_eq!(chain_ok().unwrap(), 101);
}

#[test]
fn question_mark_err_propagation() {
    let r = chain_err();
    assert!(r.is_err());
    assert!(matches!(r.unwrap_err(), ToolError::NestedOptionType));
}

fn io_to_tool_error() -> adze_tool::error::Result<()> {
    let _ = std::fs::read("/nonexistent/path/unlikely")?;
    Ok(())
}

#[test]
fn question_mark_io_conversion() {
    let r = io_to_tool_error();
    assert!(r.is_err());
    assert!(matches!(r.unwrap_err(), ToolError::Io(_)));
}

// ============================================================
// 14. Convenience constructors produce correct Display output
// ============================================================

#[test]
fn convenience_string_too_long_display() {
    let e = ToolError::string_too_long("encode", 256);
    assert_eq!(
        e.to_string(),
        "string too long for encode: length 256 exceeds maximum"
    );
}

#[test]
fn convenience_complex_symbols_display() {
    let e = ToolError::complex_symbols_not_normalized("canonicalize");
    assert_eq!(
        e.to_string(),
        "complex symbols should be normalized before canonicalize"
    );
}

#[test]
fn convenience_expected_symbol_type_display() {
    let e = ToolError::expected_symbol_type("named");
    assert_eq!(e.to_string(), "expected named symbol");
}

#[test]
fn convenience_expected_action_type_display() {
    let e = ToolError::expected_action_type("goto");
    assert_eq!(e.to_string(), "expected goto action");
}

#[test]
fn convenience_expected_error_type_display() {
    let e = ToolError::expected_error_type("runtime");
    assert_eq!(e.to_string(), "expected runtime error");
}

#[test]
fn convenience_grammar_validation_display() {
    let e = ToolError::grammar_validation("unused symbol");
    assert_eq!(e.to_string(), "grammar validation failed: unused symbol");
}

// ============================================================
// 15. StringTooLong boundary values
// ============================================================

#[test]
fn string_too_long_zero_length() {
    let e = ToolError::string_too_long("op", 0);
    assert!(e.to_string().contains("length 0"));
}

#[test]
fn string_too_long_max_usize() {
    let e = ToolError::string_too_long("op", usize::MAX);
    assert!(e.to_string().contains(&usize::MAX.to_string()));
}

// ============================================================
// 16. Multiple IO error kinds
// ============================================================

#[test]
fn io_not_found() {
    let e: ToolError = io::Error::new(io::ErrorKind::NotFound, "not found").into();
    assert!(e.to_string().contains("not found"));
}

#[test]
fn io_permission_denied() {
    let e: ToolError = io::Error::new(io::ErrorKind::PermissionDenied, "no perms").into();
    assert!(e.to_string().contains("no perms"));
}

#[test]
fn io_already_exists() {
    let e: ToolError = io::Error::new(io::ErrorKind::AlreadyExists, "exists").into();
    assert!(e.to_string().contains("exists"));
}

#[test]
fn io_invalid_input() {
    let e: ToolError = io::Error::new(io::ErrorKind::InvalidInput, "bad input").into();
    assert!(e.to_string().contains("bad input"));
}

// ============================================================
// 17. Formatting traits: Display vs Debug differ
// ============================================================

#[test]
fn display_and_debug_differ_for_struct_variant() {
    let e = ToolError::ExpectedStringLiteral {
        context: "a".into(),
        actual: "b".into(),
    };
    let display = format!("{e}");
    let debug = format!("{e:?}");
    // Debug includes variant name, Display does not
    assert!(debug.contains("ExpectedStringLiteral"));
    assert!(!display.contains("ExpectedStringLiteral"));
}

#[test]
fn display_and_debug_differ_for_unit_variant() {
    let e = ToolError::MultipleWordRules;
    let display = format!("{e}");
    let debug = format!("{e:?}");
    assert!(debug.contains("MultipleWordRules"));
    assert!(!display.contains("MultipleWordRules"));
}

// ============================================================
// 18. Send + Sync bounds (important for async usage)
// ============================================================

fn _assert_send<T: Send>() {}
fn _assert_sync<T: Sync>() {}

#[test]
fn tool_error_is_send() {
    _assert_send::<ToolError>();
}

// Note: syn::Error is not Sync, so ToolError won't be Sync either.
// We just test Send which is the critical bound.

// ============================================================
// 19. non_exhaustive: can match with wildcard
// ============================================================

#[test]
fn non_exhaustive_requires_wildcard() {
    let e = ToolError::MultipleWordRules;
    // non_exhaustive means we need a wildcard arm outside the crate
    match e {
        ToolError::MultipleWordRules => {}
        ToolError::MultiplePrecedenceAttributes => {}
        ToolError::ExpectedStringLiteral { .. } => {}
        ToolError::ExpectedIntegerLiteral { .. } => {}
        ToolError::ExpectedPathType { .. } => {}
        ToolError::ExpectedSingleSegmentPath { .. } => {}
        ToolError::NestedOptionType => {}
        ToolError::StructHasNoFields { .. } => {}
        ToolError::ComplexSymbolsNotNormalized { .. } => {}
        ToolError::ExpectedSymbolType { .. } => {}
        ToolError::ExpectedActionType { .. } => {}
        ToolError::ExpectedErrorType { .. } => {}
        ToolError::StringTooLong { .. } => {}
        ToolError::InvalidProduction { .. } => {}
        ToolError::GrammarValidation { .. } => {}
        ToolError::Other(_) => {}
        ToolError::Io(_) => {}
        ToolError::Json(_) => {}
        ToolError::Ir(_) => {}
        ToolError::Glr(_) => {}
        ToolError::TableGen(_) => {}
        ToolError::SynError { .. } => {}
        _ => {} // required because #[non_exhaustive]
    }
}

// ============================================================
// 20. Chained conversions: inner crate error → ToolError
// ============================================================

#[test]
fn ir_invalid_symbol_through_chain() {
    fn inner() -> adze_tool::error::Result<()> {
        Err(adze_ir::IrError::InvalidSymbol("s".into()))?;
        Ok(())
    }
    assert!(matches!(inner().unwrap_err(), ToolError::Ir(_)));
}

#[test]
fn tablegen_through_chain() {
    fn inner() -> adze_tool::error::Result<()> {
        Err(adze_tablegen::TableGenError::EmptyGrammar)?;
        Ok(())
    }
    assert!(matches!(inner().unwrap_err(), ToolError::TableGen(_)));
}

#[test]
fn glr_through_chain() {
    fn inner() -> adze_tool::error::Result<()> {
        Err(adze_glr_core::GLRError::StateMachine("x".into()))?;
        Ok(())
    }
    assert!(matches!(inner().unwrap_err(), ToolError::Glr(_)));
}

#[test]
fn syn_through_chain() {
    fn inner() -> adze_tool::error::Result<()> {
        let span = proc_macro2::Span::call_site();
        Err(syn::Error::new(span, "chain test"))?;
        Ok(())
    }
    assert!(matches!(inner().unwrap_err(), ToolError::SynError { .. }));
}

#[test]
fn string_through_chain() {
    fn inner() -> adze_tool::error::Result<()> {
        Err(String::from("chain string"))?;
        Ok(())
    }
    let e = inner().unwrap_err();
    assert!(matches!(e, ToolError::Other(ref s) if s == "chain string"));
}
