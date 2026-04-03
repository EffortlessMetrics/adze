//! BDD tests for parsetable-metadata crate.
//!
//! These tests verify the public API behavior using Given/When/Then style.

use adze_parsetable_metadata::*;

// =============================================================================
// Magic Number and Constants Tests
// =============================================================================

#[test]
fn given_magic_number_when_checking_value_then_is_rspt() {
    // Given / When
    let magic = MAGIC_NUMBER;

    // Then
    assert_eq!(&magic, b"RSPT");
}

#[test]
fn given_format_version_when_checking_value_then_is_positive() {
    // Given / When
    let version = FORMAT_VERSION;

    // Then
    assert!(version > 0);
}

#[test]
fn given_metadata_schema_version_when_checking_value_then_is_non_empty() {
    // Given / When
    let schema = METADATA_SCHEMA_VERSION;

    // Then
    assert!(!schema.is_empty());
}

// =============================================================================
// FeatureFlags Tests
// =============================================================================

#[test]
fn given_feature_flags_when_all_disabled_then_serializes_correctly() {
    // Given
    let flags = FeatureFlags {
        glr_enabled: false,
        external_scanner: false,
        incremental: false,
    };

    // When
    let json = serde_json::to_string(&flags).unwrap();
    let parsed: FeatureFlags = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(flags, parsed);
}

#[test]
fn given_feature_flags_when_all_enabled_then_serializes_correctly() {
    // Given
    let flags = FeatureFlags {
        glr_enabled: true,
        external_scanner: true,
        incremental: true,
    };

    // When
    let json = serde_json::to_string(&flags).unwrap();
    let parsed: FeatureFlags = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(flags, parsed);
}

#[test]
fn given_feature_flags_when_only_glr_enabled_then_correct_state() {
    // Given
    let flags = FeatureFlags {
        glr_enabled: true,
        external_scanner: false,
        incremental: false,
    };

    // When / Then
    assert!(flags.glr_enabled);
    assert!(!flags.external_scanner);
    assert!(!flags.incremental);
}

// =============================================================================
// TableStatistics Tests
// =============================================================================

#[test]
fn given_table_statistics_when_creating_then_holds_values() {
    // Given
    let stats = TableStatistics {
        state_count: 100,
        symbol_count: 50,
        rule_count: 30,
        conflict_count: 2,
        multi_action_cells: 5,
    };

    // When / Then
    assert_eq!(stats.state_count, 100);
    assert_eq!(stats.symbol_count, 50);
    assert_eq!(stats.rule_count, 30);
    assert_eq!(stats.conflict_count, 2);
    assert_eq!(stats.multi_action_cells, 5);
}

#[test]
fn given_table_statistics_when_serializing_then_roundtrips() {
    // Given
    let stats = TableStatistics {
        state_count: 200,
        symbol_count: 75,
        rule_count: 45,
        conflict_count: 0,
        multi_action_cells: 0,
    };

    // When
    let json = serde_json::to_string(&stats).unwrap();
    let parsed: TableStatistics = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(stats, parsed);
}

#[test]
fn given_table_statistics_with_no_conflicts_when_checking_then_conflict_count_is_zero() {
    // Given
    let stats = TableStatistics {
        state_count: 50,
        symbol_count: 25,
        rule_count: 15,
        conflict_count: 0,
        multi_action_cells: 0,
    };

    // When / Then
    assert_eq!(stats.conflict_count, 0);
    assert_eq!(stats.multi_action_cells, 0);
}

// =============================================================================
// GrammarInfo Tests
// =============================================================================

#[test]
fn given_grammar_info_when_creating_then_holds_values() {
    // Given
    let info = GrammarInfo {
        name: "json".to_string(),
        version: "1.0.0".to_string(),
        language: "json".to_string(),
    };

    // When / Then
    assert_eq!(info.name, "json");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.language, "json");
}

#[test]
fn given_grammar_info_when_serializing_then_roundtrips() {
    // Given
    let info = GrammarInfo {
        name: "python".to_string(),
        version: "2.0.0".to_string(),
        language: "python".to_string(),
    };

    // When
    let json = serde_json::to_string(&info).unwrap();
    let parsed: GrammarInfo = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(info, parsed);
}

// =============================================================================
// GenerationInfo Tests
// =============================================================================

#[test]
fn given_generation_info_when_creating_then_holds_values() {
    // Given
    let info = GenerationInfo {
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        tool_version: "0.1.0".to_string(),
        rust_version: "1.92.0".to_string(),
        host_triple: "x86_64-unknown-linux-gnu".to_string(),
    };

    // When / Then
    assert_eq!(info.timestamp, "2025-01-01T00:00:00Z");
    assert_eq!(info.tool_version, "0.1.0");
    assert_eq!(info.rust_version, "1.92.0");
    assert_eq!(info.host_triple, "x86_64-unknown-linux-gnu");
}

#[test]
fn given_generation_info_when_serializing_then_roundtrips() {
    // Given
    let info = GenerationInfo {
        timestamp: "2025-06-15T12:30:00Z".to_string(),
        tool_version: "0.2.0".to_string(),
        rust_version: "1.93.0".to_string(),
        host_triple: "aarch64-apple-darwin".to_string(),
    };

    // When
    let json = serde_json::to_string(&info).unwrap();
    let parsed: GenerationInfo = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(info, parsed);
}

// =============================================================================
// ParsetableMetadata Tests
// =============================================================================

#[test]
fn given_full_metadata_when_serializing_then_roundtrips() {
    // Given
    let metadata = ParsetableMetadata {
        schema_version: METADATA_SCHEMA_VERSION.to_string(),
        grammar: GrammarInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            language: "test".to_string(),
        },
        generation: GenerationInfo {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            tool_version: "0.1.0".to_string(),
            rust_version: "1.92.0".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
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

    // When
    let json = serde_json::to_string(&metadata).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();

    // Then
    assert_eq!(metadata, parsed);
}

#[test]
fn given_metadata_with_optional_fields_when_serializing_then_roundtrips() {
    // Given
    let metadata = ParsetableMetadata {
        schema_version: METADATA_SCHEMA_VERSION.to_string(),
        grammar: GrammarInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            language: "test".to_string(),
        },
        generation: GenerationInfo {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            tool_version: "0.1.0".to_string(),
            rust_version: "1.92.0".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
        },
        statistics: TableStatistics {
            state_count: 10,
            symbol_count: 5,
            rule_count: 3,
            conflict_count: 0,
            multi_action_cells: 0,
        },
        features: FeatureFlags {
            glr_enabled: true,
            external_scanner: true,
            incremental: true,
        },
        feature_profile: Some(ParserFeatureProfileSnapshot::new(false, false, true, false)),
        governance: Some(GovernanceMetadata::with_counts("core", 5, 8, "core:5/8")),
    };

    // When
    let json = serde_json::to_string(&metadata).unwrap();
    let parsed = ParsetableMetadata::parse_json(&json).unwrap();

    // Then
    assert_eq!(metadata, parsed);
}

#[test]
fn given_metadata_when_using_from_bytes_then_parses_correctly() {
    // Given
    let metadata = ParsetableMetadata {
        schema_version: "1.0".to_string(),
        grammar: GrammarInfo {
            name: "json".to_string(),
            version: "0.1.0".to_string(),
            language: "json".to_string(),
        },
        generation: GenerationInfo {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            tool_version: "0.1.0".to_string(),
            rust_version: "1.92.0".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
        },
        statistics: TableStatistics {
            state_count: 1,
            symbol_count: 1,
            rule_count: 1,
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

    // When
    let bytes = serde_json::to_vec(&metadata).unwrap();
    let parsed = ParsetableMetadata::from_bytes(&bytes).unwrap();

    // Then
    assert_eq!(metadata, parsed);
}

// =============================================================================
// ParsetableError Tests
// =============================================================================

#[test]
fn given_io_error_when_converting_to_parsetable_error_then_works() {
    // Given
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");

    // When
    let err: ParsetableError = io_err.into();

    // Then
    assert!(matches!(err, ParsetableError::Io(_)));
}

#[test]
fn given_serialization_error_when_displaying_then_shows_message() {
    // Given
    let err = ParsetableError::Serialization("bad json".to_string());

    // When
    let msg = format!("{}", err);

    // Then
    assert!(msg.contains("Serialization error"));
    assert!(msg.contains("bad json"));
}

#[test]
fn given_invalid_metadata_error_when_displaying_then_shows_message() {
    // Given
    let err = ParsetableError::InvalidMetadata("missing field".to_string());

    // When
    let msg = format!("{}", err);

    // Then
    assert!(msg.contains("Invalid metadata"));
    assert!(msg.contains("missing field"));
}

#[test]
fn given_hash_error_when_displaying_then_shows_message() {
    // Given
    let err = ParsetableError::HashError("sha256 failed".to_string());

    // When
    let msg = format!("{}", err);

    // Then
    assert!(msg.contains("Grammar hash computation failed"));
    assert!(msg.contains("sha256 failed"));
}

// =============================================================================
// Governance Types Re-export Tests
// =============================================================================

#[test]
fn given_parser_feature_profile_snapshot_when_creating_then_works() {
    // Given / When
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);

    // Then - just verify it can be created
    let _ = format!("{:?}", snap);
}

#[test]
fn given_governance_metadata_when_creating_default_then_works() {
    // Given / When
    let meta = GovernanceMetadata::default();

    // Then - just verify it can be created
    let _ = format!("{:?}", meta);
}

#[test]
fn given_governance_metadata_with_counts_when_creating_then_works() {
    // Given / When
    let meta = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");

    // Then - just verify it can be created
    let _ = format!("{:?}", meta);
}
