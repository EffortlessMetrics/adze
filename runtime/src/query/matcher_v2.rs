// Enhanced query pattern matching with predicate evaluation
use super::ast::*;
use super::predicate_eval::PredicateContext;
use crate::parser_v4::ParseNode;
use std::collections::HashMap;

/// A match of a query pattern
#[derive(Debug, Clone)]
pub struct QueryMatch {
    /// Pattern index that matched
    pub pattern_index: usize,
    /// Captured nodes
    pub captures: Vec<QueryCapture>,
}

/// A captured node in a query match
#[derive(Debug, Clone)]
pub struct QueryCapture {
    /// Capture index
    pub index: u32,
    /// The captured node
    pub node: ParseNode,
}

/// State for matching a pattern
#[derive(Debug)]
struct MatchState {
    /// Current captures
    captures: HashMap<u32, ParseNode>,
}

/// Query pattern matcher with source text
pub struct QueryMatcher<'a> {
    query: &'a Query,
    source: &'a str,
}

impl<'a> QueryMatcher<'a> {
    /// Create a new query matcher with source text
    pub fn new(query: &'a Query, source: &'a str) -> Self {
        QueryMatcher { query, source }
    }

    /// Match all patterns in the query against a parse tree
    pub fn matches(&self, root: &ParseNode) -> Vec<QueryMatch> {
        let mut matches = Vec::new();

        // Try each pattern
        for (pattern_index, pattern) in self.query.patterns.iter().enumerate() {
            self.match_pattern(pattern_index, pattern, root, &mut matches);
        }

        matches
    }

    /// Match a single pattern against the tree
    fn match_pattern(
        &self,
        pattern_index: usize,
        pattern: &Pattern,
        root: &ParseNode,
        matches: &mut Vec<QueryMatch>,
    ) {
        // Walk the tree and try to match at each node
        self.match_pattern_at_node(pattern_index, pattern, root, matches);
    }

    /// Try to match pattern starting at a specific node
    fn match_pattern_at_node(
        &self,
        pattern_index: usize,
        pattern: &Pattern,
        node: &ParseNode,
        matches: &mut Vec<QueryMatch>,
    ) {
        // Try to match the pattern at this node
        let mut state = MatchState {
            captures: HashMap::new(),
        };

        if self.match_node(&pattern.root, node, &mut state) {
            // Check predicates with source text
            let predicate_ctx = PredicateContext::new(self.source);
            if pattern
                .predicates
                .iter()
                .all(|pred| predicate_ctx.evaluate(pred, &state.captures))
            {
                // Convert captures to vector
                let mut captures: Vec<_> = state
                    .captures
                    .into_iter()
                    .map(|(index, node)| QueryCapture { index, node })
                    .collect();
                captures.sort_by_key(|c| c.index);

                matches.push(QueryMatch {
                    pattern_index,
                    captures,
                });
            }
        }

        // Recursively try child nodes
        for child in &node.children {
            self.match_pattern_at_node(pattern_index, pattern, child, matches);
        }
    }

    /// Match a pattern node against a parse node
    fn match_node(&self, pattern: &PatternNode, node: &ParseNode, state: &mut MatchState) -> bool {
        // Check symbol
        if pattern.symbol != node.symbol {
            return false;
        }

        // TODO: Check if named/anonymous matches once node metadata is available

        // Capture if needed
        if let Some(capture_id) = pattern.capture {
            state.captures.insert(capture_id, node.clone());
        }

        // Match based on quantifier
        match pattern.quantifier {
            Quantifier::One => self.match_children_one(pattern, node, state),
            Quantifier::Optional => self.match_children_optional(pattern, node, state),
            Quantifier::Plus => self.match_children_plus(pattern, node, state),
            Quantifier::Star => self.match_children_star(pattern, node, state),
        }
    }

    /// Match children with One quantifier
    fn match_children_one(
        &self,
        pattern: &PatternNode,
        node: &ParseNode,
        state: &mut MatchState,
    ) -> bool {
        // Check field assertions
        for (field_name, field_pattern) in &pattern.fields {
            // Find child with this field name
            let field_node = node
                .children
                .iter()
                .find(|child| child.field_name.as_ref() == Some(field_name));

            if let Some(field_node) = field_node {
                if !self.match_node(field_pattern, field_node, state) {
                    return false;
                }
            } else {
                return false; // Required field not found
            }
        }

        // If pattern has explicit children, match them
        if !pattern.children.is_empty() {
            return self.match_child_sequence(&pattern.children, &node.children, 0, 0, state);
        }

        true
    }

    /// Match children with Optional quantifier
    fn match_children_optional(
        &self,
        pattern: &PatternNode,
        node: &ParseNode,
        state: &mut MatchState,
    ) -> bool {
        // Optional always matches, but we try to match if possible
        self.match_children_one(pattern, node, state);
        true
    }

    /// Match children with Plus quantifier
    fn match_children_plus(
        &self,
        pattern: &PatternNode,
        node: &ParseNode,
        state: &mut MatchState,
    ) -> bool {
        // Must match at least once
        if !self.match_children_one(pattern, node, state) {
            return false;
        }

        // Try to match more (simplified - in reality would need backtracking)
        true
    }

    /// Match children with Star quantifier
    fn match_children_star(
        &self,
        pattern: &PatternNode,
        node: &ParseNode,
        state: &mut MatchState,
    ) -> bool {
        // Star always matches (zero or more)
        self.match_children_plus(pattern, node, state);
        true
    }

    /// Match a sequence of pattern children against node children
    fn match_child_sequence(
        &self,
        pattern_children: &[PatternChild],
        node_children: &[ParseNode],
        pattern_idx: usize,
        node_idx: usize,
        state: &mut MatchState,
    ) -> bool {
        // Base case: all patterns matched
        if pattern_idx >= pattern_children.len() {
            // TODO: Check for extra nodes once metadata is available
            return true;
        }

        // Base case: no more nodes but patterns remain
        if node_idx >= node_children.len() {
            // Check if remaining patterns are all optional
            return pattern_children[pattern_idx..]
                .iter()
                .all(|p| matches!(p, PatternChild::Node(n) if n.quantifier != Quantifier::One));
        }

        // TODO: Skip extra nodes once metadata is available

        // Try to match current pattern
        match &pattern_children[pattern_idx] {
            PatternChild::Node(pattern_node) => {
                if self.match_node(pattern_node, &node_children[node_idx], state) {
                    // Pattern matched, continue with next
                    self.match_child_sequence(
                        pattern_children,
                        node_children,
                        pattern_idx + 1,
                        node_idx + 1,
                        state,
                    )
                } else if pattern_node.quantifier != Quantifier::One {
                    // Optional pattern, skip it
                    self.match_child_sequence(
                        pattern_children,
                        node_children,
                        pattern_idx + 1,
                        node_idx,
                        state,
                    )
                } else {
                    false
                }
            }
            PatternChild::Token(_token) => {
                // For now, assume tokens match (would need lexer info)
                self.match_child_sequence(
                    pattern_children,
                    node_children,
                    pattern_idx + 1,
                    node_idx + 1,
                    state,
                )
            }
        }
    }
}

/// Iterator over query matches
pub struct QueryMatches<'a> {
    matcher: QueryMatcher<'a>,
    root: &'a ParseNode,
    #[allow(dead_code)]
    pattern_index: usize,
    done: bool,
}

impl<'a> QueryMatches<'a> {
    /// Create a new query matches iterator
    pub fn new(query: &'a Query, root: &'a ParseNode, source: &'a str) -> Self {
        QueryMatches {
            matcher: QueryMatcher::new(query, source),
            root,
            pattern_index: 0,
            done: false,
        }
    }
}

impl<'a> Iterator for QueryMatches<'a> {
    type Item = QueryMatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Get all matches (simplified - real implementation would be incremental)
        let matches = self.matcher.matches(self.root);
        self.done = true;

        matches.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::SymbolId;

    fn make_node(symbol: u16, start: usize, end: usize) -> ParseNode {
        let symbol_id = SymbolId(symbol);
        ParseNode {
            symbol: symbol_id,
            symbol_id,
            children: vec![],
            start_byte: start,
            end_byte: end,
            field_name: None,
        }
    }

    #[test]
    fn test_predicate_matching() {
        // Create a simple query with predicates
        let query_str = r#"
            (identifier) @name
            (#eq? @name "test")
        "#;

        // Mock symbol IDs
        let identifier_symbol = SymbolId(1);

        // Create a mock query (normally would use compile_query)
        let mut query = Query {
            source: query_str.to_string(),
            patterns: vec![],
            capture_names: HashMap::new(),
            property_settings: vec![],
            property_predicates: vec![],
        };

        query.capture_names.insert("name".to_string(), 0);

        let pattern = Pattern {
            root: PatternNode {
                symbol: identifier_symbol,
                children: vec![],
                fields: HashMap::new(),
                capture: Some(0),
                is_named: true,
                quantifier: Quantifier::One,
            },
            predicates: vec![Predicate::Eq {
                capture1: 0,
                capture2: None,
                value: Some("test".to_string()),
            }],
            start_byte: 0,
        };

        query.patterns.push(pattern);

        // Create test tree
        let source = "test other test";
        let symbol_id = SymbolId(0);
        let root = ParseNode {
            symbol: symbol_id,
            symbol_id,
            children: vec![
                make_node(1, 0, 4),   // "test"
                make_node(1, 5, 10),  // "other"
                make_node(1, 11, 15), // "test"
            ],
            start_byte: 0,
            end_byte: 15,
            field_name: None,
        };

        // Match with predicates
        let matcher = QueryMatcher::new(&query, source);
        let matches = matcher.matches(&root);

        // Should match only the "test" identifiers
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].captures[0].node.start_byte, 0);
        assert_eq!(matches[1].captures[0].node.start_byte, 11);
    }
}
