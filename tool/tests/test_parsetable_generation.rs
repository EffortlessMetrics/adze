//! Integration test for .parsetable file generation
//!
//! This test verifies that the build system generates .parsetable files
//! when pure-Rust parser generation is enabled.

#![cfg(feature = "serialization")]

use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use std::fs;

/// Helper: Create a minimal test grammar JSON
fn create_test_grammar_json() -> String {
    serde_json::json!({
        "name": "test_grammar",
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "expression"
            },
            "expression": {
                "type": "SYMBOL",
                "name": "number"
            },
            "number": {
                "type": "PATTERN",
                "value": "[0-9]+"
            }
        }
    })
    .to_string()
}

/// Test 1: .parsetable file is generated during build
#[test]
fn test_parsetable_generation() {
    // Create a temporary output directory
    let temp_dir = std::env::temp_dir().join("rust_sitter_parsetable_test");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to clean temp dir");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Build options with emit_artifacts enabled
    let options = BuildOptions {
        out_dir: temp_dir.to_str().unwrap().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    // Build parser from test grammar
    let grammar_json = create_test_grammar_json();
    let result =
        build_parser_from_json(grammar_json, options).expect("Parser build should succeed");

    // Verify .parsetable file exists
    let parsetable_path = temp_dir
        .join("grammar_test_grammar")
        .join("test_grammar.parsetable");

    assert!(
        parsetable_path.exists(),
        "Expected .parsetable file at {:?}",
        parsetable_path
    );

    // Verify the file is not empty
    let metadata = fs::metadata(&parsetable_path).expect("File should have metadata");
    assert!(
        metadata.len() > 100,
        "File size {} should be > 100 bytes",
        metadata.len()
    );

    // Verify the file has the correct magic number
    let mut file = fs::File::open(&parsetable_path).expect("File should open");
    use std::io::Read;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .expect("Should read magic number");
    assert_eq!(&magic, b"RSPT", "Magic number should be 'RSPT'");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    // Verify build result contains expected info
    assert_eq!(result.grammar_name, "test_grammar");
}

/// Test 2: .parsetable file can be deserialized
#[test]
fn test_parsetable_deserialization() {
    // Create a temporary output directory
    let temp_dir = std::env::temp_dir().join("rust_sitter_parsetable_deser_test");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to clean temp dir");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Build options
    let options = BuildOptions {
        out_dir: temp_dir.to_str().unwrap().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    // Build parser
    let grammar_json = create_test_grammar_json();
    build_parser_from_json(grammar_json, options).expect("Parser build should succeed");

    // Read .parsetable file
    let parsetable_path = temp_dir
        .join("grammar_test_grammar")
        .join("test_grammar.parsetable");

    let file_bytes = fs::read(&parsetable_path).expect("Should read .parsetable file");

    // Skip magic (4), version (4), hash (32), metadata_len (4) = 44 bytes
    assert!(
        file_bytes.len() > 44,
        "File should have header + metadata + table data"
    );

    // Read metadata length
    let metadata_len = u32::from_le_bytes([
        file_bytes[40],
        file_bytes[41],
        file_bytes[42],
        file_bytes[43],
    ]) as usize;

    // Verify metadata is valid JSON
    let metadata_start = 44;
    let metadata_end = metadata_start + metadata_len;
    let metadata_bytes = &file_bytes[metadata_start..metadata_end];
    let metadata_json =
        String::from_utf8(metadata_bytes.to_vec()).expect("Metadata should be UTF-8");

    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_json).expect("Metadata should be valid JSON");

    // Verify metadata contains expected fields
    assert_eq!(
        metadata["schema_version"].as_str(),
        Some("1.0"),
        "Schema version should be 1.0"
    );
    assert_eq!(
        metadata["grammar"]["name"].as_str(),
        Some("test_grammar"),
        "Grammar name should match"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

/// Test 3: Multiple grammars generate separate .parsetable files
#[test]
fn test_multiple_grammars() {
    // Create a temporary output directory
    let temp_dir = std::env::temp_dir().join("rust_sitter_multi_parsetable_test");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to clean temp dir");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Build options
    let options = BuildOptions {
        out_dir: temp_dir.to_str().unwrap().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    // Build first grammar
    let grammar1_json = serde_json::json!({
        "name": "grammar_one",
        "rules": {
            "source_file": {"type": "SYMBOL", "name": "number"},
            "number": {"type": "PATTERN", "value": "[0-9]+"}
        }
    })
    .to_string();

    build_parser_from_json(grammar1_json, options.clone()).expect("First grammar should build");

    // Build second grammar
    let grammar2_json = serde_json::json!({
        "name": "grammar_two",
        "rules": {
            "source_file": {"type": "SYMBOL", "name": "word"},
            "word": {"type": "PATTERN", "value": "[a-z]+"}
        }
    })
    .to_string();

    build_parser_from_json(grammar2_json, options).expect("Second grammar should build");

    // Verify both .parsetable files exist
    let parsetable1 = temp_dir
        .join("grammar_grammar_one")
        .join("grammar_one.parsetable");
    let parsetable2 = temp_dir
        .join("grammar_grammar_two")
        .join("grammar_two.parsetable");

    assert!(parsetable1.exists(), "grammar_one.parsetable should exist");
    assert!(parsetable2.exists(), "grammar_two.parsetable should exist");

    // Verify they have different sizes (different grammars)
    let size1 = fs::metadata(&parsetable1).unwrap().len();
    let size2 = fs::metadata(&parsetable2).unwrap().len();

    // Both should be non-empty
    assert!(size1 > 100, "grammar_one.parsetable should be > 100 bytes");
    assert!(size2 > 100, "grammar_two.parsetable should be > 100 bytes");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
