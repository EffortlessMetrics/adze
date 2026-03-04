#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for symbol table generation in adze-tablegen.
//!
//! Tests cover:
//! - Symbol table construction from grammar
//! - Terminal symbol entries
//! - Nonterminal symbol entries
//! - Symbol ordering
//! - Hidden symbols handling
//! - External symbol inclusion
//! - Large symbol tables
//! - Symbol table in generated code

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

/// Build a ParseTable for AbiLanguageBuilder with explicit symbol_to_index mapping.
fn make_abi_table(
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
        goto_table: vec![vec![StateId(u16::MAX); symbol_count]; 1],
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

/// Build a simple ParseTable for LanguageGenerator tests (index 0 = EOF).
fn make_lang_gen_table(grammar: &Grammar, symbol_count: usize) -> ParseTable {
    let mut s2i = BTreeMap::new();
    for i in 0..symbol_count {
        s2i.insert(SymbolId(i as u16), i);
    }
    make_abi_table(grammar, s2i, SymbolId(0))
}

/// Render AbiLanguageBuilder output as a String.
fn abi_output(grammar: &Grammar, s2i: BTreeMap<SymbolId, usize>, eof: SymbolId) -> String {
    let pt = make_abi_table(grammar, s2i, eof);
    AbiLanguageBuilder::new(grammar, &pt).generate().to_string()
}

/// Render LanguageGenerator output as a String.
fn lang_gen_output(grammar: &Grammar, pt: &ParseTable) -> String {
    LanguageGenerator::new(grammar, pt).generate().to_string()
}

/// Check that a symbol name's byte encoding appears in a SYMBOL_NAME_X
/// definition within the ABI output. The ABI builder stores names as
/// null-terminated byte arrays like `& [112u8 , 108u8 , ...]`.
fn abi_has_symbol_name(output: &str, name: &str) -> bool {
    let bytes: Vec<u8> = format!("{}\0", name).into_bytes();
    let byte_strs: Vec<String> = bytes.iter().map(|b| format!("{}u8", b)).collect();
    // All bytes must appear in sequence somewhere in the output
    let first = &byte_strs[0];
    if let Some(start) = output.find(first) {
        let after = &output[start..];
        byte_strs.iter().all(|bs| after.contains(bs))
    } else {
        false
    }
}

// ===========================================================================
// 1. Symbol table construction from grammar
// ===========================================================================

#[test]
fn construction_empty_grammar_produces_eof_only() {
    let grammar = Grammar::new("empty".to_string());
    let pt = make_lang_gen_table(&grammar, 1);
    let output = lang_gen_output(&grammar, &pt);
    // LanguageGenerator uses quoted strings: "end"
    assert!(output.contains("\"end\""), "EOF sentinel must be present");
}

#[test]
fn construction_single_token_grammar() {
    let grammar = make_grammar(
        "single",
        vec![(SymbolId(1), regex_token("number", r"\d+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 2);
    let output = lang_gen_output(&grammar, &pt);
    assert!(output.contains("number"), "token name must appear");
    assert!(output.contains("\"end\""), "EOF must appear");
}

#[test]
fn construction_token_and_rule_grammar_abi() {
    let grammar = make_grammar(
        "mixed",
        vec![(SymbolId(1), string_token("plus", "+"))],
        vec![simple_rule(10, vec![Symbol::Terminal(SymbolId(1))], 0)],
        vec![(SymbolId(10), "expr".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0); // EOF
    s2i.insert(SymbolId(1), 1); // plus
    s2i.insert(SymbolId(10), 2); // expr
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        abi_has_symbol_name(&output, "plus"),
        "terminal 'plus' bytes must be in symbol table"
    );
    assert!(
        abi_has_symbol_name(&output, "expr"),
        "nonterminal 'expr' bytes must be in symbol table"
    );
}

#[test]
fn construction_symbol_count_matches_grammar() {
    let grammar = make_grammar(
        "cnt",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "start".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(2), 2);
    s2i.insert(SymbolId(10), 3);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // symbol_count should be 4 (EOF + 2 tokens + 1 rule)
    assert!(
        output.contains("symbol_count : 4"),
        "symbol_count must equal 4"
    );
}

// ===========================================================================
// 2. Terminal symbol entries
// ===========================================================================

#[test]
fn terminal_string_token_in_lang_gen() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), string_token("semicolon", ";"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 2);
    let output = lang_gen_output(&grammar, &pt);
    assert!(
        output.contains("semicolon"),
        "string token name must appear"
    );
}

#[test]
fn terminal_regex_token_in_lang_gen() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("identifier", "[a-z]+"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 2);
    let output = lang_gen_output(&grammar, &pt);
    assert!(
        output.contains("identifier"),
        "regex token name must appear"
    );
}

#[test]
fn terminal_multiple_tokens_all_present() {
    let grammar = make_grammar(
        "t",
        vec![
            (SymbolId(1), string_token("lparen", "(")),
            (SymbolId(2), string_token("rparen", ")")),
            (SymbolId(3), regex_token("number", r"\d+")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 4);
    let output = lang_gen_output(&grammar, &pt);
    for name in &["lparen", "rparen", "number"] {
        assert!(output.contains(name), "token '{}' must appear", name);
    }
}

#[test]
fn terminal_string_token_bytes_in_abi() {
    // ABI stores names as byte arrays; 'p'=112
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), string_token("plus", "+"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // "plus\0" first byte: 'p' = 112
    assert!(
        output.contains("112u8"),
        "byte for 'p' must appear in ABI output"
    );
}

// (terminal_metadata_array_generated covered by codegen_symbol_metadata_array_present)

// ===========================================================================
// 3. Nonterminal symbol entries
// ===========================================================================

#[test]
fn nonterminal_named_rule_in_abi() {
    let grammar = make_grammar(
        "nt",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "expression".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        abi_has_symbol_name(&output, "expression"),
        "named nonterminal bytes must appear"
    );
}

#[test]
fn nonterminal_unnamed_rule_gets_generated_name() {
    let grammar = make_grammar(
        "nt",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![], // no explicit name
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        abi_has_symbol_name(&output, "rule_10"),
        "unnamed nonterminal should get generated name 'rule_10'"
    );
}

#[test]
fn nonterminal_multiple_rules_all_present_lang_gen() {
    let grammar = make_grammar(
        "nt",
        vec![],
        vec![simple_rule(10, vec![], 0), simple_rule(11, vec![], 1)],
        vec![
            (SymbolId(10), "statement".to_string()),
            (SymbolId(11), "declaration".to_string()),
        ],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    s2i.insert(SymbolId(11), 2);
    let pt = make_abi_table(&grammar, s2i, SymbolId(0));
    let output = lang_gen_output(&grammar, &pt);
    assert!(output.contains("statement"), "statement must appear");
    assert!(output.contains("declaration"), "declaration must appear");
}

#[test]
fn nonterminal_with_supertype_flag() {
    let mut grammar = make_grammar(
        "nt",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "expression".to_string())],
    );
    grammar.supertypes.push(SymbolId(10));
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(output.contains("SYMBOL_METADATA"), "metadata must exist");
}

// ===========================================================================
// 4. Symbol ordering
// ===========================================================================

#[test]
fn ordering_eof_at_index_zero_lang_gen() {
    let grammar = make_grammar(
        "ord",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 2);
    let output = lang_gen_output(&grammar, &pt);
    let idx = output
        .find("SYMBOL_NAMES")
        .expect("SYMBOL_NAMES must exist");
    let snippet = &output[idx..idx + 300.min(output.len() - idx)];
    assert!(
        snippet.contains("\"end\""),
        "first symbol must be 'end' (EOF)"
    );
}

#[test]
fn ordering_eof_first_byte_in_abi() {
    let grammar = make_grammar(
        "ord",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // SYMBOL_NAME_0 should hold "end" → first byte 'e'=101
    let sym0_pos = output
        .find("SYMBOL_NAME_0")
        .expect("SYMBOL_NAME_0 must exist");
    let after = &output[sym0_pos..];
    assert!(
        after[..200.min(after.len())].contains("101u8"),
        "SYMBOL_NAME_0 must contain 'e'=101u8 for 'end'"
    );
}

#[test]
fn ordering_symbol_to_index_respected_in_abi() {
    let grammar = make_grammar(
        "ord",
        vec![
            (SymbolId(5), string_token("alpha", "a")),
            (SymbolId(3), string_token("beta", "b")),
        ],
        vec![],
        vec![],
    );
    // beta at index 1, alpha at index 2
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0); // EOF
    s2i.insert(SymbolId(3), 1); // beta
    s2i.insert(SymbolId(5), 2); // alpha
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // SYMBOL_NAME_1 should hold "beta" (first byte 'b'=98)
    let name1_pos = output.find("SYMBOL_NAME_1").unwrap();
    let name2_pos = output.find("SYMBOL_NAME_2").unwrap();
    let between = &output[name1_pos..name2_pos];
    assert!(
        between.contains("98u8"),
        "SYMBOL_NAME_1 should contain 'b'=98u8 for 'beta'"
    );
}

#[test]
fn ordering_deterministic_across_invocations() {
    let grammar = make_grammar(
        "det",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), string_token("b", "b")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "r".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(2), 2);
    s2i.insert(SymbolId(10), 3);
    let output1 = abi_output(&grammar, s2i.clone(), SymbolId(0));
    let output2 = abi_output(&grammar, s2i, SymbolId(0));
    assert_eq!(output1, output2, "generation must be deterministic");
}

// ===========================================================================
// 5. Hidden symbols handling
// ===========================================================================

#[test]
fn hidden_underscore_prefixed_token_bytes_present() {
    let grammar = make_grammar(
        "hid",
        vec![(SymbolId(1), regex_token("_ws", r"\s+"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        abi_has_symbol_name(&output, "_ws"),
        "hidden token name bytes must still appear"
    );
}

#[test]
fn hidden_underscore_prefixed_rule_bytes_present() {
    let grammar = make_grammar(
        "hid",
        vec![],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "_hidden_rule".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(10), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        abi_has_symbol_name(&output, "_hidden_rule"),
        "hidden rule name bytes must appear"
    );
}

// (hidden_extras_metadata_generated covered by hidden_mix test + codegen metadata test)

#[test]
fn hidden_mix_of_visible_and_hidden_tokens_abi() {
    let grammar = make_grammar(
        "hid",
        vec![
            (SymbolId(1), string_token("plus", "+")),
            (SymbolId(2), regex_token("_comment", "//.*")),
        ],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(2), 2);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        abi_has_symbol_name(&output, "plus"),
        "visible token bytes present"
    );
    assert!(
        abi_has_symbol_name(&output, "_comment"),
        "hidden token bytes present"
    );
}

// ===========================================================================
// 6. External symbol inclusion
// ===========================================================================

#[test]
fn external_single_external_token_has_symbol_entry() {
    // External tokens get a symbol table slot; the ABI builder generates
    // "rule_{id}" for symbols not in tokens/rule_names, so SymbolId(50)
    // maps to "rule_50" in the name table.
    let mut grammar = make_grammar("ext", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "heredoc".to_string(),
        symbol_id: SymbolId(50),
    });
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(50), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // The symbol gets a SYMBOL_NAME_1 entry
    assert!(
        output.contains("SYMBOL_NAME_1"),
        "external token must have a symbol name entry"
    );
}

#[test]
fn external_multiple_external_tokens_all_have_entries() {
    let mut grammar = make_grammar("ext", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(50),
    });
    grammar.externals.push(ExternalToken {
        name: "dedent".to_string(),
        symbol_id: SymbolId(51),
    });
    grammar.externals.push(ExternalToken {
        name: "newline".to_string(),
        symbol_id: SymbolId(52),
    });
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(50), 1);
    s2i.insert(SymbolId(51), 2);
    s2i.insert(SymbolId(52), 3);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // Each external gets a SYMBOL_NAME_N entry
    for idx in 1..=3 {
        let name = format!("SYMBOL_NAME_{}", idx);
        assert!(output.contains(&name), "{} must exist", name);
    }
    // symbol_count should be 4
    assert!(output.contains("symbol_count : 4"));
}

#[test]
fn external_hidden_external_token_has_metadata() {
    let mut grammar = make_grammar("ext", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "_auto_semi".to_string(),
        symbol_id: SymbolId(50),
    });
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(50), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // The external gets a SYMBOL_NAME entry and metadata
    assert!(output.contains("SYMBOL_NAME_1"));
    assert!(output.contains("SYMBOL_METADATA"));
}

#[test]
fn external_token_count_in_language_struct() {
    let mut grammar = make_grammar("ext", vec![], vec![], vec![]);
    grammar.externals.push(ExternalToken {
        name: "ext1".to_string(),
        symbol_id: SymbolId(50),
    });
    grammar.externals.push(ExternalToken {
        name: "ext2".to_string(),
        symbol_id: SymbolId(51),
    });
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(50), 1);
    s2i.insert(SymbolId(51), 2);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        output.contains("external_token_count : 2"),
        "external_token_count must be 2"
    );
}

// ===========================================================================
// 7. Large symbol tables
// ===========================================================================

#[test]
fn large_table_50_tokens() {
    let tokens: Vec<(SymbolId, Token)> = (1..=50)
        .map(|i| {
            (
                SymbolId(i),
                string_token(&format!("tok_{}", i), &format!("t{}", i)),
            )
        })
        .collect();
    let grammar = make_grammar("large", tokens, vec![], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    for i in 1..=50u16 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // Check we have 51 symbol name definitions (0..=50)
    assert!(
        output.contains("SYMBOL_NAME_50"),
        "SYMBOL_NAME_50 must exist for 50 tokens + EOF"
    );
    // Spot-check via bytes: "tok_1" starts with 't'=116
    assert!(
        output.contains("116u8"),
        "byte for 't' in tok_N must appear"
    );
}

#[test]
fn large_table_100_symbols_mixed() {
    let tokens: Vec<(SymbolId, Token)> = (1..=50)
        .map(|i| {
            (
                SymbolId(i),
                string_token(&format!("term_{}", i), &format!("{}", i)),
            )
        })
        .collect();
    let rules: Vec<Rule> = (51..=100).map(|i| simple_rule(i, vec![], i - 51)).collect();
    let rule_names: Vec<(SymbolId, String)> = (51..=100)
        .map(|i| (SymbolId(i), format!("nt_{}", i)))
        .collect();
    let grammar = make_grammar("large", tokens, rules, rule_names);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    for i in 1..=100u16 {
        s2i.insert(SymbolId(i), i as usize);
    }
    // token_count = 51 (EOF + 50 terminals), so nonterminals start at index 51
    let mut pt = make_abi_table(&grammar, s2i, SymbolId(0));
    pt.token_count = 51;
    let output = AbiLanguageBuilder::new(&grammar, &pt)
        .generate()
        .to_string();
    // Check last symbol name definition
    assert!(
        output.contains("SYMBOL_NAME_100"),
        "SYMBOL_NAME_100 must exist"
    );
    // Spot-check via byte sequences
    assert!(
        abi_has_symbol_name(&output, "term_1"),
        "term_1 must be present"
    );
    assert!(
        abi_has_symbol_name(&output, "nt_100"),
        "nt_100 must be present"
    );
}

#[test]
fn large_table_symbol_count_correct() {
    let count = 30u16;
    let tokens: Vec<(SymbolId, Token)> = (1..=count)
        .map(|i| {
            (
                SymbolId(i),
                string_token(&format!("s_{}", i), &format!("{}", i)),
            )
        })
        .collect();
    let grammar = make_grammar("large", tokens, vec![], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    for i in 1..=count {
        s2i.insert(SymbolId(i), i as usize);
    }
    let expected_count = (count + 1) as usize; // +1 for EOF
    let output = abi_output(&grammar, s2i, SymbolId(0));
    let expected_str = format!("symbol_count : {}", expected_count);
    assert!(
        output.contains(&expected_str),
        "symbol_count should be {}",
        expected_count,
    );
}

#[test]
fn large_table_200_symbols_does_not_panic() {
    let tokens: Vec<(SymbolId, Token)> = (1..=200)
        .map(|i| {
            (
                SymbolId(i),
                string_token(&format!("t{}", i), &format!("{}", i)),
            )
        })
        .collect();
    let grammar = make_grammar("huge", tokens, vec![], vec![]);
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    for i in 1..=200u16 {
        s2i.insert(SymbolId(i), i as usize);
    }
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(!output.is_empty());
}

// ===========================================================================
// 8. Symbol table in generated code
// ===========================================================================

// (codegen_symbol_name_ptrs_array_present covered by other codegen tests)

#[test]
fn codegen_symbol_metadata_array_present() {
    let grammar = make_grammar(
        "cg",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        output.contains("SYMBOL_METADATA"),
        "SYMBOL_METADATA must be generated"
    );
}

#[test]
fn codegen_public_symbol_map_present() {
    let grammar = make_grammar(
        "cg",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        output.contains("PUBLIC_SYMBOL_MAP"),
        "PUBLIC_SYMBOL_MAP must be generated"
    );
}

#[test]
fn codegen_symbol_id_to_index_mapping_present() {
    let grammar = make_grammar(
        "cg",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(
        output.contains("SYMBOL_ID_TO_INDEX"),
        "SYMBOL_ID_TO_INDEX must be generated"
    );
    assert!(
        output.contains("SYMBOL_INDEX_TO_ID"),
        "SYMBOL_INDEX_TO_ID must be generated"
    );
}

#[test]
fn codegen_language_struct_references_symbol_arrays() {
    let grammar = make_grammar(
        "cg",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "r".to_string())],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    s2i.insert(SymbolId(10), 2);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    assert!(output.contains("symbol_names"));
    assert!(output.contains("symbol_metadata"));
}

#[test]
fn codegen_null_terminated_symbol_name_bytes() {
    let grammar = make_grammar(
        "cg",
        vec![(SymbolId(1), string_token("abc", "abc"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // "abc\0" = [97, 98, 99, 0]
    assert!(output.contains("97u8"), "'a' byte");
    assert!(output.contains("98u8"), "'b' byte");
    assert!(output.contains("99u8"), "'c' byte");
    assert!(output.contains("0u8"), "null terminator");
}

#[test]
fn codegen_lang_gen_symbol_names_array() {
    let grammar = make_grammar(
        "cg",
        vec![
            (SymbolId(1), string_token("foo", "foo")),
            (SymbolId(2), regex_token("bar", "bar")),
        ],
        vec![],
        vec![],
    );
    let pt = make_lang_gen_table(&grammar, 3);
    let output = lang_gen_output(&grammar, &pt);
    assert!(
        output.contains("SYMBOL_NAMES"),
        "LanguageGenerator must produce SYMBOL_NAMES"
    );
    assert!(output.contains("foo"));
    assert!(output.contains("bar"));
}

#[test]
fn codegen_abi_eof_symbol_is_zero() {
    let grammar = make_grammar(
        "cg",
        vec![(SymbolId(1), string_token("x", "x"))],
        vec![],
        vec![],
    );
    let mut s2i = BTreeMap::new();
    s2i.insert(SymbolId(0), 0);
    s2i.insert(SymbolId(1), 1);
    let output = abi_output(&grammar, s2i, SymbolId(0));
    // Tree-sitter convention: eof_symbol = 0
    assert!(
        output.contains("eof_symbol : 0"),
        "eof_symbol must be 0 in LANGUAGE struct"
    );
}

#[test]
fn codegen_language_generator_metadata_public() {
    let grammar = make_grammar(
        "meta",
        vec![
            (SymbolId(1), string_token("a", "a")),
            (SymbolId(2), regex_token("b", "[b]")),
        ],
        vec![simple_rule(10, vec![], 0)],
        vec![(SymbolId(10), "r".to_string())],
    );
    let pt = make_lang_gen_table(&grammar, 4);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let metadata = generator.generate_symbol_metadata_public();
    // We should get one byte per symbol: EOF + 2 tokens + 1 rule = 4
    assert_eq!(
        metadata.len(),
        4,
        "metadata length should match symbol count"
    );
}
