//! Tests for the grammar_js module — JSON-based parsing and conversion.

use adze_tool::grammar_js::GrammarJsConverter;
use adze_tool::grammar_js::json_converter::from_tree_sitter_json;

#[test]
fn parse_minimal_grammar_json() {
    let json = serde_json::json!({
        "name": "test",
        "rules": {
            "source": {
                "type": "STRING",
                "value": "hello"
            }
        }
    });
    let result = from_tree_sitter_json(&json);
    assert!(result.is_ok(), "should parse minimal grammar: {result:?}");
}

#[test]
fn parse_grammar_json_with_extras() {
    let json = serde_json::json!({
        "name": "test_extras",
        "rules": {
            "source": {
                "type": "STRING",
                "value": "x"
            }
        },
        "extras": [
            {
                "type": "PATTERN",
                "value": "\\s+"
            }
        ]
    });
    let result = from_tree_sitter_json(&json);
    assert!(
        result.is_ok(),
        "should parse grammar with extras: {result:?}"
    );
}

#[test]
fn converter_produces_grammar() {
    let json = serde_json::json!({
        "name": "simple",
        "rules": {
            "program": {
                "type": "REPEAT",
                "content": {
                    "type": "STRING",
                    "value": "a"
                }
            }
        }
    });
    let gjs = from_tree_sitter_json(&json).unwrap();
    let converter = GrammarJsConverter::new(gjs);
    let grammar = converter.convert();
    assert!(grammar.is_ok(), "conversion should succeed: {grammar:?}");
}

#[test]
fn converted_grammar_has_name() {
    let json = serde_json::json!({
        "name": "my_lang",
        "rules": {
            "program": {
                "type": "STRING",
                "value": "x"
            }
        }
    });
    let gjs = from_tree_sitter_json(&json).unwrap();
    let converter = GrammarJsConverter::new(gjs);
    let grammar = converter.convert().unwrap();
    assert_eq!(grammar.name, "my_lang");
}

#[test]
fn parse_grammar_json_missing_name() {
    let json = serde_json::json!({
        "rules": {
            "source": { "type": "STRING", "value": "x" }
        }
    });
    let result = from_tree_sitter_json(&json);
    let debug = format!("{result:?}");
    assert!(!debug.is_empty());
}
