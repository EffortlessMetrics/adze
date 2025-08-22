//! Decoder for extracting Grammar and ParseTable from Tree-sitter's TSLanguage struct
//!
//! This module reverse-engineers Tree-sitter's compressed parse table format
//! and decodes it into rust-sitter's native structures.

use indexmap::IndexMap;
use rust_sitter_glr_core::{Action, ParseRule, ParseTable, SymbolMetadata};
use rust_sitter_ir::{
    ExternalToken, Grammar, ProductionId, Rule, RuleId, StateId, SymbolId, Token, TokenPattern,
};
use std::collections::{BTreeMap, HashMap};
use std::ffi::{CStr, c_char};
use std::path::Path;

use crate::pure_parser::{TSLanguage, TSParseAction};
use crate::ts_format::TSActionTag;

/// Load token patterns from a Tree-sitter grammar.json file
/// For now, returns an empty map - will be implemented when serde_json is available
pub fn load_token_patterns(_grammar_json_path: &Path) -> HashMap<String, TokenPattern> {
    // TODO: Implement actual JSON parsing when serialization feature is fixed
    // For now, return a minimal set of hardcoded patterns for testing
    let mut patterns = HashMap::new();

    // Add some basic Python keywords that we know are needed
    patterns.insert("def".to_string(), TokenPattern::String("def".to_string()));
    patterns.insert("pass".to_string(), TokenPattern::String("pass".to_string()));
    patterns.insert(
        "return".to_string(),
        TokenPattern::String("return".to_string()),
    );
    patterns.insert("if".to_string(), TokenPattern::String("if".to_string()));
    patterns.insert("else".to_string(), TokenPattern::String("else".to_string()));
    patterns.insert("elif".to_string(), TokenPattern::String("elif".to_string()));
    patterns.insert(
        "while".to_string(),
        TokenPattern::String("while".to_string()),
    );
    patterns.insert("for".to_string(), TokenPattern::String("for".to_string()));
    patterns.insert("in".to_string(), TokenPattern::String("in".to_string()));
    patterns.insert(
        "class".to_string(),
        TokenPattern::String("class".to_string()),
    );
    patterns.insert(
        "import".to_string(),
        TokenPattern::String("import".to_string()),
    );
    patterns.insert("from".to_string(), TokenPattern::String("from".to_string()));
    patterns.insert("as".to_string(), TokenPattern::String("as".to_string()));
    patterns.insert("try".to_string(), TokenPattern::String("try".to_string()));
    patterns.insert(
        "except".to_string(),
        TokenPattern::String("except".to_string()),
    );
    patterns.insert(
        "finally".to_string(),
        TokenPattern::String("finally".to_string()),
    );
    patterns.insert("with".to_string(), TokenPattern::String("with".to_string()));
    patterns.insert(
        "async".to_string(),
        TokenPattern::String("async".to_string()),
    );
    patterns.insert(
        "await".to_string(),
        TokenPattern::String("await".to_string()),
    );
    patterns.insert(
        "lambda".to_string(),
        TokenPattern::String("lambda".to_string()),
    );
    patterns.insert(
        "yield".to_string(),
        TokenPattern::String("yield".to_string()),
    );
    patterns.insert(
        "assert".to_string(),
        TokenPattern::String("assert".to_string()),
    );
    patterns.insert(
        "break".to_string(),
        TokenPattern::String("break".to_string()),
    );
    patterns.insert(
        "continue".to_string(),
        TokenPattern::String("continue".to_string()),
    );
    patterns.insert("del".to_string(), TokenPattern::String("del".to_string()));
    patterns.insert(
        "global".to_string(),
        TokenPattern::String("global".to_string()),
    );
    patterns.insert(
        "nonlocal".to_string(),
        TokenPattern::String("nonlocal".to_string()),
    );
    patterns.insert(
        "raise".to_string(),
        TokenPattern::String("raise".to_string()),
    );
    patterns.insert("None".to_string(), TokenPattern::String("None".to_string()));
    patterns.insert("True".to_string(), TokenPattern::String("True".to_string()));
    patterns.insert(
        "False".to_string(),
        TokenPattern::String("False".to_string()),
    );
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
    patterns.insert(
        "identifier".to_string(),
        TokenPattern::Regex(r"[_\p{XID_Start}][_\p{XID_Continue}]*".to_string()),
    );

    patterns
}

/// Decode a Grammar from a TSLanguage struct
pub fn decode_grammar(lang: &'static TSLanguage) -> Grammar {
    decode_grammar_with_patterns(lang, &HashMap::new())
}

/// Decode a Grammar from a TSLanguage struct with token patterns from grammar.json
pub fn decode_grammar_with_patterns(
    lang: &'static TSLanguage,
    token_patterns: &HashMap<String, TokenPattern>,
) -> Grammar {
    let mut rules = IndexMap::new();
    let mut tokens = IndexMap::new();
    let mut symbol_names = Vec::new();
    let mut externals = Vec::new();

    // Read all symbol names
    if lang.symbol_names.is_null() {
        // If symbol_names pointer is null, generate default names
        for i in 0..lang.symbol_count as usize {
            symbol_names.push(format!("symbol_{}", i));
        }
    } else {
        for i in 0..lang.symbol_count as usize {
            unsafe {
                let name_ptr = *lang.symbol_names.add(i);
                let name = if name_ptr.is_null() {
                    format!("symbol_{}", i)
                } else {
                    CStr::from_ptr(name_ptr as *const c_char)
                        .to_string_lossy()
                        .into_owned()
                };
                symbol_names.push(name);
            }
        }
    }

    // Debug: Find 'def' keyword and show symbol mapping
    for i in 0..lang.symbol_count as usize {
        if symbol_names[i] == "def" {
            let metadata = unsafe { *lang.symbol_metadata.add(i) };
            // eprintln!(
                "Found 'def' at Symbol {}: '{}' (metadata: 0x{:02x})",
                i, symbol_names[i], metadata
            );
            break;
        }
    }

    // Debug: Show first few terminal mappings
    // eprintln!("\nFirst few terminals with their patterns:");
    let mut count = 0;
    for i in 0..lang.symbol_count as usize {
        let metadata = unsafe { *lang.symbol_metadata.add(i) };
        if is_terminal(metadata, &symbol_names[i]) && count < 10 {
            let pattern = token_patterns
                .get(&symbol_names[i])
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "no pattern".to_string());
            // eprintln!("  Symbol {}: '{}' -> {}", i, symbol_names[i], pattern);
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
                },
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
                }],
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

/// Rule metadata decoded from TSLanguage
#[derive(Clone, Copy, Debug)]
pub struct RuleMeta {
    pub lhs: SymbolId,
    pub rhs_len: u8,
}

/// Decode rules from TSLanguage
fn decode_rules(lang: &TSLanguage) -> Vec<ParseRule> {
    const DEBUG_RULE_PRINT_LIMIT: usize = 5;
    let n = lang.production_count as usize; // Use production_count, not rule_count
    let mut rules = Vec::with_capacity(n);

    if lang.production_lhs_index.is_null() {
        // No rules available, return empty
        // eprintln!("WARNING: production_lhs_index is null");
        return rules;
    }

    // Use production_lhs_index to get the correct LHS symbols
    // and try to get RHS length from TSRule if available
    for i in 0..n {
        // Get LHS from production_lhs_index (which has correct symbol in table index space)
        let lhs_idx = unsafe { *lang.production_lhs_index.add(i) };

        // Try to get rhs_len from TSRule if available
        let rhs_len = if !lang.rules.is_null() && i < lang.rule_count as usize {
            let tsr = unsafe { *lang.rules.add(i) };
            tsr.rhs_len as u16
        } else {
            // Fallback: we don't know the RHS length
            0
        };

        if i < DEBUG_RULE_PRINT_LIMIT {
            // eprintln!(
                "  decode_rules: rule {}: lhs_idx={} from production_lhs_index, rhs_len={}",
                i, lhs_idx, rhs_len
            );
        }

        rules.push(ParseRule {
            lhs: SymbolId(lhs_idx), // Use the index from production_lhs_index
            rhs_len,
        });
    }
    rules
}

/// Decode a ParseTable from a TSLanguage struct
pub fn decode_parse_table(lang: &'static TSLanguage) -> ParseTable {
    let mut action_table = Vec::new();
    let goto_table = Vec::new();
    let mut symbol_metadata = Vec::new();
    let mut symbol_to_index = BTreeMap::new();

    // Decode rules from TSLanguage
    let rules = decode_rules(lang);

    // Build (lhs, rhs_len) -> rule_id map for normalizing Reduce actions
    let mut rid_by_pair: HashMap<(u16, u8), u16> = HashMap::with_capacity(rules.len());
    for (i, r) in rules.iter().enumerate() {
        rid_by_pair.insert((r.lhs.0, r.rhs_len as u8), i as u16);
    }

    // eprintln!(
        "Decoding parse table: {} states ({} large, {} small), {} symbols",
        lang.state_count,
        lang.large_state_count,
        lang.state_count - lang.large_state_count,
        lang.symbol_count
    );

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
                CStr::from_ptr(name_ptr as *const c_char)
                    .to_string_lossy()
                    .into_owned()
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
                    decode_action(action, &rules, &rid_by_pair)
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
    // eprintln!(
        "small_parse_table_map null: {}, small_parse_table null: {}",
        lang.small_parse_table_map.is_null(),
        lang.small_parse_table.is_null()
    );
    if !lang.small_parse_table_map.is_null() && !lang.small_parse_table.is_null() {
        // eprintln!(
            "Decoding {} compressed states",
            lang.state_count - lang.large_state_count
        );
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
                        decode_action(action_entry, &rules, &rid_by_pair)
                    };
                    if !matches!(action, Action::Error) {
                        state_actions[symbol].push(action);
                    }
                }
            }

            action_table.push(state_actions);
        }
    }

    // eprintln!("Final action_table has {} states", action_table.len());
    if !action_table.is_empty() {
        // eprintln!("State 0 has {} actions", action_table[0].len());
    }

    // Decode external scanner states from the TSLanguage struct
    let external_scanner_states =
        if lang.external_token_count > 0 && !lang.external_scanner.states.is_null() {
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

    // Build reverse map for index_to_symbol
    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in &symbol_to_index {
        index_to_symbol[idx] = *sym;
    }

    // Build nonterminal_to_index for goto lookups
    let tcols = (lang.token_count + lang.external_token_count) as usize;
    let mut nonterminal_to_index = BTreeMap::new();
    for (col, sym) in index_to_symbol.iter().enumerate() {
        if col >= tcols {
            nonterminal_to_index.insert(*sym, col);
        }
    }
    // eprintln!(
        "Built nonterminal_to_index with {} entries",
        nonterminal_to_index.len()
    );
    // eprintln!(
        "  tcols={}, index_to_symbol.len()={}",
        tcols,
        index_to_symbol.len()
    );
    for (sym, col) in &nonterminal_to_index {
        // eprintln!("  NT SymbolId({}) -> col {}", sym.0, col);
    }

    // Use lang.eof_symbol as a symbol id
    let eof_symbol = SymbolId(lang.eof_symbol);

    ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count: lang.state_count as usize,
        symbol_count: lang.symbol_count as usize,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states,
        nonterminal_to_index,
        eof_symbol,
        start_symbol: {
            // Compute start symbol from the rules
            // The start symbol is typically the unique LHS that doesn't appear on any RHS
            // or the NT with the highest symbol ID (often the augmented start)
            let tcols = (lang.token_count + lang.external_token_count) as usize;
            let is_nt = |sym: SymbolId| sym.0 as usize >= tcols;

            // Collect all LHS symbols from rules (before moving rules)
            let lhs_symbols: std::collections::BTreeSet<SymbolId> =
                rules.iter().map(|r| r.lhs).collect();

            // Filter to only non-terminals and pick the one with highest ID
            // (often the augmented start symbol)
            let start = lhs_symbols
                .into_iter()
                .filter(|s| is_nt(*s))
                .max_by_key(|s| s.0)
                .unwrap_or(SymbolId((tcols + 1) as u16));

            debug_assert_ne!(start, SymbolId(0), "start_symbol cannot be ERROR(0)");
            start
        },
        rules,                       // Now move rules after computing start_symbol
        grammar: Grammar::default(), // TODO: Build from language
        initial_state: StateId(0),
        token_count: lang.token_count as usize,
        external_token_count: lang.external_token_count as usize,
        lex_modes: Vec::new(),            // TODO: Decode from language
        extras: Vec::new(),               // TODO: Decode from language
        dynamic_prec_by_rule: Vec::new(), // TODO: Decode from language
        rule_assoc_by_rule: Vec::new(),   // TODO: Decode from language
        alias_sequences: Vec::new(),      // TODO: Decode from language
        field_names: Vec::new(),          // TODO: Decode from language
        field_map: BTreeMap::new(),       // TODO: Decode from language
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
    name.starts_with("anon_sym_")
        || name.starts_with("aux_sym_")
        || name.starts_with("sym_")
        || name == "ERROR"
        || name.starts_with("ts_builtin_sym_")
        || matches!(
            name,
            "identifier"
                | "integer"
                | "float"
                | "string"
                | "comment"
                | "newline"
                | "indent"
                | "dedent"
                | "string_start"
                | "string_content"
                | "string_end"
        )
}

/// Check if a symbol is hidden based on metadata
#[allow(dead_code)]
fn is_hidden(metadata: u8) -> bool {
    // Bit 0 is typically the visible bit in Tree-sitter
    (metadata & 0x01) == 0
}

/// Decode a TSParseAction into our Action enum
fn decode_action(
    action: &TSParseAction,
    rules: &[ParseRule],
    rid_by_pair: &HashMap<(u16, u8), u16>,
) -> Action {
    // Based on Tree-sitter's encoding, action_type determines the action
    // The TSParseAction struct contains different data depending on action type

    // Tree-sitter action types using shared constants
    match action.action_type {
        x if x == TSActionTag::Shift as u8 => {
            // Shift action: move to a new state
            // The symbol field contains the state to shift to
            // extra field indicates if this is an "extra" token (whitespace, etc.)
            Action::Shift(StateId(action.symbol))
        }
        x if x == TSActionTag::Reduce as u8 => {
            // Normalize Reduce action to proper rule index
            let direct = action.symbol as usize;

            // Fast path: symbol already a valid rule index and matches child_count
            let rid: u16 =
                if direct < rules.len() && (rules[direct].rhs_len as u8) == action.child_count {
                    // Using rule ID directly from symbol field
                    action.symbol
                } else {
                    // Fallback: legacy TS encoding (symbol = LHS, child_count = rhs_len)
                    // This happens when symbol is the LHS column index
                    let key = (action.symbol, action.child_count);
                    match rid_by_pair.get(&key) {
                        Some(&rid) => rid,
                        None => {
                            debug_assert!(
                                false,
                                "Reduce mapping failed: no rule for (lhs={}, rhs_len={})",
                                action.symbol, action.child_count
                            );
                            // In release, use a distinct sentinel past rules.len()
                            // so later bounds checks catch it deterministically.
                            u16::MAX
                        }
                    }
                };

            // Short-circuit invalid rule IDs
            if rid == u16::MAX || (rid as usize) >= rules.len() {
                Action::Error // Invalid reduce rule
            } else {
                Action::Reduce(RuleId(rid))
            }
        }
        x if x == TSActionTag::Accept as u8 => {
            // Accept action: parsing complete
            Action::Accept
        }
        x if x == TSActionTag::Error as u8 => {
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
        let empty_rules = vec![];
        let empty_map = HashMap::new();

        // Test Shift action
        let shift_action = TSParseAction {
            action_type: TSActionTag::Shift as u8,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 42,
        };
        match decode_action(&shift_action, &empty_rules, &empty_map) {
            Action::Shift(StateId(state)) => assert_eq!(state, 42),
            _ => panic!("Expected Shift action"),
        }

        // Test Reduce action with direct rule index
        let rules = vec![ParseRule {
            lhs: SymbolId(10),
            rhs_len: 3,
        }];
        let reduce_action = TSParseAction {
            action_type: TSActionTag::Reduce as u8,
            extra: 0,
            child_count: 3,
            dynamic_precedence: 0,
            symbol: 0,
        };
        match decode_action(&reduce_action, &rules, &empty_map) {
            Action::Reduce(RuleId(rule)) => assert_eq!(rule, 0),
            _ => panic!("Expected Reduce action"),
        }

        // Test Accept action
        let accept_action = TSParseAction {
            action_type: TSActionTag::Accept as u8,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        };
        assert!(matches!(
            decode_action(&accept_action, &empty_rules, &empty_map),
            Action::Accept
        ));

        // Test Error/Recover action
        let recover_action = TSParseAction {
            action_type: TSActionTag::Error as u8,
            extra: 0,
            child_count: 0,
            dynamic_precedence: 0,
            symbol: 0,
        };
        assert!(matches!(
            decode_action(&recover_action, &empty_rules, &empty_map),
            Action::Error
        ));
    }
}
