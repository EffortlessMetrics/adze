//! Comprehensive tests for BuildOptions and BuildStats types.

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildStats, build_parser};

// ── BuildOptions tests ──

#[test]
fn build_options_default() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn build_options_custom_out_dir() {
    let opts = BuildOptions {
        out_dir: "/tmp/test".to_string(),
        ..BuildOptions::default()
    };
    assert_eq!(opts.out_dir, "/tmp/test");
}

#[test]
fn build_options_emit_artifacts_off() {
    let opts = BuildOptions {
        emit_artifacts: false,
        ..BuildOptions::default()
    };
    assert!(!opts.emit_artifacts);
}

#[test]
fn build_options_emit_artifacts_on() {
    let opts = BuildOptions {
        emit_artifacts: true,
        ..BuildOptions::default()
    };
    assert!(opts.emit_artifacts);
}

#[test]
fn build_options_compress_off() {
    let opts = BuildOptions {
        compress_tables: false,
        ..BuildOptions::default()
    };
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_all_custom() {
    let opts = BuildOptions {
        out_dir: "/custom".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/custom");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

// ── build_parser tests ──

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

#[test]
fn build_simple_grammar() {
    let g = simple_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "simple");
}

#[test]
fn build_result_has_parser_code() {
    let g = simple_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_result_has_node_types_json() {
    let g = simple_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_result_stats_nonzero() {
    let g = simple_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_two_alt_grammar() {
    let g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test2".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_chain_grammar() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test3".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_result_name_matches() {
    let g = GrammarBuilder::new("myparser")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test4".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    assert_eq!(result.grammar_name, "myparser");
}

#[test]
fn build_node_types_is_json() {
    let g = simple_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test5".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser(g, opts).unwrap();
    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn build_without_compress() {
    let g = simple_grammar();
    let opts = BuildOptions {
        out_dir: "/tmp/build_test6".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser(g, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let opts1 = BuildOptions {
        out_dir: "/tmp/build_det1".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let opts2 = BuildOptions {
        out_dir: "/tmp/build_det2".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r1 = build_parser(g1, opts1).unwrap();
    let r2 = build_parser(g2, opts2).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}
