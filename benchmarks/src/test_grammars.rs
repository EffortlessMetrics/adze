/// Test grammars for benchmarking
use rust_sitter_ir::{Grammar, Rule, SymbolId, Token, TokenPattern, FieldId, RuleId, StateId, ProductionId};
use rust_sitter_glr_core::{ParseTable, ActionCell, Action, SymbolMetadata};
use indexmap::IndexMap;
use std::collections::BTreeMap;

/// Test token structure
#[derive(Debug, Clone)]
pub struct TestToken {
    pub symbol: SymbolId,
    pub text: Vec<u8>,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Load a simple arithmetic grammar for testing
pub fn load_arithmetic_grammar() -> (Grammar, ParseTable) {
    // Create a simple arithmetic grammar
    let mut grammar = Grammar {
        name: "arithmetic".to_string(),
        rules: IndexMap::new(),
        tokens: IndexMap::new(),
        precedences: vec![],
        conflicts: vec![],
        externals: vec![],
        extras: vec![],
        fields: IndexMap::new(),
        supertypes: vec![],
        inline_rules: vec![],
        alias_sequences: IndexMap::new(),
        production_ids: IndexMap::new(),
        max_alias_sequence_length: 0,
        rule_names: IndexMap::new(),
        symbol_registry: None,
    };
    
    // Add simple tokens
    let number_token = SymbolId(1);
    let plus_token = SymbolId(2);
    let minus_token = SymbolId(3);
    let mult_token = SymbolId(4);
    let div_token = SymbolId(5);
    
    grammar.tokens.insert(number_token, Token {
        symbol_id: number_token,
        pattern: TokenPattern::Regex("[0-9]+".to_string()),
        is_inline: false,
    });
    
    grammar.tokens.insert(plus_token, Token {
        symbol_id: plus_token,
        pattern: TokenPattern::String("+".to_string()),
        is_inline: false,
    });
    
    grammar.tokens.insert(minus_token, Token {
        symbol_id: minus_token,
        pattern: TokenPattern::String("-".to_string()),
        is_inline: false,
    });
    
    grammar.tokens.insert(mult_token, Token {
        symbol_id: mult_token,
        pattern: TokenPattern::String("*".to_string()),
        is_inline: false,
    });
    
    grammar.tokens.insert(div_token, Token {
        symbol_id: div_token,
        pattern: TokenPattern::String("/".to_string()),
        is_inline: false,
    });
    
    // Add simple rules
    let expr_symbol = SymbolId(0);
    grammar.rules.insert(expr_symbol, vec![
        Rule {
            rule_id: RuleId(0),
            symbol_id: expr_symbol,
            body: vec![number_token],
            precedence: None,
            associativity: None,
            is_fragile: false,
            field_map: IndexMap::new(),
            alias_sequence_id: None,
        },
        Rule {
            rule_id: RuleId(1),
            symbol_id: expr_symbol,
            body: vec![expr_symbol, plus_token, expr_symbol],
            precedence: None,
            associativity: None,
            is_fragile: false,
            field_map: IndexMap::new(),
            alias_sequence_id: None,
        },
    ]);
    
    // Create a simple parse table
    let mut action_table = vec![vec![ActionCell::new(); 6]; 10];
    let mut goto_table = vec![vec![StateId(0); 6]; 10];
    
    // Add some basic actions
    action_table[0][1] = ActionCell(vec![Action::Shift(StateId(1))]);
    action_table[1][0] = ActionCell(vec![Action::Reduce(RuleId(0))]);
    
    let mut symbol_to_index = BTreeMap::new();
    for i in 0..6 {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    
    let table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![SymbolMetadata::default(); 6],
        state_count: 10,
        symbol_count: 6,
        symbol_to_index,
        external_scanner_states: vec![vec![false; 0]; 10],
    };
    
    (grammar, table)
}

/// Tokenize arithmetic expression
pub fn tokenize_arithmetic(input: &str) -> Vec<TestToken> {
    let mut tokens = Vec::new();
    let mut position = 0;
    
    for ch in input.chars() {
        let start = position;
        let end = position + ch.len_utf8();
        
        let symbol = match ch {
            '0'..='9' => SymbolId(1),
            '+' => SymbolId(2),
            '-' => SymbolId(3),
            '*' => SymbolId(4),
            '/' => SymbolId(5),
            ' ' | '\t' | '\n' => {
                position = end;
                continue;
            }
            _ => {
                position = end;
                continue;
            }
        };
        
        tokens.push(TestToken {
            symbol,
            text: ch.to_string().into_bytes(),
            start_byte: start,
            end_byte: end,
        });
        
        position = end;
    }
    
    tokens
}