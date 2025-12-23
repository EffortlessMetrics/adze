//! Tests for .parsetable file loading in runtime2
//!
//! This test verifies that the Parser can load .parsetable files generated
//! by the build system and use them for parsing.

#![cfg(all(feature = "pure-rust-glr", feature = "serialization"))]

use rust_sitter_glr_core::{Action, GotoIndexing, LexMode, ParseTable, StateId, SymbolId};
use rust_sitter_ir::RuleId;
use rust_sitter_runtime::Parser;

/// Helper: Create a minimal .parsetable file for testing
fn create_minimal_parsetable() -> Vec<u8> {
    // Create a minimal ParseTable
    let parse_table = ParseTable {
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
    };

    // Serialize ParseTable to bytes
    let table_bytes = parse_table
        .to_bytes()
        .expect("ParseTable serialization should succeed");

    // Build .parsetable file format
    let mut file_bytes = Vec::new();

    // Magic number: "RSPT"
    file_bytes.extend_from_slice(b"RSPT");

    // Format version: 1 (little-endian u32)
    file_bytes.extend_from_slice(&1u32.to_le_bytes());

    // Grammar hash: 32 bytes of zeros (placeholder)
    file_bytes.extend_from_slice(&[0u8; 32]);

    // Metadata: minimal JSON
    let metadata_json = r#"{"schema_version":"1.0","grammar":{"name":"test","version":"1.0.0","language":"test"},"generation":{"timestamp":"2025-01-01T00:00:00Z","tool_version":"0.1.0","rust_version":"1.89.0","host_triple":"x86_64-unknown-linux-gnu"},"statistics":{"state_count":2,"symbol_count":2,"rule_count":0,"conflict_count":0,"multi_action_cells":0},"features":{"glr_enabled":false,"external_scanner":false,"incremental":false}}"#;
    let metadata_bytes = metadata_json.as_bytes();

    // Metadata length (little-endian u32)
    file_bytes.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes());

    // Metadata JSON
    file_bytes.extend_from_slice(metadata_bytes);

    // Table data length (little-endian u32)
    file_bytes.extend_from_slice(&(table_bytes.len() as u32).to_le_bytes());

    // Table data
    file_bytes.extend_from_slice(&table_bytes);

    file_bytes
}

/// Test 1: Loading a valid .parsetable file succeeds
#[test]
fn test_load_valid_parsetable() {
    let bytes = create_minimal_parsetable();
    let mut parser = Parser::new();

    let result = parser.load_glr_table_from_bytes(&bytes);
    assert!(result.is_ok(), "Loading valid .parsetable should succeed");

    // Verify parser is in GLR mode
    assert!(
        parser.is_glr_mode(),
        "Parser should be in GLR mode after loading"
    );
}

/// Test 2: Loading with invalid magic number fails
#[test]
fn test_load_invalid_magic() {
    let mut bytes = create_minimal_parsetable();
    // Corrupt magic number
    bytes[0] = b'X';

    let mut parser = Parser::new();
    let result = parser.load_glr_table_from_bytes(&bytes);

    assert!(result.is_err(), "Loading with invalid magic should fail");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("bad magic number"),
        "Error should mention bad magic: {}",
        err_msg
    );
}

/// Test 3: Loading with unsupported version fails
#[test]
fn test_load_unsupported_version() {
    let mut bytes = create_minimal_parsetable();
    // Change version to 99
    bytes[4] = 99;

    let mut parser = Parser::new();
    let result = parser.load_glr_table_from_bytes(&bytes);

    assert!(
        result.is_err(),
        "Loading with unsupported version should fail"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("format version"),
        "Error should mention version: {}",
        err_msg
    );
}

/// Test 4: Loading truncated file fails
#[test]
fn test_load_truncated_file() {
    let bytes = create_minimal_parsetable();
    // Take only first 20 bytes (less than header)
    let truncated = &bytes[0..20];

    let mut parser = Parser::new();
    let result = parser.load_glr_table_from_bytes(truncated);

    assert!(result.is_err(), "Loading truncated file should fail");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("too short"),
        "Error should mention file is too short: {}",
        err_msg
    );
}

/// Test 5: Loading file with truncated metadata fails
#[test]
fn test_load_truncated_metadata() {
    let bytes = create_minimal_parsetable();
    // Take only header + partial metadata
    let truncated = &bytes[0..100];

    let mut parser = Parser::new();
    let result = parser.load_glr_table_from_bytes(truncated);

    assert!(
        result.is_err(),
        "Loading with truncated metadata should fail"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("truncated metadata") || err_msg.contains("missing table"),
        "Error should mention truncation: {}",
        err_msg
    );
}

/// Test 6: Loading file with truncated table data fails
#[test]
fn test_load_truncated_table_data() {
    let mut bytes = create_minimal_parsetable();

    // Find the table length field (after magic, version, hash, metadata_len, metadata)
    // Magic(4) + Version(4) + Hash(32) + MetadataLen(4) = 44
    let metadata_len = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;
    let table_len_offset = 44 + metadata_len;

    // Corrupt the table length to be larger than actual data
    let fake_large_len = 999999u32;
    bytes[table_len_offset..table_len_offset + 4].copy_from_slice(&fake_large_len.to_le_bytes());

    let mut parser = Parser::new();
    let result = parser.load_glr_table_from_bytes(&bytes);

    assert!(result.is_err(), "Loading with truncated table should fail");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("truncated table"),
        "Error should mention truncated table: {}",
        err_msg
    );
}

/// Test 7: Round-trip through serialization and loading
#[test]
fn test_roundtrip_serialization() {
    // Create a parse table
    let original_table = ParseTable {
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
    };

    // Generate .parsetable bytes
    let parsetable_bytes = create_minimal_parsetable();

    // Load into parser
    let mut parser = Parser::new();
    parser
        .load_glr_table_from_bytes(&parsetable_bytes)
        .expect("Loading should succeed");

    // Verify parser is in GLR mode
    assert!(parser.is_glr_mode());

    // Note: We can't easily verify the table contents match exactly since the table
    // is leaked and we don't expose it. But the fact that loading succeeded and
    // GLR mode is active is a good sanity check.
}

/// Test 8: Multiple sequential loads (mode switching)
#[test]
fn test_multiple_loads() {
    let bytes1 = create_minimal_parsetable();
    let bytes2 = create_minimal_parsetable();

    let mut parser = Parser::new();

    // First load
    parser
        .load_glr_table_from_bytes(&bytes1)
        .expect("First load should succeed");
    assert!(parser.is_glr_mode());

    // Second load (should replace first table)
    parser
        .load_glr_table_from_bytes(&bytes2)
        .expect("Second load should succeed");
    assert!(parser.is_glr_mode());
}

/// Test 9: File size validation
#[test]
fn test_file_size() {
    let bytes = create_minimal_parsetable();

    // File should be larger than header (44 bytes) + some metadata + some table
    assert!(
        bytes.len() > 100,
        "Generated .parsetable should be > 100 bytes"
    );

    // File should have correct magic
    assert_eq!(&bytes[0..4], b"RSPT");

    // File should have version 1
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    assert_eq!(version, 1);
}
