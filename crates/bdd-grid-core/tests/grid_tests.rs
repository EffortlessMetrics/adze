use adze_bdd_grid_core::*;

#[test]
fn bdd_phase_display_core() {
    assert_eq!(format!("{}", BddPhase::Core), "Core");
}

#[test]
fn bdd_phase_display_runtime() {
    assert_eq!(format!("{}", BddPhase::Runtime), "Runtime");
}

#[test]
fn scenario_status_implemented_properties() {
    let s = BddScenarioStatus::Implemented;
    assert!(s.implemented());
    assert_eq!(s.label(), "IMPLEMENTED");
    assert_eq!(s.icon(), "✅");
    assert_eq!(s.detail(), "");
}

#[test]
fn scenario_status_deferred_properties() {
    let s = BddScenarioStatus::Deferred { reason: "wip" };
    assert!(!s.implemented());
    assert_eq!(s.label(), "DEFERRED");
    assert_eq!(s.icon(), "⏳");
    assert_eq!(s.detail(), "wip");
}

#[test]
fn scenario_status_display() {
    assert_eq!(format!("{}", BddScenarioStatus::Implemented), "Implemented");
    assert!(format!("{}", BddScenarioStatus::Deferred { reason: "todo" }).contains("todo"));
}

#[test]
fn scenario_status_for_phase() {
    let scenario = BddScenario {
        id: 1,
        title: "test",
        reference: "REF-1",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Deferred { reason: "later" },
    };
    assert!(scenario.status(BddPhase::Core).implemented());
    assert!(!scenario.status(BddPhase::Runtime).implemented());
}

#[test]
fn scenario_display() {
    let scenario = BddScenario {
        id: 42,
        title: "my scenario",
        reference: "REF",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    };
    let display = format!("{scenario}");
    assert!(display.contains("42"));
    assert!(display.contains("my scenario"));
}

#[test]
fn bdd_progress_counts_implemented() {
    let scenarios = [
        BddScenario {
            id: 1,
            title: "a",
            reference: "R",
            core_status: BddScenarioStatus::Implemented,
            runtime_status: BddScenarioStatus::Deferred { reason: "x" },
        },
        BddScenario {
            id: 2,
            title: "b",
            reference: "R",
            core_status: BddScenarioStatus::Deferred { reason: "y" },
            runtime_status: BddScenarioStatus::Implemented,
        },
    ];
    let (done, total) = bdd_progress(BddPhase::Core, &scenarios);
    assert_eq!(done, 1);
    assert_eq!(total, 2);

    let (done_rt, total_rt) = bdd_progress(BddPhase::Runtime, &scenarios);
    assert_eq!(done_rt, 1);
    assert_eq!(total_rt, 2);
}

#[test]
fn bdd_progress_empty_scenarios() {
    let (done, total) = bdd_progress(BddPhase::Core, &[]);
    assert_eq!(done, 0);
    assert_eq!(total, 0);
}

#[test]
fn bdd_progress_report_contains_scenario_details() {
    let scenarios = [BddScenario {
        id: 1,
        title: "detect conflicts",
        reference: "REF-1",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Deferred { reason: "pending" },
    }];
    let report = bdd_progress_report(BddPhase::Core, &scenarios, "Core Phase");
    assert!(report.contains("Core Phase"));
    assert!(report.contains("detect conflicts"));
    assert!(report.contains("1/1"));
}

#[test]
fn bdd_progress_report_shows_deferred_reason() {
    let scenarios = [BddScenario {
        id: 1,
        title: "s1",
        reference: "R",
        core_status: BddScenarioStatus::Deferred {
            reason: "not ready",
        },
        runtime_status: BddScenarioStatus::Implemented,
    }];
    let report = bdd_progress_report(BddPhase::Core, &scenarios, "Test");
    assert!(report.contains("not ready"));
    assert!(report.contains("remaining deferred"));
}

#[test]
fn glr_grid_constant_has_items() {
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
    assert_eq!(GLR_CONFLICT_PRESERVATION_GRID.len(), 8);
}

#[test]
fn bdd_grid_validation_helpers_detect_invalid_rows() {
    let scenarios = [
        BddScenario {
            id: 9,
            title: "scenario a",
            reference: "ref",
            core_status: BddScenarioStatus::Implemented,
            runtime_status: BddScenarioStatus::Implemented,
        },
        BddScenario {
            id: 9,
            title: "",
            reference: "",
            core_status: BddScenarioStatus::Deferred { reason: "" },
            runtime_status: BddScenarioStatus::Deferred { reason: "later" },
        },
    ];

    assert!(!bdd_grid_is_valid(&scenarios));
    let issues = bdd_grid_validation_issues(&scenarios);
    assert!(issues.contains(&BddGridValidationIssue::DuplicateScenarioId { id: 9 }));
    assert!(issues.contains(&BddGridValidationIssue::EmptyTitle { id: 9 }));
    assert!(issues.contains(&BddGridValidationIssue::EmptyReference { id: 9 }));
    assert!(
        issues.contains(&BddGridValidationIssue::DeferredWithoutReason {
            id: 9,
            phase: BddPhase::Core,
        })
    );
}

#[test]
fn bdd_progress_report_warns_for_invalid_grid() {
    let scenarios = [BddScenario {
        id: 1,
        title: "test",
        reference: "ref",
        core_status: BddScenarioStatus::Deferred { reason: "" },
        runtime_status: BddScenarioStatus::Implemented,
    }];

    let report = bdd_progress_report(BddPhase::Core, &scenarios, "Core");
    assert!(report.contains("Grid validation issues detected: 1"));
}
