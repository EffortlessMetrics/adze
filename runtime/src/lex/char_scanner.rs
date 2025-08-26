use super::token_source::{Token, TokenSource};
use crate::lexer::{GrammarLexer, Token as LexerToken};

/// CharScanner wraps the existing GrammarLexer to implement TokenSource
pub struct CharScanner<'a> {
    lexer: GrammarLexer,
    input: &'a [u8],
    position: usize,
    cached_token: Option<Token>,
}

impl<'a> CharScanner<'a> {
    /// Create a new [`CharScanner`] over the given input and lexer.
    pub fn new(lexer: GrammarLexer, input: &'a [u8]) -> Self {
        Self {
            lexer,
            input,
            position: 0,
            cached_token: None,
        }
    }

    fn convert_token(&self, lexer_token: LexerToken) -> Token {
        Token {
            sym: lexer_token.symbol.0,
            start: lexer_token.start,
            len: lexer_token.end - lexer_token.start,
        }
    }
}

impl<'a> TokenSource for CharScanner<'a> {
    fn peek(&mut self) -> Option<Token> {
        if self.cached_token.is_none() {
            if let Some(lexer_token) = self.lexer.next_token(self.input, self.position) {
                self.cached_token = Some(self.convert_token(lexer_token));
            }
        }
        self.cached_token
    }

    fn bump(&mut self) {
        if let Some(token) = self.cached_token.take() {
            self.position = token.start + token.len;
        }
    }

    fn offset(&self) -> usize {
        self.position
    }
}
