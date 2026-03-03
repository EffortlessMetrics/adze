//! Integration tests for the governance-runtime-reporting crate.

use adze_governance_runtime_reporting::*;

#[test]
fn runtime_report_contains_governance_status() {
    let profile = ParserFeatureProfile::current();
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Runtime Phase",
        profile,
    );
    assert!(report.contains("Governance status:"));
    assert!(report.contains("Runtime Phase"));
}

#[test]
fn runtime_report_contains_feature_profile() {
    let profile = ParserFeatureProfile::current();
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Core",
        profile,
    );
    assert!(report.contains("Feature profile:"));
}

#[test]
fn runtime_report_contains_backend_info() {
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "GLR Report",
        profile,
    );
    assert!(report.contains("Non-conflict backend:"));
    assert!(report.contains("Conflict profiles:"));
}

#[test]
fn runtime_report_with_empty_scenarios() {
    let profile = ParserFeatureProfile::current();
    let report = bdd_progress_report_with_profile_runtime(BddPhase::Core, &[], "Empty", profile);
    assert!(report.contains("Empty"));
    assert!(report.contains("Governance status: 0/0"));
}

#[test]
fn status_line_core_phase_prefix() {
    let profile = ParserFeatureProfile::current();
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert!(status.starts_with("core:"));
}

#[test]
fn status_line_runtime_phase_prefix() {
    let profile = ParserFeatureProfile::current();
    let status =
        bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert!(status.starts_with("runtime:"));
}

#[test]
fn bdd_progress_counts_are_consistent() {
    let (core_impl, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (rt_impl, rt_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);
    assert_eq!(core_total, rt_total);
    assert!(core_impl <= core_total);
    assert!(rt_impl <= rt_total);
}

#[test]
fn describe_backend_for_conflicts_nonempty() {
    let desc = describe_backend_for_conflicts(ParserFeatureProfile::current());
    assert!(!desc.is_empty());
}
