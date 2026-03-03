//! Integration tests for the governance-matrix-core-impl crate.

use adze_governance_matrix_core_impl::*;

#[test]
fn snapshot_is_fully_implemented_when_counts_match() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };
    assert!(snap.is_fully_implemented());
}

#[test]
fn snapshot_not_fully_implemented_when_partial() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Runtime,
        implemented: 2,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };
    assert!(!snap.is_fully_implemented());
}

#[test]
fn snapshot_non_conflict_backend_resolves() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 0,
        total: 0,
        profile: ParserFeatureProfile::current(),
    };
    let backend = snap.non_conflict_backend();
    assert_eq!(
        backend,
        ParserFeatureProfile::current().resolve_backend(false)
    );
}

#[test]
fn matrix_standard_uses_core_phase() {
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn matrix_report_includes_title() {
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    let report = matrix.report("Integration Test Title");
    assert!(report.contains("Integration Test Title"));
}

#[test]
fn matrix_status_line_starts_with_phase() {
    let profile = ParserFeatureProfile::current();
    let matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);
    let status = matrix.status_line();
    assert!(status.starts_with("runtime:"));
}

#[test]
fn describe_backend_for_conflicts_returns_nonempty() {
    let profile = ParserFeatureProfile::current();
    let desc = describe_backend_for_conflicts(profile);
    assert!(!desc.is_empty());
}

#[test]
fn bdd_governance_snapshot_from_grid_has_correct_totals() {
    let profile = ParserFeatureProfile::current();
    let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert_eq!(snap.phase, BddPhase::Core);
    assert_eq!(snap.total, GLR_CONFLICT_PRESERVATION_GRID.len());
    assert!(snap.implemented <= snap.total);
    assert_eq!(snap.profile, profile);
}
