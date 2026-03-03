#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
//! Serialization of parse tables and language structures for testing.

// Table serialization for testing and debugging
// This module allows us to serialize parse tables and language structures for comparison

use crate::abi::*;
use crate::compress::CompressedTables;
use adze_glr_core::ParseTable;
use adze_ir::Grammar;
use serde::{Deserialize, Serialize};

/// Serializable representation of a Language for testing
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableLanguage {
    pub version: u32,
    pub symbol_count: u32,
    pub alias_count: u32,
    pub token_count: u32,
    pub external_token_count: u32,
    pub state_count: u32,
    pub large_state_count: u32,
    pub production_id_count: u32,
    pub field_count: u32,
    pub symbol_names: Vec<String>,
    pub field_names: Vec<String>,
    pub symbol_metadata: Vec<u8>,
    pub parse_table: Vec<u16>,
    pub small_parse_table_map: Vec<u32>,
    pub lex_modes: Vec<SerializableLexState>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableLexState {
    pub lex_state: u16,
    pub external_lex_state: u16,
}

/// Serialize a grammar and parse table to JSON
pub fn serialize_language(
    grammar: &Grammar,
    parse_table: &ParseTable,
    compressed: Option<&CompressedTables>,
) -> Result<String, serde_json::Error> {
    let language = build_serializable_language(grammar, parse_table, compressed);
    serde_json::to_string_pretty(&language)
}

fn build_serializable_language(
    grammar: &Grammar,
    parse_table: &ParseTable,
    compressed: Option<&CompressedTables>,
) -> SerializableLanguage {
    // Generate symbol names with deterministic ordering
    let symbol_names = generate_symbol_names(grammar);
    let field_names = generate_field_names(grammar);
    let symbol_metadata = generate_symbol_metadata(grammar);
    let (parse_table_data, small_table_map) = generate_parse_table_data(compressed);
    let lex_modes = generate_lex_modes(parse_table);

    SerializableLanguage {
        version: TREE_SITTER_LANGUAGE_VERSION,
        symbol_count: calculate_symbol_count(grammar) as u32,
        alias_count: 0,
        token_count: grammar.tokens.len() as u32,
        external_token_count: grammar.externals.len() as u32,
        state_count: parse_table.state_count as u32,
        large_state_count: 0,
        production_id_count: calculate_production_count(grammar) as u32,
        field_count: grammar.fields.len() as u32,
        symbol_names,
        field_names,
        symbol_metadata,
        parse_table: parse_table_data,
        small_parse_table_map: small_table_map,
        lex_modes,
    }
}

fn generate_symbol_names(grammar: &Grammar) -> Vec<String> {
    let mut names = vec!["end".to_string()]; // EOF

    // Sort tokens by ID
    let mut tokens: Vec<_> = grammar.tokens.iter().collect();
    tokens.sort_by_key(|(id, _)| id.0);
    for (_, token) in tokens {
        names.push(token.name.clone());
    }

    // Sort non-terminals by ID
    let mut rules: Vec<_> = grammar.rules.iter().collect();
    rules.sort_by_key(|(id, _)| id.0);
    for (id, _) in rules {
        let name = grammar
            .rule_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("rule_{}", id.0));
        names.push(name);
    }

    // Add externals
    for external in &grammar.externals {
        names.push(external.name.clone());
    }

    names
}

fn generate_field_names(grammar: &Grammar) -> Vec<String> {
    // Fields must be in lexicographic order
    let mut fields: Vec<_> = grammar.fields.iter().collect();
    fields.sort_by_key(|(_, name)| name.as_str());
    fields.into_iter().map(|(_, name)| name.clone()).collect()
}

fn generate_symbol_metadata(grammar: &Grammar) -> Vec<u8> {
    let mut metadata = Vec::new();

    // EOF
    metadata.push(create_symbol_metadata(true, false, false, false, false));

    // Tokens
    let mut tokens: Vec<_> = grammar.tokens.iter().collect();
    tokens.sort_by_key(|(id, _)| id.0);
    for (_, token) in tokens {
        let visible = !token.name.starts_with('_');
        let named = visible && matches!(&token.pattern, adze_ir::TokenPattern::Regex(_));
        metadata.push(create_symbol_metadata(visible, named, false, false, false));
    }

    // Non-terminals
    let mut rules: Vec<_> = grammar.rules.iter().collect();
    rules.sort_by_key(|(id, _)| id.0);
    for (id, _) in rules {
        let name = grammar
            .rule_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("rule_{}", id.0));
        let visible = !name.starts_with('_');
        let named = visible;
        let supertype = grammar.supertypes.contains(id);
        metadata.push(create_symbol_metadata(
            visible, named, false, false, supertype,
        ));
    }

    // Externals
    for external in &grammar.externals {
        let visible = !external.name.starts_with('_');
        let named = visible;
        metadata.push(create_symbol_metadata(visible, named, false, false, false));
    }

    metadata
}

fn generate_parse_table_data(compressed: Option<&CompressedTables>) -> (Vec<u16>, Vec<u32>) {
    if let Some(compressed) = compressed {
        let mut table_data = Vec::new();
        let mut map_data = Vec::new();

        // Simplified: just collect basic data
        for entry in &compressed.action_table.data {
            table_data.push(entry.symbol);
            // Encode action based on Tree-sitter format
            match &entry.action {
                adze_glr_core::Action::Shift(state) => table_data.push(state.0),
                adze_glr_core::Action::Reduce(rule) => {
                    // Tree-sitter uses 1-based production IDs
                    table_data.push(0x8000 | (rule.0 + 1))
                }
                adze_glr_core::Action::Accept => table_data.push(0xFFFF),
                adze_glr_core::Action::Error => table_data.push(0xFFFE),
                adze_glr_core::Action::Recover => table_data.push(0xFFFD),
                adze_glr_core::Action::Fork(_) => table_data.push(0xFFFE),
                _ => table_data.push(0xFFFE), // Unknown action type // Expected: V for Recover
            }
        }

        for &offset in &compressed.action_table.row_offsets {
            map_data.push(offset as u32);
        }

        (table_data, map_data)
    } else {
        (vec![], vec![])
    }
}

fn generate_lex_modes(parse_table: &ParseTable) -> Vec<SerializableLexState> {
    (0..parse_table.state_count)
        .map(|i| SerializableLexState {
            lex_state: i as u16,
            external_lex_state: 0,
        })
        .collect()
}

fn calculate_symbol_count(grammar: &Grammar) -> usize {
    1 + // EOF
    grammar.tokens.len() +
    grammar.rules.len() +
    grammar.externals.len()
}

fn calculate_production_count(grammar: &Grammar) -> usize {
    grammar
        .rules
        .values()
        .flat_map(|rules| rules.iter())
        .count()
}

/// Serialize compressed tables for comparison
pub fn serialize_compressed_tables(tables: &CompressedTables) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct SerializableTables {
        action_table: SerializableActionTable,
        goto_table: SerializableGotoTable,
        small_table_threshold: usize,
    }

    #[derive(Serialize)]
    struct SerializableActionTable {
        entries: Vec<(u16, String)>, // (symbol, action description)
        row_offsets: Vec<u16>,
        default_actions: Vec<String>,
    }

    #[derive(Serialize)]
    struct SerializableGotoTable {
        entries: Vec<String>, // String representation of entries
        row_offsets: Vec<u16>,
    }

    let action_entries: Vec<_> = tables
        .action_table
        .data
        .iter()
        .map(|entry| {
            let action_str = match &entry.action {
                adze_glr_core::Action::Shift(s) => format!("Shift({})", s.0),
                adze_glr_core::Action::Reduce(r) => format!("Reduce({})", r.0),
                adze_glr_core::Action::Accept => "Accept".to_string(),
                adze_glr_core::Action::Error => "Error".to_string(),
                adze_glr_core::Action::Recover => "Recover".to_string(),
                adze_glr_core::Action::Fork(actions) => format!("Fork({})", actions.len()),
                _ => "Unknown".to_string(),
            };
            (entry.symbol, action_str)
        })
        .collect();

    let default_actions: Vec<_> = tables
        .action_table
        .default_actions
        .iter()
        .map(|action| match action {
            adze_glr_core::Action::Shift(s) => format!("Shift({})", s.0),
            adze_glr_core::Action::Reduce(r) => format!("Reduce({})", r.0),
            adze_glr_core::Action::Accept => "Accept".to_string(),
            adze_glr_core::Action::Error => "Error".to_string(),
            adze_glr_core::Action::Recover => "Recover".to_string(),
            adze_glr_core::Action::Fork(actions) => format!("Fork({})", actions.len()),
            _ => "Unknown".to_string(),
        })
        .collect();

    let goto_entries: Vec<_> = tables
        .goto_table
        .data
        .iter()
        .map(|entry| match entry {
            crate::compress::CompressedGotoEntry::Single(s) => format!("Single({})", s),
            crate::compress::CompressedGotoEntry::RunLength { state, count } => {
                format!("RunLength({}, {})", state, count)
            }
        })
        .collect();

    let serializable = SerializableTables {
        action_table: SerializableActionTable {
            entries: action_entries,
            row_offsets: tables.action_table.row_offsets.clone(),
            default_actions,
        },
        goto_table: SerializableGotoTable {
            entries: goto_entries,
            row_offsets: tables.goto_table.row_offsets.clone(),
        },
        small_table_threshold: tables.small_table_threshold,
    };

    serde_json::to_string_pretty(&serializable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::*;

    #[test]
    fn test_deterministic_serialization() {
        let mut grammar = Grammar::new("test".to_string());

        // Add tokens in random order
        grammar.tokens.insert(
            SymbolId(3),
            Token {
                name: "c".to_string(),
                pattern: TokenPattern::String("c".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "a".to_string(),
                pattern: TokenPattern::String("a".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(2),
            Token {
                name: "b".to_string(),
                pattern: TokenPattern::String("b".to_string()),
                fragile: false,
            },
        );

        let parse_table = crate::empty_table!(states: 1, terms: 3, nonterms: 0);

        let language = build_serializable_language(&grammar, &parse_table, None);

        // Check that symbols are sorted by ID
        assert_eq!(language.symbol_names[0], "end");
        assert_eq!(language.symbol_names[1], "a");
        assert_eq!(language.symbol_names[2], "b");
        assert_eq!(language.symbol_names[3], "c");
    }

    #[test]
    fn test_field_ordering() {
        let mut grammar = Grammar::new("test".to_string());

        // Add fields in random order
        grammar.fields.insert(FieldId(0), "zebra".to_string());
        grammar.fields.insert(FieldId(1), "apple".to_string());
        grammar.fields.insert(FieldId(2), "mango".to_string());

        let parse_table = crate::empty_table!(states: 1, terms: 0, nonterms: 0);

        let language = build_serializable_language(&grammar, &parse_table, None);

        // Check that fields are sorted lexicographically
        assert_eq!(language.field_names[0], "apple");
        assert_eq!(language.field_names[1], "mango");
        assert_eq!(language.field_names[2], "zebra");
    }
}
