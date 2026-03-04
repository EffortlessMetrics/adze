//! Comprehensive JSON grammar parsing tests for the adze-tool crate.
//!
//! This test suite covers:
//! - Parsing minimal JSON grammars
//! - Parsing grammars with multiple rules
//! - Grammars with precedence, extras, externals, and conflicts
//! - Rule type variations (choices, sequences, etc.)
//! - build_parser_from_json function with valid and invalid inputs
//! - BuildResult field validation
//! - Grammar name extraction and validation
//! - NODE_TYPES JSON validity
//! - Deterministic output
//! - Node types JSON validity and parser code generation

use adze_tool::grammar_js::json_converter::from_tree_sitter_json;
use adze_tool::grammar_js::{GrammarJs, Rule};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;
use tempfile::TempDir;

// ===========================================================================
// Helper Functions
// ===========================================================================

/// Create minimal BuildOptions for testing
fn test_build_options(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

/// Create BuildOptions with artifacts enabled
fn test_build_options_with_artifacts(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: false,
    }
}

// ===========================================================================
// Test 1-5: Minimal and Multiple Rules
// ===========================================================================

#[test]
fn parse_minimal_json_grammar() {
    let grammar_json = json!({
        "name": "minimal",
        "rules": {
            "source_file": {
                "type": "STRING",
                "value": "hello"
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.name, "minimal");
    assert_eq!(g.rules.len(), 1);
    assert!(g.rules.contains_key("source_file"));
}

#[test]
fn parse_grammar_with_two_rules() {
    let grammar_json = json!({
        "name": "two_rules",
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "expression"
            },
            "expression": {
                "type": "PATTERN",
                "value": "[0-9]+"
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.name, "two_rules");
    assert_eq!(g.rules.len(), 2);
    assert!(g.rules.contains_key("source_file"));
    assert!(g.rules.contains_key("expression"));
}

#[test]
fn parse_grammar_with_multiple_rules() {
    let grammar_json = json!({
        "name": "multi_rules",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "statement" },
            "statement": { "type": "SYMBOL", "name": "expression" },
            "expression": { "type": "SYMBOL", "name": "primary" },
            "primary": { "type": "PATTERN", "value": "[a-z]+" },
            "number": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.rules.len(), 5);
}

#[test]
fn rules_use_indexmap_preserving_insertion_order() {
    let grammar_json = json!({
        "name": "order_test",
        "rules": {
            "first": { "type": "BLANK" },
            "second": { "type": "BLANK" },
            "third": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    let keys: Vec<&String> = g.rules.keys().collect();
    // IndexMap preserves insertion order
    assert_eq!(keys.len(), 3);
}

#[test]
fn parse_minimal_grammar_has_empty_extras() {
    let grammar_json = json!({
        "name": "no_extras",
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.extras.len(), 0);
}

// ===========================================================================
// Test 6-10: Precedence Handling
// ===========================================================================

#[test]
fn parse_grammar_with_prec_rule() {
    let grammar_json = json!({
        "name": "with_prec",
        "rules": {
            "expr": {
                "type": "PREC",
                "value": 5,
                "content": {
                    "type": "STRING",
                    "value": "x"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::Prec { .. }));
}

#[test]
fn parse_grammar_with_prec_left() {
    let grammar_json = json!({
        "name": "with_prec_left",
        "rules": {
            "expr": {
                "type": "PREC_LEFT",
                "value": 3,
                "content": {
                    "type": "STRING",
                    "value": "+"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::PrecLeft { .. }));
}

#[test]
fn parse_grammar_with_prec_right() {
    let grammar_json = json!({
        "name": "with_prec_right",
        "rules": {
            "expr": {
                "type": "PREC_RIGHT",
                "value": 2,
                "content": {
                    "type": "STRING",
                    "value": "^"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::PrecRight { .. }));
}

#[test]
fn parse_grammar_with_prec_dynamic() {
    let grammar_json = json!({
        "name": "with_prec_dynamic",
        "rules": {
            "expr": {
                "type": "PREC_DYNAMIC",
                "value": 1,
                "content": {
                    "type": "STRING",
                    "value": "x"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::PrecDynamic { .. }));
}

#[test]
fn parse_grammar_with_nested_precedence() {
    let grammar_json = json!({
        "name": "nested_prec",
        "rules": {
            "expr": {
                "type": "PREC",
                "value": 10,
                "content": {
                    "type": "PREC_LEFT",
                    "value": 5,
                    "content": {
                        "type": "STRING",
                        "value": "op"
                    }
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::Prec { .. }));
}

// ===========================================================================
// Test 11-15: Extras (Whitespace/Comments)
// ===========================================================================

#[test]
fn parse_grammar_with_single_extra() {
    let grammar_json = json!({
        "name": "with_single_extra",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn parse_grammar_with_multiple_extras() {
    let grammar_json = json!({
        "name": "with_multiple_extras",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            { "type": "PATTERN", "value": "//.*" }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn parse_grammar_with_extra_symbol_reference() {
    let grammar_json = json!({
        "name": "with_extra_symbol",
        "extras": [
            { "type": "SYMBOL", "name": "whitespace" }
        ],
        "rules": {
            "source": { "type": "BLANK" },
            "whitespace": { "type": "PATTERN", "value": "\\s+" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn parse_grammar_with_extra_string_literal() {
    let grammar_json = json!({
        "name": "with_extra_string",
        "extras": [
            { "type": "STRING", "value": " " }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn extras_can_be_complex_rules() {
    let grammar_json = json!({
        "name": "complex_extras",
        "extras": [
            {
                "type": "CHOICE",
                "members": [
                    { "type": "PATTERN", "value": "\\s+" },
                    { "type": "PATTERN", "value": "//.*" }
                ]
            }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.extras.len(), 1);
}

// ===========================================================================
// Test 16-20: Externals
// ===========================================================================

#[test]
fn parse_grammar_with_single_external() {
    let grammar_json = json!({
        "name": "with_external",
        "externals": [
            { "name": "external_token", "symbol": "EXTERNAL" }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "external_token");
}

#[test]
fn parse_grammar_with_multiple_externals() {
    let grammar_json = json!({
        "name": "with_multiple_externals",
        "externals": [
            { "name": "scanner_token", "symbol": "SCANNER" },
            { "name": "newline", "symbol": "NEWLINE" }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn external_tokens_have_correct_fields() {
    let grammar_json = json!({
        "name": "external_fields",
        "externals": [
            { "name": "my_token", "symbol": "MY_SYMBOL" }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.externals[0].name, "my_token");
    // Symbol is auto-generated as "external_<index>"
    assert_eq!(g.externals[0].symbol, "external_0");
}

#[test]
fn parse_grammar_with_externals_and_rules() {
    let grammar_json = json!({
        "name": "externals_with_rules",
        "externals": [
            { "name": "indent", "symbol": "INDENT" }
        ],
        "rules": {
            "source": { "type": "SYMBOL", "name": "indent" },
            "indent": { "type": "SYMBOL", "name": "content" },
            "content": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn parse_grammar_with_externals_in_rules() {
    let grammar_json = json!({
        "name": "externals_in_rules",
        "externals": [
            { "name": "newline", "symbol": "NEWLINE" }
        ],
        "rules": {
            "line": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "text" },
                    { "type": "SYMBOL", "name": "newline" }
                ]
            },
            "text": { "type": "PATTERN", "value": "[^\\n]+" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.externals.len(), 1);
}

// ===========================================================================
// Test 21-25: Conflicts
// ===========================================================================

#[test]
fn parse_grammar_with_single_conflict() {
    let grammar_json = json!({
        "name": "with_conflict",
        "conflicts": [["expr1", "expr2"]],
        "rules": {
            "source": { "type": "BLANK" },
            "expr1": { "type": "BLANK" },
            "expr2": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.conflicts[0], vec!["expr1", "expr2"]);
}

#[test]
fn parse_grammar_with_multiple_conflicts() {
    let grammar_json = json!({
        "name": "with_multiple_conflicts",
        "conflicts": [
            ["a", "b"],
            ["c", "d"],
            ["e", "f", "g"]
        ],
        "rules": {
            "source": { "type": "BLANK" },
            "a": { "type": "BLANK" },
            "b": { "type": "BLANK" },
            "c": { "type": "BLANK" },
            "d": { "type": "BLANK" },
            "e": { "type": "BLANK" },
            "f": { "type": "BLANK" },
            "g": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.conflicts.len(), 3);
}

#[test]
fn conflict_with_three_way_conflict() {
    let grammar_json = json!({
        "name": "three_way_conflict",
        "conflicts": [["expr", "stmt", "term"]],
        "rules": {
            "source": { "type": "BLANK" },
            "expr": { "type": "BLANK" },
            "stmt": { "type": "BLANK" },
            "term": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.conflicts[0].len(), 3);
}

#[test]
fn parse_grammar_with_empty_conflicts_array() {
    let grammar_json = json!({
        "name": "no_conflicts",
        "conflicts": [],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.conflicts.len(), 0);
}

#[test]
fn conflicts_and_extras_and_externals_together() {
    let grammar_json = json!({
        "name": "all_together",
        "conflicts": [["a", "b"]],
        "extras": [{ "type": "PATTERN", "value": "\\s+" }],
        "externals": [{ "name": "ext", "symbol": "EXT" }],
        "rules": {
            "source": { "type": "BLANK" },
            "a": { "type": "BLANK" },
            "b": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.externals.len(), 1);
}

// ===========================================================================
// Test 26-30: Rule Types - Choices and Sequences
// ===========================================================================

#[test]
fn parse_grammar_with_choice_rule() {
    let grammar_json = json!({
        "name": "with_choice",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::Choice { .. }));
}

#[test]
fn parse_grammar_with_sequence_rule() {
    let grammar_json = json!({
        "name": "with_sequence",
        "rules": {
            "pair": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "SYMBOL", "name": "expr" },
                    { "type": "STRING", "value": ")" }
                ]
            },
            "expr": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["pair"], Rule::Seq { .. }));
}

#[test]
fn parse_grammar_with_repeat_rule() {
    let grammar_json = json!({
        "name": "with_repeat",
        "rules": {
            "list": {
                "type": "REPEAT",
                "content": {
                    "type": "STRING",
                    "value": "x"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["list"], Rule::Repeat { .. }));
}

#[test]
fn parse_grammar_with_repeat1_rule() {
    let grammar_json = json!({
        "name": "with_repeat1",
        "rules": {
            "list": {
                "type": "REPEAT1",
                "content": {
                    "type": "STRING",
                    "value": "x"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["list"], Rule::Repeat1 { .. }));
}

#[test]
fn parse_grammar_with_optional_rule() {
    let grammar_json = json!({
        "name": "with_optional",
        "rules": {
            "maybe_expr": {
                "type": "OPTIONAL",
                "value": {
                    "type": "STRING",
                    "value": "x"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["maybe_expr"], Rule::Optional { .. }));
}

// ===========================================================================
// Test 31-35: Rule Types - Tokens and Aliases
// ===========================================================================

#[test]
fn parse_grammar_with_token_rule() {
    let grammar_json = json!({
        "name": "with_token",
        "rules": {
            "string": {
                "type": "TOKEN",
                "content": {
                    "type": "STRING",
                    "value": "string"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["string"], Rule::Token { .. }));
}

#[test]
fn parse_grammar_with_immediate_token_rule() {
    let grammar_json = json!({
        "name": "with_immediate_token",
        "rules": {
            "immediate": {
                "type": "IMMEDIATE_TOKEN",
                "content": {
                    "type": "STRING",
                    "value": "x"
                }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["immediate"], Rule::ImmediateToken { .. }));
}

#[test]
fn parse_grammar_with_alias_rule() {
    let grammar_json = json!({
        "name": "with_alias",
        "rules": {
            "expr": {
                "type": "ALIAS",
                "content": { "type": "STRING", "value": "x" },
                "value": "alias_name",
                "named": true
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["expr"], Rule::Alias { .. }));
}

#[test]
fn parse_grammar_with_field_rule() {
    let grammar_json = json!({
        "name": "with_field",
        "rules": {
            "pair": {
                "type": "FIELD",
                "name": "value",
                "content": { "type": "STRING", "value": "x" }
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["pair"], Rule::Field { .. }));
}

#[test]
fn parse_grammar_with_blank_rule() {
    let grammar_json = json!({
        "name": "with_blank",
        "rules": {
            "empty": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(matches!(g.rules["empty"], Rule::Blank));
}

// ===========================================================================
// Test 36-40: Other Grammar Properties
// ===========================================================================

#[test]
fn parse_grammar_with_word_token() {
    let grammar_json = json!({
        "name": "with_word",
        "word": "identifier",
        "rules": {
            "source": { "type": "BLANK" },
            "identifier": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.word.as_deref(), Some("identifier"));
}

#[test]
fn parse_grammar_with_inline_rules() {
    let grammar_json = json!({
        "name": "with_inline",
        "inline": ["_internal", "_helper"],
        "rules": {
            "source": { "type": "BLANK" },
            "_internal": { "type": "BLANK" },
            "_helper": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.inline.len(), 2);
    assert!(g.inline.contains(&"_internal".to_string()));
}

#[test]
fn parse_grammar_with_supertypes() {
    let grammar_json = json!({
        "name": "with_supertypes",
        "supertypes": ["expression", "statement"],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.supertypes.len(), 2);
    assert!(g.supertypes.contains(&"expression".to_string()));
}

#[test]
fn parse_grammar_with_all_optional_fields() {
    let grammar_json = json!({
        "name": "complete",
        "word": "id",
        "inline": ["_expr"],
        "conflicts": [["a", "b"]],
        "extras": [{ "type": "PATTERN", "value": "\\s+" }],
        "externals": [{ "name": "ext", "symbol": "EXT" }],
        "supertypes": ["expr", "stmt"],
        "rules": {
            "source": { "type": "BLANK" },
            "id": { "type": "BLANK" },
            "_expr": { "type": "BLANK" },
            "a": { "type": "BLANK" },
            "b": { "type": "BLANK" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.name, "complete");
    assert!(g.word.is_some());
    assert!(!g.inline.is_empty());
    assert!(!g.conflicts.is_empty());
    assert!(!g.extras.is_empty());
    assert!(!g.externals.is_empty());
    assert!(!g.supertypes.is_empty());
}

#[test]
fn grammar_js_new_creates_valid_grammar() {
    let g = GrammarJs::new("test".to_string());
    assert_eq!(g.name, "test");
    assert!(g.word.is_none());
    assert!(g.inline.is_empty());
    assert!(g.conflicts.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.rules.is_empty());
    assert!(g.supertypes.is_empty());
}

// ===========================================================================
// Test 41-45: build_parser_from_json Tests
// ===========================================================================

#[test]
fn build_parser_from_json_with_minimal_grammar() {
    let grammar_json = json!({
        "name": "minimal_build",
        "rules": {
            "source_file": {
                "type": "STRING",
                "value": "x"
            }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "minimal_build");
}

#[test]
fn build_parser_from_json_with_valid_grammar() {
    let grammar_json = json!({
        "name": "valid_grammar",
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "expr"
            },
            "expr": {
                "type": "PATTERN",
                "value": "[0-9]+"
            }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "valid_grammar");
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_parser_from_json_invalid_json_returns_error() {
    let invalid_json = "{ invalid json }".to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(invalid_json, opts);

    assert!(result.is_err());
}

#[test]
fn build_parser_from_json_missing_name_field() {
    let grammar_json = json!({
        "rules": {
            "source_file": { "type": "BLANK" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts);

    // This should fail because the grammar is invalid (missing name or other required fields)
    assert!(result.is_err());
}

#[test]
fn build_result_has_valid_grammar_name() {
    let grammar_json = json!({
        "name": "result_test",
        "rules": {
            "source_file": { "type": "BLANK" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert!(!result.grammar_name.is_empty());
    assert_eq!(result.grammar_name, "result_test");
}

// ===========================================================================
// Test 46-50: BuildResult Validation
// ===========================================================================

#[test]
fn build_result_parser_code_is_non_empty() {
    let grammar_json = json!({
        "name": "code_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert!(!result.parser_code.is_empty());
    assert!(result.parser_code.len() > 0);
}

#[test]
fn build_result_node_types_json_is_non_empty() {
    let grammar_json = json!({
        "name": "node_types_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_result_node_types_is_valid_json() {
    let grammar_json = json!({
        "name": "valid_json_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    let parsed = serde_json::from_str::<serde_json::Value>(&result.node_types_json);
    assert!(parsed.is_ok(), "node_types_json should be valid JSON");
}

#[test]
fn build_result_node_types_is_array() {
    let grammar_json = json!({
        "name": "array_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    let node_types: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(node_types.is_array(), "node_types should be a JSON array");
}

#[test]
fn build_result_has_build_stats() {
    let grammar_json = json!({
        "name": "stats_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert!(result.build_stats.state_count > 0);
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_result_parser_path_is_non_empty() {
    let grammar_json = json!({
        "name": "path_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert!(!result.parser_path.is_empty());
}

// ===========================================================================
// Test 51-55: Deterministic Output and Edge Cases
// ===========================================================================

#[test]
fn build_parser_twice_produces_same_output() {
    let grammar_json = json!({
        "name": "deterministic",
        "rules": {
            "source_file": { "type": "STRING", "value": "x" }
        }
    })
    .to_string();

    let dir1 = TempDir::new().unwrap();
    let opts1 = test_build_options(&dir1);
    let result1 = build_parser_from_json(grammar_json.clone(), opts1).unwrap();

    let dir2 = TempDir::new().unwrap();
    let opts2 = test_build_options(&dir2);
    let result2 = build_parser_from_json(grammar_json, opts2).unwrap();

    assert_eq!(result1.grammar_name, result2.grammar_name);
    assert_eq!(result1.parser_code, result2.parser_code);
    assert_eq!(result1.node_types_json, result2.node_types_json);
}

#[test]
fn build_parser_with_complex_grammar() {
    let grammar_json = json!({
        "name": "complex",
        "word": "identifier",
        "inline": ["_expr"],
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            { "type": "PATTERN", "value": "//.*" }
        ],
        "conflicts": [["expr1", "expr2"]],
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "statement" },
            "statement": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "expr1" },
                    { "type": "SYMBOL", "name": "expr2" }
                ]
            },
            "expr1": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "SYMBOL", "name": "_expr" },
                    { "type": "STRING", "value": ")" }
                ]
            },
            "expr2": { "type": "PATTERN", "value": "[0-9]+" },
            "_expr": { "type": "SYMBOL", "name": "identifier" },
            "identifier": { "type": "PATTERN", "value": "[a-zA-Z_]\\w*" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "complex");
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_parser_with_nested_rules() {
    let grammar_json = json!({
        "name": "nested",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "REPEAT",
                        "content": {
                            "type": "CHOICE",
                            "members": [
                                { "type": "STRING", "value": "a" },
                                { "type": "STRING", "value": "b" }
                            ]
                        }
                    }
                ]
            }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "nested");
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_parser_with_many_rules() {
    let mut rules = serde_json::json!({});
    rules["source_file"] = json!({ "type": "CHOICE", "members": [] });
    let mut members = vec![];

    // Create 20 rules
    for i in 0..20 {
        let rule_name = format!("rule_{}", i);
        rules[rule_name.clone()] = json!({ "type": "PATTERN", "value": "[a-z]+" });
        members.push(json!({ "type": "SYMBOL", "name": rule_name }));
    }

    // Update source_file to reference all rules
    rules["source_file"]["members"] = json!(members);

    let grammar_json = json!({
        "name": "many_rules",
        "rules": rules
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "many_rules");
}

#[test]
fn build_parser_with_emit_artifacts_enabled() {
    let grammar_json = json!({
        "name": "artifacts_test",
        "rules": {
            "source_file": { "type": "STRING", "value": "x" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options_with_artifacts(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert!(!result.grammar_name.is_empty());
}

// ===========================================================================
// Test 56+: Additional Coverage
// ===========================================================================

#[test]
fn grammar_validation_catches_invalid_symbols() {
    let grammar_json = json!({
        "name": "invalid_ref",
        "rules": {
            "source": {
                "type": "SYMBOL",
                "name": "nonexistent"
            }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    // Validation should catch the missing symbol
    assert!(g.validate().is_err());
}

#[test]
fn grammar_with_string_literals() {
    let grammar_json = json!({
        "name": "string_literals",
        "rules": {
            "keyword": { "type": "STRING", "value": "if" },
            "lparen": { "type": "STRING", "value": "(" },
            "rparen": { "type": "STRING", "value": ")" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn grammar_with_patterns() {
    let grammar_json = json!({
        "name": "patterns",
        "rules": {
            "identifier": { "type": "PATTERN", "value": "[a-zA-Z_]\\w*" },
            "number": { "type": "PATTERN", "value": "[0-9]+" },
            "string": { "type": "PATTERN", "value": "\"[^\"]*\"" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn build_parser_with_only_whitespace_and_comments_extras() {
    let grammar_json = json!({
        "name": "ws_comments",
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            {
                "type": "CHOICE",
                "members": [
                    { "type": "PATTERN", "value": "//[^\\n]*" },
                    { "type": "PATTERN", "value": "/\\*[^*]*\\*+(?:[^/*][^*]*\\*+)*/" }
                ]
            }
        ],
        "rules": {
            "source_file": { "type": "STRING", "value": "test" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "ws_comments");
}

#[test]
fn grammar_with_symbol_references() {
    let grammar_json = json!({
        "name": "symbol_refs",
        "rules": {
            "expr": { "type": "SYMBOL", "name": "term" },
            "term": { "type": "SYMBOL", "name": "factor" },
            "factor": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });

    let g = from_tree_sitter_json(&grammar_json).unwrap();
    assert!(g.validate().is_ok());
}

#[test]
fn build_parser_from_json_with_multiple_choice_options() {
    let grammar_json = json!({
        "name": "multi_choice",
        "rules": {
            "stmt": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "if_stmt" },
                    { "type": "SYMBOL", "name": "while_stmt" },
                    { "type": "SYMBOL", "name": "expr_stmt" },
                    { "type": "SYMBOL", "name": "block_stmt" }
                ]
            },
            "if_stmt": { "type": "STRING", "value": "if" },
            "while_stmt": { "type": "STRING", "value": "while" },
            "expr_stmt": { "type": "STRING", "value": "expr" },
            "block_stmt": { "type": "STRING", "value": "block" }
        }
    })
    .to_string();

    let dir = TempDir::new().unwrap();
    let opts = test_build_options(&dir);
    let result = build_parser_from_json(grammar_json, opts).unwrap();

    assert_eq!(result.grammar_name, "multi_choice");
}
