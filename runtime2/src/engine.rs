//! Engine adapter for GLR-core integration

use crate::{error::ParseError, language::Language, tree::Tree};
use rust_sitter_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use rust_sitter_ir::Grammar;
use std::sync::Arc;

/// GLR parser engine wrapper
pub struct GlrEngine {
    parse_table: ParseTable,
    grammar: Grammar,
    first_follow: FirstFollowSets,
}

impl GlrEngine {
    /// Create a new GLR engine from a language
    pub fn new(language: &Language) -> Result<Self, ParseError> {
        // For now, we need the grammar to be provided separately
        // In the final integration, this would come from the Language struct
        Err(ParseError::with_msg("GLR engine requires grammar - not yet wired"))
    }
    
    /// Create engine from grammar (for testing)
    pub fn from_grammar(grammar: Grammar) -> Result<Self, ParseError> {
        let first_follow = FirstFollowSets::compute(&grammar);
        let parse_table = build_lr1_automaton(&grammar, &first_follow)
            .map_err(|e| ParseError::with_msg(&format!("Failed to build parse table: {}", e)))?;
        
        Ok(Self {
            parse_table,
            grammar,
            first_follow,
        })
    }
    
    /// Parse input text
    pub fn parse(&mut self, input: &[u8]) -> Result<Tree, ParseError> {
        // We need to adapt the existing GLRParser from runtime/src/glr_parser.rs
        // For now, return a stub
        Err(ParseError::with_msg("GLR parsing not yet implemented in adapter"))
    }
    
    /// Parse with old tree for incremental parsing
    pub fn parse_incremental(&mut self, input: &[u8], old_tree: &Tree) -> Result<Tree, ParseError> {
        // Incremental parsing will reuse portions of old_tree
        let _ = old_tree;
        self.parse(input)
    }
    
    /// Get the parse table for direct access
    pub fn parse_table(&self) -> &ParseTable {
        &self.parse_table
    }
    
    /// Get the grammar
    pub fn grammar(&self) -> &Grammar {
        &self.grammar
    }
}

/// Forest representation for GLR parse results
pub struct Forest {
    /// The actual forest implementation will depend on GLR parser
    inner: ForestInner,
}

enum ForestInner {
    /// Placeholder for when we have the actual GLR forest
    Stub,
    /// Actual GLR parse forest when available
    #[allow(dead_code)]
    Glr(Arc<dyn std::any::Any + Send + Sync>),
}

impl Forest {
    /// Create a stub forest for testing
    pub fn stub() -> Self {
        Self {
            inner: ForestInner::Stub,
        }
    }
}

/// Convert a forest to a Tree
pub fn forest_to_tree(forest: Forest) -> Tree {
    match forest.inner {
        ForestInner::Stub => {
            // Return a placeholder tree
            Tree::new_stub()
        }
        ForestInner::Glr(_) => {
            // TODO: Convert actual GLR forest to tree
            Tree::new_stub()
        }
    }
}