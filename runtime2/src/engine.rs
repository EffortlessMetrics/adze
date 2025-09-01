//! Engine adapter for GLR-core integration

use crate::{error::ParseError, language::Language, tree::Tree};

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::{Driver, Forest as CoreForest};

pub enum Forest {
    #[cfg(feature = "glr-core")]
    Glr(CoreForest),
    #[cfg(not(feature = "glr-core"))]
    Stub,
}

pub fn parse_full(language: &Language, input: &[u8]) -> Result<Forest, ParseError> {
    #[cfg(feature = "glr-core")]
    {
        // Check if language has parse table
        if language.parse_table.is_none() {
            return Err(ParseError::with_msg(
                "Language missing parse table - GLR integration pending",
            ));
        }

        let parse_table = language.parse_table.as_ref().unwrap();
        let mut drv = Driver::new(parse_table);

        let tok_fn = language.tokenize.as_ref().ok_or_else(|| {
            ParseError::with_msg(
                "Language has no tokenizer; generated grammar must set `Language::tokenize`",
            )
        })?;
        let toks = tok_fn(input).map(|t| (t.kind, t.start, t.end));

        let forest = drv.parse_tokens(toks)?;
        return Ok(Forest::Glr(forest));
    }

    #[cfg(not(feature = "glr-core"))]
    {
        let _ = (language, input);
        Err(ParseError::with_msg("GLR core feature not enabled"))
    }
}

pub fn parse_incremental(
    language: &Language,
    input: &[u8],
    _old: &Tree,
) -> Result<Forest, ParseError> {
    // Call the same path now; replace with proper reuse later.
    parse_full(language, input)
}
