//! Core BDD grid contracts used for feature/progress reporting.
//!
//! This crate intentionally owns only scenario-grid concerns (what is tracked and how it
//! is summarized) so governance and parser crates can compose behavior without inheriting
//! unrelated policy details.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::Write;

pub use adze_bdd_scenario_core::{BddPhase, BddScenario, BddScenarioStatus};

/// Validation issue found in a BDD scenario grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BddGridValidationIssue {
    /// A scenario id appeared more than once.
    DuplicateScenarioId {
        /// Duplicated id value.
        id: u8,
    },
    /// A scenario had an empty title.
    EmptyTitle {
        /// Scenario id for the invalid row.
        id: u8,
    },
    /// A scenario had an empty reference path.
    EmptyReference {
        /// Scenario id for the invalid row.
        id: u8,
    },
    /// A deferred status was declared without a reason.
    DeferredWithoutReason {
        /// Scenario id for the invalid row.
        id: u8,
        /// Phase that contains the invalid deferred status.
        phase: BddPhase,
    },
}

/// GLR conflict-preservation scenario ledger.
pub const GLR_CONFLICT_PRESERVATION_GRID: &[BddScenario] = &[
    BddScenario {
        id: 1,
        title: "Detect shift/reduce conflicts in ambiguous grammars",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 2,
        title: "Preserve conflicts with precedence ordering (PreferShift)",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 3,
        title: "Preserve conflicts with precedence ordering (PreferReduce)",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 4,
        title: "Use Fork for No Precedence Information",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 5,
        title: "Use Fork for Non-Associative Conflicts",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 6,
        title: "Generate multi-action cells in parse tables",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 7,
        title: "GLR runtime explores both paths",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Deferred {
            reason: "runtime2 integration pending",
        },
        runtime_status: BddScenarioStatus::Implemented,
    },
    BddScenario {
        id: 8,
        title: "Precedence ordering affects tree selection",
        reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
        core_status: BddScenarioStatus::Deferred {
            reason: "runtime2 integration pending",
        },
        runtime_status: BddScenarioStatus::Implemented,
    },
];

/// Aggregate progress for a phase.
///
/// # Examples
///
/// ```
/// use adze_bdd_grid_core::*;
///
/// let scenarios = [BddScenario {
///     id: 1,
///     title: "example",
///     reference: "REF-1",
///     core_status: BddScenarioStatus::Implemented,
///     runtime_status: BddScenarioStatus::Deferred { reason: "todo" },
/// }];
/// let (done, total) = bdd_progress(BddPhase::Core, &scenarios);
/// assert_eq!(done, 1);
/// assert_eq!(total, 1);
/// ```
pub fn bdd_progress(phase: BddPhase, scenarios: &[BddScenario]) -> (usize, usize) {
    let mut implemented = 0usize;
    for scenario in scenarios {
        if scenario.status(phase).implemented() {
            implemented += 1;
        }
    }
    (implemented, scenarios.len())
}

/// Collect validation issues for a scenario grid.
///
/// A valid grid should satisfy all of the following:
/// - scenario ids are unique
/// - scenario titles are non-empty (after trim)
/// - references are non-empty (after trim)
/// - deferred statuses include a non-empty reason (after trim)
pub fn bdd_grid_validation_issues(scenarios: &[BddScenario]) -> Vec<BddGridValidationIssue> {
    let mut issues = Vec::new();
    let mut seen_ids = [false; u8::MAX as usize + 1];

    for scenario in scenarios {
        let id_idx = usize::from(scenario.id);
        if seen_ids[id_idx] {
            issues.push(BddGridValidationIssue::DuplicateScenarioId { id: scenario.id });
        } else {
            seen_ids[id_idx] = true;
        }

        if scenario.title.trim().is_empty() {
            issues.push(BddGridValidationIssue::EmptyTitle { id: scenario.id });
        }

        if scenario.reference.trim().is_empty() {
            issues.push(BddGridValidationIssue::EmptyReference { id: scenario.id });
        }

        for phase in [BddPhase::Core, BddPhase::Runtime] {
            if let BddScenarioStatus::Deferred { reason } = scenario.status(phase)
                && reason.trim().is_empty()
            {
                issues.push(BddGridValidationIssue::DeferredWithoutReason {
                    id: scenario.id,
                    phase,
                });
            }
        }
    }

    issues
}

/// Return whether a scenario grid passes validation checks.
pub fn bdd_grid_is_valid(scenarios: &[BddScenario]) -> bool {
    bdd_grid_validation_issues(scenarios).is_empty()
}

/// Shared formatting for BDD progress summaries.
///
/// # Examples
///
/// ```
/// use adze_bdd_grid_core::*;
///
/// let report = bdd_progress_report(
///     BddPhase::Runtime,
///     GLR_CONFLICT_PRESERVATION_GRID,
///     "Runtime",
/// );
/// assert!(report.contains("Runtime"));
/// assert!(report.contains("Scenario 1"));
/// ```
pub fn bdd_progress_report(
    phase: BddPhase,
    scenarios: &[BddScenario],
    phase_title: &str,
) -> String {
    let mut out = String::new();

    let (implemented, total) = bdd_progress(phase, scenarios);
    out.push_str("\n=== BDD GLR Conflict Preservation Test Summary ===\n");
    out.push_str(phase_title);
    out.push('\n');
    out.push('\n');

    for scenario in scenarios {
        let status = scenario.status(phase);
        let _ = write!(
            out,
            "{} Scenario {}: {} - {}",
            status.icon(),
            scenario.id,
            scenario.title,
            status.label()
        );
        let detail = status.detail();
        if !detail.is_empty() {
            out.push_str(" (");
            out.push_str(detail);
            out.push(')');
        }
        out.push('\n');
    }

    let validation_issues = bdd_grid_validation_issues(scenarios);
    if !validation_issues.is_empty() {
        let _ = write!(
            out,
            "\n⚠️ Grid validation issues detected: {}",
            validation_issues.len()
        );
    }

    out.push('\n');
    let _ = write!(
        out,
        "{}: {}/{} scenarios complete",
        phase_title, implemented, total
    );
    if implemented < total {
        out.push_str("\nNext: Implement remaining deferred scenarios.");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_has_expected_item_count() {
        assert_eq!(GLR_CONFLICT_PRESERVATION_GRID.len(), 8);
    }

    #[test]
    fn progress_summary_reports_counts() {
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert_eq!(implemented, 6);
        assert_eq!(total, 8);
    }

    #[test]
    fn progress_report_text_includes_title() {
        let report =
            bdd_progress_report(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, "Runtime");
        assert!(report.contains("Runtime"));
        assert!(report.contains("Scenario 1"));
    }

    #[test]
    fn grid_validation_passes_for_builtin_grid() {
        assert!(bdd_grid_is_valid(GLR_CONFLICT_PRESERVATION_GRID));
        assert!(bdd_grid_validation_issues(GLR_CONFLICT_PRESERVATION_GRID).is_empty());
    }

    #[test]
    fn grid_validation_detects_common_issues() {
        let scenarios = [
            BddScenario {
                id: 1,
                title: "  ",
                reference: "docs/ref",
                core_status: BddScenarioStatus::Deferred { reason: "" },
                runtime_status: BddScenarioStatus::Implemented,
            },
            BddScenario {
                id: 1,
                title: "valid title",
                reference: "",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Deferred { reason: "   " },
            },
        ];

        let issues = bdd_grid_validation_issues(&scenarios);
        assert!(!issues.is_empty());
        assert!(issues.contains(&BddGridValidationIssue::DuplicateScenarioId { id: 1 }));
        assert!(issues.contains(&BddGridValidationIssue::EmptyTitle { id: 1 }));
        assert!(issues.contains(&BddGridValidationIssue::EmptyReference { id: 1 }));
        assert!(
            issues.contains(&BddGridValidationIssue::DeferredWithoutReason {
                id: 1,
                phase: BddPhase::Core,
            })
        );
        assert!(
            issues.contains(&BddGridValidationIssue::DeferredWithoutReason {
                id: 1,
                phase: BddPhase::Runtime,
            })
        );
    }
}
