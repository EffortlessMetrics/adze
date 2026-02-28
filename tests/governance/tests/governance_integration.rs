//! Cross-crate governance pipeline integration tests.
//!
//! Validates the end-to-end flow:
//!   feature-policy-core → governance-metadata → parsetable-metadata
//!   bdd-grid-core → governance-runtime-core → governance-runtime-reporting

use adze_bdd_grid_core::{
    BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress, bdd_progress_report,
};
use adze_feature_policy_core::ParserFeatureProfile;
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};
use adze_governance_runtime_core::{
    bdd_governance_matrix_for_profile, bdd_progress_report_for_profile, resolve_backend_for_profile,
};
use adze_governance_runtime_reporting::bdd_progress_report_with_profile_runtime;
use adze_parsetable_metadata::{
    FeatureFlags, GenerationInfo, GrammarInfo, METADATA_SCHEMA_VERSION, ParsetableMetadata,
    TableStatistics,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn glr_profile() -> ParserFeatureProfile {
    ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    }
}

fn ts_profile() -> ParserFeatureProfile {
    ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    }
}

fn build_metadata(
    profile: ParserFeatureProfile,
    governance: GovernanceMetadata,
) -> ParsetableMetadata {
    ParsetableMetadata {
        schema_version: METADATA_SCHEMA_VERSION.to_string(),
        grammar: GrammarInfo {
            name: "test-grammar".to_string(),
            version: "0.1.0".to_string(),
            language: "test".to_string(),
        },
        generation: GenerationInfo {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            tool_version: "0.1.0".to_string(),
            rust_version: "1.92.0".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
        },
        statistics: TableStatistics {
            state_count: 42,
            symbol_count: 10,
            rule_count: 8,
            conflict_count: 2,
            multi_action_cells: 3,
        },
        features: FeatureFlags {
            glr_enabled: profile.glr,
            external_scanner: false,
            incremental: false,
        },
        feature_profile: Some(ParserFeatureProfileSnapshot::from_profile(profile)),
        governance: Some(governance),
    }
}

// ===================================================================
// 1. feature-policy-core: profile creation and backend resolution
// ===================================================================

#[test]
fn profile_creation_and_backend_resolution() {
    let profile = glr_profile();
    assert!(profile.has_pure_rust());
    assert!(profile.has_glr());
    assert!(!profile.has_tree_sitter());

    assert_eq!(
        profile.resolve_backend(false).name(),
        "pure-Rust GLR parser"
    );
    assert_eq!(profile.resolve_backend(true).name(), "pure-Rust GLR parser");
}

#[test]
fn tree_sitter_profile_backend_resolution() {
    let profile = ts_profile();
    assert!(!profile.has_pure_rust());
    assert!(!profile.has_glr());
    assert!(profile.has_tree_sitter());

    assert_eq!(
        profile.resolve_backend(false).name(),
        "tree-sitter C runtime"
    );
}

// ===================================================================
// 2. feature-policy-core → governance-metadata snapshot roundtrip
// ===================================================================

#[test]
fn profile_to_snapshot_roundtrip() {
    let profile = glr_profile();
    let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);

    assert_eq!(snapshot.pure_rust, profile.pure_rust);
    assert_eq!(snapshot.tree_sitter_standard, profile.tree_sitter_standard);
    assert_eq!(snapshot.tree_sitter_c2rust, profile.tree_sitter_c2rust);
    assert_eq!(snapshot.glr, profile.glr);

    let restored = snapshot.as_profile();
    assert_eq!(restored, profile);
}

#[test]
fn snapshot_serialization_roundtrip() {
    let snapshot = ParserFeatureProfileSnapshot::from_profile(glr_profile());
    let json = serde_json::to_string(&snapshot).expect("serialize snapshot");
    let restored: ParserFeatureProfileSnapshot =
        serde_json::from_str(&json).expect("deserialize snapshot");
    assert_eq!(snapshot, restored);
}

// ===================================================================
// 3. governance-metadata: GovernanceMetadata from BDD grid
// ===================================================================

#[test]
fn governance_metadata_from_grid() {
    let profile = glr_profile();
    let gov = GovernanceMetadata::for_grid(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    let (expected_impl, expected_total) =
        bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    assert_eq!(gov.implemented, expected_impl);
    assert_eq!(gov.total, expected_total);
    assert_eq!(gov.phase, "core");
    assert!(gov.status_line.contains("core:"));
}

#[test]
fn governance_metadata_serialization_roundtrip() {
    let gov = GovernanceMetadata::for_grid(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        glr_profile(),
    );
    let json = serde_json::to_string(&gov).expect("serialize governance");
    let restored: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize governance");
    assert_eq!(gov, restored);
}

// ===================================================================
// 4. parsetable-metadata: full metadata embedding + serde roundtrip
// ===================================================================

#[test]
fn parsetable_metadata_embeds_governance_and_profile() {
    let profile = glr_profile();
    let gov = GovernanceMetadata::for_grid(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    let meta = build_metadata(profile, gov.clone());

    assert_eq!(meta.feature_profile.as_ref().unwrap().glr, profile.glr);
    assert_eq!(
        meta.governance.as_ref().unwrap().implemented,
        gov.implemented
    );
}

#[test]
fn parsetable_metadata_json_roundtrip() {
    let profile = glr_profile();
    let gov =
        GovernanceMetadata::for_grid(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
    let meta = build_metadata(profile, gov);

    let json = serde_json::to_string_pretty(&meta).expect("serialize metadata");
    let restored = ParsetableMetadata::parse_json(&json).expect("parse_json");
    assert_eq!(meta, restored);
}

#[test]
fn parsetable_metadata_bytes_roundtrip() {
    let profile = glr_profile();
    let gov = GovernanceMetadata::for_grid(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    let meta = build_metadata(profile, gov);

    let bytes = serde_json::to_vec(&meta).expect("serialize to bytes");
    let restored = ParsetableMetadata::from_bytes(&bytes).expect("from_bytes");
    assert_eq!(meta, restored);
}

// ===================================================================
// 5. bdd-grid-core: BDD progress tracking across phases
// ===================================================================

#[test]
fn bdd_grid_progress_core_phase() {
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    assert_eq!(total, 8);
    assert_eq!(implemented, 6); // scenarios 7 & 8 deferred for core
}

#[test]
fn bdd_grid_progress_runtime_phase() {
    let (implemented, total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);
    assert_eq!(total, 8);
    assert_eq!(implemented, 8); // all runtime scenarios implemented
}

#[test]
fn bdd_scenario_status_per_phase() {
    let scenario7 = GLR_CONFLICT_PRESERVATION_GRID[6]; // id=7
    assert!(!scenario7.status(BddPhase::Core).implemented());
    assert!(scenario7.status(BddPhase::Runtime).implemented());

    let scenario1 = GLR_CONFLICT_PRESERVATION_GRID[0]; // id=1
    assert!(scenario1.status(BddPhase::Core).implemented());
    assert!(scenario1.status(BddPhase::Runtime).implemented());
}

#[test]
fn bdd_deferred_scenario_carries_reason() {
    let scenario7 = GLR_CONFLICT_PRESERVATION_GRID[6];
    match scenario7.status(BddPhase::Core) {
        BddScenarioStatus::Deferred { reason } => {
            assert!(!reason.is_empty(), "deferred reason must not be empty");
        }
        BddScenarioStatus::Implemented => panic!("scenario 7 core should be deferred"),
    }
}

#[test]
fn bdd_progress_report_contains_all_scenarios() {
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Core Phase");
    for scenario in GLR_CONFLICT_PRESERVATION_GRID {
        assert!(
            report.contains(scenario.title),
            "report missing scenario: {}",
            scenario.title
        );
    }
}

// ===================================================================
// 6. governance-runtime-core: matrix + backend resolution
// ===================================================================

#[test]
fn governance_matrix_snapshot_counts_match_grid() {
    let profile = glr_profile();
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
    let snap = matrix.snapshot();

    let (expected_impl, expected_total) =
        bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    assert_eq!(snap.implemented, expected_impl);
    assert_eq!(snap.total, expected_total);
    assert_eq!(snap.phase, BddPhase::Core);
    assert_eq!(snap.profile, profile);
}

#[test]
fn governance_matrix_report_includes_phase_title() {
    let profile = glr_profile();
    let report = bdd_progress_report_for_profile(BddPhase::Runtime, "Runtime Phase", profile);
    assert!(report.contains("Runtime Phase"));
}

#[test]
fn resolve_backend_through_governance_runtime_core() {
    assert_eq!(
        resolve_backend_for_profile(glr_profile(), true).name(),
        "pure-Rust GLR parser"
    );
    assert_eq!(
        resolve_backend_for_profile(ts_profile(), false).name(),
        "tree-sitter C runtime"
    );
}

// ===================================================================
// 7. governance-runtime-reporting: runtime report formatting
// ===================================================================

#[test]
fn runtime_reporting_includes_governance_sections() {
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Runtime",
        glr_profile(),
    );

    assert!(report.contains("Governance status:"));
    assert!(report.contains("Feature profile:"));
    assert!(report.contains("Non-conflict backend:"));
    assert!(report.contains("Conflict profiles:"));
}

// ===================================================================
// 8. End-to-end: profile → snapshot → metadata → serialize → verify
// ===================================================================

#[test]
fn end_to_end_glr_governance_pipeline() {
    // Step 1: Create a profile (feature-policy-core)
    let profile = glr_profile();

    // Step 2: Convert to snapshot (governance-metadata)
    let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);
    assert_eq!(snapshot.as_profile(), profile);

    // Step 3: Build governance metadata from BDD grid
    let gov =
        GovernanceMetadata::for_grid(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert!(gov.status_line.contains("runtime:"));
    assert!(gov.status_line.contains("GLR"));

    // Step 4: Embed in parsetable metadata
    let meta = build_metadata(profile, gov.clone());

    // Step 5: Serialize → deserialize (simulates artifact persistence)
    let json = serde_json::to_string(&meta).expect("serialize");
    let restored = ParsetableMetadata::parse_json(&json).expect("deserialize");

    // Step 6: Verify the full pipeline survived serialization
    let restored_snapshot = restored.feature_profile.expect("profile present");
    assert_eq!(restored_snapshot.as_profile(), profile);

    let restored_gov = restored.governance.expect("governance present");
    assert_eq!(restored_gov.implemented, gov.implemented);
    assert_eq!(restored_gov.total, gov.total);
    assert_eq!(restored_gov.phase, gov.phase);

    // Step 7: Verify runtime governance matrix agrees
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Runtime, profile);
    let snap = matrix.snapshot();
    assert_eq!(snap.implemented, restored_gov.implemented);
    assert_eq!(snap.total, restored_gov.total);

    // Step 8: Verify reporting produces valid output
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "E2E Test",
        profile,
    );
    assert!(report.contains("E2E Test"));
    assert!(report.contains("Governance status:"));
}

#[test]
fn end_to_end_tree_sitter_profile_pipeline() {
    let profile = ts_profile();

    let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);
    assert_eq!(snapshot.non_conflict_backend(), "tree-sitter C runtime");

    let gov = GovernanceMetadata::for_grid(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    let meta = build_metadata(profile, gov);
    let bytes = serde_json::to_vec(&meta).expect("to bytes");
    let restored = ParsetableMetadata::from_bytes(&bytes).expect("from bytes");

    assert_eq!(
        restored
            .feature_profile
            .unwrap()
            .resolve_non_conflict_backend()
            .name(),
        "tree-sitter C runtime"
    );
    // Core phase has deferred scenarios → not complete
    assert!(!restored.governance.unwrap().is_complete());
}

// ===================================================================
// 9. Snapshot backend resolution matches profile resolution
// ===================================================================

#[test]
fn snapshot_backend_resolution_matches_profile() {
    let profile = glr_profile();
    let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);

    assert_eq!(
        snapshot.resolve_non_conflict_backend(),
        profile.resolve_backend(false)
    );
    assert_eq!(
        snapshot.resolve_conflict_backend(),
        profile.resolve_backend(true)
    );
}

// ===================================================================
// 10. GovernanceMetadata with_counts constructor + completeness
// ===================================================================

#[test]
fn governance_metadata_with_counts_and_completeness() {
    let complete = GovernanceMetadata::with_counts("runtime", 8, 8, "runtime:8/8:GLR");
    assert!(complete.is_complete());

    let incomplete = GovernanceMetadata::with_counts("core", 6, 8, "core:6/8:GLR");
    assert!(!incomplete.is_complete());
}

// ===================================================================
// 11. BDD governance matrix fully-implemented check
// ===================================================================

#[test]
fn governance_matrix_fully_implemented_check() {
    let profile = glr_profile();

    let runtime_matrix = bdd_governance_matrix_for_profile(BddPhase::Runtime, profile);
    assert!(runtime_matrix.is_fully_implemented());

    let core_matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
    assert!(!core_matrix.is_fully_implemented());
}

// ===================================================================
// 12. Metadata without governance (optional fields)
// ===================================================================

#[test]
fn parsetable_metadata_without_governance_fields() {
    let meta = ParsetableMetadata {
        schema_version: METADATA_SCHEMA_VERSION.to_string(),
        grammar: GrammarInfo {
            name: "bare".to_string(),
            version: "0.1.0".to_string(),
            language: "test".to_string(),
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

    let json = serde_json::to_string(&meta).expect("serialize");
    let restored = ParsetableMetadata::parse_json(&json).expect("parse");
    assert!(restored.feature_profile.is_none());
    assert!(restored.governance.is_none());
}

// ===================================================================
// 13. Cross-phase governance metadata consistency
// ===================================================================

#[test]
fn governance_metadata_consistent_across_phases() {
    let profile = glr_profile();

    let core_gov =
        GovernanceMetadata::for_grid(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    let runtime_gov =
        GovernanceMetadata::for_grid(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Same grid → same total
    assert_eq!(core_gov.total, runtime_gov.total);
    // Runtime has more scenarios implemented than core
    assert!(runtime_gov.implemented >= core_gov.implemented);
    // Phases are distinct
    assert_ne!(core_gov.phase, runtime_gov.phase);
}
