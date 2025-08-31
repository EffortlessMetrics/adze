#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A token produced by the lexer.
pub struct Token {
    /// Symbol identifier for this token.
    pub sym: u16,
    /// Starting byte offset in the source.
    pub start: usize,
    /// Length of the token in bytes.
    pub len: usize,
}

/// A source of tokens for the parser.
pub trait TokenSource {
    /// Peek at the next token without consuming it.
    fn peek(&mut self) -> Option<Token>;
    /// Advance past the previously peeked token.
    fn bump(&mut self);
    /// Current byte offset.
    fn offset(&self) -> usize;
}
