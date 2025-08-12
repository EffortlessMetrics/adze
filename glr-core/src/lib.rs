// GLR core may need unsafe for performance-critical parser algorithms
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), warn(missing_docs))]

//! GLR parser generation algorithms for pure-Rust Tree-sitter
//! This module implements the core GLR state machine generation and conflict resolution

use fixedbitset::FixedBitSet;
use indexmap::IndexMap;
use rust_sitter_ir::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Error types and Result alias for GLR operations.
pub mod error;
/// Back-compat alias: prefer `GlrError`; `GLRError` remains for now.
pub use GLRError as GlrError;
pub use error::Result as GlrResult;

/// Stable imports for downstream users during 0.8.0-dev.
pub mod prelude {
    pub use crate::{ParseTable, FirstFollowSets, build_lr1_automaton};
}

// Keep available, but don't promise public docs yet:
#[doc(hidden)]
pub mod advanced_conflict;
#[doc(hidden)]
pub mod conflict_resolution;
#[doc(hidden)]
pub mod conflict_visualizer;
#[doc(hidden)]
pub mod disambiguation;
#[doc(hidden)]
pub mod gss;
#[doc(hidden)]
pub mod gss_arena;
#[doc(hidden)]
pub mod parse_forest;
#[doc(hidden)]
pub mod perf_optimizations;
#[doc(hidden)]
pub mod precedence_compare;
#[doc(hidden)]
pub mod symbol_comparison;
#[doc(hidden)]
pub mod version_info;

#[doc(hidden)]
pub use advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
#[doc(hidden)]
pub use conflict_resolution::{RuntimeConflictResolver, VecWrapperResolver};
#[doc(hidden)]
pub use conflict_visualizer::{ConflictVisualizer, generate_dot_graph};
#[doc(hidden)]
pub use gss::{GSSStats, GraphStructuredStack, StackNode};
#[doc(hidden)]
pub use parse_forest::{ForestNode, ParseError, ParseForest, ParseNode, ParseTree};
#[doc(hidden)]
pub use perf_optimizations::{ParseTableCache, PerfStats, StackDeduplicator, StackPool};
#[doc(hidden)]
pub use precedence_compare::{
    PrecedenceComparison, PrecedenceInfo, StaticPrecedenceResolver, compare_precedences,
};
#[doc(hidden)]
pub use symbol_comparison::{compare_symbols, compare_versions_with_symbols};
#[doc(hidden)]
pub use version_info::{CompareResult, VersionInfo, compare_versions};

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
    fn get_max_symbol_id(symbol: &Symbol) -> u16 {
        match symbol {
            Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id.0,
            Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
                Self::get_max_symbol_id(inner)
            }
            Symbol::Choice(choices) => choices
                .iter()
                .map(Self::get_max_symbol_id)
                .max()
                .unwrap_or(0),
            Symbol::Sequence(seq) => seq.iter().map(Self::get_max_symbol_id).max().unwrap_or(0),
            Symbol::Epsilon => 0,
        }
    }
    /// Compute FIRST/FOLLOW sets for the given grammar
    pub fn compute(grammar: &Grammar) -> Self {
        // Find the maximum symbol ID to determine the size needed
        let max_rule_id = grammar.rules.keys().map(|id| id.0).max().unwrap_or(0);
        let max_token_id = grammar.tokens.keys().map(|id| id.0).max().unwrap_or(0);
        let max_external_id = grammar
            .externals
            .iter()
            .map(|e| e.symbol_id.0)
            .max()
            .unwrap_or(0);

        // Also check max symbol ID in all rule RHS
        let mut max_rhs_id = 0u16;
        for rules in grammar.rules.values() {
            for rule in rules {
                for symbol in &rule.rhs {
                    max_rhs_id = max_rhs_id.max(Self::get_max_symbol_id(symbol));
                }
            }
        }

        let symbol_count = (max_rule_id
            .max(max_token_id)
            .max(max_external_id)
            .max(max_rhs_id)
            + 1) as usize;

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

            for rule in grammar.all_rules() {
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
                        Symbol::Epsilon => {
                            // Epsilon doesn't contribute to FIRST set
                            // but keeps rule nullable
                        }
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_) => {
                            // These should be normalized before FIRST/FOLLOW computation
                            panic!(
                                "Complex symbols should be normalized before FIRST/FOLLOW computation"
                            );
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
        // Initialize FOLLOW(start_symbol) with EOF
        if let Some(start_symbol) = grammar.start_symbol() {
            if let Some(follow_set) = follow.get_mut(&start_symbol) {
                follow_set.insert(0); // EOF symbol
            }
        }

        changed = true;
        while changed {
            changed = false;

            for rule in grammar.all_rules() {
                // Special handling for rules of the form A -> A B (left recursion)
                if rule.rhs.len() >= 2 {
                    if let (Symbol::NonTerminal(first_id), Symbol::NonTerminal(second_id)) =
                        (&rule.rhs[0], &rule.rhs[1])
                    {
                        if *first_id == rule.lhs {
                            // This is a left-recursive rule like Module_body_vec_contents -> Module_body_vec_contents Statement
                            // FIRST(Statement) should be in FOLLOW(Module_body_vec_contents)
                            if let Some(first_of_second) = first.get(second_id) {
                                if let Some(follow_set) = follow.get_mut(&rule.lhs) {
                                    let old_len = follow_set.count_ones(..);
                                    follow_set.union_with(first_of_second);
                                    if follow_set.count_ones(..) > old_len {
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                }

                for (i, symbol) in rule.rhs.iter().enumerate() {
                    if let Symbol::NonTerminal(id) | Symbol::External(id) = symbol {
                        // Add FIRST of remaining symbols to FOLLOW of current symbol
                        let remaining = &rule.rhs[i + 1..];
                        let first_of_remaining =
                            Self::first_of_sequence_static(remaining, &first, &nullable);

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
        let mut result = FixedBitSet::with_capacity(nullable.len());

        for symbol in symbols {
            match symbol {
                Symbol::Terminal(id) => {
                    result.insert(id.0 as usize);
                    break;
                }
                Symbol::Epsilon => {
                    // Epsilon doesn't contribute to FIRST set, continue to next symbol
                }
                Symbol::NonTerminal(id) | Symbol::External(id) => {
                    if let Some(symbol_first) = first.get(id) {
                        result.union_with(symbol_first);
                    }

                    if !nullable.contains(id.0 as usize) {
                        break;
                    }
                }
                Symbol::Optional(_)
                | Symbol::Repeat(_)
                | Symbol::RepeatOne(_)
                | Symbol::Choice(_)
                | Symbol::Sequence(_) => {
                    panic!("Complex symbols should be normalized before FIRST/FOLLOW computation");
                }
            }
        }

        result
    }

    fn sequence_is_nullable(symbols: &[Symbol], nullable: &FixedBitSet) -> bool {
        symbols.iter().all(|symbol| match symbol {
            Symbol::Terminal(_) => false,
            Symbol::NonTerminal(id) | Symbol::External(id) => nullable.contains(id.0 as usize),
            Symbol::Epsilon => true,
            Symbol::Optional(_)
            | Symbol::Repeat(_)
            | Symbol::RepeatOne(_)
            | Symbol::Choice(_)
            | Symbol::Sequence(_) => {
                panic!("Complex symbols should be normalized before FIRST/FOLLOW computation");
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
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LRItem {
    /// Owning rule for this item/state
    pub rule_id: RuleId,
    /// Position within the rule's RHS
    pub position: usize,
    /// Lookahead symbol for LR(1) parsing
    pub lookahead: SymbolId,
}

impl LRItem {
    /// Construct an `LRItem` from its owning rule, dot position, and lookahead symbol.
    pub fn new(rule_id: RuleId, position: usize, lookahead: SymbolId) -> Self {
        Self {
            rule_id,
            position,
            lookahead,
        }
    }

    /// Check if this item is at the end of the rule (reduce item)
    pub fn is_reduce_item(&self, grammar: &Grammar) -> bool {
        if let Some(rule) = grammar
            .all_rules()
            .find(|r| r.production_id.0 == self.rule_id.0)
        {
            self.position >= rule.rhs.len()
        } else {
            false
        }
    }

    /// Get the symbol after the dot (next symbol to parse)
    pub fn next_symbol<'a>(&self, grammar: &'a Grammar) -> Option<&'a Symbol> {
        if let Some(rule) = grammar
            .all_rules()
            .find(|r| r.production_id.0 == self.rule_id.0)
        {
            rule.rhs.get(self.position)
        } else {
            None
        }
    }
}

/// Set of LR(1) items representing a parser state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSet {
    pub items: BTreeSet<LRItem>,
    pub id: StateId,
}

impl ItemSet {
    pub fn new(id: StateId) -> Self {
        Self {
            items: BTreeSet::new(),
            id,
        }
    }

    pub fn add_item(&mut self, item: LRItem) {
        self.items.insert(item);
    }

    /// Compute closure of this item set
    pub fn closure(&mut self, grammar: &Grammar, first_follow: &FirstFollowSets) {
        let _initial_size = self.items.len();

        let mut added = true;
        let mut _iteration = 0;
        while added {
            added = false;
            _iteration += 1;
            let current_items: Vec<_> = self.items.iter().cloned().collect();

            for item in current_items {
                if let Some(Symbol::NonTerminal(symbol_id)) = item.next_symbol(grammar) {
                    // Find all rules with this symbol as LHS
                    if let Some(rules) = grammar.get_rules_for_symbol(*symbol_id) {
                        for rule in rules {
                            // Compute FIRST of β α where β is the rest of the current rule
                            // and α is the lookahead
                            let mut beta = Vec::new();
                            if let Some(current_rule) = grammar
                                .all_rules()
                                .find(|r| r.production_id.0 == item.rule_id.0)
                            {
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
                                    if rule.rhs.is_empty() {
                                        // Empty production
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Closure complete
    }

    /// Compute GOTO for a given symbol
    pub fn goto(
        &self,
        symbol: &Symbol,
        grammar: &Grammar,
        _first_follow: &FirstFollowSets,
    ) -> ItemSet {
        let mut new_set = ItemSet::new(StateId(0)); // ID will be assigned later

        // Add all items where the dot can advance over the given symbol
        for item in &self.items {
            if let Some(next_sym) = item.next_symbol(grammar) {
                if std::mem::discriminant(next_sym) == std::mem::discriminant(symbol) {
                    match (next_sym, symbol) {
                        (Symbol::Terminal(a), Symbol::Terminal(b))
                        | (Symbol::NonTerminal(a), Symbol::NonTerminal(b))
                        | (Symbol::External(a), Symbol::External(b))
                            if a == b =>
                        {
                            let new_item =
                                LRItem::new(item.rule_id, item.position + 1, item.lookahead);
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
    /// Build canonical collection of LR(1) item sets for augmented grammar
    pub fn build_canonical_collection_augmented(
        grammar: &Grammar,
        first_follow: &FirstFollowSets,
        augmented_start: SymbolId,
        _original_start: SymbolId,
    ) -> Self {
        let mut collection = Self {
            sets: Vec::new(),
            goto_table: IndexMap::new(),
        };

        // Create initial state with the augmented start rule S' -> S $
        let mut initial_set = ItemSet::new(StateId(0));

        // Find the augmented start rule
        if let Some(augmented_rules) = grammar.get_rules_for_symbol(augmented_start) {
            for rule in augmented_rules {
                // Add S' -> • S with lookahead $ (EOF)
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
        eprintln!(
            "Initial state 0 after closure has {} items:",
            initial_set.items.len()
        );
        for item in &initial_set.items {
            // Print each item to debug
            if let Some(rule) = grammar
                .all_rules()
                .find(|r| r.production_id.0 == item.rule_id.0)
            {
                let mut rhs_str = String::new();
                for (idx, sym) in rule.rhs.iter().enumerate() {
                    if idx == item.position {
                        rhs_str.push_str(" • ");
                    }
                    match sym {
                        Symbol::Terminal(id) => rhs_str.push_str(&format!("T({}) ", id.0)),
                        Symbol::NonTerminal(id) => rhs_str.push_str(&format!("NT({}) ", id.0)),
                        _ => rhs_str.push_str("? "),
                    }
                }
                if item.position == rule.rhs.len() {
                    rhs_str.push_str(" • ");
                }
                eprintln!(
                    "  Item: NT({}) -> {}, lookahead={}",
                    rule.lhs.0, rhs_str, item.lookahead.0
                );
            }
        }

        collection.sets.push(initial_set);
        let mut state_counter = 1;

        // Build all reachable states (same as before)
        let mut i = 0;
        while i < collection.sets.len() {
            let current_set = collection.sets[i].clone();

            // Debug: Print all items in this state
            for item in &current_set.items {
                if let Some(rule) = grammar
                    .all_rules()
                    .find(|r| r.production_id.0 == item.rule_id.0)
                {
                    let mut rhs_str = String::new();
                    for (idx, sym) in rule.rhs.iter().enumerate() {
                        if idx == item.position {
                            rhs_str.push_str(" • ");
                        }
                        rhs_str.push_str(&format!("{:?} ", sym));
                    }
                    if item.position == rule.rhs.len() {
                        rhs_str.push_str(" • ");
                    }
                    // "  [{}] {:?} -> {} , lookahead={}"
                }
            }

            // Find all symbols that can be shifted from this state
            let mut symbols = BTreeSet::new();
            let mut _terminal_count = 0;
            let mut _non_terminal_count = 0;
            if i == 0 {
                eprintln!("State 0: Finding symbols that can be shifted...");
            }
            for item in &current_set.items {
                if let Some(symbol) = item.next_symbol(grammar) {
                    match symbol {
                        Symbol::Terminal(_id) => {
                            _terminal_count += 1;
                        }
                        Symbol::NonTerminal(_id) => {
                            _non_terminal_count += 1;
                        }
                        Symbol::External(_id) => {
                            _terminal_count += 1; // Count externals as terminals
                        }
                        _ => {}
                    }
                    symbols.insert(symbol.clone());
                    if i == 0 {
                        // Check if this is 'def'
                        if let Symbol::Terminal(id) = &symbol {
                            if let Some(token) = grammar.tokens.get(id) {
                                if matches!(token.pattern, TokenPattern::String(ref s) if s == "def")
                                {
                                    eprintln!("  Found 'def' as shiftable symbol: {:?}", symbol);
                                }
                            }
                        }
                        eprintln!("  Can shift symbol: {:?}", symbol);
                    }
                }
            }

            // Debug: symbols.len(), terminal_count, non_terminal_count
            // Compute GOTO for each symbol
            for symbol in symbols {
                let goto_set = current_set.goto(&symbol, grammar, first_follow);

                if !goto_set.items.is_empty() {
                    // Check if this set already exists
                    let existing_state = collection
                        .sets
                        .iter()
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
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => {
                            panic!(
                                "Complex symbols should be normalized before LR item generation"
                            );
                        }
                    };
                    if current_set.id.0 == 0 {
                        eprintln!(
                            "  State 0 GOTO: symbol {:?} -> state {}",
                            symbol_id, target_state.0
                        );
                    }
                    collection
                        .goto_table
                        .insert((current_set.id, symbol_id), target_state);
                    // "DEBUG: Added goto({}, {}) = {}"
                }
            }

            i += 1;
        }

        collection
    }

    /// Build canonical collection of LR(1) item sets
    pub fn build_canonical_collection(grammar: &Grammar, first_follow: &FirstFollowSets) -> Self {
        let mut collection = Self {
            sets: Vec::new(),
            goto_table: IndexMap::new(),
        };

        // Create initial state with augmented start rule
        let mut initial_set = ItemSet::new(StateId(0));

        // Find the start symbol (LHS of the first rule in grammar)
        if let Some(start_symbol) = grammar.start_symbol() {
            // Debug: grammar.rule_names.get(&start_symbol)

            // Add items for ALL rules with the start symbol as LHS
            if let Some(start_rules) = grammar.get_rules_for_symbol(start_symbol) {
                for rule in start_rules.iter() {
                    // Debug: idx, rule.lhs, rule.rhs, rule.production_id.0
                    let start_item = LRItem::new(
                        RuleId(rule.production_id.0),
                        0,
                        SymbolId(0), // EOF symbol
                    );
                    initial_set.add_item(start_item);
                    // Debug: rule.production_id.0
                }
            }

            // Compute closure
            initial_set.closure(grammar, first_follow);
        }

        // Only add initial set if it has items
        if initial_set.items.is_empty() {
            // Handle empty initial set if needed
        } else {
            for _item in &initial_set.items {
                // Debug: item.rule_id.0, item.position, item.lookahead.0
            }
        }

        collection.sets.push(initial_set);
        let mut state_counter = 1;

        // Build all reachable states
        let mut i = 0;
        while i < collection.sets.len() {
            let current_set = collection.sets[i].clone();

            // Debug: Print all items in this state
            for item in &current_set.items {
                if let Some(rule) = grammar
                    .all_rules()
                    .find(|r| r.production_id.0 == item.rule_id.0)
                {
                    let mut rhs_str = String::new();
                    for (idx, sym) in rule.rhs.iter().enumerate() {
                        if idx == item.position {
                            rhs_str.push_str(" • ");
                        }
                        rhs_str.push_str(&format!("{:?} ", sym));
                    }
                    if item.position == rule.rhs.len() {
                        rhs_str.push_str(" • ");
                    }
                    // "  [{}] {:?} -> {} , lookahead={}"
                }
            }

            // Find all symbols that can be shifted from this state
            let mut symbols = BTreeSet::new();
            let mut _terminal_count = 0;
            let mut _non_terminal_count = 0;
            if i == 0 {
                eprintln!("State 0: Finding symbols that can be shifted...");
            }
            for item in &current_set.items {
                if let Some(symbol) = item.next_symbol(grammar) {
                    match symbol {
                        Symbol::Terminal(_id) => {
                            _terminal_count += 1;
                        }
                        Symbol::NonTerminal(_id) => {
                            _non_terminal_count += 1;
                        }
                        Symbol::External(_id) => {
                            _terminal_count += 1; // Count externals as terminals
                        }
                        _ => {}
                    }
                    symbols.insert(symbol.clone());
                    if i == 0 {
                        // Check if this is 'def'
                        if let Symbol::Terminal(id) = &symbol {
                            if let Some(token) = grammar.tokens.get(id) {
                                if matches!(token.pattern, TokenPattern::String(ref s) if s == "def")
                                {
                                    eprintln!("  Found 'def' as shiftable symbol: {:?}", symbol);
                                }
                            }
                        }
                        eprintln!("  Can shift symbol: {:?}", symbol);
                    }
                }
            }

            // Debug: symbols.len(), terminal_count, non_terminal_count
            for item in &current_set.items {
                if let Some(symbol) = item.next_symbol(grammar) {
                    let _symbol_id = match &symbol {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id,
                        _ => panic!("Complex symbol"),
                    };
                    // "  Item rule_id={}, position={}, next_symbol={:?} (id={})"
                }
            }

            for symbol in &symbols {
                let _symbol_id = match symbol {
                    Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => id,
                    _ => panic!("Complex symbol"),
                };
            }

            // Compute GOTO for each symbol
            for symbol in symbols {
                let goto_set = current_set.goto(&symbol, grammar, first_follow);

                if !goto_set.items.is_empty() {
                    // Check if this set already exists
                    let existing_state = collection
                        .sets
                        .iter()
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
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => {
                            panic!(
                                "Complex symbols should be normalized before LR item generation"
                            );
                        }
                    };
                    if current_set.id.0 == 0 {
                        eprintln!(
                            "  State 0 GOTO: symbol {:?} -> state {}",
                            symbol_id, target_state.0
                        );
                    }
                    collection
                        .goto_table
                        .insert((current_set.id, symbol_id), target_state);
                    // "DEBUG: Added goto({}, {}) = {}"
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
    pub action_table: Vec<Vec<ActionCell>>,
    pub goto_table: Vec<Vec<StateId>>,
    pub symbol_metadata: Vec<SymbolMetadata>,
    pub state_count: usize,
    pub symbol_count: usize,
    pub symbol_to_index: BTreeMap<SymbolId, usize>,
    /// For each state, a bitset indicating which external tokens are valid
    pub external_scanner_states: Vec<Vec<bool>>,
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

/// Action cell that can hold multiple actions for GLR
pub type ActionCell = Vec<Action>;

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
                    // Check if this is a reduction to the start symbol with EOF lookahead
                    let mut is_accept = false;

                    // Find the rule that corresponds to this rule ID
                    if let Some(start_symbol) = grammar.start_symbol() {
                        // Look through all rules to find the one with this rule ID
                        for rule in grammar.all_rules() {
                            if rule.production_id.0 == item.rule_id.0 {
                                // Check if this rule reduces to the start symbol and we have EOF lookahead
                                is_accept =
                                    rule.lhs == start_symbol && item.lookahead == SymbolId(0);
                                break;
                            }
                        }
                    }

                    let action = if is_accept {
                        Action::Accept
                    } else {
                        Action::Reduce(item.rule_id)
                    };

                    actions_by_symbol
                        .entry(item.lookahead)
                        .or_default()
                        .push(action);
                } else if let Some(symbol) = item.next_symbol(grammar) {
                    // Shift action
                    let symbol_id = match symbol {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => {
                            *id
                        }
                        Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => {
                            panic!(
                                "Complex symbols should be normalized before LR item generation"
                            );
                        }
                    };

                    if let Some(target_state) = item_sets.goto_table.get(&(item_set.id, symbol_id))
                    {
                        let action = Action::Shift(*target_state);
                        actions_by_symbol.entry(symbol_id).or_default().push(action);
                    }
                }
            }

            // Check for conflicts
            for (symbol_id, actions) in actions_by_symbol {
                if actions.len() > 1 {
                    let conflict_type = if actions.iter().any(|a| matches!(a, Action::Shift(_)))
                        && actions.iter().any(|a| matches!(a, Action::Reduce(_)))
                    {
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

/// Check if a symbol can derive the start symbol through unit productions
#[allow(dead_code)]
fn can_derive_start(grammar: &Grammar, symbol: SymbolId, start: SymbolId) -> bool {
    if symbol == start {
        return true;
    }

    // Check if there's a rule symbol -> start
    if let Some(rules) = grammar.get_rules_for_symbol(symbol) {
        for rule in rules {
            if rule.rhs.len() == 1 {
                if let Symbol::NonTerminal(target) = &rule.rhs[0] {
                    if *target == start {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Build LR(1) automaton (parse table) from grammar
pub fn build_lr1_automaton(
    grammar: &Grammar,
    first_follow: &FirstFollowSets,
) -> Result<ParseTable, GLRError> {
    // Debug: Print some rules to see their structure
    let mut rule_count = 0;
    for rule in grammar.all_rules() {
        if rule_count >= 10 {
            break;
        }
        let mut rhs_str = String::new();
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(id) => rhs_str.push_str(&format!("T({}) ", id.0)),
                Symbol::NonTerminal(id) => rhs_str.push_str(&format!("NT({}) ", id.0)),
                _ => rhs_str.push_str("? "),
            }
        }
        rule_count += 1;
    }

    // Create augmented grammar with S' -> S $ rule
    let mut augmented_grammar = grammar.clone();

    // Find the original start symbol
    let original_start =
        grammar
            .start_symbol()
            .ok_or(GLRError::GrammarError(GrammarError::UnresolvedSymbol(
                SymbolId(0),
            )))?;

    if let Some(_name) = grammar.rule_names.get(&original_start) {}

    // Create a new start symbol S' with a high ID that won't conflict
    let augmented_start = SymbolId(65535); // High ID to avoid conflicts

    // Add S' -> S rule (we'll handle $ implicitly in the LR construction)
    let augmented_rule = Rule {
        lhs: augmented_start,
        rhs: vec![Symbol::NonTerminal(original_start)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(65535), // High ID to avoid conflicts
    };
    augmented_grammar
        .rules
        .insert(augmented_start, vec![augmented_rule]);
    augmented_grammar
        .rule_names
        .insert(augmented_start, "$start".to_string());

    // "DEBUG: Added augmented start rule: {} -> {}"
    // Build canonical collection of LR(1) item sets with augmented grammar
    let collection = ItemSetCollection::build_canonical_collection_augmented(
        &augmented_grammar,
        first_follow,
        augmented_start,
        original_start,
    );

    // Create mapping from symbol IDs to table indices
    let mut symbol_to_index = BTreeMap::new();
    let mut max_symbol_id = 0u16;

    // IMPORTANT: EOF symbol (ID 0) must always have index 0 in Tree-sitter
    symbol_to_index.insert(SymbolId(0), 0);

    // Collect and sort symbols with proper ordering:
    // 1. Tokens first (terminals)
    // 2. Then non-terminals
    // 3. Then externals
    // Within each category, sort by symbol ID for determinism

    let mut token_symbols = Vec::new();
    let mut non_terminal_symbols = Vec::new();
    let mut external_symbols = Vec::new();

    // Collect token IDs
    for &symbol_id in grammar.tokens.keys() {
        token_symbols.push(symbol_id);
        max_symbol_id = max_symbol_id.max(symbol_id.0);
    }

    // Also collect terminals from rule RHS that might not be in grammar.tokens
    for rule in augmented_grammar.all_rules() {
        for symbol in &rule.rhs {
            if let Symbol::Terminal(id) = symbol {
                if !token_symbols.contains(id) {
                    token_symbols.push(*id);
                    max_symbol_id = max_symbol_id.max(id.0);
                }
            }
        }
    }

    token_symbols.sort_by_key(|s| s.0);

    // Collect non-terminal symbols (LHS of rules)
    let mut non_terminals_set = BTreeSet::new();
    for rule in grammar.all_rules() {
        non_terminals_set.insert(rule.lhs);
    }
    for &symbol_id in &non_terminals_set {
        // Skip if already in tokens (shouldn't happen but be safe)
        if !grammar.tokens.contains_key(&symbol_id) {
            non_terminal_symbols.push(symbol_id);
            max_symbol_id = max_symbol_id.max(symbol_id.0);
        }
    }
    non_terminal_symbols.sort_by_key(|s| s.0);

    // Collect external IDs
    for external in &grammar.externals {
        external_symbols.push(external.symbol_id);
        max_symbol_id = max_symbol_id.max(external.symbol_id.0);
    }
    external_symbols.sort_by_key(|s| s.0);

    // Now assign indices: tokens first, then non-terminals, then externals
    for symbol_id in token_symbols {
        if !symbol_to_index.contains_key(&symbol_id) {
            let idx = symbol_to_index.len();
            symbol_to_index.insert(symbol_id, idx);
        }
    }

    for symbol_id in non_terminal_symbols {
        if !symbol_to_index.contains_key(&symbol_id) {
            symbol_to_index.insert(symbol_id, symbol_to_index.len());
        }
    }

    for symbol_id in external_symbols {
        if !symbol_to_index.contains_key(&symbol_id) {
            symbol_to_index.insert(symbol_id, symbol_to_index.len());
        }
    }

    // Calculate the final symbol count after adding all symbols including EOF
    let indexed_symbol_count = symbol_to_index.len();

    // Create parse table with proper dimensions
    let state_count = collection.sets.len();
    let symbol_count = indexed_symbol_count; // Keep for compatibility

    let mut action_table = vec![vec![Vec::new(); indexed_symbol_count]; state_count];
    let mut goto_table = vec![vec![StateId(0); indexed_symbol_count]; state_count];

    // Track conflicts as we build the table
    let mut conflicts_by_state: BTreeMap<(usize, usize), Vec<Action>> = BTreeMap::new();

    // Debug: Print goto table entries
    eprintln!(
        "DEBUG: Collection goto table has {} entries",
        collection.goto_table.len()
    );
    eprintln!(
        "DEBUG: Augmented grammar has {} tokens",
        augmented_grammar.tokens.len()
    );

    // First, add shift actions from goto table for terminals
    // This must be done BEFORE reduce actions to enable shift/reduce conflict detection
    let mut _terminal_count = 0;
    let mut _non_terminal_count = 0;

    for ((from_state, symbol), to_state) in &collection.goto_table {
        // Check if this symbol is a terminal (token or external)
        let is_terminal = augmented_grammar.tokens.contains_key(symbol)
            || augmented_grammar
                .externals
                .iter()
                .any(|e| e.symbol_id == *symbol)
            || symbol.0 == 0; // EOF is also a terminal

        if from_state.0 == 0 {
            eprintln!(
                "State 0 goto entry: symbol {} -> state {}, is_terminal={}",
                symbol.0, to_state.0, is_terminal
            );
        }

        if is_terminal {
            _terminal_count += 1;
            if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
                let state_idx = from_state.0 as usize;
                if state_idx < action_table.len() && symbol_idx < action_table[state_idx].len() {
                    // Add as a shift action
                    let new_action = Action::Shift(*to_state);
                    if state_idx == 0 {
                        eprintln!(
                            "DEBUG: Adding shift action to state 0: symbol {} (idx={}) -> state {}",
                            symbol.0, symbol_idx, to_state.0
                        );
                    }
                    add_action_with_conflict(
                        &mut action_table,
                        &mut conflicts_by_state,
                        state_idx,
                        symbol_idx,
                        new_action,
                    );
                } else if state_idx == 0 {
                    eprintln!(
                        "DEBUG: SKIPPING shift for state 0: bounds check failed - state_idx={}, symbol_idx={}, action_table.len={}, inner_len={}",
                        state_idx,
                        symbol_idx,
                        action_table.len(),
                        if state_idx < action_table.len() {
                            action_table[state_idx].len()
                        } else {
                            0
                        }
                    );
                }
            } else if from_state.0 == 0 {
                eprintln!(
                    "DEBUG: Terminal {} not in symbol_to_index for state 0",
                    symbol.0
                );
            }
        } else {
            _non_terminal_count += 1;
        }
    }

    // Handle "extras" (like comments, whitespace, and external tokens marked as extras).
    // In every state, for every "extra" token, if there isn't already a specific
    // action, add a self-looping SHIFT action. This allows extras to appear
    // anywhere in the grammar without changing the parser's state.
    for state_idx in 0..state_count {
        for extra_symbol_id in &augmented_grammar.extras {
            if let Some(&symbol_idx) = symbol_to_index.get(extra_symbol_id) {
                // Check if an action already exists for this extra token in this state.
                // Only add self-loop if no action exists yet (empty cell means no action)
                if action_table[state_idx][symbol_idx].is_empty() {
                    // Add a self-looping shift that stays in the same state
                    action_table[state_idx][symbol_idx]
                        .push(Action::Shift(StateId(state_idx as u16)));
                }
            }
        }
    }

    // Now fill action table with reduce actions
    for item_set in &collection.sets {
        let state_idx = item_set.id.0 as usize;

        for item in &item_set.items {
            if item.is_reduce_item(&augmented_grammar) {
                // Check if this is a reduce by the augmented start rule
                if let Some(rule) = augmented_grammar
                    .all_rules()
                    .find(|r| r.production_id.0 == item.rule_id.0)
                {
                    if rule.lhs == augmented_start && item.lookahead == SymbolId(0) {
                        // This is S' -> S • with lookahead $, add accept action
                        if let Some(&eof_idx) = symbol_to_index.get(&SymbolId(0)) {
                            add_action_with_conflict(
                                &mut action_table,
                                &mut conflicts_by_state,
                                state_idx,
                                eof_idx,
                                Action::Accept,
                            );
                        }
                    } else {
                        // Regular reduce action - but check precedence first

                        // Check if this is an empty production
                        let rule = augmented_grammar
                            .all_rules()
                            .find(|r| r.production_id.0 == item.rule_id.0)
                            .expect("Rule not found");
                        let is_empty_production = rule.rhs.is_empty();

                        // For empty productions, we need to add reduce actions for all symbols in FOLLOW set
                        let lookaheads_to_check: Vec<SymbolId> = if is_empty_production {
                            // Get FOLLOW set for the LHS of this rule
                            if let Some(follow_set) = first_follow.follow(rule.lhs) {
                                let symbols: Vec<_> =
                                    follow_set.ones().map(|idx| SymbolId(idx as u16)).collect();
                                for sym in &symbols {
                                    if symbol_to_index.contains_key(sym) {}
                                }
                                symbols
                            } else {
                                vec![item.lookahead]
                            }
                        } else {
                            vec![item.lookahead]
                        };

                        for lookahead in lookaheads_to_check {
                            if let Some(&lookahead_idx) = symbol_to_index.get(&lookahead) {
                                let new_action = Action::Reduce(item.rule_id);

                                // Always add reduce actions - let conflict resolution handle precedence
                                add_action_with_conflict(
                                    &mut action_table,
                                    &mut conflicts_by_state,
                                    state_idx,
                                    lookahead_idx,
                                    new_action,
                                );

                                // Debug: Log reduce actions being added
                                // "DEBUG: State {} - Adding reduce action for lookahead {} (symbol {}) -> reduce by rule {}"
                            }
                        }
                    }
                }
            }
            // Note: Shift actions were already added before this loop
        }
    }

    // Shift actions were already added before reduce actions

    // Resolve conflicts using precedence
    let precedence_resolver = StaticPrecedenceResolver::from_grammar(&augmented_grammar);

    for ((state_idx, symbol_idx), actions) in conflicts_by_state {
        if actions.len() > 1 {
            // Try to resolve shift/reduce conflicts using precedence
            let mut shift_action = None;
            let mut reduce_actions = Vec::new();

            for action in &actions {
                match action {
                    Action::Shift(_) => {
                        shift_action = Some(action.clone());
                    }
                    Action::Reduce(_rule_id) => {
                        reduce_actions.push(action.clone());
                    }
                    _ => {}
                }
            }

            // Handle shift/reduce conflicts with precedence
            if let (Some(_shift), Some(reduce)) = (shift_action.as_ref(), reduce_actions.first()) {
                // Get the symbol that triggers the shift
                let symbol_id = symbol_to_index
                    .iter()
                    .find(|&(_, &idx)| idx == symbol_idx)
                    .map(|(sym, _)| *sym)
                    .unwrap_or(SymbolId(0));

                let shift_prec = precedence_resolver.token_precedence(symbol_id);

                let reduce_prec = if let Action::Reduce(rule_id) = reduce {
                    precedence_resolver.rule_precedence(*rule_id)
                } else {
                    None
                };

                // "DEBUG: Conflict resolution - symbol {} (id={}) shift_prec={:?}, reduce rule {} prec={:?}"
                match compare_precedences(shift_prec, reduce_prec) {
                    PrecedenceComparison::PreferShift => {
                        // For GLR, we still keep both actions but can mark preference
                        eprintln!(
                            "State {}: Precedence prefers shift over reduce for symbol {}",
                            state_idx, symbol_idx
                        );
                    }
                    PrecedenceComparison::PreferReduce => {
                        // For GLR, we still keep both actions but can mark preference
                        eprintln!(
                            "State {}: Precedence prefers reduce over shift for symbol {}",
                            state_idx, symbol_idx
                        );
                    }
                    PrecedenceComparison::Error => {
                        eprintln!(
                            "State {}: Non-associative conflict for symbol {}",
                            state_idx, symbol_idx
                        );
                    }
                    PrecedenceComparison::None => {
                        eprintln!(
                            "State {}: No precedence info for conflict at symbol {}",
                            state_idx, symbol_idx
                        );
                    }
                }
            }

            // For GLR, we keep all conflicting actions in the cell
            // The runtime parser will handle forking
            // The actions are already in the cell from add_action_with_conflict
        }
        // No need to do anything - actions are already in the cell
    }

    // Add non-terminal goto entries to the goto table
    for ((from_state, symbol), _to_state) in &collection.goto_table {
        // Check if this symbol is a non-terminal
        let is_terminal = augmented_grammar.tokens.contains_key(symbol)
            || augmented_grammar
                .externals
                .iter()
                .any(|e| e.symbol_id == *symbol)
            || symbol.0 == 0; // EOF is also a terminal

        if !is_terminal {
            if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
                let state_idx = from_state.0 as usize;
                if state_idx < goto_table.len() && symbol_idx < goto_table[state_idx].len() {
                    // "DEBUG: Setting goto for state {} non-terminal {} (id={}) -> state {}"
                }
            }
        }
    }

    // Fill goto table from collection's goto_table (kept for compatibility)
    for ((from_state, symbol), to_state) in &collection.goto_table {
        let from_idx = from_state.0 as usize;
        if let Some(&symbol_idx) = symbol_to_index.get(symbol) {
            goto_table[from_idx][symbol_idx] = *to_state;
        }
    }

    // Post-process is no longer needed with proper augmentation
    // The accept action is added when we see S' -> S • with EOF lookahead

    // But we still need to handle the original grammar's symbol mapping
    if let Some(_start_symbol) = grammar.start_symbol() {
        // Find all states and check if they need EOF reduce actions
        for (state_idx, _item_set) in collection.sets.iter().enumerate() {
            // Skip this post-processing - handled by augmentation
            let needs_eof_reduce = false;
            let reduce_rule_id: Option<RuleId> = None;

            // If we found a reduce item that needs EOF action, ensure it's in the action table
            if needs_eof_reduce {
                if let Some(rule_id) = reduce_rule_id {
                    if let Some(&eof_idx) = symbol_to_index.get(&SymbolId(0)) {
                        // Check if EOF action already exists
                        if action_table[state_idx][eof_idx].is_empty() {
                            action_table[state_idx][eof_idx].push(Action::Reduce(rule_id));
                        }
                    }
                }
            }
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
    for symbol_id in grammar.rules.keys() {
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

    // Compute external scanner states
    // For each state, determine which external tokens are valid
    // Now we only track validity - transitions are in the main action table
    let mut external_scanner_states =
        vec![vec![false; augmented_grammar.externals.len()]; state_count];

    // Create a mapping from external symbol_id to index
    let mut external_symbol_to_idx = BTreeMap::new();
    for (idx, external) in augmented_grammar.externals.iter().enumerate() {
        external_symbol_to_idx.insert(external.symbol_id, idx);
    }

    // Determine which external tokens are valid in each state
    // An external token is valid if there's a shift action for it in that state
    for state_idx in 0..state_count {
        for (external_idx, external) in augmented_grammar.externals.iter().enumerate() {
            // Check if this external has a shift action in this state
            if let Some(&symbol_idx) = symbol_to_index.get(&external.symbol_id) {
                // Check if any action in the cell is a shift
                if action_table[state_idx][symbol_idx]
                    .iter()
                    .any(|a| matches!(a, Action::Shift(_)))
                {
                    external_scanner_states[state_idx][external_idx] = true;
                }
            }
        }
    }

    Ok(ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count,
        symbol_count,
        symbol_to_index,
        external_scanner_states,
    })
}

/// Add an action to the parse table, tracking conflicts
fn add_action_with_conflict(
    action_table: &mut Vec<Vec<ActionCell>>,
    conflicts_by_state: &mut BTreeMap<(usize, usize), Vec<Action>>,
    state_idx: usize,
    symbol_idx: usize,
    new_action: Action,
) {
    // Bounds check
    if state_idx >= action_table.len() || symbol_idx >= action_table[0].len() {
        panic!(
            "Index out of bounds in add_action_with_conflict: state_idx={}, symbol_idx={}, table_size={}x{}",
            state_idx,
            symbol_idx,
            action_table.len(),
            if action_table.is_empty() {
                0
            } else {
                action_table[0].len()
            }
        );
    }

    let current_cell = &mut action_table[state_idx][symbol_idx];

    // Check if this action already exists
    if !current_cell.iter().any(|a| action_eq(a, &new_action)) {
        // Add the action to the cell
        current_cell.push(new_action.clone());

        // If there are now multiple actions, track as a conflict
        if current_cell.len() > 1 {
            let entry = conflicts_by_state
                .entry((state_idx, symbol_idx))
                .or_default();
            *entry = current_cell.clone();
        }
    }
}

/// Build LR(1) automaton using the GlrResult type alias
/// 
/// This is a convenience wrapper that uses the crate-level Result type.
/// Use this when migrating code to the new error handling pattern.
pub fn build_lr1_automaton_res(
    grammar: &Grammar,
    first_follow: &FirstFollowSets,
) -> GlrResult<ParseTable> {
    build_lr1_automaton(grammar, first_follow)
}

/// Check if two actions are equivalent
fn action_eq(a: &Action, b: &Action) -> bool {
    match (a, b) {
        (Action::Shift(s1), Action::Shift(s2)) => s1 == s2,
        (Action::Reduce(r1), Action::Reduce(r2)) => r1 == r2,
        (Action::Accept, Action::Accept) => true,
        (Action::Error, Action::Error) => true,
        (Action::Fork(a1), Action::Fork(a2)) => {
            a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| action_eq(x, y))
        }
        _ => false,
    }
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
        let mut set = std::collections::BTreeSet::new();
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
            lhs: SymbolId(0),                         // S
            rhs: vec![Symbol::Terminal(SymbolId(1))], // a
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.entry(SymbolId(0)).or_default().push(rule);

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
            rhs: vec![],      // empty
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.entry(SymbolId(0)).or_default().push(rule);

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
            Action::Shift(StateId(1)) => {}
            _ => panic!("Expected shift action"),
        }

        match reduce {
            Action::Reduce(RuleId(2)) => {}
            _ => panic!("Expected reduce action"),
        }

        match accept {
            Action::Accept => {}
            _ => panic!("Expected accept action"),
        }

        match error {
            Action::Error => {}
            _ => panic!("Expected error action"),
        }

        match fork {
            Action::Fork(actions) => {
                assert_eq!(actions.len(), 2);
                assert_eq!(actions[0], shift);
                assert_eq!(actions[1], reduce);
            }
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
        let resolver = ConflictResolver { conflicts: vec![] };

        assert!(resolver.conflicts.is_empty());
    }

    #[test]
    fn test_parse_table_creation() {
        let parse_table = ParseTable {
            action_table: vec![vec![vec![Action::Error]; 5]; 3], // 3 states, 5 symbols
            goto_table: vec![vec![StateId(0); 5]; 3],
            symbol_metadata: vec![],
            state_count: 3,
            symbol_count: 5,
            symbol_to_index: BTreeMap::new(),
            external_scanner_states: vec![],
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
        grammar.rules.entry(SymbolId(0)).or_default().push(rule);

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
        grammar.rules.entry(SymbolId(0)).or_default().push(rule);

        // Item at position 0: S -> • a b
        let item1 = LRItem::new(RuleId(0), 0, SymbolId(0));
        if let Some(symbol) = item1.next_symbol(&grammar) {
            match symbol {
                Symbol::Terminal(SymbolId(1)) => {}
                _ => panic!("Expected terminal symbol with id 1"),
            }
        } else {
            panic!("Expected next symbol");
        }

        // Item at position 1: S -> a • b
        let item2 = LRItem::new(RuleId(0), 1, SymbolId(0));
        if let Some(symbol) = item2.next_symbol(&grammar) {
            match symbol {
                Symbol::Terminal(SymbolId(2)) => {}
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
            GLRError::GrammarError(_) => {}
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
