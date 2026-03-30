//! Property-based tests for parsetable-metadata.

use proptest::prelude::*;

use adze_parsetable_metadata::{
    FeatureFlags, GenerationInfo, GrammarInfo, ParsetableMetadata, TableStatistics,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary FeatureFlags values.
fn arb_feature_flags() -> impl Strategy<Value = FeatureFlags> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(glr_enabled, external_scanner, incremental)| FeatureFlags {
            glr_enabled,
            external_scanner,
            incremental,
        },
    )
}

/// Generate arbitrary TableStatistics values.
fn arb_table_statistics() -> impl Strategy<Value = TableStatistics> {
    (
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
    )
        .prop_map(
            |(state_count, symbol_count, rule_count, conflict_count, multi_action_cells)| {
                TableStatistics {
                    state_count,
                    symbol_count,
                    rule_count,
                    conflict_count,
                    multi_action_cells,
                }
            },
        )
}

/// Generate arbitrary non-empty strings.
fn arb_non_empty_string() -> impl Strategy<Value = String> {
    ".{1,50}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate arbitrary GrammarInfo values.
fn arb_grammar_info() -> impl Strategy<Value = GrammarInfo> {
    (
        arb_non_empty_string(),
        arb_non_empty_string(),
        arb_non_empty_string(),
    )
        .prop_map(|(name, version, language)| GrammarInfo {
            name,
            version,
            language,
        })
}

/// Generate arbitrary GenerationInfo values.
fn arb_generation_info() -> impl Strategy<Value = GenerationInfo> {
    (
        arb_non_empty_string(),
        arb_non_empty_string(),
        arb_non_empty_string(),
        arb_non_empty_string(),
    )
        .prop_map(
            |(timestamp, tool_version, rust_version, host_triple)| GenerationInfo {
                timestamp,
                tool_version,
                rust_version,
                host_triple,
            },
        )
}

/// Generate arbitrary ParsetableMetadata values.
fn arb_metadata() -> impl Strategy<Value = ParsetableMetadata> {
    (
        arb_grammar_info(),
        arb_generation_info(),
        arb_table_statistics(),
        arb_feature_flags(),
    )
        .prop_map(
            |(grammar, generation, statistics, features)| ParsetableMetadata {
                schema_version: "1.0".to_string(),
                grammar,
                generation,
                statistics,
                features,
                feature_profile: None,
                governance: None,
            },
        )
}

// ---------------------------------------------------------------------------
// 1 – FeatureFlags tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn feature_flags_copy_preserves_fields(flags in arb_feature_flags()) {
        let flags2 = flags;
        prop_assert_eq!(flags.glr_enabled, flags2.glr_enabled);
        prop_assert_eq!(flags.external_scanner, flags2.external_scanner);
        prop_assert_eq!(flags.incremental, flags2.incremental);
    }

    #[test]
    fn feature_flags_eq_reflexive(flags in arb_feature_flags()) {
        prop_assert!(flags == flags);
    }

    #[test]
    fn feature_flags_serde_roundtrip(flags in arb_feature_flags()) {
        let json = serde_json::to_string(&flags).unwrap();
        let parsed: FeatureFlags = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(flags, parsed);
    }
}

// ---------------------------------------------------------------------------
// 2 – TableStatistics tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn table_statistics_copy_preserves_fields(stats in arb_table_statistics()) {
        let stats2 = stats;
        prop_assert_eq!(stats.state_count, stats2.state_count);
        prop_assert_eq!(stats.symbol_count, stats2.symbol_count);
        prop_assert_eq!(stats.rule_count, stats2.rule_count);
        prop_assert_eq!(stats.conflict_count, stats2.conflict_count);
        prop_assert_eq!(stats.multi_action_cells, stats2.multi_action_cells);
    }

    #[test]
    fn table_statistics_eq_reflexive(stats in arb_table_statistics()) {
        prop_assert!(stats == stats);
    }

    #[test]
    fn table_statistics_serde_roundtrip(stats in arb_table_statistics()) {
        let json = serde_json::to_string(&stats).unwrap();
        let parsed: TableStatistics = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(stats, parsed);
    }
}

// ---------------------------------------------------------------------------
// 3 – GrammarInfo tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn grammar_info_eq_reflexive(info in arb_grammar_info()) {
        prop_assert!(info == info);
    }

    #[test]
    fn grammar_info_clone_equals_original(info in arb_grammar_info()) {
        let cloned = info.clone();
        prop_assert_eq!(info, cloned);
    }

    #[test]
    fn grammar_info_serde_roundtrip(info in arb_grammar_info()) {
        let json = serde_json::to_string(&info).unwrap();
        let parsed: GrammarInfo = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(info, parsed);
    }
}

// ---------------------------------------------------------------------------
// 4 – GenerationInfo tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn generation_info_eq_reflexive(info in arb_generation_info()) {
        prop_assert!(info == info);
    }

    #[test]
    fn generation_info_clone_equals_original(info in arb_generation_info()) {
        let cloned = info.clone();
        prop_assert_eq!(info, cloned);
    }

    #[test]
    fn generation_info_serde_roundtrip(info in arb_generation_info()) {
        let json = serde_json::to_string(&info).unwrap();
        let parsed: GenerationInfo = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(info, parsed);
    }
}

// ---------------------------------------------------------------------------
// 5 – ParsetableMetadata tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_eq_reflexive(metadata in arb_metadata()) {
        prop_assert!(metadata == metadata);
    }

    #[test]
    fn metadata_clone_equals_original(metadata in arb_metadata()) {
        let cloned = metadata.clone();
        prop_assert_eq!(metadata, cloned);
    }

    #[test]
    fn metadata_serde_roundtrip(metadata in arb_metadata()) {
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed = ParsetableMetadata::parse_json(&json).unwrap();
        prop_assert_eq!(metadata, parsed);
    }

    #[test]
    fn metadata_from_bytes_roundtrip(metadata in arb_metadata()) {
        let bytes = serde_json::to_vec(&metadata).unwrap();
        let parsed = ParsetableMetadata::from_bytes(&bytes).unwrap();
        prop_assert_eq!(metadata, parsed);
    }
}

// ---------------------------------------------------------------------------
// 6 – Hash consistency tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn feature_flags_hash_consistent(flags in arb_feature_flags()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        flags.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        flags.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }

    #[test]
    fn table_statistics_hash_consistent(stats in arb_table_statistics()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        stats.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        stats.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }
}

// ---------------------------------------------------------------------------
// 7 – JSON format tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_json_is_valid_object(metadata in arb_metadata()) {
        let json = serde_json::to_string(&metadata).unwrap();
        // Should be a JSON object (starts with '{')
        let starts = json.starts_with('{');
        let ends = json.ends_with('}');
        prop_assert!(starts && ends);
    }

    #[test]
    fn feature_flags_json_is_valid_object(flags in arb_feature_flags()) {
        let json = serde_json::to_string(&flags).unwrap();
        let starts = json.starts_with('{');
        let ends = json.ends_with('}');
        prop_assert!(starts && ends);
    }
}

// ---------------------------------------------------------------------------
// 8 – Debug format tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn feature_flags_debug_contains_fields(flags in arb_feature_flags()) {
        let debug = format!("{:?}", flags);
        let contains_field = debug.contains("glr_enabled") || debug.contains("FeatureFlags");
        prop_assert!(contains_field);
    }

    #[test]
    fn table_statistics_debug_contains_fields(stats in arb_table_statistics()) {
        let debug = format!("{:?}", stats);
        let contains_field = debug.contains("state_count") || debug.contains("TableStatistics");
        prop_assert!(contains_field);
    }
}
