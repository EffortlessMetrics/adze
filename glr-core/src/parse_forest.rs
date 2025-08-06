use crate::{Grammar, SymbolId};
use std::collections::HashMap;

/// A parse forest represents all possible parse trees for ambiguous input
#[derive(Debug, Clone)]
pub struct ParseForest {
    pub roots: Vec<ForestNode>,
    pub nodes: HashMap<usize, ForestNode>,
    pub grammar: Grammar,
    pub source: String,
}

/// A node in the parse forest that may have multiple alternatives
#[derive(Debug, Clone)]
pub struct ForestNode {
    pub id: usize,
    pub symbol: SymbolId,
    pub span: (usize, usize),
    pub alternatives: Vec<ForestAlternative>,
}

/// One possible parse alternative for a forest node
#[derive(Debug, Clone)]
pub struct ForestAlternative {
    pub children: Vec<usize>, // IDs of child nodes
}

impl ForestNode {
    pub fn is_complete(&self) -> bool {
        // A node is complete if it has at least one alternative
        !self.alternatives.is_empty()
    }
}

/// A single parse tree extracted from the forest
#[derive(Debug, Clone)]
pub struct ParseTree {
    pub root: ParseNode,
    pub source: String,
}

/// A node in a concrete parse tree
#[derive(Debug, Clone)]
pub struct ParseNode {
    pub symbol: SymbolId,
    pub span: (usize, usize),
    pub children: Vec<ParseNode>,
}

/// Parse error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Incomplete parse")]
    Incomplete,

    #[error("Parse failed: {0}")]
    Failed(String),

    #[error("Unknown error")]
    Unknown,
}
