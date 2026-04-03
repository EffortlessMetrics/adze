//! Contract lock test - verifies that public API remains stable.
//!
//! This crate owns the serialization model used by tablegen and runtime2.

use adze_parsetable_metadata::{
    FORMAT_VERSION, FeatureFlags, GenerationInfo, GrammarInfo, MAGIC_NUMBER,
    METADATA_SCHEMA_VERSION, ParsetableError, ParsetableMetadata, TableStatistics,
};

/// Verify all public constants exist with expected values.
#[test]
fn test_contract_lock_constants() {
    // Verify MAGIC_NUMBER constant exists
    assert_eq!(&MAGIC_NUMBER, b"RSPT");

    // Verify FORMAT_VERSION constant exists
    const { assert!(FORMAT_VERSION > 0) };

    // Verify METADATA_SCHEMA_VERSION constant exists
    assert!(!METADATA_SCHEMA_VERSION.is_empty());
}

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ParsetableError enum exists with expected variants
    let _io_err = ParsetableError::Io(std::io::Error::other("test"));
    let _ser_err = ParsetableError::Serialization("bad json".to_string());
    let _meta_err = ParsetableError::InvalidMetadata("missing field".to_string());
    let _hash_err = ParsetableError::HashError("sha256 failed".to_string());

    // Verify Debug trait is implemented
    let _debug = format!("{_ser_err:?}");

    // Verify Display trait is implemented (via thiserror::Error)
    let _display = format!("{_ser_err}");

    // Verify GrammarInfo struct exists with expected fields
    let _grammar = GrammarInfo {
        name: "json".into(),
        version: "1.0.0".into(),
        language: "json".into(),
    };

    // Verify GenerationInfo struct exists with expected fields
    let _generation = GenerationInfo {
        timestamp: "2025-01-01T00:00:00Z".into(),
        tool_version: "0.1.0".into(),
        rust_version: "1.92.0".into(),
        host_triple: "x86_64-unknown-linux-gnu".into(),
    };

    // Verify TableStatistics struct exists with expected fields
    let _stats = TableStatistics {
        state_count: 10,
        symbol_count: 5,
        rule_count: 3,
        conflict_count: 0,
        multi_action_cells: 0,
    };

    // Verify FeatureFlags struct exists with expected fields
    let _flags = FeatureFlags {
        glr_enabled: false,
        external_scanner: false,
        incremental: false,
    };

    // Verify ParsetableMetadata struct exists with expected fields
    let _metadata = ParsetableMetadata {
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
    };
}

/// Verify ParsetableMetadata methods exist.
#[test]
fn test_contract_lock_metadata_methods() {
    let metadata = ParsetableMetadata {
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
    };

    // Verify from_bytes method exists
    let json = serde_json::to_string(&metadata).unwrap();
    let bytes = json.as_bytes();
    let parsed = ParsetableMetadata::from_bytes(bytes).unwrap();
    assert_eq!(metadata, parsed);

    // Verify parse_json method exists
    let parsed2 = ParsetableMetadata::parse_json(&json).unwrap();
    assert_eq!(metadata, parsed2);
}

/// Verify serde roundtrip for all types.
#[test]
fn test_contract_lock_serde_roundtrip() {
    // FeatureFlags roundtrip
    let flags = FeatureFlags {
        glr_enabled: true,
        external_scanner: false,
        incremental: true,
    };
    let json = serde_json::to_string(&flags).unwrap();
    let deserialized: FeatureFlags = serde_json::from_str(&json).unwrap();
    assert_eq!(flags, deserialized);

    // TableStatistics roundtrip
    let stats = TableStatistics {
        state_count: 100,
        symbol_count: 50,
        rule_count: 30,
        conflict_count: 2,
        multi_action_cells: 5,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: TableStatistics = serde_json::from_str(&json).unwrap();
    assert_eq!(stats, deserialized);

    // GrammarInfo roundtrip
    let grammar = GrammarInfo {
        name: "python".into(),
        version: "3.12".into(),
        language: "python".into(),
    };
    let json = serde_json::to_string(&grammar).unwrap();
    let deserialized: GrammarInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar, deserialized);

    // GenerationInfo roundtrip
    let generation = GenerationInfo {
        timestamp: "2025-06-15T12:00:00Z".into(),
        tool_version: "0.8.0".into(),
        rust_version: "1.92.0".into(),
        host_triple: "aarch64-unknown-linux-gnu".into(),
    };
    let json = serde_json::to_string(&generation).unwrap();
    let deserialized: GenerationInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(generation, deserialized);
}

/// Verify ParsetableError Display implementation.
#[test]
fn test_contract_lock_error_display() {
    let err = ParsetableError::Serialization("bad json".to_string());
    assert!(format!("{err}").contains("bad json"));

    let err = ParsetableError::InvalidMetadata("missing field".to_string());
    assert!(format!("{err}").contains("missing field"));

    let err = ParsetableError::HashError("sha256 failed".to_string());
    assert!(format!("{err}").contains("sha256 failed"));

    let err = ParsetableError::Io(std::io::Error::other("io error"));
    assert!(format!("{err}").contains("I/O error"));
}

/// Verify re-exported governance types.
#[test]
fn test_contract_lock_reexports() {
    // Verify GovernanceMetadata is accessible
    use adze_parsetable_metadata::GovernanceMetadata;
    let _meta = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");

    // Verify ParserFeatureProfileSnapshot is accessible
    use adze_parsetable_metadata::ParserFeatureProfileSnapshot;
    let _snap = ParserFeatureProfileSnapshot::new(true, false, true, false);
}
