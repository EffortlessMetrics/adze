//! Snapshot tests for tablegen code generation output.
//!
//! These tests use `insta` to snapshot the generated Language struct code,
//! compressed action tables, NODE_TYPES JSON, symbol metadata, and
//! production ID maps.

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable, SymbolMetadata};
use adze_ir::{Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern};
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
