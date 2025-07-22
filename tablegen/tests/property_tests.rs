use proptest::prelude::*;
use rust_sitter_glr_core::{Action, StateId, RuleId, SymbolId as GlrSymbolId};
use rust_sitter_tablegen::{TableCompressor, CompressedActionEntry};

/// Property: Action encoding and decoding should round-trip correctly
#[test]
fn prop_action_encoding_roundtrip() {
    let compressor = TableCompressor::new();
    
    proptest!(|(state_id: u16, rule_id: u16)| {
        // Test shift actions
        if state_id < 0x8000 {
            let action = Action::Shift(StateId(state_id));
            let encoded = compressor.encode_action_small(&action);
            prop_assert!(encoded.is_ok());
            prop_assert_eq!(encoded.unwrap(), state_id);
        }
        
        // Test reduce actions  
        if rule_id < 0x4000 {
            let action = Action::Reduce(RuleId(rule_id));
            let encoded = compressor.encode_action_small(&action);
            prop_assert!(encoded.is_ok());
            prop_assert_eq!(encoded.unwrap(), 0x8000 | (rule_id << 1));
        }
    });
}

/// Property: Accept and Error actions have fixed encodings
#[test]
fn prop_special_action_encoding() {
    let compressor = TableCompressor::new();
    
    // Accept is always 0xFFFF
    let accept = Action::Accept;
    assert_eq!(compressor.encode_action_small(&accept).unwrap(), 0xFFFF);
    
    // Error is always 0xFFFE
    let error = Action::Error;
    assert_eq!(compressor.encode_action_small(&error).unwrap(), 0xFFFE);
}

/// Property: Large state/rule IDs should fail encoding
#[test]
fn prop_large_id_encoding_fails() {
    let compressor = TableCompressor::new();
    
    proptest!(|(large_state: u16, large_rule: u16)| {
        // States >= 0x8000 should fail
        if large_state >= 0x8000 {
            let action = Action::Shift(StateId(large_state));
            prop_assert!(compressor.encode_action_small(&action).is_err());
        }
        
        // Rules >= 0x4000 should fail
        if large_rule >= 0x4000 {
            let action = Action::Reduce(RuleId(large_rule));
            prop_assert!(compressor.encode_action_small(&action).is_err());
        }
    });
}

/// Property: Compressed action table maintains semantics
#[test]
fn prop_action_table_compression_preserves_semantics() {
    proptest!(|(table_data: Vec<Vec<u8>>)| {
        // Create a random action table
        let action_table: Vec<Vec<Action>> = table_data.iter()
            .map(|row| {
                row.iter()
                    .map(|&byte| match byte % 4 {
                        0 => Action::Shift(StateId((byte as u16) % 100)),
                        1 => Action::Reduce(RuleId((byte as u16) % 50)),
                        2 => Action::Accept,
                        _ => Action::Error,
                    })
                    .collect()
            })
            .collect();
        
        if action_table.is_empty() {
            return Ok(());
        }
        
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_action_table_small(&action_table);
        
        prop_assert!(compressed.is_ok());
        let compressed = compressed.unwrap();
        
        // Check that row offsets are monotonic
        for i in 1..compressed.row_offsets.len() {
            prop_assert!(compressed.row_offsets[i] >= compressed.row_offsets[i-1]);
        }
        
        // Check that default actions are set for each row
        prop_assert_eq!(compressed.default_actions.len(), action_table.len());
    });
}

/// Property: Goto table compression with run-length encoding
#[test]
fn prop_goto_table_run_length_encoding() {
    use rust_sitter_tablegen::CompressedGotoEntry;
    
    proptest!(|(run_length: u8, state: u16)| {
        let run_length = (run_length % 10) + 1; // 1-10
        let state_id = StateId(state % 1000);
        
        // Create a goto table with runs
        let goto_row = vec![state_id; run_length as usize];
        let goto_table = vec![goto_row];
        
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_goto_table_small(&goto_table);
        
        prop_assert!(compressed.is_ok());
        let compressed = compressed.unwrap();
        
        // Runs of 3+ should be compressed
        if run_length >= 3 {
            let has_run_length = compressed.data.iter().any(|entry| {
                matches!(entry, CompressedGotoEntry::RunLength { .. })
            });
            prop_assert!(has_run_length);
        }
        
        // Total decompressed size should match original
        let decompressed_count: usize = compressed.data.iter()
            .map(|entry| match entry {
                CompressedGotoEntry::Single(_) => 1,
                CompressedGotoEntry::RunLength { count, .. } => *count as usize,
            })
            .sum();
        prop_assert_eq!(decompressed_count, run_length as usize);
    });
}

/// Property: Symbol count limits
#[test]
fn prop_symbol_count_limits() {
    proptest!(|(symbol_count: u32, state_count: u32)| {
        // Tree-sitter has practical limits on these
        let symbol_count = symbol_count % 10000;
        let state_count = state_count % 50000;
        
        let parse_table = rust_sitter_glr_core::ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: state_count as usize,
            symbol_count: symbol_count as usize,
        };
        
        let compressed = rust_sitter_tablegen::compress::CompressedParseTable::from_parse_table(&parse_table);
        
        prop_assert_eq!(compressed.symbol_count(), symbol_count as usize);
        prop_assert_eq!(compressed.state_count(), state_count as usize);
    });
}

/// Property: Field names must maintain lexicographic order
#[test]
fn prop_field_names_ordering() {
    use rust_sitter_ir::{Grammar, FieldId};
    
    proptest!(|(mut field_names: Vec<String>)| {
        // Ensure unique names
        field_names.sort();
        field_names.dedup();
        
        if field_names.is_empty() {
            return Ok(());
        }
        
        let mut grammar = Grammar::new("test".to_string());
        
        // Add fields in order
        for (i, name) in field_names.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        
        // Verify they're stored in order
        let stored: Vec<_> = grammar.fields.values().cloned().collect();
        for i in 1..stored.len() {
            prop_assert!(stored[i-1] < stored[i], 
                "Field names not in lexicographic order: {} >= {}", 
                stored[i-1], stored[i]);
        }
    });
}

/// Property: Table compression is deterministic
#[test]
fn prop_compression_deterministic() {
    proptest!(|(seed: u64)| {
        // Generate a deterministic action table from seed
        let mut rng = proptest::test_runner::TestRng::from_seed(
            proptest::test_runner::RngAlgorithm::ChaCha,
            &seed.to_le_bytes()
        );
        
        let action_table: Vec<Vec<Action>> = (0..10)
            .map(|_| {
                (0..10)
                    .map(|_| {
                        if rng.gen_bool(0.5) {
                            Action::Shift(StateId(rng.gen_range(0..100)))
                        } else {
                            Action::Error
                        }
                    })
                    .collect()
            })
            .collect();
        
        let compressor = TableCompressor::new();
        
        // Compress twice
        let compressed1 = compressor.compress_action_table_small(&action_table).unwrap();
        let compressed2 = compressor.compress_action_table_small(&action_table).unwrap();
        
        // Should be identical
        prop_assert_eq!(compressed1.data.len(), compressed2.data.len());
        prop_assert_eq!(compressed1.row_offsets, compressed2.row_offsets);
        
        // Compare default actions
        for (a1, a2) in compressed1.default_actions.iter().zip(&compressed2.default_actions) {
            prop_assert_eq!(a1, a2);
        }
    });
}

/// Property: Encoded values fit in u16
#[test]
fn prop_encoded_values_fit_u16() {
    let compressor = TableCompressor::new();
    
    proptest!(|(state: u16, rule: u16)| {
        // All valid actions should encode to u16 values
        let actions = vec![
            Action::Shift(StateId(state % 0x8000)),
            Action::Reduce(RuleId(rule % 0x4000)),
            Action::Accept,
            Action::Error,
        ];
        
        for action in actions {
            if let Ok(encoded) = compressor.encode_action_small(&action) {
                // Encoded value is already u16, just verify it's in valid ranges
                match action {
                    Action::Shift(_) => prop_assert!(encoded < 0x8000),
                    Action::Reduce(_) => prop_assert!(encoded >= 0x8000 && encoded < 0xFFFE),
                    Action::Accept => prop_assert_eq!(encoded, 0xFFFF),
                    Action::Error => prop_assert_eq!(encoded, 0xFFFE),
                    _ => {}
                }
            }
        }
    });
}

/// Property: Row offsets are valid
#[test]
fn prop_row_offsets_valid() {
    proptest!(|(num_rows: u8)| {
        let num_rows = (num_rows % 50) as usize + 1;
        
        // Create action table with variable row sizes
        let action_table: Vec<Vec<Action>> = (0..num_rows)
            .map(|i| {
                (0..((i % 10) + 1))
                    .map(|j| Action::Shift(StateId((i * 10 + j) as u16)))
                    .collect()
            })
            .collect();
        
        let compressor = TableCompressor::new();
        let compressed = compressor.compress_action_table_small(&action_table).unwrap();
        
        // Row offsets should have num_rows + 1 entries (includes sentinel)
        prop_assert_eq!(compressed.row_offsets.len(), num_rows + 1);
        
        // Each row offset should point to valid data range
        for i in 0..num_rows {
            let start = compressed.row_offsets[i] as usize;
            let end = compressed.row_offsets[i + 1] as usize;
            prop_assert!(start <= end);
            prop_assert!(end <= compressed.data.len());
        }
    });
}