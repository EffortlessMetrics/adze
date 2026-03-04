//! Profile-aware report/status rendering for BDD governance tracking.
//!
//! This crate intentionally owns formatting concerns so governance matrix core
//! logic can stay focused on typed snapshots and matrix composition.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::Write;

pub use adze_bdd_grid_core::{BddPhase, BddScenario, bdd_progress, bdd_progress_report};
pub use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};
pub use adze_governance_status_core::{
    GLR_CONFLICT_FALLBACK, bdd_progress_status_line, describe_backend_for_conflicts,
};

/// Compose BDD progress with parser profile diagnostics in one report.
pub fn bdd_progress_report_with_profile(
    phase: BddPhase,
    scenarios: &[BddScenario],
    phase_title: &str,
    profile: ParserFeatureProfile,
) -> String {
    let mut out = bdd_progress_report(phase, scenarios, phase_title);
    let (implemented, total) = bdd_progress(phase, scenarios);

    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "Feature profile: {profile}");
    let _ = writeln!(
        &mut out,
        "Non-conflict backend: {}",
        profile.resolve_backend(false).name()
    );
    let _ = writeln!(
        &mut out,
        "Conflict grammars: {}",
        describe_backend_for_conflicts(profile)
    );
    let _ = writeln!(
        &mut out,
        "Governance progress: {implemented}/{total} scenarios implemented"
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID;

    #[test]
    fn conflict_backend_description_prefers_glr() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        assert_eq!(
            describe_backend_for_conflicts(profile),
            ParserBackend::GLR.name()
        );
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
}
