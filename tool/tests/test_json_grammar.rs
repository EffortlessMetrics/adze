use std::path::Path;
use rust_sitter_tool::pure_rust_builder::{build_parser_from_grammar_js, BuildOptions};
use rust_sitter_ir::Grammar;
use rust_sitter_glr_core::FirstFollowSets;

#[test]
fn test_json_grammar_parsing() {
    // Load the JSON grammar
    let grammar_path = Path::new("../tests/grammars/json/grammar.js");
    assert!(grammar_path.exists(), "JSON grammar.js not found");
    
    // Read grammar content first to debug
    let grammar_content = std::fs::read_to_string(&grammar_path)
        .expect("Failed to read grammar.js");
    println!("Grammar.js content length: {} bytes", grammar_content.len());
    println!("First 100 chars: {}", &grammar_content[..100.min(grammar_content.len())]);
    
    // Create a temporary directory for build outputs
    let temp_dir = std::env::temp_dir().join("rust_sitter_json_test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    // Try to build the parser
    let options = BuildOptions {
        out_dir: temp_dir.to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let result = build_parser_from_grammar_js(grammar_path, options);
    
    match result {
        Ok(build_result) => {
            println!("Successfully built JSON parser!");
            println!("Grammar name: {}", build_result.grammar_name);
            
            // Check that parser code was generated
            assert!(!build_result.parser_code.is_empty());
            
            // Check that NODE_TYPES was generated
            assert!(!build_result.node_types_json.is_empty());
            
            // Parse the NODE_TYPES JSON to verify it's valid
            println!("NODE_TYPES JSON preview: {}", &build_result.node_types_json[..200.min(build_result.node_types_json.len())]);
            let node_types: serde_json::Value = serde_json::from_str(&build_result.node_types_json)
                .expect("Invalid NODE_TYPES JSON");
            
            // Debug the structure
            println!("NODE_TYPES type: {:?}", node_types);
            
            // Verify we have the expected node types
            let node_types_array = node_types.as_array().expect("NODE_TYPES should be an array");
            assert!(!node_types_array.is_empty());
            
            // Look for specific JSON node types
            let node_names: Vec<&str> = node_types_array
                .iter()
                .filter_map(|n| n["type"].as_str())
                .collect();
            
            assert!(node_names.contains(&"object"));
            assert!(node_names.contains(&"array"));
            assert!(node_names.contains(&"string"));
            assert!(node_names.contains(&"number"));
            assert!(node_names.contains(&"pair"));
        }
        Err(e) => {
            eprintln!("Build error details:");
            eprintln!("Error: {:?}", e);
            eprintln!("Current dir: {:?}", std::env::current_dir());
            eprintln!("Grammar path: {:?}", grammar_path);
            eprintln!("Temp dir: {:?}", temp_dir);
            panic!("Failed to build JSON parser: {}", e);
        }
    }
}

#[test]
fn test_json_sample_files() {
    // Create some sample JSON files to parse
    let samples = vec![
        (r#"{"hello": "world"}"#, "simple object"),
        (r#"[1, 2, 3]"#, "simple array"),
        (r#"{"nested": {"value": 42}}"#, "nested object"),
        (r#"[{"a": 1}, {"b": 2}]"#, "array of objects"),
        (r#"null"#, "null value"),
        (r#"true"#, "boolean true"),
        (r#"false"#, "boolean false"),
        (r#"123.456"#, "number"),
        (r#""hello world""#, "string"),
    ];
    
    // For now, just verify the samples are valid JSON
    for (json, description) in samples {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
        assert!(parsed.is_ok(), "Invalid JSON for {}: {}", description, json);
    }
}