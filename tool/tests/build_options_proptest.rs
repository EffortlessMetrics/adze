#![allow(clippy::needless_range_loop)]

//! Property-based tests for build options and configuration in adze-tool.

use adze_tool::cli::OutputFormat;
use adze_tool::error::ToolError;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, BuildStats};
use adze_tool::scanner_build::{ScannerLanguage, ScannerSource};
use proptest::prelude::*;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn build_options_strategy() -> impl Strategy<Value = BuildOptions> {
    ("[a-zA-Z0-9/_\\-]{0,64}", any::<bool>(), any::<bool>()).prop_map(
        |(out_dir, emit_artifacts, compress_tables)| BuildOptions {
            out_dir,
            emit_artifacts,
            compress_tables,
        },
    )
}

fn build_stats_strategy() -> impl Strategy<Value = BuildStats> {
    (0usize..10_000, 0usize..1_000, 0usize..5_000).prop_map(
        |(state_count, symbol_count, conflict_cells)| BuildStats {
            state_count,
            symbol_count,
            conflict_cells,
        },
    )
}

fn build_result_strategy() -> impl Strategy<Value = BuildResult> {
    (
        "[a-z][a-z0-9_]{0,15}",
        "[a-zA-Z0-9/_\\-]{1,32}",
        "[a-zA-Z0-9 (){};\\n]{0,128}",
        "\\{[a-zA-Z0-9:, \"]*\\}",
        build_stats_strategy(),
    )
        .prop_map(
            |(grammar_name, parser_path, parser_code, node_types_json, build_stats)| BuildResult {
                grammar_name,
                parser_path,
                parser_code,
                node_types_json,
                build_stats,
            },
        )
}

fn scanner_language_strategy() -> impl Strategy<Value = ScannerLanguage> {
    prop_oneof![
        Just(ScannerLanguage::C),
        Just(ScannerLanguage::Cpp),
        Just(ScannerLanguage::Rust),
    ]
}

fn scanner_source_strategy() -> impl Strategy<Value = ScannerSource> {
    (
        "[a-z][a-z0-9_]{0,15}\\.(c|cc|rs)",
        scanner_language_strategy(),
        "[a-z][a-z0-9_]{0,15}",
    )
        .prop_map(|(path, language, grammar_name)| ScannerSource {
            path: PathBuf::from(path),
            language,
            grammar_name,
        })
}

// ---------------------------------------------------------------------------
// 1. BuildOptions — defaults and stability
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn default_options_are_stable(_seed in 0u32..1000) {
        let a = BuildOptions::default();
        let b = BuildOptions::default();
        // Default values should be consistent across calls
        prop_assert_eq!(a.compress_tables, b.compress_tables);
        prop_assert_eq!(a.emit_artifacts, b.emit_artifacts);
        // out_dir depends on env, but should be equal in same process
        prop_assert_eq!(&a.out_dir, &b.out_dir);
    }
}

proptest! {
    #[test]
    fn default_compress_tables_is_true(_seed in 0u32..100) {
        let opts = BuildOptions::default();
        prop_assert!(opts.compress_tables);
    }
}

proptest! {
    #[test]
    fn default_emit_artifacts_is_false(_seed in 0u32..100) {
        let opts = BuildOptions::default();
        prop_assert!(!opts.emit_artifacts);
    }
}

// ---------------------------------------------------------------------------
// 2. BuildOptions — Clone behavior
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_preserves_all_fields(opts in build_options_strategy()) {
        let cloned = opts.clone();
        prop_assert_eq!(&opts.out_dir, &cloned.out_dir);
        prop_assert_eq!(opts.emit_artifacts, cloned.emit_artifacts);
        prop_assert_eq!(opts.compress_tables, cloned.compress_tables);
    }
}

proptest! {
    #[test]
    fn clone_is_independent(opts in build_options_strategy()) {
        let mut cloned = opts.clone();
        cloned.emit_artifacts = !cloned.emit_artifacts;
        // Mutation of clone must not affect original
        prop_assert_ne!(opts.emit_artifacts, cloned.emit_artifacts);
    }
}

proptest! {
    #[test]
    fn double_clone_preserves_fields(opts in build_options_strategy()) {
        let c1 = opts.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&opts.out_dir, &c2.out_dir);
        prop_assert_eq!(opts.emit_artifacts, c2.emit_artifacts);
        prop_assert_eq!(opts.compress_tables, c2.compress_tables);
    }
}

// ---------------------------------------------------------------------------
// 3. BuildOptions — Debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_format_contains_struct_name(opts in build_options_strategy()) {
        let debug_str = format!("{:?}", opts);
        prop_assert!(debug_str.contains("BuildOptions"));
    }
}

proptest! {
    #[test]
    fn debug_format_contains_field_names(opts in build_options_strategy()) {
        let debug_str = format!("{:?}", opts);
        prop_assert!(debug_str.contains("out_dir"));
        prop_assert!(debug_str.contains("emit_artifacts"));
        prop_assert!(debug_str.contains("compress_tables"));
    }
}

proptest! {
    #[test]
    fn debug_format_reflects_bool_values(
        out_dir in "[a-z]{1,8}",
        emit in any::<bool>(),
        compress in any::<bool>(),
    ) {
        let opts = BuildOptions {
            out_dir,
            emit_artifacts: emit,
            compress_tables: compress,
        };
        let debug_str = format!("{:?}", opts);
        let expected_emit = format!("emit_artifacts: {}", emit);
        let expected_compress = format!("compress_tables: {}", compress);
        prop_assert!(debug_str.contains(&expected_emit));
        prop_assert!(debug_str.contains(&expected_compress));
    }
}

proptest! {
    #[test]
    fn debug_format_is_nonempty(opts in build_options_strategy()) {
        let debug_str = format!("{:?}", opts);
        prop_assert!(!debug_str.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 4. BuildOptions — all combinations valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_bool_combinations_produce_valid_options(
        out_dir in "[a-z]{1,8}",
        emit in any::<bool>(),
        compress in any::<bool>(),
    ) {
        let opts = BuildOptions {
            out_dir: out_dir.clone(),
            emit_artifacts: emit,
            compress_tables: compress,
        };
        prop_assert_eq!(&opts.out_dir, &out_dir);
        prop_assert_eq!(opts.emit_artifacts, emit);
        prop_assert_eq!(opts.compress_tables, compress);
    }
}

proptest! {
    #[test]
    fn empty_out_dir_is_accepted(emit in any::<bool>(), compress in any::<bool>()) {
        let opts = BuildOptions {
            out_dir: String::new(),
            emit_artifacts: emit,
            compress_tables: compress,
        };
        prop_assert!(opts.out_dir.is_empty());
    }
}

proptest! {
    #[test]
    fn long_out_dir_is_accepted(
        segment in "[a-z]{1,10}",
        depth in 1usize..20,
    ) {
        let out_dir: String = std::iter::repeat(segment.as_str())
            .take(depth)
            .collect::<Vec<_>>()
            .join("/");
        let opts = BuildOptions {
            out_dir: out_dir.clone(),
            emit_artifacts: false,
            compress_tables: true,
        };
        prop_assert_eq!(&opts.out_dir, &out_dir);
    }
}

// ---------------------------------------------------------------------------
// 5. BuildStats — Debug formatting and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn build_stats_debug_contains_field_names(stats in build_stats_strategy()) {
        let debug_str = format!("{:?}", stats);
        prop_assert!(debug_str.contains("state_count"));
        prop_assert!(debug_str.contains("symbol_count"));
        prop_assert!(debug_str.contains("conflict_cells"));
    }
}

proptest! {
    #[test]
    fn build_stats_fields_round_trip(
        state_count in 0usize..100_000,
        symbol_count in 0usize..10_000,
        conflict_cells in 0usize..50_000,
    ) {
        let stats = BuildStats {
            state_count,
            symbol_count,
            conflict_cells,
        };
        prop_assert_eq!(stats.state_count, state_count);
        prop_assert_eq!(stats.symbol_count, symbol_count);
        prop_assert_eq!(stats.conflict_cells, conflict_cells);
    }
}

proptest! {
    #[test]
    fn build_stats_debug_contains_values(stats in build_stats_strategy()) {
        let debug_str = format!("{:?}", stats);
        prop_assert!(debug_str.contains(&stats.state_count.to_string()));
        prop_assert!(debug_str.contains(&stats.symbol_count.to_string()));
        prop_assert!(debug_str.contains(&stats.conflict_cells.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 6. BuildResult — Debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn build_result_debug_contains_struct_name(result in build_result_strategy()) {
        let debug_str = format!("{:?}", result);
        prop_assert!(debug_str.contains("BuildResult"));
    }
}

proptest! {
    #[test]
    fn build_result_debug_contains_grammar_name(result in build_result_strategy()) {
        let debug_str = format!("{:?}", result);
        prop_assert!(debug_str.contains(&result.grammar_name));
    }
}

proptest! {
    #[test]
    fn build_result_fields_accessible(result in build_result_strategy()) {
        prop_assert!(!result.grammar_name.is_empty());
        prop_assert!(!result.parser_path.is_empty());
        // parser_code and node_types_json can be any string
        let _ = &result.parser_code;
        let _ = &result.node_types_json;
        let _ = result.build_stats.state_count;
    }
}

// ---------------------------------------------------------------------------
// 7. ScannerLanguage — extension mapping
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn scanner_language_extension_is_nonempty(lang in scanner_language_strategy()) {
        prop_assert!(!lang.extension().is_empty());
    }
}

proptest! {
    #[test]
    fn scanner_language_extension_is_deterministic(lang in scanner_language_strategy()) {
        let ext1 = lang.extension();
        let ext2 = lang.extension();
        prop_assert_eq!(ext1, ext2);
    }
}

proptest! {
    #[test]
    fn scanner_language_extensions_are_valid(lang in scanner_language_strategy()) {
        let ext = lang.extension();
        let valid = ["c", "cc", "rs"];
        prop_assert!(valid.contains(&ext), "unexpected extension: {}", ext);
    }
}

proptest! {
    #[test]
    fn scanner_language_debug_is_nonempty(lang in scanner_language_strategy()) {
        let debug_str = format!("{:?}", lang);
        prop_assert!(!debug_str.is_empty());
    }
}

proptest! {
    #[test]
    fn scanner_language_clone_eq(lang in scanner_language_strategy()) {
        let cloned = lang;
        prop_assert_eq!(lang, cloned);
    }
}

// ---------------------------------------------------------------------------
// 8. ScannerSource — Clone and Debug
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn scanner_source_clone_preserves_fields(src in scanner_source_strategy()) {
        let cloned = src.clone();
        prop_assert_eq!(src.path, cloned.path);
        prop_assert_eq!(src.language, cloned.language);
        prop_assert_eq!(&src.grammar_name, &cloned.grammar_name);
    }
}

proptest! {
    #[test]
    fn scanner_source_debug_contains_grammar_name(src in scanner_source_strategy()) {
        let debug_str = format!("{:?}", src);
        prop_assert!(debug_str.contains(&src.grammar_name));
    }
}

// ---------------------------------------------------------------------------
// 9. ToolError — Display formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn tool_error_display_is_nonempty(msg in "[a-zA-Z0-9 ]{1,50}") {
        let err = ToolError::Other(msg);
        let display = format!("{}", err);
        prop_assert!(!display.is_empty());
    }
}

proptest! {
    #[test]
    fn tool_error_other_roundtrips_message(msg in "[a-zA-Z0-9 ]{1,50}") {
        let err = ToolError::Other(msg.clone());
        let display = format!("{}", err);
        prop_assert_eq!(&display, &msg);
    }
}

proptest! {
    #[test]
    fn tool_error_string_too_long_contains_length(
        op in "[a-z]{1,10}",
        len in 0usize..10_000,
    ) {
        let err = ToolError::string_too_long(&op, len);
        let display = format!("{}", err);
        prop_assert!(display.contains(&len.to_string()));
        prop_assert!(display.contains(&op));
    }
}

proptest! {
    #[test]
    fn tool_error_grammar_validation_contains_reason(reason in "[a-zA-Z0-9 ]{1,40}") {
        let err = ToolError::grammar_validation(&reason);
        let display = format!("{}", err);
        prop_assert!(display.contains(&reason));
    }
}

// ---------------------------------------------------------------------------
// 10. OutputFormat — Debug and Clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn output_format_debug_is_nonempty(
        variant in prop_oneof![
            Just(OutputFormat::Tree),
            Just(OutputFormat::Sexp),
            Just(OutputFormat::Json),
            Just(OutputFormat::Dot),
        ]
    ) {
        let debug_str = format!("{:?}", variant);
        prop_assert!(!debug_str.is_empty());
    }
}

proptest! {
    #[test]
    fn output_format_clone_debug_matches(
        variant in prop_oneof![
            Just(OutputFormat::Tree),
            Just(OutputFormat::Sexp),
            Just(OutputFormat::Json),
            Just(OutputFormat::Dot),
        ]
    ) {
        let cloned = variant.clone();
        prop_assert_eq!(format!("{:?}", variant), format!("{:?}", cloned));
    }
}
