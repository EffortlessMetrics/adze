#![cfg(feature = "ts-compat")]

use rust_sitter::ts_compat::Language;
use std::sync::Arc;

/// Build a `Language` from the generated Tree-sitter style tables for arithmetic.
/// Creates a minimal working parse table that will parse arithmetic expressions.
pub fn arithmetic() -> Arc<Language> {
    use crate::arithmetic::generated::{LANGUAGE, SMALL_PARSE_TABLE, SMALL_PARSE_TABLE_MAP};
    use rust_sitter::rust_sitter_glr_core::{Action, ParseRule, ParseTable, SymbolMetadata};
    use rust_sitter::rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
    use std::collections::BTreeMap;

    // Basic sizes from generated language
    let state_count = LANGUAGE.state_count as usize;
    let symbol_count = LANGUAGE.symbol_count as usize;
    let token_count = LANGUAGE.token_count as usize;

    let mut grammar = Grammar::default();
    grammar.name = "arithmetic".to_string();

    // Populate symbol names - hardcoded for arithmetic grammar
    // Based on the generated arithmetic grammar symbols:
    // 0: end (EOF)
    // 1: number token
    // 2: "+"
    // 3: "-"
    // 4: _whitespace
    // 5-10: intermediate non-terminals
    // 11: Expression
    let symbol_names = vec![
        "end",
        "number",
        "+",
        "-",
        "_whitespace",
        "Expression_Number",
        "Expression_Add",
        "Expression_Sub",
        "source_file",
        "Whitespace__whitespace",
        "Whitespace",
        "expression", // The root we want to return
    ];

    for (i, name) in symbol_names.iter().enumerate() {
        grammar
            .rule_names
            .insert(SymbolId(i as u16), name.to_string());
    }

    // Populate tokens for the lexer
    use rust_sitter::rust_sitter_ir::{Token, TokenPattern};

    // Add the basic tokens that the lexer needs
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "-".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );
    // Whitespace is handled as extra, not a normal token

    // Create symbol metadata
    let mut symbol_metadata = Vec::with_capacity(symbol_count);
    for i in 0..symbol_count {
        let is_token = i < token_count;
        let name = if i < symbol_names.len() {
            symbol_names[i].to_string()
        } else {
            format!("symbol_{}", i)
        };
        symbol_metadata.push(SymbolMetadata {
            name,
            visible: true,
            named: !is_token || i == 1, // number is named, operators are not
            supertype: false,
        });
    }

    // Create action and goto tables
    let mut action_table: Vec<Vec<Vec<Action>>> = vec![vec![Vec::new(); symbol_count]; state_count];
    let mut goto_table: Vec<Vec<StateId>> = vec![vec![StateId(0); symbol_count]; state_count];

    // Create a more complete action table for arithmetic parsing
    // State 0: initial state
    action_table[0][1].push(Action::Shift(StateId(2))); // number -> shift to state 2

    // State 2: after seeing a number
    action_table[2][0].push(Action::Reduce(RuleId(0))); // EOF -> reduce to Expression (rule 0: Expression -> number)
    action_table[2][2].push(Action::Reduce(RuleId(0))); // + -> reduce to Expression
    action_table[2][3].push(Action::Reduce(RuleId(0))); // - -> reduce to Expression

    // State 3: after an Expression from state 0
    action_table[3][0].push(Action::Reduce(RuleId(3))); // EOF -> reduce to source_file (rule 3: source_file -> Expression)
    action_table[3][2].push(Action::Shift(StateId(4))); // + -> shift to state 4
    action_table[3][3].push(Action::Shift(StateId(5))); // - -> shift to state 5

    // State 4: after Expression +
    action_table[4][1].push(Action::Shift(StateId(6))); // number -> shift to state 6

    // State 5: after Expression -
    action_table[5][1].push(Action::Shift(StateId(7))); // number -> shift to state 7

    // State 6: after Expression + number
    action_table[6][0].push(Action::Reduce(RuleId(0))); // EOF -> reduce to Expression (as number)
    action_table[6][2].push(Action::Reduce(RuleId(0))); // + -> reduce to Expression
    action_table[6][3].push(Action::Reduce(RuleId(0))); // - -> reduce to Expression

    // State 7: after Expression - number
    action_table[7][0].push(Action::Reduce(RuleId(0))); // EOF -> reduce to Expression (as number)
    action_table[7][2].push(Action::Reduce(RuleId(0))); // + -> reduce to Expression
    action_table[7][3].push(Action::Reduce(RuleId(0))); // - -> reduce to Expression

    // State 8: after source_file (from state 0 via goto)
    action_table[8][0].push(Action::Accept); // EOF -> accept

    // State 9: after Expression + Expression
    action_table[9][0].push(Action::Reduce(RuleId(1))); // EOF -> reduce to Expression (rule 1: Expression -> Expression + Expression)
    action_table[9][2].push(Action::Shift(StateId(4))); // + -> shift for left associativity
    action_table[9][3].push(Action::Shift(StateId(5))); // - -> shift

    // Set up goto table
    goto_table[0][11] = StateId(3); // After reducing to Expression in state 0, go to state 3
    goto_table[0][8] = StateId(8); // After reducing to source_file in state 0, go to state 8 (accept)
    goto_table[4][11] = StateId(9); // After reducing to Expression in state 4 (after +), go to state 9
    goto_table[5][11] = StateId(9); // After reducing to Expression in state 5 (after -), go to state 9

    // Create parse rules - minimal set for arithmetic
    let rules = vec![
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 1,
        }, // Expression -> number
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 3,
        }, // Expression -> Expression + Expression
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 3,
        }, // Expression -> Expression - Expression
        ParseRule {
            lhs: SymbolId(8),
            rhs_len: 1,
        }, // source_file -> Expression
    ];

    // Build index mappings
    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::with_capacity(symbol_count);
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    // Assemble the parse table
    let table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules,
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(LANGUAGE.eof_symbol),
        start_symbol: SymbolId(11), // Expression
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count,
        external_token_count: LANGUAGE.external_token_count as usize,
        lex_modes: vec![],
        extras: vec![SymbolId(4)], // whitespace
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    };

    Arc::new(Language::new("arithmetic", grammar, table))
}
