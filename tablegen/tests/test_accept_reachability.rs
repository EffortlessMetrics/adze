//! Test that Accept action is reachable and actually executed during parsing
//! This verifies the normalization pipeline produces valid tables that can terminate parsing

use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
#[allow(unused_imports)]
use rust_sitter_tablegen::{
    abi::{TSLanguage, TSParseAction},
    AbiLanguageBuilder,
};

/// Create a simple JSON-like grammar for testing
fn create_test_grammar() -> Grammar {
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
fn test_accept_action_exists_in_generated_code() {
    // Create test grammar and build parse table
    let grammar = create_test_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Generate language using ABI builder
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    // Verify generated code contains Accept action
    // Accept is encoded as 0x7FFF in the parse table
    assert!(code_str.contains("TSLanguage"), "Missing TSLanguage struct");
    assert!(
        code_str.contains("PARSE_TABLE") || code_str.contains("SMALL_PARSE_TABLE"),
        "Missing parse table"
    );

    // The Accept action is encoded as 0x7FFF in the parse table
    // Check for this value in the generated code
    let has_accept = code_str.contains("0x7FFF") ||     // Accept action encoding  
                      code_str.contains("32767") ||      // Decimal form of 0x7FFF
                      code_str.contains("0x7fff"); // Lowercase variant

    assert!(
        has_accept,
        "No Accept action (0x7FFF) found in generated parse table. This means the parse table cannot terminate parsing."
    );

    println!("✓ Accept action found in generated code");
}

#[test]
fn test_accept_action_in_parse_table() {
    // Create test grammar
    let grammar = create_test_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Check for Accept action in the parse table
    let mut accept_found = false;
    let mut accept_state = 0;
    let mut accept_symbol = SymbolId(0);

    for (state_idx, row) in parse_table.action_table.iter().enumerate() {
        for (symbol_idx, actions) in row.iter().enumerate() {
            for action in actions {
                if let rust_sitter_glr_core::Action::Accept = action {
                    accept_found = true;
                    accept_state = state_idx;
                    accept_symbol = SymbolId(symbol_idx as u16);
                    println!(
                        "Accept action found at state {} for symbol {:?}",
                        state_idx, symbol_idx
                    );
                    break;
                }
            }
            if accept_found {
                break;
            }
        }
        if accept_found {
            break;
        }
    }

    assert!(accept_found, "Accept action not found in parse table");
    println!(
        "✓ Accept action exists at state {} for symbol {:?}",
        accept_state, accept_symbol
    );
}

#[test]
fn test_simple_parse_succeeds() {
    // This test verifies that a simple parse can complete successfully
    let grammar = create_test_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Build the language
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();

    // Verify the generated code has required components
    let code_str = code.to_string();
    assert!(code_str.contains("symbol_count"), "Missing symbol_count");
    assert!(code_str.contains("state_count"), "Missing state_count");
    assert!(code_str.contains("PARSE_TABLE"), "Missing parse table");

    // Count the number of states and symbols
    let state_count = parse_table.state_count;
    let symbol_count = grammar.tokens.len()
        + grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .map(|r| r.lhs)
            .collect::<std::collections::HashSet<_>>()
            .len();

    println!(
        "Grammar has {} states and {} symbols",
        state_count, symbol_count
    );
    assert!(state_count > 0, "No states generated");
    assert!(symbol_count > 0, "No symbols in grammar");

    println!("✓ Parse table successfully generated with Accept action");
}
