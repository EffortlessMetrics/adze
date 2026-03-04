//! Tests for the pure Rust builder module — BuildOptions, BuildStats, build pipeline.

use adze_tool::GrammarConverter;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

#[test]
fn build_options_default() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn build_options_custom() {
    let opts = BuildOptions {
        out_dir: "/tmp/test-build".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/tmp/test-build");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn build_parser_produces_result() {
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/adze-test-build".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "build_parser should succeed for sample grammar: {result:?}"
    );
}

#[test]
fn build_result_has_stats() {
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/adze-test-build-stats".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(grammar, opts).unwrap();
    let debug = format!("{:?}", result.build_stats);
    assert!(!debug.is_empty());
}

#[test]
fn build_parser_without_compression() {
    let grammar = GrammarConverter::create_sample_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/adze-test-build-nocompress".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser(grammar, opts);
    assert!(
        result.is_ok(),
        "build without compression should succeed: {result:?}"
    );
}
