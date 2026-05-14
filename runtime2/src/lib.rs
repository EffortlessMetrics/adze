//! GLR runtime for adze parsers with Tree-sitter API compatibility
//!
//! This crate provides a runtime that mimics Tree-sitter's API while using
//! GLR parsing internally to handle ambiguous grammars.
//!
//! # Quick start
//! ```ignore
//! use adze_runtime::{Parser, Language, Token};
//! // `parse_table` and metadata would come from a generated parser crate.
//! let lang = Language::builder()
//!     .parse_table(todo!())
//!     .symbol_metadata(vec![])
//!     .build()
//!     .unwrap()
//!     .with_static_tokens(vec![
//!         Token { kind: 1, start: 0, end: 1 },
//!         Token { kind: 0, start: 1, end: 1 }, // EOF
//!     ]);
//! let mut p = Parser::new();
//! p.set_language(lang).unwrap();
//! let _ = p.parse("a", None);
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod external_scanner;
pub mod language;
pub mod node;
pub mod parser;
pub mod tree;

#[cfg(feature = "glr")]
mod builder;
#[cfg(feature = "glr")]
mod engine;
/// Forest-to-tree conversion for GLR parsing (Phase 3.2)
#[cfg(feature = "pure-rust")]
pub mod forest_converter;
/// GLR parsing engine (Phase 3.1)
#[cfg(feature = "pure-rust")]
pub mod glr_engine;
/// Token types and lexing helpers.
pub mod token;
/// Lexical scanner (tokenizer) for GLR parsing (Phase 3.2)
#[cfg(feature = "pure-rust")]
pub mod tokenizer;

/// Test helper utilities for creating stub languages and parse tables.
///
/// Available for tests and when the `test-utils` feature is enabled.
#[cfg(any(test, feature = "test-utils"))]
pub mod test_helpers;

// Re-exports for convenience
pub use adze_parsetable_metadata::{
    GovernanceMetadata, ParserFeatureProfileSnapshot, ParsetableMetadata,
};
pub use error::{ParseError, ParseErrorKind};
pub use external_scanner::{ExternalScanner, ScanResult};
pub use language::Language;
pub use node::{Node, Point};
pub use parser::Parser;
pub use token::Token;
pub use tree::Tree;

// Governance + feature-flag reporting compatibility surface for runtime2 consumers.
pub use adze_bdd_governance_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime2, bdd_governance_snapshot,
    bdd_progress, bdd_progress_report, bdd_progress_report_for_profile,
    bdd_progress_report_with_profile, bdd_progress_status_line,
    bdd_progress_status_line_for_profile, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
};

#[cfg(feature = "incremental_glr")]
pub use tree::EditError;

/// Return the active runtime2 parser feature profile.
pub const fn parser_feature_profile_for_current_runtime2() -> ParserFeatureProfile {
    parser_feature_profile_for_runtime2(cfg!(feature = "pure-rust"))
}

/// Resolve the backend for the active runtime2 feature profile.
pub const fn current_backend_for_runtime2(has_conflicts: bool) -> ParserBackend {
    resolve_runtime2_backend(cfg!(feature = "pure-rust"), has_conflicts)
}

/// Build a governance matrix for a runtime2-compatible profile.
pub fn bdd_governance_matrix_for_runtime2_profile(
    phase: BddPhase,
    pure_rust_glr: bool,
) -> BddGovernanceMatrix {
    bdd_governance_matrix_for_runtime2(phase, pure_rust_glr)
}

/// Build a BDD report for an explicit runtime2 profile.
pub fn bdd_progress_report_for_runtime2_profile(
    phase: BddPhase,
    phase_title: &str,
    profile: ParserFeatureProfile,
) -> String {
    adze_bdd_governance_core::bdd_progress_report_with_profile_runtime(
        phase,
        GLR_CONFLICT_PRESERVATION_GRID,
        phase_title,
        profile,
    )
}

/// Build a BDD status line for an explicit runtime2 profile.
pub fn bdd_progress_status_line_for_runtime2_profile(
    phase: BddPhase,
    profile: ParserFeatureProfile,
) -> String {
    bdd_progress_status_line_for_profile(phase, profile)
}

/// Resolve runtime2 backend resolution from an explicit profile.
pub const fn resolve_backend_for_runtime2_profile(
    profile: ParserFeatureProfile,
    has_conflicts: bool,
) -> ParserBackend {
    resolve_backend_for_profile(profile, has_conflicts)
}

/// Resolve runtime2 backend resolution directly from the `pure-rust-glr` toggle.
pub const fn resolve_runtime2_backend(pure_rust_glr: bool, has_conflicts: bool) -> ParserBackend {
    resolve_backend_for_profile(
        parser_feature_profile_for_runtime2(pure_rust_glr),
        has_conflicts,
    )
}

/// Build a runtime2 governance snapshot for an explicit profile.
pub fn runtime2_governance_snapshot(
    phase: BddPhase,
    profile: ParserFeatureProfile,
) -> BddGovernanceSnapshot {
    bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile)
}

/// Resolve the backend for the active runtime2 feature profile.
///
/// This mirrors the runtime crate helper shape (`current_backend_for`) while
/// preserving the runtime2 context for consumers that need an explicit entry point.
pub const fn current_backend_for(has_conflicts: bool) -> ParserBackend {
    current_backend_for_runtime2(has_conflicts)
}

/// Build the BDD progress report for the active runtime2 profile.
///
/// Uses the active runtime2 feature profile and the canonical GLR scenario grid.
pub fn bdd_progress_report_for_current_profile(phase: BddPhase, phase_title: &str) -> String {
    bdd_progress_report_for_runtime2_profile(
        phase,
        phase_title,
        parser_feature_profile_for_current_runtime2(),
    )
}

/// Build the BDD progress status line for the active runtime2 profile.
///
/// Status line is a compact machine-readable summary suitable for CI logging.
pub fn bdd_progress_status_line_for_current_profile(phase: BddPhase) -> String {
    bdd_progress_status_line_for_runtime2_profile(
        phase,
        parser_feature_profile_for_current_runtime2(),
    )
}

/// Input edit information for incremental parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEdit {
    /// Byte offset where the edit starts
    pub start_byte: usize,
    /// Byte offset where the edit ended in the old text
    pub old_end_byte: usize,
    /// Byte offset where the edit ends in the new text
    pub new_end_byte: usize,
    /// Point where the edit starts
    pub start_position: Point,
    /// Point where the edit ended in the old text
    pub old_end_position: Point,
    /// Point where the edit ends in the new text
    pub new_end_position: Point,
}

/// Query system types for pattern matching over parse trees.
///
/// This module is a stub for future implementation. Requires the `query` feature.
#[cfg(feature = "query")]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
pub mod query {
    /// A compiled query for matching patterns in syntax trees.
    ///
    /// Queries are compiled from S-expression patterns and can be executed
    /// against any parse tree to find matching nodes.
    pub struct Query;
    /// A stateful cursor for executing queries against a tree.
    ///
    /// Manages iteration state when running a [`Query`] against a parse tree,
    /// yielding [`QueryMatch`] results.
    pub struct QueryCursor;
    /// A single match result from executing a query.
    ///
    /// Contains the captured nodes that matched the query pattern.
    pub struct QueryMatch;
}
