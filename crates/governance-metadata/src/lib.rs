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
use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};
use serde::{Deserialize, Serialize};

/// Snapshot of parser feature flags captured in build artifacts and diagnostics.
///
/// # Examples
///
/// ```
/// use adze_governance_metadata::ParserFeatureProfileSnapshot;
///
/// let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);
/// assert!(snap.pure_rust);
/// assert!(snap.tree_sitter_c2rust);
/// assert!(!snap.glr);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParserFeatureProfileSnapshot {
    /// Pure-rust mode flag.
    pub pure_rust: bool,
    /// `tree-sitter-standard` feature flag.
    pub tree_sitter_standard: bool,
    /// `tree-sitter-c2rust` feature flag.
    pub tree_sitter_c2rust: bool,
    /// GLR feature flag.
    pub glr: bool,
}

impl ParserFeatureProfileSnapshot {
    /// Create a snapshot from explicit parser feature flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_governance_metadata::ParserFeatureProfileSnapshot;
    ///
    /// let snap = ParserFeatureProfileSnapshot::new(false, true, false, true);
    /// assert!(!snap.pure_rust);
    /// assert!(snap.tree_sitter_standard);
    /// assert!(snap.glr);
    /// ```
    #[must_use]
    pub const fn new(
        pure_rust: bool,
        tree_sitter_standard: bool,
        tree_sitter_c2rust: bool,
        glr: bool,
    ) -> Self {
        Self {
            pure_rust,
            tree_sitter_standard,
            tree_sitter_c2rust,
            glr,
        }
    }

    /// Create a snapshot from the parser-profile contract.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_governance_metadata::ParserFeatureProfileSnapshot;
    /// use adze_feature_policy_core::ParserFeatureProfile;
    ///
    /// let profile = ParserFeatureProfile {
    ///     pure_rust: true, tree_sitter_standard: false,
    ///     tree_sitter_c2rust: false, glr: true,
    /// };
    /// let snap = ParserFeatureProfileSnapshot::from_profile(profile);
    /// assert!(snap.pure_rust);
    /// assert!(snap.glr);
    /// ```
    #[must_use]
    pub const fn from_profile(profile: ParserFeatureProfile) -> Self {
        Self {
            pure_rust: profile.pure_rust,
            tree_sitter_standard: profile.tree_sitter_standard,
            tree_sitter_c2rust: profile.tree_sitter_c2rust,
            glr: profile.glr,
        }
    }

    /// Resolve an equivalent parser-profile from this snapshot.
    #[must_use]
    pub const fn as_profile(self) -> ParserFeatureProfile {
        ParserFeatureProfile {
            pure_rust: self.pure_rust,
            tree_sitter_standard: self.tree_sitter_standard,
            tree_sitter_c2rust: self.tree_sitter_c2rust,
            glr: self.glr,
        }
    }

    /// Build a snapshot from Cargo feature environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            pure_rust: env_flag(&["CARGO_FEATURE_PURE_RUST", "ADZE_USE_PURE_RUST"]),
            tree_sitter_standard: env_flag(&["CARGO_FEATURE_TREE_SITTER_STANDARD"]),
            tree_sitter_c2rust: env_flag(&["CARGO_FEATURE_TREE_SITTER_C2RUST"]),
            glr: env_flag(&["CARGO_FEATURE_GLR"]),
        }
    }

    /// Return the non-conflict backend name implied by this profile.
    #[must_use]
    pub const fn non_conflict_backend(self) -> &'static str {
        if self.glr {
            ParserBackend::GLR.name()
        } else if self.pure_rust {
            ParserBackend::PureRust.name()
        } else {
            ParserBackend::TreeSitter.name()
        }
    }

    /// Resolve the non-conflict backend for this profile.
    #[must_use]
    pub const fn resolve_non_conflict_backend(self) -> ParserBackend {
        self.as_profile().resolve_backend(false)
    }

    /// Resolve backend selection for a grammar with conflicts.
    #[must_use]
    pub const fn resolve_conflict_backend(self) -> ParserBackend {
        self.as_profile().resolve_backend(true)
    }
}

fn env_flag(names: &[&str]) -> bool {
    names.iter().any(|name| std::env::var(name).is_ok())
}

/// BDD governance metadata embedded in generated parse artifacts.
///
/// # Examples
///
/// ```
/// use adze_governance_metadata::GovernanceMetadata;
///
/// let meta = GovernanceMetadata::with_counts("core", 5, 8, "core:5/8");
/// assert_eq!(meta.implemented, 5);
/// assert_eq!(meta.total, 8);
/// assert!(!meta.is_complete());
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_governance_metadata::GovernanceMetadata;
    ///
    /// let done = GovernanceMetadata::with_counts("rt", 8, 8, "rt:8/8");
    /// assert!(done.is_complete());
    ///
    /// let wip = GovernanceMetadata::with_counts("rt", 3, 8, "rt:3/8");
    /// assert!(!wip.is_complete());
    /// ```
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.implemented == self.total
    }

    /// Construct a governance snapshot from explicit counts.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_governance_metadata::GovernanceMetadata;
    ///
    /// let meta = GovernanceMetadata::with_counts("runtime", 6, 8, "runtime:6/8");
    /// assert_eq!(meta.phase, "runtime");
    /// assert_eq!(meta.implemented, 6);
    /// ```
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
    fn profile_snapshot_new() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);
        assert!(snap.pure_rust);
        assert!(!snap.tree_sitter_standard);
        assert!(snap.tree_sitter_c2rust);
        assert!(!snap.glr);
    }

    #[test]
    fn profile_snapshot_roundtrip_via_profile() {
        let snap = ParserFeatureProfileSnapshot::new(false, true, false, true);
        let profile = snap.as_profile();
        let snap2 = ParserFeatureProfileSnapshot::from_profile(profile);
        assert_eq!(snap, snap2);
    }

    #[test]
    fn profile_snapshot_serde_roundtrip() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, true, true);
        let json = serde_json::to_string(&snap).unwrap();
        let deserialized: ParserFeatureProfileSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, deserialized);
    }

    #[test]
    fn non_conflict_backend_name_is_non_empty() {
        let snap = ParserFeatureProfileSnapshot::new(false, false, false, false);
        assert!(!snap.non_conflict_backend().is_empty());
    }

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

    #[test]
    fn non_conflict_backend_glr_profile() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
        assert_eq!(snap.non_conflict_backend(), ParserBackend::GLR.name());
    }

    #[test]
    fn non_conflict_backend_pure_rust_profile() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, false, false);
        assert_eq!(snap.non_conflict_backend(), ParserBackend::PureRust.name());
    }

    #[test]
    fn non_conflict_backend_tree_sitter_fallback() {
        let snap = ParserFeatureProfileSnapshot::new(false, true, false, false);
        assert_eq!(snap.non_conflict_backend(), ParserBackend::TreeSitter.name());
    }

    #[test]
    fn resolve_non_conflict_and_conflict_backend() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
        let non_conflict = snap.resolve_non_conflict_backend();
        let conflict = snap.resolve_conflict_backend();
        assert_eq!(non_conflict, snap.as_profile().resolve_backend(false));
        assert_eq!(conflict, snap.as_profile().resolve_backend(true));
    }

    #[test]
    fn profile_snapshot_debug_format() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);
        let debug = format!("{:?}", snap);
        assert!(debug.contains("ParserFeatureProfileSnapshot"));
        assert!(debug.contains("pure_rust: true"));
    }

    #[test]
    fn profile_snapshot_hash_consistency() {
        use std::collections::HashSet;
        let a = ParserFeatureProfileSnapshot::new(true, false, true, false);
        let b = ParserFeatureProfileSnapshot::new(true, false, true, false);
        let c = ParserFeatureProfileSnapshot::new(false, false, true, false);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn profile_snapshot_copy_semantics() {
        let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);
        let copied = snap;
        assert_eq!(snap, copied);
    }

    #[test]
    fn from_env_with_no_vars() {
        // With no governance env vars set, all fields should be false
        // (unless actual Cargo features are active)
        let snap = ParserFeatureProfileSnapshot::from_env();
        let _ = snap.non_conflict_backend();
        // Just verify it doesn't panic
    }

    #[test]
    fn governance_metadata_clone_eq() {
        let meta = GovernanceMetadata::with_counts("core", 3, 5, "core:3/5");
        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }

    #[test]
    fn governance_metadata_for_grid_runtime_phase() {
        use adze_bdd_grid_core::BddScenarioStatus;
        let scenarios = [BddScenario {
            id: 1,
            title: "test",
            reference: "T-1",
            core_status: BddScenarioStatus::Deferred { reason: "later" },
            runtime_status: BddScenarioStatus::Implemented,
        }];
        let profile = ParserFeatureProfile::current();
        let meta = GovernanceMetadata::for_grid(BddPhase::Runtime, &scenarios, profile);
        assert_eq!(meta.phase, "runtime");
        assert_eq!(meta.implemented, 1);
        assert_eq!(meta.total, 1);
    }

    #[test]
    fn governance_metadata_for_grid_empty_scenarios() {
        let profile = ParserFeatureProfile::current();
        let meta = GovernanceMetadata::for_grid(BddPhase::Core, &[], profile);
        assert_eq!(meta.implemented, 0);
        assert_eq!(meta.total, 0);
    }

    #[test]
    fn governance_metadata_debug_format() {
        let meta = GovernanceMetadata::with_counts("core", 1, 2, "core:1/2");
        let debug = format!("{:?}", meta);
        assert!(debug.contains("GovernanceMetadata"));
        assert!(debug.contains("core"));
    }
}
