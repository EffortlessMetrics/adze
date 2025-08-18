#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
//! Integration tests for exact cell verification between Tree-sitter and Rust-sitter.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
#[ignore] // Since we need actual ts-bridge built
fn verify_exact_cell_after_lbrace() {
    // First, ensure we have the extracted JSON tables
    let tables_path = "../../tools/ts-bridge/test-data/json-tables.json";

    if !Path::new(tables_path).exists() {
        eprintln!("Extracting JSON grammar tables...");
        // Build ts-bridge if needed
        let output = Command::new("cargo")
            .args(&["build", "-p", "ts-bridge", "--features", "with-grammars"])
            .output()
            .expect("Failed to build ts-bridge");

        if !output.status.success() {
            panic!(
                "Failed to build ts-bridge: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Extract tables
        let output = Command::new("cargo")
            .args(&[
                "run",
                "-p",
                "ts-bridge",
                "--features",
                "with-grammars",
                "--",
                "--grammar",
                "json",
                "--output",
                tables_path,
            ])
            .output()
            .expect("Failed to run ts-bridge");

        if !output.status.success() {
            panic!(
                "Failed to extract tables: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // Load the extracted tables
    let json_content = fs::read_to_string(tables_path).expect("Failed to read extracted tables");
    let tables: serde_json::Value =
        serde_json::from_str(&json_content).expect("Failed to parse JSON tables");

    // Get symbol mappings
    let symbols = tables["symbols"]
        .as_array()
        .expect("No symbols array in extracted tables");

    let lbrace_idx = symbols
        .iter()
        .position(|s| s.as_str() == Some("{"))
        .expect("No '{' symbol found") as u16;

    let rbrace_idx = symbols
        .iter()
        .position(|s| s.as_str() == Some("}"))
        .expect("No '}' symbol found") as u16;

    eprintln!(
        "Symbol indices: '{{' = {}, '}}' = {}",
        lbrace_idx, rbrace_idx
    );

    // Check initial state (should be 1 for Tree-sitter)
    let parse_table = &tables["parse_table"];

    // Get actions for state 1, symbol '{'
    let state_1_actions = parse_table["1"]
        .as_object()
        .expect("No state 1 in parse table");

    let lbrace_actions = state_1_actions.get(&lbrace_idx.to_string());

    if let Some(actions) = lbrace_actions {
        eprintln!("Actions for (state=1, symbol='{{'): {:?}", actions);

        // Find the shift action
        let shift_state = actions
            .as_array()
            .and_then(|arr| {
                arr.iter().find_map(|a| {
                    if a["type"].as_str() == Some("S") {
                        a["state"].as_u64()
                    } else {
                        None
                    }
                })
            })
            .expect("No shift action found for '{'");

        eprintln!("Shift on '{{' goes to state {}", shift_state);

        // Now check that state for '}'
        let next_state_actions = parse_table[&shift_state.to_string()]
            .as_object()
            .expect(&format!("No state {} in parse table", shift_state));

        let rbrace_actions = next_state_actions.get(&rbrace_idx.to_string());

        eprintln!(
            "Actions for (state={}, symbol='}}'): {:?}",
            shift_state, rbrace_actions
        );

        // Verify we have non-error actions
        if let Some(actions) = rbrace_actions {
            let has_valid_action = actions
                .as_array()
                .map(|arr| {
                    arr.iter().any(|a| {
                        let action_type = a["type"].as_str().unwrap_or("");
                        matches!(action_type, "S" | "R" | "A")
                    })
                })
                .unwrap_or(false);

            assert!(has_valid_action,
                "Expected Shift/Reduce/Accept action for '}}' after '{{', but found only error recovery");
        } else {
            panic!("No actions found for '}}' after '{{'");
        }
    } else {
        panic!("No actions found for '{{' in initial state");
    }
}

#[test]
fn verify_runtime_behavior_matches_extraction() {
    use tree_sitter_json as ts_json;

    // First verify runtime behavior
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&ts_json::LANGUAGE.into()).unwrap();

    let test_cases = [
        ("{}", false, "empty object"),
        ("[]", false, "empty array"),
        (r#"{"key": "value"}"#, false, "simple object"),
        ("{", true, "incomplete object"),
        (r#"{"key": }"#, true, "missing value"),
        ("{,}", true, "leading comma"),
    ];

    for (input, should_error, desc) in test_cases {
        let tree = parser.parse(input, None).unwrap();
        let has_error = tree.root_node().has_error();

        eprintln!("{}: '{}' -> has_error={}", desc, input, has_error);

        assert_eq!(
            has_error, should_error,
            "Mismatch for {}: expected error={}, got={}",
            desc, should_error, has_error
        );
    }
}
