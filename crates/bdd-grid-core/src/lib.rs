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

/// Integrity issue detected in a BDD scenario grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BddGridIssue {
    /// Grid has no scenarios.
    EmptyGrid,
    /// Scenario id is zero, which is reserved as invalid.
    ZeroScenarioId,
    /// Scenario id is duplicated in the grid.
    DuplicateScenarioId {
        /// The duplicated id.
        id: u8,
    },
    /// Scenario title is empty.
    MissingTitle {
        /// Scenario id whose title is missing.
        id: u8,
    },
    /// Scenario reference path is empty.
    MissingReference {
        /// Scenario id whose reference is missing.
        id: u8,
    },
}

impl BddGridIssue {
    /// Human-readable issue message used by summaries.
    pub const fn message(self) -> &'static str {
        match self {
            Self::EmptyGrid => "grid has no scenarios",
            Self::ZeroScenarioId => "scenario id must be non-zero",
            Self::DuplicateScenarioId { .. } => "scenario id is duplicated",
            Self::MissingTitle { .. } => "scenario title is empty",
            Self::MissingReference { .. } => "scenario reference is empty",
        }
    }
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

/// Validate a scenario grid and return all detected issues.
///
/// # Examples
///
/// ```
/// use adze_bdd_grid_core::{BddGridIssue, bdd_grid_issues};
///
/// let issues = bdd_grid_issues(&[]);
/// assert_eq!(issues, vec![BddGridIssue::EmptyGrid]);
/// ```
pub fn bdd_grid_issues(scenarios: &[BddScenario]) -> Vec<BddGridIssue> {
    let mut issues = Vec::new();
    if scenarios.is_empty() {
        issues.push(BddGridIssue::EmptyGrid);
        return issues;
    }

    let mut seen = [false; u8::MAX as usize + 1];
    for scenario in scenarios {
        let id = usize::from(scenario.id);
        if id == 0 {
            issues.push(BddGridIssue::ZeroScenarioId);
        } else if seen[id] {
            issues.push(BddGridIssue::DuplicateScenarioId { id: scenario.id });
        } else {
            seen[id] = true;
        }

        if scenario.title.trim().is_empty() {
            issues.push(BddGridIssue::MissingTitle { id: scenario.id });
        }
        if scenario.reference.trim().is_empty() {
            issues.push(BddGridIssue::MissingReference { id: scenario.id });
        }
    }
    issues
}

/// Returns true when the supplied scenario grid has no integrity issues.
pub fn bdd_grid_is_valid(scenarios: &[BddScenario]) -> bool {
    bdd_grid_issues(scenarios).is_empty()
}

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
    let issues = bdd_grid_issues(scenarios);
    out.push_str("\n=== BDD GLR Conflict Preservation Test Summary ===\n");
    out.push_str(phase_title);
    out.push('\n');
    out.push('\n');

    if !issues.is_empty() {
        out.push_str("⚠️ Grid integrity issues:\n");
        for issue in issues {
            let _ = writeln!(out, "- {}", issue.message());
        }
        out.push('\n');
    }

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
    fn canonical_grid_has_no_integrity_issues() {
        assert!(bdd_grid_is_valid(GLR_CONFLICT_PRESERVATION_GRID));
        assert!(bdd_grid_issues(GLR_CONFLICT_PRESERVATION_GRID).is_empty());
    }

    #[test]
    fn integrity_checks_flag_invalid_rows() {
        let grid = [
            BddScenario {
                id: 1,
                title: "has title",
                reference: "doc.md",
                core_status: BddScenarioStatus::Implemented,
                runtime_status: BddScenarioStatus::Implemented,
            },
            BddScenario {
                id: 1,
                title: "",
                reference: "",
                core_status: BddScenarioStatus::Deferred { reason: "todo" },
                runtime_status: BddScenarioStatus::Deferred { reason: "todo" },
            },
        ];
        let issues = bdd_grid_issues(&grid);
        assert!(issues.contains(&BddGridIssue::DuplicateScenarioId { id: 1 }));
        assert!(issues.contains(&BddGridIssue::MissingTitle { id: 1 }));
        assert!(issues.contains(&BddGridIssue::MissingReference { id: 1 }));
    }
}
