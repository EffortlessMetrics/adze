//! GLR parse forest representation and manipulation.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// GLR Parse Forest implementation
// This module implements a Shared Packed Parse Forest (SPPF) for efficient
// representation of ambiguous parse trees.

use crate::parser_v4::ParseNode;
use crate::stack_pool::StackPool;
use rust_sitter_ir::{RuleId, SymbolId};
use std::collections::HashMap;
use std::rc::Rc;

/// A node in the parse forest that can represent multiple parse trees
#[derive(Debug, Clone)]
pub enum ForestNode {
    /// Terminal node (leaf)
    Terminal {
        symbol: SymbolId,
        start: usize,
        end: usize,
        text: Vec<u8>,
    },
    /// Non-terminal node with one or more interpretations
    NonTerminal {
        symbol: SymbolId,
        start: usize,
        end: usize,
        /// Alternative interpretations of this non-terminal
        alternatives: Vec<PackedNode>,
    },
}

/// A packed node represents one way to derive a non-terminal
#[derive(Debug, Clone)]
pub struct PackedNode {
    /// The rule that was used for this derivation
    pub rule_id: RuleId,
    /// The children in this derivation
    pub children: Vec<Rc<ForestNode>>,
}

/// Graph-Structured Stack (GSS) for GLR parsing
/// This allows sharing of common prefixes between different parse paths
#[derive(Debug)]
pub struct GSSNode {
    /// Parser state
    pub state: usize,
    /// Links to parent nodes (can have multiple parents in GLR)
    pub parents: Vec<GSSLink>,
    /// Unique identifier
    pub id: usize,
}

/// A link in the GSS representing a reduction path
#[derive(Debug, Clone)]
pub struct GSSLink {
    /// Parent GSS node
    pub parent: usize, // Index into GSS node pool
    /// Parse tree node created by this link
    pub tree_node: Rc<ForestNode>,
}

/// Statistics for GLR parsing
#[derive(Debug, Default, Clone)]
pub struct GLRStats {
    /// Total number of GSS nodes created
    pub total_nodes_created: usize,
    /// Maximum number of active heads at any point
    pub max_active_heads: usize,
    /// Total number of forks performed
    pub total_forks: usize,
    /// Total number of merges performed
    pub total_merges: usize,
    /// Number of shared forest nodes (cache hits)
    pub forest_cache_hits: usize,
}

/// GLR parser state that supports forking
pub struct GLRParserState {
    /// Pool of GSS nodes
    pub gss_nodes: Vec<GSSNode>,
    /// Active GSS node heads (frontier of parsing)
    pub active_heads: Vec<usize>,
    /// Next GSS node ID
    pub next_gss_id: usize,
    /// Cache for sharing forest nodes
    pub forest_cache: HashMap<(SymbolId, usize, usize), Rc<ForestNode>>,
    /// Statistics for performance monitoring
    pub stats: GLRStats,
    /// Stack pool for efficient memory reuse
    pub stack_pool: Rc<StackPool<usize>>,
}

impl Default for GLRParserState {
    fn default() -> Self {
        Self::new()
    }
}

impl GLRParserState {
    pub fn new() -> Self {
        let mut state = Self {
            gss_nodes: Vec::new(),
            active_heads: Vec::new(),
            next_gss_id: 0,
            forest_cache: HashMap::new(),
            stats: GLRStats::default(),
            stack_pool: Rc::new(StackPool::new(64)), // Default pool size
        };

        // Create initial GSS node for state 0
        let initial_node = GSSNode {
            state: 0,
            parents: Vec::new(),
            id: 0,
        };
        state.gss_nodes.push(initial_node);
        state.active_heads.push(0);
        state.next_gss_id = 1;
        state.stats.total_nodes_created = 1;
        state.stats.max_active_heads = 1;

        state
    }

    /// Fork the parser state for handling ambiguity
    pub fn fork(&mut self, _gss_node_idx: usize, new_state: usize) -> usize {
        // Check if a node with this state already exists at this position
        for &head in &self.active_heads {
            if self.gss_nodes[head].state == new_state {
                // Reuse existing node
                return head;
            }
        }

        // Create new GSS node
        let new_node = GSSNode {
            state: new_state,
            parents: Vec::new(),
            id: self.next_gss_id,
        };
        self.next_gss_id += 1;

        let new_idx = self.gss_nodes.len();
        self.gss_nodes.push(new_node);
        self.active_heads.push(new_idx);

        // Update statistics
        self.stats.total_nodes_created += 1;
        self.stats.total_forks += 1;
        self.stats.max_active_heads = self.stats.max_active_heads.max(self.active_heads.len());

        new_idx
    }

    /// Create or retrieve a cached forest node
    pub fn get_or_create_forest_node(
        &mut self,
        symbol: SymbolId,
        start: usize,
        end: usize,
        create_fn: impl FnOnce() -> ForestNode,
    ) -> Rc<ForestNode> {
        let key = (symbol, start, end);

        if let Some(node) = self.forest_cache.get(&key) {
            self.stats.forest_cache_hits += 1;
            return node.clone();
        }

        let node = Rc::new(create_fn());
        self.forest_cache.insert(key, node.clone());
        node
    }

    /// Merge parse trees when multiple derivations lead to the same state
    pub fn merge_trees(
        &mut self,
        symbol: SymbolId,
        start: usize,
        end: usize,
        new_alternative: PackedNode,
    ) -> Rc<ForestNode> {
        let key = (symbol, start, end);

        if let Some(existing) = self.forest_cache.get(&key) {
            // Cannot mutate through Rc, need to create new node with merged alternatives
            if let ForestNode::NonTerminal { alternatives, .. } = existing.as_ref() {
                let mut new_alts = alternatives.clone();
                new_alts.push(new_alternative);
                let merged = Rc::new(ForestNode::NonTerminal {
                    symbol,
                    start,
                    end,
                    alternatives: new_alts,
                });
                self.forest_cache.insert(key, merged.clone());
                self.stats.total_merges += 1;
                merged
            } else {
                existing.clone()
            }
        } else {
            // Create new non-terminal with single alternative
            let node = Rc::new(ForestNode::NonTerminal {
                symbol,
                start,
                end,
                alternatives: vec![new_alternative],
            });
            self.forest_cache.insert(key, node.clone());
            node
        }
    }

    /// Get a reference to the parser statistics
    pub fn get_stats(&self) -> &GLRStats {
        &self.stats
    }
}

/// Convert a forest node to a single parse tree (picking first alternative)
pub fn forest_to_parse_tree(forest: &ForestNode) -> ParseNode {
    match forest {
        ForestNode::Terminal {
            symbol, start, end, ..
        } => ParseNode {
            symbol: *symbol,
            symbol_id: *symbol,
            start_byte: *start,
            end_byte: *end,
            children: Vec::new(),
            field_name: None,
        },
        ForestNode::NonTerminal {
            symbol,
            start,
            end,
            alternatives,
        } => {
            // For now, just pick the first alternative
            // TODO: Provide API to explore all alternatives
            let first_alt = &alternatives[0];
            let children = first_alt
                .children
                .iter()
                .map(|child| forest_to_parse_tree(child))
                .collect();

            ParseNode {
                symbol: *symbol,
                symbol_id: *symbol,
                start_byte: *start,
                end_byte: *end,
                children,
                field_name: None,
            }
        }
    }
}
