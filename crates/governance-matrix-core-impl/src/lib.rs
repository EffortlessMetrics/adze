//! Core implementation of governance matrix snapshots, profiles, and reporting.
//!
//! This crate owns the combined BDD/feature-policy logic and is reused by facade
//! crates to keep governance API behavior interoperable across parser and runtime
//! entry points.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::Write;

pub use adze_bdd_grid_core::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress,
    bdd_progress_report,
};
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

/// Typed composition of a BDD scenario grid and a parser feature profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BddGovernanceMatrix {
    /// BDD phase being evaluated.
    pub phase: BddPhase,
    /// Active parser feature profile for this build.
    pub profile: ParserFeatureProfile,
    /// Scenario source used for reporting.
    pub scenarios: &'static [BddScenario],
}

impl BddGovernanceMatrix {
    /// Construct a matrix view from an explicit scenario slice.
    pub const fn new(
        phase: BddPhase,
        profile: ParserFeatureProfile,
        scenarios: &'static [BddScenario],
    ) -> Self {
        Self {
            phase,
            profile,
            scenarios,
        }
    }

    /// Construct the canonical matrix for conflict-preservation development.
    pub const fn standard(profile: ParserFeatureProfile) -> Self {
        Self {
            phase: BddPhase::Core,
            profile,
            scenarios: GLR_CONFLICT_PRESERVATION_GRID,
        }
    }

    /// Build a full snapshot for the configured matrix.
    pub fn snapshot(self) -> BddGovernanceSnapshot {
        bdd_governance_snapshot(self.phase, self.scenarios, self.profile)
    }

    /// Render a profile-aware progress report for the configured matrix.
    pub fn report(self, phase_title: &str) -> String {
        bdd_progress_report_with_profile(self.phase, self.scenarios, phase_title, self.profile)
    }

    /// Render a compact status line for the configured matrix.
    pub fn status_line(self) -> String {
        bdd_progress_status_line(self.phase, self.scenarios, self.profile)
    }

    /// Returns true when all scenarios in the matrix are implemented.
    pub fn is_fully_implemented(self) -> bool {
        self.snapshot().is_fully_implemented()
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

/// Compose BDD progress with parser profile diagnostics in one report.
pub fn bdd_progress_report_with_profile(
    phase: BddPhase,
    scenarios: &[BddScenario],
    phase_title: &str,
    profile: ParserFeatureProfile,
) -> String {
    let mut out = bdd_progress_report(phase, scenarios, phase_title);
    let snapshot = bdd_governance_snapshot(phase, scenarios, profile);

    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "Feature profile: {profile}");
    let _ = writeln!(
        &mut out,
        "Non-conflict backend: {}",
        snapshot.non_conflict_backend().name()
    );
    let _ = writeln!(
        &mut out,
        "Conflict grammars: {}",
        describe_backend_for_conflicts(profile)
    );
    let _ = writeln!(
        &mut out,
        "Governance progress: {}/{} scenarios implemented",
        snapshot.implemented, snapshot.total
    );

    out
}

/// Return a stable machine-readable status line for dashboards and CI.
pub fn bdd_progress_status_line(
    phase: BddPhase,
    scenarios: &[BddScenario],
    profile: ParserFeatureProfile,
) -> String {
    let snapshot = bdd_governance_snapshot(phase, scenarios, profile);
    let backend = snapshot.non_conflict_backend().name();
    let phase_label = match phase {
        BddPhase::Core => "core",
        BddPhase::Runtime => "runtime",
    };

    format!(
        "{phase_label}:{implemented}/{total}:{backend}:{profile}",
        implemented = snapshot.implemented,
        total = snapshot.total,
        backend = backend,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reports_expected_counts() {
        let snapshot = bdd_governance_snapshot(
            BddPhase::Core,
            GLR_CONFLICT_PRESERVATION_GRID,
            ParserFeatureProfile::current(),
        );
        assert_eq!(snapshot.implemented, 6);
        assert_eq!(snapshot.total, GLR_CONFLICT_PRESERVATION_GRID.len());
        assert_eq!(snapshot.phase, BddPhase::Core);
    }

    #[test]
    fn status_line_stable_shape() {
        let profile = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };

        let status =
            bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(status.starts_with("runtime:"));
        assert!(status.contains("tree-sitter C runtime"));
        assert!(status.contains("tree-sitter-standard"));
    }

    #[test]
    fn report_with_profile_is_annotated() {
        let profile = ParserFeatureProfile::current();
        let report = bdd_progress_report_with_profile(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Runtime",
            profile,
        );

        assert!(report.contains("Feature profile:"));
        assert!(report.contains("Non-conflict backend:"));
        assert!(report.contains("Conflict grammars:"));
        assert!(report.contains("Governance progress:"));
    }

    #[test]
    fn matrix_adapter_stable_api() {
        let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
        assert_eq!(matrix.phase, BddPhase::Core);
        assert!(
            matrix
                .report("Core")
                .contains("=== BDD GLR Conflict Preservation Test Summary ===")
        );
        assert!(matrix.status_line().starts_with("core:"));
    }
}
