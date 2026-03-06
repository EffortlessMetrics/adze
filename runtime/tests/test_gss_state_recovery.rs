#[cfg(feature = "incremental_glr")]
#[cfg(test)]
mod incremental_glr_tests {
    #[cfg(feature = "ts-compat")]
    use adze::adze_ir as ir;
    use adze::glr_incremental::{
        ChunkIdentifier, GLREdit, GLRToken, SUBTREE_REUSE_COUNT, get_reuse_count,
        reset_reuse_counter,
    };

    #[cfg(not(feature = "ts-compat"))]
    use adze_ir as ir;

    use ir::SymbolId;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_subtree_reuse_counter() {
        // Reset the counter
        reset_reuse_counter();
        assert_eq!(get_reuse_count(), 0);

        // Simulate reuse
        SUBTREE_REUSE_COUNT.fetch_add(1, Ordering::SeqCst);
        SUBTREE_REUSE_COUNT.fetch_add(1, Ordering::SeqCst);

        assert_eq!(SUBTREE_REUSE_COUNT.load(Ordering::SeqCst), 2);
        assert_eq!(get_reuse_count(), 2);
    }

    #[test]
    fn test_chunk_identifier_prefix_boundary() {
        let edit = GLREdit {
            old_range: 6..7,
            new_text: b"*".to_vec(),
            old_token_range: 3..4,
            new_tokens: vec![],
            old_tokens: vec![],
            old_forest: None,
        };
        let chunk_id = ChunkIdentifier::new(None, &edit);

        // Create test tokens representing "1 + 2 - 3" -> "1 + 2 * 3"
        let old_tokens = vec![
            GLRToken {
                symbol: SymbolId(1),
                text: b"1".to_vec(),
                start_byte: 0,
                end_byte: 1,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"+".to_vec(),
                start_byte: 2,
                end_byte: 3,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"2".to_vec(),
                start_byte: 4,
                end_byte: 5,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"-".to_vec(), // This gets changed to *
                start_byte: 6,
                end_byte: 7,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"3".to_vec(),
                start_byte: 8,
                end_byte: 9,
            },
        ];

        let new_tokens = vec![
            GLRToken {
                symbol: SymbolId(1),
                text: b"1".to_vec(),
                start_byte: 0,
                end_byte: 1,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"+".to_vec(),
                start_byte: 2,
                end_byte: 3,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"2".to_vec(),
                start_byte: 4,
                end_byte: 5,
            },
            GLRToken {
                symbol: SymbolId(3), // Different symbol for *
                text: b"*".to_vec(),
                start_byte: 6,
                end_byte: 7,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"3".to_vec(),
                start_byte: 8,
                end_byte: 9,
            },
        ];

        // Test prefix boundary - should find 3 unchanged tokens before the edit
        let prefix_len = chunk_id.find_prefix_boundary(&old_tokens, &new_tokens);
        assert_eq!(prefix_len, 3); // "1", "+", "2" are unchanged and before edit
    }

    #[test]
    fn test_chunk_identifier_suffix_boundary() {
        let edit = GLREdit {
            old_range: 6..7,
            new_text: b"*".to_vec(),
            old_token_range: 3..4,
            new_tokens: vec![],
            old_tokens: vec![],
            old_forest: None,
        };
        let chunk_id = ChunkIdentifier::new(None, &edit);

        let old_tokens = vec![
            GLRToken {
                symbol: SymbolId(1),
                text: b"1".to_vec(),
                start_byte: 0,
                end_byte: 1,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"+".to_vec(),
                start_byte: 2,
                end_byte: 3,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"2".to_vec(),
                start_byte: 4,
                end_byte: 5,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"-".to_vec(), // This gets changed
                start_byte: 6,
                end_byte: 7,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"3".to_vec(),
                start_byte: 8,
                end_byte: 9,
            },
        ];

        let new_tokens = vec![
            GLRToken {
                symbol: SymbolId(1),
                text: b"1".to_vec(),
                start_byte: 0,
                end_byte: 1,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"+".to_vec(),
                start_byte: 2,
                end_byte: 3,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"2".to_vec(),
                start_byte: 4,
                end_byte: 5,
            },
            GLRToken {
                symbol: SymbolId(3),
                text: b"*".to_vec(),
                start_byte: 6,
                end_byte: 7,
            },
            GLRToken {
                symbol: SymbolId(1),
                text: b"3".to_vec(),
                start_byte: 8,
                end_byte: 9,
            },
        ];

        // Test suffix boundary - should find 1 unchanged token after the edit
        let edit_delta = 0; // Same length replacement
        let suffix_len = chunk_id.find_suffix_boundary(&old_tokens, &new_tokens, edit_delta);
        assert_eq!(suffix_len, 1); // "3" is unchanged and after edit
    }

    #[test]
    fn test_glr_token_creation() {
        let token = GLRToken {
            symbol: SymbolId(42),
            text: b"test".to_vec(),
            start_byte: 10,
            end_byte: 14,
        };

        assert_eq!(token.symbol, SymbolId(42));
        assert_eq!(token.text, b"test");
        assert_eq!(token.start_byte, 10);
        assert_eq!(token.end_byte, 14);
    }

    #[test]
    fn test_glr_edit_creation() {
        let edit = GLREdit {
            old_range: 5..8,
            new_text: b"new".to_vec(),
            old_token_range: 2..3,
            new_tokens: vec![GLRToken {
                symbol: SymbolId(1),
                text: b"new".to_vec(),
                start_byte: 5,
                end_byte: 8,
            }],
            old_tokens: vec![GLRToken {
                symbol: SymbolId(2),
                text: b"old".to_vec(),
                start_byte: 5,
                end_byte: 8,
            }],
            old_forest: None,
        };

        assert_eq!(edit.old_range, 5..8);
        assert_eq!(edit.new_text, b"new");
        assert_eq!(edit.old_token_range, 2..3);
        assert_eq!(edit.new_tokens.len(), 1);
        assert_eq!(edit.old_tokens.len(), 1);
    }
}
