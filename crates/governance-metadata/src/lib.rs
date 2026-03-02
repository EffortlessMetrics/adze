//! Shared governance metadata building blocks for BDD scenario progress and parser feature profiles.
//!
//! This crate intentionally owns the small, reusable metadata types used by build-time
//! artifacts and runtime dashboards.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_bdd_grid_core::{BddPhase, BddScenario, bdd_progress};
use adze_feature_policy_core::ParserFeatureProfile;
use serde::{Deserialize, Serialize};

pub use adze_parser_feature_snapshot_core::ParserFeatureProfileSnapshot;

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
    pub fn is_complete(&self) -> bool {
        self.implemented == self.total
    }

    /// Construct a governance snapshot from explicit counts.
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
