use crate::{Grammar, SymbolId};
use std::collections::HashMap;

/// Special symbol ID for error nodes (outside grammar's symbol space)
pub const ERROR_SYMBOL: SymbolId = SymbolId(u16::MAX);

/// Extra metadata carried by leaf/ERROR nodes.
#[derive(Debug, Clone, Copy, Default)]
pub struct ErrorMeta {
    /// If true, this terminal was inserted by recovery (zero-width).
    pub missing: bool,
    /// If true, this node demarcates a run of skipped/unknown bytes.
    pub is_error: bool,
    /// Error cost accumulated for this recovery action
    pub cost: u32,
}

/// A parse forest represents all possible parse trees for ambiguous input
#[derive(Debug, Clone)]
pub struct ParseForest {
    pub roots: Vec<ForestNode>,
    pub nodes: HashMap<usize, ForestNode>,
    pub grammar: Grammar,
    pub source: String,
    /// Next available node ID
    pub next_node_id: usize,
}

impl ParseForest {
    /// Create an error chunk node for skipped bytes
    pub fn push_error_chunk(&mut self, span: (usize, usize)) -> usize {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.insert(id, ForestNode {
            id,
            symbol: ERROR_SYMBOL,
            span,
            alternatives: vec![ForestAlternative { children: vec![] }],
            error_meta: ErrorMeta { 
                is_error: true, 
                missing: false,
                cost: 1,
            },
        });
        id
    }
}

/// A node in the parse forest that may have multiple alternatives
#[derive(Debug, Clone)]
pub struct ForestNode {
    pub id: usize,
    pub symbol: SymbolId,
    pub span: (usize, usize),
    pub alternatives: Vec<ForestAlternative>,
    /// Error metadata for terminal/error nodes
    pub error_meta: ErrorMeta,
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
