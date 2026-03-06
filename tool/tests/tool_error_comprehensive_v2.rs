//! Comprehensive tests for ToolError API and error conversions.

use adze_tool::error::ToolError;

// ─── Variant construction ───

#[test]
fn multiple_word_rules_error() {
    let e = ToolError::MultipleWordRules;
    let msg = e.to_string();
    assert!(msg.contains("multiple word rules"));
}

#[test]
fn multiple_precedence_attributes_error() {
    let e = ToolError::MultiplePrecedenceAttributes;
    let msg = e.to_string();
    assert!(msg.contains("prec"));
}

#[test]
fn expected_string_literal_error() {
    let e = ToolError::ExpectedStringLiteral {
        context: "token pattern".to_string(),
        actual: "42".to_string(),
    };
    let msg = e.to_string();
    assert!(msg.contains("token pattern"));
    assert!(msg.contains("42"));
}

#[test]
fn expected_integer_literal_error() {
    let e = ToolError::ExpectedIntegerLiteral {
        actual: "\"hello\"".to_string(),
    };
    let msg = e.to_string();
    assert!(msg.contains("integer literal"));
    assert!(msg.contains("\"hello\""));
}

#[test]
fn expected_path_type_error() {
    let e = ToolError::ExpectedPathType {
        actual: "impl Trait".to_string(),
    };
    let msg = e.to_string();
    assert!(msg.contains("path"));
}

#[test]
fn expected_single_segment_path_error() {
    let e = ToolError::ExpectedSingleSegmentPath {
        actual: "a::b::c".to_string(),
    };
    let msg = e.to_string();
    assert!(msg.contains("single segment"));
}

#[test]
fn nested_option_type_error() {
    let e = ToolError::NestedOptionType;
    let msg = e.to_string();
    assert!(msg.contains("Option<Option<_>>"));
}

#[test]
fn struct_has_no_fields_error() {
    let e = ToolError::StructHasNoFields {
        name: "EmptyStruct".to_string(),
    };
    let msg = e.to_string();
    assert!(msg.contains("EmptyStruct"));
    assert!(msg.contains("no non-skipped fields"));
}

#[test]
fn invalid_production_error() {
    let e = ToolError::InvalidProduction {
        details: "missing LHS".to_string(),
    };
    let msg = e.to_string();
    assert!(msg.contains("invalid production"));
    assert!(msg.contains("missing LHS"));
}

#[test]
fn other_error_from_string() {
    let e = ToolError::Other("custom error".to_string());
    assert_eq!(e.to_string(), "custom error");
}

// ─── Convenience constructors ───

#[test]
fn string_too_long_constructor() {
    let e = ToolError::string_too_long("extract", 999);
    let msg = e.to_string();
    assert!(msg.contains("extract"));
    assert!(msg.contains("999"));
}

#[test]
fn complex_symbols_not_normalized_constructor() {
    let e = ToolError::complex_symbols_not_normalized("table generation");
    let msg = e.to_string();
    assert!(msg.contains("normalized"));
    assert!(msg.contains("table generation"));
}

#[test]
fn expected_symbol_type_constructor() {
    let e = ToolError::expected_symbol_type("Terminal");
    let msg = e.to_string();
    assert!(msg.contains("Terminal"));
}

#[test]
fn expected_action_type_constructor() {
    let e = ToolError::expected_action_type("Shift");
    let msg = e.to_string();
    assert!(msg.contains("Shift"));
}

#[test]
fn expected_error_type_constructor() {
    let e = ToolError::expected_error_type("ParseError");
    let msg = e.to_string();
    assert!(msg.contains("ParseError"));
}

#[test]
fn grammar_validation_constructor() {
    let e = ToolError::grammar_validation("undefined symbol");
    let msg = e.to_string();
    assert!(msg.contains("undefined symbol"));
}

// ─── From implementations ───

#[test]
fn from_string() {
    let e: ToolError = "test error".to_string().into();
    assert_eq!(e.to_string(), "test error");
}

#[test]
fn from_str() {
    let e: ToolError = "test error".into();
    assert_eq!(e.to_string(), "test error");
}

#[test]
fn from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e: ToolError = io_err.into();
    let msg = e.to_string();
    assert!(msg.contains("file not found"));
}

#[test]
fn from_json_error() {
    let json_err: serde_json::Error = serde_json::from_str::<i32>("not json").unwrap_err();
    let e: ToolError = json_err.into();
    let msg = e.to_string();
    assert!(!msg.is_empty());
}

// ─── Debug format ───

#[test]
fn debug_format_variants() {
    let variants: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
        ToolError::Other("debug test".to_string()),
    ];
    for v in &variants {
        let d = format!("{:?}", v);
        assert!(!d.is_empty());
    }
}

#[test]
fn debug_format_string_too_long() {
    let e = ToolError::string_too_long("op", 42);
    let d = format!("{:?}", e);
    assert!(d.contains("StringTooLong"));
}

// ─── Error trait ───

#[test]
fn error_trait_display() {
    let e = ToolError::MultipleWordRules;
    let msg = format!("{}", e);
    assert!(!msg.is_empty());
}

#[test]
fn error_trait_source_io() {
    let io_err = std::io::Error::other("inner");
    let e: ToolError = io_err.into();
    // transparent error: source may or may not propagate
    let _source = std::error::Error::source(&e);
}

#[test]
fn error_trait_source_other() {
    let e = ToolError::Other("test".to_string());
    let source = std::error::Error::source(&e);
    assert!(source.is_none());
}

// ─── Result type alias ───

#[test]
fn result_ok() {
    let r: adze_tool::error::Result<i32> = Ok(42);
    assert!(r.is_ok());
}

#[test]
fn result_err() {
    let r: adze_tool::error::Result<i32> = Err(ToolError::MultipleWordRules);
    assert!(r.is_err());
}

// ─── Variant field values ───

#[test]
fn expected_string_literal_fields() {
    let e = ToolError::ExpectedStringLiteral {
        context: "field".to_string(),
        actual: "123".to_string(),
    };
    if let ToolError::ExpectedStringLiteral { context, actual } = e {
        assert_eq!(context, "field");
        assert_eq!(actual, "123");
    }
}

#[test]
fn expected_integer_literal_field() {
    let e = ToolError::ExpectedIntegerLiteral {
        actual: "abc".to_string(),
    };
    if let ToolError::ExpectedIntegerLiteral { actual } = e {
        assert_eq!(actual, "abc");
    }
}

#[test]
fn struct_has_no_fields_field() {
    let e = ToolError::StructHasNoFields {
        name: "MyStruct".to_string(),
    };
    if let ToolError::StructHasNoFields { name } = e {
        assert_eq!(name, "MyStruct");
    }
}

#[test]
fn string_too_long_fields() {
    if let ToolError::StringTooLong { operation, length } = ToolError::string_too_long("parse", 100)
    {
        assert_eq!(operation, "parse");
        assert_eq!(length, 100);
    }
}

#[test]
fn complex_symbols_not_normalized_field() {
    if let ToolError::ComplexSymbolsNotNormalized { operation } =
        ToolError::complex_symbols_not_normalized("codegen")
    {
        assert_eq!(operation, "codegen");
    }
}

// ─── Edge cases ───

#[test]
fn empty_string_error() {
    let e: ToolError = "".into();
    assert_eq!(e.to_string(), "");
}

#[test]
fn long_string_error() {
    let long = "x".repeat(10000);
    let e: ToolError = long.clone().into();
    assert_eq!(e.to_string(), long);
}

#[test]
fn string_too_long_zero_length() {
    let e = ToolError::string_too_long("op", 0);
    let msg = e.to_string();
    assert!(msg.contains("0"));
}

#[test]
fn string_too_long_max_length() {
    let e = ToolError::string_too_long("op", usize::MAX);
    let msg = e.to_string();
    assert!(!msg.is_empty());
}

#[test]
fn grammar_validation_empty_reason() {
    let e = ToolError::grammar_validation("");
    let msg = e.to_string();
    assert!(msg.contains("grammar validation"));
}

#[test]
fn multiple_errors_in_sequence() {
    let errors: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
        "custom".into(),
        ToolError::string_too_long("x", 1),
        ToolError::complex_symbols_not_normalized("y"),
        ToolError::expected_symbol_type("z"),
        ToolError::expected_action_type("a"),
        ToolError::expected_error_type("b"),
        ToolError::grammar_validation("c"),
    ];
    for e in &errors {
        assert!(!e.to_string().is_empty());
        assert!(!format!("{:?}", e).is_empty());
    }
    assert_eq!(errors.len(), 10);
}
