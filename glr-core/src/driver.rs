//! Public driver that runs the GLR engine and returns a trait-object forest.

use crate::forest_view::{Forest, ForestView, Span};
use crate::parse_forest::{ParseForest, ForestNode, ForestAlternative};
use crate::{ParseTable, Action, StateId, SymbolId, RuleId};
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

/// A GLR parse stack
#[derive(Debug, Clone)]
struct ParseStack {
    states: Vec<StateId>,
    nodes: Vec<usize>, // Node IDs in the forest
}

/// GLR parser state
struct GlrState {
    stacks: Vec<ParseStack>,
    forest: ParseForest,
    next_node_id: usize,
}

impl<'t> Driver<'t> {
    pub fn new(tables: &'t ParseTable) -> Self {
        Self { tables }
    }

    /// Parse from a token stream.
    pub fn parse_tokens<I>(&mut self, tokens: I) -> Result<Forest, GlrError>
    where
        I: Iterator<Item = (u32 /* kind */, u32 /* start */, u32 /* end */)>,
    {
        let mut state = GlrState {
            stacks: vec![ParseStack {
                states: vec![StateId(0)],
                nodes: vec![],
            }],
            forest: ParseForest {
                roots: vec![],
                nodes: HashMap::new(),
                grammar: crate::Grammar::default(), // TODO: Pass grammar
                source: String::new(),
            },
            next_node_id: 0,
        };
        
        // Process each token
        for (kind, start, end) in tokens {
            let symbol = SymbolId(kind as u16);
            
            // Process this token on all active stacks
            let mut new_stacks = Vec::new();
            
            let stacks = std::mem::take(&mut state.stacks);
            for stack in stacks {
                // Get the current state
                let current_state = *stack.states.last().unwrap();
                
                // Look up actions for this state and symbol
                if let Some(actions) = self.get_actions(current_state, symbol) {
                    for action in actions {
                        match action {
                            Action::Shift(next_state) => {
                                // Create a terminal node
                                let node = ForestNode {
                                    id: state.next_node_id,
                                    symbol,
                                    span: (start as usize, end as usize),
                                    alternatives: vec![ForestAlternative { children: vec![] }],
                                };
                                let node_id = node.id;
                                state.forest.nodes.insert(node_id, node);
                                state.next_node_id += 1;
                                
                                // Push the new state and node
                                let mut new_stack = stack.clone();
                                new_stack.states.push(*next_state);
                                new_stack.nodes.push(node_id);
                                new_stacks.push(new_stack);
                            }
                            Action::Reduce(rule_id) => {
                                // Handle reduction (simplified for now)
                                new_stacks.push(self.reduce(&mut state, stack.clone(), *rule_id)?);
                            }
                            Action::Accept => {
                                // Accept state reached
                                if !stack.nodes.is_empty() {
                                    let root_id = *stack.nodes.last().unwrap();
                                    if let Some(root) = state.forest.nodes.get(&root_id).cloned() {
                                        state.forest.roots.push(root);
                                    }
                                }
                                return Ok(Self::wrap_forest(state.forest));
                            }
                            Action::Error => {
                                // Skip this stack
                            }
                            Action::Fork(actions) => {
                                // Handle multiple actions (GLR fork)
                                for fork_action in actions {
                                    match fork_action {
                                        Action::Shift(next_state) => {
                                            let node = ForestNode {
                                                id: state.next_node_id,
                                                symbol,
                                                span: (start as usize, end as usize),
                                                alternatives: vec![ForestAlternative { children: vec![] }],
                                            };
                                            let node_id = node.id;
                                            state.forest.nodes.insert(node_id, node);
                                            state.next_node_id += 1;
                                            
                                            let mut new_stack = stack.clone();
                                            new_stack.states.push(*next_state);
                                            new_stack.nodes.push(node_id);
                                            new_stacks.push(new_stack);
                                        }
                                        Action::Reduce(rule_id) => {
                                            new_stacks.push(self.reduce(&mut state, stack.clone(), *rule_id)?);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // No valid actions - this stack dies
                }
            }
            
            state.stacks = new_stacks;
            
            if state.stacks.is_empty() {
                return Err(GlrError::Parse("No valid parse paths".into()));
            }
        }
        
        // Process EOF (symbol 0 typically)
        let eof_symbol = SymbolId(0);
        
        for stack in state.stacks {
            let current_state = *stack.states.last().unwrap();
            if let Some(actions) = self.get_actions(current_state, eof_symbol) {
                for action in actions {
                    if let Action::Accept = action {
                        if !stack.nodes.is_empty() {
                            let root_id = *stack.nodes.last().unwrap();
                            if let Some(root) = state.forest.nodes.get(&root_id).cloned() {
                                state.forest.roots.push(root);
                            }
                        }
                        return Ok(Self::wrap_forest(state.forest));
                    }
                }
            }
        }
        
        Err(GlrError::Parse("Input not accepted".into()))
    }
    
    /// Perform a reduce action
    fn reduce(&self, _state: &mut GlrState, stack: ParseStack, _rule_id: RuleId) -> Result<ParseStack, GlrError> {
        // Simplified reduction - in a real implementation we'd need to:
        // 1. Pop the right number of states/nodes based on the rule
        // 2. Create a new non-terminal node
        // 3. Look up the goto state
        // For now, just return the stack unchanged
        Ok(stack)
    }
    
    /// Get actions for a state and symbol
    fn get_actions(&self, state: StateId, symbol: SymbolId) -> Option<&[Action]> {
        // Look up in the action table
        if let Some(symbol_idx) = self.tables.symbol_to_index.get(&symbol) {
            let state_idx = state.0 as usize;
            if state_idx < self.tables.action_table.len() && *symbol_idx < self.tables.action_table[state_idx].len() {
                let cell = &self.tables.action_table[state_idx][*symbol_idx];
                if !cell.is_empty() {
                    return Some(cell);
                }
            }
        }
        None
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