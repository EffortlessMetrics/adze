/// A lexical token the GLR engine consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// Symbol id (terminal) in the grammar.
    pub kind: u32,
    /// Byte range (half-open).
    pub start: u32,
    /// Byte offset where the token ends.
    pub end: u32,
}
