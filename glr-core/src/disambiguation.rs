use crate::parse_forest::{ForestNode, ParseError, ParseForest, ParseNode, ParseTree};

impl ParseForest {
    /// Convert forest to single tree - for now just pick first complete parse
    pub fn to_single_tree(self) -> Result<ParseTree, ParseError> {
        // Find all complete parses (reached EOF at root symbol)
        let complete_parses: Vec<_> = self
            .roots
            .iter()
            .filter(|r| {
                if let Some(start) = self.grammar.start_symbol() {
                    r.symbol == start && r.is_complete()
                } else {
                    false
                }
            })
            .collect();

        if complete_parses.is_empty() {
            return Err(ParseError::Incomplete);
        }

        // For now: pick first. Later: scoring heuristics
        Ok(self.extract_tree(complete_parses[0]))
    }

    fn extract_tree(&self, root: &ForestNode) -> ParseTree {
        ParseTree {
            root: self.build_tree_node(root),
            source: self.source.clone(),
        }
    }

    fn build_tree_node(&self, forest_node: &ForestNode) -> ParseNode {
        // Take first alternative (later: scoring)
        let alt = &forest_node.alternatives[0];

        ParseNode {
            symbol: forest_node.symbol,
            span: forest_node.span,
            children: alt
                .children
                .iter()
                .map(|child_id| self.build_tree_node(&self.nodes[child_id]))
                .collect(),
        }
    }
}
