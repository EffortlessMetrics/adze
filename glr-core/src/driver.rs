//! Public driver that runs the GLR engine and returns a trait-object forest.

use crate::forest_view::{Forest, ForestView, Span};
use crate::parse_forest::{ParseForest, ForestNode, ForestAlternative};
use crate::{ParseTable, Action, StateId, SymbolId, RuleId};
use std::collections::HashMap;

/// Error type for GLR parsing operations
#[derive(thiserror::Error, Debug)]
pub enum GlrError {
    /// Lexer error
    #[error("lexer error: {0}")]
    Lex(String),
    /// Parse error
    #[error("parse error: {0}")]
    Parse(String),
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// GLR parser driver that executes the parsing algorithm
pub struct Driver<'t> {
    /// LR tables used by the driver
    #[allow(dead_code)]
    tables: &'t ParseTable,
}

/// A GLR parse stack
#[derive(Debug, Clone, Default)]
struct ParseStack {
    states: Vec<StateId>,
    nodes: Vec<usize>, // Node IDs in the forest
    pos: usize,        // Current byte position (end of last consumed token)
}

/// GLR parser state
struct GlrState {
    stacks: Vec<ParseStack>,
    forest: ParseForest,
    next_node_id: usize,
}

impl<'t> Driver<'t> {
    /// Create a new driver with the given parse tables
    pub fn new(tables: &'t ParseTable) -> Self {
        Self { tables }
    }

    /// Parse from a token stream.
    pub fn parse_tokens<I>(&mut self, tokens: I) -> Result<Forest, GlrError>
    where
        I: IntoIterator<Item = (u32 /* kind */, u32 /* start */, u32 /* end */)>,
    {
        // Initialize state with grammar from parse table
        let mut state = GlrState {
            stacks: vec![ParseStack {
                states: vec![StateId(0)],
                nodes: vec![],
                pos: 0,
            }],
            forest: ParseForest {
                roots: vec![],
                nodes: HashMap::new(),
                grammar: self.tables.grammar().clone(),
                source: String::new(),
            },
            next_node_id: 0,
        };

        // Main token loop
        for (kind, start, end) in tokens.into_iter() {
            // Add debug assert for token width
            debug_assert!(kind <= u16::MAX as u32, "terminal id overflow");
            let lookahead = SymbolId(kind as u16);
            
            let stacks = std::mem::take(&mut state.stacks);
            let mut new_stacks = Vec::with_capacity(stacks.len());

            for mut stk in stacks {
                // 1) Closure: apply all reduces available on this lookahead BEFORE any shift
                self.reduce_closure(&mut state, &mut stk, lookahead)?;

                // 2) Then apply shifts for this lookahead
                for action in self.tables.actions(*stk.states.last().unwrap(), lookahead) {
                    match *action {
                        Action::Shift(ns) => {
                            let node_id = self.push_terminal(&mut state, lookahead, (start as usize, end as usize));
                            let mut s2 = stk.clone();
                            s2.states.push(ns);
                            s2.nodes.push(node_id);
                            s2.pos = end as usize; // Update position to token end
                            new_stacks.push(s2);
                        }
                        Action::Accept => {
                            // Accept on lookahead (rare, usually on EOF)
                            if let Some(&root_id) = stk.nodes.last() {
                                if let Some(root) = state.forest.nodes.get(&root_id).cloned() {
                                    // Assert the accepted root is the start symbol (catches table/config drift)
                                    debug_assert_eq!(
                                        root.symbol, self.tables.start_symbol(),
                                        "accepted non-start symbol: {:?} != {:?}",
                                        root.symbol, self.tables.start_symbol()
                                    );
                                    state.forest.roots.push(root);
                                }
                            }
                            return Ok(Self::wrap_forest(state.forest));
                        }
                        Action::Reduce(rid) => {
                            // If your table encodes reduce+shift conflicts, we still need to try the reduce path
                            let s2 = self.reduce_once(&mut state, stk.clone(), rid)?;
                            // After a single reduce, we can still be able to shift this lookahead
                            let mut s2_clone = s2.clone();
                            self.reduce_closure(&mut state, &mut s2_clone, lookahead)?;
                            for a2 in self.tables.actions(*s2_clone.states.last().unwrap(), lookahead) {
                                if let Action::Shift(ns) = *a2 {
                                    let node_id = self.push_terminal(&mut state, lookahead, (start as usize, end as usize));
                                    let mut s3 = s2_clone.clone();
                                    s3.states.push(ns);
                                    s3.nodes.push(node_id);
                                    s3.pos = end as usize; // Update position to token end
                                    new_stacks.push(s3);
                                }
                            }
                        }
                        Action::Error => { /* drop path */ }
                        Action::Fork(ref xs) => {
                            // If your generator emits Fork, just treat as a set of actions
                            for a in xs {
                                if let Action::Shift(ns) = *a {
                                    let node_id = self.push_terminal(&mut state, lookahead, (start as usize, end as usize));
                                    let mut s2 = stk.clone();
                                    s2.states.push(ns);
                                    s2.nodes.push(node_id);
                                    s2.pos = end as usize; // Update position to token end
                                    new_stacks.push(s2);
                                } else if let Action::Reduce(rid) = *a {
                                    let mut s2 = self.reduce_once(&mut state, stk.clone(), rid)?;
                                    self.reduce_closure(&mut state, &mut s2, lookahead)?;
                                    // After closure, check if we can shift
                                    for a2 in self.tables.actions(*s2.states.last().unwrap(), lookahead) {
                                        if let Action::Shift(ns) = *a2 {
                                            let node_id = self.push_terminal(&mut state, lookahead, (start as usize, end as usize));
                                            let mut s3 = s2.clone();
                                            s3.states.push(ns);
                                            s3.nodes.push(node_id);
                                            s3.pos = end as usize;
                                            new_stacks.push(s3);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if new_stacks.is_empty() {
                let top_state = state.stacks[0].states.last().copied().unwrap_or(StateId(0));
                return Err(GlrError::Parse(format!(
                    "no valid parse paths at byte {} (state={}, symbol={})",
                    start, top_state.0, lookahead.0
                )));
            }
            state.stacks = new_stacks;
        }

        // EOF phase - use the table's EOF symbol instead of hardcoded 0
        let eof = self.tables.eof();
        let stacks = std::mem::take(&mut state.stacks);
        for mut stk in stacks {
            self.reduce_closure(&mut state, &mut stk, eof)?;
            for action in self.tables.actions(*stk.states.last().unwrap(), eof) {
                match *action {
                    Action::Accept => {
                        if let Some(&root_id) = stk.nodes.last() {
                            if let Some(root) = state.forest.nodes.get(&root_id).cloned() {
                                // Assert the accepted root is the start symbol (catches table/config drift)
                                debug_assert_eq!(
                                    root.symbol, self.tables.start_symbol(),
                                    "accepted non-start symbol: {:?} != {:?}",
                                    root.symbol, self.tables.start_symbol()
                                );
                                state.forest.roots.push(root);
                            }
                        }
                        return Ok(Self::wrap_forest(state.forest));
                    }
                    Action::Reduce(rid) => {
                        let s2 = self.reduce_once(&mut state, stk.clone(), rid)?;
                        // Try accept after reduce
                        for a2 in self.tables.actions(*s2.states.last().unwrap(), eof) {
                            if let Action::Accept = *a2 {
                                if let Some(&root_id) = s2.nodes.last() {
                                    if let Some(root) = state.forest.nodes.get(&root_id).cloned() {
                                        // Assert the accepted root is the start symbol (catches table/config drift)
                                        debug_assert_eq!(
                                            root.symbol, self.tables.start_symbol(),
                                            "accepted non-start symbol: {:?} != {:?}",
                                            root.symbol, self.tables.start_symbol()
                                        );
                                        state.forest.roots.push(root);
                                    }
                                }
                                return Ok(Self::wrap_forest(state.forest));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(GlrError::Parse(format!(
            "input not accepted: EOF phase failed (expected start symbol {}, got {} root(s))",
            self.tables.start_symbol().0,
            state.forest.roots.len()
        )))
    }

    #[inline]
    fn push_terminal(&self, st: &mut GlrState, sym: SymbolId, span: (usize, usize)) -> usize {
        let id = st.next_node_id;
        st.next_node_id += 1;
        st.forest.nodes.insert(id, ForestNode {
            id,
            symbol: sym,
            span,
            alternatives: vec![ForestAlternative { children: vec![] }],
        });
        id
    }

    /// Apply exactly one reduce(rid) to `stack`; return the new stack with a pushed goto state.
    fn reduce_once(&self, st: &mut GlrState, mut stack: ParseStack, rid: RuleId) -> Result<ParseStack, GlrError> {
        let (lhs, rhs_len) = self.tables.rule(rid);
        if rhs_len as usize > stack.nodes.len() || rhs_len as usize > stack.states.len().saturating_sub(1) {
            return Err(GlrError::Parse(format!(
                "reduce underflow: rule {} requires {} symbols but stack has {}",
                rid.0, rhs_len, stack.nodes.len()
            )));
        }

        // Pop rhs_len nodes/states (states pop rhs_len; bottom state remains)
        let child_ids: Vec<usize> = stack.nodes.split_off(stack.nodes.len() - rhs_len as usize);
        let goto_from = *stack.states.get(stack.states.len() - 1 - rhs_len as usize).unwrap();
        stack.states.truncate(stack.states.len() - rhs_len as usize);

        // Span = [first_child.start, last_child.end], or current position if empty production
        let (start, end) = if child_ids.is_empty() {
            // Empty production - use current position
            (stack.pos, stack.pos)
        } else {
            let first = st.forest.nodes.get(child_ids.first().unwrap()).unwrap().span.0;
            let last = st.forest.nodes.get(child_ids.last().unwrap()).unwrap().span.1;
            (first, last)
        };

        // Build nonterminal node
        let id = st.next_node_id;
        st.next_node_id += 1;
        st.forest.nodes.insert(id, ForestNode {
            id,
            symbol: lhs,
            span: (start, end),
            alternatives: vec![ForestAlternative { children: child_ids }],
        });

        // Goto
        let Some(ns) = self.tables.goto(goto_from, lhs) else {
            return Err(GlrError::Parse(format!(
                "missing goto: no transition from state {} on symbol {}",
                goto_from.0, lhs.0
            )));
        };
        stack.states.push(ns);
        stack.nodes.push(id);
        Ok(stack)
    }

    /// Keep reducing as long as there is at least one reduce for (top, lookahead).
    fn reduce_closure(&self, st: &mut GlrState, stack: &mut ParseStack, lookahead: SymbolId) -> Result<(), GlrError> {
        loop {
            let state = *stack.states.last().unwrap();
            let mut did_reduce = false;
            for action in self.tables.actions(state, lookahead) {
                if let Action::Reduce(rid) = *action {
                    *stack = self.reduce_once(st, std::mem::take(stack), rid)?;
                    did_reduce = true;
                    break; // Re-evaluate from new top after one reduce
                }
            }
            if !did_reduce {
                break;
            }
        }
        Ok(())
    }

    /// Convert internal parse forest to public Forest
    pub(crate) fn wrap_forest(mut forest: ParseForest) -> Forest {
        // Deterministic root selection: prefer largest span, then earliest start position
        forest.roots.sort_by_key(|n| (
            std::cmp::Reverse(n.span.1.saturating_sub(n.span.0)), // Largest span first
            n.span.0  // Then earliest start position
        ));
        // Remove duplicate roots by ID
        forest.roots.dedup_by_key(|n| n.id);
        
        let view = Box::new(ParseForestView::new(forest));
        Forest { view }
    }
}

struct ParseForestView {
    forest: ParseForest,
    roots_cache: Vec<u32>,
    children_cache: HashMap<u32, Vec<u32>>,
}

impl ParseForestView {
    fn new(forest: ParseForest) -> Self {
        let roots_cache = forest.roots.iter().map(|n| n.id as u32).collect();
        
        // Pre-compute children cache
        let mut children_cache = HashMap::new();
        for (&id, node) in &forest.nodes {
            if !node.alternatives.is_empty() {
                // Take first alternative (best)
                let children: Vec<u32> = node.alternatives[0].children
                    .iter()
                    .map(|&c| c as u32)
                    .collect();
                children_cache.insert(id as u32, children);
            }
        }
        
        Self { 
            forest, 
            roots_cache,
            children_cache,
        }
    }
}

impl ForestView for ParseForestView {
    fn roots(&self) -> &[u32] {
        &self.roots_cache
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
            Span { start: node.span.0 as u32, end: node.span.1 as u32 }
        } else {
            Span { start: 0, end: 0 }
        }
    }

    fn best_children(&self, id: u32) -> &[u32] {
        self.children_cache.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}