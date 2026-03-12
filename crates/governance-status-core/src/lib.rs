//! Status-line and backend-description helpers for governance reporting.
//!
//! This crate isolates machine-readable status output and conflict-backend
//! descriptions so BDD governance cores can focus on snapshot/report assembly.

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

/// Return a stable machine-readable status line for dashboards and CI.
pub fn bdd_progress_status_line(
    phase: BddPhase,
    scenarios: &[BddScenario],
    profile: ParserFeatureProfile,
) -> String {
    let (implemented, total) = bdd_progress(phase, scenarios);
    let backend = profile.resolve_backend(false).name();
    let phase_label = match phase {
        BddPhase::Core => "core",
        BddPhase::Runtime => "runtime",
    };

    format!(
        "{phase_label}:{implemented}/{total}:{backend}:{profile}",
        implemented = implemented,
        total = total,
        backend = backend,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID;

    #[test]
    fn fallback_mentions_glr() {
        assert!(GLR_CONFLICT_FALLBACK.contains("GLR") || GLR_CONFLICT_FALLBACK.contains("glr"));
    }

    #[test]
    fn describe_backend_returns_glr_for_glr_profile() {
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
    fn status_line_is_phase_prefixed() {
        let profile = ParserFeatureProfile::current();
        let core =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let runtime =
            bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

        assert!(core.starts_with("core:"));
        assert!(runtime.starts_with("runtime:"));
    }
}
