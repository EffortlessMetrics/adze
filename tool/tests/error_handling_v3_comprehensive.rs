//! Comprehensive tests for adze-tool error handling and ToolError types.
//!
//! Covers: variant construction, Display/Debug formatting, JSON parse errors,
//! grammar structure errors, From conversions, error chaining, and edge cases.

use adze_tool::error::ToolError;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;
use std::error::Error;
use tempfile::TempDir;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn build_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

fn try_build_json(
    value: &serde_json::Value,
) -> anyhow::Result<adze_tool::pure_rust_builder::BuildResult> {
    let (_dir, opts) = build_opts();
    build_parser_from_json(serde_json::to_string(value).unwrap(), opts)
}

fn try_build_raw(raw: &str) -> anyhow::Result<adze_tool::pure_rust_builder::BuildResult> {
    let (_dir, opts) = build_opts();
    build_parser_from_json(raw.to_string(), opts)
}

// =========================================================================
// 1. ToolError variant construction (8 tests)
// =========================================================================

#[test]
fn test_construct_multiple_word_rules() {
    let err = ToolError::MultipleWordRules;
    assert!(matches!(err, ToolError::MultipleWordRules));
}

#[test]
fn test_construct_multiple_precedence_attributes() {
    let err = ToolError::MultiplePrecedenceAttributes;
    assert!(matches!(err, ToolError::MultiplePrecedenceAttributes));
}

#[test]
fn test_construct_expected_string_literal() {
    let err = ToolError::ExpectedStringLiteral {
        context: "rule name".into(),
        actual: "123".into(),
    };
    assert!(matches!(err, ToolError::ExpectedStringLiteral { .. }));
}

#[test]
fn test_construct_expected_integer_literal() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "not_a_number".into(),
    };
    assert!(matches!(err, ToolError::ExpectedIntegerLiteral { .. }));
}

#[test]
fn test_construct_expected_path_type() {
    let err = ToolError::ExpectedPathType {
        actual: "fn()".into(),
    };
    assert!(matches!(err, ToolError::ExpectedPathType { .. }));
}

#[test]
fn test_construct_nested_option_type() {
    let err = ToolError::NestedOptionType;
    assert!(matches!(err, ToolError::NestedOptionType));
}

#[test]
fn test_construct_struct_has_no_fields() {
    let err = ToolError::StructHasNoFields {
        name: "Empty".into(),
    };
    assert!(matches!(err, ToolError::StructHasNoFields { .. }));
}

#[test]
fn test_construct_other_from_string() {
    let err = ToolError::Other("custom error".into());
    assert!(matches!(err, ToolError::Other(_)));
}

// =========================================================================
// 2. ToolError Display format (8 tests)
// =========================================================================

#[test]
fn test_display_multiple_word_rules() {
    let msg = ToolError::MultipleWordRules.to_string();
    assert!(msg.contains("word rule"), "expected 'word rule' in: {msg}");
}

#[test]
fn test_display_multiple_precedence_attributes() {
    let msg = ToolError::MultiplePrecedenceAttributes.to_string();
    assert!(msg.contains("prec"), "expected 'prec' in: {msg}");
}

#[test]
fn test_display_expected_string_literal_includes_both_fields() {
    let err = ToolError::ExpectedStringLiteral {
        context: "field annotation".into(),
        actual: "bool".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("field annotation"), "missing context: {msg}");
    assert!(msg.contains("bool"), "missing actual: {msg}");
}

#[test]
fn test_display_expected_integer_literal_includes_actual() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "xyz".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("xyz"), "missing actual value: {msg}");
}

#[test]
fn test_display_nested_option() {
    let msg = ToolError::NestedOptionType.to_string();
    assert!(msg.contains("Option"), "expected 'Option' in: {msg}");
}

#[test]
fn test_display_struct_has_no_fields_includes_name() {
    let err = ToolError::StructHasNoFields {
        name: "EmptyStruct".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("EmptyStruct"), "missing struct name: {msg}");
}

#[test]
fn test_display_grammar_validation_includes_reason() {
    let err = ToolError::GrammarValidation {
        reason: "missing start rule".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("missing start rule"), "missing reason: {msg}");
}

#[test]
fn test_display_other_passthrough() {
    let err = ToolError::Other("custom message here".into());
    assert_eq!(err.to_string(), "custom message here");
}

// =========================================================================
// 3. ToolError Debug format (5 tests)
// =========================================================================

#[test]
fn test_debug_unit_variant_name() {
    let err = ToolError::MultipleWordRules;
    let dbg = format!("{err:?}");
    assert!(
        dbg.contains("MultipleWordRules"),
        "debug should contain variant name: {dbg}"
    );
}

#[test]
fn test_debug_struct_variant_fields() {
    let err = ToolError::ExpectedStringLiteral {
        context: "ctx".into(),
        actual: "act".into(),
    };
    let dbg = format!("{err:?}");
    assert!(dbg.contains("ctx"), "debug should contain context: {dbg}");
    assert!(dbg.contains("act"), "debug should contain actual: {dbg}");
}

#[test]
fn test_debug_other_contains_message() {
    let err = ToolError::Other("debug payload".into());
    let dbg = format!("{err:?}");
    assert!(
        dbg.contains("debug payload"),
        "debug should contain message: {dbg}"
    );
}

#[test]
fn test_debug_grammar_validation() {
    let err = ToolError::GrammarValidation {
        reason: "no rules defined".into(),
    };
    let dbg = format!("{err:?}");
    assert!(
        dbg.contains("GrammarValidation"),
        "debug should contain variant: {dbg}"
    );
}

#[test]
fn test_debug_string_too_long() {
    let err = ToolError::StringTooLong {
        operation: "extract".into(),
        length: 9999,
    };
    let dbg = format!("{err:?}");
    assert!(dbg.contains("9999"), "debug should contain length: {dbg}");
}

// =========================================================================
// 4. Validation errors from invalid JSON (8 tests)
// =========================================================================

#[test]
fn test_build_from_empty_string_fails() {
    let result = try_build_raw("");
    assert!(result.is_err());
}

#[test]
fn test_build_from_invalid_json_syntax_fails() {
    let result = try_build_raw("{not valid json}");
    assert!(result.is_err());
}

#[test]
fn test_build_from_json_null_fails() {
    let result = try_build_raw("null");
    assert!(result.is_err());
}

#[test]
fn test_build_from_json_array_fails() {
    let result = try_build_raw("[]");
    assert!(result.is_err());
}

#[test]
fn test_build_from_json_number_fails() {
    let result = try_build_raw("42");
    assert!(result.is_err());
}

#[test]
fn test_build_from_json_string_literal_fails() {
    let result = try_build_raw("\"hello\"");
    assert!(result.is_err());
}

#[test]
fn test_build_from_json_boolean_fails() {
    let result = try_build_raw("true");
    assert!(result.is_err());
}

#[test]
fn test_build_from_truncated_json_fails() {
    let result = try_build_raw("{\"name\": \"test\", ");
    assert!(result.is_err());
}

// =========================================================================
// 5. Grammar errors from bad structure (8 tests)
// =========================================================================

#[test]
fn test_build_empty_object_fails() {
    let result = try_build_json(&json!({}));
    assert!(result.is_err());
}

#[test]
fn test_build_missing_rules_key_fails() {
    let result = try_build_json(&json!({"name": "test_lang"}));
    assert!(result.is_err());
}

#[test]
fn test_build_rules_not_object_fails() {
    let result = try_build_json(&json!({"name": "test_lang", "rules": "not_an_object"}));
    assert!(result.is_err());
}

#[test]
fn test_build_rules_null_fails() {
    let result = try_build_json(&json!({"name": "test_lang", "rules": null}));
    assert!(result.is_err());
}

#[test]
fn test_build_empty_rules_fails() {
    let result = try_build_json(&json!({"name": "test_lang", "rules": {}}));
    assert!(result.is_err());
}

#[test]
fn test_build_rule_value_not_object_fails() {
    let result = try_build_json(&json!({
        "name": "test_lang",
        "rules": {
            "source": 42
        }
    }));
    assert!(result.is_err());
}

#[test]
fn test_build_rule_missing_type_field_fails() {
    let result = try_build_json(&json!({
        "name": "test_lang",
        "rules": {
            "source": { "value": "x" }
        }
    }));
    assert!(result.is_err());
}

#[test]
fn test_build_rule_unknown_type_fails() {
    let result = try_build_json(&json!({
        "name": "test_lang",
        "rules": {
            "source": { "type": "NONEXISTENT_TYPE", "value": "x" }
        }
    }));
    assert!(result.is_err());
}

// =========================================================================
// 6. TableGen / downstream error conversions (5 tests)
// =========================================================================

#[test]
fn test_from_tablegen_error() {
    let tg_err = adze_tablegen::TableGenError::InvalidInput("bad table data");
    let tool_err: ToolError = tg_err.into();
    assert!(matches!(tool_err, ToolError::TableGen(_)));
    let msg = tool_err.to_string();
    assert!(msg.contains("bad table data"), "got: {msg}");
}

#[test]
fn test_from_tablegen_compression_error() {
    let tg_err = adze_tablegen::TableGenError::Compression("overflow".into());
    let tool_err: ToolError = tg_err.into();
    assert!(matches!(tool_err, ToolError::TableGen(_)));
}

#[test]
fn test_from_ir_error() {
    let ir_err = adze_ir::IrError::InvalidSymbol("bad_sym".into());
    let tool_err: ToolError = ir_err.into();
    assert!(matches!(tool_err, ToolError::Ir(_)));
    let msg = tool_err.to_string();
    assert!(msg.contains("bad_sym"), "got: {msg}");
}

#[test]
fn test_from_ir_duplicate_rule() {
    let ir_err = adze_ir::IrError::DuplicateRule("rule_x".into());
    let tool_err: ToolError = ir_err.into();
    assert!(matches!(tool_err, ToolError::Ir(_)));
}

#[test]
fn test_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let tool_err: ToolError = io_err.into();
    assert!(matches!(tool_err, ToolError::Io(_)));
    let msg = tool_err.to_string();
    assert!(msg.contains("file missing"), "got: {msg}");
}

// =========================================================================
// 7. Error recovery and chaining (5 tests)
// =========================================================================

#[test]
fn test_tool_error_is_std_error() {
    let err: Box<dyn Error> = Box::new(ToolError::MultipleWordRules);
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_io_error_transparent_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
    let expected_msg = io_err.to_string();
    let tool_err: ToolError = io_err.into();
    // transparent delegates Display to the inner error
    assert_eq!(tool_err.to_string(), expected_msg);
}

#[test]
fn test_json_error_transparent_display() {
    let json_result: Result<serde_json::Value, _> = serde_json::from_str("{invalid");
    let json_err = json_result.unwrap_err();
    let expected_msg = json_err.to_string();
    let tool_err: ToolError = json_err.into();
    assert_eq!(tool_err.to_string(), expected_msg);
}

#[test]
fn test_from_str_creates_other() {
    let tool_err: ToolError = "something went wrong".into();
    assert!(matches!(tool_err, ToolError::Other(_)));
    assert_eq!(tool_err.to_string(), "something went wrong");
}

#[test]
fn test_from_string_creates_other() {
    let tool_err: ToolError = String::from("string error").into();
    assert!(matches!(tool_err, ToolError::Other(_)));
    assert_eq!(tool_err.to_string(), "string error");
}

// =========================================================================
// 8. Edge cases (8 tests)
// =========================================================================

#[test]
fn test_empty_string_context_in_expected_string_literal() {
    let err = ToolError::ExpectedStringLiteral {
        context: String::new(),
        actual: String::new(),
    };
    // Should not panic even with empty fields
    let msg = err.to_string();
    assert!(msg.contains("expected string literal"), "got: {msg}");
}

#[test]
fn test_unicode_in_error_messages() {
    let err = ToolError::Other("日本語エラー 🦀".into());
    assert_eq!(err.to_string(), "日本語エラー 🦀");
}

#[test]
fn test_unicode_in_grammar_validation() {
    let err = ToolError::GrammarValidation {
        reason: "règle manquante «début»".into(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("règle manquante"),
        "unicode reason preserved: {msg}"
    );
}

#[test]
fn test_very_long_error_message() {
    let long_msg = "x".repeat(10_000);
    let err = ToolError::Other(long_msg.clone());
    assert_eq!(err.to_string().len(), 10_000);
}

#[test]
fn test_newlines_in_error_message() {
    let err = ToolError::Other("line1\nline2\nline3".into());
    let msg = err.to_string();
    assert!(msg.contains('\n'), "newlines should be preserved: {msg}");
}

#[test]
fn test_convenience_string_too_long() {
    let err = ToolError::string_too_long("tokenize", 50_000);
    let msg = err.to_string();
    assert!(msg.contains("tokenize"), "got: {msg}");
    assert!(msg.contains("50000"), "got: {msg}");
}

#[test]
fn test_convenience_grammar_validation() {
    let err = ToolError::grammar_validation("duplicate start symbol");
    let msg = err.to_string();
    assert!(msg.contains("duplicate start symbol"), "got: {msg}");
}

#[test]
fn test_convenience_expected_symbol_type() {
    let err = ToolError::expected_symbol_type("terminal");
    let msg = err.to_string();
    assert!(msg.contains("terminal"), "got: {msg}");
}

// =========================================================================
// 9. Additional variant coverage (8 tests)
// =========================================================================

#[test]
fn test_construct_invalid_production() {
    let err = ToolError::InvalidProduction {
        details: "empty RHS".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("empty RHS"), "got: {msg}");
}

#[test]
fn test_construct_complex_symbols_not_normalized() {
    let err = ToolError::ComplexSymbolsNotNormalized {
        operation: "FIRST computation".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("FIRST computation"), "got: {msg}");
}

#[test]
fn test_construct_expected_symbol_type() {
    let err = ToolError::ExpectedSymbolType {
        expected: "non-terminal".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("non-terminal"), "got: {msg}");
}

#[test]
fn test_construct_expected_action_type() {
    let err = ToolError::ExpectedActionType {
        expected: "shift".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("shift"), "got: {msg}");
}

#[test]
fn test_construct_expected_error_type() {
    let err = ToolError::ExpectedErrorType {
        expected: "syntax".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("syntax"), "got: {msg}");
}

#[test]
fn test_convenience_complex_symbols_not_normalized() {
    let err = ToolError::complex_symbols_not_normalized("LR construction");
    let msg = err.to_string();
    assert!(msg.contains("LR construction"), "got: {msg}");
}

#[test]
fn test_convenience_expected_action_type() {
    let err = ToolError::expected_action_type("reduce");
    let msg = err.to_string();
    assert!(msg.contains("reduce"), "got: {msg}");
}

#[test]
fn test_convenience_expected_error_type() {
    let err = ToolError::expected_error_type("parse");
    let msg = err.to_string();
    assert!(msg.contains("parse"), "got: {msg}");
}

// =========================================================================
// 10. build_parser_from_json error messages (5 tests)
// =========================================================================

#[test]
fn test_build_invalid_json_error_message_mentions_json() {
    let err = try_build_raw("<<<").unwrap_err();
    let msg = format!("{err:#}");
    // anyhow chain should mention JSON parsing failure
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("json") || lower.contains("parse") || lower.contains("expected"),
        "error should mention JSON/parse issue: {msg}"
    );
}

#[test]
fn test_build_missing_rules_error_is_descriptive() {
    let err = try_build_json(&json!({"name": "test_lang"})).unwrap_err();
    let msg = format!("{err:#}");
    assert!(!msg.is_empty(), "error message should not be empty");
}

#[test]
fn test_build_error_chain_has_context() {
    let err = try_build_raw("not json at all!").unwrap_err();
    // anyhow errors should have a chain of context
    let chain: Vec<String> = err.chain().map(|e| e.to_string()).collect();
    assert!(!chain.is_empty(), "error chain should not be empty");
}

#[test]
fn test_build_empty_name_falls_back_to_unknown() {
    // Grammar with empty name and no rules still fails, but name shouldn't cause crash
    let result = try_build_json(&json!({"name": "", "rules": {}}));
    assert!(result.is_err());
}

#[test]
fn test_build_name_as_number_still_fails_gracefully() {
    let result = try_build_json(&json!({"name": 42, "rules": {}}));
    assert!(result.is_err());
}
