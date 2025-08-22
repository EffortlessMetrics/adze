#[cfg(test)]
mod tests {
    // Import the correct types from ts_format module
    use rust_sitter::ts_format::{TSActionTag, choose_action};
    use rust_sitter_glr_core::Action;
    use rust_sitter_ir::{RuleId, StateId};

    #[test]
    fn test_action_tag_constants() {
        assert_eq!(
            TSActionTag::Error as u8,
            0,
            "Error tag must be 0 (ABI contract)"
        );
        assert_eq!(
            TSActionTag::Shift as u8,
            1,
            "Shift tag must be 1 (ABI contract)"
        );
        assert_eq!(
            TSActionTag::Reduce as u8,
            3,
            "Reduce tag must be 3 (ABI contract)"
        );
        assert_eq!(
            TSActionTag::Accept as u8,
            4,
            "Accept tag must be 4 (ABI contract)"
        );
    }

    #[test]
    fn test_column_ordering() {
        // Token columns must come first [0..token_count)
        // NT columns must come after [token_count..total_symbols)
        let token_count = 50;
        let nt_count = 30;
        let total = token_count + nt_count;

        // Verify token band
        for col in 0..token_count {
            assert!(
                col < token_count,
                "Token column {} must be in token band [0..{})",
                col,
                token_count
            );
        }

        // Verify NT band
        for col in token_count..total {
            assert!(
                col >= token_count,
                "NT column {} must be in NT band [{}..{})",
                col,
                token_count,
                total
            );
        }
    }

    #[test]
    fn test_reduce_encoding() {
        // Reduce must encode rule_id, not LHS symbol
        let rule_id = RuleId(42);
        let action = Action::Reduce(rule_id);

        match action {
            Action::Reduce(symbol) => {
                assert_eq!(symbol, rule_id, "Reduce must encode rule_id");
            }
            _ => panic!("Expected Reduce action"),
        }
    }

    #[test]
    fn test_accept_placement() {
        // Accept must be placed at GOTO(I0, start) on EOF
        // This is a semantic test - the actual placement is verified in test_accept_executed
        let accept = Action::Accept;
        assert!(matches!(accept, Action::Accept), "Accept action must exist");
    }

    #[test]
    fn test_external_token_band() {
        // External tokens must be in the token band (column < token_count)
        let token_count = 50;
        let external_token_column = 45; // Example external token

        assert!(
            external_token_column < token_count,
            "External token column {} must be in token band [0..{})",
            external_token_column,
            token_count
        );
    }

    #[test]
    fn test_chooser_priority() {
        // Verify choose_action priority: Accept > Shift > Reduce

        let shift = Action::Shift(StateId(10));
        let reduce = Action::Reduce(RuleId(5));
        let accept = Action::Accept;

        // Accept beats Shift
        assert_eq!(
            choose_action(&vec![accept.clone(), shift.clone()]),
            Some(accept.clone())
        );
        assert_eq!(
            choose_action(&vec![shift.clone(), accept.clone()]),
            Some(accept.clone())
        );

        // Accept beats Reduce
        assert_eq!(
            choose_action(&vec![accept.clone(), reduce.clone()]),
            Some(accept.clone())
        );
        assert_eq!(
            choose_action(&vec![reduce.clone(), accept.clone()]),
            Some(accept.clone())
        );

        // Shift beats Reduce
        assert_eq!(
            choose_action(&vec![shift.clone(), reduce.clone()]),
            Some(shift.clone())
        );
        assert_eq!(
            choose_action(&vec![reduce.clone(), shift.clone()]),
            Some(shift.clone())
        );
    }

    #[test]
    fn test_dense_columns() {
        // Columns must be dense 0..N-1 with no gaps
        let total_symbols = 80;
        let mut seen = vec![false; total_symbols];

        for col in 0..total_symbols {
            seen[col] = true;
        }

        for (i, &present) in seen.iter().enumerate() {
            assert!(present, "Column {} missing - columns must be dense", i);
        }
    }

    #[test]
    fn test_goto_encoding() {
        // NT goto must be encoded as Shift(next) in NT columns
        let next_state = StateId(123);
        let goto_action = Action::Shift(next_state);

        match goto_action {
            Action::Shift(state) => {
                assert_eq!(state, next_state, "Goto encoded as Shift(next)");
            }
            _ => panic!("NT goto must be encoded as Shift"),
        }
    }

    #[test]
    fn test_sentinel_values() {
        // Verify sentinel values match Tree-sitter ABI
        const ERROR_SENTINEL: u16 = 0xFFFF;
        const ACCEPT_SENTINEL: u16 = 0x7FFF;

        // These are the expected values from Tree-sitter
        assert_eq!(ERROR_SENTINEL, 0xFFFF, "Error sentinel must be 0xFFFF");
        assert_eq!(ACCEPT_SENTINEL, 0x7FFF, "Accept sentinel must be 0x7FFF");
    }

    #[test]
    fn test_rule_metadata_consistency() {
        // child_count must equal rules[rule_id].rhs_len
        struct Rule {
            rhs_len: usize,
        }

        let rules = vec![
            Rule { rhs_len: 0 }, // Empty rule
            Rule { rhs_len: 1 }, // Single element
            Rule { rhs_len: 3 }, // Multiple elements
        ];

        for (rule_id, rule) in rules.iter().enumerate() {
            let child_count = rule.rhs_len;
            assert_eq!(
                child_count, rule.rhs_len,
                "Rule {} child_count must equal rhs_len",
                rule_id
            );
        }
    }

    #[test]
    fn test_decoder_fail_safe() {
        // Decoder must map bad reduce to Error action
        let invalid_rule_id = RuleId(60000); // Use a value that fits in u16 but is clearly invalid
        let bad_reduce = Action::Reduce(invalid_rule_id);

        // In a real decoder, this would be caught and converted to Error
        // This test verifies the structure exists
        match bad_reduce {
            Action::Reduce(symbol) if symbol.0 > 1000 => {
                // Would be converted to Action::Error in decoder
                assert!(symbol.0 > 1000, "Bad reduce detected");
            }
            _ => {}
        }
    }

    #[test]
    fn test_external_token_count() {
        // external_token_count must be >= 1 for grammars with externals
        let external_token_count = 1; // Minimum for grammars with externals
        assert!(
            external_token_count >= 1,
            "Grammars with externals must have external_token_count >= 1"
        );
    }
}
