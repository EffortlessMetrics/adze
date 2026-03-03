#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for BuildOptions, BuildResult, BuildStats, and build configuration
//! in the adze-tool crate.

use std::fs;
use std::path::Path;

use adze_tool::GrammarConverter;
use adze_tool::pure_rust_builder::{
    BuildOptions, build_parser, build_parser_from_grammar_js, build_parser_from_json,
};
use adze_tool::scanner_build::{ScannerBuilder, ScannerLanguage};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_opts(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn simple_grammar_js() -> &'static str {
    r#"
module.exports = grammar({
  name: 'simple',
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/
  }
});
"#
}

fn build_grammar_js(js: &str, opts: BuildOptions) -> adze_tool::pure_rust_builder::BuildResult {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    build_parser_from_grammar_js(&path, opts).unwrap()
}

// =========================================================================
// 1. BuildOptions — defaults and construction
// =========================================================================

#[test]
fn default_options_compress_tables_is_true() {
    let opts = BuildOptions::default();
    assert!(
        opts.compress_tables,
        "compress_tables should default to true"
    );
}

#[test]
fn default_options_emit_artifacts_is_false() {
    let opts = BuildOptions::default();
    assert!(
        !opts.emit_artifacts,
        "emit_artifacts should default to false"
    );
}

#[test]
fn default_options_out_dir_is_nonempty() {
    let opts = BuildOptions::default();
    assert!(
        !opts.out_dir.is_empty(),
        "out_dir should have a fallback value"
    );
}

#[test]
fn custom_options_all_fields() {
    let opts = BuildOptions {
        out_dir: "/custom/output".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/custom/output");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn options_clone_is_independent() {
    let opts = BuildOptions {
        out_dir: "/a".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let mut cloned = opts.clone();
    cloned.out_dir = "/b".to_string();
    cloned.emit_artifacts = false;
    assert_eq!(opts.out_dir, "/a");
    assert!(opts.emit_artifacts);
}

#[test]
fn options_debug_impl() {
    let opts = BuildOptions {
        out_dir: "/tmp/dbg".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let dbg = format!("{:?}", opts);
    assert!(dbg.contains("out_dir"));
    assert!(dbg.contains("emit_artifacts"));
    assert!(dbg.contains("compress_tables"));
}

// =========================================================================
// 2. BuildResult & BuildStats — structure and content
// =========================================================================

#[test]
fn build_result_has_grammar_name() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert_eq!(result.grammar_name, "sample");
}

#[test]
fn build_result_parser_code_nonempty() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_result_node_types_json_valid() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn build_result_parser_path_exists() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert!(
        Path::new(&result.parser_path).exists(),
        "parser file should be written to disk"
    );
}

#[test]
fn build_stats_state_count_positive() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert!(
        result.build_stats.state_count > 0,
        "parse table must have at least one state"
    );
}

#[test]
fn build_stats_symbol_count_positive() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    assert!(
        result.build_stats.symbol_count > 0,
        "parse table must have at least one symbol"
    );
}

#[test]
fn build_stats_debug_impl() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    let dbg = format!("{:?}", result.build_stats);
    assert!(dbg.contains("state_count"));
    assert!(dbg.contains("symbol_count"));
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn build_result_debug_impl() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    let dbg = format!("{:?}", result);
    assert!(dbg.contains("grammar_name"));
    assert!(dbg.contains("build_stats"));
}

// =========================================================================
// 3. Compression toggle
// =========================================================================

#[test]
fn build_with_compression_succeeds() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let mut opts = temp_opts(&dir);
    opts.compress_tables = true;
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "compressed build should succeed: {result:?}"
    );
}

#[test]
fn build_without_compression_succeeds() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let mut opts = temp_opts(&dir);
    opts.compress_tables = false;
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "uncompressed build should succeed: {result:?}"
    );
}

#[test]
fn compressed_and_uncompressed_same_grammar_name() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let g1 = GrammarConverter::create_sample_grammar();
    let g2 = GrammarConverter::create_sample_grammar();

    let mut opts1 = temp_opts(&dir1);
    opts1.compress_tables = true;
    let mut opts2 = temp_opts(&dir2);
    opts2.compress_tables = false;

    let r1 = build_parser(g1, opts1).unwrap();
    let r2 = build_parser(g2, opts2).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// =========================================================================
// 4. Artifact emission
// =========================================================================

#[test]
fn emit_artifacts_creates_grammar_directory() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let _result = build_parser(grammar, opts).unwrap();
    let grammar_dir = dir.path().join("grammar_sample");
    assert!(grammar_dir.exists(), "grammar dir should be created");
}

#[test]
fn emit_artifacts_writes_ir_json() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let _result = build_parser(grammar, opts).unwrap();
    let ir_path = dir.path().join("grammar_sample/grammar.ir.json");
    assert!(ir_path.exists(), "grammar IR JSON should be emitted");
    let content = fs::read_to_string(&ir_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn emit_artifacts_writes_node_types_json() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let _result = build_parser(grammar, opts).unwrap();
    let nt_path = dir.path().join("grammar_sample/NODE_TYPES.json");
    assert!(nt_path.exists(), "NODE_TYPES.json should be emitted");
}

#[test]
fn no_emit_artifacts_skips_ir_json() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let _result = build_parser(grammar, opts).unwrap();
    let ir_path = dir.path().join("grammar_sample/grammar.ir.json");
    assert!(!ir_path.exists(), "IR JSON should NOT be emitted");
}

// =========================================================================
// 5. Parser output file naming
// =========================================================================

#[test]
fn parser_file_uses_lowercase_grammar_name() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    let filename = Path::new(&result.parser_path)
        .file_name()
        .unwrap()
        .to_string_lossy();
    assert!(
        filename.contains("sample"),
        "parser filename should contain grammar name"
    );
    assert!(
        filename.ends_with(".rs"),
        "parser file should have .rs extension"
    );
}

#[test]
fn parser_file_contains_grammar_name_constant() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    let content = fs::read_to_string(&result.parser_path).unwrap();
    assert!(
        content.contains("GRAMMAR_NAME"),
        "parser file should declare GRAMMAR_NAME"
    );
    assert!(
        content.contains("\"sample\""),
        "GRAMMAR_NAME should be the grammar name"
    );
}

// =========================================================================
// 6. build_parser_from_grammar_js
// =========================================================================

#[test]
fn build_from_grammar_js_returns_correct_name() {
    let dir = TempDir::new().unwrap();
    let result = build_grammar_js(simple_grammar_js(), temp_opts(&dir));
    assert_eq!(result.grammar_name, "simple");
}

#[test]
fn build_from_grammar_js_generates_parser_code() {
    let dir = TempDir::new().unwrap();
    let result = build_grammar_js(simple_grammar_js(), temp_opts(&dir));
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_from_grammar_js_nonexistent_file_errors() {
    let opts = BuildOptions {
        out_dir: "/tmp".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_grammar_js(Path::new("/no/such/grammar.js"), opts);
    assert!(result.is_err(), "missing file should produce an error");
}

// =========================================================================
// 7. build_parser_from_json
// =========================================================================

#[test]
fn build_from_json_simple_grammar() {
    let dir = TempDir::new().unwrap();
    let json = serde_json::json!({
        "name": "json_test",
        "word": null,
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "value"
            },
            "value": {
                "type": "PATTERN",
                "value": "\\d+"
            }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ],
        "conflicts": [],
        "precedences": [],
        "externals": [],
        "inline": [],
        "supertypes": []
    });
    let result = build_parser_from_json(json.to_string(), temp_opts(&dir));
    assert!(result.is_ok(), "JSON build should succeed: {result:?}");
    let r = result.unwrap();
    assert_eq!(r.grammar_name, "json_test");
}

#[test]
fn build_from_json_invalid_json_errors() {
    let dir = TempDir::new().unwrap();
    let result = build_parser_from_json("not json".to_string(), temp_opts(&dir));
    assert!(result.is_err(), "invalid JSON should error");
}

// =========================================================================
// 8. ScannerBuilder & ScannerLanguage configuration
// =========================================================================

#[test]
fn scanner_language_extensions() {
    assert_eq!(ScannerLanguage::C.extension(), "c");
    assert_eq!(ScannerLanguage::Cpp.extension(), "cc");
    assert_eq!(ScannerLanguage::Rust.extension(), "rs");
}

#[test]
fn scanner_language_equality() {
    assert_eq!(ScannerLanguage::C, ScannerLanguage::C);
    assert_ne!(ScannerLanguage::C, ScannerLanguage::Rust);
}

#[test]
fn scanner_builder_find_no_scanner() {
    let dir = TempDir::new().unwrap();
    let builder = ScannerBuilder::new(
        "test_grammar",
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
    );
    let result = builder.find_scanner().unwrap();
    assert!(result.is_none(), "no scanner should be found in empty dir");
}

#[test]
fn scanner_builder_find_c_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.c"), "// scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    let src = result.unwrap();
    assert_eq!(src.language, ScannerLanguage::C);
    assert_eq!(src.grammar_name, "test");
}

#[test]
fn scanner_builder_find_rust_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("scanner.rs"), "// scanner").unwrap();
    let builder = ScannerBuilder::new(
        "my_lang",
        dir.path().to_path_buf(),
        dir.path().to_path_buf(),
    );
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().language, ScannerLanguage::Rust);
}

#[test]
fn scanner_builder_find_named_scanner() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("python_scanner.c"), "// scanner").unwrap();
    let builder = ScannerBuilder::new("python", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    let src = result.unwrap();
    assert_eq!(src.language, ScannerLanguage::C);
    assert_eq!(src.grammar_name, "python");
}

// =========================================================================
// 9. Repeated builds / idempotency
// =========================================================================

#[test]
fn rebuild_same_grammar_overwrites_parser() {
    let dir = TempDir::new().unwrap();
    let g1 = GrammarConverter::create_sample_grammar();
    let g2 = GrammarConverter::create_sample_grammar();

    let r1 = build_parser(g1, temp_opts(&dir)).unwrap();
    let r2 = build_parser(g2, temp_opts(&dir)).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
    assert!(
        Path::new(&r2.parser_path).exists(),
        "second build should produce a file"
    );
}

// =========================================================================
// 10. Edge cases
// =========================================================================

#[test]
fn build_parser_with_deeply_nested_out_dir() {
    let dir = TempDir::new().unwrap();
    let deep = dir.path().join("a/b/c/d/e");
    fs::create_dir_all(&deep).unwrap();
    let opts = BuildOptions {
        out_dir: deep.to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "deeply nested out_dir should work: {result:?}"
    );
}

#[test]
fn build_stats_conflict_cells_non_negative() {
    let dir = TempDir::new().unwrap();
    let grammar = GrammarConverter::create_sample_grammar();
    let result = build_parser(grammar, temp_opts(&dir)).unwrap();
    // conflict_cells is usize, so always >= 0, but verify it's a reasonable value
    assert!(
        result.build_stats.conflict_cells
            <= result.build_stats.state_count * result.build_stats.symbol_count,
        "conflict cells cannot exceed total cells"
    );
}
