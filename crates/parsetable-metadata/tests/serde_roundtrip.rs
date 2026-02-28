//! Serde JSON roundtrip tests for all serializable types in parsetable-metadata.

use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};
use adze_parsetable_metadata::{
    FeatureFlags, GenerationInfo, GrammarInfo, ParsetableMetadata, TableStatistics,
};

fn sample_grammar_info() -> GrammarInfo {
    GrammarInfo {
        name: "test_grammar".to_string(),
        version: "1.2.3".to_string(),
        language: "rust".to_string(),
    }
}

fn sample_generation_info() -> GenerationInfo {
    GenerationInfo {
        timestamp: "2025-01-15T12:00:00Z".to_string(),
        tool_version: "0.1.0".to_string(),
        rust_version: "1.92.0".to_string(),
        host_triple: "x86_64-unknown-linux-gnu".to_string(),
    }
}

fn sample_table_statistics() -> TableStatistics {
    TableStatistics {
        state_count: 256,
        symbol_count: 42,
        rule_count: 100,
        conflict_count: 3,
        multi_action_cells: 7,
    }
}

fn sample_feature_flags() -> FeatureFlags {
    FeatureFlags {
        glr_enabled: true,
        external_scanner: false,
        incremental: true,
    }
}

fn sample_metadata_minimal() -> ParsetableMetadata {
    ParsetableMetadata {
        schema_version: "1.0".to_string(),
        grammar: sample_grammar_info(),
        generation: sample_generation_info(),
        statistics: sample_table_statistics(),
        features: sample_feature_flags(),
        feature_profile: None,
        governance: None,
    }
}

fn sample_metadata_full() -> ParsetableMetadata {
    ParsetableMetadata {
        schema_version: "1.0".to_string(),
        grammar: sample_grammar_info(),
        generation: sample_generation_info(),
        statistics: sample_table_statistics(),
        features: sample_feature_flags(),
        feature_profile: Some(ParserFeatureProfileSnapshot::new(true, false, true, true)),
        governance: Some(GovernanceMetadata::with_counts(
            "runtime",
            5,
            10,
            "runtime:5/10",
        )),
    }
}

// --- GrammarInfo ---

#[test]
fn roundtrip_grammar_info() {
    let original = sample_grammar_info();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GrammarInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_grammar_info_pretty() {
    let original = sample_grammar_info();
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: GrammarInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- GenerationInfo ---

#[test]
fn roundtrip_generation_info() {
    let original = sample_generation_info();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GenerationInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_generation_info_pretty() {
    let original = sample_generation_info();
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: GenerationInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- TableStatistics ---

#[test]
fn roundtrip_table_statistics() {
    let original = sample_table_statistics();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: TableStatistics = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_table_statistics_pretty() {
    let original = sample_table_statistics();
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: TableStatistics = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_table_statistics_zeros() {
    let original = TableStatistics {
        state_count: 0,
        symbol_count: 0,
        rule_count: 0,
        conflict_count: 0,
        multi_action_cells: 0,
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: TableStatistics = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- FeatureFlags ---

#[test]
fn roundtrip_feature_flags() {
    let original = sample_feature_flags();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: FeatureFlags = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_feature_flags_pretty() {
    let original = sample_feature_flags();
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: FeatureFlags = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_feature_flags_all_false() {
    let original = FeatureFlags {
        glr_enabled: false,
        external_scanner: false,
        incremental: false,
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: FeatureFlags = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_feature_flags_all_true() {
    let original = FeatureFlags {
        glr_enabled: true,
        external_scanner: true,
        incremental: true,
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: FeatureFlags = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- ParsetableMetadata (minimal, no optional fields) ---

#[test]
fn roundtrip_parsetable_metadata_minimal() {
    let original = sample_metadata_minimal();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: ParsetableMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_parsetable_metadata_minimal_pretty() {
    let original = sample_metadata_minimal();
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: ParsetableMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- ParsetableMetadata (full, all optional fields populated) ---

#[test]
fn roundtrip_parsetable_metadata_full() {
    let original = sample_metadata_full();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: ParsetableMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_parsetable_metadata_full_pretty() {
    let original = sample_metadata_full();
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: ParsetableMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- ParsetableMetadata::from_bytes / parse_json helpers ---

#[test]
fn roundtrip_parsetable_metadata_from_bytes() {
    let original = sample_metadata_full();
    let json = serde_json::to_vec(&original).expect("serialize to vec");
    let deserialized = ParsetableMetadata::from_bytes(&json).expect("from_bytes");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_parsetable_metadata_parse_json() {
    let original = sample_metadata_full();
    let json = serde_json::to_string_pretty(&original).expect("serialize");
    let deserialized = ParsetableMetadata::parse_json(&json).expect("parse_json");
    assert_eq!(original, deserialized);
}

// --- serde(default) behaviour: missing optional fields deserialize to None ---

#[test]
fn deserialize_metadata_missing_optional_fields() {
    let json = r#"{
        "schema_version": "1.0",
        "grammar": { "name": "g", "version": "0.1", "language": "test" },
        "generation": {
            "timestamp": "2025-01-01T00:00:00Z",
            "tool_version": "0.1.0",
            "rust_version": "1.92.0",
            "host_triple": "x86_64-unknown-linux-gnu"
        },
        "statistics": {
            "state_count": 1,
            "symbol_count": 1,
            "rule_count": 1,
            "conflict_count": 0,
            "multi_action_cells": 0
        },
        "features": {
            "glr_enabled": false,
            "external_scanner": false,
            "incremental": false
        }
    }"#;
    let meta: ParsetableMetadata = serde_json::from_str(json).expect("deserialize");
    assert_eq!(meta.feature_profile, None);
    assert_eq!(meta.governance, None);
}
