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
use std::collections::BTreeSet;

pub use adze_bdd_scenario_core::{BddPhase, BddScenario, BddScenarioStatus};

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

/// Validate that a BDD scenario grid is well-formed.
///
/// Returns a list of human-readable issues. An empty list means the grid passed
/// all built-in checks.
///
/// Current checks:
/// - scenario IDs are unique
/// - scenario IDs are strictly increasing (stable report ordering)
/// - scenario titles are non-empty after trimming
/// - scenario references are non-empty after trimming
pub fn bdd_grid_integrity_issues(scenarios: &[BddScenario]) -> Vec<String> {
    let mut issues = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut previous_id = None::<u8>;

    for (index, scenario) in scenarios.iter().enumerate() {
        if !seen_ids.insert(scenario.id) {
            issues.push(format!(
                "Scenario index {index} has duplicate id {}.",
                scenario.id
            ));
        }

        if let Some(prev) = previous_id
            && scenario.id <= prev
        {
            issues.push(format!(
                "Scenario index {index} has non-increasing id {} (previous id was {prev}).",
                scenario.id
            ));
        }
        previous_id = Some(scenario.id);

        if scenario.title.trim().is_empty() {
            issues.push(format!(
                "Scenario index {index} (id {}) has an empty title.",
                scenario.id
            ));
        }

        if scenario.reference.trim().is_empty() {
            issues.push(format!(
                "Scenario index {index} (id {}) has an empty reference.",
                scenario.id
            ));
        }
    }

    issues
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
    let integrity_issues = bdd_grid_integrity_issues(scenarios);
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
    if !integrity_issues.is_empty() {
        out.push_str("\nGrid integrity warnings:");
        for issue in integrity_issues {
            out.push_str("\n- ");
            out.push_str(&issue);
        }
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
    fn canonical_grid_has_no_integrity_issues() {
        let issues = bdd_grid_integrity_issues(GLR_CONFLICT_PRESERVATION_GRID);
        assert!(issues.is_empty(), "unexpected integrity issues: {issues:?}");
    }

    #[test]
    fn malformed_grid_reports_integrity_issues() {
        let malformed = [
            BddScenario {
                id: 2,
                title: "first",
                reference: "REF-1",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Implemented,
            },
            BddScenario {
                id: 2,
                title: " ",
                reference: "",
                core_status: BddScenarioStatus::Deferred { reason: "todo" },
                runtime_status: BddScenarioStatus::Deferred { reason: "todo" },
            },
        ];

        let issues = bdd_grid_integrity_issues(&malformed);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.contains("duplicate id")));
        assert!(issues.iter().any(|issue| issue.contains("empty title")));
        assert!(issues.iter().any(|issue| issue.contains("empty reference")));
    }
}
