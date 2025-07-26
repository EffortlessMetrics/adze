// Incremental parsing support for GLR parser
// This module provides efficient reparsing of edited documents

use crate::subtree::Subtree;
use crate::glr_parser::GLRParser;
use crate::glr_lexer::TokenWithPosition;
use rust_sitter_ir::{Grammar, SymbolId};
use std::sync::Arc;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Represents a text edit operation
#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    /// Starting byte offset of the edit
    pub start_byte: usize,
    /// Old ending byte offset (before the edit)
    pub old_end_byte: usize,
    /// New ending byte offset (after the edit)
    pub new_end_byte: usize,
    /// Starting position (line, column)
    pub start_position: Position,
    /// Old ending position
    pub old_end_position: Position,
    /// New ending position
    pub new_end_position: Position,
}

/// Position in text (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Edit {
    /// Create a simple edit from byte ranges
    pub fn new(start_byte: usize, old_end_byte: usize, new_end_byte: usize) -> Self {
        Edit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: Position { line: 0, column: 0 },
            old_end_position: Position { line: 0, column: 0 },
            new_end_position: Position { line: 0, column: 0 },
        }
    }

    /// Apply this edit to adjust a byte offset
    pub fn apply_to_offset(&self, offset: usize) -> Option<usize> {
        if offset < self.start_byte {
            Some(offset)
        } else if offset < self.old_end_byte {
            None // Offset is within the edited region
        } else {
            let delta = self.new_end_byte as isize - self.old_end_byte as isize;
            Some((offset as isize + delta) as usize)
        }
    }

    /// Check if this edit affects a given byte range
    pub fn affects_range(&self, start: usize, end: usize) -> bool {
        !(end <= self.start_byte || start >= self.old_end_byte)
    }
}

/// Pool of reusable subtrees from previous parse
pub struct SubtreePool {
    /// Subtrees indexed by their hash
    subtrees_by_hash: HashMap<u64, Vec<Arc<Subtree>>>,
    /// Subtrees indexed by byte range
    subtrees_by_range: HashMap<(usize, usize), Vec<Arc<Subtree>>>,
    /// Grammar for symbol lookup
    #[allow(dead_code)]
    grammar: Arc<Grammar>,
}

impl SubtreePool {
    pub fn new(grammar: Arc<Grammar>) -> Self {
        SubtreePool {
            subtrees_by_hash: HashMap::new(),
            subtrees_by_range: HashMap::new(),
            grammar,
        }
    }

    /// Add all subtrees from a parse tree to the pool
    pub fn add_tree(&mut self, root: Arc<Subtree>) {
        self.add_subtree_recursive(root);
    }

    fn add_subtree_recursive(&mut self, subtree: Arc<Subtree>) {
        // Calculate hash for this subtree
        let hash = self.hash_subtree(&subtree);
        
        // Index by hash
        self.subtrees_by_hash
            .entry(hash)
            .or_insert_with(Vec::new)
            .push(subtree.clone());
        
        // Index by range
        let range = (subtree.node.byte_range.start, subtree.node.byte_range.end);
        self.subtrees_by_range
            .entry(range)
            .or_insert_with(Vec::new)
            .push(subtree.clone());
        
        // Recursively add children
        for child in &subtree.children {
            self.add_subtree_recursive(child.clone());
        }
    }

    /// Hash a subtree for comparison
    fn hash_subtree(&self, subtree: &Subtree) -> u64 {
        let mut hasher = DefaultHasher::new();
        subtree.node.symbol_id.hash(&mut hasher);
        subtree.node.byte_range.start.hash(&mut hasher);
        subtree.node.byte_range.end.hash(&mut hasher);
        
        // Hash children
        for child in &subtree.children {
            self.hash_subtree(child).hash(&mut hasher);
        }
        
        hasher.finish()
    }

    /// Find a reusable subtree at the given position
    pub fn find_reusable(&self, position: usize, symbol: SymbolId) -> Option<Arc<Subtree>> {
        // Look for subtrees that start at this position
        for ((start, _), subtrees) in &self.subtrees_by_range {
            if *start == position {
                for subtree in subtrees {
                    if subtree.node.symbol_id == symbol {
                        return Some(subtree.clone());
                    }
                }
            }
        }
        None
    }

    /// Mark subtrees affected by an edit as invalid
    pub fn invalidate_edit(&mut self, edit: &Edit) {
        // Remove subtrees that overlap with the edit
        self.subtrees_by_range.retain(|(start, end), _| {
            !edit.affects_range(*start, *end)
        });
        
        // Adjust positions of subtrees after the edit
        let mut adjusted_subtrees = HashMap::new();
        for ((start, end), subtrees) in self.subtrees_by_range.drain() {
            if let (Some(new_start), Some(new_end)) = 
                (edit.apply_to_offset(start), edit.apply_to_offset(end)) {
                adjusted_subtrees.insert((new_start, new_end), subtrees);
            }
        }
        self.subtrees_by_range = adjusted_subtrees;
        
        // Clear hash index (will be rebuilt on next parse)
        self.subtrees_by_hash.clear();
    }
}

/// Incremental GLR parser that reuses subtrees from previous parses
pub struct IncrementalGLRParser {
    /// The underlying GLR parser
    parser: GLRParser,
    /// Pool of reusable subtrees
    subtree_pool: SubtreePool,
    /// Grammar reference
    grammar: Arc<Grammar>,
    /// Statistics for reuse tracking
    stats: ReuseStats,
}

#[derive(Debug, Default)]
pub struct ReuseStats {
    pub subtrees_reused: usize,
    pub bytes_reused: usize,
    pub total_bytes: usize,
}

impl IncrementalGLRParser {
    pub fn new(parser: GLRParser, grammar: Arc<Grammar>) -> Self {
        let subtree_pool = SubtreePool::new(grammar.clone());
        IncrementalGLRParser {
            parser,
            subtree_pool,
            grammar,
            stats: ReuseStats::default(),
        }
    }

    /// Parse with incremental support, reusing subtrees from previous parse
    pub fn parse_incremental(
        &mut self,
        tokens: &[TokenWithPosition],
        edits: &[Edit],
        previous_tree: Option<Arc<Subtree>>,
    ) -> Result<Arc<Subtree>, String> {
        // Reset statistics
        self.stats = ReuseStats::default();
        
        // If we have a previous tree, add it to the pool
        if let Some(tree) = previous_tree {
            self.subtree_pool.add_tree(tree);
        }
        
        // Apply edits to invalidate affected subtrees
        for edit in edits {
            self.subtree_pool.invalidate_edit(edit);
        }
        
        // Parse with subtree reuse
        self.parse_with_reuse(tokens)
    }

    fn parse_with_reuse(&mut self, tokens: &[TokenWithPosition]) -> Result<Arc<Subtree>, String> {
        let mut token_index = 0;
        self.stats.total_bytes = tokens.last()
            .map(|t| t.byte_offset + t.byte_length)
            .unwrap_or(0);
        
        // Reset parser state
        self.parser.reset();
        
        // Enable subtree reuse now that inject_subtree properly handles reductions
        let enable_reuse = true;
        
        while token_index < tokens.len() {
            let token = &tokens[token_index];
            
            // Check if we can reuse a subtree at this position
            if enable_reuse {
                if let Some(reusable) = self.try_reuse_subtree(token.byte_offset, &tokens[token_index..]) {
                    // Skip tokens covered by the reused subtree
                    let end_byte = reusable.node.byte_range.end;
                    self.stats.subtrees_reused += 1;
                    self.stats.bytes_reused += end_byte - token.byte_offset;
                    
                    // Inject the reused subtree into the parser
                    self.parser.inject_subtree(reusable);
                    
                    // Skip to the next token after the reused subtree
                    while token_index < tokens.len() && tokens[token_index].byte_offset < end_byte {
                        token_index += 1;
                    }
                    continue;
                }
            }
            
            // Normal parsing for this token
            self.parser.process_token(token.symbol_id, &token.text, token.byte_offset);
            token_index += 1;
        }
        
        // Process EOF and finalize parsing
        self.parser.process_eof();
        self.parser.finish()
    }

    fn try_reuse_subtree(&self, position: usize, remaining_tokens: &[TokenWithPosition]) -> Option<Arc<Subtree>> {
        // For now, we'll use a simple heuristic: try to reuse subtrees that match
        // the expected symbol at this position
        if remaining_tokens.is_empty() {
            return None;
        }
        
        // Get expected symbols from parser state
        let expected_symbols = self.parser.expected_symbols();
        
        // Try to find a reusable subtree for each expected symbol
        for symbol in expected_symbols {
            if let Some(subtree) = self.subtree_pool.find_reusable(position, symbol) {
                // Verify that the subtree matches the tokens
                if self.verify_subtree_matches(&subtree, remaining_tokens) {
                    return Some(subtree);
                }
            }
        }
        
        None
    }

    fn verify_subtree_matches(&self, subtree: &Subtree, tokens: &[TokenWithPosition]) -> bool {
        // Check if the subtree's text matches the tokens it would cover
        let subtree_end = subtree.node.byte_range.end;
        let mut current_byte = subtree.node.byte_range.start;
        
        for token in tokens {
            if token.byte_offset >= subtree_end {
                break;
            }
            
            if token.byte_offset != current_byte {
                return false; // Gap in tokens
            }
            
            current_byte = token.byte_offset + token.byte_length;
        }
        
        current_byte >= subtree_end
    }

    /// Get reuse statistics from the last parse
    pub fn stats(&self) -> &ReuseStats {
        &self.stats
    }

    /// Clear the subtree pool
    pub fn clear_pool(&mut self) {
        self.subtree_pool = SubtreePool::new(self.grammar.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_affects_range() {
        let edit = Edit::new(10, 15, 20);
        
        assert!(!edit.affects_range(0, 10));   // Before edit
        assert!(edit.affects_range(5, 12));    // Overlaps start
        assert!(edit.affects_range(12, 18));   // Within edit
        assert!(edit.affects_range(14, 20));   // Overlaps end
        assert!(!edit.affects_range(15, 25));  // After edit
    }

    #[test]
    fn test_edit_apply_to_offset() {
        let edit = Edit::new(10, 15, 20); // Insert 5 bytes
        
        assert_eq!(edit.apply_to_offset(5), Some(5));    // Before edit
        assert_eq!(edit.apply_to_offset(12), None);      // Within edit
        assert_eq!(edit.apply_to_offset(20), Some(25));  // After edit
    }

    #[test]
    fn test_subtree_pool_invalidation() {
        let grammar = Arc::new(Grammar::new("test".to_string()));
        let mut pool = SubtreePool::new(grammar);
        
        // Add some dummy entries
        pool.subtrees_by_range.insert((5, 10), vec![]);
        pool.subtrees_by_range.insert((15, 20), vec![]);
        pool.subtrees_by_range.insert((25, 30), vec![]);
        
        // Apply edit that affects middle range
        let edit = Edit::new(12, 18, 22);
        pool.invalidate_edit(&edit);
        
        // Check that unaffected ranges are preserved and adjusted
        assert!(pool.subtrees_by_range.contains_key(&(5, 10)));    // Unaffected
        assert!(!pool.subtrees_by_range.contains_key(&(15, 20)));  // Removed
        assert!(pool.subtrees_by_range.contains_key(&(29, 34)));   // Adjusted
    }
}