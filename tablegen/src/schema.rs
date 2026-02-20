/// Parse Table Schema Validation
/// Purpose: Ensure parse tables conform to the encoding contract
/// Related: runtime/pure_parser.rs decode_action(), tablegen/compress.rs
///
/// This module provides schema validation to prevent encoding/decoding mismatches
/// between table generation and runtime parsing.
use rust_sitter_glr_core::{Action, ParseTable, StateId};
use rust_sitter_ir::RuleId;
use std::collections::HashSet;

/// Schema validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    /// Invalid action encoding
    InvalidActionEncoding {
        /// The action that failed encoding validation
        action: Action,
        /// The encoded u16 value that was produced
        encoded_value: u16,
        /// Human-readable explanation of the encoding failure
        reason: String,
    },

    /// State ID out of bounds
    InvalidStateId {
        /// The invalid state ID that was encountered
        state_id: u16,
        /// The maximum number of valid states in the table
        max_states: usize,
    },

    /// Symbol ID out of bounds
    InvalidSymbolId {
        /// The invalid symbol ID that was encountered
        symbol_id: u16,
        /// The maximum number of valid symbols in the grammar
        max_symbols: usize,
    },

    /// Production ID out of bounds
    InvalidProductionId {
        /// The invalid production ID that was encountered
        production_id: u16,
        /// The maximum number of valid productions in the grammar
        max_productions: usize,
    },

    /// Duplicate entry in action table
    DuplicateActionEntry {
        /// The state ID where the duplicate was found
        state: u16,
        /// The symbol ID where the duplicate was found
        symbol: u16,
    },

    /// Missing Accept state
    MissingAcceptState,

    /// Invalid EOF handling
    InvalidEOFHandling {
        /// Human-readable explanation of the EOF handling failure
        reason: String,
    },

    /// Compressed table integrity failure
    CompressedTableIntegrity {
        /// Human-readable explanation of the integrity failure
        reason: String,
    },
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::InvalidActionEncoding {
                action,
                encoded_value,
                reason,
            } => write!(
                f,
                "Invalid action encoding: {:?} encoded as 0x{:04X} - {}",
                action, encoded_value, reason
            ),
            SchemaError::InvalidStateId {
                state_id,
                max_states,
            } => write!(f, "State ID {} exceeds maximum {}", state_id, max_states),
            SchemaError::InvalidSymbolId {
                symbol_id,
                max_symbols,
            } => write!(f, "Symbol ID {} exceeds maximum {}", symbol_id, max_symbols),
            SchemaError::InvalidProductionId {
                production_id,
                max_productions,
            } => write!(
                f,
                "Production ID {} exceeds maximum {}",
                production_id, max_productions
            ),
            SchemaError::DuplicateActionEntry { state, symbol } => {
                write!(
                    f,
                    "Duplicate action entry for state {}, symbol {}",
                    state, symbol
                )
            }
            SchemaError::MissingAcceptState => write!(f, "No Accept action found in parse table"),
            SchemaError::InvalidEOFHandling { reason } => {
                write!(f, "Invalid EOF handling: {}", reason)
            }
            SchemaError::CompressedTableIntegrity { reason } => {
                write!(f, "Compressed table integrity check failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for SchemaError {}

/// Action encoding contract validator
///
/// Validates that actions are encoded according to the contract:
/// ```text
/// 0x0000        → Error
/// 0x0001-0x7FFF → Shift(N)
/// 0x8000-0xFFFE → Reduce(N & 0x7FFF)
/// 0xFFFF        → Accept
/// ```
pub fn validate_action_encoding(action: &Action) -> Result<u16, SchemaError> {
    match action {
        Action::Error => Ok(0x0000),

        Action::Shift(state) => {
            let state_val = state.0;
            if state_val == 0 {
                Err(SchemaError::InvalidActionEncoding {
                    action: action.clone(),
                    encoded_value: 0,
                    reason: "Shift(0) would encode as 0x0000, which is reserved for Error"
                        .to_string(),
                })
            } else if state_val >= 0x8000 {
                Err(SchemaError::InvalidActionEncoding {
                    action: action.clone(),
                    encoded_value: state_val,
                    reason: "Shift state >= 0x8000 would have high bit set, conflicting with Reduce encoding".to_string(),
                })
            } else {
                Ok(state_val)
            }
        }

        Action::Reduce(production_id) => {
            let prod_val = production_id.0;
            if prod_val >= 0x7FFF {
                Err(SchemaError::InvalidActionEncoding {
                    action: action.clone(),
                    encoded_value: 0x8000 | prod_val,
                    reason: "Reduce production ID >= 0x7FFF would encode as 0xFFFF (Accept)"
                        .to_string(),
                })
            } else {
                Ok(0x8000 | prod_val)
            }
        }

        Action::Accept => Ok(0xFFFF),

        // These variants are not directly encoded in the parse table
        // They are runtime constructs used by the GLR parser
        Action::Recover => Err(SchemaError::InvalidActionEncoding {
            action: action.clone(),
            encoded_value: 0,
            reason: "Recover actions are not encoded in parse tables (runtime only)".to_string(),
        }),
        Action::Fork(_) => Err(SchemaError::InvalidActionEncoding {
            action: action.clone(),
            encoded_value: 0,
            reason: "Fork actions are not encoded in parse tables (runtime only)".to_string(),
        }),

        // Wildcard pattern for any future Action variants (non-exhaustive enum)
        _ => Err(SchemaError::InvalidActionEncoding {
            action: action.clone(),
            encoded_value: 0,
            reason: "Unknown action variant cannot be encoded".to_string(),
        }),
    }
}

/// Validate that a u16 encoding decodes to the expected action
pub fn validate_action_decoding(encoded: u16, expected: &Action) -> Result<(), SchemaError> {
    let decoded = decode_action_from_encoding(encoded);

    if &decoded != expected {
        Err(SchemaError::InvalidActionEncoding {
            action: expected.clone(),
            encoded_value: encoded,
            reason: format!(
                "Encoding 0x{:04X} decodes to {:?}, not {:?}",
                encoded, decoded, expected
            ),
        })
    } else {
        Ok(())
    }
}

/// Decode action from u16 encoding (mirrors runtime/pure_parser.rs)
fn decode_action_from_encoding(encoded: u16) -> Action {
    if encoded == 0xFFFF {
        Action::Accept
    } else if encoded == 0 {
        Action::Error
    } else if encoded & 0x8000 != 0 {
        Action::Reduce(RuleId(encoded & 0x7FFF))
    } else {
        Action::Shift(StateId(encoded))
    }
}

/// Validate a complete parse table
pub fn validate_parse_table(table: &ParseTable) -> Result<(), Vec<SchemaError>> {
    let mut errors = Vec::new();

    // Track seen actions to detect duplicates
    let mut seen_actions: HashSet<(u16, u16)> = HashSet::new();
    let mut has_accept = false;

    // Validate all actions
    for (state_idx, action_row) in table.action_table.iter().enumerate() {
        for (symbol_idx, action_cell) in action_row.iter().enumerate() {
            for action in action_cell {
                // Validate action encoding
                match validate_action_encoding(action) {
                    Ok(encoded) => {
                        // Validate round-trip
                        if let Err(e) = validate_action_decoding(encoded, action) {
                            errors.push(e);
                        }

                        // Check for Accept
                        if matches!(action, Action::Accept) {
                            has_accept = true;
                        }
                    }
                    Err(e) => errors.push(e),
                }

                // Check for duplicate entries (same state/symbol pair)
                let key = (state_idx as u16, symbol_idx as u16);
                if action_cell.len() == 1 && seen_actions.contains(&key) {
                    errors.push(SchemaError::DuplicateActionEntry {
                        state: state_idx as u16,
                        symbol: symbol_idx as u16,
                    });
                }
                seen_actions.insert(key);

                // Validate state bounds for Shift
                if let Action::Shift(next_state) = action
                    && (next_state.0 as usize) >= table.action_table.len()
                {
                    errors.push(SchemaError::InvalidStateId {
                        state_id: next_state.0,
                        max_states: table.action_table.len(),
                    });
                }

                // Validate production bounds for Reduce
                if let Action::Reduce(production_id) = action {
                    // Note: We'd need production count from table metadata
                    // For now, just check it's not the invalid 0x7FFF value
                    if production_id.0 == 0x7FFF {
                        errors.push(SchemaError::InvalidProductionId {
                            production_id: production_id.0,
                            max_productions: 0x7FFF,
                        });
                    }
                }
            }
        }
    }

    // Check that there's at least one Accept action
    if !has_accept {
        errors.push(SchemaError::MissingAcceptState);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_encoding() {
        assert_eq!(validate_action_encoding(&Action::Error), Ok(0x0000));
    }

    #[test]
    fn test_accept_encoding() {
        assert_eq!(validate_action_encoding(&Action::Accept), Ok(0xFFFF));
    }

    #[test]
    fn test_shift_encoding() {
        assert_eq!(validate_action_encoding(&Action::Shift(StateId(1))), Ok(1));
        assert_eq!(
            validate_action_encoding(&Action::Shift(StateId(100))),
            Ok(100)
        );
        assert_eq!(
            validate_action_encoding(&Action::Shift(StateId(0x7FFF))),
            Ok(0x7FFF)
        );
    }

    #[test]
    fn test_reduce_encoding() {
        assert_eq!(
            validate_action_encoding(&Action::Reduce(RuleId(0))),
            Ok(0x8000)
        );
        assert_eq!(
            validate_action_encoding(&Action::Reduce(RuleId(1))),
            Ok(0x8001)
        );
        assert_eq!(
            validate_action_encoding(&Action::Reduce(RuleId(100))),
            Ok(0x8064)
        );
    }

    #[test]
    fn test_shift_zero_invalid() {
        let result = validate_action_encoding(&Action::Shift(StateId(0)));
        assert!(result.is_err());
        match result {
            Err(SchemaError::InvalidActionEncoding { action, .. }) => {
                assert_eq!(action, Action::Shift(StateId(0)));
            }
            _ => panic!("Expected InvalidActionEncoding error"),
        }
    }

    #[test]
    fn test_reduce_overflow_invalid() {
        let result = validate_action_encoding(&Action::Reduce(RuleId(0x7FFF)));
        assert!(result.is_err());
    }

    #[test]
    fn test_decoding_roundtrip() {
        let test_cases = vec![
            (0x0000, Action::Error),
            (0x0001, Action::Shift(StateId(1))),
            (0x7FFF, Action::Shift(StateId(0x7FFF))),
            (0x8000, Action::Reduce(RuleId(0))),
            (0x8001, Action::Reduce(RuleId(1))),
            (0xFFFE, Action::Reduce(RuleId(0x7FFE))),
            (0xFFFF, Action::Accept),
        ];

        for (encoded, expected_action) in test_cases {
            let decoded = decode_action_from_encoding(encoded);
            assert_eq!(
                decoded, expected_action,
                "Encoding 0x{:04X} should decode to {:?}",
                encoded, expected_action
            );
        }
    }
}
