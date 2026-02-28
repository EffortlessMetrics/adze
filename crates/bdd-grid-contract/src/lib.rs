//! BDD scenario grid contracts.
//!
//! This crate owns the domain contract for BDD scenario tracking and reporting,
//! with no baked-in fixture datasets.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::Write;

/// BDD status phase for a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BddPhase {
    /// Parser-core validation phase (glr-core).
    Core,
    /// Runtime integration phase (runtime/runtime2).
    Runtime,
}

/// Scenario status for a feature matrix row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BddScenarioStatus {
    /// Completed in a given phase.
    Implemented,
    /// Deferred with reason text.
    Deferred {
        /// Explanation for why the scenario is deferred.
        reason: &'static str,
    },
}

impl BddScenarioStatus {
    /// Whether this scenario is complete.
    pub const fn implemented(self) -> bool {
        matches!(self, Self::Implemented)
    }

    /// Status icon used in logs and summaries.
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Implemented => "✅",
            Self::Deferred { .. } => "⏳",
        }
    }

    /// Short status label for printouts.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Implemented => "IMPLEMENTED",
            Self::Deferred { .. } => "DEFERRED",
        }
    }

    /// Optional detail text for deferred scenarios.
    pub const fn detail(self) -> &'static str {
        match self {
            Self::Implemented => "",
            Self::Deferred { reason } => reason,
        }
    }
}

/// Shared BDD scenario ledger entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BddScenario {
    /// Scenario identifier.
    pub id: u8,
    /// Scenario title/description.
    pub title: &'static str,
    /// Reference document for this scenario.
    pub reference: &'static str,
    /// Core phase status (glr-core).
    pub core_status: BddScenarioStatus,
    /// Runtime phase status (runtime/runtime2 integration).
    pub runtime_status: BddScenarioStatus,
}

impl BddScenario {
    /// Return status for a given phase.
    pub const fn status(self, phase: BddPhase) -> BddScenarioStatus {
        match phase {
            BddPhase::Core => self.core_status,
            BddPhase::Runtime => self.runtime_status,
        }
    }
}

/// Aggregate progress for a phase.
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
