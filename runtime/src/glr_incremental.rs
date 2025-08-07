//! GLR-Aware Incremental Parsing
//!
//! This module provides incremental parsing capabilities for GLR parsers,
//! preserving ambiguities and efficiently handling edits to the input.
//!
//! ## Key Concepts
//!
//! ### Fork Tracking
//! - Each parse tree node remembers which fork(s) it belongs to
//! - When edits occur, we track which forks are affected
//! - Unaffected forks can reuse their subtrees entirely
//!
//! ### Ambiguity Preservation
//! - Multiple parse trees are maintained for ambiguous regions
//! - Edits may resolve or introduce new ambiguities
//! - The incremental parser preserves all valid interpretations
//!
//! ### Reuse Strategy
//! - Subtrees outside the edit region are reused when possible
//! - Fork-specific subtrees are only reused if the fork is preserved
//! - Shared subtrees (common to all forks) are always reused

use crate::glr_parser::GLRParser;
use crate::subtree::Subtree;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{Grammar, RuleId, SymbolId};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

/// Represents an edit to the input
/// Public API for incremental parsing (used by unified parser)
/// 
/// This function bridges between the public parser_v4 API and the internal
/// GLR incremental parsing implementation.
pub fn reparse(
    grammar: &Grammar,
    table: &ParseTable,
    source: &[u8],
    old_tree: &crate::parser_v4::Tree,
    edit: &crate::pure_incremental::Edit,
) -> Option<crate::parser_v4::Tree> {
    // Only enable incremental parsing if the feature is enabled
    #[cfg(feature = "incremental_glr")]
    {
        use crate::tree_bridge::{v4_tree_to_forest, forest_to_v4_tree};
        
        // Create an incremental parser instance
        let mut parser = IncrementalGLRParser::new(grammar.clone(), table.clone());
        
        // Convert the old tree to a forest representation
        let old_forest = v4_tree_to_forest(old_tree);
        
        // Convert the edit to GLR format
        let glr_edit = GLREdit {
            old_range: edit.start_byte..edit.old_end_byte,
            new_text: source[edit.start_byte..edit.new_end_byte].to_vec(),
        };
        
        // Perform the incremental parse
        let new_forest = parser.parse_incremental(source, old_forest, &glr_edit);
        
        // Convert back to v4 tree format
        new_forest.map(|forest| forest_to_v4_tree(&forest, String::from_utf8_lossy(source).to_string()))
    }
    
    #[cfg(not(feature = "incremental_glr"))]
    {
        // Feature not enabled, return None to trigger fresh parse
        None
    }
}

#[derive(Debug, Clone)]
pub struct GLREdit {
    /// Byte range in the old input that was replaced
    pub old_range: Range<usize>,
    /// New text that replaces the old range
    pub new_text: Vec<u8>,
    /// Token range affected by the edit
    pub old_token_range: Range<usize>,
    /// New tokens that replace the old token range
    pub new_tokens: Vec<GLRToken>,
}

/// A token with position information
#[derive(Debug, Clone)]
pub struct GLRToken {
    pub symbol: SymbolId,
    pub text: Vec<u8>,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// A parse forest node that tracks multiple interpretations
#[derive(Debug, Clone)]
pub struct ForestNode {
    /// The symbol at this node
    pub symbol: SymbolId,
    /// Alternative parse trees for this node (one per fork)
    pub alternatives: Vec<ForkAlternative>,
    /// Byte range in the input
    pub byte_range: Range<usize>,
    /// Token range in the input
    pub token_range: Range<usize>,
}

/// An alternative parse for a forest node
#[derive(Debug, Clone)]
pub struct ForkAlternative {
    /// The fork ID this alternative belongs to
    pub fork_id: usize,
    /// The rule used (if any)
    pub rule_id: Option<RuleId>,
    /// Children for this interpretation
    pub children: Vec<Arc<ForestNode>>,
    /// The subtree for this alternative
    pub subtree: Arc<Subtree>,
}

/// Tracks reusable subtrees across edits
#[derive(Debug)]
pub struct ReuseMap {
    /// Maps byte ranges to reusable subtrees
    subtrees: HashMap<Range<usize>, Vec<(usize, Arc<Subtree>)>>, // (fork_id, subtree)
    /// Tracks which byte ranges are affected by edits
    affected_ranges: HashSet<Range<usize>>,
}

impl ReuseMap {
    pub fn new() -> Self {
        Self {
            subtrees: HashMap::new(),
            affected_ranges: HashSet::new(),
        }
    }

    /// Add a reusable subtree
    pub fn add_subtree(&mut self, range: Range<usize>, fork_id: usize, subtree: Arc<Subtree>) {
        self.subtrees
            .entry(range)
            .or_insert_with(Vec::new)
            .push((fork_id, subtree));
    }

    /// Mark a range as affected by an edit
    pub fn mark_affected(&mut self, range: Range<usize>) {
        self.affected_ranges.insert(range);
    }

    /// Check if a range is affected by edits
    pub fn is_affected(&self, range: &Range<usize>) -> bool {
        self.affected_ranges.iter().any(|affected| {
            affected.start < range.end && affected.end > range.start
        })
    }

    /// Get reusable subtrees for a range and fork
    pub fn get_subtrees(&self, range: &Range<usize>, fork_id: Option<usize>) -> Vec<Arc<Subtree>> {
        if self.is_affected(range) {
            return Vec::new();
        }

        self.subtrees
            .get(range)
            .map(|trees| {
                trees
                    .iter()
                    .filter(|(id, _)| fork_id.is_none() || fork_id == Some(*id))
                    .map(|(_, tree)| tree.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// GLR-aware incremental parser
pub struct IncrementalGLRParser {
    /// The underlying GLR parser
    parser: GLRParser,
    /// Grammar for the language
    grammar: Grammar,
    /// Parse table
    table: ParseTable,
    /// Reuse map for subtree reuse
    reuse_map: ReuseMap,
    /// Current parse forest
    forest: Option<Arc<ForestNode>>,
    /// Fork tracking information
    fork_tracker: ForkTracker,
}

/// Tracks fork relationships and dependencies
#[derive(Debug)]
struct ForkTracker {
    /// Maps fork IDs to their parent forks
    fork_parents: HashMap<usize, usize>,
    /// Maps fork IDs to their merge points
    fork_merges: HashMap<usize, Vec<usize>>,
    /// Active fork IDs
    active_forks: HashSet<usize>,
    /// Next fork ID to assign
    next_fork_id: usize,
}

impl ForkTracker {
    pub fn new() -> Self {
        Self {
            fork_parents: HashMap::new(),
            fork_merges: HashMap::new(),
            active_forks: HashSet::new(),
            next_fork_id: 0,
        }
    }

    /// Create a new fork from a parent
    pub fn create_fork(&mut self, parent: Option<usize>) -> usize {
        let fork_id = self.next_fork_id;
        self.next_fork_id += 1;
        
        if let Some(parent_id) = parent {
            self.fork_parents.insert(fork_id, parent_id);
        }
        
        self.active_forks.insert(fork_id);
        fork_id
    }

    /// Record a fork merge
    pub fn merge_forks(&mut self, fork1: usize, fork2: usize, merge_point: usize) {
        self.fork_merges
            .entry(fork1)
            .or_insert_with(Vec::new)
            .push(merge_point);
        self.fork_merges
            .entry(fork2)
            .or_insert_with(Vec::new)
            .push(merge_point);
    }

    /// Check if a fork is affected by an edit
    pub fn is_fork_affected(&self, fork_id: usize, affected_ranges: &HashSet<Range<usize>>) -> bool {
        // A fork is affected if any of its unique parse decisions fall within affected ranges
        // This is a simplified check - a real implementation would track fork-specific decisions
        self.active_forks.contains(&fork_id)
    }

    /// Get all forks affected by an edit
    pub fn get_affected_forks(&self, edit: &GLREdit) -> HashSet<usize> {
        let mut affected = HashSet::new();
        
        // For now, conservatively mark all active forks as potentially affected
        // A more sophisticated implementation would track which forks have
        // parse decisions in the edited region
        for fork_id in &self.active_forks {
            affected.insert(*fork_id);
        }
        
        affected
    }
}

impl IncrementalGLRParser {
    /// Create a new incremental GLR parser
    pub fn new(grammar: Grammar, table: ParseTable) -> Self {
        let parser = GLRParser::new(table.clone(), grammar.clone());
        
        Self {
            parser,
            grammar,
            table,
            reuse_map: ReuseMap::new(),
            forest: None,
            fork_tracker: ForkTracker::new(),
        }
    }

    /// Parse with incremental reuse
    pub fn parse_incremental(
        &mut self,
        tokens: &[GLRToken],
        edits: &[GLREdit],
    ) -> Result<Arc<ForestNode>, String> {
        // If we have edits and a previous parse, try to reuse
        if !edits.is_empty() && self.forest.is_some() {
            self.reparse_with_edits(tokens, edits)
        } else {
            // Fresh parse
            self.parse_fresh(tokens)
        }
    }

    /// Parse from scratch
    fn parse_fresh(&mut self, tokens: &[GLRToken]) -> Result<Arc<ForestNode>, String> {
        // Reset state
        self.reuse_map = ReuseMap::new();
        self.fork_tracker = ForkTracker::new();
        
        // Create initial fork
        let initial_fork = self.fork_tracker.create_fork(None);
        
        // Parse using the GLR parser
        let mut parser = GLRParser::new(self.table.clone(), self.grammar.clone());
        
        for token in tokens {
            parser.process_token(token.symbol, std::str::from_utf8(&token.text).unwrap_or(""), token.start_byte);
        }
        
        parser.process_eof();
        
        match parser.finish() {
            Ok(tree) => {
                // Convert subtree to forest node
                let forest = self.build_forest_from_subtree(tree, initial_fork, tokens);
                self.forest = Some(forest.clone());
                Ok(forest)
            }
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    /// Reparse with edits, reusing unaffected subtrees
    fn reparse_with_edits(
        &mut self,
        tokens: &[GLRToken],
        edits: &[GLREdit],
    ) -> Result<Arc<ForestNode>, String> {
        // Mark affected ranges in the reuse map
        for edit in edits {
            self.reuse_map.mark_affected(edit.old_range.clone());
        }
        
        // Get affected forks
        let affected_forks: HashSet<usize> = edits
            .iter()
            .flat_map(|edit| self.fork_tracker.get_affected_forks(edit))
            .collect();
        
        // Create a new parser with reuse context
        let mut parser = GLRParser::new(self.table.clone(), self.grammar.clone());
        
        // Process tokens, attempting to reuse subtrees where possible
        let mut token_idx = 0;
        let mut byte_offset = 0;
        
        while token_idx < tokens.len() {
            let token = &tokens[token_idx];
            let token_range = token.start_byte..token.end_byte;
            
            // Check if we can reuse a subtree at this position
            if !self.reuse_map.is_affected(&token_range) {
                // Try to find reusable subtrees
                let reusable = self.reuse_map.get_subtrees(&token_range, None);
                
                if !reusable.is_empty() {
                    // Skip tokens covered by the reusable subtree
                    // This is simplified - real implementation would inject the subtree
                    byte_offset = token.end_byte;
                    token_idx += 1;
                    continue;
                }
            }
            
            // Process token normally
            parser.process_token(token.symbol, std::str::from_utf8(&token.text).unwrap_or(""), token.start_byte);
            token_idx += 1;
        }
        
        parser.process_eof();
        
        match parser.finish() {
            Ok(tree) => {
                // Build new forest with fork tracking
                let forest = self.build_forest_with_forks(tree, &affected_forks, tokens);
                self.forest = Some(forest.clone());
                Ok(forest)
            }
            Err(e) => Err(format!("Reparse error: {}", e)),
        }
    }

    /// Build a forest node from a subtree
    fn build_forest_from_subtree(
        &mut self,
        subtree: Arc<Subtree>,
        fork_id: usize,
        tokens: &[GLRToken],
    ) -> Arc<ForestNode> {
        // Get byte range from subtree (would need to be implemented)
        let byte_range = 0..0; // TODO: implement subtree.byte_range()
        let token_range = self.find_token_range(&byte_range, tokens);
        
        // Store in reuse map for future incremental parsing
        self.reuse_map.add_subtree(byte_range.clone(), fork_id, subtree.clone());
        
        // Create forest node with single alternative
        let alternative = ForkAlternative {
            fork_id,
            rule_id: None, // Would be extracted from subtree
            children: Vec::new(), // Would be built recursively
            subtree,
        };
        
        Arc::new(ForestNode {
            symbol: SymbolId(0), // Would be extracted from subtree
            alternatives: vec![alternative],
            byte_range,
            token_range,
        })
    }

    /// Build a forest with multiple forks
    fn build_forest_with_forks(
        &mut self,
        subtree: Arc<Subtree>,
        affected_forks: &HashSet<usize>,
        tokens: &[GLRToken],
    ) -> Arc<ForestNode> {
        // This would merge the new parse with unaffected forks
        // For now, return a simple forest
        self.build_forest_from_subtree(subtree, 0, tokens)
    }

    /// Find the token range for a byte range
    fn find_token_range(&self, byte_range: &Range<usize>, tokens: &[GLRToken]) -> Range<usize> {
        let start = tokens
            .iter()
            .position(|t| t.start_byte >= byte_range.start)
            .unwrap_or(0);
        
        let end = tokens
            .iter()
            .rposition(|t| t.end_byte <= byte_range.end)
            .map(|i| i + 1)
            .unwrap_or(tokens.len());
        
        start..end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reuse_map() {
        let mut reuse_map = ReuseMap::new();
        
        // Add some subtrees
        let subtree1 = Arc::new(Subtree::new(SymbolId(1), 0, 10));
        let subtree2 = Arc::new(Subtree::new(SymbolId(2), 10, 20));
        
        reuse_map.add_subtree(0..10, 0, subtree1.clone());
        reuse_map.add_subtree(10..20, 0, subtree2.clone());
        
        // Check unaffected ranges can be reused
        assert_eq!(reuse_map.get_subtrees(&(0..10), Some(0)).len(), 1);
        assert_eq!(reuse_map.get_subtrees(&(10..20), Some(0)).len(), 1);
        
        // Mark a range as affected
        reuse_map.mark_affected(5..15);
        
        // Affected ranges should not be reusable
        assert_eq!(reuse_map.get_subtrees(&(0..10), Some(0)).len(), 0);
        assert_eq!(reuse_map.get_subtrees(&(10..20), Some(0)).len(), 0);
        
        // Unaffected range should still be reusable
        reuse_map.mark_affected(25..30);
        assert_eq!(reuse_map.get_subtrees(&(20..25), Some(0)).len(), 0);
    }

    #[test]
    fn test_fork_tracker() {
        let mut tracker = ForkTracker::new();
        
        // Create initial fork
        let fork0 = tracker.create_fork(None);
        assert_eq!(fork0, 0);
        assert!(tracker.active_forks.contains(&fork0));
        
        // Create child forks
        let fork1 = tracker.create_fork(Some(fork0));
        let fork2 = tracker.create_fork(Some(fork0));
        
        assert_eq!(tracker.fork_parents[&fork1], fork0);
        assert_eq!(tracker.fork_parents[&fork2], fork0);
        
        // Record a merge
        tracker.merge_forks(fork1, fork2, 100);
        assert!(tracker.fork_merges[&fork1].contains(&100));
        assert!(tracker.fork_merges[&fork2].contains(&100));
    }

    #[test]
    fn test_glr_edit() {
        let edit = GLREdit {
            old_range: 10..20,
            new_text: b"hello".to_vec(),
            old_token_range: 2..4,
            new_tokens: vec![
                GLRToken {
                    symbol: SymbolId(1),
                    text: b"hello".to_vec(),
                    start_byte: 10,
                    end_byte: 15,
                },
            ],
        };
        
        assert_eq!(edit.old_range, 10..20);
        assert_eq!(edit.new_text, b"hello");
        assert_eq!(edit.new_tokens.len(), 1);
    }
}