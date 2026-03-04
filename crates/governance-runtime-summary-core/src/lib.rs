//! Focused formatting helpers for runtime governance summary lines.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::Write;

pub use adze_governance_matrix_contract::{ParserFeatureProfile, describe_backend_for_conflicts};

/// Build the runtime governance summary section appended to BDD progress reports.
#[must_use]
pub fn runtime_governance_summary(
    implemented: usize,
    total: usize,
    profile: ParserFeatureProfile,
) -> String {
    let mut out = String::new();
    append_runtime_governance_summary(&mut out, implemented, total, profile);
    out
}

/// Append runtime governance summary lines to an existing output buffer.
pub fn append_runtime_governance_summary(
    out: &mut String,
    implemented: usize,
    total: usize,
    profile: ParserFeatureProfile,
) {
    let _ = writeln!(
        out,
        "Governance status: {implemented}/{total} scenarios implemented"
    );
    let _ = writeln!(out, "Feature profile: {profile}");
    let _ = writeln!(
        out,
        "Non-conflict backend: {}",
        profile.resolve_backend(false).name()
    );
    let _ = writeln!(
        out,
        "Conflict profiles: {}",
        describe_backend_for_conflicts(profile)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_includes_key_runtime_lines() {
        let profile = ParserFeatureProfile::current();
        let summary = runtime_governance_summary(5, 8, profile);

        assert!(summary.contains("Governance status: 5/8 scenarios implemented"));
        assert!(summary.contains("Feature profile:"));
        assert!(summary.contains("Non-conflict backend:"));
        assert!(summary.contains("Conflict profiles:"));
    }

    #[test]
    fn append_summary_preserves_existing_prefix() {
        let profile = ParserFeatureProfile::current();
        let mut out = String::from("prefix\n");

        append_runtime_governance_summary(&mut out, 0, 0, profile);

        assert!(out.starts_with("prefix\n"));
        assert!(out.contains("Governance status: 0/0 scenarios implemented"));
    }
}
