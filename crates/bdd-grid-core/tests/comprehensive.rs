// Comprehensive tests for BDD grid core
use adze_bdd_grid_core::*;

// ---------------------------------------------------------------------------
// BddPhase
// ---------------------------------------------------------------------------

#[test]
fn phase_core() {
    assert_eq!(BddPhase::Core, BddPhase::Core);
}

#[test]
fn phase_runtime() {
    assert_eq!(BddPhase::Runtime, BddPhase::Runtime);
}

#[test]
fn phase_ne() {
    assert_ne!(BddPhase::Core, BddPhase::Runtime);
}

#[test]
fn phase_debug() {
    let d = format!("{:?}", BddPhase::Core);
    assert!(d.contains("Core"));
}

#[test]
fn phase_display() {
    let d = format!("{}", BddPhase::Core);
    assert!(d.contains("Core"));
    let r = format!("{}", BddPhase::Runtime);
    assert!(r.contains("Runtime"));
}

#[test]
fn phase_clone() {
    let p = BddPhase::Core;
    let p2 = p;
    assert_eq!(p, p2);
}

// ---------------------------------------------------------------------------
// BddScenarioStatus
// ---------------------------------------------------------------------------

#[test]
fn status_implemented() {
    let s = BddScenarioStatus::Implemented;
    assert!(s.implemented());
}

#[test]
fn status_deferred() {
    let s = BddScenarioStatus::Deferred { reason: "wip" };
    assert!(!s.implemented());
}

#[test]
fn status_icon_implemented() {
    let s = BddScenarioStatus::Implemented;
    let icon = s.icon();
    assert!(!icon.is_empty());
}

#[test]
fn status_icon_deferred() {
    let s = BddScenarioStatus::Deferred { reason: "later" };
    let icon = s.icon();
    assert!(!icon.is_empty());
}

#[test]
fn status_label_implemented() {
    let s = BddScenarioStatus::Implemented;
    assert_eq!(s.label(), "IMPLEMENTED");
}

#[test]
fn status_detail_implemented() {
    let s = BddScenarioStatus::Implemented;
    // Detail for implemented should be empty or a fixed string
    let _ = s.detail();
}

#[test]
fn status_detail_deferred() {
    let s = BddScenarioStatus::Deferred { reason: "wip" };
    assert_eq!(s.detail(), "wip");
}

#[test]
fn status_display_implemented() {
    let s = format!("{}", BddScenarioStatus::Implemented);
    assert!(s.contains("Implemented"));
}

#[test]
fn status_display_deferred() {
    let s = format!("{}", BddScenarioStatus::Deferred { reason: "wip" });
    assert!(s.contains("Deferred"));
    assert!(s.contains("wip"));
}

#[test]
fn status_eq() {
    assert_eq!(
        BddScenarioStatus::Implemented,
        BddScenarioStatus::Implemented
    );
}

#[test]
fn status_ne() {
    assert_ne!(
        BddScenarioStatus::Implemented,
        BddScenarioStatus::Deferred { reason: "x" }
    );
}

// ---------------------------------------------------------------------------
// BddScenario
// ---------------------------------------------------------------------------

fn make_scenario(core: BddScenarioStatus, runtime: BddScenarioStatus) -> BddScenario {
    BddScenario {
        id: 1,
        title: "test scenario",
        reference: "TEST-001",
        core_status: core,
        runtime_status: runtime,
    }
}

#[test]
fn scenario_core_status() {
    let s = make_scenario(
        BddScenarioStatus::Implemented,
        BddScenarioStatus::Deferred { reason: "wip" },
    );
    assert!(s.status(BddPhase::Core).implemented());
    assert!(!s.status(BddPhase::Runtime).implemented());
}

#[test]
fn scenario_runtime_status() {
    let s = make_scenario(
        BddScenarioStatus::Deferred { reason: "later" },
        BddScenarioStatus::Implemented,
    );
    assert!(!s.status(BddPhase::Core).implemented());
    assert!(s.status(BddPhase::Runtime).implemented());
}

#[test]
fn scenario_debug() {
    let s = make_scenario(
        BddScenarioStatus::Implemented,
        BddScenarioStatus::Implemented,
    );
    let d = format!("{:?}", s);
    assert!(d.contains("BddScenario"));
}

// ---------------------------------------------------------------------------
// bdd_progress
// ---------------------------------------------------------------------------

#[test]
fn progress_all_implemented() {
    let scenarios = vec![
        make_scenario(
            BddScenarioStatus::Implemented,
            BddScenarioStatus::Implemented,
        ),
        make_scenario(
            BddScenarioStatus::Implemented,
            BddScenarioStatus::Implemented,
        ),
    ];
    let (done, total) = bdd_progress(BddPhase::Core, &scenarios);
    assert_eq!(done, 2);
    assert_eq!(total, 2);
}

#[test]
fn progress_none_implemented() {
    let scenarios = vec![make_scenario(
        BddScenarioStatus::Deferred { reason: "wip" },
        BddScenarioStatus::Deferred { reason: "wip" },
    )];
    let (done, total) = bdd_progress(BddPhase::Core, &scenarios);
    assert_eq!(done, 0);
    assert_eq!(total, 1);
}

#[test]
fn progress_mixed() {
    let scenarios = vec![
        make_scenario(
            BddScenarioStatus::Implemented,
            BddScenarioStatus::Deferred { reason: "wip" },
        ),
        make_scenario(
            BddScenarioStatus::Deferred { reason: "wip" },
            BddScenarioStatus::Implemented,
        ),
    ];
    let (done, total) = bdd_progress(BddPhase::Core, &scenarios);
    assert_eq!(done, 1);
    assert_eq!(total, 2);
}

#[test]
fn progress_empty() {
    let (done, total) = bdd_progress(BddPhase::Core, &[]);
    assert_eq!(done, 0);
    assert_eq!(total, 0);
}

#[test]
fn progress_runtime_phase() {
    let scenarios = vec![make_scenario(
        BddScenarioStatus::Deferred { reason: "wip" },
        BddScenarioStatus::Implemented,
    )];
    let (done, total) = bdd_progress(BddPhase::Runtime, &scenarios);
    assert_eq!(done, 1);
    assert_eq!(total, 1);
}

// ---------------------------------------------------------------------------
// bdd_progress_report
// ---------------------------------------------------------------------------

#[test]
fn progress_report_basic() {
    let scenarios = vec![make_scenario(
        BddScenarioStatus::Implemented,
        BddScenarioStatus::Implemented,
    )];
    let report = bdd_progress_report(BddPhase::Core, &scenarios, "test");
    assert!(!report.is_empty());
}

#[test]
fn progress_report_empty() {
    let report = bdd_progress_report(BddPhase::Core, &[], "test");
    assert!(!report.is_empty());
}
