//! Shared governance metadata building blocks for BDD scenario progress and parser feature profiles.
//!
//! This crate intentionally owns reusable governance metadata types while
//! delegating parser feature snapshots to a dedicated SRP microcrate.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_bdd_grid_core::{BddPhase, BddScenario, bdd_progress};
use adze_feature_policy_core::ParserFeatureProfile;
pub use adze_feature_profile_snapshot_core::ParserFeatureProfileSnapshot;
use serde::{Deserialize, Serialize};

/// BDD governance metadata embedded in generated parse artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceMetadata {
    /// Phase label for this BDD snapshot.
    pub phase: String,
    /// Implemented scenario count.
    pub implemented: usize,
    /// Total scenario count.
    pub total: usize,
    /// Stable status line for dashboards.
    pub status_line: String,
}

impl GovernanceMetadata {
    /// Whether all known scenarios are complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.total > 0 && self.implemented == self.total
    }

    /// Construct a governance snapshot from explicit counts.
    #[must_use]
    pub fn with_counts(
        phase: impl Into<String>,
        implemented: usize,
        total: usize,
        status_line: impl Into<String>,
    ) -> Self {
        Self {
            phase: phase.into(),
            implemented,
            total,
            status_line: status_line.into(),
        }
    }

    /// Build metadata from a BDD scenario grid and parser feature profile.
    #[must_use]
    pub fn for_grid(
        phase: BddPhase,
        scenarios: &[BddScenario],
        profile: ParserFeatureProfile,
    ) -> Self {
        let (implemented, total) = bdd_progress(phase, scenarios);
        Self {
            phase: phase_name(phase).to_string(),
            implemented,
            total,
            status_line: status_line(phase, implemented, total, profile),
        }
    }
}

impl Default for GovernanceMetadata {
    fn default() -> Self {
        Self {
            phase: "runtime".to_string(),
            implemented: 0,
            total: 0,
            status_line: "runtime:0/0".to_string(),
        }
    }
}

fn phase_name(phase: BddPhase) -> &'static str {
    match phase {
        BddPhase::Core => "core",
        BddPhase::Runtime => "runtime",
    }
}

fn status_line(
    phase: BddPhase,
    implemented: usize,
    total: usize,
    profile: ParserFeatureProfile,
) -> String {
    let backend = profile.resolve_backend(false).name();
    let phase_label = phase_name(phase);
    format!("{phase_label}:{implemented}/{total}:{backend}:{profile}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_metadata_default() {
        let meta = GovernanceMetadata::default();
        assert_eq!(meta.phase, "runtime");
        assert_eq!(meta.implemented, 0);
        assert_eq!(meta.total, 0);
        assert!(!meta.is_complete());
    }

    #[test]
    fn governance_metadata_with_counts() {
        let meta = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");
        assert_eq!(meta.implemented, 5);
        assert_eq!(meta.total, 10);
        assert!(!meta.is_complete());
    }

    #[test]
    fn governance_metadata_complete() {
        let meta = GovernanceMetadata::with_counts("core", 8, 8, "done");
        assert!(meta.is_complete());
    }

    #[test]
    fn governance_metadata_serde_roundtrip() {
        let meta = GovernanceMetadata::with_counts("runtime", 3, 7, "runtime:3/7");
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: GovernanceMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, deserialized);
    }

    #[test]
    fn governance_metadata_for_grid() {
        use adze_bdd_grid_core::BddScenarioStatus;
        let scenarios = [BddScenario {
            id: 1,
            title: "test",
            reference: "T-1",
            core_status: BddScenarioStatus::Implemented,
            runtime_status: BddScenarioStatus::Deferred { reason: "wip" },
        }];
        let profile = ParserFeatureProfile::current();
        let meta = GovernanceMetadata::for_grid(BddPhase::Core, &scenarios, profile);
        assert_eq!(meta.phase, "core");
        assert_eq!(meta.total, 1);
        assert_eq!(meta.implemented, 1);
        assert!(!meta.status_line.is_empty());
    }
}
