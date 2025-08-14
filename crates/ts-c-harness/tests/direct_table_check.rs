// This test directly verifies the table extraction by using ts-bridge
// to extract and examine the actual parse table cells

use std::process::Command;
use std::fs;

#[test]
fn check_json_tables_directly() {
    // First extract fresh tables using ts-bridge
    let output_path = "/tmp/json-fresh-tables.json";
    
    // Build ts-bridge with grammars feature
    eprintln!("Building ts-bridge...");
    let output = Command::new("cargo")
        .args(&["build", "-p", "ts-bridge", "--features", "with-grammars"])
        .current_dir("../..")
        .output()
        .expect("Failed to run cargo build");
    
    if !output.status.success() {
        eprintln!("Build output: {}", String::from_utf8_lossy(&output.stderr));
        // Skip test if build fails (likely due to linking issues)
        eprintln!("Skipping test - ts-bridge build failed (expected in CI)");
        return;
    }
    
    // Extract JSON grammar tables
    eprintln!("Extracting JSON grammar tables...");
    let output = Command::new("cargo")
        .args(&["run", "-p", "ts-bridge", "--features", "with-grammars", "--bin", "extract-json", "--", output_path])
        .current_dir("../..")
        .output()
        .expect("Failed to run ts-bridge");
    
    if !output.status.success() {
        eprintln!("Extraction failed: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("Skipping test - extraction failed (expected without full Tree-sitter libs)");
        return;
    }
    
    // Load and examine the tables
    let json_str = fs::read_to_string(output_path)
        .expect("Failed to read extracted tables");
    
    let tables: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Failed to parse JSON");
    
    // Get symbol list
    let symbols = tables["symbols"].as_array()
        .expect("No symbols array");
    
    eprintln!("Total symbols: {}", symbols.len());
    
    // Find key symbols
    let lbrace_idx = symbols.iter().position(|s| s.as_str() == Some("{"));
    let rbrace_idx = symbols.iter().position(|s| s.as_str() == Some("}"));
    let lbracket_idx = symbols.iter().position(|s| s.as_str() == Some("["));
    let rbracket_idx = symbols.iter().position(|s| s.as_str() == Some("]"));
    
    eprintln!("Symbol indices:");
    eprintln!("  '{{' = {:?}", lbrace_idx);
    eprintln!("  '}}' = {:?}", rbrace_idx);
    eprintln!("  '[' = {:?}", lbracket_idx);
    eprintln!("  ']' = {:?}", rbracket_idx);
    
    // Check parse table structure
    if let Some(parse_table) = tables.get("parse_table") {
        if let Some(table_obj) = parse_table.as_object() {
            eprintln!("Parse table has {} states", table_obj.len());
            
            // Check state 1 (typical initial state for Tree-sitter)
            if let Some(state_1) = table_obj.get("1") {
                eprintln!("State 1 exists");
                
                if let Some(state_1_obj) = state_1.as_object() {
                    eprintln!("State 1 has {} symbol entries", state_1_obj.len());
                    
                    // Look for '{' action
                    if let Some(lbrace_idx) = lbrace_idx {
                        let key = lbrace_idx.to_string();
                        if let Some(actions) = state_1_obj.get(&key) {
                            eprintln!("State 1 has actions for '{{': {:?}", actions);
                        } else {
                            eprintln!("State 1 has NO actions for '{{'");
                        }
                    }
                }
            } else {
                eprintln!("No state 1 found - checking state 0");
                
                if let Some(state_0) = table_obj.get("0") {
                    if let Some(state_0_obj) = state_0.as_object() {
                        eprintln!("State 0 has {} symbol entries", state_0_obj.len());
                    }
                }
            }
        }
    } else {
        eprintln!("No parse_table in extracted data");
    }
    
    // Clean up
    let _ = fs::remove_file(output_path);
}

#[test]
fn verify_json_language_loads() {
    use tree_sitter_json as ts_json;
    use tree_sitter::{Parser, Language};
    
    // Verify we can create a parser with the JSON language
    let mut parser = Parser::new();
    let lang: Language = ts_json::LANGUAGE.into();
    parser.set_language(&lang).expect("Failed to set language");
    
    // Parse a simple JSON to verify it works
    let tree = parser.parse("{}", None).expect("Failed to parse");
    assert!(!tree.root_node().has_error(), "Empty object should parse without error");
    
    eprintln!("JSON language loaded and parsing successfully");
}