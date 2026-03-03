//! Snapshot tests for NodeTypesGenerator output.

use adze_ir::builder::GrammarBuilder;
use adze_tablegen::node_types::NodeTypesGenerator;

fn gen_node_types(name: &str) -> String {
    let grammar = match name {
        "minimal" => GrammarBuilder::new("minimal")
            .token("x", "x")
            .rule("start", vec!["x"])
            .build(),
        "two_rules" => GrammarBuilder::new("two_rules")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .build(),
        "with_extras" => GrammarBuilder::new("with_extras")
            .token("num", r"\d+")
            .token("ws", r"\s+")
            .rule("expr", vec!["num"])
            .extra("ws")
            .build(),
        _ => panic!("Unknown grammar: {name}"),
    };

    let generator = NodeTypesGenerator::new(&grammar);
    match generator.generate() {
        Ok(json) => json,
        Err(e) => format!("ERROR: {e}"),
    }
}

#[test]
fn node_types_minimal() {
    let output = gen_node_types("minimal");
    insta::assert_snapshot!(output);
}

#[test]
fn node_types_two_rules() {
    let output = gen_node_types("two_rules");
    insta::assert_snapshot!(output);
}

#[test]
fn node_types_with_extras() {
    let output = gen_node_types("with_extras");
    insta::assert_snapshot!(output);
}

#[test]
fn node_types_is_valid_json() {
    let output = gen_node_types("minimal");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
    assert!(parsed.is_array(), "node_types should be a JSON array");
}

#[test]
fn node_types_entries_have_required_fields() {
    let output = gen_node_types("minimal");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
    if let Some(arr) = parsed.as_array() {
        for entry in arr {
            assert!(entry.get("type").is_some(), "each entry needs 'type'");
            assert!(entry.get("named").is_some(), "each entry needs 'named'");
        }
    }
}
