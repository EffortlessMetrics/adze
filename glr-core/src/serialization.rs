//! ParseTable serialization for GLR mode
//!
//! This module provides serialization/deserialization for ParseTable using postcard,
//! enabling pure-Rust GLR runtime to bypass TSLanguage ABI limitations.
//!
//! # Specification
//! - [ParseTable Serialization Spec](../../../docs/specs/PARSE_TABLE_SERIALIZATION_SPEC.md)
//! - [GLR v1 Completion Contract](../../../docs/specs/GLR_V1_COMPLETION_CONTRACT.md) (AC-4)
//!
//! # Contract
//!
//! ## Correctness
//! - Round-trip: `table == deserialize(serialize(table))`
//! - Multi-action cells must be preserved exactly
//! - No data loss through serialization
//!
//! ## Performance
//! - Serialization: < 50ms for 1000-state grammar
//! - Deserialization: < 10ms for 1000-state grammar
//! - Binary size: ≤ 2× compressed TSLanguage
//!
//! ## Safety
//! - No unsafe code in serialization/deserialization
//! - Invalid bytes return Err, never panic
//! - Format version validation
//!
//! # Example
//!
//! ```no_run
//! use adze_glr_core::ParseTable;
//!
//! # fn build_parse_table() -> ParseTable { todo!() }
//! let table = build_parse_table();
//!
//! // Serialize
//! let bytes = table.to_bytes()?;
//!
//! // Deserialize
//! let restored = ParseTable::from_bytes(&bytes)?;
//!
//! assert_eq!(bytes, restored.to_bytes()?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::ParseTable;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Current format version for ParseTable serialization
///
/// Increment this when making breaking changes to the serialization format.
/// Version history:
/// - v1: Initial implementation with bincode
/// - v2: Migrated serialization to postcard
pub const PARSE_TABLE_FORMAT_VERSION: u32 = 2;

/// Errors during ParseTable serialization
#[derive(Debug, Error)]
pub enum SerializationError {
    /// Postcard encoding failed
    #[error("Postcard encoding failed: {0}")]
    EncodingFailed(#[from] postcard::Error),

    /// ParseTable validation failed
    #[error("ParseTable validation failed: {0}")]
    ValidationFailed(String),
}

/// Errors during ParseTable deserialization
#[derive(Debug, Error)]
pub enum DeserializationError {
    /// Postcard decoding failed
    #[error("Postcard decoding failed: {0}")]
    DecodingFailed(#[from] postcard::Error),

    /// Incompatible format version
    #[error("Incompatible format version: expected {expected}, got {actual}")]
    IncompatibleVersion { expected: u32, actual: u32 },

    /// ParseTable validation failed
    #[error("ParseTable validation failed: {0}")]
    ValidationFailed(String),
}

/// Wrapper for ParseTable with version information
///
/// This struct is used for serialization to include version metadata
/// and allow for future format migrations.
#[derive(Debug, Serialize, Deserialize)]
struct VersionedParseTable {
    /// Format version for compatibility checking
    version: u32,

    /// Serialized ParseTable data
    data: Vec<u8>,
}

impl ParseTable {
    /// Serialize ParseTable to bytes using postcard
    ///
    /// # Contract
    /// - Must serialize all fields without data loss
    /// - Must be deterministic (same input → same output)
    /// - Must not panic on valid ParseTable
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: Serialized bytes
    /// - `Err(SerializationError)`: If serialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use adze_glr_core::ParseTable;
    /// # fn create_table() -> ParseTable { todo!() }
    /// let table = create_table();
    /// let bytes = table.to_bytes()?;
    /// assert!(bytes.len() > 0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializationError> {
        // Serialize the ParseTable itself first
        let table_bytes = postcard::to_stdvec(self)?;

        // Wrap with version information
        let versioned = VersionedParseTable {
            version: PARSE_TABLE_FORMAT_VERSION,
            data: table_bytes,
        };

        // Serialize the versioned wrapper
        let bytes = postcard::to_stdvec(&versioned)?;

        Ok(bytes)
    }

    /// Deserialize ParseTable from bytes
    ///
    /// # Contract
    /// - Must validate format_version compatibility
    /// - Must reconstruct exact ParseTable structure
    /// - Must preserve multi-action cells
    /// - Must not panic on invalid bytes (return Err)
    ///
    /// # Returns
    /// - `Ok(ParseTable)`: Deserialized table
    /// - `Err(DeserializationError)`: If bytes are invalid or incompatible
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use adze_glr_core::ParseTable;
    /// # fn get_serialized_bytes() -> Vec<u8> { todo!() }
    /// let bytes = get_serialized_bytes();
    /// let table = ParseTable::from_bytes(&bytes)?;
    /// assert!(table.state_count > 0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializationError> {
        // Deserialize the versioned wrapper first
        let versioned: VersionedParseTable = postcard::from_bytes(bytes)?;

        // Validate version compatibility
        if versioned.version != PARSE_TABLE_FORMAT_VERSION {
            return Err(DeserializationError::IncompatibleVersion {
                expected: PARSE_TABLE_FORMAT_VERSION,
                actual: versioned.version,
            });
        }

        // Deserialize the actual ParseTable
        let table: ParseTable = postcard::from_bytes(&versioned.data)?;

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert_eq!(PARSE_TABLE_FORMAT_VERSION, 2);
    }

    #[test]
    fn test_serialization_error_display() {
        let err = SerializationError::ValidationFailed("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_deserialization_error_display() {
        let err = DeserializationError::IncompatibleVersion {
            expected: 2,
            actual: 1,
        };
        assert!(err.to_string().contains("expected 2"));
        assert!(err.to_string().contains("got 1"));
    }
}
