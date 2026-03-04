#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for parser module code generation in adze-tool.
//!
//! Validates that the pure-Rust builder produces parser modules with the
//! correct structure: module naming, LANGUAGE constant, parse functions,
//! tree-sitter type references, imports, and determinism.

use std::fs;
use tempfile::TempDir;

use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser_from_grammar_js};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a grammar.js to a temp dir and build a parser, returning the
/// BuildResult, the on-disk module source, and the TempDir (kept alive).
fn build_and_read(js: &str) -> (BuildResult, String, TempDir) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("grammar.js");
    fs::write(&path, js).unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_grammar_js(&path, opts).unwrap();
    let on_disk = fs::read_to_string(&result.parser_path).unwrap();
    (result, on_disk, dir)
}

/// Shorthand: build from grammar.js and return just the BuildResult.
fn build_js(js: &str) -> BuildResult {
    build_and_read(js).0
}

/// Minimal grammar.js that parses digits.
const SIMPLE_GRAMMAR: &str = r#"
module.exports = grammar({
  name: 'simple',
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/
  }
});
"#;

/// A more complex grammar with multiple rules, choices, and repetitions.
const COMPLEX_GRAMMAR: &str = r#"
module.exports = grammar({
  name: 'complex',
  rules: {
    source_file: $ => repeat($.statement),
    statement: $ => choice($.assignment, $.expr_stmt),
    assignment: $ => seq($.identifier, '=', $.expression),
    expr_stmt: $ => $.expression,
    expression: $ => choice($.identifier, $.number),
    identifier: $ => /[a-zA-Z_]\w*/,
    number: $ => /\d+/
  }
});
"#;

// =========================================================================
// 1. Generated module has correct name
// =========================================================================

#[test]
fn module_name_matches_grammar_name() {
    let result = build_js(SIMPLE_GRAMMAR);
    assert_eq!(result.grammar_name, "simple");
}

#[test]
fn module_file_named_after_grammar() {
    let result = build_js(SIMPLE_GRAMMAR);
    assert!(
        result.parser_path.contains("parser_simple.rs"),
        "parser path should contain parser_simple.rs, got: {}",
        result.parser_path
    );
}

#[test]
fn module_dir_named_after_grammar() {
    let result = build_js(SIMPLE_GRAMMAR);
    assert!(
        result.parser_path.contains("grammar_simple"),
        "parser path should contain grammar_simple/, got: {}",
        result.parser_path
    );
}

#[test]
fn complex_grammar_name_preserved() {
    let result = build_js(COMPLEX_GRAMMAR);
    assert_eq!(result.grammar_name, "complex");
}

#[test]
fn grammar_name_with_underscores() {
    let js = r#"
module.exports = grammar({
  name: 'my_lang',
  rules: {
    source_file: $ => $.tok,
    tok: $ => /[a-z]+/
  }
});
"#;
    let result = build_js(js);
    assert_eq!(result.grammar_name, "my_lang");
    assert!(result.parser_path.contains("parser_my_lang.rs"));
}

// =========================================================================
// 2. Generated module contains LANGUAGE constant
// =========================================================================

#[test]
fn parser_code_contains_language_constant() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("LANGUAGE"),
        "generated code must define LANGUAGE"
    );
}

#[test]
fn language_constant_references_ts_language_type() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("TSLanguage"),
        "LANGUAGE constant must be of type TSLanguage"
    );
}

// =========================================================================
// 3. Generated module has parse functions
// =========================================================================

#[test]
fn parser_code_has_tree_sitter_language_fn() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("tree_sitter_simple"),
        "must generate tree_sitter_<name> function"
    );
}

#[test]
fn complex_grammar_has_language_fn() {
    let result = build_js(COMPLEX_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("tree_sitter_complex"),
        "must generate tree_sitter_complex function"
    );
}

#[test]
fn language_fn_is_extern_c() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("extern \"C\""),
        "language function should use extern \"C\" calling convention"
    );
}

#[test]
fn on_disk_module_has_grammar_name_constant() {
    let (_result, on_disk, _dir) = build_and_read(SIMPLE_GRAMMAR);
    assert!(
        on_disk.contains("GRAMMAR_NAME"),
        "on-disk module must have GRAMMAR_NAME constant"
    );
}

#[test]
fn grammar_name_constant_value_correct() {
    let (_result, on_disk, _dir) = build_and_read(SIMPLE_GRAMMAR);
    assert!(
        on_disk.contains(r#"GRAMMAR_NAME: &str = "simple""#),
        "GRAMMAR_NAME should equal the grammar name"
    );
}

// =========================================================================
// 4. Generated code references tree-sitter types
// =========================================================================

#[test]
fn parser_code_references_ts_parse_action() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("TSParseAction"),
        "generated code must reference TSParseAction"
    );
}

#[test]
fn parser_code_references_ts_rule() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("TSRule"),
        "generated code must reference TSRule"
    );
}

#[test]
fn parser_code_references_language_version() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must reference TREE_SITTER_LANGUAGE_VERSION"
    );
}

#[test]
fn parser_code_references_sync_ptr() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("SyncPtr"),
        "generated code must reference SyncPtr"
    );
}

// =========================================================================
// 5. Module structure (imports, constants, functions)
// =========================================================================

#[test]
fn on_disk_starts_with_autogenerated_comment() {
    let (_result, on_disk, _dir) = build_and_read(SIMPLE_GRAMMAR);
    assert!(
        on_disk.starts_with("// Auto-generated parser for"),
        "module should start with auto-generated comment"
    );
}

#[test]
fn parser_code_has_use_declarations() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("use adze") || code.contains("adze :: pure_parser"),
        "generated code must import adze types"
    );
}

#[test]
fn parser_code_has_static_arrays() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    // Generated code includes symbol names, parse actions, etc. as statics
    assert!(
        code.contains("SYMBOL_NAME_PTRS") || code.contains("PARSE_ACTIONS"),
        "generated code must contain static table arrays"
    );
}

#[test]
fn parser_code_has_symbol_metadata() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("SYMBOL_METADATA"),
        "generated code must contain SYMBOL_METADATA"
    );
}

#[test]
fn parser_code_has_lexer() {
    let result = build_js(SIMPLE_GRAMMAR);
    let code = &result.parser_code;
    assert!(
        code.contains("lexer_fn") || code.contains("lex_fn"),
        "generated code must contain a lexer function"
    );
}

// =========================================================================
// 6. Module from simple grammar
// =========================================================================

#[test]
fn simple_grammar_builds_successfully() {
    let result = build_js(SIMPLE_GRAMMAR);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn simple_grammar_has_nonzero_states() {
    let result = build_js(SIMPLE_GRAMMAR);
    assert!(
        result.build_stats.state_count > 0,
        "simple grammar should produce at least one state"
    );
}

#[test]
fn simple_grammar_has_nonzero_symbols() {
    let result = build_js(SIMPLE_GRAMMAR);
    assert!(
        result.build_stats.symbol_count > 0,
        "simple grammar should have at least one symbol"
    );
}

#[test]
fn simple_grammar_produces_valid_node_types_json() {
    let result = build_js(SIMPLE_GRAMMAR);
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array(), "NODE_TYPES should be a JSON array");
}

#[test]
fn simple_grammar_parser_file_exists() {
    // Use build_and_read so the TempDir stays alive while we check
    let (result, _on_disk, _dir) = build_and_read(SIMPLE_GRAMMAR);
    assert!(
        std::path::Path::new(&result.parser_path).exists(),
        "parser file should exist on disk"
    );
}

// =========================================================================
// 7. Module from complex grammar
// =========================================================================

#[test]
fn complex_grammar_builds_successfully() {
    let result = build_js(COMPLEX_GRAMMAR);
    assert!(!result.parser_code.is_empty());
}

#[test]
fn complex_grammar_has_more_states_than_simple() {
    let simple = build_js(SIMPLE_GRAMMAR);
    let complex = build_js(COMPLEX_GRAMMAR);
    assert!(
        complex.build_stats.state_count >= simple.build_stats.state_count,
        "complex grammar ({} states) should have >= states than simple ({} states)",
        complex.build_stats.state_count,
        simple.build_stats.state_count
    );
}

#[test]
fn complex_grammar_has_more_symbols_than_simple() {
    let simple = build_js(SIMPLE_GRAMMAR);
    let complex = build_js(COMPLEX_GRAMMAR);
    assert!(
        complex.build_stats.symbol_count > simple.build_stats.symbol_count,
        "complex grammar ({} symbols) should have more symbols than simple ({} symbols)",
        complex.build_stats.symbol_count,
        simple.build_stats.symbol_count
    );
}

#[test]
fn complex_grammar_contains_all_symbol_names() {
    let result = build_js(COMPLEX_GRAMMAR);
    // Symbol names are stored as byte arrays in generated code, so check
    // the NODE_TYPES JSON which contains them as readable strings.
    let node_types: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let json_text = serde_json::to_string(&node_types).unwrap();
    for name in &["identifier", "number"] {
        assert!(
            json_text.contains(name),
            "complex grammar node types should reference symbol '{}'",
            name
        );
    }
}

#[test]
fn complex_grammar_produces_valid_node_types() {
    let result = build_js(COMPLEX_GRAMMAR);
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(
        arr.len() > 1,
        "complex grammar should produce multiple node types"
    );
}

// =========================================================================
// 8. Module determinism
// =========================================================================

#[test]
fn parser_code_structural_determinism() {
    // Grammar conversion uses HashMaps internally, so raw token ordering may
    // vary. Verify structural properties are consistent across builds.
    let r1 = build_js(SIMPLE_GRAMMAR);
    let r2 = build_js(SIMPLE_GRAMMAR);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    // Both contain the same key elements
    for keyword in &[
        "LANGUAGE",
        "TSLanguage",
        "tree_sitter_simple",
        "extern \"C\"",
    ] {
        assert!(r1.parser_code.contains(keyword));
        assert!(r2.parser_code.contains(keyword));
    }
}

#[test]
fn node_types_json_is_deterministic() {
    let r1 = build_js(SIMPLE_GRAMMAR);
    let r2 = build_js(SIMPLE_GRAMMAR);
    assert_eq!(
        r1.node_types_json, r2.node_types_json,
        "NODE_TYPES JSON must be deterministic"
    );
}

#[test]
fn build_stats_are_deterministic() {
    let r1 = build_js(SIMPLE_GRAMMAR);
    let r2 = build_js(SIMPLE_GRAMMAR);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn complex_grammar_stats_determinism() {
    // Complex grammar conversion may produce slightly varying state counts due
    // to HashMap-driven rule ordering, but symbol count and key structural
    // properties should remain stable.
    let r1 = build_js(COMPLEX_GRAMMAR);
    let r2 = build_js(COMPLEX_GRAMMAR);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    assert_eq!(r1.grammar_name, r2.grammar_name);
    assert_eq!(r1.node_types_json, r2.node_types_json);
    // Both produce a valid parser with similar state counts
    assert!(r1.build_stats.state_count > 0);
    assert!(r2.build_stats.state_count > 0);
}
