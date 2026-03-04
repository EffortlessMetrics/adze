//! Comprehensive tests for ToolError in adze-tool.

use adze_tool::error::ToolError;

// === Variant construction ===

#[test]
fn multiple_word_rules() {
    let e = ToolError::MultipleWordRules;
    let s = e.to_string();
    assert!(s.contains("multiple word rules"));
}

#[test]
fn multiple_precedence_attributes() {
    let e = ToolError::MultiplePrecedenceAttributes;
    let s = e.to_string();
    assert!(s.contains("prec"));
}

#[test]
fn expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: "token".into(),
        actual: "42".into(),
    };
    let s = e.to_string();
    assert!(s.contains("token"));
    assert!(s.contains("42"));
}

#[test]
fn expected_integer_literal() {
    let e = ToolError::ExpectedIntegerLiteral {
        actual: "\"foo\"".into(),
    };
    let s = e.to_string();
    assert!(s.contains("integer"));
}

#[test]
fn expected_path_type() {
    let e = ToolError::ExpectedPathType {
        actual: "impl Foo".into(),
    };
    let s = e.to_string();
    assert!(s.contains("path"));
}

#[test]
fn expected_single_segment_path() {
    let e = ToolError::ExpectedSingleSegmentPath {
        actual: "std::collections::HashMap".into(),
    };
    let s = e.to_string();
    assert!(s.contains("single segment"));
}

#[test]
fn nested_option_type() {
    let e = ToolError::NestedOptionType;
    let s = e.to_string();
    assert!(s.contains("Option"));
}

#[test]
fn struct_has_no_fields() {
    let e = ToolError::StructHasNoFields {
        name: "Empty".into(),
    };
    let s = e.to_string();
    assert!(s.contains("Empty"));
    assert!(s.contains("no non-skipped fields"));
}

#[test]
fn complex_symbols_not_normalized_variant() {
    let e = ToolError::ComplexSymbolsNotNormalized {
        operation: "codegen".into(),
    };
    let s = e.to_string();
    assert!(s.contains("normalized"));
    assert!(s.contains("codegen"));
}

#[test]
fn expected_symbol_type_variant() {
    let e = ToolError::ExpectedSymbolType {
        expected: "terminal".into(),
    };
    let s = e.to_string();
    assert!(s.contains("terminal"));
}

#[test]
fn expected_action_type_variant() {
    let e = ToolError::ExpectedActionType {
        expected: "shift".into(),
    };
    let s = e.to_string();
    assert!(s.contains("shift"));
}

#[test]
fn expected_error_type_variant() {
    let e = ToolError::ExpectedErrorType {
        expected: "parse".into(),
    };
    let s = e.to_string();
    assert!(s.contains("parse"));
}

#[test]
fn string_too_long_variant() {
    let e = ToolError::StringTooLong {
        operation: "extract".into(),
        length: 999,
    };
    let s = e.to_string();
    assert!(s.contains("extract"));
    assert!(s.contains("999"));
}

#[test]
fn invalid_production() {
    let e = ToolError::InvalidProduction {
        details: "empty RHS".into(),
    };
    let s = e.to_string();
    assert!(s.contains("empty RHS"));
}

#[test]
fn grammar_validation_variant() {
    let e = ToolError::GrammarValidation {
        reason: "no start".into(),
    };
    let s = e.to_string();
    assert!(s.contains("no start"));
}

#[test]
fn other_variant() {
    let e = ToolError::Other("custom error".into());
    let s = e.to_string();
    assert!(s.contains("custom error"));
}

// === Convenience constructors ===

#[test]
fn string_too_long_constructor() {
    let e = ToolError::string_too_long("op", 42);
    let s = e.to_string();
    assert!(s.contains("op"));
    assert!(s.contains("42"));
}

#[test]
fn complex_symbols_constructor() {
    let e = ToolError::complex_symbols_not_normalized("build");
    let s = e.to_string();
    assert!(s.contains("build"));
}

#[test]
fn expected_symbol_type_constructor() {
    let e = ToolError::expected_symbol_type("nonterminal");
    let s = e.to_string();
    assert!(s.contains("nonterminal"));
}

#[test]
fn expected_action_type_constructor() {
    let e = ToolError::expected_action_type("reduce");
    let s = e.to_string();
    assert!(s.contains("reduce"));
}

#[test]
fn expected_error_type_constructor() {
    let e = ToolError::expected_error_type("syntax");
    let s = e.to_string();
    assert!(s.contains("syntax"));
}

#[test]
fn grammar_validation_constructor() {
    let e = ToolError::grammar_validation("unreachable rules");
    let s = e.to_string();
    assert!(s.contains("unreachable"));
}

// === From conversions ===

#[test]
fn from_string() {
    let e: ToolError = String::from("hello error").into();
    match e {
        ToolError::Other(s) => assert_eq!(s, "hello error"),
        _ => panic!("expected Other variant"),
    }
}

#[test]
fn from_str() {
    let e: ToolError = "str error".into();
    match e {
        ToolError::Other(s) => assert_eq!(s, "str error"),
        _ => panic!("expected Other variant"),
    }
}

#[test]
fn from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let e: ToolError = io_err.into();
    let s = e.to_string();
    assert!(s.contains("missing"));
}

#[test]
fn from_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let e: ToolError = json_err.into();
    let s = e.to_string();
    assert!(!s.is_empty());
}

// === Debug impl ===

#[test]
fn debug_multiple_word_rules() {
    let e = ToolError::MultipleWordRules;
    let d = format!("{:?}", e);
    assert!(d.contains("MultipleWordRules"));
}

#[test]
fn debug_expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: "test".into(),
        actual: "x".into(),
    };
    let d = format!("{:?}", e);
    assert!(d.contains("ExpectedStringLiteral"));
}

#[test]
fn debug_other() {
    let e = ToolError::Other("test".into());
    let d = format!("{:?}", e);
    assert!(d.contains("Other"));
}

// === Display consistency ===

#[test]
fn display_all_variants_nonempty() {
    let variants: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::ExpectedStringLiteral {
            context: "c".into(),
            actual: "a".into(),
        },
        ToolError::ExpectedIntegerLiteral { actual: "a".into() },
        ToolError::ExpectedPathType { actual: "a".into() },
        ToolError::ExpectedSingleSegmentPath { actual: "a".into() },
        ToolError::NestedOptionType,
        ToolError::StructHasNoFields { name: "S".into() },
        ToolError::ComplexSymbolsNotNormalized {
            operation: "o".into(),
        },
        ToolError::ExpectedSymbolType {
            expected: "e".into(),
        },
        ToolError::ExpectedActionType {
            expected: "e".into(),
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
        ToolError::Other("x".into()),
    ];
    for v in &variants {
        let s = v.to_string();
        assert!(!s.is_empty(), "variant {:?} has empty display", v);
    }
}

// === Error trait ===

#[test]
fn error_is_std_error() {
    fn assert_error<E: std::error::Error>(_e: &E) {}
    let e = ToolError::MultipleWordRules;
    assert_error(&e);
}

#[test]
fn error_source_io() {
    use std::error::Error;
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
    let e: ToolError = io_err.into();
    // IO variant may or may not expose source through transparent
    let _ = e.source();
}

#[test]
fn error_source_other_is_none() {
    use std::error::Error;
    let e = ToolError::Other("test".into());
    assert!(e.source().is_none());
}

// === Specific message content ===

#[test]
fn multiple_word_rules_message_exact() {
    let e = ToolError::MultipleWordRules;
    assert_eq!(
        e.to_string(),
        "multiple word rules specified - only one word rule is allowed per grammar"
    );
}

#[test]
fn multiple_precedence_message_exact() {
    let e = ToolError::MultiplePrecedenceAttributes;
    assert_eq!(
        e.to_string(),
        "only one of prec, prec_left, and prec_right can be specified"
    );
}

#[test]
fn nested_option_message_exact() {
    let e = ToolError::NestedOptionType;
    assert_eq!(e.to_string(), "Option<Option<_>> is not supported");
}

// === Edge cases ===

#[test]
fn empty_string_other() {
    let e = ToolError::Other(String::new());
    assert_eq!(e.to_string(), "");
}

#[test]
fn empty_context_expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: String::new(),
        actual: String::new(),
    };
    let s = e.to_string();
    assert!(s.contains("expected string literal"));
}

#[test]
fn very_long_operation_string_too_long() {
    let long_op = "x".repeat(1000);
    let e = ToolError::string_too_long(&long_op, usize::MAX);
    let s = e.to_string();
    assert!(s.contains(&long_op));
}

#[test]
fn unicode_in_error_fields() {
    let e = ToolError::Other("ℝ → ℂ mapping failed".into());
    assert!(e.to_string().contains("ℝ"));
}

#[test]
fn zero_length_string_too_long() {
    let e = ToolError::string_too_long("test", 0);
    assert!(e.to_string().contains("0"));
}
