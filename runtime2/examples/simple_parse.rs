//! Simple example showing the Tree-sitter-compatible API

use rust_sitter_runtime::{language::SymbolMetadata, Language, Parser};

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

fn stub_language() -> Language {
    let table = empty_parse_table();
    let builder = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["placeholder".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(vec![]);

    #[cfg(feature = "glr-core")]
    let builder = builder.tokenizer(|_| Box::new(std::iter::empty()));

    builder.build().unwrap()
}

fn main() {
    // Create a parser
    let mut parser = Parser::new();

    // In a real scenario, you'd load a language from a generated crate
    // For now, we build a stub language
    let language = stub_language();

    // Set the language
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Parse some input (will return a stub tree for now)
    let input = "def hello():\n    print('Hello, world!')";
    match parser.parse_utf8(input, None) {
        Ok(tree) => {
            println!("Parse successful!");
            println!("Root node: {:?}", tree.root_node());

            // In a real implementation, you could walk the tree
            let root = tree.root_node();
            println!("Node kind: {}", root.kind());
            println!("Byte range: {:?}", root.byte_range());
            println!("Child count: {}", root.child_count());
        }
        Err(e) => {
            eprintln!("Parse failed: {}", e);
        }
    }

    // Example of incremental parsing (when implemented)
    println!("\nIncremental parsing example:");
    let edited_input = "def hello():\n    print('Hello, Rust!')";
    match parser.parse_utf8(edited_input, None) {
        // Would pass old_tree in real usage
        Ok(_tree) => println!("Incremental parse successful!"),
        Err(e) => eprintln!("Incremental parse failed: {}", e),
    }
}
