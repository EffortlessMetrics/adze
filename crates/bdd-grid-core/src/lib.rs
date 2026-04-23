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
    out.push_str("=== BDD Scenario Progress Summary ===\n");
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
    if let Some(reference) = scenarios.first().map(|scenario| scenario.reference) {
        let _ = write!(out, "\nReference: {reference}");
    }
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
    fn progress_report_uses_generic_heading_and_reference() {
        let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Core");
        assert!(report.starts_with("=== BDD Scenario Progress Summary ==="));
        assert!(report.contains("Reference: docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md"));
    }

    #[test]
    fn progress_report_omits_reference_for_empty_grid() {
        let report = bdd_progress_report(BddPhase::Core, &[], "Core");
        assert!(!report.contains("Reference:"));
    }
}
