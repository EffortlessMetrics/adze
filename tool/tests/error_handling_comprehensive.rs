#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for error handling throughout adze-tool.
//!
//! Covers: build with missing/empty/invalid source files, no grammar module,
//! build stats generation, helpful error messages, multiple error reporting,
//! and build config validation.

use std::fs;
use std::path::Path;

use adze_tool::ToolResult;
use adze_tool::error::ToolError;
use adze_tool::pure_rust_builder::{
    BuildOptions, build_parser_from_grammar_js, build_parser_from_json,
};
use tempfile::TempDir;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn grammars_from_rust(code: &str) -> ToolResult<Vec<serde_json::Value>> {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(&src, code).unwrap();
    adze_tool::generate_grammars(&src)
}

fn try_build_js(js: &str) -> anyhow::Result<adze_tool::pure_rust_builder::BuildResult> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    build_parser_from_grammar_js(&path, opts)
}

fn opts_in(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

// =========================================================================
// 1. Build with missing source file
// =========================================================================

#[test]
fn generate_grammars_missing_file_panics() {
    // syn_inline_mod panics internally when the file does not exist.
    // Verify the panic is raised rather than silent corruption.
    let result = std::panic::catch_unwind(|| {
        let _ = adze_tool::generate_grammars(Path::new("/tmp/nonexistent_adze_test_file.rs"));
    });
    assert!(
        result.is_err(),
        "missing file should cause a panic from syn_inline_mod"
    );
}

#[test]
fn build_parser_for_crate_missing_file_panics() {
    // build_parser_for_crate delegates to generate_grammars which uses
    // syn_inline_mod; verify the panic is raised.
    let result = std::panic::catch_unwind(|| {
        let _ = adze_tool::pure_rust_builder::build_parser_for_crate(
            Path::new("/tmp/nonexistent_adze_test_file.rs"),
            BuildOptions::default(),
        );
    });
    assert!(result.is_err(), "missing file should cause a panic");
}

#[test]
fn build_parser_from_grammar_js_missing_file() {
    let result = build_parser_from_grammar_js(
        Path::new("/tmp/nonexistent_adze_grammar.js"),
        BuildOptions::default(),
    );
    assert!(result.is_err(), "should fail for missing grammar.js");
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.to_lowercase().contains("read") || msg.to_lowercase().contains("no such file"),
        "error should reference read failure, got: {msg}"
    );
}

// =========================================================================
// 2. Build with empty source file
// =========================================================================

#[test]
fn empty_source_file_yields_no_grammars() {
    let result = grammars_from_rust("");
    assert!(result.is_ok(), "empty file should not error");
    assert!(
        result.unwrap().is_empty(),
        "empty file should yield no grammars"
    );
}

#[test]
fn whitespace_only_source_file_yields_no_grammars() {
    let result = grammars_from_rust("   \n\t\n   ");
    assert!(result.is_ok(), "whitespace-only should not error");
    assert!(result.unwrap().is_empty());
}

#[test]
fn empty_grammar_js_fails_gracefully() {
    let result = try_build_js("");
    assert!(result.is_err(), "empty grammar.js should fail");
}

// =========================================================================
// 3. Build with invalid Rust syntax
// =========================================================================

#[test]
fn invalid_rust_syntax_unclosed_brace_panics() {
    // syn_inline_mod panics on unparseable Rust; verify the panic.
    let result = std::panic::catch_unwind(|| {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("lib.rs");
        fs::write(&src, "mod foo {").unwrap();
        let _ = adze_tool::generate_grammars(&src);
    });
    assert!(result.is_err(), "unclosed brace should cause a panic");
}

#[test]
fn invalid_rust_syntax_random_tokens_panics() {
    let result = std::panic::catch_unwind(|| {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("lib.rs");
        fs::write(&src, "@@@ ??? !!!").unwrap();
        let _ = adze_tool::generate_grammars(&src);
    });
    assert!(result.is_err(), "random tokens should cause a panic");
}

#[test]
fn invalid_rust_syntax_incomplete_attribute_panics() {
    let result = std::panic::catch_unwind(|| {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("lib.rs");
        fs::write(&src, "#[adze::grammar(").unwrap();
        let _ = adze_tool::generate_grammars(&src);
    });
    assert!(result.is_err(), "incomplete attribute should cause a panic");
}

// =========================================================================
// 4. Build with no grammar module
// =========================================================================

#[test]
fn plain_module_no_grammar_attr() {
    let gs = grammars_from_rust(
        r#"
        mod my_module {
            pub struct Foo;
        }
        "#,
    )
    .unwrap();
    assert!(
        gs.is_empty(),
        "module without #[adze::grammar] should produce nothing"
    );
}

#[test]
fn struct_only_no_grammar() {
    let gs = grammars_from_rust("pub struct Bar { x: i32 }").unwrap();
    assert!(gs.is_empty());
}

#[test]
fn function_only_no_grammar() {
    let gs = grammars_from_rust("fn main() { println!(\"hello\"); }").unwrap();
    assert!(gs.is_empty());
}

#[test]
fn nested_module_without_grammar_attr() {
    let gs = grammars_from_rust(
        r#"
        mod outer {
            mod inner {
                pub enum E { A, B }
            }
        }
        "#,
    )
    .unwrap();
    assert!(gs.is_empty());
}

// =========================================================================
// 5. Build stats generation
// =========================================================================

fn build_simple_grammar_js() -> adze_tool::pure_rust_builder::BuildResult {
    try_build_js(
        r#"
module.exports = grammar({
  name: 'stats_test',
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/
  }
});
        "#,
    )
    .expect("simple grammar should build")
}

#[test]
fn build_stats_state_count_positive() {
    let result = build_simple_grammar_js();
    assert!(
        result.build_stats.state_count > 0,
        "state_count should be > 0"
    );
}

#[test]
fn build_stats_symbol_count_positive() {
    let result = build_simple_grammar_js();
    assert!(
        result.build_stats.symbol_count > 0,
        "symbol_count should be > 0"
    );
}

#[test]
fn build_stats_conflict_cells_non_negative() {
    let result = build_simple_grammar_js();
    // conflict_cells is usize, always >= 0; verify it's a reasonable number
    assert!(
        result.build_stats.conflict_cells
            <= result.build_stats.state_count * result.build_stats.symbol_count,
        "conflict_cells should not exceed total cells"
    );
}

#[test]
fn build_stats_populated_on_success() {
    let result = build_simple_grammar_js();
    // All stats fields should be populated and the grammar name should be set
    assert_eq!(result.grammar_name, "stats_test");
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// =========================================================================
// 6. Error messages contain helpful information
// =========================================================================

#[test]
fn tool_error_expected_string_literal_has_context() {
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
fn tool_error_struct_no_fields_names_struct() {
    let err = ToolError::StructHasNoFields {
        name: "EmptyNode".into(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("EmptyNode"),
        "error should name the struct, got: {msg}"
    );
}

#[test]
fn tool_error_invalid_production_includes_details() {
    let err = ToolError::InvalidProduction {
        details: "empty RHS in rule `expr`".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("empty RHS"), "got: {msg}");
    assert!(msg.contains("invalid production"), "got: {msg}");
}

#[test]
fn tool_error_grammar_validation_includes_reason() {
    let err = ToolError::GrammarValidation {
        reason: "start symbol undefined".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("start symbol undefined"), "got: {msg}");
    assert!(
        msg.to_lowercase().contains("validation"),
        "should mention validation, got: {msg}"
    );
}

#[test]
fn io_error_preserves_message() {
    let io_err = std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "access denied to /secret",
    );
    let err: ToolError = io_err.into();
    let msg = err.to_string();
    assert!(
        msg.contains("access denied"),
        "IO context should be preserved, got: {msg}"
    );
}

#[test]
fn json_error_includes_position() {
    let json_err = serde_json::from_str::<serde_json::Value>("{ bad }").unwrap_err();
    let err: ToolError = json_err.into();
    let msg = err.to_string();
    // serde_json errors include line/column info
    assert!(
        msg.contains("line") || msg.contains("column") || msg.contains("key"),
        "JSON error should include position info, got: {msg}"
    );
}

// =========================================================================
// 7. Multiple errors reported (not just first)
// =========================================================================

#[test]
fn collect_multiple_tool_errors() {
    let errors: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::MultiplePrecedenceAttributes,
        ToolError::NestedOptionType,
    ];
    assert_eq!(errors.len(), 3);
    // Each error has a distinct discriminant
    for i in 0..errors.len() {
        for j in (i + 1)..errors.len() {
            assert_ne!(
                std::mem::discriminant(&errors[i]),
                std::mem::discriminant(&errors[j]),
            );
        }
    }
}

#[test]
fn error_chain_source_callable_on_wrapped_errors() {
    // Verify source() is callable on all wrapper variants via std::error::Error
    let io_err: ToolError = std::io::Error::new(std::io::ErrorKind::NotFound, "file.rs").into();
    let _ = std::error::Error::source(&io_err); // must compile and not panic

    let json_err: ToolError = serde_json::from_str::<serde_json::Value>("bad")
        .unwrap_err()
        .into();
    let _ = std::error::Error::source(&json_err);

    let syn_err: ToolError = syn::Error::new(proc_macro2::Span::call_site(), "oops").into();
    let _ = std::error::Error::source(&syn_err);
}

#[test]
fn multiple_error_messages_are_distinct() {
    let errors: Vec<ToolError> = vec![
        ToolError::ExpectedStringLiteral {
            context: "field A".into(),
            actual: "num".into(),
        },
        ToolError::ExpectedStringLiteral {
            context: "field B".into(),
            actual: "bool".into(),
        },
        ToolError::ExpectedIntegerLiteral {
            actual: "xyz".into(),
        },
    ];
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    // First two share variant but differ in context
    assert_ne!(messages[0], messages[1]);
    assert_ne!(messages[0], messages[2]);
}

#[test]
fn result_vec_can_accumulate_errors() {
    fn validate_fields(names: &[&str]) -> Vec<ToolError> {
        let mut errs = Vec::new();
        for name in names {
            if name.is_empty() {
                errs.push(ToolError::StructHasNoFields {
                    name: "(empty)".into(),
                });
            }
            if name.contains("::") {
                errs.push(ToolError::ExpectedSingleSegmentPath {
                    actual: name.to_string(),
                });
            }
        }
        errs
    }
    let errs = validate_fields(&["", "std::vec::Vec", "ok"]);
    assert_eq!(errs.len(), 2, "should collect both errors");
}

// =========================================================================
// 8. Build config validation
// =========================================================================

#[test]
fn build_options_default_has_reasonable_values() {
    let opts = BuildOptions::default();
    assert!(!opts.out_dir.is_empty(), "out_dir should not be empty");
    assert!(
        opts.compress_tables,
        "compress_tables should default to true"
    );
}

#[test]
fn build_options_clone_is_equivalent() {
    let opts = BuildOptions {
        out_dir: "/tmp/test".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, opts.out_dir);
    assert_eq!(cloned.emit_artifacts, opts.emit_artifacts);
    assert_eq!(cloned.compress_tables, opts.compress_tables);
}

#[test]
fn build_options_debug_impl() {
    let opts = BuildOptions {
        out_dir: "/some/path".into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let dbg = format!("{opts:?}");
    assert!(
        dbg.contains("BuildOptions"),
        "Debug should include type name, got: {dbg}"
    );
    assert!(
        dbg.contains("/some/path"),
        "Debug should include out_dir, got: {dbg}"
    );
}

#[test]
fn build_with_emit_artifacts_creates_files() {
    let dir = TempDir::new().unwrap();
    let grammar_js = r#"
module.exports = grammar({
  name: 'artifact_test',
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/
  }
});
    "#;
    let path = dir.path().join("grammar.js");
    fs::write(&path, grammar_js).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let result = build_parser_from_grammar_js(&path, opts).unwrap();
    // Parser file should exist
    assert!(
        Path::new(&result.parser_path).exists(),
        "parser file should be written"
    );
    // Artifact directory should be created
    let grammar_dir = dir.path().join("grammar_artifact_test");
    assert!(grammar_dir.exists(), "artifact directory should exist");
}

#[test]
fn build_without_emit_artifacts_still_writes_parser() {
    let dir = TempDir::new().unwrap();
    let grammar_js = r#"
module.exports = grammar({
  name: 'no_artifact',
  rules: {
    source_file: $ => $.expression,
    expression: $ => /[a-z]+/
  }
});
    "#;
    let path = dir.path().join("grammar.js");
    fs::write(&path, grammar_js).unwrap();
    let opts = opts_in(&dir);
    let result = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(Path::new(&result.parser_path).exists());
}

#[test]
fn build_from_json_invalid_json_string() {
    let result = build_parser_from_json("not valid json".to_string(), BuildOptions::default());
    assert!(result.is_err());
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.to_lowercase().contains("json") || msg.to_lowercase().contains("parse"),
        "should mention JSON parse failure, got: {msg}"
    );
}

#[test]
fn build_from_json_missing_name_field() {
    let json = r#"{"rules": {"source_file": {"type": "PATTERN", "value": "\\d+"}}}"#;
    let result = build_parser_from_json(json.to_string(), BuildOptions::default());
    // Should either handle gracefully with "unknown" name or fail with a message
    match result {
        Ok(r) => assert!(!r.grammar_name.is_empty()),
        Err(e) => {
            let msg = format!("{e:#}");
            assert!(!msg.is_empty(), "error should have a message");
        }
    }
}

#[test]
fn build_from_json_empty_rules() {
    let json = r#"{"name": "empty_rules", "rules": {}}"#;
    let result = build_parser_from_json(json.to_string(), BuildOptions::default());
    // An empty grammar should fail during conversion or table building
    assert!(result.is_err(), "empty rules should fail to build");
}
