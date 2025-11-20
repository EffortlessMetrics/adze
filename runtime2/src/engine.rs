//! Engine adapter for GLR-core integration
//!
//! This module provides the bridge between the runtime2 parser and the GLR-core
//! parsing engine. It handles:
//!
//! - **Parse table validation**: Ensures languages have valid parse tables
//! - **Token stream processing**: Converts tokenizer output to GLR-core format
//! - **Forest management**: Wraps GLR parse forests for runtime consumption
//!
//! # Architecture
//!
//! The engine adapter follows a clear separation of concerns:
//!
//! ```text
//! ┌─────────────────┐
//! │   Parser API    │  (runtime2/parser.rs)
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  Engine Adapter │  (this module)
//! │                 │
//! │  • Validation   │  Verify parse table presence
//! │  • Tokenization │  Call Language::tokenize
//! │  • GLR Driver   │  Invoke glr-core::Driver
//! │  • Forest Wrap  │  Wrap CoreForest → Forest
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   GLR Driver    │  (glr-core/driver.rs)
//! └─────────────────┘
//! ```
//!
//! # Parse Flow
//!
//! 1. **Validation**: Check that `Language` has parse table and tokenizer
//! 2. **Tokenization**: Call `Language::tokenize(input)` to scan tokens
//! 3. **Parsing**: Create `Driver` with parse table, feed tokens
//! 4. **Forest Construction**: GLR-core builds parse forest
//! 5. **Wrapping**: Wrap `CoreForest` in `Forest::Glr` for runtime
//!
//! # Feature Gates
//!
//! This module uses feature gates to enable GLR functionality:
//!
//! - `#[cfg(feature = "glr-core")]`: GLR parsing with glr-core engine
//! - Without `glr-core`: Stub implementation that returns errors
//!
//! # Example Usage
//!
//! ```ignore
//! use runtime2::engine::parse_full;
//! use runtime2::language::Language;
//!
//! let language = Language {
//!     parse_table: Some(&PARSE_TABLE),
//!     tokenize: Some(|input| { /* tokenization */ }),
//!     // ... other fields
//! };
//!
//! let input = b"1 + 2 * 3";
//! let forest = parse_full(&language, input)?;
//! // Convert forest to tree with builder module
//! ```
//!
//! # Error Handling
//!
//! Returns `ParseError` if:
//! - Language has no parse table
//! - Language has no tokenizer
//! - GLR-core parsing fails (syntax error, ambiguity, etc.)
//! - Feature `glr-core` is not enabled

use crate::{error::ParseError, language::Language, tree::Tree};

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::{Driver, Forest as CoreForest};

/// Parse forest representation
///
/// This enum wraps the GLR-core parse forest for consumption by the runtime.
/// It provides a feature-gated abstraction over the GLR forest structure.
///
/// # Variants
///
/// - **`Glr(CoreForest)`**: GLR parse forest from glr-core engine
///   - Available with `#[cfg(feature = "glr-core")]`
///   - Contains the complete parse forest with all ambiguities preserved
///   - Can be converted to a tree using `builder::forest_to_tree()`
///
/// - **`Stub`**: Placeholder when GLR-core is disabled
///   - Used when compiling without `glr-core` feature
///   - Allows compilation but parsing will fail at runtime
///
/// # Conversion to Tree
///
/// Use `builder::forest_to_tree()` to convert a forest to a tree:
///
/// ```ignore
/// use runtime2::engine::Forest;
/// use runtime2::builder::forest_to_tree;
///
/// let forest = Forest::Glr(core_forest);
/// let tree = forest_to_tree(forest); // Performs disambiguation
/// ```
///
/// # Memory Model
///
/// - Forests are **moved** from the engine to the builder
/// - No cloning or borrowing required during conversion
/// - Memory-efficient for large parse results
pub enum Forest {
    #[cfg(feature = "glr-core")]
    Glr(CoreForest),
    #[cfg(not(feature = "glr-core"))]
    Stub,
}

/// Parse input from scratch using the GLR engine
///
/// This is the primary parsing entry point for full (non-incremental) parsing.
/// It takes a language definition and input bytes, returning a parse forest.
///
/// # Arguments
///
/// - `language`: Language definition with parse table and tokenizer
/// - `input`: Input bytes to parse (typically UTF-8 text)
///
/// # Returns
///
/// - `Ok(Forest)`: Parse forest containing all valid parse trees
/// - `Err(ParseError)`: If validation fails or parsing encounters errors
///
/// # Validation Steps
///
/// 1. **Parse table check**: Ensures `language.parse_table.is_some()`
/// 2. **Tokenizer check**: Ensures `language.tokenize.is_some()`
/// 3. **GLR-core driver creation**: Constructs `Driver` with parse table
/// 4. **Token stream parsing**: Feeds tokens to GLR driver
///
/// # Parse Algorithm
///
/// The GLR engine uses a generalized LR parsing algorithm:
///
/// - **Multi-action cells**: Each state/symbol can have multiple valid actions
/// - **Runtime forking**: Parser forks on conflicts, exploring all paths
/// - **Graph-Structured Stack (GSS)**: Efficient representation of all parse states
/// - **Shared Packed Parse Forest (SPPF)**: Compact forest of all parse trees
///
/// # Performance Characteristics
///
/// - **Time**: O(n) for LR grammars, O(n³) worst case for ambiguous grammars
/// - **Space**: O(n) for GSS, O(n²) worst case for highly ambiguous inputs
/// - **Typical**: Most practical grammars parse in O(n) time
///
/// # Example
///
/// ```ignore
/// use runtime2::engine::parse_full;
/// use runtime2::language::Language;
/// use runtime2::builder::forest_to_tree;
///
/// // Assume language is set up with parse table and tokenizer
/// let language = Language { /* ... */ };
/// let input = b"1 + 2 * 3";
///
/// // Parse to forest
/// let forest = parse_full(&language, input)?;
///
/// // Convert to tree (performs disambiguation)
/// let tree = forest_to_tree(forest);
/// assert_eq!(tree.root_node().kind(), "expr");
/// # Ok::<(), ParseError>(())
/// ```
///
/// # Errors
///
/// Returns `ParseError` in the following cases:
///
/// - **No parse table**: `language.parse_table.is_none()`
/// - **No tokenizer**: `language.tokenize.is_none()`
/// - **Syntax error**: Input doesn't match grammar
/// - **Feature disabled**: Compiled without `glr-core` feature
///
/// # Feature Gates
///
/// - Requires `#[cfg(feature = "glr-core")]`
/// - Without feature: returns error immediately
///
/// # See Also
///
/// - [`parse_incremental`]: Incremental parsing with old tree reuse
/// - [`Forest`]: Parse forest representation
/// - [`crate::builder::forest_to_tree`]: Forest-to-tree conversion
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

/// Parse input incrementally, reusing parts of the old tree (stub implementation)
///
/// **Current Status**: This is a stub implementation that performs full parsing.
/// True incremental parsing with subtree reuse is planned for a future release.
///
/// # Incremental Parsing Concept
///
/// Incremental parsing optimizes re-parsing after edits by reusing unchanged
/// subtrees from the previous parse. The algorithm works as follows:
///
/// 1. **Identify affected range**: Determine which part of the input changed
/// 2. **Find reusable subtrees**: Locate subtrees outside the affected range
/// 3. **Re-parse affected portion**: Parse only the changed section
/// 4. **Merge results**: Combine reused subtrees with newly parsed nodes
///
/// # Performance Benefits (Planned)
///
/// - **Reduced parsing time**: Only re-parse affected portions (typically 1-10%)
/// - **Lower memory allocation**: Reuse existing tree nodes
/// - **Faster editor responsiveness**: Sub-millisecond updates for small edits
///
/// # Current Behavior
///
/// This implementation currently:
/// - **Ignores** the `old_tree` parameter
/// - **Performs** full parsing via `parse_full()`
/// - **Returns** a complete new forest
///
/// # Future Implementation
///
/// The planned incremental algorithm will:
/// 1. Compute edit distance between old and new input
/// 2. Identify reusable subtrees using tree-sitter's `Tree::edit()` API
/// 3. Extract reusable portions from old forest
/// 4. Parse only the affected range
/// 5. Merge reused and new subtrees into unified forest
///
/// # Arguments
///
/// - `language`: Language definition with parse table and tokenizer
/// - `input`: New input bytes to parse
/// - `_old`: Previous parse tree (currently unused)
///
/// # Returns
///
/// - `Ok(Forest)`: Parse forest for the new input
/// - `Err(ParseError)`: If parsing fails
///
/// # Example
///
/// ```ignore
/// use runtime2::engine::{parse_full, parse_incremental};
/// use runtime2::builder::forest_to_tree;
///
/// let language = Language { /* ... */ };
///
/// // Initial parse
/// let input1 = b"let x = 1";
/// let forest1 = parse_full(&language, input1)?;
/// let tree1 = forest_to_tree(forest1);
///
/// // Incremental parse (currently performs full parse)
/// let input2 = b"let x = 2";
/// let forest2 = parse_incremental(&language, input2, &tree1)?;
/// let tree2 = forest_to_tree(forest2);
/// # Ok::<(), ParseError>(())
/// ```
///
/// # Migration Path
///
/// When true incremental parsing is implemented:
/// - This function signature will remain stable
/// - The `_old` parameter will be used for subtree reuse
/// - Performance will improve without API changes
/// - Existing code will automatically benefit
///
/// # See Also
///
/// - [`parse_full`]: Full parsing without incremental optimization
/// - [`Tree::edit`]: Tree-sitter edit API for tracking changes
#[allow(dead_code)]
pub fn parse_incremental(
    language: &Language,
    input: &[u8],
    _old: &Tree,
) -> Result<Forest, ParseError> {
    // Call the same path now; replace with proper reuse later.
    parse_full(language, input)
}
