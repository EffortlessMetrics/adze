use rust_sitter_runtime::{language::SymbolMetadata, Language, Parser, Token};

#[cfg(feature = "glr-core")]
fn empty_parse_table() -> &'static rust_sitter_glr_core::ParseTable {
    use rust_sitter_glr_core::{GotoIndexing, ParseTable};
    use rust_sitter_ir::{Grammar, StateId, SymbolId};
    use std::collections::BTreeMap;

    Box::leak(Box::new(ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        grammar: Grammar::new("stub".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }))
}

#[cfg(not(feature = "glr-core"))]
fn empty_parse_table() -> rust_sitter_runtime::language::ParseTable {
    rust_sitter_runtime::language::ParseTable {
        state_count: 0,
        action_table: vec![],
        small_parse_table: None,
        small_parse_table_map: None,
    }
}

#[cfg(feature = "glr-core")]
fn stub_language(tokens: Vec<Token>) -> Language {
    Language::builder()
        .parse_table(empty_parse_table())
        .symbol_names(vec!["placeholder".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(vec![])
        .tokenizer(move |_| Box::new(tokens.clone().into_iter()))
        .build()
        .unwrap()
}

#[cfg(not(feature = "glr-core"))]
fn stub_language(_tokens: Vec<Token>) -> Language {
    Language::builder()
        .parse_table(empty_parse_table())
        .symbol_names(vec!["placeholder".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(vec![])
        .build()
        .unwrap()
}

fn main() {
    // Create a stub language with test tokens to validate the tokenizer hookup
    // This will error on actual parsing since we don't have real parse tables yet
    let lang = stub_language(vec![
        Token {
            kind: 1,
            start: 0,
            end: 1,
        }, // First token 'a'
        Token {
            kind: 2,
            start: 1,
            end: 2,
        }, // Second token 'b'
        Token {
            kind: 0,
            start: 2,
            end: 2,
        }, // EOF token (kind 0)
    ]);

    let mut p = Parser::new();

    // Try to set language - this will fail in GLR mode if no parse table
    match p.set_language(lang) {
        Ok(_) => println!("✓ Language set successfully"),
        Err(e) => {
            println!(
                "✗ Failed to set language (expected in GLR mode without tables): {}",
                e
            );
            println!("  This is expected until parse tables are generated.");
            return;
        }
    }

    println!("Attempting to parse 'ab' with manual tokens...");
    match p.parse("ab", None) {
        Ok(tree) => {
            println!("✓ Parsed successfully!");
            println!("  Root node: {:?}", tree.root_node());
            println!("  Root kind: {}", tree.root_kind());
        }
        Err(e) => {
            println!("✗ Parse error (expected without real tables): {}", e);
        }
    }
}
