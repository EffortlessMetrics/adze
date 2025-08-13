//! Engine adapter for GLR-core integration

use crate::{error::ParseError, language::Language, tree::Tree};

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::{Driver, Forest as CoreForest};

pub enum Forest {
    #[cfg(feature = "glr-core")]
    Glr(CoreForest),
    Stub,
}

pub fn parse_full(language: &Language, input: &[u8]) -> Result<Forest, ParseError> {
    #[cfg(feature = "glr-core")]
    {
        // Check if language has parse table
        if language.parse_table.is_none() {
            return Err(ParseError::with_msg("Language missing parse table - GLR integration pending"));
        }
        
        let parse_table = language.parse_table.as_ref().unwrap();
        let mut drv = Driver::new(parse_table);
        
        // For now, just a trivial byte-lexer: one token = one byte (kind = byte).
        // Replace with your generated lexer/tokenizer.
        let toks = input.iter().enumerate().map(|(i, b)| (*b as u32, i as u32, i as u32 + 1));
        
        let forest = drv.parse_tokens(toks).map_err(map_glr_err)?;
        return Ok(Forest::Glr(forest));
    }
    
    #[cfg(not(feature = "glr-core"))]
    {
        let _ = (language, input);
        Ok(Forest::Stub)
    }
}

pub fn parse_incremental(language: &Language, input: &[u8], _old: &Tree) -> Result<Forest, ParseError> {
    // Call the same path now; replace with proper reuse later.
    parse_full(language, input)
}

#[cfg(feature = "glr-core")]
fn map_glr_err(e: rust_sitter_glr_core::driver::GlrError) -> ParseError {
    ParseError::with_msg(&e.to_string())
}