// Comprehensive tests for parsetable-metadata
use adze_parsetable_metadata::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn magic_number_is_rspt() {
    assert_eq!(MAGIC_NUMBER, [0x52, 0x53, 0x50, 0x54]);
    assert_eq!(&MAGIC_NUMBER, b"RSPT");
}

#[test]
fn format_version() {
    assert_eq!(FORMAT_VERSION, 1);
}

#[test]
fn schema_version() {
    assert_eq!(METADATA_SCHEMA_VERSION, "1.0");
}

// ---------------------------------------------------------------------------
// Helper to build metadata
// ---------------------------------------------------------------------------

fn sample_metadata() -> ParsetableMetadata {
    ParsetableMetadata {
        schema_version: METADATA_SCHEMA_VERSION.to_string(),
        grammar: GrammarInfo {
            name: "json".into(),
            version: "1.0.0".into(),
            language: "json".into(),
        },
        generation: GenerationInfo {
            timestamp: "2025-01-01T00:00:00Z".into(),
            tool_version: "0.1.0".into(),
            rust_version: "1.92.0".into(),
            host_triple: "x86_64-unknown-linux-gnu".into(),
        },
        statistics: TableStatistics {
            state_count: 10,
            symbol_count: 5,
            rule_count: 3,
            conflict_count: 0,
            multi_action_cells: 0,
        },
        features: FeatureFlags {
            glr_enabled: false,
            external_scanner: false,
            incremental: false,
        },
        feature_profile: None,
        governance: None,
    }
}

// ---------------------------------------------------------------------------
// Serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn json_roundtrip() {
    let m = sample_metadata();
    let json = serde_json::to_string(&m).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();
    assert_eq!(m, parsed);
}

#[test]
fn json_pretty_roundtrip() {
    let m = sample_metadata();
    let json = serde_json::to_string_pretty(&m).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();
    assert_eq!(m, parsed);
}

#[test]
fn bytes_roundtrip() {
    let m = sample_metadata();
    let bytes = serde_json::to_vec(&m).unwrap();
    let parsed = ParsetableMetadata::from_bytes(&bytes).unwrap();
    assert_eq!(m, parsed);
}

// ---------------------------------------------------------------------------
// GrammarInfo
// ---------------------------------------------------------------------------

#[test]
fn grammar_info_fields() {
    let g = GrammarInfo {
        name: "test".into(),
        version: "0.1.0".into(),
        language: "rust".into(),
    };
    assert_eq!(g.name, "test");
    assert_eq!(g.version, "0.1.0");
    assert_eq!(g.language, "rust");
}

#[test]
fn grammar_info_debug() {
    let g = GrammarInfo {
        name: "x".into(),
        version: "1.0".into(),
        language: "y".into(),
    };
    let d = format!("{:?}", g);
    assert!(d.contains("GrammarInfo"));
}

// ---------------------------------------------------------------------------
// GenerationInfo
// ---------------------------------------------------------------------------

#[test]
fn generation_info_fields() {
    let g = GenerationInfo {
        timestamp: "now".into(),
        tool_version: "0.1".into(),
        rust_version: "1.92".into(),
        host_triple: "x86_64".into(),
    };
    assert_eq!(g.timestamp, "now");
    assert_eq!(g.tool_version, "0.1");
}

// ---------------------------------------------------------------------------
// TableStatistics
// ---------------------------------------------------------------------------

#[test]
fn table_stats_fields() {
    let s = TableStatistics {
        state_count: 100,
        symbol_count: 50,
        rule_count: 25,
        conflict_count: 3,
        multi_action_cells: 7,
    };
    assert_eq!(s.state_count, 100);
    assert_eq!(s.symbol_count, 50);
    assert_eq!(s.rule_count, 25);
    assert_eq!(s.conflict_count, 3);
    assert_eq!(s.multi_action_cells, 7);
}

// ---------------------------------------------------------------------------
// FeatureFlags
// ---------------------------------------------------------------------------

#[test]
fn feature_flags_all_false() {
    let f = FeatureFlags {
        glr_enabled: false,
        external_scanner: false,
        incremental: false,
    };
    assert!(!f.glr_enabled);
    assert!(!f.external_scanner);
    assert!(!f.incremental);
}

#[test]
fn feature_flags_all_true() {
    let f = FeatureFlags {
        glr_enabled: true,
        external_scanner: true,
        incremental: true,
    };
    assert!(f.glr_enabled);
    assert!(f.external_scanner);
    assert!(f.incremental);
}

// ---------------------------------------------------------------------------
// Optional fields
// ---------------------------------------------------------------------------

#[test]
fn metadata_with_feature_profile() {
    let mut m = sample_metadata();
    m.feature_profile = Some(ParserFeatureProfileSnapshot::new(true, false, true, false));
    let json = serde_json::to_string(&m).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();
    assert_eq!(m, parsed);
}

#[test]
fn metadata_with_governance() {
    let mut m = sample_metadata();
    m.governance = Some(GovernanceMetadata::with_counts("test", 5, 10, "half done"));
    let json = serde_json::to_string(&m).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();
    assert_eq!(m, parsed);
}

#[test]
fn metadata_without_optional_fields() {
    let m = sample_metadata();
    assert!(m.feature_profile.is_none());
    assert!(m.governance.is_none());
    let json = serde_json::to_string(&m).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();
    assert_eq!(parsed.feature_profile, None);
    assert_eq!(parsed.governance, None);
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn parse_invalid_json() {
    let result = ParsetableMetadata::parse_json("not json");
    assert!(result.is_err());
}

#[test]
fn parse_empty_json() {
    let result = ParsetableMetadata::parse_json("{}");
    assert!(result.is_err());
}

#[test]
fn from_bytes_invalid() {
    let result = ParsetableMetadata::from_bytes(b"invalid");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// ParsetableError variants
// ---------------------------------------------------------------------------

#[test]
fn error_serialization() {
    let e = ParsetableError::Serialization("bad".into());
    let msg = format!("{}", e);
    assert!(msg.contains("Serialization"));
}

#[test]
fn error_invalid_metadata() {
    let e = ParsetableError::InvalidMetadata("missing field".into());
    let msg = format!("{}", e);
    assert!(msg.contains("Invalid metadata"));
}

#[test]
fn error_hash() {
    let e = ParsetableError::HashError("hash failed".into());
    let msg = format!("{}", e);
    assert!(msg.contains("hash"));
}

#[test]
fn error_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e = ParsetableError::Io(io_err);
    let msg = format!("{}", e);
    assert!(msg.contains("I/O"));
}

// ---------------------------------------------------------------------------
// Clone and equality
// ---------------------------------------------------------------------------

#[test]
fn metadata_clone() {
    let m = sample_metadata();
    let m2 = m.clone();
    assert_eq!(m, m2);
}

#[test]
fn metadata_ne() {
    let m1 = sample_metadata();
    let mut m2 = sample_metadata();
    m2.grammar.name = "different".into();
    assert_ne!(m1, m2);
}
