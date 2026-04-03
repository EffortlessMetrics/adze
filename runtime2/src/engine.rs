//! Engine adapter for GLR-core integration
//!
//! This module provides the interface between the runtime2 parser API and the
//! GLR parsing engine. It handles tokenization, parse table validation, and
//! forest construction.

use crate::{error::ParseError, language::Language, tree::Tree};

#[cfg(feature = "glr")]
use adze_glr_core::{Driver, Forest as CoreForest};

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
    #[cfg(feature = "glr")]
    Glr(CoreForest),
    #[cfg(not(feature = "glr"))]
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
    #[cfg(feature = "glr")]
    {
        let parse_table = language.parse_table.ok_or_else(|| {
            ParseError::with_msg("Language missing parse table - GLR integration pending")
        })?;
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

    #[cfg(not(feature = "glr"))]
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

#[cfg(all(feature = "glr", test))]
mod tests {
    use super::*;
    use crate::{Token, language::SymbolMetadata, tree::Tree};
    use adze_glr_core::{Action, ParseTable, StateId, SymbolId};

    fn shift_accept_table() -> &'static ParseTable {
        let mut symbol_to_index = std::collections::BTreeMap::new();
        symbol_to_index.insert(SymbolId(0), 0);
        symbol_to_index.insert(SymbolId(1), 1);

        let table = ParseTable {
            state_count: 2,
            symbol_count: 2,
            symbol_to_index,
            start_symbol: SymbolId(1),
            index_to_symbol: vec![SymbolId(0), SymbolId(1)],
            action_table: vec![
                vec![vec![], vec![Action::Shift(StateId(1))]],
                vec![vec![Action::Accept], vec![]],
            ],
            goto_table: vec![vec![], vec![]],
            ..Default::default()
        };
        Box::leak(Box::new(table))
    }

    fn tiny_language() -> Language {
        Language::builder()
            .parse_table(shift_accept_table())
            .symbol_names(vec!["eof".into(), "token".into()])
            .symbol_metadata(vec![
                SymbolMetadata {
                    is_terminal: true,
                    is_visible: true,
                    is_supertype: false,
                },
                SymbolMetadata {
                    is_terminal: true,
                    is_visible: true,
                    is_supertype: false,
                },
            ])
            .tokenizer(|_| {
                Box::new(
                    vec![
                        Token {
                            kind: 1,
                            start: 0,
                            end: 1,
                        },
                        Token {
                            kind: 0,
                            start: 1,
                            end: 1,
                        },
                    ]
                    .into_iter(),
                )
            })
            .build()
            .unwrap()
    }

    #[test]
    fn given_parse_full_and_parse_incremental_when_inputs_match_then_forest_shape_is_identical() {
        let language = tiny_language();
        let input = b"abc";

        let full = parse_full(&language, input).expect("full parse should succeed");
        let incremental = parse_incremental(&language, input, &Tree::new_stub())
            .expect("incremental parse should match full parse path");

        match (full, incremental) {
            (Forest::Glr(full_forest), Forest::Glr(incremental_forest)) => {
                let full_view = full_forest.view();
                let inc_view = incremental_forest.view();
                assert_eq!(full_view.roots().len(), inc_view.roots().len());
            }
        }
    }
}
