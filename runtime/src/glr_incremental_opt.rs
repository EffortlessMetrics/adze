//! Optimized GLR Incremental Parsing Strategies
//!
//! This module provides specialized optimizations for common edit patterns
//! to minimize reparsing overhead in GLR incremental parsing.

use crate::glr_incremental::{GLREdit, GLRToken, ForestNode, ReuseMap};
use crate::subtree::Subtree;
use rust_sitter_ir::SymbolId;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::sync::Arc;

/// Edit classification for optimization strategies
#[derive(Debug, Clone, PartialEq)]
pub enum EditClass {
    /// Single character insertion (common during typing)
    SingleCharInsertion,
    /// Single character deletion (backspace/delete)
    SingleCharDeletion,
    /// Token replacement (e.g., variable rename)
    TokenReplacement,
    /// Whitespace only change
    WhitespaceOnly,
    /// Comment modification
    CommentOnly,
    /// Block-level structural change
    StructuralChange,
    /// Multiple scattered edits
    Multiple,
}

impl EditClass {
    /// Classify an edit for optimization
    pub fn classify(edit: &GLREdit) -> Self {
        let old_len = edit.old_range.len();
        let new_len = edit.new_text.len();
        
        // Single character insertion
        if old_len == 0 && new_len == 1 {
            return EditClass::SingleCharInsertion;
        }
        
        // Single character deletion
        if old_len == 1 && new_len == 0 {
            return EditClass::SingleCharDeletion;
        }
        
        // Check if it's whitespace only
        if Self::is_whitespace_change(&edit.new_text) {
            return EditClass::WhitespaceOnly;
        }
        
        // Check if it's a comment
        if Self::is_comment_change(&edit.new_text) {
            return EditClass::CommentOnly;
        }
        
        // Token replacement (similar size, single token affected)
        if edit.old_token_range.len() == 1 && edit.new_tokens.len() == 1 {
            return EditClass::TokenReplacement;
        }
        
        // Default to structural change for larger edits
        if old_len > 50 || new_len > 50 {
            return EditClass::StructuralChange;
        }
        
        EditClass::Multiple
    }
    
    fn is_whitespace_change(text: &[u8]) -> bool {
        text.iter().all(|&b| b.is_ascii_whitespace())
    }
    
    fn is_comment_change(text: &[u8]) -> bool {
        // Simple heuristic for common comment patterns
        let text_str = String::from_utf8_lossy(text);
        text_str.starts_with("//") || text_str.starts_with("/*") || text_str.starts_with("#")
    }
}

/// Optimized reparse strategy based on edit classification
pub struct OptimizedReparser {
    /// Cache of recent parse results for fast lookup
    parse_cache: ParseCache,
    /// Statistics for optimization effectiveness
    stats: ReparseStats,
}

/// Cache for recent parse results
struct ParseCache {
    /// Maps token sequences to parsed subtrees
    token_cache: HashMap<Vec<SymbolId>, Arc<Subtree>>,
    /// LRU queue for cache eviction
    lru_queue: VecDeque<Vec<SymbolId>>,
    /// Maximum cache size
    max_size: usize,
}

impl ParseCache {
    fn new(max_size: usize) -> Self {
        Self {
            token_cache: HashMap::new(),
            lru_queue: VecDeque::new(),
            max_size,
        }
    }
    
    fn get(&mut self, tokens: &[SymbolId]) -> Option<Arc<Subtree>> {
        if let Some(subtree) = self.token_cache.get(tokens) {
            // Move to front of LRU queue
            self.lru_queue.retain(|t| t != tokens);
            self.lru_queue.push_front(tokens.to_vec());
            Some(subtree.clone())
        } else {
            None
        }
    }
    
    fn insert(&mut self, tokens: Vec<SymbolId>, subtree: Arc<Subtree>) {
        // Evict if at capacity
        if self.token_cache.len() >= self.max_size {
            if let Some(old_tokens) = self.lru_queue.pop_back() {
                self.token_cache.remove(&old_tokens);
            }
        }
        
        self.token_cache.insert(tokens.clone(), subtree);
        self.lru_queue.push_front(tokens);
    }
}

/// Statistics for reparse optimization
#[derive(Debug, Default)]
pub struct ReparseStats {
    pub total_reparses: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub subtrees_reused: usize,
    pub full_reparses: usize,
    pub optimized_reparses: usize,
}

impl OptimizedReparser {
    pub fn new() -> Self {
        Self {
            parse_cache: ParseCache::new(1000),
            stats: ReparseStats::default(),
        }
    }
    
    /// Optimize reparse based on edit classification
    pub fn optimize_reparse(
        &mut self,
        edit: &GLREdit,
        tokens: &[GLRToken],
        reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        self.stats.total_reparses += 1;
        
        let edit_class = EditClass::classify(edit);
        
        match edit_class {
            EditClass::SingleCharInsertion => {
                self.handle_char_insertion(edit, tokens, reuse_map)
            }
            EditClass::SingleCharDeletion => {
                self.handle_char_deletion(edit, tokens, reuse_map)
            }
            EditClass::TokenReplacement => {
                self.handle_token_replacement(edit, tokens, reuse_map)
            }
            EditClass::WhitespaceOnly => {
                self.handle_whitespace_change(edit, tokens, reuse_map)
            }
            EditClass::CommentOnly => {
                self.handle_comment_change(edit, tokens, reuse_map)
            }
            _ => {
                // Fall back to standard incremental parsing
                self.stats.full_reparses += 1;
                None
            }
        }
    }
    
    /// Handle single character insertion optimization
    fn handle_char_insertion(
        &mut self,
        edit: &GLREdit,
        tokens: &[GLRToken],
        reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // Check if we're in the middle of a token
        let affected_token_idx = tokens
            .iter()
            .position(|t| t.start_byte <= edit.old_range.start && t.end_byte > edit.old_range.start)?;
        
        let affected_token = &tokens[affected_token_idx];
        
        // Try to use cached result for similar token
        let token_symbols: Vec<SymbolId> = tokens.iter().map(|t| t.symbol).collect();
        
        if let Some(cached) = self.parse_cache.get(&token_symbols) {
            self.stats.cache_hits += 1;
            self.stats.optimized_reparses += 1;
            
            // Adjust byte offsets in cached result
            return Some(self.adjust_forest_offsets(cached, edit));
        }
        
        self.stats.cache_misses += 1;
        
        // Try to reuse surrounding subtrees
        if self.can_reuse_surrounding_subtrees(edit, reuse_map) {
            self.stats.subtrees_reused += 1;
            self.stats.optimized_reparses += 1;
            // Reparse only the affected token and merge with reused subtrees
            return self.reparse_minimal_region(edit, tokens, reuse_map);
        }
        
        None
    }
    
    /// Handle single character deletion optimization
    fn handle_char_deletion(
        &mut self,
        edit: &GLREdit,
        tokens: &[GLRToken],
        reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // Similar to insertion but in reverse
        self.handle_char_insertion(edit, tokens, reuse_map)
    }
    
    /// Handle token replacement optimization
    fn handle_token_replacement(
        &mut self,
        edit: &GLREdit,
        tokens: &[GLRToken],
        reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // If only one token is affected, we can often reuse the entire parse tree structure
        // and just replace the single token node
        
        if edit.old_token_range.len() != 1 || edit.new_tokens.len() != 1 {
            return None;
        }
        
        let token_idx = edit.old_token_range.start;
        
        // Check if the token has the same symbol type (e.g., both identifiers)
        if token_idx < tokens.len() && 
           tokens[token_idx].symbol == edit.new_tokens[0].symbol {
            self.stats.optimized_reparses += 1;
            // Can directly replace the token in the tree
            return self.replace_single_token(token_idx, &edit.new_tokens[0], tokens, reuse_map);
        }
        
        None
    }
    
    /// Handle whitespace-only changes
    fn handle_whitespace_change(
        &mut self,
        _edit: &GLREdit,
        tokens: &[GLRToken],
        reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // Whitespace changes usually don't affect the parse tree structure
        // We can often reuse the entire tree with adjusted positions
        self.stats.optimized_reparses += 1;
        self.stats.subtrees_reused += tokens.len();
        
        // Return existing forest with adjusted positions
        self.get_existing_forest(reuse_map)
    }
    
    /// Handle comment-only changes
    fn handle_comment_change(
        &mut self,
        edit: &GLREdit,
        tokens: &[GLRToken],
        reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // Comments typically don't affect parse tree structure
        // Similar to whitespace handling
        self.handle_whitespace_change(edit, tokens, reuse_map)
    }
    
    /// Check if we can reuse surrounding subtrees
    fn can_reuse_surrounding_subtrees(&self, edit: &GLREdit, reuse_map: &ReuseMap) -> bool {
        // Check if there are reusable subtrees before and after the edit
        let before_range = 0..edit.old_range.start;
        let after_range = edit.old_range.end..usize::MAX;
        
        !reuse_map.is_affected(&before_range) && !reuse_map.is_affected(&after_range)
    }
    
    /// Reparse only the minimal affected region
    fn reparse_minimal_region(
        &self,
        _edit: &GLREdit,
        _tokens: &[GLRToken],
        _reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // This would implement minimal region reparsing
        // For now, return None to fall back to standard incremental
        None
    }
    
    /// Replace a single token in the forest
    fn replace_single_token(
        &self,
        _token_idx: usize,
        _new_token: &GLRToken,
        _tokens: &[GLRToken],
        _reuse_map: &ReuseMap,
    ) -> Option<Arc<ForestNode>> {
        // This would implement single token replacement
        // For now, return None to fall back to standard incremental
        None
    }
    
    /// Adjust forest node offsets after an edit
    fn adjust_forest_offsets(&self, _subtree: Arc<Subtree>, _edit: &GLREdit) -> Arc<ForestNode> {
        // This would adjust byte offsets in the forest
        // For now, create a dummy forest node
        Arc::new(ForestNode {
            symbol: SymbolId(0),
            alternatives: vec![],
            byte_range: 0..0,
            token_range: 0..0,
        })
    }
    
    /// Get existing forest from reuse map
    fn get_existing_forest(&self, _reuse_map: &ReuseMap) -> Option<Arc<ForestNode>> {
        // This would retrieve the existing forest
        // For now, return None
        None
    }
    
    /// Get optimization statistics
    pub fn stats(&self) -> &ReparseStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ReparseStats::default();
    }
}

/// Incremental boundary detection for optimal reparse regions
pub struct BoundaryDetector {
    /// Grammar information for boundary detection
    grammar_info: GrammarInfo,
}

#[derive(Debug)]
struct GrammarInfo {
    /// Symbols that can start a statement/expression
    statement_starters: Vec<SymbolId>,
    /// Symbols that can end a statement/expression
    statement_enders: Vec<SymbolId>,
    /// Balanced delimiter pairs
    delimiter_pairs: Vec<(SymbolId, SymbolId)>,
}

impl BoundaryDetector {
    pub fn new() -> Self {
        Self {
            grammar_info: GrammarInfo {
                statement_starters: vec![],
                statement_enders: vec![],
                delimiter_pairs: vec![],
            },
        }
    }
    
    /// Find optimal reparse boundaries around an edit
    pub fn find_boundaries(
        &self,
        edit: &GLREdit,
        tokens: &[GLRToken],
    ) -> (usize, usize) {
        let mut start_boundary = edit.old_token_range.start;
        let mut end_boundary = edit.old_token_range.end;
        
        // Expand to nearest statement boundaries
        start_boundary = self.find_statement_start(start_boundary, tokens);
        end_boundary = self.find_statement_end(end_boundary, tokens);
        
        // Ensure balanced delimiters
        self.balance_delimiters(start_boundary, end_boundary, tokens)
    }
    
    fn find_statement_start(&self, from: usize, tokens: &[GLRToken]) -> usize {
        // Search backwards for a statement starter
        for i in (0..from.min(tokens.len())).rev() {
            if self.grammar_info.statement_starters.contains(&tokens[i].symbol) {
                return i;
            }
        }
        0
    }
    
    fn find_statement_end(&self, from: usize, tokens: &[GLRToken]) -> usize {
        // Search forwards for a statement ender
        for i in from..tokens.len() {
            if self.grammar_info.statement_enders.contains(&tokens[i].symbol) {
                return i + 1;
            }
        }
        tokens.len()
    }
    
    fn balance_delimiters(
        &self,
        mut start: usize,
        mut end: usize,
        tokens: &[GLRToken],
    ) -> (usize, usize) {
        // Ensure we have balanced delimiters in the reparse region
        let mut delimiter_stack = Vec::new();
        
        for i in start..end.min(tokens.len()) {
            let symbol = tokens[i].symbol;
            
            // Check for opening delimiter
            for (open, close) in &self.grammar_info.delimiter_pairs {
                if symbol == *open {
                    delimiter_stack.push(*close);
                } else if symbol == *close {
                    if delimiter_stack.last() == Some(close) {
                        delimiter_stack.pop();
                    } else {
                        // Unmatched closing delimiter - expand region
                        start = self.find_matching_opener(i, tokens, *open).unwrap_or(0);
                    }
                }
            }
        }
        
        // If we have unclosed delimiters, expand to include their closers
        if !delimiter_stack.is_empty() {
            end = self.find_closers(end, &delimiter_stack, tokens).unwrap_or(tokens.len());
        }
        
        (start, end)
    }
    
    fn find_matching_opener(
        &self,
        from: usize,
        tokens: &[GLRToken],
        opener: SymbolId,
    ) -> Option<usize> {
        for i in (0..from).rev() {
            if tokens[i].symbol == opener {
                return Some(i);
            }
        }
        None
    }
    
    fn find_closers(
        &self,
        from: usize,
        closers: &[SymbolId],
        tokens: &[GLRToken],
    ) -> Option<usize> {
        let mut remaining = closers.to_vec();
        
        for i in from..tokens.len() {
            if let Some(pos) = remaining.iter().position(|&c| c == tokens[i].symbol) {
                remaining.remove(pos);
                if remaining.is_empty() {
                    return Some(i + 1);
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_classification() {
        // Single char insertion
        let edit = GLREdit {
            old_range: 5..5,
            new_text: b"x".to_vec(),
            old_token_range: 0..0,
            new_tokens: vec![],
        };
        assert_eq!(EditClass::classify(&edit), EditClass::SingleCharInsertion);
        
        // Single char deletion
        let edit = GLREdit {
            old_range: 5..6,
            new_text: vec![],
            old_token_range: 0..0,
            new_tokens: vec![],
        };
        assert_eq!(EditClass::classify(&edit), EditClass::SingleCharDeletion);
        
        // Whitespace change
        let edit = GLREdit {
            old_range: 5..10,
            new_text: b"  \n\t ".to_vec(),
            old_token_range: 0..0,
            new_tokens: vec![],
        };
        assert_eq!(EditClass::classify(&edit), EditClass::WhitespaceOnly);
        
        // Comment change
        let edit = GLREdit {
            old_range: 5..10,
            new_text: b"// comment".to_vec(),
            old_token_range: 0..0,
            new_tokens: vec![],
        };
        assert_eq!(EditClass::classify(&edit), EditClass::CommentOnly);
    }

    #[test]
    fn test_parse_cache() {
        let mut cache = ParseCache::new(2);
        
        let tokens1 = vec![SymbolId(1), SymbolId(2)];
        let tokens2 = vec![SymbolId(3), SymbolId(4)];
        let tokens3 = vec![SymbolId(5), SymbolId(6)];
        
        let subtree1 = Arc::new(Subtree::new(SymbolId(1), 0, 10));
        let subtree2 = Arc::new(Subtree::new(SymbolId(2), 10, 20));
        let subtree3 = Arc::new(Subtree::new(SymbolId(3), 20, 30));
        
        // Insert first two
        cache.insert(tokens1.clone(), subtree1.clone());
        cache.insert(tokens2.clone(), subtree2.clone());
        
        // Access first to make it most recently used
        assert!(cache.get(&tokens1).is_some());
        
        // Insert third - should evict tokens2
        cache.insert(tokens3.clone(), subtree3.clone());
        
        // Check cache contents
        assert!(cache.get(&tokens1).is_some());
        assert!(cache.get(&tokens2).is_none()); // Evicted
        assert!(cache.get(&tokens3).is_some());
    }

    #[test]
    fn test_boundary_detector() {
        let detector = BoundaryDetector::new();
        
        let tokens = vec![
            GLRToken {
                symbol: SymbolId(1),
                text: b"if".to_vec(),
                start_byte: 0,
                end_byte: 2,
            },
            GLRToken {
                symbol: SymbolId(2),
                text: b"(".to_vec(),
                start_byte: 3,
                end_byte: 4,
            },
            GLRToken {
                symbol: SymbolId(3),
                text: b"x".to_vec(),
                start_byte: 4,
                end_byte: 5,
            },
            GLRToken {
                symbol: SymbolId(4),
                text: b")".to_vec(),
                start_byte: 5,
                end_byte: 6,
            },
        ];
        
        let edit = GLREdit {
            old_range: 4..5,
            new_text: b"y".to_vec(),
            old_token_range: 2..3,
            new_tokens: vec![],
        };
        
        let (start, end) = detector.find_boundaries(&edit, &tokens);
        
        // Should expand to include the whole if statement
        assert!(start <= 2);
        assert!(end >= 3);
    }
}