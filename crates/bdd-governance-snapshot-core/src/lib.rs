//! Snapshot and backend policy primitives for BDD governance composition.
//!
//! This crate keeps profile-dependent snapshot logic narrowly scoped so matrix
//! and reporting crates can compose it without duplicating backend semantics.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_grid_core::{BddPhase, BddScenario, bdd_progress};
pub use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

/// Advisory profile description for conflict-capable grammars.
pub const GLR_CONFLICT_FALLBACK: &str =
    "Pure-rust without GLR: conflicts panic unless `glr` feature is enabled";

/// Snapshot of governance progress for one phase and feature profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BddGovernanceSnapshot {
    /// The phase being evaluated.
    pub phase: BddPhase,
    /// Number of implemented scenarios.
    pub implemented: usize,
    /// Total number of scenarios in the slice.
    pub total: usize,
    /// The active parser feature profile used to interpret behavior.
    pub profile: ParserFeatureProfile,
}

impl BddGovernanceSnapshot {
    /// Returns true when all scenarios for this phase are implemented.
    pub const fn is_fully_implemented(self) -> bool {
        self.implemented == self.total
    }

    /// Convenience helper to expose the active non-conflict backend.
    pub const fn non_conflict_backend(self) -> ParserBackend {
        self.profile.resolve_backend(false)
    }
}

/// Describe the conflict backend behavior for a given feature profile.
pub const fn describe_backend_for_conflicts(profile: ParserFeatureProfile) -> &'static str {
    if profile.glr {
        ParserBackend::GLR.name()
    } else if profile.pure_rust {
        GLR_CONFLICT_FALLBACK
    } else {
        ParserBackend::TreeSitter.name()
    }
}

/// Build a compact governance snapshot for a phase.
pub fn bdd_governance_snapshot(
    phase: BddPhase,
    scenarios: &[BddScenario],
    profile: ParserFeatureProfile,
) -> BddGovernanceSnapshot {
    let (implemented, total) = bdd_progress(phase, scenarios);
    BddGovernanceSnapshot {
        phase,
        implemented,
        total,
        profile,
    }
}
