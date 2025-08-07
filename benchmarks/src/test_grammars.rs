//! Test grammars for benchmarking incremental parsing
//! 
//! This module provides real, functional grammars that can be used
//! to test and benchmark the incremental parsing implementation.

use rust_sitter_ir::{Grammar, RuleId, SymbolId, Rule, Token, TokenPattern, Precedence, Associativity};
use rust_sitter_glr_core::{ParseTable, CanonicalCollection, ActionCell, Action};
use std::collections::BTreeMap;

/// Create a simple arithmetic grammar for testing
/// 
/// Grammar:
/// ```
/// Expression -> Number
/// Expression -> Expression '-' Expression  (left associative, precedence 1)
/// Expression -> Expression '*' Expression  (left associative, precedence 2)
/// ```
pub fn create_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::default();
    
    // Define symbol IDs
    let expr_id = SymbolId(1);
    let number_id = SymbolId(2);
    let minus_id = SymbolId(3);
    let times_id = SymbolId(4);
    
    // Add tokens
    grammar.tokens.insert(number_id, Token {
        symbol_id: number_id,
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        precedence: None,
    });
    
    grammar.tokens.insert(minus_id, Token {
        symbol_id: minus_id,
        pattern: TokenPattern::String("-".to_string()),
        precedence: None,
    });
    
    grammar.tokens.insert(times_id, Token {
        symbol_id: times_id,
        pattern: TokenPattern::String("*".to_string()),
        precedence: None,
    });
    
    // Add rules
    // Expression -> Number
    grammar.rules.entry(expr_id).or_default().push(Rule {
        rule_id: RuleId(0),
        lhs: expr_id,
        rhs: vec![rust_sitter_ir::Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        alias: None,
        field_map: BTreeMap::new(),
    });
    
    // Expression -> Expression '-' Expression
    grammar.rules.entry(expr_id).or_default().push(Rule {
        rule_id: RuleId(1),
        lhs: expr_id,
        rhs: vec![
            rust_sitter_ir::Symbol::NonTerminal(expr_id),
            rust_sitter_ir::Symbol::Terminal(minus_id),
            rust_sitter_ir::Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(Precedence(1)),
        associativity: Some(Associativity::Left),
        alias: None,
        field_map: BTreeMap::new(),
    });
    
    // Expression -> Expression '*' Expression
    grammar.rules.entry(expr_id).or_default().push(Rule {
        rule_id: RuleId(2),
        lhs: expr_id,
        rhs: vec![
            rust_sitter_ir::Symbol::NonTerminal(expr_id),
            rust_sitter_ir::Symbol::Terminal(times_id),
            rust_sitter_ir::Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(Precedence(2)),
        associativity: Some(Associativity::Left),
        alias: None,
        field_map: BTreeMap::new(),
    });
    
    // Set start symbol
    grammar.start_symbol = Some(expr_id);
    
    grammar
}

/// Create a parse table for the arithmetic grammar
pub fn create_arithmetic_parse_table(grammar: &Grammar) -> ParseTable {
    // Compute FIRST/FOLLOW sets
    let first_follow = rust_sitter_glr_core::FirstFollowSets::compute(grammar);
    
    // Build canonical collection of LR(1) items
    let collection = CanonicalCollection::build(grammar, &first_follow);
    
    // Build parse table from the collection
    build_glr_parse_table(grammar, &collection)
}

/// Build a GLR parse table from a canonical collection
/// This preserves conflicts for GLR parsing
fn build_glr_parse_table(grammar: &Grammar, collection: &CanonicalCollection) -> ParseTable {
    let state_count = collection.sets.len();
    let symbol_count = grammar.get_max_symbol_id() as usize + 1;
    
    // Create symbol to index mapping
    let mut symbol_to_index = BTreeMap::new();
    let mut index = 0;
    
    // Add all tokens
    for &symbol_id in grammar.tokens.keys() {
        symbol_to_index.insert(symbol_id, index);
        index += 1;
    }
    
    // Add all non-terminals
    for &symbol_id in grammar.rules.keys() {
        if !symbol_to_index.contains_key(&symbol_id) {
            symbol_to_index.insert(symbol_id, index);
            index += 1;
        }
    }
    
    // Initialize action and goto tables
    let mut action_table = vec![vec![Vec::new(); symbol_count]; state_count];
    let mut goto_table = vec![vec![rust_sitter_ir::StateId(0); symbol_count]; state_count];
    
    // Fill action table
    for (state_idx, item_set) in collection.sets.iter().enumerate() {
        let mut state_actions = vec![Vec::new(); symbol_count];
        
        for item in &item_set.items {
            if let Some(next_symbol) = item.next_symbol(grammar) {
                // Shift action
                if let rust_sitter_ir::Symbol::Terminal(symbol_id) = next_symbol {
                    if let Some(&next_state) = collection.goto_table.get(&(item_set.id, symbol_id)) {
                        let symbol_idx = *symbol_to_index.get(&symbol_id).unwrap_or(&0);
                        state_actions[symbol_idx].push(Action::Shift(next_state));
                    }
                }
            } else {
                // Reduce action (item is complete)
                for &lookahead in &item.lookahead {
                    if let Some(&symbol_idx) = symbol_to_index.get(&lookahead) {
                        state_actions[symbol_idx].push(Action::Reduce(item.rule_id));
                    }
                }
            }
        }
        
        action_table[state_idx] = state_actions;
    }
    
    // Fill goto table
    for ((from_state, symbol_id), &to_state) in &collection.goto_table {
        if grammar.rules.contains_key(&symbol_id) {
            // It's a non-terminal
            if let Some(&symbol_idx) = symbol_to_index.get(&symbol_id) {
                goto_table[from_state.0 as usize][symbol_idx] = to_state;
            }
        }
    }
    
    ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        external_scanner_states: vec![vec![false]; state_count],
    }
}

/// Extension trait to get maximum symbol ID from grammar
trait GrammarExt {
    fn get_max_symbol_id(&self) -> u16;
}

impl GrammarExt for Grammar {
    fn get_max_symbol_id(&self) -> u16 {
        let mut max_id = 0u16;
        
        // Check tokens
        for &symbol_id in self.tokens.keys() {
            max_id = max_id.max(symbol_id.0);
        }
        
        // Check rules
        for &symbol_id in self.rules.keys() {
            max_id = max_id.max(symbol_id.0);
        }
        
        // Check rule RHS
        for rules in self.rules.values() {
            for rule in rules {
                for symbol in &rule.rhs {
                    max_id = max_id.max(get_symbol_id(symbol));
                }
            }
        }
        
        max_id
    }
}

fn get_symbol_id(symbol: &rust_sitter_ir::Symbol) -> u16 {
    match symbol {
        rust_sitter_ir::Symbol::Terminal(id) |
        rust_sitter_ir::Symbol::NonTerminal(id) |
        rust_sitter_ir::Symbol::External(id) => id.0,
        _ => 0,
    }
}

/// Tokenize arithmetic expression for testing
pub fn tokenize_arithmetic(input: &str) -> Vec<rust_sitter::glr_incremental::GLRToken> {
    use rust_sitter::glr_incremental::GLRToken;
    let mut tokens = Vec::new();
    let mut position = 0;
    let bytes = input.as_bytes();
    
    while position < bytes.len() {
        let ch = bytes[position];
        
        match ch {
            b'0'..=b'9' => {
                // Parse number
                let start = position;
                while position < bytes.len() && bytes[position].is_ascii_digit() {
                    position += 1;
                }
                tokens.push(GLRToken {
                    symbol: SymbolId(2), // number_id
                    text: bytes[start..position].to_vec(),
                    start_byte: start,
                    end_byte: position,
                });
            }
            b'-' => {
                tokens.push(GLRToken {
                    symbol: SymbolId(3), // minus_id
                    text: vec![b'-'],
                    start_byte: position,
                    end_byte: position + 1,
                });
                position += 1;
            }
            b'*' => {
                tokens.push(GLRToken {
                    symbol: SymbolId(4), // times_id
                    text: vec![b'*'],
                    start_byte: position,
                    end_byte: position + 1,
                });
                position += 1;
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                // Skip whitespace
                position += 1;
            }
            _ => {
                // Skip unknown characters
                position += 1;
            }
        }
    }
    
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arithmetic_grammar_creation() {
        let grammar = create_arithmetic_grammar();
        assert!(grammar.start_symbol.is_some());
        assert_eq!(grammar.tokens.len(), 3);
        assert_eq!(grammar.rules.get(&SymbolId(1)).unwrap().len(), 3);
    }
    
    #[test]
    fn test_arithmetic_parse_table_creation() {
        let grammar = create_arithmetic_grammar();
        let table = create_arithmetic_parse_table(&grammar);
        assert!(table.state_count > 0);
        assert!(table.symbol_count > 0);
    }
    
    #[test]
    fn test_tokenization() {
        let tokens = tokenize_arithmetic("1 + 2 * 3");
        assert_eq!(tokens.len(), 5); // Should be: 1, +, 2, *, 3
        
        let tokens = tokenize_arithmetic("42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].symbol, SymbolId(2)); // number_id
    }
}