//! Core BDD scenario status and ledger-row contracts.
//!
//! This crate isolates the scenario representation from broader grid concerns so
//! policy and reporting crates can depend on a smaller SRP-focused surface.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt;

/// BDD status phase for a scenario.
///
/// # Examples
///
/// ```
/// use adze_bdd_scenario_core::BddPhase;
///
/// let phase = BddPhase::Core;
/// assert_eq!(phase, BddPhase::Core);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BddPhase {
    /// Parser-core validation phase (glr-core).
    Core,
    /// Runtime integration phase (runtime/runtime2).
    Runtime,
}

impl fmt::Display for BddPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Scenario status for a feature matrix row.
///
/// # Examples
///
/// ```
/// use adze_bdd_scenario_core::BddScenarioStatus;
///
/// let done = BddScenarioStatus::Implemented;
/// assert!(done.implemented());
/// assert_eq!(done.label(), "IMPLEMENTED");
///
/// let pending = BddScenarioStatus::Deferred { reason: "wip" };
/// assert!(!pending.implemented());
/// assert_eq!(pending.detail(), "wip");
/// ```
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

impl fmt::Display for BddScenarioStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Implemented => write!(f, "Implemented"),
            Self::Deferred { reason } => write!(f, "Deferred: {reason}"),
        }
    }
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

impl fmt::Display for BddScenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scenario {}: {}", self.id, self.title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_helpers_work() {
        let done = BddScenarioStatus::Implemented;
        assert!(done.implemented());
        assert_eq!(done.icon(), "✅");
        assert_eq!(done.label(), "IMPLEMENTED");
        assert_eq!(done.detail(), "");

        let deferred = BddScenarioStatus::Deferred { reason: "wip" };
        assert!(!deferred.implemented());
        assert_eq!(deferred.icon(), "⏳");
        assert_eq!(deferred.label(), "DEFERRED");
        assert_eq!(deferred.detail(), "wip");
    }

    #[test]
    fn scenario_returns_status_for_phase() {
        let scenario = BddScenario {
            id: 7,
            title: "GLR runtime explores both paths",
            reference: "docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md",
            core_status: BddScenarioStatus::Deferred { reason: "pending" },
            runtime_status: BddScenarioStatus::Implemented,
        };

        assert_eq!(scenario.status(BddPhase::Core), scenario.core_status);
        assert_eq!(scenario.status(BddPhase::Runtime), scenario.runtime_status);
    }
}
