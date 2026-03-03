//! Snapshot tests for tablegen code generation output.
//!
//! These tests use `insta` to snapshot the generated Language struct code,
//! compressed action tables, NODE_TYPES JSON, symbol metadata, and
//! production ID maps.

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{
    ExternalToken, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::StaticLanguageGenerator;
use std::collections::BTreeMap;

/// Build a minimal arithmetic-like grammar: `expr -> number`
fn make_minimal_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("minimal".to_string());

    // Token: number (regex)
    let num_token = Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), num_token);

    // Token: "+" (string literal)
    let plus_token = Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(2), plus_token);

    // Rule: expr -> number
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.rules.insert(SymbolId(3), vec![rule]);
    grammar
        .rule_names
        .insert(SymbolId(3), "expression".to_string());

    // Symbol layout: 0=ERROR, 1=number, 2=plus, 3=EOF, 4=expression(NT)
    let eof_symbol = SymbolId(3);
    let start_symbol = SymbolId(4);
    let symbol_count = 5;
    let state_count = 3;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    // Action table: 3 states x 5 symbols
    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    // State 0: shift number -> state 1
    actions[0][1] = vec![Action::Shift(StateId(1))];
    // State 1: reduce expr -> number on EOF
    actions[1][3] = vec![Action::Reduce(adze_ir::RuleId(0))];
    // State 2: accept on EOF
    actions[2][3] = vec![Action::Accept];

    let gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, 4);

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
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
        token_count: 3, // ERROR, number, plus (EOF is at index 3 = token_count + 0 externals)
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
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

// ── 1. Language struct code generation ────────────────────────────────

#[test]
fn snapshot_language_struct_codegen() {
    let (grammar, parse_table) = make_minimal_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();
    // Pretty-print the token stream for a stable, readable snapshot
    let formatted = prettyprint_tokens(&code.to_string());
    insta::assert_snapshot!("language_struct_codegen", formatted);
}

// ── 2. Compressed action table output format ─────────────────────────

#[test]
fn snapshot_compressed_action_table() {
    let (grammar, parse_table) = make_minimal_grammar();
    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
    // compress_tables may fail if state-0 validation doesn't pass;
    // that's fine—we snapshot the error in that case.
    let result = generator.compress_tables();
    let output = match result {
        Ok(()) => {
            let tables = generator
                .compressed_tables
                .as_ref()
                .expect("compressed_tables should be Some after compress_tables()");
            format_compressed_action_table(&tables.action_table)
        }
        Err(e) => format!("compression error: {e}"),
    };
    insta::assert_snapshot!("compressed_action_table", output);
}

// ── 3. NODE_TYPES JSON generation ────────────────────────────────────

#[test]
fn snapshot_node_types_json() {
    let (grammar, parse_table) = make_minimal_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let json = generator.generate_node_types();
    insta::assert_snapshot!("node_types_json", json);
}

// ── 4. Symbol metadata array generation ──────────────────────────────

#[test]
fn snapshot_symbol_metadata() {
    let (grammar, parse_table) = make_minimal_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let metadata = generator.generate_symbol_metadata_public();
    let formatted = metadata
        .iter()
        .enumerate()
        .map(|(i, byte)| format!("[{i}] 0b{byte:08b}"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("symbol_metadata", formatted);
}

// ── 5. Production ID map generation ──────────────────────────────────

#[test]
fn snapshot_production_id_map() {
    let (grammar, parse_table) = make_minimal_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let count = generator.count_production_ids_public();
    let output = format!("production_id_count: {count}");
    insta::assert_snapshot!("production_id_map", output);
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Minimal pretty-printer: insert newlines at semicolons and braces for readability.
fn prettyprint_tokens(raw: &str) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                out.push('{');
                out.push('\n');
                indent += 1;
                push_indent(&mut out, indent);
            }
            '}' => {
                out.push('\n');
                indent = indent.saturating_sub(1);
                push_indent(&mut out, indent);
                out.push('}');
                if chars.peek() != Some(&';') {
                    out.push('\n');
                    push_indent(&mut out, indent);
                }
            }
            ';' => {
                out.push(';');
                out.push('\n');
                push_indent(&mut out, indent);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("    ");
    }
}

fn format_compressed_action_table(table: &adze_tablegen::CompressedActionTable) -> String {
    let mut out = String::new();
    out.push_str(&format!("row_offsets: {:?}\n", table.row_offsets));
    out.push_str(&format!("default_actions: {:?}\n", table.default_actions));
    out.push_str("entries:\n");
    for (i, entry) in table.data.iter().enumerate() {
        out.push_str(&format!(
            "  [{i}] symbol={}, action={:?}\n",
            entry.symbol, entry.action
        ));
    }
    out
}

// ── Grammar builders for additional tests ───────────────────────────

/// Build an expression grammar with binary operators: `expr -> expr '+' expr | number`
///
/// Symbol layout:
///   0: ERROR, 1: number, 2: plus, 3: EOF, 4: expr (NT)
///
/// States:
///   0: initial (shift number->1)
///   1: saw number (reduce or shift plus)
///   2: saw plus (shift number->3)
///   3: saw second number (reduce)
///   4: accepted expr
fn make_expression_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("expr".to_string());

    let num_token = Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), num_token);

    let plus_token = Token {
        name: "+".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(2), plus_token);

    // Rule 0: expr -> number
    let rule0 = Rule {
        lhs: SymbolId(4),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    // Rule 1: expr -> expr '+' expr
    let rule1 = Rule {
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
    };
    grammar.rules.insert(SymbolId(4), vec![rule0, rule1]);
    grammar
        .rule_names
        .insert(SymbolId(4), "expression".to_string());

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
    // State 0: shift number -> 1
    actions[0][1] = vec![Action::Shift(StateId(1))];
    // State 1: reduce expr -> number on EOF; shift plus -> 2
    actions[1][3] = vec![Action::Reduce(adze_ir::RuleId(0))];
    actions[1][2] = vec![Action::Shift(StateId(2))];
    // State 2: shift number -> 3
    actions[2][1] = vec![Action::Shift(StateId(3))];
    // State 3: reduce expr -> expr '+' expr on EOF and plus
    actions[3][3] = vec![Action::Reduce(adze_ir::RuleId(1))];
    actions[3][2] = vec![Action::Reduce(adze_ir::RuleId(1))];
    // State 4: accept on EOF
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

/// Build a grammar with external scanner tokens.
///
/// Symbol layout:
///   0: ERROR, 1: identifier, 2: EOF, 3: INDENT (ext), 4: DEDENT (ext),
///   5: NEWLINE (ext), 6: block (NT)
fn make_external_scanner_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("indented".to_string());

    let ident_token = Token {
        name: "identifier".to_string(),
        pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), ident_token);

    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(3),
    });
    grammar.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(4),
    });
    grammar.externals.push(ExternalToken {
        name: "NEWLINE".to_string(),
        symbol_id: SymbolId(5),
    });

    // Rule 0: block -> INDENT identifier DEDENT
    let rule = Rule {
        lhs: SymbolId(6),
        rhs: vec![
            Symbol::Terminal(SymbolId(3)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(4)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.rules.insert(SymbolId(6), vec![rule]);
    grammar
        .rule_names
        .insert(SymbolId(6), "block".to_string());

    let eof_symbol = SymbolId(2);
    let start_symbol = SymbolId(6);
    let symbol_count = 7;
    let state_count = 4;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    // State 0: shift INDENT -> 1
    actions[0][3] = vec![Action::Shift(StateId(1))];
    // State 1: shift identifier -> 2
    actions[1][1] = vec![Action::Shift(StateId(2))];
    // State 2: shift DEDENT -> 3
    actions[2][4] = vec![Action::Shift(StateId(3))];
    // State 3: reduce on EOF
    actions[3][2] = vec![Action::Reduce(adze_ir::RuleId(0))];

    let mut gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    gotos[0][6] = StateId(3);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, 6);

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![ParseRule {
            lhs: SymbolId(6),
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
                name: "NEWLINE".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(5),
            },
            SymbolMetadata {
                name: "block".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(6),
            },
        ],
        token_count: 2, // ERROR(0) + identifier(1); EOF at index 2 = token_count + externals(3)... wait
        external_token_count: 3,
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
        external_scanner_states: vec![vec![true; 3]; state_count],
        dynamic_prec_by_rule: vec![0],
        rule_assoc_by_rule: vec![0],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

/// Build a grammar with many states (8 states) for testing larger tables.
///
/// Simulates a chain grammar: S -> A B C D, A -> 'a', B -> 'b', C -> 'c', D -> 'd'
/// Symbol layout:
///   0: ERROR, 1: 'a', 2: 'b', 3: 'c', 4: 'd', 5: EOF,
///   6: S(NT), 7: A(NT), 8: B(NT), 9: C(NT), 10: D(NT)
fn make_many_states_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("chain".to_string());

    let names = ["a", "b", "c", "d"];
    for (i, name) in names.iter().enumerate() {
        grammar.tokens.insert(
            SymbolId((i + 1) as u16),
            Token {
                name: name.to_string(),
                pattern: TokenPattern::String(name.to_string()),
                fragile: false,
            },
        );
    }

    // A -> 'a', B -> 'b', C -> 'c', D -> 'd'
    for i in 0..4u16 {
        let nt = SymbolId(7 + i);
        let term = SymbolId(1 + i);
        grammar.rules.insert(
            nt,
            vec![Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(term)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i),
            }],
        );
        grammar
            .rule_names
            .insert(nt, format!("nt_{}", names[i as usize]));
    }
    // S -> A B C D
    let s_sym = SymbolId(6);
    grammar.rules.insert(
        s_sym,
        vec![Rule {
            lhs: s_sym,
            rhs: vec![
                Symbol::NonTerminal(SymbolId(7)),
                Symbol::NonTerminal(SymbolId(8)),
                Symbol::NonTerminal(SymbolId(9)),
                Symbol::NonTerminal(SymbolId(10)),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(4),
        }],
    );
    grammar
        .rule_names
        .insert(s_sym, "start".to_string());

    let eof_symbol = SymbolId(5);
    let start_symbol = SymbolId(6);
    let symbol_count = 11;
    let state_count = 8;

    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();

    let mut actions = vec![vec![vec![]; symbol_count]; state_count];
    // State 0: shift 'a' -> 1
    actions[0][1] = vec![Action::Shift(StateId(1))];
    // State 1: reduce A -> 'a'
    actions[1][2] = vec![Action::Reduce(adze_ir::RuleId(0))];
    // State 2: shift 'b' -> 3
    actions[2][2] = vec![Action::Shift(StateId(3))];
    // State 3: reduce B -> 'b'
    actions[3][3] = vec![Action::Reduce(adze_ir::RuleId(1))];
    // State 4: shift 'c' -> 5
    actions[4][3] = vec![Action::Shift(StateId(5))];
    // State 5: reduce C -> 'c'
    actions[5][4] = vec![Action::Reduce(adze_ir::RuleId(2))];
    // State 6: shift 'd' -> 7
    actions[6][4] = vec![Action::Shift(StateId(7))];
    // State 7: reduce D -> 'd', accept
    actions[7][5] = vec![Action::Reduce(adze_ir::RuleId(3))];

    let mut gotos = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    gotos[0][7] = StateId(2);  // A
    gotos[2][8] = StateId(4);  // B
    gotos[4][9] = StateId(6);  // C
    gotos[6][10] = StateId(7); // D

    let mut nonterminal_to_index = BTreeMap::new();
    for i in 6..=10 {
        nonterminal_to_index.insert(SymbolId(i as u16), i);
    }

    let mut sym_meta = Vec::new();
    // ERROR
    sym_meta.push(SymbolMetadata {
        name: "ERROR".to_string(),
        is_visible: false,
        is_named: false,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(0),
    });
    // terminals a, b, c, d
    for (i, name) in names.iter().enumerate() {
        sym_meta.push(SymbolMetadata {
            name: name.to_string(),
            is_visible: true,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId((i + 1) as u16),
        });
    }
    // EOF
    sym_meta.push(SymbolMetadata {
        name: "EOF".to_string(),
        is_visible: false,
        is_named: false,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(5),
    });
    // nonterminals S, A, B, C, D
    let nt_names = ["start", "nt_a", "nt_b", "nt_c", "nt_d"];
    for (i, name) in nt_names.iter().enumerate() {
        sym_meta.push(SymbolMetadata {
            name: name.to_string(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId((6 + i) as u16),
        });
    }

    let parse_table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![
            ParseRule { lhs: SymbolId(7), rhs_len: 1 },
            ParseRule { lhs: SymbolId(8), rhs_len: 1 },
            ParseRule { lhs: SymbolId(9), rhs_len: 1 },
            ParseRule { lhs: SymbolId(10), rhs_len: 1 },
            ParseRule { lhs: SymbolId(6), rhs_len: 4 },
        ],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: sym_meta,
        token_count: 5, // ERROR + a + b + c + d
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
        dynamic_prec_by_rule: vec![0; 5],
        rule_assoc_by_rule: vec![0; 5],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, parse_table)
}

// ── 6. Expression grammar: full codegen ──────────────────────────────

#[test]
fn snapshot_expr_grammar_codegen() {
    let (grammar, parse_table) = make_expression_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();
    let formatted = prettyprint_tokens(&code.to_string());
    insta::assert_snapshot!("expr_grammar_codegen", formatted);
}

// ── 7. Expression grammar: NODE_TYPES ────────────────────────────────

#[test]
fn snapshot_expr_node_types_json() {
    let (grammar, parse_table) = make_expression_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let json = generator.generate_node_types();
    insta::assert_snapshot!("expr_node_types_json", json);
}

// ── 8. Expression grammar: symbol metadata ───────────────────────────

#[test]
fn snapshot_expr_symbol_metadata() {
    let (grammar, parse_table) = make_expression_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let metadata = generator.generate_symbol_metadata_public();
    let formatted = metadata
        .iter()
        .enumerate()
        .map(|(i, byte)| format!("[{i}] 0b{byte:08b}"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("expr_symbol_metadata", formatted);
}

// ── 9. Expression grammar: compressed action table ───────────────────

#[test]
fn snapshot_expr_compressed_action_table() {
    let (grammar, parse_table) = make_expression_grammar();
    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
    let result = generator.compress_tables();
    let output = match result {
        Ok(()) => {
            let tables = generator
                .compressed_tables
                .as_ref()
                .expect("compressed_tables should be Some after compress_tables()");
            format_compressed_action_table(&tables.action_table)
        }
        Err(e) => format!("compression error: {e}"),
    };
    insta::assert_snapshot!("expr_compressed_action_table", output);
}

// ── 10. External scanner grammar: codegen ────────────────────────────

#[test]
fn snapshot_external_scanner_codegen() {
    let (grammar, parse_table) = make_external_scanner_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();
    let formatted = prettyprint_tokens(&code.to_string());
    insta::assert_snapshot!("external_scanner_codegen", formatted);
}

// ── 11. External scanner grammar: NODE_TYPES ─────────────────────────

#[test]
fn snapshot_external_scanner_node_types() {
    let (grammar, parse_table) = make_external_scanner_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let json = generator.generate_node_types();
    insta::assert_snapshot!("external_scanner_node_types", json);
}

// ── 12. External scanner grammar: symbol metadata ────────────────────

#[test]
fn snapshot_external_scanner_symbol_metadata() {
    let (grammar, parse_table) = make_external_scanner_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let metadata = generator.generate_symbol_metadata_public();
    let formatted = metadata
        .iter()
        .enumerate()
        .map(|(i, byte)| format!("[{i}] 0b{byte:08b}"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("external_scanner_symbol_metadata", formatted);
}

// ── 13. Many-states grammar: codegen ─────────────────────────────────

#[test]
fn snapshot_many_states_codegen() {
    let (grammar, parse_table) = make_many_states_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();
    let formatted = prettyprint_tokens(&code.to_string());
    insta::assert_snapshot!("many_states_codegen", formatted);
}

// ── 14. Many-states grammar: NODE_TYPES ──────────────────────────────

#[test]
fn snapshot_many_states_node_types() {
    let (grammar, parse_table) = make_many_states_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let json = generator.generate_node_types();
    insta::assert_snapshot!("many_states_node_types", json);
}

// ── 15. Many-states grammar: symbol metadata ─────────────────────────

#[test]
fn snapshot_many_states_symbol_metadata() {
    let (grammar, parse_table) = make_many_states_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let metadata = generator.generate_symbol_metadata_public();
    let formatted = metadata
        .iter()
        .enumerate()
        .map(|(i, byte)| format!("[{i}] 0b{byte:08b}"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("many_states_symbol_metadata", formatted);
}

// ── 16. Many-states grammar: compressed action table ─────────────────

#[test]
fn snapshot_many_states_compressed_action_table() {
    let (grammar, parse_table) = make_many_states_grammar();
    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
    let result = generator.compress_tables();
    let output = match result {
        Ok(()) => {
            let tables = generator
                .compressed_tables
                .as_ref()
                .expect("compressed_tables should be Some after compress_tables()");
            format_compressed_action_table(&tables.action_table)
        }
        Err(e) => format!("compression error: {e}"),
    };
    insta::assert_snapshot!("many_states_compressed_action_table", output);
}

// ── 17. Minimal grammar: ABI constants in generated code ─────────────

#[test]
fn snapshot_abi_constants() {
    let (grammar, parse_table) = make_minimal_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code().to_string();

    // Extract key ABI constants from the generated code
    let mut abi_lines = Vec::new();
    for token in code.split_whitespace() {
        // Capture version, counts, etc. from the LANGUAGE struct
        if token.starts_with("TREE_SITTER_LANGUAGE_VERSION")
            || token.starts_with("EXTERNAL_TOKEN_COUNT")
        {
            // These are `const NAME: TYPE = VALUE;` — already captured via snapshot_language_struct
            continue;
        }
    }

    // Structured extraction of the ABI-relevant numbers
    let extract = |key: &str| -> String {
        // Look for `key : VALUE` patterns in token stream output
        if let Some(pos) = code.find(key) {
            let rest = &code[pos + key.len()..];
            // Skip `:` and whitespace
            let rest = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
            // Take up to next `,` or `}`
            let end = rest
                .find(|c: char| c == ',' || c == '}')
                .unwrap_or(rest.len());
            let val = rest[..end].trim();
            format!("{key} = {val}")
        } else {
            format!("{key} = <not found>")
        }
    };

    let keys = [
        "version",
        "symbol_count",
        "alias_count",
        "token_count",
        "external_token_count",
        "state_count",
        "large_state_count",
        "production_id_count",
        "field_count",
        "max_alias_sequence_length",
    ];
    for key in &keys {
        abi_lines.push(extract(key));
    }

    let output = abi_lines.join("\n");
    insta::assert_snapshot!("abi_constants_minimal", output);
}

// ── 18. Expression grammar: ABI constants ────────────────────────────

#[test]
fn snapshot_expr_abi_constants() {
    let (grammar, parse_table) = make_expression_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code().to_string();

    let extract = |key: &str| -> String {
        if let Some(pos) = code.find(key) {
            let rest = &code[pos + key.len()..];
            let rest = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
            let end = rest
                .find(|c: char| c == ',' || c == '}')
                .unwrap_or(rest.len());
            format!("{key} = {}", rest[..end].trim())
        } else {
            format!("{key} = <not found>")
        }
    };

    let keys = [
        "version",
        "symbol_count",
        "alias_count",
        "token_count",
        "external_token_count",
        "state_count",
        "large_state_count",
        "production_id_count",
        "field_count",
    ];
    let output = keys.iter().map(|k| extract(k)).collect::<Vec<_>>().join("\n");
    insta::assert_snapshot!("abi_constants_expr", output);
}

// ── 19. External scanner grammar: ABI constants ──────────────────────

#[test]
fn snapshot_external_scanner_abi_constants() {
    let (grammar, parse_table) = make_external_scanner_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code().to_string();

    let extract = |key: &str| -> String {
        if let Some(pos) = code.find(key) {
            let rest = &code[pos + key.len()..];
            let rest = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
            let end = rest
                .find(|c: char| c == ',' || c == '}')
                .unwrap_or(rest.len());
            format!("{key} = {}", rest[..end].trim())
        } else {
            format!("{key} = <not found>")
        }
    };

    let keys = [
        "version",
        "symbol_count",
        "token_count",
        "external_token_count",
        "state_count",
    ];
    let output = keys.iter().map(|k| extract(k)).collect::<Vec<_>>().join("\n");
    insta::assert_snapshot!("abi_constants_external_scanner", output);
}

// ── 20. Minimal grammar: primary state IDs ───────────────────────────

#[test]
fn snapshot_primary_state_ids() {
    let (grammar, parse_table) = make_minimal_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code().to_string();

    // Extract PRIMARY_STATE_IDS array from the generated code
    let output = extract_static_array(&code, "PRIMARY_STATE_IDS");
    insta::assert_snapshot!("primary_state_ids_minimal", output);
}

// ── 21. Expression grammar: primary state IDs ────────────────────────

#[test]
fn snapshot_expr_primary_state_ids() {
    let (grammar, parse_table) = make_expression_grammar();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code().to_string();

    let output = extract_static_array(&code, "PRIMARY_STATE_IDS");
    insta::assert_snapshot!("primary_state_ids_expr", output);
}

// ── 22. Expression grammar: production ID count ──────────────────────

#[test]
fn snapshot_expr_production_id_count() {
    let (grammar, parse_table) = make_expression_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let count = generator.count_production_ids_public();
    let output = format!("production_id_count: {count}");
    insta::assert_snapshot!("production_id_count_expr", output);
}

// ── 23. Many-states grammar: production ID count ─────────────────────

#[test]
fn snapshot_many_states_production_id_count() {
    let (grammar, parse_table) = make_many_states_grammar();
    let generator = adze_tablegen::language_gen::LanguageGenerator::new(&grammar, &parse_table);
    let count = generator.count_production_ids_public();
    let output = format!("production_id_count: {count}");
    insta::assert_snapshot!("production_id_count_many_states", output);
}

/// Extract a `static NAME: &[...] = &[...];` array from generated code for snapshotting.
fn extract_static_array(code: &str, name: &str) -> String {
    // Pattern: `static NAME : & [TYPE] = & [ITEMS] ;`
    // In token-stream output the spaces are slightly different, so search flexibly.
    let search = format!("static {name}");
    if let Some(start) = code.find(&search) {
        // Find the `= &[` after the name
        let rest = &code[start..];
        if let Some(eq_pos) = rest.find("= &") {
            let arr_start = &rest[eq_pos..];
            // Find matching `]`
            if let Some(bracket_start) = arr_start.find('[') {
                let inside = &arr_start[bracket_start..];
                if let Some(bracket_end) = inside.find(']') {
                    let array_content = &inside[..=bracket_end];
                    return format!("{name} = &{array_content}");
                }
            }
        }
    }
    format!("{name} = <not found>")
}
