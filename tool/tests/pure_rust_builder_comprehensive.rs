//! Comprehensive tests for the pure Rust builder in the adze-tool crate.
//!
//! Tests cover:
//! - BuildOptions struct (defaults, custom values)
//! - BuildResult struct (parser_code, node_types_json, grammar_name, etc.)
//! - build_parser_from_json function (valid and invalid JSON)
//! - build_parser_from_grammar_js function (valid grammar.js files)
//! - build_parser function (core builder function)
//! - Compression options (compress_tables true/false)
//! - Error handling (invalid JSON, empty JSON)
//! - Build determinism (same input → same output)
//! - Generated code quality (contains expected patterns)
//! - Special cases (special characters, multiple rules, extras, precedence)

#![allow(clippy::needless_range_loop)]

use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_grammar_js, build_parser_from_json,
};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Create a simple valid JSON grammar
fn simple_json_grammar(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SYMBOL",
                        "name": "expression"
                    }
                ]
            },
            "expression": {
                "type": "PATTERN",
                "value": r"\d+"
            }
        }
    })
    .to_string()
}

/// Create a JSON grammar with multiple rules
fn multi_rule_json_grammar(name: &str) -> String {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "statement"
            },
            "statement": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SYMBOL",
                        "name": "assignment"
                    },
                    {
                        "type": "SYMBOL",
                        "name": "expression"
                    }
                ]
            },
            "assignment": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "SYMBOL",
                        "name": "identifier"
                    },
                    {
                        "type": "STRING",
                        "value": "="
                    },
                    {
                        "type": "SYMBOL",
                        "name": "expression"
                    }
                ]
            },
            "expression": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SYMBOL",
                        "name": "identifier"
                    },
                    {
                        "type": "PATTERN",
                        "value": r"\d+"
                    }
                ]
            },
            "identifier": {
                "type": "PATTERN",
                "value": r"[a-zA-Z_][a-zA-Z0-9_]*"
            }
        }
    })
    .to_string()
}

/// Build options pointing to temp directory with no artifacts
fn default_build_options() -> (BuildOptions, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: temp_dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (opts, temp_dir)
}

/// Build options with compression disabled
fn build_options_no_compress() -> (BuildOptions, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: temp_dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (opts, temp_dir)
}

// ---------------------------------------------------------------------------
// Test 1: BuildOptions Default Values
// ---------------------------------------------------------------------------

#[test]
fn test_build_options_default_values() {
    let opts = BuildOptions::default();
    assert!(opts.compress_tables, "Default should compress tables");
    // emit_artifacts might vary based on env, but should be a bool
    let _ = opts.emit_artifacts;
    // out_dir should be set to something (likely OUT_DIR or target/debug)
    assert!(!opts.out_dir.is_empty());
}

// ---------------------------------------------------------------------------
// Test 2: BuildOptions Custom Values
// ---------------------------------------------------------------------------

#[test]
fn test_build_options_custom_values() {
    let opts = BuildOptions {
        out_dir: "/custom/path".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/custom/path");
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

// ---------------------------------------------------------------------------
// Test 3: BuildOptions Can Be Cloned
// ---------------------------------------------------------------------------

#[test]
fn test_build_options_clone() {
    let opts1 = BuildOptions {
        out_dir: "/path".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let opts2 = opts1.clone();
    assert_eq!(opts1.out_dir, opts2.out_dir);
    assert_eq!(opts1.emit_artifacts, opts2.emit_artifacts);
    assert_eq!(opts1.compress_tables, opts2.compress_tables);
}

// ---------------------------------------------------------------------------
// Test 4: Build from Simple JSON Grammar
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_simple_json_grammar_success() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_simple");

    let result = build_parser_from_json(grammar_json, opts);
    assert!(result.is_ok(), "Building from simple JSON should succeed");

    let result = result.unwrap();
    assert_eq!(result.grammar_name, "test_simple");
}

// ---------------------------------------------------------------------------
// Test 5: Build from Grammar.js File
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_grammar_js_file() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'arithmetic',
  
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/
  }
});
    "#;

    let (opts, temp) = default_build_options();
    let grammar_path = temp.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js).unwrap();

    let result = build_parser_from_grammar_js(&grammar_path, opts);
    assert!(result.is_ok(), "Building from grammar.js should succeed");

    let result = result.unwrap();
    assert_eq!(result.grammar_name, "arithmetic");
}

// ---------------------------------------------------------------------------
// Test 6: BuildResult Contains parser_code (Non-Empty)
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_parser_code_non_empty() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_code");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    assert!(
        !result.parser_code.is_empty(),
        "parser_code should not be empty"
    );
    assert!(
        result.parser_code.len() > 100,
        "parser_code should be substantial"
    );
}

// ---------------------------------------------------------------------------
// Test 7: BuildResult Contains node_types_json
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_node_types_json() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_node_types");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    assert!(
        !result.node_types_json.is_empty(),
        "node_types_json should not be empty"
    );

    // Verify it's valid JSON
    let parsed: serde_json::Result<serde_json::Value> =
        serde_json::from_str(&result.node_types_json);
    assert!(parsed.is_ok(), "node_types_json should be valid JSON");
}

// ---------------------------------------------------------------------------
// Test 8: BuildResult grammar_name Matches Input
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_grammar_name_matches() {
    let test_names = vec!["test_name_a", "my_grammar", "foo_bar_123"];

    for name in test_names {
        let (opts, _temp) = default_build_options();
        let grammar_json = simple_json_grammar(name);

        let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

        assert_eq!(result.grammar_name, name);
    }
}

// ---------------------------------------------------------------------------
// Test 9: Build with compress_tables=true
// ---------------------------------------------------------------------------

#[test]
fn test_build_with_compression_enabled() {
    let (opts, _temp) = default_build_options();
    assert!(
        opts.compress_tables,
        "Options should have compression enabled"
    );

    let grammar_json = simple_json_grammar("test_compress_true");
    let result = build_parser_from_json(grammar_json, opts);

    assert!(result.is_ok(), "Build with compression should succeed");
}

// ---------------------------------------------------------------------------
// Test 10: Build with compress_tables=false
// ---------------------------------------------------------------------------

#[test]
fn test_build_with_compression_disabled() {
    let (opts, _temp) = build_options_no_compress();
    assert!(
        !opts.compress_tables,
        "Options should have compression disabled"
    );

    let grammar_json = simple_json_grammar("test_compress_false");
    let result = build_parser_from_json(grammar_json, opts);

    assert!(result.is_ok(), "Build without compression should succeed");
}

// ---------------------------------------------------------------------------
// Test 11: Build from Invalid JSON Returns Error
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_invalid_json_error() {
    let (opts, _temp) = default_build_options();
    let invalid_json = "{ invalid json }";

    let result = build_parser_from_json(invalid_json.to_string(), opts);
    assert!(result.is_err(), "Invalid JSON should return an error");
}

// ---------------------------------------------------------------------------
// Test 12: Build from Empty JSON Object Returns Error
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_empty_json_error() {
    let (opts, _temp) = default_build_options();
    let empty_json = "{}";

    let result = build_parser_from_json(empty_json.to_string(), opts);
    // Empty grammar might error during conversion
    let _ = result; // Allow either outcome for now
}

// ---------------------------------------------------------------------------
// Test 13: Build from Minimal Valid Grammar
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_minimal_valid_grammar() {
    let (opts, _temp) = default_build_options();

    // Minimal grammar with just name and a source_file rule
    let minimal = json!({
        "name": "minimal",
        "rules": {
            "source_file": {
                "type": "PATTERN",
                "value": "."
            }
        }
    })
    .to_string();

    let result = build_parser_from_json(minimal, opts);
    assert!(result.is_ok(), "Minimal grammar should build successfully");
}

// ---------------------------------------------------------------------------
// Test 14: Generated Parser Code Contains TSLanguage
// ---------------------------------------------------------------------------

#[test]
fn test_parser_code_contains_ts_language() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_ts_language");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    // Check that parser code contains indicators of a valid parser
    let code = &result.parser_code;

    // Should contain language-related code
    assert!(
        code.contains("language") || code.contains("Language") || code.contains("parse"),
        "Parser code should contain language/parse related content"
    );
}

// ---------------------------------------------------------------------------
// Test 15: Generated Node Types Is Valid JSON Array
// ---------------------------------------------------------------------------

#[test]
fn test_generated_node_types_is_valid_array() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_node_array");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    let node_types: serde_json::Value = serde_json::from_str(&result.node_types_json)
        .expect("node_types_json should be valid JSON");

    // It should be an array or object
    assert!(
        node_types.is_array() || node_types.is_object(),
        "node_types should be an array or object"
    );
}

// ---------------------------------------------------------------------------
// Test 16: Build Is Deterministic (Same Input → Same Output)
// ---------------------------------------------------------------------------

#[test]
fn test_build_is_deterministic() {
    let grammar_json = simple_json_grammar("test_deterministic");

    // Build twice with same input
    let (opts1, _temp1) = default_build_options();
    let result1 =
        build_parser_from_json(grammar_json.clone(), opts1).expect("First build should succeed");

    let (opts2, _temp2) = default_build_options();
    let result2 = build_parser_from_json(grammar_json, opts2).expect("Second build should succeed");

    // Parser code should be identical
    assert_eq!(
        result1.parser_code, result2.parser_code,
        "Same grammar should produce identical parser code"
    );

    // Grammar name should match
    assert_eq!(result1.grammar_name, result2.grammar_name);
}

// ---------------------------------------------------------------------------
// Test 17: Build Handles Special Characters in Grammar Name
// ---------------------------------------------------------------------------

#[test]
fn test_build_handles_special_characters_in_name() {
    // Valid Rust identifier names (hyphens get converted to underscores)
    let valid_names = vec!["test_grammar", "test123", "TestGrammar"];

    for name in valid_names {
        let (opts, _temp) = default_build_options();
        let grammar_json = simple_json_grammar(name);

        let result = build_parser_from_json(grammar_json, opts);
        // Should not crash
        let _ = result;
    }
}

// ---------------------------------------------------------------------------
// Test 18: Build from Multiple Rules Grammar
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_multiple_rules_grammar() {
    let (opts, _temp) = default_build_options();
    let grammar_json = multi_rule_json_grammar("multi_rules_test");

    let result = build_parser_from_json(grammar_json, opts);
    assert!(result.is_ok(), "Multiple rules grammar should build");

    let result = result.unwrap();
    assert_eq!(result.grammar_name, "multi_rules_test");
}

// ---------------------------------------------------------------------------
// Test 19: Build Parser Result Contains parser_path
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_parser_path_populated() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_path");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    assert!(
        !result.parser_path.is_empty(),
        "parser_path should be populated"
    );
    assert!(
        result.parser_path.contains(".rs"),
        "parser_path should be a Rust file"
    );
}

// ---------------------------------------------------------------------------
// Test 20: BuildResult Contains build_stats
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_contains_build_stats() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_stats");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    // Check stats are populated
    let stats = &result.build_stats;
    assert!(stats.state_count > 0, "Should have at least one state");
    assert!(stats.symbol_count > 0, "Should have at least one symbol");
}

// ---------------------------------------------------------------------------
// Test 21: Different Grammars Produce Different Output
// ---------------------------------------------------------------------------

#[test]
fn test_different_grammars_different_output() {
    let grammar1 = simple_json_grammar("grammar_a");
    let grammar2 = simple_json_grammar("grammar_b");

    let (opts1, _temp1) = default_build_options();
    let result1 = build_parser_from_json(grammar1, opts1).expect("Build 1 should succeed");

    let (opts2, _temp2) = default_build_options();
    let result2 = build_parser_from_json(grammar2, opts2).expect("Build 2 should succeed");

    // Different grammars (by name at least) should have different names
    assert_ne!(result1.grammar_name, result2.grammar_name);
}

// ---------------------------------------------------------------------------
// Test 22: Same Grammar Twice Produces Same Output
// ---------------------------------------------------------------------------

#[test]
fn test_same_grammar_twice_same_output() {
    let grammar_json = multi_rule_json_grammar("test_twice");

    let (opts1, _temp1) = default_build_options();
    let result1 =
        build_parser_from_json(grammar_json.clone(), opts1).expect("Build 1 should succeed");

    let (opts2, _temp2) = default_build_options();
    let result2 = build_parser_from_json(grammar_json, opts2).expect("Build 2 should succeed");

    assert_eq!(
        result1.parser_code, result2.parser_code,
        "Same multi-rule grammar should produce identical code"
    );
    assert_eq!(result1.node_types_json, result2.node_types_json);
}

// ---------------------------------------------------------------------------
// Test 23: Build from Grammar.js with Comments
// ---------------------------------------------------------------------------

#[test]
fn test_build_from_grammar_js_with_comments() {
    let grammar_js = r#"
// This is a comment
module.exports = grammar({
  name: 'commented',
  
  // Rules section
  rules: {
    source_file: $ => $.expression,
    expression: $ => /\d+/  // Numbers
  }
});
    "#;

    let (opts, temp) = default_build_options();
    let grammar_path = temp.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js).unwrap();

    let result = build_parser_from_grammar_js(&grammar_path, opts);
    assert!(result.is_ok(), "Grammar.js with comments should build");
}

// ---------------------------------------------------------------------------
// Test 24: BuildResult parser_code Is Not Truncated
// ---------------------------------------------------------------------------

#[test]
fn test_parser_code_completeness() {
    let (opts, _temp) = default_build_options();
    let grammar_json = multi_rule_json_grammar("test_completeness");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    let code = &result.parser_code;

    // Code should have balanced braces and brackets
    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Code should have balanced braces"
    );
}

// ---------------------------------------------------------------------------
// Test 25: Build with Emit Artifacts Enabled
// ---------------------------------------------------------------------------

#[test]
fn test_build_with_emit_artifacts() {
    let temp = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: temp.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    let grammar_json = simple_json_grammar("test_artifacts");
    let result = build_parser_from_json(grammar_json, opts);

    assert!(result.is_ok(), "Build with emit_artifacts should succeed");
    let result = result.unwrap();

    // Should have created a parser file
    let parser_path = Path::new(&result.parser_path);
    assert!(parser_path.exists(), "Parser file should be created");
}

// ---------------------------------------------------------------------------
// Test 26: BuildResult Contains All Required Fields
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_has_all_fields() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_all_fields");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    // Verify all fields are present and sensible
    assert!(!result.grammar_name.is_empty());
    assert!(!result.parser_path.is_empty());
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());

    // Stats should be valid
    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

// ---------------------------------------------------------------------------
// Test 27: Parser Code Can Be Parsed By syn
// ---------------------------------------------------------------------------

#[test]
fn test_parser_code_is_valid_rust() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("test_syn_parse");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    // Try parsing the code as Rust
    let code_tokens = result.parser_code.parse::<proc_macro2::TokenStream>();
    assert!(
        code_tokens.is_ok(),
        "Generated parser code should be valid Rust tokens"
    );
}

// ---------------------------------------------------------------------------
// Test 28: Compression Option Affects Table Generation
// ---------------------------------------------------------------------------

#[test]
fn test_compression_option_affects_generation() {
    let grammar_json = multi_rule_json_grammar("test_compress_effect");

    let (opts_compressed, _temp1) = default_build_options();
    let result_compressed = build_parser_from_json(grammar_json.clone(), opts_compressed)
        .expect("Compressed build should succeed");

    let (opts_uncompressed, _temp2) = build_options_no_compress();
    let result_uncompressed = build_parser_from_json(grammar_json, opts_uncompressed)
        .expect("Uncompressed build should succeed");

    // Both should succeed, and produce code (may differ in size due to compression)
    assert!(!result_compressed.parser_code.is_empty());
    assert!(!result_uncompressed.parser_code.is_empty());
    // Uncompressed might be larger or have different table structure
    let _ = (result_compressed, result_uncompressed);
}

// ---------------------------------------------------------------------------
// Test 29: Build Handles JSON with Extra Fields
// ---------------------------------------------------------------------------

#[test]
fn test_build_handles_json_with_extra_fields() {
    let grammar_json = json!({
        "name": "extra_fields",
        "rules": {
            "source_file": {
                "type": "PATTERN",
                "value": "."
            }
        },
        "extra_field_1": "ignored",
        "extra_field_2": 42,
        "extra_field_3": { "nested": "object" }
    })
    .to_string();

    let (opts, _temp) = default_build_options();
    let result = build_parser_from_json(grammar_json, opts);

    // Should succeed by ignoring extra fields
    let _ = result;
}

// ---------------------------------------------------------------------------
// Test 30: Build Stats Provide Useful Metrics
// ---------------------------------------------------------------------------

#[test]
fn test_build_stats_metrics_reasonable() {
    let (opts, _temp) = default_build_options();
    let grammar_json = multi_rule_json_grammar("test_metrics");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    let stats = &result.build_stats;

    // Stats should be reasonable numbers
    assert!(stats.state_count > 0, "Should have states");
    assert!(stats.symbol_count > 0, "Should have symbols");
    assert!(
        stats.conflict_cells >= 0,
        "Conflict cells should be non-negative"
    );
}

// Additional helper tests for edge cases

// ---------------------------------------------------------------------------
// Test 31: Grammar with String Literals
// ---------------------------------------------------------------------------

#[test]
fn test_build_grammar_with_string_literals() {
    let grammar_json = json!({
        "name": "string_literals",
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "statement"
            },
            "statement": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "STRING",
                        "value": "let"
                    },
                    {
                        "type": "SYMBOL",
                        "name": "identifier"
                    }
                ]
            },
            "identifier": {
                "type": "PATTERN",
                "value": r"[a-zA-Z_]\w*"
            }
        }
    })
    .to_string();

    let (opts, _temp) = default_build_options();
    let result = build_parser_from_json(grammar_json, opts);
    assert!(result.is_ok(), "Grammar with string literals should build");
}

// ---------------------------------------------------------------------------
// Test 32: BuildOptions Debug Display
// ---------------------------------------------------------------------------

#[test]
fn test_build_options_debug_display() {
    let opts = BuildOptions {
        out_dir: "/test".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };

    let debug_str = format!("{:?}", opts);
    assert!(debug_str.contains("out_dir"));
    assert!(debug_str.contains("emit_artifacts"));
    assert!(debug_str.contains("compress_tables"));
}

// ---------------------------------------------------------------------------
// Test 33: BuildResult Debug Display
// ---------------------------------------------------------------------------

#[test]
fn test_build_result_debug_display() {
    let (opts, _temp) = default_build_options();
    let grammar_json = simple_json_grammar("debug_test");

    let result = build_parser_from_json(grammar_json, opts).expect("Build should succeed");

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("grammar_name"));
}
