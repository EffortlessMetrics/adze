// Grammar optimization passes for the pure-Rust Tree-sitter implementation
// This module implements various optimizations to improve parser performance

use crate::{Grammar, Rule, Symbol, SymbolId, ProductionId, TokenPattern, Token};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

/// Grammar optimizer that applies various optimization passes
pub struct GrammarOptimizer {
    /// Track which symbols are actually used
    used_symbols: HashSet<SymbolId>,
    /// Track which rules can be inlined
    inlinable_rules: HashSet<SymbolId>,
    /// Track left-recursive rules for special handling
    left_recursive_rules: HashSet<SymbolId>,
}

impl GrammarOptimizer {
    pub fn new() -> Self {
        GrammarOptimizer {
            used_symbols: HashSet::new(),
            inlinable_rules: HashSet::new(),
            left_recursive_rules: HashSet::new(),
        }
    }

    /// Optimize a grammar by applying all optimization passes
    pub fn optimize(&mut self, grammar: &mut Grammar) -> OptimizationStats {
        let mut stats = OptimizationStats::default();

        // Phase 1: Analysis
        self.analyze_grammar(grammar);

        // Phase 2: Optimizations
        stats.removed_unused_symbols = self.remove_unused_symbols(grammar);
        stats.inlined_rules = self.inline_simple_rules(grammar);
        stats.merged_tokens = self.merge_equivalent_tokens(grammar);
        stats.optimized_left_recursion = self.optimize_left_recursion(grammar);
        stats.eliminated_unit_rules = self.eliminate_unit_rules(grammar);

        // Phase 3: Cleanup
        self.renumber_symbols(grammar);

        stats
    }

    /// Analyze the grammar to collect information for optimization
    fn analyze_grammar(&mut self, grammar: &Grammar) {
        // Mark start symbol as used
        if let Some(start_symbol) = grammar.start_symbol() {
            self.used_symbols.insert(start_symbol);
        }
        
        // Also mark all rule LHS as used (they define the symbols)
        for symbol_id in grammar.rules.keys() {
            self.used_symbols.insert(*symbol_id);
        }

        // Analyze all rules
        for rules in grammar.rules.values() {
            for rule in rules {
                // Mark symbols used in productions
                for symbol in &rule.rhs {
                match symbol {
                    Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => {
                        self.used_symbols.insert(*id);
                    }
                }
            }

            // Check if rule is inlinable (simple, non-recursive)
            if rule.rhs.len() == 1 && !self.is_recursive_rule(rule, grammar) {
                self.inlinable_rules.insert(rule.lhs);
            }

            // Check for left recursion
            if self.is_left_recursive(rule) {
                self.left_recursive_rules.insert(rule.lhs);
            }
            }
        }

        // Note: We don't mark tokens as used here - they need to be referenced in rules
    }

    /// Remove symbols that are never referenced
    fn remove_unused_symbols(&mut self, grammar: &mut Grammar) -> usize {
        let mut removed = 0;

        // Remove unused rules
        let unused_rules: Vec<_> = grammar.rules
            .iter()
            .filter(|(id, _)| !self.used_symbols.contains(id))
            .map(|(id, _)| *id)
            .collect();

        for id in unused_rules {
            grammar.rules.shift_remove(&id);
            removed += 1;
        }

        // Remove unused tokens
        let unused_tokens: Vec<_> = grammar.tokens
            .iter()
            .filter(|(id, _)| !self.used_symbols.contains(id))
            .map(|(id, _)| *id)
            .collect();

        for id in unused_tokens {
            grammar.tokens.shift_remove(&id);
            removed += 1;
        }

        removed
    }

    /// Inline simple rules that just reference another symbol
    fn inline_simple_rules(&mut self, grammar: &mut Grammar) -> usize {
        let mut inlined = 0;
        let mut replacements = HashMap::new();

        // Find rules to inline
        for (symbol_id, rules) in &grammar.rules {
            if self.inlinable_rules.contains(symbol_id) {
                // Only inline if all rules for this symbol have exactly one RHS symbol
                if rules.len() == 1 && rules[0].rhs.len() == 1 {
                    if let Some(target) = rules[0].rhs.first() {
                        replacements.insert(*symbol_id, target.clone());
                    }
                }
            }
        }

        // Apply replacements
        for rules in grammar.rules.values_mut() {
            for rule in rules.iter_mut() {
                let mut modified = false;
                for symbol in &mut rule.rhs {
                    if let Symbol::NonTerminal(id) = symbol {
                        if let Some(replacement) = replacements.get(id) {
                            *symbol = replacement.clone();
                            modified = true;
                        }
                    }
                }
                if modified {
                    inlined += 1;
                }
            }
        }

        // Remove inlined rules
        for (id, _) in &replacements {
            grammar.rules.shift_remove(id);
            grammar.inline_rules.push(*id);
        }

        inlined
    }

    /// Merge tokens with identical patterns
    fn merge_equivalent_tokens(&mut self, grammar: &mut Grammar) -> usize {
        let mut merged = 0;
        let mut pattern_to_id: HashMap<String, SymbolId> = HashMap::new();
        let mut replacements: HashMap<SymbolId, SymbolId> = HashMap::new();

        // Find equivalent tokens
        for (id, token) in &grammar.tokens {
            let pattern_str = match &token.pattern {
                TokenPattern::String(s) => s.clone(),
                TokenPattern::Regex(r) => r.clone(),
            };

            if let Some(&existing_id) = pattern_to_id.get(&pattern_str) {
                // Found duplicate
                replacements.insert(*id, existing_id);
                merged += 1;
            } else {
                pattern_to_id.insert(pattern_str, *id);
            }
        }

        // Apply replacements in rules
        for rules in grammar.rules.values_mut() {
            for rule in rules.iter_mut() {
                for symbol in &mut rule.rhs {
                    if let Symbol::Terminal(id) = symbol {
                        if let Some(&new_id) = replacements.get(id) {
                            *symbol = Symbol::Terminal(new_id);
                        }
                    }
                }
            }
        }

        // Remove duplicate tokens
        for (old_id, _) in &replacements {
            grammar.tokens.shift_remove(old_id);
        }

        merged
    }

    /// Optimize left-recursive rules by transforming them
    fn optimize_left_recursion(&mut self, grammar: &mut Grammar) -> usize {
        let mut optimized = 0;

        // For each left-recursive rule, transform it
        // A -> A α | β becomes:
        // A -> β A'
        // A' -> α A' | ε
        let left_recursive: Vec<_> = self.left_recursive_rules.iter().cloned().collect();

        for symbol in left_recursive {
            if let Some(rules) = self.extract_rules_for_symbol(grammar, symbol) {
                let (recursive_rules, base_rules) = self.partition_recursive_rules(&rules, symbol);

                if !recursive_rules.is_empty() && !base_rules.is_empty() {
                    // Create new symbol for the recursive part
                    let new_symbol = self.create_new_symbol(grammar);
                    
                    // Transform the rules
                    self.transform_left_recursion(
                        grammar,
                        symbol,
                        new_symbol,
                        recursive_rules,
                        base_rules,
                    );
                    
                    optimized += 1;
                }
            }
        }

        optimized
    }

    /// Eliminate unit rules (A -> B)
    fn eliminate_unit_rules(&mut self, grammar: &mut Grammar) -> usize {
        let mut eliminated = 0;
        let mut unit_rules = Vec::new();

        // Find unit rules
        for rule in grammar.all_rules() {
            if rule.rhs.len() == 1 {
                if let Symbol::NonTerminal(_) = &rule.rhs[0] {
                    unit_rules.push(rule.clone());
                }
            }
        }

        // For each unit rule A -> B, add rules A -> γ for each B -> γ
        for unit_rule in unit_rules {
            if let Symbol::NonTerminal(target) = &unit_rule.rhs[0] {
                if let Some(target_rules) = grammar.get_rules_for_symbol(*target) {
                    for target_rule in target_rules {
                    // Create new rule A -> γ
                    let new_rule = Rule {
                        lhs: unit_rule.lhs,
                        rhs: target_rule.rhs.clone(),
                        precedence: target_rule.precedence.or(unit_rule.precedence),
                        associativity: target_rule.associativity.or(unit_rule.associativity),
                        fields: target_rule.fields.clone(),
                        production_id: self.create_new_production_id(grammar),
                    };

                    grammar.add_rule(new_rule);
                    eliminated += 1;
                    }
                }
                // Remove the unit rule
                // TODO: Fix this for new Grammar structure
                // grammar.rules.retain(|_, r| r != &unit_rule);
            }
        }

        eliminated
    }

    /// Check if a rule is recursive
    fn is_recursive_rule(&self, rule: &Rule, grammar: &Grammar) -> bool {
        let mut visited = HashSet::new();
        self.contains_symbol_recursive(&rule.rhs, rule.lhs, grammar, &mut visited)
    }

    /// Check if a rule is left-recursive
    fn is_left_recursive(&self, rule: &Rule) -> bool {
        if let Some(Symbol::NonTerminal(id)) = rule.rhs.first() {
            *id == rule.lhs
        } else {
            false
        }
    }

    /// Recursively check if symbols contain a target symbol
    fn contains_symbol_recursive(
        &self,
        symbols: &[Symbol],
        target: SymbolId,
        grammar: &Grammar,
        visited: &mut HashSet<SymbolId>,
    ) -> bool {
        for symbol in symbols {
            match symbol {
                Symbol::NonTerminal(id) if *id == target => return true,
                Symbol::NonTerminal(id) if !visited.contains(id) => {
                    visited.insert(*id);
                    
                    // Check all rules for this non-terminal
                    if let Some(rules) = grammar.get_rules_for_symbol(*id) {
                        for rule in rules {
                            if self.contains_symbol_recursive(&rule.rhs, target, grammar, visited) {
                                return true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Extract all rules for a given symbol
    fn extract_rules_for_symbol(&self, grammar: &Grammar, symbol: SymbolId) -> Option<Vec<Rule>> {
        grammar.get_rules_for_symbol(symbol).cloned()
    }

    /// Partition rules into recursive and non-recursive
    fn partition_recursive_rules(
        &self,
        rules: &[Rule],
        symbol: SymbolId,
    ) -> (Vec<Rule>, Vec<Rule>) {
        let mut recursive = Vec::new();
        let mut non_recursive = Vec::new();

        for rule in rules {
            if let Some(Symbol::NonTerminal(id)) = rule.rhs.first() {
                if *id == symbol {
                    recursive.push(rule.clone());
                } else {
                    non_recursive.push(rule.clone());
                }
            } else {
                non_recursive.push(rule.clone());
            }
        }

        (recursive, non_recursive)
    }

    /// Create a new unique symbol ID
    fn create_new_symbol(&self, grammar: &Grammar) -> SymbolId {
        let max_id = grammar.rules.keys()
            .chain(grammar.tokens.keys())
            .map(|id| id.0)
            .max()
            .unwrap_or(0);

        SymbolId(max_id + 1)
    }

    /// Create a new unique production ID
    fn create_new_production_id(&self, grammar: &Grammar) -> ProductionId {
        let max_id = grammar.rules.values()
            .flat_map(|rules| rules.iter())
            .map(|r| r.production_id.0)
            .max()
            .unwrap_or(0);

        ProductionId(max_id + 1)
    }

    /// Transform left-recursive rules
    fn transform_left_recursion(
        &mut self,
        grammar: &mut Grammar,
        original_symbol: SymbolId,
        new_symbol: SymbolId,
        recursive_rules: Vec<Rule>,
        base_rules: Vec<Rule>,
    ) {
        // Remove original rules
        // TODO: Fix this for new Grammar structure
        // grammar.rules.retain(|_, r| r.lhs != original_symbol);
        grammar.rules.remove(&original_symbol);

        // Add transformed base rules: A -> β A'
        for base_rule in base_rules {
            let mut new_rhs = base_rule.rhs.clone();
            new_rhs.push(Symbol::NonTerminal(new_symbol));

            let new_rule = Rule {
                lhs: original_symbol,
                rhs: new_rhs,
                precedence: base_rule.precedence,
                associativity: base_rule.associativity,
                fields: base_rule.fields,
                production_id: self.create_new_production_id(grammar),
            };

            grammar.add_rule(new_rule);
        }

        // Add recursive rules: A' -> α A' | ε
        for recursive_rule in recursive_rules {
            // Remove the left-recursive symbol
            let mut new_rhs: Vec<_> = recursive_rule.rhs[1..].to_vec();
            new_rhs.push(Symbol::NonTerminal(new_symbol));

            let new_rule = Rule {
                lhs: new_symbol,
                rhs: new_rhs,
                precedence: recursive_rule.precedence,
                associativity: recursive_rule.associativity,
                fields: Vec::new(), // Fields need to be adjusted
                production_id: self.create_new_production_id(grammar),
            };

            grammar.add_rule(new_rule);
        }

        // Add epsilon rule: A' -> ε
        let epsilon_rule = Rule {
            lhs: new_symbol,
            rhs: Vec::new(),
            precedence: None,
            associativity: None,
            fields: Vec::new(),
            production_id: self.create_new_production_id(grammar),
        };

        grammar.add_rule(epsilon_rule);
    }

    /// Renumber symbols to be contiguous
    fn renumber_symbols(&mut self, grammar: &mut Grammar) {
        let mut old_to_new: HashMap<SymbolId, SymbolId> = HashMap::new();
        let mut next_id = 1u16; // 0 is reserved for EOF

        // Assign new IDs
        for old_id in grammar.tokens.keys().chain(grammar.rules.keys()) {
            if !old_to_new.contains_key(old_id) {
                old_to_new.insert(*old_id, SymbolId(next_id));
                next_id += 1;
            }
        }

        // Update tokens
        let mut new_tokens = IndexMap::new();
        for (old_id, token) in grammar.tokens.drain(..) {
            if let Some(&new_id) = old_to_new.get(&old_id) {
                new_tokens.insert(new_id, token);
            }
        }
        grammar.tokens = new_tokens;

        // Update rules
        let mut new_rules = IndexMap::new();
        for (old_id, mut rules) in grammar.rules.drain(..) {
            // Update each rule in the vector
            for rule in &mut rules {
                // Update LHS
                if let Some(&new_id) = old_to_new.get(&rule.lhs) {
                    rule.lhs = new_id;
                }

                // Update RHS
                for symbol in &mut rule.rhs {
                    match symbol {
                        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => {
                            if let Some(&new_id) = old_to_new.get(id) {
                                *id = new_id;
                            }
                        }
                    }
                }
            }
            
            // Insert with possibly updated key
            let new_key = if let Some(&new_id) = old_to_new.get(&old_id) {
                new_id
            } else {
                old_id
            };
            new_rules.insert(new_key, rules);
        }
        grammar.rules = new_rules;

        // Update other references
        grammar.supertypes = grammar.supertypes
            .iter()
            .filter_map(|id| old_to_new.get(id).copied())
            .collect();

        grammar.inline_rules = grammar.inline_rules
            .iter()
            .filter_map(|id| old_to_new.get(id).copied())
            .collect();

        // Update external tokens
        for external in &mut grammar.externals {
            if let Some(&new_id) = old_to_new.get(&external.symbol_id) {
                external.symbol_id = new_id;
            }
        }
    }
}

/// Statistics about optimizations performed
#[derive(Debug, Default)]
pub struct OptimizationStats {
    pub removed_unused_symbols: usize,
    pub inlined_rules: usize,
    pub merged_tokens: usize,
    pub optimized_left_recursion: usize,
    pub eliminated_unit_rules: usize,
}

/// Convenience function to optimize a grammar
pub fn optimize_grammar(mut grammar: Grammar) -> anyhow::Result<Grammar> {
    let mut optimizer = GrammarOptimizer::new();
    optimizer.optimize(&mut grammar);
    Ok(grammar)
}

impl OptimizationStats {
    /// Get total number of optimizations performed
    pub fn total(&self) -> usize {
        self.removed_unused_symbols
            + self.inlined_rules
            + self.merged_tokens
            + self.optimized_left_recursion
            + self.eliminated_unit_rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PrecedenceKind, Associativity};

    fn create_test_grammar() -> Grammar {
        let mut grammar = Grammar::new("test".to_string());

        // Add some tokens
        grammar.tokens.insert(SymbolId(1), Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        });

        grammar.tokens.insert(SymbolId(2), Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        });

        // Add an unused token
        grammar.tokens.insert(SymbolId(99), Token {
            name: "unused".to_string(),
            pattern: TokenPattern::String("unused".to_string()),
            fragile: false,
        });

        // Add rules
        let expr = SymbolId(3);
        let term = SymbolId(4);

        // expr -> expr + term (left recursive)
        grammar.add_rule(Rule {
            lhs: expr,
            rhs: vec![
                Symbol::NonTerminal(expr),
                Symbol::Terminal(SymbolId(1)),
                Symbol::NonTerminal(term),
            ],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        });

        // expr -> term
        grammar.add_rule(Rule {
            lhs: expr,
            rhs: vec![Symbol::NonTerminal(term)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        });

        // term -> number
        grammar.add_rule(Rule {
            lhs: term,
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        });

        grammar
    }

    #[test]
    fn test_remove_unused_symbols() {
        let mut grammar = create_test_grammar();
        let mut optimizer = GrammarOptimizer::new();
        
        optimizer.analyze_grammar(&grammar);
        
        println!("Used symbols: {:?}", optimizer.used_symbols);
        println!("Tokens before: {:?}", grammar.tokens.keys().collect::<Vec<_>>());
        println!("Rules: {:?}", grammar.rules.values().map(|r| (r.lhs, &r.rhs)).collect::<Vec<_>>());
        
        let removed = optimizer.remove_unused_symbols(&mut grammar);
        
        println!("Removed: {}", removed);
        println!("Tokens after: {:?}", grammar.tokens.keys().collect::<Vec<_>>());

        // We expect to remove: SymbolId(99) token, and the rule key symbols 5 and 6
        assert!(removed >= 1); // At least the unused token should be removed
        assert!(!grammar.tokens.contains_key(&SymbolId(99)));
    }

    #[test]
    fn test_eliminate_unit_rules() {
        let mut grammar = create_test_grammar();
        let mut optimizer = GrammarOptimizer::new();
        
        optimizer.analyze_grammar(&grammar);
        let eliminated = optimizer.eliminate_unit_rules(&mut grammar);

        assert!(eliminated > 0); // expr -> term is a unit rule
    }

    #[test]
    fn test_optimization_stats() {
        let mut grammar = create_test_grammar();
        let mut optimizer = GrammarOptimizer::new();
        
        let stats = optimizer.optimize(&mut grammar);
        
        assert!(stats.total() > 0);
        println!("Optimization stats: {:?}", stats);
    }
}