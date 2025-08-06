// Advanced conflict resolution strategies for GLR parsing
// This module provides additional conflict resolution capabilities beyond the basic resolver

use crate::{Action, ParseTable};
use rust_sitter_ir::{Associativity, Grammar, PrecedenceKind, SymbolId};
use std::collections::HashMap;

/// Statistics about conflict resolution
#[derive(Debug, Clone, Default)]
pub struct ConflictStats {
    pub shift_reduce_conflicts: usize,
    pub reduce_reduce_conflicts: usize,
    pub precedence_resolved: usize,
    pub associativity_resolved: usize,
    pub explicit_glr: usize,
    pub default_resolved: usize,
}

/// Advanced conflict analyzer
pub struct ConflictAnalyzer {
    /// Statistics about conflicts found
    stats: ConflictStats,
}

impl Default for ConflictAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictAnalyzer {
    pub fn new() -> Self {
        Self {
            stats: ConflictStats::default(),
        }
    }

    /// Analyze conflicts in a parse table and return statistics
    pub fn analyze_table(&mut self, _table: &ParseTable) -> ConflictStats {
        self.stats = ConflictStats::default();

        // In the actual ParseTable implementation, we'd need to check for multiple
        // actions for the same state/symbol combination. For now, this is a simplified
        // version that assumes conflicts are represented differently.

        // Note: The current ParseTable structure doesn't support multiple actions
        // per state/symbol pair, which is needed for GLR parsing.

        self.stats.clone()
    }

    #[allow(dead_code)]
    fn categorize_conflicts(&mut self, actions: &[Action]) {
        let shifts = actions
            .iter()
            .filter(|a| matches!(a, Action::Shift(_)))
            .count();
        let reduces = actions
            .iter()
            .filter(|a| matches!(a, Action::Reduce(_)))
            .count();

        if shifts > 0 && reduces > 0 {
            self.stats.shift_reduce_conflicts += shifts * reduces;
        }

        if reduces > 1 {
            self.stats.reduce_reduce_conflicts += reduces * (reduces - 1) / 2;
        }
    }

    pub fn get_stats(&self) -> &ConflictStats {
        &self.stats
    }
}

/// Precedence-based conflict resolver
pub struct PrecedenceResolver {
    /// Token precedences extracted from grammar
    token_precedences: HashMap<SymbolId, (i16, Associativity)>,
    /// Rule precedences (by the symbol they produce)
    symbol_precedences: HashMap<SymbolId, (i16, Associativity)>,
}

impl PrecedenceResolver {
    pub fn new(grammar: &Grammar) -> Self {
        let mut token_precedences = HashMap::new();
        let mut symbol_precedences = HashMap::new();

        // Extract precedence from precedence declarations
        for prec in &grammar.precedences {
            for &symbol in &prec.symbols {
                token_precedences.insert(symbol, (prec.level, prec.associativity));
            }
        }

        // Extract precedence from rules
        for (symbol_id, rules) in &grammar.rules {
            for rule in rules {
                if let Some(prec_kind) = &rule.precedence {
                    if let PrecedenceKind::Static(level) = prec_kind {
                        if let Some(assoc) = rule.associativity {
                            symbol_precedences.insert(*symbol_id, (*level, assoc));
                        }
                    }
                }
            }
        }

        Self {
            token_precedences,
            symbol_precedences,
        }
    }

    /// Check if a shift/reduce conflict can be resolved by precedence
    pub fn can_resolve_shift_reduce(
        &self,
        shift_symbol: SymbolId,
        reduce_symbol: SymbolId,
    ) -> Option<PrecedenceDecision> {
        let shift_prec = self.token_precedences.get(&shift_symbol)?;
        let reduce_prec = self.symbol_precedences.get(&reduce_symbol)?;

        if shift_prec.0 > reduce_prec.0 {
            Some(PrecedenceDecision::PreferShift)
        } else if reduce_prec.0 > shift_prec.0 {
            Some(PrecedenceDecision::PreferReduce)
        } else {
            // Same precedence - check associativity
            match reduce_prec.1 {
                Associativity::Left => Some(PrecedenceDecision::PreferReduce),
                Associativity::Right => Some(PrecedenceDecision::PreferShift),
                Associativity::None => Some(PrecedenceDecision::Error),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecedenceDecision {
    PreferShift,
    PreferReduce,
    Error, // Non-associative conflict
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, ParseTable, StateId};
    use rust_sitter_ir::{
        Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
    };

    #[test]
    fn test_conflict_analyzer() {
        let table = ParseTable {
            action_table: vec![vec![Action::Shift(StateId(1))]],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 1,
            symbol_count: 1,
            symbol_to_index: std::collections::BTreeMap::new(),
        };

        let mut analyzer = ConflictAnalyzer::new();
        let stats = analyzer.analyze_table(&table);

        // Since the current ParseTable doesn't support multiple actions,
        // we expect no conflicts
        assert_eq!(stats.shift_reduce_conflicts, 0);
        assert_eq!(stats.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn test_precedence_resolver() {
        let mut grammar = Grammar::new("test".to_string());

        // Add precedence declarations
        grammar.precedences.push(Precedence {
            level: 1,
            associativity: Associativity::Left,
            symbols: vec![SymbolId(1)],
        });

        grammar.precedences.push(Precedence {
            level: 2,
            associativity: Associativity::Right,
            symbols: vec![SymbolId(2)],
        });

        // Add a rule with precedence
        grammar.rules.insert(
            SymbolId(3),
            vec![Rule {
                lhs: SymbolId(3),
                rhs: vec![Symbol::Terminal(SymbolId(1))],
                precedence: Some(PrecedenceKind::Static(1)),
                associativity: Some(Associativity::Left),
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );

        let resolver = PrecedenceResolver::new(&grammar);

        // Test shift has higher precedence
        let decision = resolver.can_resolve_shift_reduce(SymbolId(2), SymbolId(3));
        assert_eq!(decision, Some(PrecedenceDecision::PreferShift));

        // Test same precedence with left associativity
        let decision = resolver.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
        assert_eq!(decision, Some(PrecedenceDecision::PreferReduce));
    }
}
