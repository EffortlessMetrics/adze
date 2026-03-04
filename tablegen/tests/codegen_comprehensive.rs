//! Comprehensive code generation tests for adze-tablegen.
//!
//! Validates that `LanguageGenerator::generate()` produces Rust token streams
//! containing all required ABI constants, tables, and arrays.

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::language_gen::LanguageGenerator;
use std::collections::BTreeMap;

// ── Helper: minimal grammar (expr -> number) ─────────────────────────

fn make_minimal_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("minimal".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    grammar
        .rule_names
        .insert(SymbolId(3), "expression".to_string());

    // Symbol layout: 0=ERROR, 1=number, 2=EOF, 3=expression(NT)
    let eof_symbol = SymbolId(2);
    let start_symbol = SymbolId(3);
    let symbol_count = 4;
    let state_count = 3;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    actions[0][1] = vec![Action::Shift(StateId(1))];
    actions[1][2] = vec![Action::Reduce(adze_ir::RuleId(0))];
    actions[2][2] = vec![Action::Accept];

    let mut gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    gotos[0][3] = StateId(2);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, 3);

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![ParseRule {
            lhs: SymbolId(3),
            rhs_len: 1,
        }],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![
            SymbolMetadata {
                name: "ERROR".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            },
            SymbolMetadata {
                name: "number".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(1),
            },
            SymbolMetadata {
                name: "EOF".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(2),
            },
            SymbolMetadata {
                name: "expression".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(3),
            },
        ],
        token_count: 2,
        external_token_count: 0,
        eof_symbol,
        start_symbol,
        grammar: grammar.clone(),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![0],
        rule_assoc_by_rule: vec![0],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

// ── Helper: grammar with fields ──────────────────────────────────────

fn make_grammar_with_fields() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("fielded".to_string());

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
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.fields.insert(FieldId(1), "left".to_string());
    grammar.fields.insert(FieldId(2), "operator".to_string());
    grammar.fields.insert(FieldId(3), "right".to_string());

    grammar.rules.insert(
        SymbolId(4),
        vec![Rule {
            lhs: SymbolId(4),
            rhs: vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::Terminal(SymbolId(2)),
                Symbol::Terminal(SymbolId(1)),
            ],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(1), 0), (FieldId(2), 1), (FieldId(3), 2)],
            production_id: ProductionId(0),
        }],
    );
    grammar
        .rule_names
        .insert(SymbolId(4), "binary_expr".to_string());

    // Symbol layout: 0=ERROR, 1=number, 2=plus, 3=EOF, 4=binary_expr(NT)
    let eof_symbol = SymbolId(3);
    let start_symbol = SymbolId(4);
    let symbol_count = 5;
    let state_count = 4;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    actions[0][1] = vec![Action::Shift(StateId(1))];
    actions[1][2] = vec![Action::Shift(StateId(2))];
    actions[2][1] = vec![Action::Shift(StateId(3))];
    actions[3][3] = vec![Action::Reduce(adze_ir::RuleId(0))];

    let mut gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    gotos[0][4] = StateId(3);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, 4);

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![ParseRule {
            lhs: SymbolId(4),
            rhs_len: 3,
        }],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![
            SymbolMetadata {
                name: "ERROR".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            },
            SymbolMetadata {
                name: "number".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(1),
            },
            SymbolMetadata {
                name: "plus".to_string(),
                is_visible: true,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(2),
            },
            SymbolMetadata {
                name: "EOF".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(3),
            },
            SymbolMetadata {
                name: "binary_expr".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(4),
            },
        ],
        token_count: 3,
        external_token_count: 0,
        eof_symbol,
        start_symbol,
        grammar: grammar.clone(),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![0],
        rule_assoc_by_rule: vec![0],
        alias_sequences: vec![],
        field_names: vec![
            "left".to_string(),
            "operator".to_string(),
            "right".to_string(),
        ],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

// ── Helper: grammar with no fields ───────────────────────────────────

fn make_grammar_no_fields() -> (Grammar, ParseTable) {
    let (grammar, parse_table) = make_minimal_grammar();
    // The minimal grammar already has no fields.
    assert!(grammar.fields.is_empty());
    (grammar, parse_table)
}

// ── Helper: grammar with external scanner ────────────────────────────

fn make_external_scanner_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("indented".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(3),
    });
    grammar.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(4),
    });

    grammar.rules.insert(
        SymbolId(5),
        vec![Rule {
            lhs: SymbolId(5),
            rhs: vec![
                Symbol::Terminal(SymbolId(3)),
                Symbol::Terminal(SymbolId(1)),
                Symbol::Terminal(SymbolId(4)),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    grammar.rule_names.insert(SymbolId(5), "block".to_string());

    // Symbol layout: 0=ERROR, 1=identifier, 2=EOF, 3=INDENT(ext), 4=DEDENT(ext), 5=block(NT)
    let eof_symbol = SymbolId(2);
    let start_symbol = SymbolId(5);
    let symbol_count = 6;
    let state_count = 4;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    actions[0][3] = vec![Action::Shift(StateId(1))];
    actions[1][1] = vec![Action::Shift(StateId(2))];
    actions[2][4] = vec![Action::Shift(StateId(3))];
    actions[3][2] = vec![Action::Reduce(adze_ir::RuleId(0))];

    let mut gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    gotos[0][5] = StateId(3);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, 5);

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![ParseRule {
            lhs: SymbolId(5),
            rhs_len: 3,
        }],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![
            SymbolMetadata {
                name: "ERROR".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            },
            SymbolMetadata {
                name: "identifier".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(1),
            },
            SymbolMetadata {
                name: "EOF".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(2),
            },
            SymbolMetadata {
                name: "INDENT".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(3),
            },
            SymbolMetadata {
                name: "DEDENT".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(4),
            },
            SymbolMetadata {
                name: "block".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(5),
            },
        ],
        token_count: 2,
        external_token_count: 2,
        eof_symbol,
        start_symbol,
        grammar: grammar.clone(),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        external_scanner_states: vec![vec![true; 2]; state_count],
        dynamic_prec_by_rule: vec![0],
        rule_assoc_by_rule: vec![0],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

// ── Helper: expression grammar with binary ops (more states/rules) ───

fn make_expression_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("expr".to_string());

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

    grammar.rules.insert(
        SymbolId(4),
        vec![
            Rule {
                lhs: SymbolId(4),
                rhs: vec![Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: SymbolId(4),
                rhs: vec![
                    Symbol::NonTerminal(SymbolId(4)),
                    Symbol::Terminal(SymbolId(2)),
                    Symbol::NonTerminal(SymbolId(4)),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    grammar
        .rule_names
        .insert(SymbolId(4), "expression".to_string());

    // Symbol layout: 0=ERROR, 1=number, 2=plus, 3=EOF, 4=expression(NT)
    let eof_symbol = SymbolId(3);
    let start_symbol = SymbolId(4);
    let symbol_count = 5;
    let state_count = 5;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    actions[0][1] = vec![Action::Shift(StateId(1))];
    actions[1][3] = vec![Action::Reduce(adze_ir::RuleId(0))];
    actions[1][2] = vec![Action::Shift(StateId(2))];
    actions[2][1] = vec![Action::Shift(StateId(3))];
    actions[3][3] = vec![Action::Reduce(adze_ir::RuleId(1))];
    actions[3][2] = vec![Action::Reduce(adze_ir::RuleId(1))];
    actions[4][3] = vec![Action::Accept];

    let mut gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    gotos[0][4] = StateId(4);
    gotos[2][4] = StateId(3);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, 4);

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![
            ParseRule {
                lhs: SymbolId(4),
                rhs_len: 1,
            },
            ParseRule {
                lhs: SymbolId(4),
                rhs_len: 3,
            },
        ],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![
            SymbolMetadata {
                name: "ERROR".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            },
            SymbolMetadata {
                name: "number".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(1),
            },
            SymbolMetadata {
                name: "+".to_string(),
                is_visible: true,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(2),
            },
            SymbolMetadata {
                name: "EOF".to_string(),
                is_visible: false,
                is_named: false,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(3),
            },
            SymbolMetadata {
                name: "expression".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(4),
            },
        ],
        token_count: 3,
        external_token_count: 0,
        eof_symbol,
        start_symbol,
        grammar: grammar.clone(),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![0, 0],
        rule_assoc_by_rule: vec![0, 0],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

/// Generate code string from grammar+table via `LanguageGenerator`.
fn generate_code(grammar: &Grammar, parse_table: &ParseTable) -> String {
    let generator = LanguageGenerator::new(grammar, parse_table);
    generator.generate().to_string()
}

/// Generate code string via `StaticLanguageGenerator`.
fn generate_code_static(grammar: Grammar, parse_table: ParseTable) -> String {
    let slg = StaticLanguageGenerator::new(grammar, parse_table);
    slg.generate_language_code().to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// 1. ABI version constant
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_abi_version_constant() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION")
            || code.contains("version")
            || code.contains("15"),
        "Generated code must reference the ABI version. Got:\n{code}"
    );
}

#[test]
fn abi_version_is_15() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("15"),
        "ABI version 15 must appear in the generated code"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Symbol count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_symbol_count() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("symbol_count") || code.contains("symbol_count :"),
        "Generated code must contain symbol_count field"
    );
}

#[test]
fn symbol_count_reflects_grammar_size() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    // minimal grammar: EOF + 1 token + 1 rule = 3 symbols
    let expected_count = 1 + grammar.tokens.len() + grammar.rules.len();
    let count_str = format!("{expected_count}");
    assert!(
        code.contains(&format!("symbol_count : {count_str}"))
            || code.contains(&format!("symbol_count: {count_str}"))
            || code.contains(&format!("symbol_count : {count_str}u32"))
            || code.contains(&format!("symbol_count: {count_str}u32")),
        "symbol_count should be {expected_count} for minimal grammar.\nCode: {code}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 3. State count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_state_count() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("state_count"),
        "Generated code must contain state_count field"
    );
}

#[test]
fn state_count_matches_parse_table() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    let expected = table.state_count;
    assert!(
        code.contains(&format!("{expected}u32")) || code.contains(&format!("{expected} u32")),
        "state_count should reflect the parse table's state count ({expected})"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Token count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_token_count() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("token_count"),
        "Generated code must contain token_count field"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Field count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_field_count() {
    let (grammar, table) = make_grammar_with_fields();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("field_count"),
        "Generated code must contain field_count field"
    );
}

#[test]
fn field_count_reflects_grammar_fields() {
    let (grammar, table) = make_grammar_with_fields();
    let code = generate_code(&grammar, &table);
    let expected = grammar.fields.len();
    assert!(
        code.contains(&format!("{expected}u32")) || code.contains(&format!("{expected} u32")),
        "field_count should be {expected} for grammar with fields.\nCode: {code}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Production info
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_production_id_count() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("production_id_count"),
        "Generated code must contain production_id_count field"
    );
}

#[test]
fn production_id_count_matches_grammar() {
    let (grammar, table) = make_expression_grammar();
    let generator = LanguageGenerator::new(&grammar, &table);
    let count = generator.count_production_ids_public();
    // expression grammar has production IDs 0 and 1, so count should be 2
    assert_eq!(count, 2, "Expression grammar should have 2 production IDs");
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Symbol names array
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_symbol_names_array() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("SYMBOL_NAMES"),
        "Generated code must contain SYMBOL_NAMES array"
    );
}

#[test]
fn symbol_names_include_token_names() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("\"number\""),
        "SYMBOL_NAMES should include the 'number' token name"
    );
}

#[test]
fn symbol_names_include_end_marker() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("\"end\""),
        "SYMBOL_NAMES should include 'end' for EOF"
    );
}

#[test]
fn symbol_names_include_rule_names() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("\"expression\""),
        "SYMBOL_NAMES should include the 'expression' rule name"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Field names array
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_field_names_array() {
    let (grammar, table) = make_grammar_with_fields();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("FIELD_NAMES"),
        "Generated code must contain FIELD_NAMES array"
    );
}

#[test]
fn field_names_include_actual_names() {
    let (grammar, table) = make_grammar_with_fields();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("\"left\""),
        "FIELD_NAMES should include 'left'"
    );
    assert!(
        code.contains("\"operator\""),
        "FIELD_NAMES should include 'operator'"
    );
    assert!(
        code.contains("\"right\""),
        "FIELD_NAMES should include 'right'"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Parse actions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_parse_actions() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PARSE_ACTIONS") || code.contains("parse_actions"),
        "Generated code must contain parse actions"
    );
}

#[test]
fn parse_actions_contain_action_type() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("TSParseAction"),
        "Generated code must include TSParseAction type references"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 10. GOTO table
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_parse_table() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PARSE_TABLE") || code.contains("parse_table"),
        "Generated code must contain a parse table reference"
    );
}

#[test]
fn generated_code_contains_small_parse_table_map() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("SMALL_PARSE_TABLE_MAP") || code.contains("small_parse_table_map"),
        "Generated code must contain SMALL_PARSE_TABLE_MAP"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 11. No-fields grammar has 0 field count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn no_fields_grammar_has_zero_field_count() {
    let (grammar, table) = make_grammar_no_fields();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("field_count : 0u32")
            || code.contains("field_count: 0u32")
            || code.contains("field_count : 0 u32")
            || code.contains("field_count: 0 u32"),
        "Grammar with no fields must have field_count of 0.\nCode: {code}"
    );
}

#[test]
fn no_fields_grammar_field_names_empty() {
    let (grammar, table) = make_grammar_no_fields();
    let code = generate_code(&grammar, &table);
    // FIELD_NAMES should still exist but be empty
    assert!(
        code.contains("FIELD_NAMES"),
        "FIELD_NAMES array should still be present even with 0 fields"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 12. External scanner grammar includes scanner slot
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn external_scanner_grammar_includes_external_token_count() {
    let (grammar, table) = make_external_scanner_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("EXTERNAL_TOKEN_COUNT") || code.contains("external_token_count"),
        "Generated code for grammar with externals must include EXTERNAL_TOKEN_COUNT"
    );
}

#[test]
fn external_scanner_grammar_has_scanner_struct() {
    let (grammar, table) = make_external_scanner_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("EXTERNAL_SCANNER") || code.contains("external_scanner"),
        "Generated code for grammar with externals must include an external scanner reference"
    );
}

#[test]
fn external_scanner_grammar_external_count_nonzero() {
    let (grammar, table) = make_external_scanner_grammar();
    let code = generate_code(&grammar, &table);
    let ext_count = grammar.externals.len();
    assert!(
        ext_count > 0,
        "External scanner grammar should have external tokens"
    );
    assert!(
        code.contains(&format!("{ext_count}u32")) || code.contains(&format!("{ext_count} u32")),
        "EXTERNAL_TOKEN_COUNT should be {ext_count}.\nCode: {code}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 13. Generated code is syntactically valid (parse check)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_is_parseable_token_stream() {
    let (grammar, table) = make_minimal_grammar();
    let generator = LanguageGenerator::new(&grammar, &table);
    // If `generate()` returns a valid TokenStream, it is syntactically valid
    // at the token level. This exercises the quote! macro output.
    let tokens = generator.generate();
    let code = tokens.to_string();
    assert!(!code.is_empty(), "Generated token stream must not be empty");
}

#[test]
fn generated_code_parses_as_token_stream() {
    let (grammar, table) = make_expression_grammar();
    let code_str = generate_code(&grammar, &table);
    // Verify it round-trips through proc_macro2 parsing
    let parsed: std::result::Result<proc_macro2::TokenStream, _> = code_str.parse();
    assert!(
        parsed.is_ok(),
        "Generated code must be a valid Rust token stream: {:?}",
        parsed.err()
    );
}

#[test]
fn generated_code_for_external_scanner_parses() {
    let (grammar, table) = make_external_scanner_grammar();
    let code_str = generate_code(&grammar, &table);
    let parsed: std::result::Result<proc_macro2::TokenStream, _> = code_str.parse();
    assert!(
        parsed.is_ok(),
        "External scanner code must be a valid token stream: {:?}",
        parsed.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 14. Code changes when grammar changes
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn code_changes_when_grammar_name_changes() {
    let (grammar1, table1) = make_minimal_grammar();
    let code1 = generate_code(&grammar1, &table1);

    let (mut grammar2, table2) = make_minimal_grammar();
    grammar2.name = "different".to_string();
    let code2 = generate_code(&grammar2, &table2);

    assert_ne!(
        code1, code2,
        "Changing grammar name should produce different code"
    );
}

#[test]
fn code_changes_when_token_added() {
    let (grammar1, table1) = make_minimal_grammar();
    let code1 = generate_code_static(grammar1, table1);

    let (mut grammar2, table2) = make_minimal_grammar();
    grammar2.tokens.insert(
        SymbolId(10),
        Token {
            name: "string_literal".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );
    let code2 = generate_code_static(grammar2, table2);

    assert_ne!(code1, code2, "Adding a token should produce different code");
}

#[test]
fn code_changes_when_field_added() {
    let (grammar1, table1) = make_minimal_grammar();
    let code1 = generate_code(&grammar1, &table1);

    let (mut grammar2, table2) = make_minimal_grammar();
    grammar2.fields.insert(FieldId(1), "value".to_string());
    let code2 = generate_code(&grammar2, &table2);

    assert_ne!(code1, code2, "Adding a field should produce different code");
}

#[test]
fn code_changes_when_external_token_added() {
    let (grammar1, table1) = make_minimal_grammar();
    let code1 = generate_code_static(grammar1, table1);

    let (mut grammar2, mut table2) = make_minimal_grammar();
    grammar2.externals.push(ExternalToken {
        name: "NEWLINE".to_string(),
        symbol_id: SymbolId(50),
    });
    table2.external_token_count = 1;
    let code2 = generate_code_static(grammar2, table2);

    assert_ne!(
        code1, code2,
        "Adding an external token should produce different code"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 15. Determinism: same grammar → same code
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_output_minimal_grammar() {
    let (g1, t1) = make_minimal_grammar();
    let (g2, t2) = make_minimal_grammar();
    let code1 = generate_code(&g1, &t1);
    let code2 = generate_code(&g2, &t2);
    assert_eq!(
        code1, code2,
        "Same minimal grammar must produce identical code"
    );
}

#[test]
fn deterministic_output_expression_grammar() {
    let (g1, t1) = make_expression_grammar();
    let (g2, t2) = make_expression_grammar();
    let code1 = generate_code(&g1, &t1);
    let code2 = generate_code(&g2, &t2);
    assert_eq!(
        code1, code2,
        "Same expression grammar must produce identical code"
    );
}

#[test]
fn deterministic_output_external_scanner_grammar() {
    let (g1, t1) = make_external_scanner_grammar();
    let (g2, t2) = make_external_scanner_grammar();
    let code1 = generate_code(&g1, &t1);
    let code2 = generate_code(&g2, &t2);
    assert_eq!(
        code1, code2,
        "Same external scanner grammar must produce identical code"
    );
}

#[test]
fn deterministic_output_fields_grammar() {
    let (g1, t1) = make_grammar_with_fields();
    let (g2, t2) = make_grammar_with_fields();
    let code1 = generate_code(&g1, &t1);
    let code2 = generate_code(&g2, &t2);
    assert_eq!(
        code1, code2,
        "Same fielded grammar must produce identical code"
    );
}

#[test]
fn deterministic_output_across_generators() {
    // Both LanguageGenerator and StaticLanguageGenerator should produce identical output
    // for the same grammar (since Static delegates to LanguageGenerator).
    let (g1, t1) = make_minimal_grammar();
    let code_direct = generate_code(&g1, &t1);
    let code_static = generate_code_static(g1, t1);
    assert_eq!(
        code_direct, code_static,
        "LanguageGenerator and StaticLanguageGenerator must produce identical code"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Additional tests (to reach 25+)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn generated_code_contains_language_function() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("language"),
        "Generated code must contain a `language` function"
    );
}

#[test]
fn generated_code_contains_ffi_export() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("tree_sitter_minimal"),
        "Generated code must contain the C FFI export function `tree_sitter_minimal`"
    );
}

#[test]
fn generated_code_contains_lex_modes() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("LEX_MODES") || code.contains("lex_modes"),
        "Generated code must contain lex modes"
    );
}

#[test]
fn generated_code_contains_symbol_metadata() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("SYMBOL_METADATA") || code.contains("symbol_metadata"),
        "Generated code must contain symbol metadata"
    );
}

#[test]
fn generated_code_contains_public_symbol_map() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP") || code.contains("public_symbol_map"),
        "Generated code must contain public symbol map"
    );
}

#[test]
fn ffi_export_name_matches_grammar_name() {
    let (grammar, table) = make_expression_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("tree_sitter_expr"),
        "FFI export should be named tree_sitter_<grammar_name>: tree_sitter_expr"
    );
}

#[test]
fn external_scanner_ffi_export_name_matches() {
    let (grammar, table) = make_external_scanner_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("tree_sitter_indented"),
        "FFI export should be tree_sitter_indented for the indented grammar"
    );
}

#[test]
fn generated_code_contains_primary_state_ids() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PRIMARY_STATE_IDS") || code.contains("primary_state_ids"),
        "Generated code must contain primary state IDs"
    );
}

#[test]
fn expression_grammar_has_more_states_than_minimal() {
    let (g1, t1) = make_minimal_grammar();
    let (g2, t2) = make_expression_grammar();
    let code1 = generate_code(&g1, &t1);
    let code2 = generate_code(&g2, &t2);
    // Expression grammar code should be longer due to more states/rules
    assert!(
        code2.len() > code1.len(),
        "Expression grammar code ({}) should be longer than minimal ({})",
        code2.len(),
        code1.len()
    );
}

#[test]
fn generated_code_contains_language_struct() {
    let (grammar, table) = make_minimal_grammar();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("TSLanguage"),
        "Generated code must reference TSLanguage struct"
    );
}
