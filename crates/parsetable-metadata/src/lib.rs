//! Typed metadata contract for `.parsetable` artifacts.
//!
//! This crate owns the serialization model used by tablegen and runtime2 when
//! emitting/parsing parsetable metadata payloads.

#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};
use serde::{Deserialize, Serialize};

/// Magic number identifying .parsetable files: "RSPT".
pub const MAGIC_NUMBER: [u8; 4] = [0x52, 0x53, 0x50, 0x54];

/// Current .parsetable format version.
pub const FORMAT_VERSION: u32 = 1;

/// Metadata schema version.
pub const METADATA_SCHEMA_VERSION: &str = "1.0";

/// Error type for metadata and table container operations.
#[derive(Debug, thiserror::Error)]
pub enum ParsetableError {
    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid metadata payload.
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    /// Grammar hash computation failed.
    #[error("Grammar hash computation failed: {0}")]
    HashError(String),
}

/// Metadata embedded in .parsetable files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsetableMetadata {
    /// Metadata schema version.
    pub schema_version: String,
    /// Grammar information.
    pub grammar: GrammarInfo,
    /// Generation information.
    pub generation: GenerationInfo,
    /// Parse table statistics.
    pub statistics: TableStatistics,
    /// Feature flags.
    pub features: FeatureFlags,
    /// Feature profile snapshot for this build artifact.
    #[serde(default)]
    pub feature_profile: Option<ParserFeatureProfileSnapshot>,
    /// BDD progress snapshot attached at generation time.
    #[serde(default)]
    pub governance: Option<GovernanceMetadata>,
}

impl ParsetableMetadata {
    /// Parse metadata from a JSON payload.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Parse metadata from a UTF-8 JSON string.
    pub fn parse_json(payload: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(payload)
    }
}

/// Grammar identification information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrammarInfo {
    /// Grammar name.
    pub name: String,
    /// Grammar version.
    pub version: String,
    /// Language name.
    pub language: String,
}

/// Generation metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationInfo {
    /// ISO8601 timestamp.
    pub timestamp: String,
    /// Tool version.
    pub tool_version: String,
    /// Rust compiler version.
    pub rust_version: String,
    /// Host triple.
    pub host_triple: String,
}

/// Parse table statistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableStatistics {
    /// Number of parser states.
    pub state_count: usize,
    /// Number of symbols.
    pub symbol_count: usize,
    /// Number of production rules.
    pub rule_count: usize,
    /// Number of GLR conflicts.
    pub conflict_count: usize,
    /// Number of action table cells with multiple actions.
    pub multi_action_cells: usize,
}

/// Parser feature flags for metadata export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureFlags {
    /// GLR parsing feature flag.
    pub glr_enabled: bool,
    /// External scanner support flag.
    pub external_scanner: bool,
    /// Incremental parsing support flag.
    pub incremental: bool,
}
