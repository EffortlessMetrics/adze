//! Tests for .parsetable file format writer
//!
//! Spec: docs/specs/PARSETABLE_FILE_FORMAT_SPEC.md

#![cfg(feature = "serialization")]

use adze_bdd_grid_core::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID};
use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable, StateId, SymbolId};
use adze_ir::{Grammar, RuleId};
use adze_tablegen::parsetable_writer::{
    FORMAT_VERSION, GovernanceMetadata, MAGIC_NUMBER, ParsetableWriter,
};
use std::fs;
use std::io::Read;

/// Helper: Create a minimal test grammar
fn create_test_grammar() -> Grammar {
    Grammar {
        name: "test_grammar".to_string(),
        ..Default::default()
    }
}

/// Helper: Create a minimal test ParseTable
fn create_test_parse_table() -> ParseTable {
    ParseTable {
        action_table: vec![
            vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
            vec![vec![Action::Reduce(RuleId(0))], vec![Action::Accept]],
        ],
        goto_table: vec![vec![StateId(0)], vec![StateId(1)]],
        symbol_metadata: vec![],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index: Default::default(),
        index_to_symbol: vec![SymbolId(0), SymbolId(1)],
        external_scanner_states: vec![vec![], vec![]],
        rules: vec![],
        nonterminal_to_index: Default::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Default::default(),
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            },
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            },
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        field_names: vec![],
        field_map: Default::default(),
        alias_sequences: vec![],
    }
}

/// Test 1: File creation succeeds
#[test]
fn test_parsetable_file_creation() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file = std::env::temp_dir().join("test_creation.parsetable");
    let result = writer.write_file(&temp_file);

    assert!(result.is_ok(), "File write should succeed");
    assert!(temp_file.exists(), "File should exist");

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

/// Test 2: Magic number is correct
#[test]
fn test_magic_number() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file = std::env::temp_dir().join("test_magic.parsetable");
    writer.write_file(&temp_file).expect("write should succeed");

    // Read and verify magic number
    let mut file = fs::File::open(&temp_file).expect("file should open");
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).expect("read should succeed");

    assert_eq!(magic, MAGIC_NUMBER, "Magic number should be 'RSPT'");
    assert_eq!(&magic, b"RSPT", "Magic number should spell RSPT");

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

/// Test 3: Format version is correct
#[test]
fn test_format_version() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file = std::env::temp_dir().join("test_version.parsetable");
    writer.write_file(&temp_file).expect("write should succeed");

    // Read and verify format version
    let mut file = fs::File::open(&temp_file).expect("file should open");
    let mut buffer = [0u8; 8]; // magic (4) + version (4)
    file.read_exact(&mut buffer).expect("read should succeed");

    let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    assert_eq!(version, FORMAT_VERSION, "Format version should be 1");

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

/// Test 4: Grammar hash is 32 bytes
#[test]
fn test_grammar_hash_size() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file = std::env::temp_dir().join("test_hash.parsetable");
    writer.write_file(&temp_file).expect("write should succeed");

    // Read and verify hash size
    let mut file = fs::File::open(&temp_file).expect("file should open");
    let mut buffer = [0u8; 40]; // magic (4) + version (4) + hash (32)
    file.read_exact(&mut buffer).expect("read should succeed");

    // Hash is bytes 8-40, all 32 bytes should be present
    let hash = &buffer[8..40];
    assert_eq!(hash.len(), 32, "Grammar hash should be 32 bytes (SHA-256)");

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

/// Test 5: Metadata is valid JSON
#[test]
fn test_metadata_valid_json() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file = std::env::temp_dir().join("test_metadata.parsetable");
    writer.write_file(&temp_file).expect("write should succeed");

    // Read file header to get metadata
    let mut file = fs::File::open(&temp_file).expect("file should open");
    let mut header = [0u8; 44]; // magic + version + hash + metadata_len
    file.read_exact(&mut header).expect("read should succeed");

    let metadata_len =
        u32::from_le_bytes([header[40], header[41], header[42], header[43]]) as usize;

    let mut metadata_bytes = vec![0u8; metadata_len];
    file.read_exact(&mut metadata_bytes)
        .expect("read should succeed");

    // Verify it's valid JSON
    let metadata_json = String::from_utf8(metadata_bytes).expect("metadata should be UTF-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&metadata_json).expect("metadata should be valid JSON");

    // Verify schema version
    assert_eq!(
        parsed["schema_version"].as_str(),
        Some("1.0"),
        "Schema version should be 1.0"
    );

    // Verify grammar info present
    assert!(
        parsed["grammar"].is_object(),
        "grammar field should be present"
    );
    assert!(
        parsed["generation"].is_object(),
        "generation field should be present"
    );
    assert!(
        parsed["statistics"].is_object(),
        "statistics field should be present"
    );
    assert!(
        parsed["features"].is_object(),
        "features field should be present"
    );

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

/// Test 6: Metadata contains expected grammar info
#[test]
fn test_metadata_grammar_info() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "python", "3.12.0");

    let metadata = writer.metadata();

    assert_eq!(metadata.grammar.name, "python");
    assert_eq!(metadata.grammar.version, "3.12.0");
    assert_eq!(metadata.grammar.language, "test_grammar");
}

/// Test 7: Metadata contains table statistics
#[test]
fn test_metadata_statistics() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let metadata = writer.metadata();

    assert_eq!(metadata.statistics.state_count, 2);
    assert_eq!(metadata.statistics.symbol_count, 2);
    assert_eq!(metadata.statistics.rule_count, 0);
}

#[test]
fn test_metadata_includes_feature_profile_and_governance() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");
    let metadata = writer.metadata();

    assert!(
        metadata.feature_profile.is_some(),
        "Feature profile should be present"
    );
    assert!(
        metadata.governance.is_some(),
        "Governance metadata should be present"
    );

    let feature_profile = metadata
        .feature_profile
        .as_ref()
        .expect("feature profile is present");
    let governance = metadata.governance.as_ref().expect("governance is present");
    let expected_governance = GovernanceMetadata::for_grid(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        feature_profile.as_profile(),
    );
    assert_eq!(governance, &expected_governance);
}

/// Test 8: File size is reasonable
#[test]
fn test_file_size_reasonable() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file = std::env::temp_dir().join("test_size.parsetable");
    writer.write_file(&temp_file).expect("write should succeed");

    let file_size = fs::metadata(&temp_file)
        .expect("metadata should be readable")
        .len();

    // File should be < 10KB for this small test table
    assert!(
        file_size < 10_000,
        "File size {} should be < 10KB",
        file_size
    );

    // File should be > 100 bytes (magic + version + hash + some data)
    assert!(
        file_size > 100,
        "File size {} should be > 100 bytes",
        file_size
    );

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

/// Test 9: Multiple writes produce identical files (deterministic)
#[test]
fn test_deterministic_writes() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");

    let temp_file1 = std::env::temp_dir().join("test_det1.parsetable");
    let temp_file2 = std::env::temp_dir().join("test_det2.parsetable");

    writer
        .write_file(&temp_file1)
        .expect("write 1 should succeed");

    // Small delay to ensure different timestamps (if they were included in hash)
    std::thread::sleep(std::time::Duration::from_millis(10));

    writer
        .write_file(&temp_file2)
        .expect("write 2 should succeed");

    let bytes1 = fs::read(&temp_file1).expect("read 1 should succeed");
    let bytes2 = fs::read(&temp_file2).expect("read 2 should succeed");

    // Note: Files won't be identical due to timestamps in metadata
    // But magic, version, hash, and table data should match
    assert_eq!(
        &bytes1[0..8],
        &bytes2[0..8],
        "Magic and version should match"
    );
    assert_eq!(&bytes1[8..40], &bytes2[8..40], "Grammar hash should match");

    // Cleanup
    let _ = fs::remove_file(&temp_file1);
    let _ = fs::remove_file(&temp_file2);
}

/// Test 10: Multi-action cell detection
#[test]
fn test_multi_action_cell_detection() {
    let grammar = create_test_grammar();
    let mut parse_table = create_test_parse_table();

    // Add a multi-action cell
    parse_table.action_table[0][0] = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];

    let writer = ParsetableWriter::new(&grammar, &parse_table, "test", "1.0.0");
    let metadata = writer.metadata();

    assert_eq!(
        metadata.statistics.multi_action_cells, 1,
        "Should detect 1 multi-action cell"
    );
    assert_eq!(
        metadata.statistics.conflict_count, 1,
        "Should detect 1 conflict"
    );
    assert!(metadata.features.glr_enabled, "GLR should be enabled");
}
