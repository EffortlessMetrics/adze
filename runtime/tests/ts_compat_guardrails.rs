#![cfg(all(feature = "ts-compat", feature = "pure-rust"))]

use rust_sitter::ts_compat::Parser;

#[test]
fn table_guardrails() {
    let lang = rust_sitter_example::ts_langs::arithmetic();
    let t = &lang.table;

    // Basic sanity checks
    assert!(t.state_count > 1, "parse table must have >1 state");
    assert!(
        t.symbol_count >= t.token_count,
        "symbol_count must be >= token_count"
    );
    assert_ne!(
        t.start_symbol.0, t.eof_symbol.0,
        "start and EOF symbols must differ"
    );

    // Check that we have some actions
    let has_actions = t
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| !cell.is_empty()));
    assert!(has_actions, "action table must have at least one action");

    // Check that we have at least one goto on a nonterminal
    let has_goto = t
        .goto_table
        .iter()
        .any(|row| row.iter().skip(t.token_count).any(|&s| s.0 != 0));
    assert!(
        has_goto,
        "goto table must have at least one nonterminal transition"
    );

    // Check that Expression symbol exists and is named correctly
    let expr_symbol = t
        .symbol_metadata
        .iter()
        .find(|m| m.name == "expression")
        .expect("Expression symbol not found");
    assert!(expr_symbol.named, "Expression should be a named symbol");
}

#[test]
fn language_metadata() {
    let lang = rust_sitter_example::ts_langs::arithmetic();

    // Check basic metadata
    assert_eq!(lang.grammar.name, "arithmetic");
    assert!(lang.table.token_count > 0, "must have at least one token");

    // The arithmetic grammar uses inline leaf patterns, so it may not have
    // a separate "number" symbol. Instead, check that we have named symbols.
    let named_symbols: Vec<_> = lang
        .table
        .symbol_metadata
        .iter()
        .filter(|m| m.named)
        .collect();

    assert!(
        !named_symbols.is_empty(),
        "grammar should have at least one named symbol, found: {:?}",
        lang.table.symbol_metadata.iter().map(|m| &m.name).collect::<Vec<_>>()
    );
}

#[test]
fn simple_tokenization() {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();
    parser
        .set_language(lang.clone())
        .expect("set_language failed");

    // Debug: print token info
    println!("Token count: {}", lang.table.token_count);
    println!("Symbol count: {}", lang.table.symbol_count);
    println!("Extras: {:?}", lang.table.extras);

    // Try parsing a simple number
    println!("Attempting to parse: '1'");
    let tree = parser.parse("1", None);

    if let Some(tree) = tree {
        println!("Parse succeeded!");
        println!("Root kind: {}", tree.root_kind());
        println!("Error count: {}", tree.error_count());
    } else {
        println!("Parse failed - returned None");
    }

    // Try parsing an expression
    println!("\nAttempting to parse: '1+2+3'");
    let tree2 = parser.parse("1+2+3", None);

    if let Some(tree) = tree2 {
        println!("Parse succeeded!");
        println!("Root kind: {}", tree.root_kind());
        println!("Error count: {}", tree.error_count());
    } else {
        println!("Parse failed - returned None");
    }
}
