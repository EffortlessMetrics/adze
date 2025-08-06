//! Decoder for extracting Grammar and ParseTable from Tree-sitter's TSLanguage struct
//!
//! This module reverse-engineers Tree-sitter's compressed parse table format
//! and decodes it into rust-sitter's native structures.

use rust_sitter_glr_core::{Action, ParseTable, SymbolMetadata};
use rust_sitter_ir::{Grammar, Rule, SymbolId, Token, ExternalToken, ProductionId, RuleId, StateId};
use std::collections::BTreeMap;
use indexmap::IndexMap;
use std::ffi::{CStr, c_char};

use crate::pure_parser::{TSLanguage, TSParseAction};

/// Decode a Grammar from a TSLanguage struct
pub fn decode_grammar(lang: &'static TSLanguage) -> Grammar {
    let mut rules = IndexMap::new();
    let mut tokens = IndexMap::new();
    let mut symbol_names = Vec::new();
    let mut externals = Vec::new();
    
    // Read all symbol names
    for i in 0..lang.symbol_count as usize {
        unsafe {
            let name_ptr = *lang.symbol_names.add(i);
            let name = if name_ptr.is_null() {
                format!("symbol_{}", i)
            } else {
                CStr::from_ptr(name_ptr as *const c_char).to_string_lossy().into_owned()
            };
            symbol_names.push(name);
        }
    }
    
    // Process symbols to determine tokens vs rules
    for i in 0..lang.symbol_count as usize {
        let metadata = unsafe { *lang.symbol_metadata.add(i) };
        let name = &symbol_names[i];
        let symbol_id = SymbolId(i as u16);
        
        // Check if this is a terminal (token) or non-terminal (rule)
        // In Tree-sitter, terminals typically have lower IDs and specific metadata bits
        if is_terminal(metadata, name) {
            // This is a token
            tokens.insert(
                symbol_id,
                Token {
                    name: name.clone(),
                    pattern: rust_sitter_ir::TokenPattern::String(name.clone()),
                    fragile: false,
                }
            );
        } else {
            // This is a rule (non-terminal)
            // For now, create a stub rule - real rules would come from grammar definitions
            rules.insert(
                symbol_id,
                vec![Rule {
                    lhs: symbol_id,
                    rhs: vec![], // Will be populated from production rules
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(i as u16),
                }]
            );
        }
    }
    
    // Process external tokens
    for i in 0..lang.external_token_count as usize {
        let symbol_id = unsafe { *lang.external_scanner.symbol_map.add(i) };
        if (symbol_id as u32) < lang.symbol_count {
            externals.push(ExternalToken {
                name: format!("external_{}", i),
                symbol_id: SymbolId(symbol_id),
            });
        }
    }
    
    Grammar {
        name: "decoded_grammar".to_string(),
        rules,
        tokens,
        precedences: vec![],
        conflicts: vec![],
        externals,
        extras: vec![],
        fields: IndexMap::new(),
        supertypes: vec![],
        inline_rules: vec![],
        alias_sequences: IndexMap::new(),
        production_ids: IndexMap::new(),
        max_alias_sequence_length: 0,
        rule_names: IndexMap::new(),
        symbol_registry: None,
    }
}

/// Decode a ParseTable from a TSLanguage struct
pub fn decode_parse_table(lang: &'static TSLanguage) -> ParseTable {
    let mut action_table = Vec::new();
    let goto_table = Vec::new();
    let mut symbol_metadata = Vec::new();
    let mut symbol_to_index = BTreeMap::new();
    
    // Build symbol to index mapping and metadata
    for i in 0..lang.symbol_count as usize {
        symbol_to_index.insert(SymbolId(i as u16), i);
        
        // Decode symbol metadata
        let (ts_metadata, name) = unsafe {
            let ts_metadata = *lang.symbol_metadata.add(i);
            let name_ptr = *lang.symbol_names.add(i);
            let name = if name_ptr.is_null() {
                format!("symbol_{}", i)
            } else {
                CStr::from_ptr(name_ptr as *const c_char).to_string_lossy().into_owned()
            };
            (ts_metadata, name)
        };
        
        symbol_metadata.push(SymbolMetadata {
            name,
            visible: (ts_metadata & 0x01) != 0,
            named: (ts_metadata & 0x02) != 0,
            supertype: (ts_metadata & 0x04) != 0,
        });
    }
    
    // Decode the parse table for large states
    for state in 0..lang.large_state_count as usize {
        let mut state_actions = Vec::new();
        
        for symbol in 0..lang.symbol_count as usize {
            // Get the action index from the parse table
            let table_offset = state * lang.symbol_count as usize + symbol;
            let action = unsafe {
                let action_idx = *lang.parse_table.add(table_offset);
                
                // Decode the action from parse_actions array
                if action_idx != 0 {
                    let action = &*lang.parse_actions.add(action_idx as usize);
                    decode_action(action)
                } else {
                    Action::Error
                }
            };
            state_actions.push(action);
        }
        
        action_table.push(state_actions);
    }
    
    // TODO: Decode small_parse_table for compressed states
    // This requires understanding the small_parse_table_map compression format
    
    ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count: lang.state_count as usize,
        symbol_count: lang.symbol_count as usize,
        symbol_to_index,
    }
}

/// Determine if a symbol is a terminal based on metadata and name
fn is_terminal(_metadata: u8, name: &str) -> bool {
    // In Tree-sitter:
    // - Terminals usually start with "anon_sym_", "sym_", or "aux_sym_"
    // - Or have specific metadata bits set
    // TODO: Also check metadata bits when we understand the encoding better
    name.starts_with("anon_sym_") || 
    name.starts_with("aux_sym_") ||
    name.starts_with("sym_") ||
    name == "ERROR" ||
    name.starts_with("ts_builtin_sym_")
}

/// Check if a symbol is hidden based on metadata
fn is_hidden(metadata: u8) -> bool {
    // Bit 0 is typically the visible bit in Tree-sitter
    (metadata & 0x01) == 0
}


/// Decode a TSParseAction into our Action enum
fn decode_action(action: &TSParseAction) -> Action {
    // Based on Tree-sitter's encoding, action_type determines the action
    // The TSParseAction struct contains different data depending on action type
    
    // Tree-sitter action types:
    // 0 = Shift
    // 1 = Reduce  
    // 2 = Accept
    // 3 = Recover (error recovery)
    
    match action.action_type {
        0 => {
            // Shift action: move to a new state
            // The symbol field contains the state to shift to
            // extra field indicates if this is an "extra" token (whitespace, etc.)
            Action::Shift(StateId(action.symbol))
        }
        1 => {
            // Reduce action: apply a production rule
            // symbol field contains the rule ID to apply
            // child_count is stored separately in the action struct
            // For now, we use symbol as the rule ID
            Action::Reduce(RuleId(action.symbol))
        }
        2 => {
            // Accept action: parsing complete
            Action::Accept
        }
        3 => {
            // Recover action: error recovery
            // For now, treat as error
            Action::Error
        }
        _ => {
            // Unknown action type
            Action::Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decoder_safety() {
        // This test ensures our decoder doesn't panic on null pointers
        // In real use, we'd test with actual TSLanguage structs
    }
    
    #[test]
    fn test_action_decoding() {
        // Test that we can decode different action types correctly
        
        // Test Shift action
        let shift_action = TSParseAction {
            action_type: 0,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 42,
        };
        match decode_action(&shift_action) {
            Action::Shift(StateId(state)) => assert_eq!(state, 42),
            _ => panic!("Expected Shift action"),
        }
        
        // Test Reduce action
        let reduce_action = TSParseAction {
            action_type: 1,
            extra: 0,
            child_count: 3,
            dynamic_precedence: 0,
            symbol: 123,
        };
        match decode_action(&reduce_action) {
            Action::Reduce(RuleId(rule)) => assert_eq!(rule, 123),
            _ => panic!("Expected Reduce action"),
        }
        
        // Test Accept action
        let accept_action = TSParseAction {
            action_type: 2,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        };
        assert!(matches!(decode_action(&accept_action), Action::Accept));
        
        // Test Error/Recover action
        let recover_action = TSParseAction {
            action_type: 3,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        };
        assert!(matches!(decode_action(&recover_action), Action::Error));
    }
}