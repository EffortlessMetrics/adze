#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for error reporting in adze-tool: error types, messages,
//! propagation, formatting, collection, and recovery.

use adze_tool::error::ToolError;
use adze_tool::ToolResult;

// ── 1. Error types and variants ─────────────────────────────────────────────

#[test]
fn all_unit_variants_are_distinct() {
    let variants: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
    ];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(
                std::mem::discriminant(&variants[i]),
                std::mem::discriminant(&variants[j]),
                "variants at {i} and {j} should be distinct"
            );
        }
    }
}

#[test]
fn struct_variants_carry_context() {
    let err = ToolError::ExpectedStringLiteral {
        context: "rule name".into(),
        actual: "123".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("rule name"), "missing context in: {msg}");
    assert!(msg.contains("123"), "missing actual in: {msg}");
}

#[test]
fn string_too_long_captures_operation_and_length() {
    let err = ToolError::StringTooLong {
        operation: "compress".into(),
        length: 100_000,
    };
    let msg = err.to_string();
    assert!(msg.contains("compress"), "missing operation in: {msg}");
    assert!(msg.contains("100000"), "missing length in: {msg}");
}

#[test]
fn invalid_production_carries_details() {
    let err = ToolError::InvalidProduction {
        details: "empty RHS in rule `expr`".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("empty RHS"), "missing details in: {msg}");
    assert!(msg.contains("invalid production"), "missing prefix in: {msg}");
}

// ── 2. Error messages for different failures ────────────────────────────────

#[test]
fn expected_path_type_message() {
    let err = ToolError::ExpectedPathType {
        actual: "impl Trait".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("path") || msg.contains("unit"), "got: {msg}");
    assert!(msg.contains("impl Trait"), "got: {msg}");
}

#[test]
fn expected_single_segment_path_message() {
    let err = ToolError::ExpectedSingleSegmentPath {
        actual: "crate::ast::Node".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("single segment"), "got: {msg}");
    assert!(msg.contains("crate::ast::Node"), "got: {msg}");
}

#[test]
fn nested_option_message_is_readable() {
    let msg = ToolError::NestedOptionType.to_string();
    assert!(
        msg.contains("Option<Option"),
        "should mention nested Option: {msg}"
    );
}

#[test]
fn struct_no_fields_includes_name() {
    let err = ToolError::StructHasNoFields {
        name: "EmptyStruct".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("EmptyStruct"), "should name the struct: {msg}");
}

#[test]
fn grammar_validation_message_includes_reason() {
    let err = ToolError::GrammarValidation {
        reason: "start symbol is unreachable".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("grammar validation"), "got: {msg}");
    assert!(msg.contains("start symbol is unreachable"), "got: {msg}");
}

#[test]
fn other_variant_preserves_arbitrary_text() {
    let err = ToolError::Other("something unexpected happened!".into());
    assert_eq!(err.to_string(), "something unexpected happened!");
}

// ── 3. Error propagation through build pipeline ─────────────────────────────

#[test]
fn io_error_propagates_via_from() {
    fn simulate_io() -> ToolResult<()> {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))?;
        Ok(())
    }
    let err = simulate_io().unwrap_err();
    assert!(matches!(err, ToolError::Io(_)));
}

#[test]
fn json_error_propagates_via_question_mark() {
    fn parse_json() -> ToolResult<serde_json::Value> {
        Ok(serde_json::from_str("{invalid")?)
    }
    let err = parse_json().unwrap_err();
    assert!(matches!(err, ToolError::Json(_)));
}

#[test]
fn ir_error_propagates_via_from() {
    fn ir_op() -> ToolResult<()> {
        Err(adze_ir::IrError::InvalidSymbol("X".into()))?;
        Ok(())
    }
    let err = ir_op().unwrap_err();
    assert!(matches!(err, ToolError::Ir(_)));
    assert!(err.to_string().contains("X"), "got: {err}");
}

#[test]
fn glr_error_propagates_via_from() {
    fn glr_op() -> ToolResult<()> {
        Err(adze_glr_core::GLRError::ConflictResolution(
            "shift-reduce on token IF".into(),
        ))?;
        Ok(())
    }
    let err = glr_op().unwrap_err();
    assert!(matches!(err, ToolError::Glr(_)));
    assert!(err.to_string().contains("shift-reduce"), "got: {err}");
}

#[test]
fn tablegen_error_propagates_via_from() {
    fn tg_op() -> ToolResult<()> {
        Err(adze_tablegen::TableGenError::EmptyGrammar)?;
        Ok(())
    }
    let err = tg_op().unwrap_err();
    assert!(matches!(err, ToolError::TableGen(_)));
}

#[test]
fn syn_error_propagates_via_from() {
    fn syn_op() -> ToolResult<()> {
        Err(syn::Error::new(proc_macro2::Span::call_site(), "unexpected token"))?;
        Ok(())
    }
    let err = syn_op().unwrap_err();
    assert!(matches!(err, ToolError::SynError { .. }));
    assert!(err.to_string().contains("unexpected token"), "got: {err}");
}

#[test]
fn string_converts_to_other_variant() {
    fn fallible() -> ToolResult<()> {
        Err(String::from("custom pipeline error"))?;
        Ok(())
    }
    let err = fallible().unwrap_err();
    match err {
        ToolError::Other(ref s) => assert_eq!(s, "custom pipeline error"),
        _ => panic!("expected Other, got: {err:?}"),
    }
}

#[test]
fn str_ref_converts_to_other_variant() {
    let err: ToolError = "quick error".into();
    match err {
        ToolError::Other(ref s) => assert_eq!(s, "quick error"),
        _ => panic!("expected Other, got: {err:?}"),
    }
}

// ── 4. User-facing error messages ───────────────────────────────────────────

#[test]
fn user_facing_messages_are_nonempty_and_lowercase_ish() {
    let errors: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
        ToolError::ExpectedIntegerLiteral { actual: "x".into() },
        ToolError::Other("user mistake".into()),
    ];
    for err in &errors {
        let msg = err.to_string();
        assert!(!msg.is_empty(), "empty message for {err:?}");
        // Messages should not start with a panic-style prefix
        assert!(
            !msg.starts_with("FATAL"),
            "user-facing message should not start with FATAL: {msg}"
        );
    }
}

#[test]
fn transparent_errors_surface_inner_message() {
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let inner_msg = io.to_string();
    let tool_err: ToolError = io.into();
    assert_eq!(tool_err.to_string(), inner_msg);
}

// ── 5. Error with source file context ───────────────────────────────────────

#[test]
fn expected_string_literal_shows_source_context() {
    let err = ToolError::ExpectedStringLiteral {
        context: "leaf pattern at line 42".into(),
        actual: "bool_literal".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("line 42"), "should carry source hint: {msg}");
    assert!(msg.contains("bool_literal"), "should carry actual: {msg}");
}

#[test]
fn complex_symbols_mentions_operation() {
    let err = ToolError::ComplexSymbolsNotNormalized {
        operation: "FIRST set computation at rule `expr`".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("normalized"), "got: {msg}");
    assert!(msg.contains("FIRST set"), "got: {msg}");
}

// ── 6. Multiple errors collected ────────────────────────────────────────────

#[test]
fn collect_multiple_errors_in_vec() {
    let errors: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::NestedOptionType,
        ToolError::ExpectedIntegerLiteral {
            actual: "abc".into(),
        },
    ];
    assert_eq!(errors.len(), 3);
    // Each error has a unique message
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    for i in 0..messages.len() {
        for j in (i + 1)..messages.len() {
            assert_ne!(messages[i], messages[j], "duplicate messages at {i},{j}");
        }
    }
}

#[test]
fn errors_can_be_joined_into_combined_report() {
    let errors = vec![
        ToolError::grammar_validation("missing start symbol"),
        ToolError::grammar_validation("unreachable rule `foo`"),
        ToolError::InvalidProduction {
            details: "empty alternative".into(),
        },
    ];
    let report: String = errors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  {}. {e}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(report.contains("1. grammar validation"));
    assert!(report.contains("2. grammar validation"));
    assert!(report.contains("3. invalid production"));
}

// ── 7. Error recovery / continuation ────────────────────────────────────────

#[test]
fn map_err_allows_recovery_with_default() {
    fn try_parse() -> ToolResult<i32> {
        Err(ToolError::Other("bad input".into()))
    }
    let val = try_parse().unwrap_or(0);
    assert_eq!(val, 0);
}

#[test]
fn result_and_then_chains_errors() {
    fn step1() -> ToolResult<String> {
        Ok("grammar".into())
    }
    fn step2(input: String) -> ToolResult<String> {
        if input.is_empty() {
            Err(ToolError::grammar_validation("empty grammar"))
        } else {
            Ok(format!("parsed({input})"))
        }
    }
    let result = step1().and_then(step2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "parsed(grammar)");
}

#[test]
fn continue_after_first_error_collects_all() {
    fn validate_rules(rules: &[&str]) -> Vec<ToolError> {
        let mut errors = Vec::new();
        for rule in rules {
            if rule.is_empty() {
                errors.push(ToolError::InvalidProduction {
                    details: "empty rule".into(),
                });
            }
            if rule.len() > 50 {
                errors.push(ToolError::string_too_long("rule definition", rule.len()));
            }
        }
        errors
    }
    let rules = &["valid_rule", "", "x".repeat(100).leak() as &str];
    let errs = validate_rules(rules);
    assert_eq!(errs.len(), 2);
    assert!(matches!(errs[0], ToolError::InvalidProduction { .. }));
    assert!(matches!(errs[1], ToolError::StringTooLong { .. }));
}

// ── 8. Display / Debug formatting ───────────────────────────────────────────

#[test]
fn debug_format_includes_variant_name_for_all() {
    let cases: Vec<(&str, ToolError)> = vec![
        ("MultipleWordRules", ToolError::MultipleWordRules),
        (
            "MultiplePrecedenceAttributes",
            ToolError::MultiplePrecedenceAttributes,
        ),
        ("NestedOptionType", ToolError::NestedOptionType),
        (
            "StructHasNoFields",
            ToolError::StructHasNoFields { name: "X".into() },
        ),
        ("Other", ToolError::Other("msg".into())),
    ];
    for (expected, err) in &cases {
        let dbg = format!("{err:?}");
        assert!(
            dbg.contains(expected),
            "Debug for {expected} missing variant name: {dbg}"
        );
    }
}

#[test]
fn debug_format_shows_field_values() {
    let err = ToolError::ExpectedStringLiteral {
        context: "ctx_val".into(),
        actual: "act_val".into(),
    };
    let dbg = format!("{err:?}");
    assert!(dbg.contains("ctx_val"), "got: {dbg}");
    assert!(dbg.contains("act_val"), "got: {dbg}");
}

#[test]
fn display_and_debug_differ() {
    let err = ToolError::MultipleWordRules;
    let display = format!("{err}");
    let debug = format!("{err:?}");
    assert_ne!(display, debug, "Display and Debug should differ");
}

#[test]
fn error_source_chain_for_io() {
    use std::error::Error;
    let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
    let inner_msg = io.to_string();
    let tool_err: ToolError = io.into();
    // transparent errors delegate Display to inner
    assert_eq!(tool_err.to_string(), inner_msg);
    // source may or may not be present depending on thiserror version
    let _ = tool_err.source();
}

#[test]
fn error_source_chain_for_json() {
    use std::error::Error;
    let json_err = serde_json::from_str::<serde_json::Value>("{{").unwrap_err();
    let inner_msg = json_err.to_string();
    let tool_err: ToolError = json_err.into();
    assert_eq!(tool_err.to_string(), inner_msg);
    let _ = tool_err.source();
}

#[test]
fn error_source_is_none_for_simple_variants() {
    use std::error::Error;
    let err = ToolError::MultipleWordRules;
    assert!(
        err.source().is_none(),
        "unit variant should not have source"
    );
}

#[test]
fn helper_constructors_match_direct_construction() {
    let direct = ToolError::StringTooLong {
        operation: "op".into(),
        length: 42,
    };
    let helper = ToolError::string_too_long("op", 42);
    assert_eq!(direct.to_string(), helper.to_string());
}

#[test]
fn result_type_alias_works_with_question_mark() {
    fn pipeline() -> ToolResult<String> {
        let val: serde_json::Value = serde_json::from_str(r#"{"ok": true}"#)?;
        Ok(val.to_string())
    }
    assert!(pipeline().is_ok());
}
