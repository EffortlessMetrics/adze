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
use std::collections::HashSet;

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

/// Validate a BDD scenario grid for common authoring errors.
///
/// The returned vector is empty when the grid is valid.
///
/// # Validation checks
///
/// - Scenario IDs are unique.
/// - Scenario IDs are strictly increasing in declaration order.
/// - Scenario titles are non-empty after trimming.
/// - Scenario references are non-empty after trimming.
pub fn bdd_grid_validation_errors(scenarios: &[BddScenario]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::with_capacity(scenarios.len());
    let mut previous_id: Option<u8> = None;

    for (index, scenario) in scenarios.iter().enumerate() {
        if !seen_ids.insert(scenario.id) {
            errors.push(format!(
                "scenario at index {index} reuses duplicate id {}",
                scenario.id
            ));
        }

        if let Some(prev) = previous_id
            && scenario.id <= prev
        {
            errors.push(format!(
                "scenario at index {index} has non-increasing id {} (previous id was {prev})",
                scenario.id
            ));
        }
        previous_id = Some(scenario.id);

        if scenario.title.trim().is_empty() {
            errors.push(format!(
                "scenario id {} at index {index} has an empty title",
                scenario.id
            ));
        }

        if scenario.reference.trim().is_empty() {
            errors.push(format!(
                "scenario id {} at index {index} has an empty reference",
                scenario.id
            ));
        }
    }

    errors
}

/// Return true when a BDD scenario grid passes validation checks.
pub fn bdd_grid_is_valid(scenarios: &[BddScenario]) -> bool {
    bdd_grid_validation_errors(scenarios).is_empty()
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

    let validation_errors = bdd_grid_validation_errors(scenarios);
    if validation_errors.is_empty() {
        out.push_str("\nGrid validation: OK.");
    } else {
        let _ = write!(
            out,
            "\nGrid validation: FAILED ({} issue(s)).",
            validation_errors.len()
        );
        for error in validation_errors {
            out.push_str("\n - ");
            out.push_str(&error);
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
        assert!(report.contains("Grid validation: OK."));
    }

    #[test]
    fn grid_validation_reports_duplicate_ids() {
        let scenarios = [
            BddScenario {
                id: 1,
                title: "First",
                reference: "ref-1",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Implemented,
            },
            BddScenario {
                id: 1,
                title: "Second",
                reference: "ref-2",
                core_status: BddScenarioStatus::Deferred { reason: "pending" },
                runtime_status: BddScenarioStatus::Deferred { reason: "pending" },
            },
        ];

        let errors = bdd_grid_validation_errors(&scenarios);
        assert!(!errors.is_empty());
        assert!(!bdd_grid_is_valid(&scenarios));
    }

    #[test]
    fn progress_report_includes_validation_failures() {
        let scenarios = [BddScenario {
            id: 1,
            title: "",
            reference: "",
            core_status: BddScenarioStatus::Implemented,
            runtime_status: BddScenarioStatus::Implemented,
        }];

        let report = bdd_progress_report(BddPhase::Core, &scenarios, "Core");
        assert!(report.contains("Grid validation: FAILED"));
        assert!(report.contains("empty title"));
        assert!(report.contains("empty reference"));
    }
}
