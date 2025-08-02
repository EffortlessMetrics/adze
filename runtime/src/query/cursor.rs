// Query cursor for efficient matching
use super::{Query, QueryMatch, QueryMatches};
use crate::parser_v3::ParseNode;
use std::ops::Range;

/// A stateful object for executing queries on a syntax tree
pub struct QueryCursor {
    /// Byte range to restrict matches
    byte_range: Option<Range<usize>>,
    /// Whether to match only the root node
    match_root: bool,
}

impl QueryCursor {
    /// Create a new query cursor
    pub fn new() -> Self {
        QueryCursor {
            byte_range: None,
            match_root: false,
        }
    }

    /// Set the byte range for matching
    pub fn set_byte_range(&mut self, range: Range<usize>) {
        self.byte_range = Some(range);
    }

    /// Clear the byte range restriction
    pub fn clear_byte_range(&mut self) {
        self.byte_range = None;
    }

    /// Set whether to match only at the root
    pub fn set_match_root(&mut self, match_root: bool) {
        self.match_root = match_root;
    }

    /// Execute a query and return all matches
    pub fn matches<'a>(&'a mut self, query: &'a Query, root: &'a ParseNode) -> QueryMatches<'a> {
        QueryMatches::new(query, root)
    }

    /// Execute a query and collect all matches into a vector
    pub fn collect_matches(&mut self, query: &Query, root: &ParseNode) -> Vec<QueryMatch> {
        self.matches(query, root).collect()
    }

    /// Check if a node is within the configured byte range
    #[allow(dead_code)]
    fn is_in_range(&self, node: &ParseNode) -> bool {
        if let Some(ref range) = self.byte_range {
            node.start_byte >= range.start && node.end_byte <= range.end
        } else {
            true
        }
    }
}

impl Default for QueryCursor {
    fn default() -> Self {
        Self::new()
    }
}
