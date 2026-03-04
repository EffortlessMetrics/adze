//! Tests for tool crate error types and constructors.

use adze_tool::error::ToolError;

#[test]
fn tool_error_multiple_word_rules() {
    let err = ToolError::MultipleWordRules;
    let display = format!("{err}");
    assert!(display.contains("word rule"));
}

#[test]
fn tool_error_multiple_precedence_attributes() {
    let err = ToolError::MultiplePrecedenceAttributes;
    let display = format!("{err}");
    assert!(display.contains("prec"));
}

#[test]
fn tool_error_expected_string_literal() {
    let err = ToolError::ExpectedStringLiteral {
        context: "token name".to_string(),
        actual: "42".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("string literal"));
    assert!(display.contains("token name"));
}

#[test]
fn tool_error_expected_integer_literal() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "abc".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("integer literal"));
}

#[test]
fn tool_error_string_too_long() {
    let err = ToolError::string_too_long("test_op", 9999);
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn tool_error_complex_symbols_not_normalized() {
    let err = ToolError::complex_symbols_not_normalized("compression");
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn tool_error_expected_symbol_type() {
    let err = ToolError::expected_symbol_type("Terminal");
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn tool_error_expected_action_type() {
    let err = ToolError::expected_action_type("Shift");
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn tool_error_grammar_validation() {
    let err = ToolError::grammar_validation("no start symbol");
    let display = format!("{err}");
    assert!(!display.is_empty());
}

#[test]
fn tool_error_debug() {
    let err = ToolError::MultipleWordRules;
    let debug = format!("{err:?}");
    assert!(debug.contains("MultipleWordRules"));
}

#[test]
fn tool_error_is_std_error() {
    let err = ToolError::MultipleWordRules;
    let _: &dyn std::error::Error = &err;
}
