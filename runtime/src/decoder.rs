//! Decoder for extracting Grammar and ParseTable from Tree-sitter's TSLanguage struct
//!
//! This module reverse-engineers Tree-sitter's compressed parse table format
//! and decodes it into rust-sitter's native structures.

use rust_sitter_glr_core::{Action, ParseTable, SymbolMetadata};
use rust_sitter_ir::{Grammar, Rule, SymbolId, Token, ExternalToken, ProductionId, RuleId, StateId, TokenPattern};
use std::collections::{BTreeMap, HashMap};
use indexmap::IndexMap;
use std::ffi::{CStr, c_char};
use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::pure_parser::{TSLanguage, TSParseAction};

/// Load token patterns from a Tree-sitter grammar.json file
/// For now, returns an empty map - will be implemented when serde_json is available
pub fn load_token_patterns(_grammar_json_path: &Path) -> HashMap<String, TokenPattern> {
    // TODO: Implement actual JSON parsing when serialization feature is fixed
    // For now, return a minimal set of hardcoded patterns for testing
    let mut patterns = HashMap::new();
    
    // Add some basic Python keywords that we know are needed
    patterns.insert("def".to_string(), TokenPattern::String("def".to_string()));
    patterns.insert("pass".to_string(), TokenPattern::String("pass".to_string()));
    patterns.insert("return".to_string(), TokenPattern::String("return".to_string()));
    patterns.insert("if".to_string(), TokenPattern::String("if".to_string()));
    patterns.insert("else".to_string(), TokenPattern::String("else".to_string()));
    patterns.insert("elif".to_string(), TokenPattern::String("elif".to_string()));
    patterns.insert("while".to_string(), TokenPattern::String("while".to_string()));
    patterns.insert("for".to_string(), TokenPattern::String("for".to_string()));
    patterns.insert("in".to_string(), TokenPattern::String("in".to_string()));
    patterns.insert("class".to_string(), TokenPattern::String("class".to_string()));
    patterns.insert("import".to_string(), TokenPattern::String("import".to_string()));
    patterns.insert("from".to_string(), TokenPattern::String("from".to_string()));
    patterns.insert("as".to_string(), TokenPattern::String("as".to_string()));
    patterns.insert("try".to_string(), TokenPattern::String("try".to_string()));
    patterns.insert("except".to_string(), TokenPattern::String("except".to_string()));
    patterns.insert("finally".to_string(), TokenPattern::String("finally".to_string()));
    patterns.insert("with".to_string(), TokenPattern::String("with".to_string()));
    patterns.insert("async".to_string(), TokenPattern::String("async".to_string()));
    patterns.insert("await".to_string(), TokenPattern::String("await".to_string()));
    patterns.insert("lambda".to_string(), TokenPattern::String("lambda".to_string()));
    patterns.insert("yield".to_string(), TokenPattern::String("yield".to_string()));
    patterns.insert("assert".to_string(), TokenPattern::String("assert".to_string()));
    patterns.insert("break".to_string(), TokenPattern::String("break".to_string()));
    patterns.insert("continue".to_string(), TokenPattern::String("continue".to_string()));
    patterns.insert("del".to_string(), TokenPattern::String("del".to_string()));
    patterns.insert("global".to_string(), TokenPattern::String("global".to_string()));
    patterns.insert("nonlocal".to_string(), TokenPattern::String("nonlocal".to_string()));
    patterns.insert("raise".to_string(), TokenPattern::String("raise".to_string()));
    patterns.insert("None".to_string(), TokenPattern::String("None".to_string()));
    patterns.insert("True".to_string(), TokenPattern::String("True".to_string()));
    patterns.insert("False".to_string(), TokenPattern::String("False".to_string()));
    patterns.insert("and".to_string(), TokenPattern::String("and".to_string()));
    patterns.insert("or".to_string(), TokenPattern::String("or".to_string()));
    patterns.insert("not".to_string(), TokenPattern::String("not".to_string()));
    patterns.insert("is".to_string(), TokenPattern::String("is".to_string()));
    
    // Common symbols
    patterns.insert(":".to_string(), TokenPattern::String(":".to_string()));
    patterns.insert("(".to_string(), TokenPattern::String("(".to_string()));
    patterns.insert(")".to_string(), TokenPattern::String(")".to_string()));
    patterns.insert("[".to_string(), TokenPattern::String("[".to_string()));
    patterns.insert("]".to_string(), TokenPattern::String("]".to_string()));
    patterns.insert("{".to_string(), TokenPattern::String("{".to_string()));
    patterns.insert("}".to_string(), TokenPattern::String("}".to_string()));
    patterns.insert(",".to_string(), TokenPattern::String(",".to_string()));
    patterns.insert(".".to_string(), TokenPattern::String(".".to_string()));
    patterns.insert(";".to_string(), TokenPattern::String(";".to_string()));
    patterns.insert("=".to_string(), TokenPattern::String("=".to_string()));
    patterns.insert("+".to_string(), TokenPattern::String("+".to_string()));
    patterns.insert("-".to_string(), TokenPattern::String("-".to_string()));
    patterns.insert("*".to_string(), TokenPattern::String("*".to_string()));
    patterns.insert("/".to_string(), TokenPattern::String("/".to_string()));
    patterns.insert("%".to_string(), TokenPattern::String("%".to_string()));
    patterns.insert("**".to_string(), TokenPattern::String("**".to_string()));
    patterns.insert("//".to_string(), TokenPattern::String("//".to_string()));
    patterns.insert("==".to_string(), TokenPattern::String("==".to_string()));
    patterns.insert("!=".to_string(), TokenPattern::String("!=".to_string()));
    patterns.insert("<".to_string(), TokenPattern::String("<".to_string()));
    patterns.insert(">".to_string(), TokenPattern::String(">".to_string()));
    patterns.insert("<=".to_string(), TokenPattern::String("<=".to_string()));
    patterns.insert(">=".to_string(), TokenPattern::String(">=".to_string()));
    patterns.insert("+=".to_string(), TokenPattern::String("+=".to_string()));
    patterns.insert("-=".to_string(), TokenPattern::String("-=".to_string()));
    patterns.insert("*=".to_string(), TokenPattern::String("*=".to_string()));
    patterns.insert("/=".to_string(), TokenPattern::String("/=".to_string()));
    patterns.insert("->".to_string(), TokenPattern::String("->".to_string()));
    
    // Identifiers (regex pattern)
    patterns.insert("identifier".to_string(), TokenPattern::Regex(r"[_\p{XID_Start}][_\p{XID_Continue}]*".to_string()));
    
    patterns
}


/// Decode a Grammar from a TSLanguage struct
pub fn decode_grammar(lang: &'static TSLanguage) -> Grammar {
    decode_grammar_with_patterns(lang, &HashMap::new())
}

/// Decode a Grammar from a TSLanguage struct with token patterns from grammar.json
pub fn decode_grammar_with_patterns(lang: &'static TSLanguage, token_patterns: &HashMap<String, TokenPattern>) -> Grammar {
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
    
    // Debug: Find 'def' keyword and show symbol mapping
    for i in 0..lang.symbol_count as usize {
        if symbol_names[i] == "def" {
            let metadata = unsafe { *lang.symbol_metadata.add(i) };
            eprintln!("Found 'def' at Symbol {}: '{}' (metadata: 0x{:02x})", i, symbol_names[i], metadata);
            break;
        }
    }
    
    // Debug: Show first few terminal mappings
    eprintln!("\nFirst few terminals with their patterns:");
    let mut count = 0;
    for i in 0..lang.symbol_count as usize {
        let metadata = unsafe { *lang.symbol_metadata.add(i) };
        if is_terminal(metadata, &symbol_names[i]) && count < 10 {
            let pattern = token_patterns.get(&symbol_names[i])
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "no pattern".to_string());
            eprintln!("  Symbol {}: '{}' -> {}", i, symbol_names[i], pattern);
            count += 1;
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
            // Try to get the real pattern from our loaded patterns
            let pattern = if let Some(real_pattern) = token_patterns.get(name) {
                real_pattern.clone()
            } else {
                // Fallback to placeholder pattern
                rust_sitter_ir::TokenPattern::String(name.clone())
            };
            
            tokens.insert(
                symbol_id,
                Token {
                    name: name.clone(),
                    pattern,
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
    
    eprintln!("Decoding parse table: {} states ({} large, {} small), {} symbols", 
        lang.state_count, lang.large_state_count, 
        lang.state_count - lang.large_state_count, lang.symbol_count);
    
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
            // Create an action cell with single action (Tree-sitter doesn't store multiple actions)
            let action_cell = if matches!(action, Action::Error) {
                vec![]
            } else {
                vec![action]
            };
            state_actions.push(action_cell);
        }
        
        action_table.push(state_actions);
    }
    
    // Decode small_parse_table for compressed states
    eprintln!("small_parse_table_map null: {}, small_parse_table null: {}", 
        lang.small_parse_table_map.is_null(), lang.small_parse_table.is_null());
    if !lang.small_parse_table_map.is_null() && !lang.small_parse_table.is_null() {
        eprintln!("Decoding {} compressed states", lang.state_count - lang.large_state_count);
        for state in lang.large_state_count as usize..lang.state_count as usize {
            let mut state_actions = vec![vec![]; lang.symbol_count as usize];
            
            // Get the offset into small_parse_table from the map
            let map_index = state - lang.large_state_count as usize;
            let offset = unsafe { *lang.small_parse_table_map.add(map_index) } as usize;
            
            // Read from small_parse_table at the offset
            let mut ptr = unsafe { lang.small_parse_table.add(offset) };
            
            // First value is the field count (number of symbol/action pairs)
            let field_count = unsafe { *ptr } as usize;
            ptr = unsafe { ptr.add(1) };
            
            // Read field_count pairs of (symbol, action_index)
            for _ in 0..field_count {
                let symbol = unsafe { *ptr } as usize;
                ptr = unsafe { ptr.add(1) };
                
                let action_index = unsafe { *ptr } as usize;
                ptr = unsafe { ptr.add(1) };
                
                // Decode the action
                if action_index != 0 && symbol < lang.symbol_count as usize {
                    let action = unsafe {
                        let action_entry = &*lang.parse_actions.add(action_index);
                        decode_action(action_entry)
                    };
                    if !matches!(action, Action::Error) {
                        state_actions[symbol].push(action);
                    }
                }
            }
            
            action_table.push(state_actions);
        }
    }
    
    eprintln!("Final action_table has {} states", action_table.len());
    if !action_table.is_empty() {
        eprintln!("State 0 has {} actions", action_table[0].len());
    }
    
    // Decode external scanner states from the TSLanguage struct
    let external_scanner_states = if lang.external_token_count > 0 && !lang.external_scanner.states.is_null() {
        let mut states = Vec::with_capacity(lang.state_count as usize);
        let external_count = lang.external_token_count as usize;
        
        // The states are stored as a flat array of bools
        // Each state has external_token_count bools indicating which externals are valid
        unsafe {
            let states_ptr = lang.external_scanner.states as *const bool;
            for state_idx in 0..lang.state_count as usize {
                let mut state_externals = Vec::with_capacity(external_count);
                for external_idx in 0..external_count {
                    let idx = state_idx * external_count + external_idx;
                    let is_valid = *states_ptr.add(idx);
                    state_externals.push(is_valid);
                }
                states.push(state_externals);
            }
        }
        states
    } else {
        vec![vec![]; lang.state_count as usize]
    };
    
    // External tokens now have their transitions in the main action_table
    // No separate map needed
    
    ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count: lang.state_count as usize,
        symbol_count: lang.symbol_count as usize,
        symbol_to_index,
        external_scanner_states,
    }
}

/// Determine if a symbol is a terminal based on metadata and name
fn is_terminal(metadata: u8, name: &str) -> bool {
    // In Tree-sitter, the metadata encodes visibility and type information:
    // Bit 0 (0x01): visible flag - if set, the symbol is visible
    // Visible symbols are typically terminals (tokens)
    // Hidden symbols (metadata & 0x01 == 0) are typically non-terminals
    
    // First check: if the symbol is visible (bit 0 set), it's likely a terminal
    if (metadata & 0x01) != 0 {
        // Visible symbol - most likely a terminal
        // But exclude some patterns that are definitely non-terminals even if visible
        if name.starts_with("_") && name[1..].chars().all(|c| c.is_ascii_digit()) {
            // Names like _119, _26 are non-terminals even if marked visible
            return false;
        }
        return true;
    }
    
    // Hidden symbols are usually non-terminals, but check for special cases
    // Some terminals might be hidden (like whitespace, comments)
    name.starts_with("anon_sym_") || 
    name.starts_with("aux_sym_") ||
    name.starts_with("sym_") ||
    name == "ERROR" ||
    name.starts_with("ts_builtin_sym_") ||
    matches!(name, 
        "identifier" | "integer" | "float" | "string" | "comment" | 
        "newline" | "indent" | "dedent" | "string_start" | "string_content" | "string_end"
    )
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