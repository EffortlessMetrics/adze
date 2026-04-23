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
    /// Scenario id was repeated more than once.
    DuplicateId {
        /// Repeated id value.
        id: u8,
    },
    /// Scenario ids are expected to be strictly ascending.
    OutOfOrderId {
        /// Previous id encountered.
        previous: u8,
        /// Current id that violated ordering.
        current: u8,
    },
    /// Scenario title must not be empty.
    EmptyTitle {
        /// Scenario id with missing title.
        id: u8,
    },
    /// Scenario reference must not be empty.
    EmptyReference {
        /// Scenario id with missing reference.
        id: u8,
    },
}

impl BddGridValidationIssue {
    /// Human-readable description for logs/reports.
    pub fn message(self) -> String {
        match self {
            Self::DuplicateId { id } => format!("duplicate scenario id: {id}"),
            Self::OutOfOrderId { previous, current } => {
                format!("scenario ids must be ascending: {previous} then {current}")
            }
            Self::EmptyTitle { id } => format!("scenario {id} has an empty title"),
            Self::EmptyReference { id } => format!("scenario {id} has an empty reference"),
        }
    }
}

/// Validate BDD grid integrity constraints used by governance/reporting layers.
///
/// Validation rules:
/// - scenario ids are unique
/// - scenario ids are strictly ascending
/// - titles and references are non-empty
pub fn bdd_grid_validation_issues(scenarios: &[BddScenario]) -> Vec<BddGridValidationIssue> {
    let mut issues = Vec::new();
    let mut seen_ids = [false; u8::MAX as usize + 1];
    let mut previous_id: Option<u8> = None;

    for scenario in scenarios {
        if let Some(previous) = previous_id {
            if scenario.id <= previous {
                issues.push(BddGridValidationIssue::OutOfOrderId {
                    previous,
                    current: scenario.id,
                });
            }
        }
        previous_id = Some(scenario.id);

        let id_idx = usize::from(scenario.id);
        if seen_ids[id_idx] {
            issues.push(BddGridValidationIssue::DuplicateId { id: scenario.id });
        } else {
            seen_ids[id_idx] = true;
        }

        if scenario.title.is_empty() {
            issues.push(BddGridValidationIssue::EmptyTitle { id: scenario.id });
        }
        if scenario.reference.is_empty() {
            issues.push(BddGridValidationIssue::EmptyReference { id: scenario.id });
        }
    }

    issues
}

/// Return true when a scenario grid passes integrity validation.
pub fn bdd_grid_is_valid(scenarios: &[BddScenario]) -> bool {
    bdd_grid_validation_issues(scenarios).is_empty()
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
    fn canonical_grid_passes_validation() {
        let issues = bdd_grid_validation_issues(GLR_CONFLICT_PRESERVATION_GRID);
        assert!(issues.is_empty());
        assert!(bdd_grid_is_valid(GLR_CONFLICT_PRESERVATION_GRID));
    }

    #[test]
    fn validation_detects_duplicate_out_of_order_and_empty_fields() {
        let scenarios = [
            BddScenario {
                id: 2,
                title: "",
                reference: "ref-2",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Implemented,
            },
            BddScenario {
                id: 1,
                title: "scenario-1",
                reference: "",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Implemented,
            },
            BddScenario {
                id: 1,
                title: "scenario-1-duplicate",
                reference: "ref-1b",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Implemented,
            },
        ];

        let issues = bdd_grid_validation_issues(&scenarios);
        assert!(issues.contains(&BddGridValidationIssue::EmptyTitle { id: 2 }));
        assert!(issues.contains(&BddGridValidationIssue::OutOfOrderId {
            previous: 2,
            current: 1,
        }));
        assert!(issues.contains(&BddGridValidationIssue::EmptyReference { id: 1 }));
        assert!(issues.contains(&BddGridValidationIssue::DuplicateId { id: 1 }));
        assert!(!bdd_grid_is_valid(&scenarios));
    }

    #[test]
    fn validation_issue_message_is_human_readable() {
        let duplicate = BddGridValidationIssue::DuplicateId { id: 7 };
        assert_eq!(duplicate.message(), "duplicate scenario id: 7");
    }
}
