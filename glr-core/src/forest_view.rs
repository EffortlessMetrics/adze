//! Object-safe view over a GLR forest/SPPF used by downstream runtimes.

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
pub trait ForestView: Send + Sync {
    /// Root candidate nodes (usually 1).
    fn roots(&self) -> &[u32];
    /// Symbol kind for a node id.
    fn kind(&self, id: u32) -> SymbolId;
    /// Byte span for a node id.
    fn span(&self, id: u32) -> Span;
    /// Children chosen for the best family.
    fn best_children(&self, id: u32) -> &[u32];
}

/// Opaque forest handle exported to consumers via trait object.
pub struct Forest {
    pub(crate) view: Box<dyn ForestView>,
}

impl Forest {
    pub fn view(&self) -> &dyn ForestView {
        &*self.view
    }
}