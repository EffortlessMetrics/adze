//! Shared runtime report formatting for governance snapshots.
//!
//! This crate centralizes the output wiring for BDD progress + parser feature
//! profile diagnostics so downstream consumers (runtime and fixtures) can render
//! interoperable status lines and reports.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::Write;

pub use adze_bdd_governance_reporting_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

/// Build a runtime-oriented governance report using an explicit feature profile.
pub fn bdd_progress_report_with_profile_runtime(
    phase: BddPhase,
    scenarios: &[BddScenario],
    phase_title: &str,
    profile: ParserFeatureProfile,
) -> String {
    let mut out = bdd_progress_report_with_profile(phase, scenarios, phase_title, profile);
    let (implemented, total) = bdd_progress(phase, scenarios);

    let _ = writeln!(
        &mut out,
        "Governance status: {implemented}/{total} scenarios implemented"
    );
    let _ = writeln!(&mut out, "Feature profile: {profile}");
    let _ = writeln!(
        &mut out,
        "Non-conflict backend: {}",
        profile.resolve_backend(false).name()
    );
    let _ = writeln!(
        &mut out,
        "Conflict profiles: {}",
        describe_backend_for_conflicts(profile)
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_report_uses_grid_and_profile_text() {
        let profile = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };

        let report = bdd_progress_report_with_profile_runtime(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Runtime",
            profile,
        );

        assert!(report.contains("Runtime"));
        assert!(report.contains("Feature profile:"));
        assert!(report.contains("Governance status:"));
    }

    #[test]
    fn runtime_status_line_is_reusable() {
        let profile = ParserFeatureProfile::current();
        let status =
            bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

        assert!(status.starts_with("runtime:"));
        assert!(status.contains(&format!("{}", profile)));
    }
}
