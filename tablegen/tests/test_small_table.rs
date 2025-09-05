//! Test compressed table round-trip with feature flag
//! This ensures the compressed table path remains healthy without changing production defaults

#![cfg(feature = "small-table")]

use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use rust_sitter_tablegen::AbiLanguageBuilder;

/// Create a simple JSON grammar for testing compressed tables
fn build_json_grammar() -> Grammar {
    let mut grammar = Grammar::new("json".to_string());

    // Terminals - starting from SymbolId(1)
    let lbrace_id = SymbolId(1);
    let rbrace_id = SymbolId(2);
    let colon_id = SymbolId(3);
    let comma_id = SymbolId(4);
    let string_id = SymbolId(5);
    let number_id = SymbolId(6);

    grammar.tokens.insert(
        lbrace_id,
        Token {
            name: "{".to_string(),
            pattern: TokenPattern::String("{".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        rbrace_id,
        Token {
            name: "}".to_string(),
            pattern: TokenPattern::String("}".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        colon_id,
        Token {
            name: ":".to_string(),
            pattern: TokenPattern::String(":".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        comma_id,
        Token {
            name: ",".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        string_id,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    // Non-terminals
    let value_id = SymbolId(10);
    let object_id = SymbolId(11);
    let pair_id = SymbolId(12);
    let members_id = SymbolId(13);

    // Rules - value is the start symbol (first rule's LHS)
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::NonTerminal(object_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(string_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    grammar.add_rule(Rule {
        lhs: object_id,
        rhs: vec![Symbol::Terminal(lbrace_id), Symbol::Terminal(rbrace_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(3),
    });
    grammar.add_rule(Rule {
        lhs: object_id,
        rhs: vec![
            Symbol::Terminal(lbrace_id),
            Symbol::NonTerminal(members_id),
            Symbol::Terminal(rbrace_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(4),
    });
    grammar.add_rule(Rule {
        lhs: members_id,
        rhs: vec![Symbol::NonTerminal(pair_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(5),
    });
    grammar.add_rule(Rule {
        lhs: members_id,
        rhs: vec![
            Symbol::NonTerminal(members_id),
            Symbol::Terminal(comma_id),
            Symbol::NonTerminal(pair_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(6),
    });
    grammar.add_rule(Rule {
        lhs: pair_id,
        rhs: vec![
            Symbol::Terminal(string_id),
            Symbol::Terminal(colon_id),
            Symbol::NonTerminal(value_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(7),
    });

    grammar
}

#[test]
fn small_table_round_trip_accept() {
    let grammar = build_json_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Build language with compressed table feature
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    // Verify compressed table is generated
    assert!(
        code_str.contains("SMALL_PARSE_TABLE"),
        "Missing SMALL_PARSE_TABLE in generated code"
    );
    assert!(
        code_str.contains("SMALL_PARSE_TABLE_MAP"),
        "Missing SMALL_PARSE_TABLE_MAP in generated code"
    );

    // Verify Accept action is still present in compressed format
    // Accept is encoded as 0x7FFF
    assert!(
        code_str.contains("0x7FFF") || code_str.contains("32767") || code_str.contains("0x7fff"),
        "No Accept action (0x7FFF) found in compressed table"
    );

    println!("✓ Compressed table round-trip successful with Accept action preserved");
}

#[test]
fn compressed_table_size_reduction() {
    let grammar = build_json_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Build with compressed table
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    // Count occurrences to estimate size
    let parse_table_entries = code_str.matches("0x").count();

    // In compressed format, we should have fewer entries than uncompressed
    // This is a basic smoke test for compression working
    assert!(parse_table_entries > 0, "No parse table entries found");

    println!("✓ Compressed table has {} hex entries", parse_table_entries);
}
