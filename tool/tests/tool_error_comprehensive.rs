//! Comprehensive tests for ToolError enum and convenience constructors.

use adze_tool::error::ToolError;

// ── 1. Variant construction ─────────────────────────────────────

#[test]
fn test_multiple_word_rules() {
    let err = ToolError::MultipleWordRules;
    let msg = format!("{}", err);
    assert!(msg.contains("word rule"), "got: {}", msg);
}

#[test]
fn test_multiple_precedence_attributes() {
    let err = ToolError::MultiplePrecedenceAttributes;
    let msg = format!("{}", err);
    assert!(msg.contains("prec"), "got: {}", msg);
}

#[test]
fn test_expected_string_literal() {
    let err = ToolError::ExpectedStringLiteral {
        context: "token pattern".to_string(),
        actual: "42".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("token pattern"));
    assert!(msg.contains("42"));
}

#[test]
fn test_expected_integer_literal() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "abc".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("abc"));
}

#[test]
fn test_expected_path_type() {
    let err = ToolError::ExpectedPathType {
        actual: "impl Foo".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("impl Foo"));
}

#[test]
fn test_expected_single_segment_path() {
    let err = ToolError::ExpectedSingleSegmentPath {
        actual: "std::string::String".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("std::string::String"));
}

#[test]
fn test_nested_option_type() {
    let err = ToolError::NestedOptionType;
    let msg = format!("{}", err);
    assert!(msg.contains("Option<Option"));
}

#[test]
fn test_struct_has_no_fields() {
    let err = ToolError::StructHasNoFields {
        name: "Empty".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Empty"));
}

#[test]
fn test_complex_symbols_not_normalized_variant() {
    let err = ToolError::ComplexSymbolsNotNormalized {
        operation: "codegen".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("codegen"));
}

#[test]
fn test_expected_symbol_type_variant() {
    let err = ToolError::ExpectedSymbolType {
        expected: "terminal".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("terminal"));
}

#[test]
fn test_expected_action_type_variant() {
    let err = ToolError::ExpectedActionType {
        expected: "Shift".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Shift"));
}

#[test]
fn test_expected_error_type_variant() {
    let err = ToolError::ExpectedErrorType {
        expected: "syntax".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("syntax"));
}

#[test]
fn test_string_too_long_variant() {
    let err = ToolError::StringTooLong {
        operation: "extract".to_string(),
        length: 99999,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("99999"));
    assert!(msg.contains("extract"));
}

#[test]
fn test_invalid_production() {
    let err = ToolError::InvalidProduction {
        details: "missing LHS".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("missing LHS"));
}

#[test]
fn test_grammar_validation_variant() {
    let err = ToolError::GrammarValidation {
        reason: "unreachable symbol".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("unreachable symbol"));
}

#[test]
fn test_other_error() {
    let err = ToolError::Other("custom error".to_string());
    let msg = format!("{}", err);
    assert_eq!(msg, "custom error");
}

// ── 2. Convenience constructors ──────────────────────────────────

#[test]
fn test_string_too_long_constructor() {
    let err = ToolError::string_too_long("parse", 5000);
    let msg = format!("{}", err);
    assert!(msg.contains("parse"));
    assert!(msg.contains("5000"));
}

#[test]
fn test_complex_symbols_not_normalized_constructor() {
    let err = ToolError::complex_symbols_not_normalized("table_gen");
    let msg = format!("{}", err);
    assert!(msg.contains("table_gen"));
}

#[test]
fn test_expected_symbol_type_constructor() {
    let err = ToolError::expected_symbol_type("non-terminal");
    let msg = format!("{}", err);
    assert!(msg.contains("non-terminal"));
}

#[test]
fn test_expected_action_type_constructor() {
    let err = ToolError::expected_action_type("Reduce");
    let msg = format!("{}", err);
    assert!(msg.contains("Reduce"));
}

#[test]
fn test_expected_error_type_constructor() {
    let err = ToolError::expected_error_type("recoverable");
    let msg = format!("{}", err);
    assert!(msg.contains("recoverable"));
}

#[test]
fn test_grammar_validation_constructor() {
    let err = ToolError::grammar_validation("no start symbol");
    let msg = format!("{}", err);
    assert!(msg.contains("no start symbol"));
}

// ── 3. From impls ───────────────────────────────────────────────

#[test]
fn test_from_string() {
    let err: ToolError = String::from("string error").into();
    let msg = format!("{}", err);
    assert_eq!(msg, "string error");
}

#[test]
fn test_from_str_ref() {
    let err: ToolError = "str error".into();
    let msg = format!("{}", err);
    assert_eq!(msg, "str error");
}

#[test]
fn test_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err: ToolError = io_err.into();
    let msg = format!("{}", err);
    assert!(msg.contains("file missing"));
}

#[test]
fn test_from_json_error() {
    let json_result: Result<serde_json::Value, _> = serde_json::from_str("not json");
    let json_err = json_result.unwrap_err();
    let err: ToolError = json_err.into();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

// ── 4. Debug trait ──────────────────────────────────────────────

#[test]
fn test_debug_output_all_variants() {
    let variants: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
        ToolError::Other("test".into()),
        ToolError::string_too_long("op", 10),
        ToolError::grammar_validation("reason"),
    ];
    for v in &variants {
        let debug = format!("{:?}", v);
        assert!(!debug.is_empty());
    }
}

// ── 5. Error trait ──────────────────────────────────────────────

#[test]
fn test_error_trait_source_for_io() {
    use std::error::Error;
    let io_err = std::io::Error::other("underlying");
    let err: ToolError = io_err.into();
    // Just verify the trait is implemented
    let _ = err.source();
}

#[test]
fn test_error_trait_source_for_other() {
    use std::error::Error;
    let err = ToolError::Other("no source".into());
    // Other variant may or may not have source
    let _ = err.source();
}

// ── 6. Display formatting consistency ───────────────────────────

#[test]
fn test_all_variants_display_non_empty() {
    let variants: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::ExpectedStringLiteral {
            context: "x".into(),
            actual: "y".into(),
        },
        ToolError::ExpectedIntegerLiteral { actual: "z".into() },
        ToolError::ExpectedPathType {
            actual: "impl T".into(),
        },
        ToolError::ExpectedSingleSegmentPath {
            actual: "a::b".into(),
        },
        ToolError::NestedOptionType,
        ToolError::StructHasNoFields { name: "S".into() },
        ToolError::ComplexSymbolsNotNormalized {
            operation: "o".into(),
        },
        ToolError::ExpectedSymbolType {
            expected: "e".into(),
        },
        ToolError::ExpectedActionType {
            expected: "a".into(),
        },
        ToolError::ExpectedErrorType {
            expected: "e".into(),
        },
        ToolError::StringTooLong {
            operation: "o".into(),
            length: 1,
        },
        ToolError::InvalidProduction {
            details: "d".into(),
        },
        ToolError::GrammarValidation { reason: "r".into() },
        ToolError::Other("other".into()),
    ];
    for v in &variants {
        let msg = format!("{}", v);
        assert!(!msg.is_empty(), "variant {:?} has empty display", v);
    }
}
