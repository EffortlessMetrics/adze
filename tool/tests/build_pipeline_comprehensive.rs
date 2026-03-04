//! Comprehensive build-pipeline tests for the adze-tool crate.
//!
//! Covers: ToolError construction/display/source for each variant,
//! ToolError From impls, build configuration defaults, scanner types,
//! error propagation patterns, and end-to-end build pipeline.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use adze_tool::ToolError;
use adze_tool::pure_rust_builder::{
    BuildOptions, build_parser_from_grammar_js, build_parser_from_json,
};
use adze_tool::scanner_build::{ScannerBuilder, ScannerLanguage, ScannerSource};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts_in(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn build_js(js: &str) -> adze_tool::pure_rust_builder::BuildResult {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    build_parser_from_grammar_js(&path, opts_in(&dir)).unwrap()
}

fn try_build_js(js: &str) -> anyhow::Result<adze_tool::pure_rust_builder::BuildResult> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    build_parser_from_grammar_js(&path, opts_in(&dir))
}

fn grammars_from_rust(code: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("lib.rs");
    fs::write(&src, code).unwrap();
    adze_tool::generate_grammars(&src).unwrap()
}

// =========================================================================
// 1. ToolError construction and display — one per variant
// =========================================================================

#[test]
fn error_display_multiple_word_rules() {
    let e = ToolError::MultipleWordRules;
    assert!(e.to_string().contains("multiple word rules"));
}

#[test]
fn error_display_multiple_precedence_attributes() {
    let e = ToolError::MultiplePrecedenceAttributes;
    assert!(e.to_string().contains("prec"));
}

#[test]
fn error_display_expected_string_literal() {
    let e = ToolError::ExpectedStringLiteral {
        context: "token".into(),
        actual: "42".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("token") && msg.contains("42"));
}

#[test]
fn error_display_expected_integer_literal() {
    let e = ToolError::ExpectedIntegerLiteral {
        actual: "abc".into(),
    };
    assert!(e.to_string().contains("abc"));
}

#[test]
fn error_display_expected_path_type() {
    let e = ToolError::ExpectedPathType {
        actual: "fn()".into(),
    };
    assert!(e.to_string().contains("fn()"));
}

#[test]
fn error_display_expected_single_segment_path() {
    let e = ToolError::ExpectedSingleSegmentPath {
        actual: "a::b::c".into(),
    };
    assert!(e.to_string().contains("a::b::c"));
}

#[test]
fn error_display_nested_option_type() {
    let e = ToolError::NestedOptionType;
    assert!(e.to_string().contains("Option<Option<_>>"));
}

#[test]
fn error_display_struct_has_no_fields() {
    let e = ToolError::StructHasNoFields {
        name: "Empty".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("Empty") && msg.contains("no non-skipped fields"));
}

#[test]
fn error_display_complex_symbols_not_normalized() {
    let e = ToolError::complex_symbols_not_normalized("FIRST set computation");
    assert!(e.to_string().contains("FIRST set computation"));
}

#[test]
fn error_display_expected_symbol_type() {
    let e = ToolError::expected_symbol_type("terminal");
    assert!(e.to_string().contains("terminal"));
}

#[test]
fn error_display_expected_action_type() {
    let e = ToolError::expected_action_type("shift");
    assert!(e.to_string().contains("shift"));
}

#[test]
fn error_display_expected_error_type() {
    let e = ToolError::expected_error_type("syntax");
    assert!(e.to_string().contains("syntax"));
}

#[test]
fn error_display_string_too_long() {
    let e = ToolError::string_too_long("extract", 99999);
    let msg = e.to_string();
    assert!(msg.contains("extract") && msg.contains("99999"));
}

#[test]
fn error_display_invalid_production() {
    let e = ToolError::InvalidProduction {
        details: "empty rhs".into(),
    };
    assert!(e.to_string().contains("empty rhs"));
}

#[test]
fn error_display_grammar_validation() {
    let e = ToolError::grammar_validation("missing start rule");
    assert!(e.to_string().contains("missing start rule"));
}

#[test]
fn error_display_other() {
    let e = ToolError::Other("custom message".into());
    assert_eq!(e.to_string(), "custom message");
}

// =========================================================================
// 2. ToolError From impls — String and &str
// =========================================================================

#[test]
fn error_from_string() {
    let e: ToolError = String::from("owned error").into();
    assert_eq!(e.to_string(), "owned error");
    assert!(matches!(e, ToolError::Other(_)));
}

#[test]
fn error_from_str_ref() {
    let e: ToolError = "borrowed error".into();
    assert_eq!(e.to_string(), "borrowed error");
    assert!(matches!(e, ToolError::Other(_)));
}

#[test]
fn error_from_io() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "gone");
    let e: ToolError = io_err.into();
    assert!(matches!(e, ToolError::Io(_)));
    assert!(e.to_string().contains("gone"));
}

#[test]
fn error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad").unwrap_err();
    let e: ToolError = json_err.into();
    assert!(matches!(e, ToolError::Json(_)));
}

#[test]
fn error_from_ir_error() {
    let ir_err = adze_ir::IrError::InvalidSymbol("sym_x".into());
    let e: ToolError = ir_err.into();
    assert!(matches!(e, ToolError::Ir(_)));
    assert!(e.to_string().contains("sym_x"));
}

// =========================================================================
// 3. ToolError source() chains
// =========================================================================

#[test]
fn source_is_none_for_simple_variants() {
    let simple_errors: Vec<ToolError> = vec![
        ToolError::MultipleWordRules,
        ToolError::NestedOptionType,
        ToolError::Other("msg".into()),
        ToolError::grammar_validation("bad"),
    ];
    for e in &simple_errors {
        assert!(
            e.source().is_none(),
            "expected no source for {:?}",
            std::mem::discriminant(e)
        );
    }
}

#[test]
fn transparent_io_delegates_display() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "no access");
    let e: ToolError = io_err.into();
    // transparent delegates Display to the inner error
    assert!(e.to_string().contains("no access"));
}

#[test]
fn transparent_json_delegates_display() {
    let json_err = serde_json::from_str::<serde_json::Value>("[").unwrap_err();
    let msg = json_err.to_string();
    let e: ToolError = json_err.into();
    assert_eq!(e.to_string(), msg);
}

#[test]
fn tool_error_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    // ToolError should be usable across threads
    assert_send_sync::<ToolError>();
}

// =========================================================================
// 4. Build configuration types and defaults
// =========================================================================

#[test]
fn build_options_default_compress_is_true() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn build_options_default_emit_artifacts_is_false() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
}

#[test]
fn build_options_clone_preserves_fields() {
    let opts = BuildOptions {
        out_dir: "/tmp/test".into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.out_dir, "/tmp/test");
    assert!(cloned.emit_artifacts);
    assert!(!cloned.compress_tables);
}

#[test]
fn build_options_debug_impl() {
    let opts = BuildOptions::default();
    let dbg = format!("{opts:?}");
    assert!(dbg.contains("BuildOptions"));
    assert!(dbg.contains("compress_tables"));
}

// =========================================================================
// 5. Scanner-related types
// =========================================================================

#[test]
fn scanner_language_c_extension() {
    assert_eq!(ScannerLanguage::C.extension(), "c");
}

#[test]
fn scanner_language_cpp_extension() {
    assert_eq!(ScannerLanguage::Cpp.extension(), "cc");
}

#[test]
fn scanner_language_rust_extension() {
    assert_eq!(ScannerLanguage::Rust.extension(), "rs");
}

#[test]
fn scanner_language_equality() {
    assert_eq!(ScannerLanguage::C, ScannerLanguage::C);
    assert_ne!(ScannerLanguage::C, ScannerLanguage::Rust);
}

#[test]
fn scanner_source_fields_accessible() {
    let src = ScannerSource {
        path: PathBuf::from("scanner.c"),
        language: ScannerLanguage::C,
        grammar_name: "test_grammar".into(),
    };
    assert_eq!(src.grammar_name, "test_grammar");
    assert_eq!(src.language, ScannerLanguage::C);
    assert_eq!(src.path, PathBuf::from("scanner.c"));
}

#[test]
fn scanner_builder_find_scanner_empty_dir() {
    let dir = TempDir::new().unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_none(), "empty dir should have no scanner");
}

// =========================================================================
// 6. Error propagation patterns
// =========================================================================

#[test]
fn malformed_json_propagates_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("{bad json".into(), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn nonexistent_grammar_js_propagates_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_grammar_js(Path::new("/nonexistent/grammar.js"), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn invalid_grammar_js_content_propagates_error() {
    let result = try_build_js("THIS IS NOT JAVASCRIPT");
    assert!(result.is_err());
}

#[test]
fn json_missing_rules_key_propagates_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json(r#"{"name":"x"}"#.into(), opts_in(&dir));
    assert!(result.is_err());
}

// =========================================================================
// 7. End-to-end build pipeline
// =========================================================================

#[test]
fn minimal_grammar_builds_with_positive_stats() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'minimal',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    );
    assert_eq!(r.grammar_name, "minimal");
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn rust_grammar_extraction_round_trip() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("rt")]
        mod grammar {
            #[adze::language]
            pub enum T { N(#[adze::leaf(pattern = r"\d+")] i32) }
        }
        "#,
    );
    let dir = TempDir::new().unwrap();
    let json_str = serde_json::to_string(&gs[0]).unwrap();
    let r = build_parser_from_json(json_str, opts_in(&dir)).unwrap();
    assert_eq!(r.grammar_name, "rt");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn node_types_json_is_valid_array() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'nt_check',
  rules: { source: $ => $.tok, tok: $ => /[a-z]+/ }
});
"#,
    );
    let val: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(val.is_array());
}
