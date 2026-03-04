// Comprehensive tests for bdd-governance-core
use adze_bdd_governance_core::*;

#[test]
fn snapshot_fully_implemented() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 6,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };
    assert!(snap.is_fully_implemented());
}

#[test]
fn snapshot_not_fully_implemented() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Runtime,
        implemented: 3,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };
    assert!(!snap.is_fully_implemented());
}

#[test]
fn snapshot_non_conflict_backend() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 6,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };
    let backend = snap.non_conflict_backend();
    assert!(!backend.name().is_empty());
}

#[test]
fn matrix_standard_is_core_phase() {
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn matrix_report_contains_summary_header() {
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    let report = matrix.report("Core");
    assert!(report.contains("BDD GLR Conflict Preservation Test Summary"));
}

#[test]
fn matrix_status_line_starts_with_phase() {
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    let line = matrix.status_line();
    assert!(line.starts_with("core:"));
}

#[test]
fn matrix_snapshot_matches_direct() {
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);
    let snap = matrix.snapshot();
    let direct = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert_eq!(snap.implemented, direct.implemented);
    assert_eq!(snap.total, direct.total);
}

#[test]
fn describe_backend_for_conflicts_non_empty() {
    let desc = describe_backend_for_conflicts(ParserFeatureProfile::current());
    assert!(!desc.is_empty());
}

#[test]
fn glr_conflict_fallback_non_empty() {
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

#[test]
fn bdd_progress_status_line_contains_phase() {
    let profile = ParserFeatureProfile::current();
    let line = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert!(line.starts_with("core:"));
}

#[test]
fn bdd_progress_status_line_runtime() {
    let profile = ParserFeatureProfile::current();
    let line = bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert!(line.starts_with("runtime:"));
}

#[test]
fn matrix_new_custom_scenarios() {
    let scenarios = &GLR_CONFLICT_PRESERVATION_GRID[..2];
    let matrix = BddGovernanceMatrix::new(
        BddPhase::Runtime,
        ParserFeatureProfile::current(),
        scenarios,
    );
    assert_eq!(matrix.phase, BddPhase::Runtime);
    assert_eq!(matrix.scenarios.len(), 2);
}

#[test]
fn snapshot_debug_format() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 3,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };
    let d = format!("{:?}", snap);
    assert!(d.contains("BddGovernanceSnapshot"));
}

#[test]
fn matrix_is_fully_implemented_delegates() {
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    let snap = matrix.snapshot();
    assert_eq!(matrix.is_fully_implemented(), snap.is_fully_implemented());
}

#[test]
fn bdd_progress_report_with_profile_annotated() {
    let profile = ParserFeatureProfile::current();
    let report = bdd_progress_report_with_profile(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Core",
        profile,
    );
    assert!(report.contains("Feature profile:"));
    assert!(report.contains("Non-conflict backend:"));
    assert!(report.contains("Governance progress:"));
}
