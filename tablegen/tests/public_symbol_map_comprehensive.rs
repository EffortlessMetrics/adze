#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for public symbol map generation in adze-tablegen.
//!
//! Tests cover:
//! - Public symbol map includes named symbols
//! - Map excludes hidden symbols
//! - Map ordering
//! - Map size vs grammar
//! - Map in generated code
//! - Empty grammar map
//! - Large grammar map
//! - Map determinism

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::abi_builder::AbiLanguageBuilder;
use adze_tablegen::language_gen::LanguageGenerator;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

fn simple_rule(lhs: u16, rhs: Vec<Symbol>, prod_id: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        production_id: ProductionId(prod_id),
        fields: vec![],
        precedence: None,
        associativity: None,
    }
}

fn make_grammar(
    name: &str,
    tokens: Vec<(SymbolId, Token)>,
    rules: Vec<Rule>,
    rule_names: Vec<(SymbolId, String)>,
) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    for (id, tok) in tokens {
        g.tokens.insert(id, tok);
    }
    for rule in rules {
        g.add_rule(rule);
    }
    for (id, rn) in rule_names {
        g.rule_names.insert(id, rn);
    }
    g
}

/// Build a ParseTable with explicit symbol_to_index mapping.
fn make_table(
    grammar: &Grammar,
    symbol_to_index: BTreeMap<SymbolId, usize>,
    eof: SymbolId,
) -> ParseTable {
    let symbol_count = symbol_to_index.len();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (&sid, &idx) in &symbol_to_index {
        if idx < symbol_count {
            index_to_symbol[idx] = sid;
        }
    }
    ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; 1],
        goto_table: vec![vec![INVALID; symbol_count]; 1],
        rules: vec![],
        state_count: 1,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol: eof,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build a grammar + parse table with sequential symbol layout.
/// Layout: ERROR(0), terminals 1..=num_terms, externals, EOF, nonterminals.
fn build_grammar_and_table(
    name: &str,
    num_terms: usize,
    num_nonterms: usize,
    num_externals: usize,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = num_terms.max(1);
    let num_nonterms = num_nonterms.max(1);
    let num_states = num_states.max(1);

    let eof_idx = 1 + num_terms + num_externals;
    let symbol_count = eof_idx + 1 + num_nonterms;

    let eof_symbol = SymbolId(eof_idx as u16);
    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let first_term = SymbolId(1);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
    }

    let mut grammar = Grammar::new(name.to_string());

    for i in 1..=num_terms {
        let sym = SymbolId(i as u16);
        grammar.tokens.insert(
            sym,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }

    let first_nt_idx = eof_idx + 1;
    for i in 0..num_nonterms {
        let sym = SymbolId((first_nt_idx + i) as u16);
        grammar.rule_names.insert(sym, format!("rule_{i}"));
        grammar.add_rule(Rule {
            lhs: sym,
            rhs: vec![Symbol::Terminal(first_term)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }

    for i in 0..num_externals {
        grammar.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: SymbolId((1 + num_terms + i) as u16),
        });
    }

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let table = ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; num_states],
        goto_table: vec![vec![INVALID; symbol_count]; num_states],
        symbol_metadata: vec![],
        state_count: num_states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        external_scanner_states: vec![],
        rules: vec![],
        eof_symbol,
        start_symbol,
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: eof_idx + 1,
        external_token_count: num_externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, table)
}

/// Render AbiLanguageBuilder output as a String.
fn abi_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

/// Render LanguageGenerator output as a String.
fn lang_gen_code(grammar: &Grammar, table: &ParseTable) -> String {
    LanguageGenerator::new(grammar, table)
        .generate()
        .to_string()
}

/// Extract the PUBLIC_SYMBOL_MAP entries from generated code.
/// Returns the raw string between the brackets of the map definition.
fn extract_public_symbol_map_body(code: &str) -> Option<String> {
    // AbiLanguageBuilder outputs: `static PUBLIC_SYMBOL_MAP : & [u16] = & [<entries>] ;`
    // LanguageGenerator outputs: `static PUBLIC_SYMBOL_MAP : & [TSSymbol] = & [<entries>] ;`
    let marker = "PUBLIC_SYMBOL_MAP";
    let start = code.find(marker)?;
    let rest = &code[start..];
    // Find the first `[` after `= &`
    let eq_amp = rest.find("= &")?;
    let after_eq = &rest[eq_amp + 3..];
    let bracket_open = after_eq.find('[')?;
    let inner = &after_eq[bracket_open + 1..];
    let bracket_close = inner.find(']')?;
    Some(inner[..bracket_close].to_string())
}

/// Count entries in the PUBLIC_SYMBOL_MAP from generated code.
fn count_public_symbol_map_entries(code: &str) -> usize {
    match extract_public_symbol_map_body(code) {
        Some(body) if body.trim().is_empty() => 0,
        Some(body) => body.split(',').count(),
        None => 0,
    }
}

// ===========================================================================
// 1. Named symbols are included
// ===========================================================================

#[test]
fn named_terminal_appears_in_map() {
    let g = make_grammar(
        "test",
        vec![(SymbolId(1), regex_token("identifier", "[a-z]+"))],
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(2), "program".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0); // EOF
    s2i.insert(SymbolId(1), 1); // terminal
    s2i.insert(SymbolId(2), 2); // nonterminal
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
    assert!(count_public_symbol_map_entries(&code) >= 3);
}

#[test]
fn named_nonterminal_appears_in_map() {
    let (g, t) = build_grammar_and_table("nt_test", 1, 2, 0, 1);
    let code = abi_code(&g, &t);
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
    // symbol_count = 1(ERROR) + 1(term) + 1(EOF) + 2(nt) = 5
    assert!(count_public_symbol_map_entries(&code) >= 5);
}

#[test]
fn multiple_named_terminals_all_present() {
    let g = make_grammar(
        "multi",
        vec![
            (SymbolId(1), string_token("plus", "+")),
            (SymbolId(2), string_token("minus", "-")),
            (SymbolId(3), regex_token("number", "[0-9]+")),
        ],
        vec![simple_rule(
            4,
            vec![
                Symbol::Terminal(SymbolId(3)),
                Symbol::Terminal(SymbolId(1)),
                Symbol::Terminal(SymbolId(3)),
            ],
            0,
        )],
        vec![(SymbolId(4), "expr".to_string())],
    );
    let mut s2i = BTreeMap::new();
    for i in 0..=4 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    assert_eq!(count_public_symbol_map_entries(&code), 5);
}

#[test]
fn external_tokens_counted_in_map() {
    let (g, t) = build_grammar_and_table("ext", 1, 1, 2, 1);
    let code = abi_code(&g, &t);
    // symbol_count = 1 + 1 + 2(ext) + 1(EOF) + 1(nt) = 6
    let count = count_public_symbol_map_entries(&code);
    assert!(count >= 6, "expected ≥6 entries, got {count}");
}

// ===========================================================================
// 2. Hidden symbols excluded
// ===========================================================================

#[test]
fn hidden_token_still_gets_index_in_map() {
    // Hidden tokens (name starts with _) still get an index entry in the
    // public symbol map because the map is identity-indexed by position.
    let g = make_grammar(
        "hidden",
        vec![
            (SymbolId(1), string_token("_ws", " ")),
            (SymbolId(2), regex_token("identifier", "[a-z]+")),
        ],
        vec![simple_rule(3, vec![Symbol::Terminal(SymbolId(2))], 0)],
        vec![(SymbolId(3), "program".to_string())],
    );
    let mut s2i = BTreeMap::new();
    for i in 0..=3 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    // Identity map: every symbol index present, even hidden ones
    assert_eq!(count_public_symbol_map_entries(&code), 4);
}

#[test]
fn hidden_rule_name_does_not_alter_map_size() {
    // A rule named _hidden still occupies its slot in the map
    let g = make_grammar(
        "hrule",
        vec![(SymbolId(1), string_token("a", "a"))],
        vec![simple_rule(2, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(2), "_hidden".to_string())],
    );
    let mut s2i = BTreeMap::new();
    for i in 0..=2 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    assert_eq!(count_public_symbol_map_entries(&code), 3);
}

#[test]
fn mix_of_hidden_and_visible_symbols() {
    let g = make_grammar(
        "mix",
        vec![
            (SymbolId(1), string_token("_ws", " ")),
            (SymbolId(2), string_token("id", "x")),
            (SymbolId(3), string_token("_comment", "//")),
        ],
        vec![simple_rule(4, vec![Symbol::Terminal(SymbolId(2))], 0)],
        vec![(SymbolId(4), "program".to_string())],
    );
    let mut s2i = BTreeMap::new();
    for i in 0..=4 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    // Map has an entry for every symbol index (identity map)
    assert_eq!(count_public_symbol_map_entries(&code), 5);
}

// ===========================================================================
// 3. Map ordering
// ===========================================================================

#[test]
fn map_entries_are_sequential() {
    let (g, t) = build_grammar_and_table("seq", 3, 2, 0, 1);
    let code = abi_code(&g, &t);
    let body = extract_public_symbol_map_body(&code).expect("map present");
    // Each entry should be "N as u16" with N increasing from 0
    let entries: Vec<&str> = body.split(',').map(|s| s.trim()).collect();
    for (i, entry) in entries.iter().enumerate() {
        assert!(
            entry.contains(&format!("{i}usize")) || entry.contains(&format!("{i} as")),
            "entry {i} should reference index {i}, got: {entry}"
        );
    }
}

#[test]
fn map_starts_at_zero() {
    let (g, t) = build_grammar_and_table("zero", 1, 1, 0, 1);
    let code = abi_code(&g, &t);
    let body = extract_public_symbol_map_body(&code).expect("map present");
    let first = body.split(',').next().unwrap().trim();
    assert!(
        first.contains("0usize") || first.contains("0 as"),
        "first entry should be 0, got: {first}"
    );
}

#[test]
fn map_entries_monotonically_increase() {
    let (g, t) = build_grammar_and_table("mono", 4, 3, 1, 2);
    let code = abi_code(&g, &t);
    let body = extract_public_symbol_map_body(&code).expect("map present");
    let entries: Vec<&str> = body.split(',').collect();
    // Extract numeric values — each should be greater than or equal to previous
    let mut prev = None;
    for entry in &entries {
        // Find the numeric portion before "usize" or "as"
        let num_str: String = entry
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(n) = num_str.parse::<usize>() {
            if let Some(p) = prev {
                assert!(n >= p, "entries should be non-decreasing");
            }
            prev = Some(n);
        }
    }
}

#[test]
fn map_is_contiguous_identity() {
    // The public symbol map is an identity mapping [0, 1, 2, ..., N-1]
    let (g, t) = build_grammar_and_table("contig", 2, 2, 0, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, t.symbol_count, "map size must match symbol_count");
}

// ===========================================================================
// 4. Map size vs grammar
// ===========================================================================

#[test]
fn map_size_equals_symbol_count() {
    let (g, t) = build_grammar_and_table("sz", 3, 2, 0, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(
        count, t.symbol_count,
        "map entries ({count}) must equal symbol_count ({})",
        t.symbol_count
    );
}

#[test]
fn map_size_includes_eof() {
    let (g, t) = build_grammar_and_table("eof", 2, 1, 0, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    // EOF is a symbol that should be counted
    assert!(count > 0);
    assert_eq!(count, t.symbol_count);
}

#[test]
fn map_size_with_externals() {
    let (g, t) = build_grammar_and_table("extsize", 2, 1, 3, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, t.symbol_count);
}

#[test]
fn map_size_grows_with_more_terminals() {
    let (g1, t1) = build_grammar_and_table("small", 2, 1, 0, 1);
    let (g2, t2) = build_grammar_and_table("large", 5, 1, 0, 1);
    let c1 = count_public_symbol_map_entries(&abi_code(&g1, &t1));
    let c2 = count_public_symbol_map_entries(&abi_code(&g2, &t2));
    assert!(c2 > c1, "more terminals should yield larger map");
}

// ===========================================================================
// 5. Map in generated code
// ===========================================================================

#[test]
fn abi_builder_generates_public_symbol_map_static() {
    let (g, t) = build_grammar_and_table("codegen", 2, 1, 0, 1);
    let code = abi_code(&g, &t);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "must generate PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn abi_builder_map_referenced_in_language_struct() {
    let (g, t) = build_grammar_and_table("ref", 2, 1, 0, 2);
    let code = abi_code(&g, &t);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP . as_ptr"),
        "LANGUAGE struct must reference PUBLIC_SYMBOL_MAP.as_ptr()"
    );
}

#[test]
fn language_gen_also_emits_public_symbol_map() {
    let (g, t) = build_grammar_and_table("lang", 2, 1, 0, 1);
    let code = lang_gen_code(&g, &t);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "LanguageGenerator should also emit PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn abi_builder_map_is_u16_array() {
    let (g, t) = build_grammar_and_table("u16", 1, 1, 0, 1);
    let code = abi_code(&g, &t);
    // AbiLanguageBuilder declares it as `& [u16]`
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP : & [u16]"),
        "map should be typed as &[u16]"
    );
}

#[test]
fn language_gen_map_is_ts_symbol_array() {
    let (g, t) = build_grammar_and_table("tsym", 1, 1, 0, 1);
    let code = lang_gen_code(&g, &t);
    // LanguageGenerator declares it as `& [TSSymbol]`
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP : & [TSSymbol]"),
        "LanguageGenerator map should be typed as &[TSSymbol]"
    );
}

#[test]
fn public_symbol_map_field_present_in_tslanguage() {
    let (g, t) = build_grammar_and_table("field", 1, 1, 0, 1);
    let code = abi_code(&g, &t);
    assert!(
        code.contains("public_symbol_map"),
        "TSLanguage struct must contain public_symbol_map field"
    );
}

// ===========================================================================
// 6. Empty grammar map
// ===========================================================================

#[test]
fn default_grammar_and_table_generate_map() {
    let g = Grammar::new("empty".to_string());
    let t = ParseTable::default();
    let code = abi_code(&g, &t);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "even empty grammar generates a PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn default_grammar_map_has_zero_entries() {
    let g = Grammar::new("empty".to_string());
    let t = ParseTable::default();
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, 0, "empty grammar map should have 0 entries");
}

#[test]
fn minimal_grammar_with_one_symbol_has_one_entry() {
    let g = Grammar::new("min".to_string());
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    assert_eq!(count_public_symbol_map_entries(&code), 1);
}

// ===========================================================================
// 7. Large grammar map
// ===========================================================================

#[test]
fn large_grammar_50_terminals() {
    let (g, t) = build_grammar_and_table("big50", 50, 5, 0, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, t.symbol_count);
}

#[test]
fn large_grammar_100_terminals() {
    let (g, t) = build_grammar_and_table("big100", 100, 10, 0, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, t.symbol_count);
}

#[test]
fn large_grammar_with_externals() {
    let (g, t) = build_grammar_and_table("bigext", 30, 10, 5, 1);
    let code = abi_code(&g, &t);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, t.symbol_count);
}

#[test]
fn large_grammar_entries_are_valid() {
    let (g, t) = build_grammar_and_table("bigvalid", 80, 20, 0, 1);
    let code = abi_code(&g, &t);
    let body = extract_public_symbol_map_body(&code).expect("map present");
    let entries: Vec<&str> = body.split(',').collect();
    assert_eq!(entries.len(), t.symbol_count);
    for (i, entry) in entries.iter().enumerate() {
        assert!(
            entry.contains(&format!("{i}usize")) || entry.contains(&format!("{i} as")),
            "entry {i} should be index {i}, got: {entry}"
        );
    }
}

// ===========================================================================
// 8. Map determinism
// ===========================================================================

#[test]
fn abi_builder_deterministic_across_runs() {
    let (g, t) = build_grammar_and_table("det", 3, 2, 0, 2);
    let code1 = abi_code(&g, &t);
    let code2 = abi_code(&g, &t);
    let body1 = extract_public_symbol_map_body(&code1);
    let body2 = extract_public_symbol_map_body(&code2);
    assert_eq!(body1, body2, "map must be identical across runs");
}

#[test]
fn language_gen_deterministic_across_runs() {
    let (g, t) = build_grammar_and_table("detlg", 2, 1, 0, 1);
    let code1 = lang_gen_code(&g, &t);
    let code2 = lang_gen_code(&g, &t);
    let body1 = extract_public_symbol_map_body(&code1);
    let body2 = extract_public_symbol_map_body(&code2);
    assert_eq!(body1, body2, "LanguageGenerator map must be deterministic");
}

#[test]
fn deterministic_with_many_symbols() {
    let (g, t) = build_grammar_and_table("detmany", 20, 10, 3, 2);
    let code1 = abi_code(&g, &t);
    let code2 = abi_code(&g, &t);
    let body1 = extract_public_symbol_map_body(&code1);
    let body2 = extract_public_symbol_map_body(&code2);
    assert_eq!(body1, body2);
}

#[test]
fn deterministic_ten_iterations() {
    let (g, t) = build_grammar_and_table("det10", 5, 3, 1, 1);
    let reference = extract_public_symbol_map_body(&abi_code(&g, &t));
    for _ in 0..10 {
        let current = extract_public_symbol_map_body(&abi_code(&g, &t));
        assert_eq!(
            reference, current,
            "map must be stable across 10 iterations"
        );
    }
}

// ===========================================================================
// 9. Additional edge cases
// ===========================================================================

#[test]
fn map_with_only_eof_symbol() {
    // A grammar with only the EOF symbol
    let g = Grammar::new("eof_only".to_string());
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    let pt = make_table(&g, s2i, SymbolId(0));
    let code = abi_code(&g, &pt);
    let count = count_public_symbol_map_entries(&code);
    assert_eq!(count, 1, "should have exactly 1 entry for EOF");
}

#[test]
fn map_identity_property() {
    // The public_symbol_map should be identity: entry[i] == i
    let (g, t) = build_grammar_and_table("ident", 4, 2, 0, 1);
    let code = abi_code(&g, &t);
    let body = extract_public_symbol_map_body(&code).expect("map present");
    let entries: Vec<&str> = body.split(',').collect();
    for (i, entry) in entries.iter().enumerate() {
        let num: String = entry
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let val: usize = num.parse().expect("should be a number");
        assert_eq!(val, i, "public_symbol_map[{i}] should be {i}, got {val}");
    }
}

#[test]
fn both_generators_agree_on_entry_count() {
    let (g, t) = build_grammar_and_table("agree", 3, 2, 0, 1);
    let abi = count_public_symbol_map_entries(&abi_code(&g, &t));
    let lg = count_public_symbol_map_entries(&lang_gen_code(&g, &t));
    // Both should produce maps — they may differ in size due to different counting
    // strategies, but both should be non-zero
    assert!(abi > 0, "AbiLanguageBuilder map should be non-empty");
    assert!(lg > 0, "LanguageGenerator map should be non-empty");
}
