#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub sym: u16,     // SymbolId.0
    pub start: usize, // byte offset
    pub len: usize,   // byte length
}

pub trait TokenSource {
    fn peek(&mut self) -> Option<Token>;
    fn bump(&mut self); // advance past peeked token
    fn offset(&self) -> usize; // current byte offset
}
