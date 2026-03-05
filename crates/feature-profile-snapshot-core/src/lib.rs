//! Snapshot of parser feature flags captured in build artifacts and diagnostics.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};
use serde::{Deserialize, Serialize};

/// Snapshot of parser feature flags captured in build artifacts and diagnostics.
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
}
