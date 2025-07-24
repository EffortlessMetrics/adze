// GLR parser generation algorithms for pure-Rust Tree-sitter
// This module implements the core GLR state machine generation and conflict resolution

use fixedbitset::FixedBitSet;
use indexmap::IndexMap;
use rust_sitter_ir::*;
// Re-export commonly used types
pub use rust_sitter_ir::{SymbolId, RuleId, StateId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod advanced_conflict;
pub mod version_info;
pub mod precedence_compare;
pub mod conflict_visualizer;
pub mod symbol_comparison;

pub use advanced_conflict::{ConflictAnalyzer, PrecedenceResolver, ConflictStats, PrecedenceDecision};
pub use version_info::{VersionInfo, CompareResult, compare_versions};
pub use precedence_compare::{
    StaticPrecedenceResolver, PrecedenceInfo, PrecedenceComparison, compare_precedences
};
pub use conflict_visualizer::{ConflictVisualizer, generate_dot_graph};
pub use symbol_comparison::{compare_symbols, compare_versions_with_symbols};

/// FIRST/FOLLOW sets computation for GLR parsing
#[derive(Debug, Clone)]
pub struct FirstFollowSets {
    first: IndexMap<SymbolId, FixedBitSet>,
    follow: IndexMap<SymbolId, FixedBitSet>,
    nullable: FixedBitSet,
    #[allow(dead_code)]
    symbol_count: usize,
}

impl FirstFollowSets {
    /// Compute FIRST/FOLLOW sets for the given grammar
    pub fn compute(grammar: &Grammar) -> Self {
        // Find the maximum symbol ID to determine the size needed
        let max_rule_id = grammar.rules.keys().map(|id| id.0).max().unwrap_or(0);
        let max_token_id = grammar.tokens.keys().map(|id| id.0).max().unwrap_or(0);
        let max_external_id = grammar.externals.iter().map(|e| e.symbol_id.0).max().unwrap_or(0);
        let symbol_count = (max_rule_id.max(max_token_id).max(max_external_id) + 1) as usize;
        
        let mut first = IndexMap::new();
        let mut follow = IndexMap::new();
        let mut nullable = FixedBitSet::with_capacity(symbol_count);

        // Initialize sets
        for &symbol_id in grammar.rules.keys().chain(grammar.tokens.keys()) {
            first.insert(symbol_id, FixedBitSet::with_capacity(symbol_count));
            follow.insert(symbol_id, FixedBitSet::with_capacity(symbol_count));
        }

        // Compute FIRST sets
        let mut changed = true;
        while changed {
            changed = false;
            
            for rule in grammar.rules.values() {
                let lhs = rule.lhs;
                let mut rule_nullable = true;
                
                for symbol in &rule.rhs {
                    match symbol {
                        Symbol::Terminal(id) => {
                            if let Some(first_set) = first.get_mut(&lhs) {
                                if !first_set.contains(id.0 as usize) {
                                    first_set.insert(id.0 as usize);
                                    changed = true;
                                }
                            }
                            rule_nullable = false;
                            break;
                        }
                        Symbol::NonTerminal(id) | Symbol::External(id) => {
                            if let Some(symbol_first) = first.get(id).cloned() {
                                if let Some(lhs_first) = first.get_mut(&lhs) {
                                    let old_len = lhs_first.count_ones(..);
                                    lhs_first.union_with(&symbol_first);
                                    if lhs_first.count_ones(..) > old_len {
                                        changed = true;
                                    }
                                }
                            }
                            
                            if !nullable.contains(id.0 as usize) {
                                rule_nullable = false;
                                break;
                            }
                        }
                    }
                }
                
                if rule_nullable && !nullable.contains(lhs.0 as usize) {
                    nullable.insert(lhs.0 as usize);
                    changed = true;
                }
            }
        }

        // Compute FOLLOW sets
        changed = true;
        while changed {
            changed = false;
            
            for rule in grammar.rules.values() {
                for (i, symbol) in rule.rhs.iter().enumerate() {
                    if let Symbol::NonTerminal(id) | Symbol::External(id) = symbol {
                        // Add FIRST of remaining symbols to FOLLOW of current symbol
                        let remaining = &rule.rhs[i + 1..];
                        let first_of_remaining = Self::first_of_sequence_static(
                            remaining, &first, &nullable
                        );
                        
                        if let Some(follow_set) = follow.get_mut(id) {
                            let old_len = follow_set.count_ones(..);
                            follow_set.union_with(&first_of_remaining);
                            if follow_set.count_ones(..) > old_len {
                                changed = true;
                            }
                        }
                        
                        // If remaining symbols are nullable, add FOLLOW of LHS
                        if Self::sequence_is_nullable(remaining, &nullable) {
                            if let Some(lhs_follow) = follow.get(&rule.lhs).cloned() {
                                if let Some(follow_set) = follow.get_mut(id) {
                                    let old_len = follow_set.count_ones(..);
                                    follow_set.union_with(&lhs_follow);
                                    if follow_set.count_ones(..) > old_len {
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            first,
            follow,
            nullable,
            symbol_count,
        }
    }

    /// Get FIRST set of a sequence of symbols
    pub fn first_of_sequence(&self, symbols: &[Symbol]) -> FixedBitSet {
        Self::first_of_sequence_static(symbols, &self.first, &self.nullable)
    }

    fn first_of_sequence_static(
        symbols: &[Symbol],
        first: &IndexMap<SymbolId, FixedBitSet>,
        nullable: &FixedBitSet,
    ) -> FixedBitSet {
        let mut result = FixedBitSet::with_capacity(first.len());
        
        for symbol in symbols {
            match symbol {
                Symbol::Terminal(id) => {
                    result.insert(id.0 as usize);
                    break;
                }
                Symbol::NonTerminal(id) | Symbol::External(id) => {
                    if let Some(symbol_first) = first.get(id) {
                        result.union_with(symbol_first);
                    }
                    
                    if !nullable.contains(id.0 as usize) {
                        break;
                    }
                }
            }
        }
        
        result
    }

    fn sequence_is_nullable(symbols: &[Symbol], nullable: &FixedBitSet) -> bool {
        symbols.iter().all(|symbol| match symbol {
            Symbol::Terminal(_) => false,
            Symbol::NonTerminal(id) | Symbol::External(id) => {
                nullable.contains(id.0 as usize)
            }
        })
    }

    /// Get FIRST set for a symbol
    pub fn first(&self, symbol: SymbolId) -> Option<&FixedBitSet> {
        self.first.get(&symbol)
    }

    /// Get FOLLOW set for a symbol
    pub fn follow(&self, symbol: SymbolId) -> Option<&FixedBitSet> {
        self.follow.get(&symbol)
    }

    /// Check if a symbol is nullable
    pub fn is_nullable(&self, symbol: SymbolId) -> bool {
        self.nullable.contains(symbol.0 as usize)
    }
}

/// LR(1) item for GLR parsing
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LRItem {
    pub rule_id: RuleId,
    pub position: usize,
    pub lookahead: SymbolId,
}

impl LRItem {
    pub fn new(rule_id: RuleId, position: usize, lookahead: SymbolId) -> Self {
        Self {
            rule_id,
            position,
            lookahead,
        }
    }

    /// Check if this item is at the end of the rule (reduce item)
    pub fn is_reduce_item(&self, grammar: &Grammar) -> bool {
        if let Some(rule) = grammar.rules.values().find(|r| r.production_id.0 == self.rule_id.0) {
            self.position >= rule.rhs.len()
        } else {
            false
        }
    }

    /// Get the symbol after the dot (next symbol to parse)
    pub fn next_symbol<'a>(&self, grammar: &'a Grammar) -> Option<&'a Symbol> {
        if let Some(rule) = grammar.rules.values().find(|r| r.production_id.0 == self.rule_id.0) {
            rule.rhs.get(self.position)
        } else {
            None
        }
    }
}

/// Set of LR(1) items representing a parser state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSet {
    pub items: HashSet<LRItem>,
    pub id: StateId,
}

impl ItemSet {
    pub fn new(id: StateId) -> Self {
        Self {
            items: HashSet::new(),
            id,
        }
    }

    pub fn add_item(&mut self, item: LRItem) {
        self.items.insert(item);
    }

    /// Compute closure of this item set
    pub fn closure(&mut self, grammar: &Grammar, first_follow: &FirstFollowSets) {
        let mut added = true;
        while added {
            added = false;
            let current_items: Vec<_> = self.items.iter().cloned().collect();
            
            for item in current_items {
                if let Some(Symbol::NonTerminal(symbol_id)) = item.next_symbol(grammar) {
                    // Find all rules with this symbol as LHS
                    for rule in grammar.rules.values() {
                        if rule.lhs == *symbol_id {
                            // Compute FIRST of β α where β is the rest of the current rule
                            // and α is the lookahead
                            let mut beta = Vec::new();
                            if let Some(current_rule) = grammar.rules.values()
                                .find(|r| r.production_id.0 == item.rule_id.0) {
                                beta.extend_from_slice(&current_rule.rhs[item.position + 1..]);
                            }
                            beta.push(Symbol::Terminal(item.lookahead));
                            
                            let first_beta_alpha = first_follow.first_of_sequence(&beta);
                            
                            // Add new items for each symbol in FIRST(β α)
                            for lookahead_idx in first_beta_alpha.ones() {
                                let new_item = LRItem::new(
                                    RuleId(rule.production_id.0),
                                    0,
                                    SymbolId(lookahead_idx as u16),
                                );
                                
                                if !self.items.contains(&new_item) {
                                    self.items.insert(new_item);
                                    added = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Compute GOTO for a given symbol
    pub fn goto(&self, symbol: &Symbol, grammar: &Grammar, _first_follow: &FirstFollowSets) -> ItemSet {
        let mut new_set = ItemSet::new(StateId(0)); // ID will be assigned later
        
        // Add all items where the dot can advance over the given symbol
        for item in &self.items {
            if let Some(next_sym) = item.next_symbol(grammar) {
                if std::mem::discriminant(next_sym) == std::mem::discriminant(symbol) {
                    match (next_sym, symbol) {
                        (Symbol::Terminal(a), Symbol::Terminal(b)) |
                        (Symbol::NonTerminal(a), Symbol::NonTerminal(b)) |
                        (Symbol::External(a), Symbol::External(b)) if a == b => {
                            let new_item = LRItem::new(
                                item.rule_id,
                                item.position + 1,
                                item.lookahead,
                            );
                            new_set.add_item(new_item);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Compute closure of the new set
        new_set.closure(grammar, _first_follow);
        new_set
    }
}

/// Collection of all LR(1) item sets (parser states)
#[derive(Debug, Clone)]
pub struct ItemSetCollection {
    pub sets: Vec<ItemSet>,
    pub goto_table: IndexMap<(StateId, SymbolId), StateId>,
}

impl ItemSetCollection {
    /// Build canonical collection of LR(1) item sets
    pub fn build_canonical_collection(grammar: &Grammar, first_follow: &FirstFollowSets) -> Self {
        let mut collection = Self {
            sets: Vec::new(),
            goto_table: IndexMap::new(),
        };

        // Create initial state with augmented start rule
        let mut initial_set = ItemSet::new(StateId(0));
        
        // Find the start symbol (LHS of the first rule in grammar)
        if let Some(start_rule) = grammar.rules.values().next() {
            let start_symbol = start_rule.lhs;
            
            // Add items for ALL rules with the start symbol as LHS
            for rule in grammar.rules.values() {
                if rule.lhs == start_symbol {
                    let start_item = LRItem::new(
                        RuleId(rule.production_id.0),
                        0,
                        SymbolId(0), // EOF symbol
                    );
                    initial_set.add_item(start_item);
                }
            }
            
            // Compute closure
            initial_set.closure(grammar, first_follow);
        }
        
        collection.sets.push(initial_set);
        let mut state_counter = 1;

        // Build all reachable states
        let mut i = 0;
        while i < collection.sets.len() {
            let current_set = collection.sets[i].clone();
            
            // Find all symbols that can be shifted from this state
            let mut symbols = HashSet::new();
            for item in &current_set.items {
                if let Some(symbol) = item.next_symbol(grammar) {
                    symbols.insert(symbol.clone());
                }
            }
            
            // Compute GOTO for each symbol
            for symbol in symbols {
                let goto_set = current_set.goto(&symbol, grammar, first_follow);
                
                if !goto_set.items.is_empty() {
                    // Check if this set already exists
                    let existing_state = collection.sets.iter()
                        .find(|set| set.items == goto_set.items)
                        .map(|set| set.id);
                    
                    let target_state = if let Some(existing_id) = existing_state {
                        existing_id
                    } else {
                        // Add new state
                        let new_id = StateId(state_counter);
                        let mut new_set = goto_set;
                        new_set.id = new_id;
                        collection.sets.push(new_set);
                        state_counter += 1;
                        new_id
                    };
                    
                    // Add to GOTO table
                    let symbol_id = match symbol {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id,
                    };
                    collection.goto_table.insert((current_set.id, symbol_id), target_state);
                }
            }
            
            i += 1;
        }

        collection
    }
}

/// GLR-compatible parse table supporting multiple actions per state
#[derive(Debug, Clone)]
pub struct ParseTable {
    pub action_table: Vec<Vec<Action>>,
    pub goto_table: Vec<Vec<StateId>>,
    pub symbol_metadata: Vec<SymbolMetadata>,
    pub state_count: usize,
    pub symbol_count: usize,
    pub symbol_to_index: HashMap<SymbolId, usize>,
}

/// Actions in GLR parse table (supporting multiple actions per state)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Shift(StateId),
    Reduce(RuleId),
    Accept,
    Error,
    Fork(Vec<Action>), // GLR fork point - multiple valid actions
}

/// Symbol metadata for the parse table
#[derive(Debug, Clone)]
pub struct SymbolMetadata {
    pub name: String,
    pub visible: bool,
    pub named: bool,
    pub supertype: bool,
}

/// Conflict detection and resolution
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub state: StateId,
    pub symbol: SymbolId,
    pub actions: Vec<Action>,
    pub conflict_type: ConflictType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    ShiftReduce,
    ReduceReduce,
}

impl ConflictResolver {
    /// Detect conflicts in the parse table
    pub fn detect_conflicts(
        item_sets: &ItemSetCollection,
        grammar: &Grammar,
        _first_follow: &FirstFollowSets,
    ) -> Self {
        let mut conflicts = Vec::new();
        
        for item_set in &item_sets.sets {
            let mut actions_by_symbol: IndexMap<SymbolId, Vec<Action>> = IndexMap::new();
            
            // Collect all possible actions for each symbol in this state
            for item in &item_set.items {
                if item.is_reduce_item(grammar) {
                    // Reduce action
                    let action = if item.rule_id.0 == 0 { // Assuming rule 0 is the augmented start rule
                        Action::Accept
                    } else {
                        Action::Reduce(item.rule_id)
                    };
                    
                    actions_by_symbol
                        .entry(item.lookahead)
                        .or_insert_with(Vec::new)
                        .push(action);
                } else if let Some(symbol) = item.next_symbol(grammar) {
                    // Shift action
                    let symbol_id = match symbol {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => *id,
                    };
                    
                    if let Some(target_state) = item_sets.goto_table.get(&(item_set.id, symbol_id)) {
                        let action = Action::Shift(*target_state);
                        actions_by_symbol
                            .entry(symbol_id)
                            .or_insert_with(Vec::new)
                            .push(action);
                    }
                }
            }
            
            // Check for conflicts
            for (symbol_id, actions) in actions_by_symbol {
                if actions.len() > 1 {
                    let conflict_type = if actions.iter().any(|a| matches!(a, Action::Shift(_))) &&
                                         actions.iter().any(|a| matches!(a, Action::Reduce(_))) {
                        ConflictType::ShiftReduce
                    } else {
                        ConflictType::ReduceReduce
                    };
                    
                    conflicts.push(Conflict {
                        state: item_set.id,
                        symbol: symbol_id,
                        actions,
                        conflict_type,
                    });
                }
            }
        }
        
        Self { conflicts }
    }

    /// Resolve conflicts using precedence and associativity rules
    pub fn resolve_conflicts(&mut self, grammar: &Grammar) {
        // Clone conflicts to avoid borrowing issues
        let mut conflicts_to_resolve = self.conflicts.clone();
        for conflict in &mut conflicts_to_resolve {
            // Apply Tree-sitter's exact conflict resolution logic
            self.resolve_single_conflict(conflict, grammar);
        }
        self.conflicts = conflicts_to_resolve;
    }

    fn resolve_single_conflict(&self, conflict: &mut Conflict, grammar: &Grammar) {
        // Implement Tree-sitter's exact precedence and associativity resolution
        // This is where we port the C logic for conflict resolution
        
        match conflict.conflict_type {
            ConflictType::ShiftReduce => {
                // Apply precedence rules between shift and reduce
                // Higher precedence wins, same precedence uses associativity
                self.resolve_shift_reduce_conflict(conflict, grammar);
            }
            ConflictType::ReduceReduce => {
                // Apply precedence rules between multiple reduces
                // Usually choose the rule that appears first in the grammar
                self.resolve_reduce_reduce_conflict(conflict, grammar);
            }
        }
    }

    fn resolve_shift_reduce_conflict(&self, conflict: &mut Conflict, grammar: &Grammar) {
        // Use Tree-sitter's exact precedence comparison logic
        let precedence_resolver = StaticPrecedenceResolver::from_grammar(grammar);
        
        let mut shift_action = None;
        let mut reduce_action = None;
        
        // Find shift and reduce actions
        for action in &conflict.actions {
            match action {
                Action::Shift(_) => shift_action = Some(action.clone()),
                Action::Reduce(_) => reduce_action = Some(action.clone()),
                _ => {}
            }
        }
        
        match (shift_action, reduce_action) {
            (Some(shift), Some(reduce)) => {
                // Get precedence info for shift token
                let shift_prec = precedence_resolver.token_precedence(conflict.symbol);
                
                // Get precedence info for reduce rule
                let reduce_prec = if let Action::Reduce(rule_id) = &reduce {
                    precedence_resolver.rule_precedence(*rule_id)
                } else {
                    None
                };
                
                // Compare precedences
                match compare_precedences(shift_prec, reduce_prec) {
                    PrecedenceComparison::PreferShift => {
                        conflict.actions = vec![shift];
                    }
                    PrecedenceComparison::PreferReduce => {
                        conflict.actions = vec![reduce];
                    }
                    PrecedenceComparison::Error => {
                        // Non-associative conflict - this is an error
                        // For now, keep both actions (GLR will handle it)
                        conflict.actions = vec![Action::Fork(vec![shift, reduce])];
                    }
                    PrecedenceComparison::None => {
                        // No precedence info - use GLR fork
                        conflict.actions = vec![Action::Fork(vec![shift, reduce])];
                    }
                }
            }
            _ => {
                // Should not happen in a shift/reduce conflict
                // Keep original actions
            }
        }
    }

    fn resolve_reduce_reduce_conflict(&self, conflict: &mut Conflict, _grammar: &Grammar) {
        // Choose the rule that appears first in the grammar
        // This is Tree-sitter's default behavior for reduce/reduce conflicts
        
        let mut best_action = None;
        let mut best_rule_id = u16::MAX;
        
        for action in &conflict.actions {
            if let Action::Reduce(rule_id) = action {
                if rule_id.0 < best_rule_id {
                    best_rule_id = rule_id.0;
                    best_action = Some(action.clone());
                }
            }
        }
        
        if let Some(action) = best_action {
            conflict.actions = vec![action];
        }
    }
}

/// Error types for GLR processing
#[derive(Debug, thiserror::Error)]
pub enum GLRError {
    #[error("Grammar error: {0}")]
    GrammarError(#[from] GrammarError),
    
    #[error("Conflict resolution failed: {0}")]
    ConflictResolution(String),
    
    #[error("State machine generation failed: {0}")]
    StateMachine(String),
}

/// Build LR(1) automaton (parse table) from grammar
pub fn build_lr1_automaton(grammar: &Grammar, first_follow: &FirstFollowSets) -> Result<ParseTable, GLRError> {
    // Build canonical collection of LR(1) item sets
    let collection = ItemSetCollection::build_canonical_collection(grammar, first_follow);
    
    // Create mapping from symbol IDs to table indices
    let mut symbol_to_index = HashMap::new();
    let mut max_symbol_id = 0u16;
    
    // Map all token IDs
    for &symbol_id in grammar.tokens.keys() {
        max_symbol_id = max_symbol_id.max(symbol_id.0);
        symbol_to_index.insert(symbol_id, symbol_to_index.len());
    }
    
    // Map all non-terminal symbols (LHS of rules)
    let mut non_terminals = HashSet::new();
    for rule in grammar.rules.values() {
        non_terminals.insert(rule.lhs);
    }
    for &symbol_id in &non_terminals {
        max_symbol_id = max_symbol_id.max(symbol_id.0);
        symbol_to_index.insert(symbol_id, symbol_to_index.len());
    }
    
    // Map all external IDs
    for external in &grammar.externals {
        max_symbol_id = max_symbol_id.max(external.symbol_id.0);
        symbol_to_index.insert(external.symbol_id, symbol_to_index.len());
    }
    
    // Add EOF symbol (ID 0 is reserved for EOF in Tree-sitter)
    symbol_to_index.insert(SymbolId(0), symbol_to_index.len());
    
    // Create parse table with proper dimensions
    let state_count = collection.sets.len();
    let indexed_symbol_count = symbol_to_index.len();
    let symbol_count = indexed_symbol_count; // Keep for compatibility
    
    let mut action_table = vec![vec![Action::Error; indexed_symbol_count]; state_count];
    let mut goto_table = vec![vec![StateId(0); indexed_symbol_count]; state_count];
    
    // Fill action table
    for item_set in &collection.sets {
        let state_idx = item_set.id.0 as usize;
        
        for item in &item_set.items {
            if item.is_reduce_item(grammar) {
                // Add reduce action
                if let Some(&lookahead_idx) = symbol_to_index.get(&item.lookahead) {
                    // Check if this is reducing the start rule with EOF lookahead
                    if let Some(start_rule) = grammar.rules.values().next() {
                        if item.rule_id.0 == start_rule.production_id.0 && item.lookahead == SymbolId(0) {
                            // This is the accept state
                            action_table[state_idx][lookahead_idx] = Action::Accept;
                        } else {
                            // Normal reduce
                            action_table[state_idx][lookahead_idx] = Action::Reduce(item.rule_id);
                        }
                    } else {
                        action_table[state_idx][lookahead_idx] = Action::Reduce(item.rule_id);
                    }
                }
            } else if let Some(next_symbol) = item.next_symbol(grammar) {
                let symbol_id = match &next_symbol {
                    Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id,
                };
                
                if let Some(&symbol_idx) = symbol_to_index.get(symbol_id) {
                    if let Symbol::Terminal(_) = next_symbol {
                        // Add shift action
                        if let Some(&goto_state) = collection.goto_table.get(&(item_set.id, *symbol_id)) {
                            action_table[state_idx][symbol_idx] = Action::Shift(goto_state);
                        }
                    }
                }
            }
        }
    }
    
    // Fill goto table from collection's goto_table
    for ((from_state, symbol), to_state) in &collection.goto_table {
        let from_idx = from_state.0 as usize;
        if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
            goto_table[from_idx][symbol_idx] = *to_state;
        }
    }
    
    
    // Build symbol metadata
    let mut symbol_metadata = Vec::new();
    
    // Add terminal symbols
    for (_, token) in &grammar.tokens {
        symbol_metadata.push(SymbolMetadata {
            name: token.name.clone(),
            visible: !token.name.starts_with('_'),
            named: !matches!(&token.pattern, TokenPattern::String(_)),
            supertype: false,
        });
    }
    
    // Add non-terminal symbols
    for (symbol_id, _) in &grammar.rules {
        let is_supertype = grammar.supertypes.contains(symbol_id);
        symbol_metadata.push(SymbolMetadata {
            name: format!("rule_{}", symbol_id.0),
            visible: true,
            named: true,
            supertype: is_supertype,
        });
    }
    
    // Add external symbols
    for external in &grammar.externals {
        symbol_metadata.push(SymbolMetadata {
            name: external.name.clone(),
            visible: !external.name.starts_with('_'),
            named: true,
            supertype: false,
        });
    }
    
    Ok(ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count,
        symbol_count,
        symbol_to_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lr_item_creation() {
        let item = LRItem::new(RuleId(1), 2, SymbolId(3));
        assert_eq!(item.rule_id, RuleId(1));
        assert_eq!(item.position, 2);
        assert_eq!(item.lookahead, SymbolId(3));
    }

    #[test]
    fn test_lr_item_equality() {
        let item1 = LRItem::new(RuleId(1), 2, SymbolId(3));
        let item2 = LRItem::new(RuleId(1), 2, SymbolId(3));
        let item3 = LRItem::new(RuleId(1), 3, SymbolId(3));
        
        assert_eq!(item1, item2);
        assert_ne!(item1, item3);
        
        // Test hashing
        let mut set = std::collections::HashSet::new();
        set.insert(item1.clone());
        assert!(set.contains(&item1));
        assert!(set.contains(&item2));
        assert!(!set.contains(&item3));
    }

    #[test]
    fn test_item_set_creation() {
        let mut item_set = ItemSet::new(StateId(0));
        let item = LRItem::new(RuleId(1), 0, SymbolId(0));
        item_set.add_item(item.clone());
        
        assert_eq!(item_set.id, StateId(0));
        assert!(item_set.items.contains(&item));
        assert_eq!(item_set.items.len(), 1);
    }

    #[test]
    fn test_item_set_duplicate_items() {
        let mut item_set = ItemSet::new(StateId(0));
        let item = LRItem::new(RuleId(1), 0, SymbolId(0));
        
        item_set.add_item(item.clone());
        item_set.add_item(item.clone()); // Add same item again
        
        // Should only contain one item (no duplicates)
        assert_eq!(item_set.items.len(), 1);
    }

    #[test]
    fn test_first_follow_empty_grammar() {
        let grammar = Grammar::new("test".to_string());
        let first_follow = FirstFollowSets::compute(&grammar);
        
        assert!(first_follow.first.is_empty());
        assert!(first_follow.follow.is_empty());
    }

    #[test]
    fn test_first_follow_simple_grammar() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add a simple rule: S -> a
        let rule = Rule {
            lhs: SymbolId(0), // S
            rhs: vec![Symbol::Terminal(SymbolId(1))], // a
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.insert(SymbolId(0), rule);
        
        // Add the terminal token
        let token = Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(1), token);
        
        let first_follow = FirstFollowSets::compute(&grammar);
        
        // FIRST(S) should contain 'a'
        assert!(first_follow.first.contains_key(&SymbolId(0)));
        if let Some(first_s) = first_follow.first(SymbolId(0)) {
            assert!(first_s.contains(1)); // Terminal 'a' has id 1
        }
        
        // S should not be nullable
        assert!(!first_follow.is_nullable(SymbolId(0)));
    }

    #[test]
    fn test_first_follow_nullable_rule() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add a rule: S -> ε (empty rule)
        let rule = Rule {
            lhs: SymbolId(0), // S
            rhs: vec![], // empty
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.insert(SymbolId(0), rule);
        
        let first_follow = FirstFollowSets::compute(&grammar);
        
        // S should be nullable
        assert!(first_follow.is_nullable(SymbolId(0)));
    }

    #[test]
    fn test_first_of_sequence() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add tokens
        let token_a = Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(1), token_a);
        
        let token_b = Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(2), token_b);
        
        let first_follow = FirstFollowSets::compute(&grammar);
        
        // Test FIRST of sequence [a, b]
        let sequence = vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))];
        let first_seq = first_follow.first_of_sequence(&sequence);
        
        // Should contain only 'a' (first terminal)
        assert!(first_seq.contains(1));
        assert!(!first_seq.contains(2));
    }

    #[test]
    fn test_action_types() {
        let shift = Action::Shift(StateId(1));
        let reduce = Action::Reduce(RuleId(2));
        let accept = Action::Accept;
        let error = Action::Error;
        let fork = Action::Fork(vec![shift.clone(), reduce.clone()]);
        
        match shift {
            Action::Shift(StateId(1)) => {},
            _ => panic!("Expected shift action"),
        }
        
        match reduce {
            Action::Reduce(RuleId(2)) => {},
            _ => panic!("Expected reduce action"),
        }
        
        match accept {
            Action::Accept => {},
            _ => panic!("Expected accept action"),
        }
        
        match error {
            Action::Error => {},
            _ => panic!("Expected error action"),
        }
        
        match fork {
            Action::Fork(actions) => {
                assert_eq!(actions.len(), 2);
                assert_eq!(actions[0], shift);
                assert_eq!(actions[1], reduce);
            },
            _ => panic!("Expected fork action"),
        }
    }

    #[test]
    fn test_action_equality() {
        let shift1 = Action::Shift(StateId(1));
        let shift2 = Action::Shift(StateId(1));
        let shift3 = Action::Shift(StateId(2));
        
        assert_eq!(shift1, shift2);
        assert_ne!(shift1, shift3);
        
        let reduce1 = Action::Reduce(RuleId(1));
        let reduce2 = Action::Reduce(RuleId(1));
        
        assert_eq!(reduce1, reduce2);
        assert_ne!(shift1, reduce1);
    }

    #[test]
    fn test_symbol_metadata() {
        let metadata = SymbolMetadata {
            name: "expression".to_string(),
            visible: true,
            named: true,
            supertype: false,
        };
        
        assert_eq!(metadata.name, "expression");
        assert!(metadata.visible);
        assert!(metadata.named);
        assert!(!metadata.supertype);
    }

    #[test]
    fn test_conflict_types() {
        let shift_reduce = ConflictType::ShiftReduce;
        let reduce_reduce = ConflictType::ReduceReduce;
        
        assert_eq!(shift_reduce, ConflictType::ShiftReduce);
        assert_eq!(reduce_reduce, ConflictType::ReduceReduce);
        assert_ne!(shift_reduce, reduce_reduce);
    }

    #[test]
    fn test_conflict_creation() {
        let conflict = Conflict {
            state: StateId(5),
            symbol: SymbolId(10),
            actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(2))],
            conflict_type: ConflictType::ShiftReduce,
        };
        
        assert_eq!(conflict.state, StateId(5));
        assert_eq!(conflict.symbol, SymbolId(10));
        assert_eq!(conflict.actions.len(), 2);
        assert_eq!(conflict.conflict_type, ConflictType::ShiftReduce);
    }

    #[test]
    fn test_conflict_resolver_creation() {
        let resolver = ConflictResolver {
            conflicts: vec![],
        };
        
        assert!(resolver.conflicts.is_empty());
    }

    #[test]
    fn test_parse_table_creation() {
        let parse_table = ParseTable {
            action_table: vec![vec![Action::Error; 5]; 3], // 3 states, 5 symbols
            goto_table: vec![vec![StateId(0); 5]; 3],
            symbol_metadata: vec![],
            state_count: 3,
            symbol_count: 5,
            symbol_to_index: HashMap::new(),
        };
        
        assert_eq!(parse_table.state_count, 3);
        assert_eq!(parse_table.symbol_count, 5);
        assert_eq!(parse_table.action_table.len(), 3);
        assert_eq!(parse_table.goto_table.len(), 3);
        assert_eq!(parse_table.action_table[0].len(), 5);
        assert_eq!(parse_table.goto_table[0].len(), 5);
    }

    #[test]
    fn test_lr_item_reduce_check() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add a rule: S -> a b
        let rule = Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.insert(SymbolId(0), rule);
        
        // Item at position 0: S -> • a b
        let item1 = LRItem::new(RuleId(0), 0, SymbolId(0));
        assert!(!item1.is_reduce_item(&grammar));
        
        // Item at position 1: S -> a • b
        let item2 = LRItem::new(RuleId(0), 1, SymbolId(0));
        assert!(!item2.is_reduce_item(&grammar));
        
        // Item at position 2: S -> a b •
        let item3 = LRItem::new(RuleId(0), 2, SymbolId(0));
        assert!(item3.is_reduce_item(&grammar));
    }

    #[test]
    fn test_lr_item_next_symbol() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add a rule: S -> a b
        let rule = Rule {
            lhs: SymbolId(0),
            rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.insert(SymbolId(0), rule);
        
        // Item at position 0: S -> • a b
        let item1 = LRItem::new(RuleId(0), 0, SymbolId(0));
        if let Some(symbol) = item1.next_symbol(&grammar) {
            match symbol {
                Symbol::Terminal(SymbolId(1)) => {},
                _ => panic!("Expected terminal symbol with id 1"),
            }
        } else {
            panic!("Expected next symbol");
        }
        
        // Item at position 1: S -> a • b
        let item2 = LRItem::new(RuleId(0), 1, SymbolId(0));
        if let Some(symbol) = item2.next_symbol(&grammar) {
            match symbol {
                Symbol::Terminal(SymbolId(2)) => {},
                _ => panic!("Expected terminal symbol with id 2"),
            }
        } else {
            panic!("Expected next symbol");
        }
        
        // Item at position 2: S -> a b •
        let item3 = LRItem::new(RuleId(0), 2, SymbolId(0));
        assert!(item3.next_symbol(&grammar).is_none());
    }

    #[test]
    fn test_item_set_collection_creation() {
        let collection = ItemSetCollection {
            sets: vec![],
            goto_table: IndexMap::new(),
        };
        
        assert!(collection.sets.is_empty());
        assert!(collection.goto_table.is_empty());
    }

    #[test]
    fn test_glr_error_types() {
        let grammar_error = GLRError::GrammarError(GrammarError::InvalidFieldOrdering);
        let conflict_error = GLRError::ConflictResolution("Test conflict".to_string());
        let state_error = GLRError::StateMachine("Test state machine error".to_string());
        
        match grammar_error {
            GLRError::GrammarError(_) => {},
            _ => panic!("Expected grammar error"),
        }
        
        match conflict_error {
            GLRError::ConflictResolution(msg) => assert_eq!(msg, "Test conflict"),
            _ => panic!("Expected conflict resolution error"),
        }
        
        match state_error {
            GLRError::StateMachine(msg) => assert_eq!(msg, "Test state machine error"),
            _ => panic!("Expected state machine error"),
        }
    }

    #[test]
    fn test_item_set_equality() {
        let mut set1 = ItemSet::new(StateId(0));
        let mut set2 = ItemSet::new(StateId(1));
        
        let item1 = LRItem::new(RuleId(1), 0, SymbolId(0));
        let item2 = LRItem::new(RuleId(2), 1, SymbolId(1));
        
        set1.add_item(item1.clone());
        set1.add_item(item2.clone());
        
        set2.add_item(item1);
        set2.add_item(item2);
        
        // Sets should be equal based on items, not ID
        assert_eq!(set1.items, set2.items);
        assert_ne!(set1.id, set2.id);
    }
}