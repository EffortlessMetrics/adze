/// A lexical token the GLR engine consumes.
#[derive(Debug, Clone, Copy)]
pub struct Token {
    /// Symbol id (terminal) in the grammar.
    pub kind: u32,
    /// Byte range (half-open).
    pub start: u32,
    /// End byte position.
    pub end: u32,
}
