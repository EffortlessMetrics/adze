#![cfg(feature = "ts-compat")]

use rust_sitter::ts_compat::Language;
use std::sync::Arc;

/// Build a `Language` from the generated Tree-sitter style tables for arithmetic.
/// Creates a minimal working parse table that will parse arithmetic expressions.
pub fn arithmetic() -> Arc<Language> {
    use crate::arithmetic::generated::{LANGUAGE, SMALL_PARSE_TABLE, SMALL_PARSE_TABLE_MAP};
    use rust_sitter::rust_sitter_glr_core::{
        Action, GotoIndexing, ParseRule, ParseTable, SymbolMetadata,
    };
    use rust_sitter::rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
    use std::collections::BTreeMap;

    // Basic sizes from generated language
    // Need at least 11 states for our manual parse table (states 0-10)
    let state_count = std::cmp::max(11, LANGUAGE.state_count as usize);
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
        "end",                    // 0
        "*",                      // 1
        "_whitespace",            // 2
        "-",                      // 3
        "number",                 // 4
        "Expression",             // 5
        "source_file",            // 6
        "Expression_Mul",         // 7
        "whitespace_pattern",     // 8
        "Whitespace__whitespace", // 9
        "Expression_Sub",         // 10
        "Expression_Number",      // 11
    ];

    for (i, name) in symbol_names.iter().enumerate() {
        grammar
            .rule_names
            .insert(SymbolId(i as u16), name.to_string());
    }

    // Populate tokens for the lexer
    use rust_sitter::rust_sitter_ir::{Token, TokenPattern};

    // Add the basic tokens that the lexer needs
    // Symbols from generated parser:
    // 1: * (multiplication)
    // 3: - (subtraction)
    // 4: number
    grammar.tokens.insert(
        SymbolId(4),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "*".to_string(),
            pattern: TokenPattern::String("*".to_string()),
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
    // Symbol IDs from generated parser:
    // 0: EOF
    // 1: * (multiplication)
    // 2: whitespace (extra)
    // 3: - (subtraction)
    // 4: number
    // 5: Expression
    // 6: source_file
    // 7: Expression_Mul
    // 10: Expression_Sub
    // 11: Expression_Number

    // State 0: initial state
    action_table[0][4].push(Action::Shift(StateId(2))); // number -> shift to state 2

    // State 2: after seeing a number
    action_table[2][0].push(Action::Reduce(RuleId(0))); // EOF -> reduce to Expression (rule 0: Expression -> number)
    action_table[2][1].push(Action::Reduce(RuleId(0))); // * -> reduce to Expression
    action_table[2][3].push(Action::Reduce(RuleId(0))); // - -> reduce to Expression

    // State 3: after an Expression from state 0
    action_table[3][0].push(Action::Reduce(RuleId(3))); // EOF -> reduce to source_file (rule 3: source_file -> Expression)
    action_table[3][1].push(Action::Shift(StateId(4))); // * -> shift to state 4 (higher precedence)
    action_table[3][3].push(Action::Shift(StateId(5))); // - -> shift to state 5

    // State 4: after Expression *
    action_table[4][4].push(Action::Shift(StateId(2))); // number -> shift to state 2 (reuse number state)

    // State 5: after Expression -
    action_table[5][4].push(Action::Shift(StateId(2))); // number -> shift to state 2 (reuse number state)

    // State 8: after source_file (from state 0 via goto)
    action_table[8][0].push(Action::Accept); // EOF -> accept

    // State 9: after Expression * Expression
    action_table[9][0].push(Action::Reduce(RuleId(1))); // EOF -> reduce to Expression (rule 1: Expression -> Expression * Expression)
    action_table[9][1].push(Action::Reduce(RuleId(1))); // * -> reduce first (left associativity)
    action_table[9][3].push(Action::Reduce(RuleId(1))); // - -> reduce first (multiplication has higher precedence)

    // State 10: after Expression - Expression
    action_table[10][0].push(Action::Reduce(RuleId(2))); // EOF -> reduce to Expression (rule 2: Expression -> Expression - Expression)
    action_table[10][1].push(Action::Shift(StateId(4))); // * -> shift (multiplication has higher precedence)
    action_table[10][3].push(Action::Reduce(RuleId(2))); // - -> reduce first (left associativity)

    // Set up goto table
    goto_table[0][5] = StateId(3); // After reducing to Expression in state 0, go to state 3
    goto_table[0][6] = StateId(8); // After reducing to source_file in state 0, go to state 8 (accept)
    goto_table[4][5] = StateId(9); // After reducing to Expression in state 4 (after *), go to state 9
    goto_table[5][5] = StateId(10); // After reducing to Expression in state 5 (after -), go to state 10

    // Create parse rules - minimal set for arithmetic
    let rules = vec![
        ParseRule {
            lhs: SymbolId(5), // Expression
            rhs_len: 1,
        }, // Expression -> number
        ParseRule {
            lhs: SymbolId(5), // Expression
            rhs_len: 3,
        }, // Expression -> Expression * Expression
        ParseRule {
            lhs: SymbolId(5), // Expression
            rhs_len: 3,
        }, // Expression -> Expression - Expression
        ParseRule {
            lhs: SymbolId(6), // source_file
            rhs_len: 1,
        }, // source_file -> Expression
    ];

    // Also populate grammar.rules for parser_v4 compatibility
    use rust_sitter::rust_sitter_ir::{ProductionId, Rule, Symbol};

    // Expression rules
    let expr_rules = vec![
        // Expression -> number
        Rule {
            lhs: SymbolId(5),                         // Expression
            rhs: vec![Symbol::Terminal(SymbolId(4))], // number
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        },
        // Expression -> Expression * Expression
        Rule {
            lhs: SymbolId(5), // Expression
            rhs: vec![
                Symbol::NonTerminal(SymbolId(5)), // Expression
                Symbol::Terminal(SymbolId(1)),    // *
                Symbol::NonTerminal(SymbolId(5)), // Expression
            ],
            precedence: Some(rust_sitter::rust_sitter_ir::PrecedenceKind::Static(2)), // Higher precedence for multiplication
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        },
        // Expression -> Expression - Expression
        Rule {
            lhs: SymbolId(5), // Expression
            rhs: vec![
                Symbol::NonTerminal(SymbolId(5)), // Expression
                Symbol::Terminal(SymbolId(3)),    // -
                Symbol::NonTerminal(SymbolId(5)), // Expression
            ],
            precedence: Some(rust_sitter::rust_sitter_ir::PrecedenceKind::Static(1)), // Lower precedence for subtraction
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        },
    ];
    grammar.rules.insert(SymbolId(5), expr_rules);

    // source_file rule
    let source_file_rules = vec![Rule {
        lhs: SymbolId(6),                            // source_file
        rhs: vec![Symbol::NonTerminal(SymbolId(5))], // Expression
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(3),
    }];
    grammar.rules.insert(SymbolId(6), source_file_rules);

    // Build index mappings
    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::with_capacity(symbol_count);
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    // Assemble the parse table
    let mut table = ParseTable {
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
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(LANGUAGE.eof_symbol),
        start_symbol: SymbolId(6), // source_file - the actual start symbol
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count,
        external_token_count: LANGUAGE.external_token_count as usize,
        lex_modes: vec![],
        extras: vec![SymbolId(4)], // whitespace
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    };

    // Auto-detect GOTO indexing mode
    table.detect_goto_indexing();

    Arc::new(Language::new("arithmetic", grammar, table))
}
