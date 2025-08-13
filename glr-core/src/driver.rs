//! Public driver that runs the GLR engine and returns a trait-object forest.

use crate::forest_view::{Forest, ForestView, Span};
use crate::parse_forest::ParseForest;
use crate::ParseTable;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum GlrError {
    #[error("lexer error: {0}")]
    Lex(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("{0}")]
    Other(String),
}

pub struct Driver<'t> {
    tables: &'t ParseTable,
}

impl<'t> Driver<'t> {
    pub fn new(tables: &'t ParseTable) -> Self {
        Self { tables }
    }

    /// Parse from a token stream.
    pub fn parse_tokens<I>(&mut self, _tokens: I) -> Result<Forest, GlrError>
    where
        I: Iterator<Item = (u32 /* kind */, u32 /* start */, u32 /* end */)>,
    {
        // TODO: call your GLR core here, produce your concrete SPPF/forest,
        // For now, return a stub until we wire the actual GLR parser
        
        Err(GlrError::Other("driver not wired yet".into()))
    }
    
    /// Convert internal parse forest to public Forest
    pub(crate) fn wrap_forest(forest: ParseForest) -> Forest {
        let view = Box::new(ParseForestView::new(forest));
        Forest { view }
    }
}

/// Adapter that implements ForestView for the internal ParseForest
struct ParseForestView {
    forest: ParseForest,
    /// Cache for converted root IDs
    root_ids: Vec<u32>,
    /// Cache for children arrays (indexed by node ID)
    children_cache: HashMap<u32, Vec<u32>>,
}

impl ParseForestView {
    fn new(forest: ParseForest) -> Self {
        // Convert root node IDs to u32
        let root_ids: Vec<u32> = forest.roots.iter().map(|node| node.id as u32).collect();
        
        // Pre-build children cache for all nodes
        let mut children_cache = HashMap::new();
        for (node_id, node) in &forest.nodes {
            // Choose first alternative if available
            if let Some(first_alt) = node.alternatives.first() {
                let children: Vec<u32> = first_alt.children.iter().map(|&id| id as u32).collect();
                children_cache.insert(*node_id as u32, children);
            } else {
                children_cache.insert(*node_id as u32, Vec::new());
            }
        }
        
        Self {
            forest,
            root_ids,
            children_cache,
        }
    }
}

impl ForestView for ParseForestView {
    fn roots(&self) -> &[u32] {
        &self.root_ids
    }
    
    fn kind(&self, id: u32) -> u32 {
        if let Some(node) = self.forest.nodes.get(&(id as usize)) {
            node.symbol.0 as u32
        } else {
            0
        }
    }
    
    fn span(&self, id: u32) -> Span {
        if let Some(node) = self.forest.nodes.get(&(id as usize)) {
            Span {
                start: node.span.0 as u32,
                end: node.span.1 as u32,
            }
        } else {
            Span { start: 0, end: 0 }
        }
    }
    
    fn best_children(&self, id: u32) -> &[u32] {
        // Return cached children array
        self.children_cache.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}