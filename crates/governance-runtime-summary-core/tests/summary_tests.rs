use adze_governance_runtime_summary_core::{ParserFeatureProfile, runtime_governance_summary};

#[test]
fn summary_supports_glr_profile() {
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    let summary = runtime_governance_summary(3, 3, profile);
    assert!(summary.contains("Governance status: 3/3"));
    assert!(summary.contains("Conflict profiles:"));
}
