//! Highlight range data types.

/// A highlighted range in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    /// Start byte offset.
    pub start_byte: usize,
    /// End byte offset.
    pub end_byte: usize,
    /// Highlight name (for example, `keyword` or `string`).
    pub highlight: String,
}

impl Highlight {
    /// Build a highlight range from byte offsets and a capture name.
    pub(crate) fn new(start_byte: usize, end_byte: usize, highlight: String) -> Self {
        Self {
            start_byte,
            end_byte,
            highlight,
        }
    }

    /// Build a sub-range that keeps the same highlight class.
    pub(crate) fn slice(&self, start_byte: usize, end_byte: usize) -> Self {
        Self::new(start_byte, end_byte, self.highlight.clone())
    }
}
