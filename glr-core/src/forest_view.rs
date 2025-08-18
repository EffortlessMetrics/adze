//! Object-safe view over a GLR forest/SPPF used by downstream runtimes.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Numeric symbol id (terminals and nonterminals share the space).
pub type SymbolId = u32;

/// Byte span in input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

/// Object-safe view of a forest/SPPF.
///
/// Notes:
/// - We keep ambiguity handling simple for now: `best_children` returns one
///   chosen family (e.g., first/longest/leftmost). You can extend this later
///   with explicit "families" if you want full ambiguity exposure.
/// - This trait's shape is stable across all build configurations.
/// - This trait is sealed and cannot be implemented outside this crate.
pub trait ForestView: sealed::Sealed + Send + Sync {
    /// Root candidate nodes (usually 1).
    fn roots(&self) -> &[u32];
    /// Symbol kind for a node id.
    fn kind(&self, id: u32) -> SymbolId;
    /// Byte span for a node id.
    fn span(&self, id: u32) -> Span;
    /// Children chosen for the best family.
    fn best_children(&self, id: u32) -> &[u32];
}

/// Test hooks for Forest (only available in test builds).
#[cfg(any(test, feature = "test-api", feature = "test-helpers"))]
pub struct ForestTestHooks {
    /// Cached error stats from the forest.
    /// (has_error_chunks, missing_terminals, total_error_cost).
    pub error_stats: (bool, usize, u32),
}

/// Opaque forest handle exported to consumers via trait object.
pub struct Forest {
    pub(crate) view: Box<dyn ForestView>,
    #[cfg(any(test, feature = "test-api", feature = "test-helpers"))]
    pub(crate) test_hooks: Option<ForestTestHooks>,
}

impl Forest {
    pub fn view(&self) -> &dyn ForestView {
        &*self.view
    }

    /// Test helper: returns (has_error_chunks, missing_terminals, total_error_cost)
    /// Only available in test builds. Not part of the stable runtime API.
    #[cfg(any(test, feature = "test-api", feature = "test-helpers"))]
    pub fn debug_error_stats(&self) -> (bool, usize, u32) {
        let hooks = self
            .test_hooks
            .as_ref()
            .expect("Forest built without test hooks");
        hooks.error_stats
    }
}
