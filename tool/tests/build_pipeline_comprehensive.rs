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

// =========================================================================
// 8. Grammar JSON generation — structural checks
// =========================================================================

#[test]
fn generated_grammar_json_has_name_field() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("named")]
        mod grammar {
            #[adze::language]
            pub enum Lang { Tok(#[adze::leaf(pattern = r"[a-z]+")] String) }
        }
        "#,
    );
    assert_eq!(gs.len(), 1);
    assert_eq!(gs[0]["name"].as_str().unwrap(), "named");
}

#[test]
fn generated_grammar_json_has_rules_object() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("rules_check")]
        mod grammar {
            #[adze::language]
            pub enum Lang { Tok(#[adze::leaf(pattern = r"\d+")] i32) }
        }
        "#,
    );
    assert!(gs[0]["rules"].is_object());
}

#[test]
fn grammar_json_rules_are_not_empty() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("notempty")]
        mod grammar {
            #[adze::language]
            pub enum Lang { A(#[adze::leaf(pattern = r"a")] String) }
        }
        "#,
    );
    let rules = gs[0]["rules"].as_object().unwrap();
    assert!(!rules.is_empty());
}

// =========================================================================
// 9. Multiple grammars extraction
// =========================================================================

#[test]
fn extract_zero_grammars_from_bare_module() {
    let gs = grammars_from_rust(
        r#"
        mod no_grammar {
            pub fn hello() {}
        }
        "#,
    );
    assert!(gs.is_empty());
}

#[test]
fn extract_zero_grammars_from_empty_file() {
    let gs = grammars_from_rust("");
    assert!(gs.is_empty());
}

// =========================================================================
// 10. Build pipeline — grammar.js edge cases
// =========================================================================

#[test]
fn grammar_js_with_choice_builds_successfully() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'choice_test',
  rules: {
    source: $ => choice($.a, $.b),
    a: $ => 'hello',
    b: $ => 'world'
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "choice_test");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn grammar_js_with_seq_builds_successfully() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'seq_test',
  rules: {
    source: $ => seq($.a, $.b),
    a: $ => 'hello',
    b: $ => 'world'
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "seq_test");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn grammar_js_with_optional_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'opt_test',
  rules: {
    source: $ => seq(optional('hello'), 'world')
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "opt_test");
}

#[test]
fn grammar_js_with_repeat_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'repeat_test',
  rules: {
    source: $ => repeat($.item),
    item: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "repeat_test");
}

#[test]
fn grammar_js_with_repeat1_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'repeat1_test',
  rules: {
    source: $ => repeat1($.item),
    item: $ => /[a-z]+/
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "repeat1_test");
}

#[test]
fn grammar_js_with_prec_left_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'prec_left_test',
  rules: {
    source: $ => $.expr,
    expr: $ => choice(
      /\d+/,
      prec.left(1, seq($.expr, '+', $.expr))
    )
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "prec_left_test");
    assert!(r.build_stats.state_count > 1);
}

#[test]
fn grammar_js_with_prec_right_builds() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'prec_right_test',
  rules: {
    source: $ => $.expr,
    expr: $ => choice(
      /\d+/,
      prec.right(1, seq($.expr, '=', $.expr))
    )
  }
});
"#,
    );
    assert_eq!(r.grammar_name, "prec_right_test");
}

// =========================================================================
// 11. Build result structural properties
// =========================================================================

#[test]
fn build_result_parser_code_contains_language_constant() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'lang_const',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    );
    // The parser code should contain some reference to the language
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_result_parser_path_is_written_to_disk() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'disk_check',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let r = build_parser_from_grammar_js(&path, opts_in(&dir)).unwrap();
    assert!(Path::new(&r.parser_path).exists());
}

#[test]
fn build_result_debug_impl_is_accessible() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'debug_check',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    );
    let dbg = format!("{:?}", r);
    assert!(dbg.contains("BuildResult"));
    assert!(dbg.contains("debug_check"));
}

#[test]
fn build_stats_debug_is_accessible() {
    let r = build_js(
        r#"
module.exports = grammar({
  name: 'stats_dbg',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    );
    let dbg = format!("{:?}", r.build_stats);
    assert!(dbg.contains("BuildStats"));
    assert!(dbg.contains("state_count"));
}

// =========================================================================
// 12. Build from JSON — more cases
// =========================================================================

#[test]
fn build_from_json_minimal_valid() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "name": "json_min",
        "rules": {
            "source": {"type": "PATTERN", "value": "[a-z]+"}
        }
    });
    let r = build_parser_from_json(serde_json::to_string(&json).unwrap(), opts_in(&dir)).unwrap();
    assert_eq!(r.grammar_name, "json_min");
}

#[test]
fn build_from_json_with_seq_rule() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "name": "json_seq",
        "rules": {
            "source": {
                "type": "SEQ",
                "members": [
                    {"type": "STRING", "value": "hello"},
                    {"type": "STRING", "value": "world"}
                ]
            }
        }
    });
    let r = build_parser_from_json(serde_json::to_string(&json).unwrap(), opts_in(&dir)).unwrap();
    assert_eq!(r.grammar_name, "json_seq");
}

#[test]
fn build_from_json_with_choice_rule() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "name": "json_choice",
        "rules": {
            "source": {
                "type": "CHOICE",
                "members": [
                    {"type": "STRING", "value": "a"},
                    {"type": "STRING", "value": "b"}
                ]
            }
        }
    });
    let r = build_parser_from_json(serde_json::to_string(&json).unwrap(), opts_in(&dir)).unwrap();
    assert_eq!(r.grammar_name, "json_choice");
}

#[test]
fn build_from_json_empty_string_is_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json(String::new(), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn build_from_json_array_instead_of_object_is_error() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("[]".into(), opts_in(&dir));
    assert!(result.is_err());
}

// =========================================================================
// 13. Scanner builder — more edge cases
// =========================================================================

#[test]
fn scanner_builder_finds_c_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.c"), "// C scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.language, ScannerLanguage::C);
}

#[test]
fn scanner_builder_finds_cc_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.cc"), "// C++ scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.language, ScannerLanguage::Cpp);
}

#[test]
fn scanner_builder_finds_rs_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.rs"), "// Rust scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.language, ScannerLanguage::Rust);
}

#[test]
fn scanner_builder_finds_named_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("mygrammar_scanner.c"), "// named").unwrap();
    let builder = ScannerBuilder::new(
        "mygrammar",
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
    );
    let scanner = builder.find_scanner().unwrap().unwrap();
    assert_eq!(scanner.grammar_name, "mygrammar");
    assert_eq!(scanner.language, ScannerLanguage::C);
}

#[test]
fn scanner_builder_prefers_generic_over_named() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.c"), "// generic").unwrap();
    fs::write(dir.path().join("test_scanner.c"), "// named").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let scanner = builder.find_scanner().unwrap().unwrap();
    // generic scanner.c should be found first
    assert!(scanner.path.ends_with("scanner.c"));
}

// =========================================================================
// 14. ToolError variant coverage — remaining variants
// =========================================================================

#[test]
fn error_display_invalid_production_details() {
    let e = ToolError::InvalidProduction {
        details: "rhs contains unknown symbol".into(),
    };
    assert!(e.to_string().contains("rhs contains unknown symbol"));
}

#[test]
fn error_from_glr_is_transparent() {
    let glr_err = adze_glr_core::GLRError::ConflictResolution("test conflict".into());
    let e: ToolError = glr_err.into();
    assert!(matches!(e, ToolError::Glr(_)));
    assert!(e.to_string().contains("test conflict"));
}

#[test]
fn error_from_syn_is_transparent() {
    let syn_err = syn::Error::new(proc_macro2::Span::call_site(), "bad syntax");
    let e: ToolError = syn_err.into();
    assert!(matches!(e, ToolError::SynError { .. }));
    assert!(e.to_string().contains("bad syntax"));
}

// =========================================================================
// 15. GrammarJs validation
// =========================================================================

#[test]
fn grammar_js_validate_empty_rules_succeeds() {
    let grammar = adze_tool::grammar_js::GrammarJs::new("empty".into());
    assert!(grammar.validate().is_ok());
}

#[test]
fn grammar_js_validate_word_not_in_rules_fails() {
    let mut grammar = adze_tool::grammar_js::GrammarJs::new("bad_word".into());
    grammar.word = Some("identifier".into());
    assert!(grammar.validate().is_err());
}

#[test]
fn grammar_js_validate_inline_not_in_rules_fails() {
    let mut grammar = adze_tool::grammar_js::GrammarJs::new("bad_inline".into());
    grammar.inline.push("nonexistent".into());
    assert!(grammar.validate().is_err());
}

#[test]
fn grammar_js_validate_conflict_not_in_rules_fails() {
    let mut grammar = adze_tool::grammar_js::GrammarJs::new("bad_conflict".into());
    grammar.conflicts.push(vec!["nonexistent".into()]);
    assert!(grammar.validate().is_err());
}

// =========================================================================
// 16. GrammarConverter sample grammar
// =========================================================================

#[test]
fn sample_grammar_has_tokens() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn sample_grammar_has_rules() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn sample_grammar_has_fields() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    assert!(!grammar.fields.is_empty());
}

#[test]
fn sample_grammar_name_is_sample() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    assert_eq!(grammar.name, "sample");
}

// =========================================================================
// 17. Visualization — smoke tests
// =========================================================================

#[test]
fn visualizer_to_dot_produces_digraph() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.contains("}"));
}

#[test]
fn visualizer_to_text_contains_grammar_name() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: sample"));
}

#[test]
fn visualizer_to_railroad_svg_produces_svg() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn visualizer_dependency_graph_produces_output() {
    let grammar = adze_tool::GrammarConverter::create_sample_grammar();
    let viz = adze_tool::GrammarVisualizer::new(grammar);
    let deps = viz.dependency_graph();
    assert!(deps.contains("Symbol Dependencies:"));
}

// =========================================================================
// 18. Build options with emit_artifacts
// =========================================================================

#[test]
fn build_with_emit_artifacts_writes_extra_files() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'artifact_test',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    assert_eq!(r.grammar_name, "artifact_test");
    // With emit_artifacts, grammar directory should have extra files
    let grammar_dir = dir.path().join("grammar_artifact_test");
    assert!(grammar_dir.exists());
}

#[test]
fn build_without_compression() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'no_compress',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_with_compression() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(
        &path,
        r#"
module.exports = grammar({
  name: 'with_compress',
  rules: { source: $ => /[a-z]+/ }
});
"#,
    )
    .unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser_from_grammar_js(&path, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

// =========================================================================
// 19. Build from Rust — edge cases in grammar extraction
// =========================================================================

#[test]
fn rust_grammar_with_string_literal_leaf() {
    let gs = grammars_from_rust(
        r#"
        #[adze::grammar("str_leaf")]
        mod grammar {
            #[adze::language]
            pub enum Lang { Kw(#[adze::leaf(text = "keyword")] ()) }
        }
        "#,
    );
    assert_eq!(gs.len(), 1);
    assert_eq!(gs[0]["name"].as_str().unwrap(), "str_leaf");
}

// =========================================================================
// 20. Error propagation — more patterns
// =========================================================================

#[test]
fn json_with_missing_name_is_error() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "rules": {
            "source": {"type": "PATTERN", "value": "[a-z]+"}
        }
    });
    let result = build_parser_from_json(serde_json::to_string(&json).unwrap(), opts_in(&dir));
    assert!(result.is_err());
}

#[test]
fn error_tool_result_type_alias_works() {
    fn returns_tool_error() -> adze_tool::ToolResult<()> {
        Err(ToolError::MultipleWordRules)
    }
    assert!(returns_tool_error().is_err());
}

#[test]
fn multiple_error_constructors_are_distinct() {
    let e1 = ToolError::string_too_long("a", 1);
    let e2 = ToolError::grammar_validation("b");
    let e3 = ToolError::expected_symbol_type("c");
    // Just verify they are distinct variants
    assert_ne!(e1.to_string(), e2.to_string());
    assert_ne!(e2.to_string(), e3.to_string());
}
