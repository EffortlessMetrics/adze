// Comprehensive tests for governance-runtime-core
use adze_governance_runtime_core::*;

#[test]
fn runtime_profile() {
    let p = parser_feature_profile_for_runtime();
    let _ = format!("{:?}", p);
}

#[test]
fn runtime2_profile_glr_false() {
    let p = parser_feature_profile_for_runtime2(false);
    assert!(!p.glr);
}

#[test]
fn runtime2_profile_glr_true() {
    let p = parser_feature_profile_for_runtime2(true);
    assert!(p.glr);
}

#[test]
fn resolve_backend_no_conflicts() {
    let p = parser_feature_profile_for_runtime();
    let b = resolve_backend_for_profile(p, false);
    let _ = format!("{:?}", b);
}

#[test]
fn resolve_backend_with_conflicts() {
    let p = parser_feature_profile_for_runtime2(true);
    let b = resolve_backend_for_profile(p, true);
    let _ = format!("{:?}", b);
}

#[test]
fn governance_matrix_for_runtime() {
    let m = bdd_governance_matrix_for_runtime();
    let _ = format!("{:?}", m);
}

#[test]
fn governance_matrix_for_runtime2_glr() {
    let m = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, true);
    let _ = format!("{:?}", m);
}

#[test]
fn governance_matrix_for_runtime2_no_glr() {
    let m = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, false);
    let _ = format!("{:?}", m);
}

#[test]
fn progress_status_line() {
    let p = parser_feature_profile_for_runtime();
    let line = bdd_progress_status_line_for_profile(BddPhase::Runtime, p);
    assert!(!line.is_empty());
}

#[test]
fn progress_report() {
    let p = parser_feature_profile_for_runtime();
    let report = bdd_progress_report_for_profile(BddPhase::Runtime, "runtime", p);
    assert!(!report.is_empty());
}

#[test]
fn progress_status_line_core_phase() {
    let p = parser_feature_profile_for_runtime2(true);
    let line = bdd_progress_status_line_for_profile(BddPhase::Core, p);
    assert!(!line.is_empty());
}
