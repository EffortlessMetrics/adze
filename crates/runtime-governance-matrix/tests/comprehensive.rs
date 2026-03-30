// Comprehensive tests for runtime-governance-matrix
use adze_runtime_governance_matrix::*;

#[test]
fn current_backend_no_conflicts() {
    let b = current_backend_for(false);
    assert!(!b.name().is_empty());
}

#[test]
fn current_backend_with_conflicts() {
    let profile = parser_feature_profile_for_runtime();

    if profile.has_glr() {
        let b = current_backend_for(true);
        assert!(!b.name().is_empty());
    }
}

#[test]
fn governance_matrix_current_core() {
    let m = bdd_governance_matrix_for_current_profile(BddPhase::Core);
    assert_eq!(m.phase, BddPhase::Core);
}

#[test]
fn governance_matrix_current_runtime() {
    let m = bdd_governance_matrix_for_current_profile(BddPhase::Runtime);
    assert_eq!(m.phase, BddPhase::Runtime);
}

#[test]
fn status_line_current_core() {
    let line = bdd_status_line_for_current_profile(BddPhase::Core);
    assert!(line.starts_with("core:"));
}

#[test]
fn status_line_current_runtime() {
    let line = bdd_status_line_for_current_profile(BddPhase::Runtime);
    assert!(line.starts_with("runtime:"));
}

#[test]
fn runtime_snapshot_core() {
    let snap = runtime_governance_snapshot(BddPhase::Core);
    assert_eq!(snap.phase, BddPhase::Core);
}

#[test]
fn runtime_snapshot_runtime() {
    let snap = runtime_governance_snapshot(BddPhase::Runtime);
    assert_eq!(snap.phase, BddPhase::Runtime);
}

#[test]
fn progress_report_current_core() {
    let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core");
    assert!(!report.is_empty());
}

#[test]
fn resolve_runtime2_backend_no_glr_no_conflicts() {
    let b = resolve_runtime2_backend(false, false);
    assert!(!b.name().is_empty());
}

#[test]
fn resolve_runtime2_backend_glr_with_conflicts() {
    let b = resolve_runtime2_backend(true, true);
    assert!(!b.name().is_empty());
}

#[test]
fn governance_matrix_for_runtime() {
    let m = bdd_governance_matrix_for_runtime();
    assert!(!m.status_line().is_empty());
}

#[test]
fn governance_matrix_for_runtime2_core() {
    let m = bdd_governance_matrix_for_runtime2(BddPhase::Core, true);
    assert_eq!(m.phase, BddPhase::Core);
}

#[test]
fn governance_matrix_for_runtime2_runtime() {
    let m = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, false);
    assert_eq!(m.phase, BddPhase::Runtime);
}

#[test]
fn runtime2_snapshot_core_glr() {
    let p = parser_feature_profile_for_runtime2(true);
    let snap = runtime2_governance_snapshot(BddPhase::Core, p);
    assert_eq!(snap.phase, BddPhase::Core);
}

#[test]
fn runtime2_snapshot_runtime_no_glr() {
    let p = parser_feature_profile_for_runtime2(false);
    let snap = runtime2_governance_snapshot(BddPhase::Runtime, p);
    assert_eq!(snap.phase, BddPhase::Runtime);
}

#[test]
fn progress_report_for_runtime2_profile_core() {
    let p = parser_feature_profile_for_runtime2(true);
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Core, "Core", p);
    assert!(!report.is_empty());
}

#[test]
fn progress_status_line_for_runtime2_profile() {
    let p = parser_feature_profile_for_runtime2(true);
    let line = bdd_progress_status_line_for_runtime2_profile(BddPhase::Core, p);
    assert!(line.starts_with("core:"));
}

#[test]
fn resolve_backend_for_runtime2_profile_no_conflicts() {
    let p = parser_feature_profile_for_runtime2(false);
    let b = resolve_backend_for_runtime2_profile(p, false);
    assert!(!b.name().is_empty());
}

#[test]
fn snapshot_consistency() {
    let snap = runtime_governance_snapshot(BddPhase::Core);
    assert!(snap.implemented <= snap.total);
}
