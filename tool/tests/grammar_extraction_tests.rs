//! Tests for tool crate public API types and functions.

use adze_tool::*;

#[test]
fn tool_error_multiple_word_rules() {
    let err = ToolError::MultipleWordRules;
    let display = format!("{err}");
    assert!(display.contains("word"));
}

#[test]
fn tool_error_multiple_precedence() {
    let err = ToolError::MultiplePrecedenceAttributes;
    let display = format!("{err}");
    assert!(display.contains("prec"));
}

#[test]
fn tool_error_nested_option() {
    let err = ToolError::NestedOptionType;
    let display = format!("{err}");
    assert!(display.contains("Option"));
}

#[test]
fn tool_error_expected_string() {
    let err = ToolError::ExpectedStringLiteral {
        context: "name".to_string(),
        actual: "42".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("name"));
}

#[test]
fn tool_error_expected_integer() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "abc".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("abc"));
}
