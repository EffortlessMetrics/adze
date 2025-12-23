//! Engine adapter for GLR-core integration
//!
//! This module provides the interface between the runtime2 parser API and the
//! GLR parsing engine. It handles tokenization, parse table validation, and
//! forest construction.

use crate::{error::ParseError, language::Language, tree::Tree};

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::{Driver, Forest as CoreForest};

/// Parse forest representation containing all valid parse trees.
///
/// A GLR parser produces a parse forest rather than a single tree because it
/// maintains all valid interpretations of ambiguous input. The forest structure
/// allows multiple parse paths to be represented efficiently through shared
/// subtrees.
///
/// # Variants
///
/// - `Glr`: Contains a GLR-core forest with full ambiguity support (requires `glr-core` feature)
/// - `Stub`: Placeholder when GLR features are disabled
///
/// # Example
///
/// ```ignore
/// use runtime2::engine::{parse_full, Forest};
///
/// let forest = parse_full(&language, b"1 + 2")?;
/// match forest {
///     Forest::Glr(core_forest) => {
///         // Multiple parse trees may exist in the forest
///         println!("Parsed with GLR engine");
///     }
///     Forest::Stub => {
///         // GLR features not enabled
///     }
/// }
/// ```
pub enum Forest {
    #[cfg(feature = "glr-core")]
    Glr(CoreForest),
    #[cfg(not(feature = "glr-core"))]
    Stub,
}

/// Parses input from scratch using the GLR parsing engine.
///
/// This function performs a complete parse of the input bytes, constructing a
/// parse forest that represents all valid interpretations according to the
/// grammar's parse tables.
///
/// # When to Use
///
/// Use `parse_full()` when:
/// - Parsing new input for the first time
/// - The input has changed substantially since the last parse
/// - You don't have a previous parse tree to reuse
/// - You want to ensure a fresh, complete parse
///
/// For incremental parsing scenarios where you have an old tree and only small
/// edits have been made, consider using `parse_incremental()` instead for better
/// performance.
///
/// # Arguments
///
/// * `language` - Language definition containing parse tables and tokenizer
/// * `input` - Raw input bytes to parse (typically UTF-8 encoded source code)
///
/// # Returns
///
/// Returns a `Forest` containing all valid parse trees, or a `ParseError` if:
/// - The language is missing a parse table
/// - The language is missing a tokenizer
/// - The GLR core feature is not enabled
/// - The input contains syntax errors that cannot be recovered
///
/// # Example
///
/// ```ignore
/// use runtime2::{Language, engine::parse_full};
///
/// let language = Language::builder()
///     .parse_table(&PARSE_TABLE)
///     .tokenizer(|input| Box::new(tokenize(input)))
///     .symbol_names(vec!["number".to_string(), "plus".to_string()])
///     .symbol_metadata(vec![/* metadata */])
///     .build()?;
///
/// let forest = parse_full(&language, b"1 + 2 + 3")?;
/// // Forest now contains all valid parse trees
/// ```
///
/// # Performance
///
/// Full parsing processes the entire input and constructs complete parse forests.
/// For large files with small edits, this can be expensive. The GLR engine
/// maintains all ambiguous interpretations, which may consume more memory than
/// deterministic parsers for highly ambiguous grammars.
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
        Ok(Forest::Glr(forest))
    }

    #[cfg(not(feature = "glr-core"))]
    {
        let _ = (language, input);
        Err(ParseError::with_msg("GLR core feature not enabled"))
    }
}

/// Parses input incrementally, reusing parts of the old parse tree.
///
/// This function performs incremental parsing by analyzing the differences between
/// the old tree and the new input, then selectively reparsing only the changed
/// regions. This can provide significant performance improvements for large files
/// with small edits.
///
/// # When to Use
///
/// Use `parse_incremental()` when:
/// - You have a valid parse tree from a previous parse
/// - Only a small portion of the input has changed (e.g., typing edits)
/// - You want to minimize parsing overhead for large files
/// - The old tree represents the same language/grammar as the new input
///
/// For first-time parsing or when the entire input is new, use `parse_full()` instead.
///
/// # Arguments
///
/// * `language` - Language definition containing parse tables and tokenizer
/// * `input` - New input bytes to parse
/// * `_old` - Previous parse tree (currently unused, will enable subtree reuse in future)
///
/// # Returns
///
/// Returns a `Forest` containing the updated parse trees, or a `ParseError` if
/// parsing fails.
///
/// # Current Implementation
///
/// **Note**: This function currently falls back to `parse_full()`. Full incremental
/// parsing with GLR-aware subtree reuse is planned for future releases. The API
/// is stable and will gain performance improvements without breaking changes.
///
/// # Example
///
/// ```ignore
/// use runtime2::{Language, engine::{parse_full, parse_incremental}};
///
/// // Initial parse
/// let old_forest = parse_full(&language, b"let x = 1;")?;
/// let old_tree = forest_to_tree(old_forest);
///
/// // After editing, reparse incrementally
/// let new_forest = parse_incremental(&language, b"let x = 2;", &old_tree)?;
/// // Future: Will reuse unchanged subtrees for better performance
/// ```
///
/// # Future Performance
///
/// When full incremental support is implemented, this function will:
/// - Identify unchanged token ranges (prefix/suffix)
/// - Reuse valid subtrees from the old forest
/// - Parse only the modified middle section
/// - Splice forests together for 10-16x performance improvement on small edits
#[allow(dead_code)]
pub fn parse_incremental(
    language: &Language,
    input: &[u8],
    _old: &Tree,
) -> Result<Forest, ParseError> {
    // Call the same path now; replace with proper reuse later.
    parse_full(language, input)
}
