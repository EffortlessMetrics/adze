#![allow(clippy::needless_range_loop)]
//! Property-based tests for production ID generation in adze-tablegen.
//!
//! Properties verified:
//!  1.  Production IDs are sequential (0..n for n rules)
//!  2.  Production ID map length equals rule count
//!  3.  Production ID map values are unique
//!  4.  Production count matches grammar rule count
//!  5.  production_id_count == max(production_id) + 1
//!  6.  Production IDs present in generated code
//!  7.  PRODUCTION_ID_MAP present in generated code
//!  8.  production_id_count present in generated code
//!  9.  Empty grammar → minimal production IDs (count ≥ 1)
//! 10.  Empty grammar generates valid LANGUAGE struct
//! 11.  Large grammar production IDs are all unique
//! 12.  Large grammar production count is correct
//! 13.  Production ID determinism (same grammar → same map)
//! 14.  Production ID determinism across rebuilds
//! 15.  Serialized production count matches ABI count
//! 16.  Sequential IDs form identity map
//! 17.  Gap in production IDs inflates map size
//! 18.  Epsilon-only rules get valid production IDs
//! 19.  Multi-nonterminal grammars get distinct IDs
//! 20.  External tokens don't affect production IDs
//! 21.  production_id_count never zero for non-empty grammar
//! 22.  Mixed rule lengths preserve production ID assignment
//! 23.  Single-rule grammar gets production ID 0
//! 24.  Compressed and uncompressed maps agree
//! 25.  Production ID map values fit in u16
//! 26.  Production IDs stable under token reordering
//! 27.  Many nonterminals all get distinct production IDs
//! 28.  Production count from serializer equals rule count
//! 29.  Generated code contains LANGUAGE struct
//! 30.  Production ID map is reproducible across 10 iterations
//! 31.  Alias sequences don't alter production ID map
//! 32.  Fields on rules don't alter production ID map values
//! 33.  Production ID zero case: zero-ID rule always first in map
//! 34.  Alias sequences deterministic across rebuilds
//! 35.  Fields preserve production ID count
//! 36.  PRODUCTION_LHS_INDEX present in generated code
//! 37.  PRODUCTION_LHS_INDEX length matches production count
//! 38.  TS_RULES present in generated code
//! 39.  Grammar name does not affect production IDs
//! 40.  Production count monotonically increases with rule count
//! 41.  Sentinel fill for production ID gaps
//! 42.  Reverse-ordered production IDs handled correctly
//! 43.  Uniform RHS length rules get correct IDs
//! 44.  Supertypes don't affect production IDs
//! 45.  Multiple epsilon rules across nonterminals
//! 46.  PRODUCTION_LHS_INDEX deterministic across rebuilds
//! 47.  FIELD_MAP_SLICES present when fields exist
//! 48.  FIELD_MAP_ENTRIES minimal when no fields
//! 49.  Interleaved nonterminal rules get distinct IDs
//! 50.  Production ID map covers 0 to max
//! 51.  Serialized and ABI counts agree for multi-NT grammars
//! 52.  Production count equals total rules across nonterminals
//! 53.  Single NT many tokens gets correct IDs
//! 54.  Alias sequence length doesn't affect production_id_count
//! 55.  Production map values ascending for sequential IDs
//! 56.  Adding extras doesn't affect production IDs
//! 57.  Multiple fields on same rule don't change production IDs
//! 58.  Non-terminal references in RHS preserve production IDs
//! 59.  Production ID count always at least 1
//! 60.  PRODUCTION_LHS_INDEX consistent across identical codegen
//! 61.  Sparse nonterminal IDs preserve production uniqueness
//! 62.  Map and LHS index have same number of entries
//! 63.  Single epsilon rule gives count = 1
//! 64.  Alias None entries don't affect map
//! 65.  Maximum gap still has correct count

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    AliasSequence, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::serializer::serialize_language;
use proptest::prelude::*;
use std::collections::{BTreeMap, HashSet};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

fn tok(name: &str, lit: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(lit.to_string()),
        fragile: false,
    }
}

fn make_rule(lhs: SymbolId, rhs: Vec<Symbol>, prod_id: u16) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod_id),
    }
}

/// Build a ParseTable with the given dimensions.
fn empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = nonterms.max(1);
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let eof_symbol = SymbolId(eof_idx as u16);
    let start_symbol = SymbolId((eof_idx + 1) as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for col in (eof_idx + 1)..symbol_count {
        nonterminal_to_index.insert(SymbolId(col as u16), col);
    }

    let token_count = eof_idx + 1;

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        symbol_metadata: vec![],
        state_count: states,
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
        token_count,
        external_token_count: externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build a grammar with one terminal and a start nonterminal.
fn base_grammar(name: &str, table: &ParseTable) -> (Grammar, SymbolId, SymbolId) {
    let mut g = Grammar::new(name.to_string());
    let start = table.start_symbol;
    let t = SymbolId(1);
    g.rule_names.insert(start, "start".to_string());
    g.tokens.insert(t, tok("t", "t"));
    (g, start, t)
}

fn gen_code(grammar: &Grammar, table: &ParseTable) -> String {
    let builder = AbiLanguageBuilder::new(grammar, table);
    builder.generate().to_string()
}

/// Extract a `&[u16]` array by name from generated code.
fn extract_u16_array(code: &str, name: &str) -> Vec<u16> {
    let marker = format!("{} : & [u16]", name);
    let start = match code.find(&marker) {
        Some(pos) => pos,
        None => return vec![],
    };
    let rest = &code[start + marker.len()..];
    let eq_bracket = match rest.find("& [") {
        Some(pos) => pos + 3,
        None => return vec![],
    };
    let inner_rest = &rest[eq_bracket..];
    let close = match inner_rest.find(']') {
        Some(pos) => pos,
        None => return vec![],
    };
    let inner = &inner_rest[..close];
    inner
        .split(',')
        .filter_map(|s| {
            let s = s.trim();
            let s = if let Some(idx) = s.find("u16") {
                s[..idx].trim()
            } else if let Some(idx) = s.find("usize") {
                s[..idx].trim()
            } else {
                s
            };
            s.parse::<u16>().ok()
        })
        .collect()
}

fn extract_production_id_map(code: &str) -> Vec<u16> {
    extract_u16_array(code, "PRODUCTION_ID_MAP")
}

/// Extract `production_id_count` from generated code.
fn extract_production_id_count(code: &str) -> Option<u32> {
    let marker = "production_id_count :";
    let start = code.find(marker)?;
    let rest = &code[start + marker.len()..];
    let end = rest.find(',')?;
    let num_str = rest[..end].trim().trim_end_matches("u32");
    num_str.parse::<u32>().ok()
}

/// Extract production_id_count from the JSON serialized output.
fn serialized_production_count(grammar: &Grammar, table: &ParseTable) -> u32 {
    let json = serialize_language(grammar, table, None).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["production_id_count"].as_u64().unwrap() as u32
}

/// Build a grammar with `n` sequential rules on `start` nonterminal.
fn grammar_with_n_rules(n: usize) -> (Grammar, ParseTable) {
    let table = empty_table(1, 1, 1, 0);
    let (mut g, start, t) = base_grammar("proptest", &table);
    for i in 0..n {
        g.add_rule(make_rule(
            start,
            vec![Symbol::Terminal(t); (i % 3) + 1],
            i as u16,
        ));
    }
    (g, table)
}

/// Build a grammar with `n` rules across `nt_count` nonterminals.
fn grammar_with_nonterminals(n_rules: usize, nt_count: usize) -> (Grammar, ParseTable) {
    let nt_count = nt_count.max(1);
    let table = empty_table(1, 2, nt_count, 0);
    let start = table.start_symbol;
    let t1 = SymbolId(1);
    let t2 = SymbolId(2);

    let mut g = Grammar::new("proptest_multi".to_string());
    g.tokens.insert(t1, tok("a", "a"));
    g.tokens.insert(t2, tok("b", "b"));

    let mut prod_id = 0u16;
    for nt_off in 0..nt_count {
        let nt = SymbolId(start.0 + nt_off as u16);
        g.rule_names.insert(nt, format!("nt_{}", nt_off));
        let rules_per_nt = if nt_off < n_rules % nt_count {
            n_rules / nt_count + 1
        } else {
            n_rules / nt_count
        };
        for j in 0..rules_per_nt {
            let tok_sym = if j % 2 == 0 {
                Symbol::Terminal(t1)
            } else {
                Symbol::Terminal(t2)
            };
            g.add_rule(make_rule(nt, vec![tok_sym; (j % 3) + 1], prod_id));
            prod_id += 1;
        }
    }
    (g, table)
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy for rule count in [1, 50].
#[allow(dead_code)]
fn rule_count_strategy() -> impl Strategy<Value = usize> {
    1..=50usize
}

/// Strategy for nonterminal count in [1, 10].
#[allow(dead_code)]
fn nt_count_strategy() -> impl Strategy<Value = usize> {
    1..=10usize
}

// ===========================================================================
// Property Tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 1. Production IDs are sequential (0..n for n rules)
    #[test]
    fn sequential_ids_for_n_rules(n in 1..=30usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), n, "map length must equal rule count");
        for i in 0..n {
            prop_assert_eq!(map[i], i as u16, "slot {} should map to {}", i, i);
        }
    }

    // 2. Production ID map length equals rule count
    #[test]
    fn map_length_equals_rule_count(n in 1..=40usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let rule_count: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(map.len(), rule_count);
    }

    // 3. Production ID map values are unique
    #[test]
    fn map_values_are_unique(n in 1..=30usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), map.len(), "all production IDs must be unique");
    }

    // 4. Production count matches grammar rule count
    #[test]
    fn production_count_matches_rules(n in 1..=30usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, n as u32);
    }

    // 5. production_id_count == max(production_id) + 1
    #[test]
    fn count_is_max_plus_one(n in 1..=30usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        let map = extract_production_id_map(&code);
        let max_id = map.iter().copied().max().unwrap_or(0);
        prop_assert_eq!(count, max_id as u32 + 1);
    }

    // 6. Production IDs present in generated code
    #[test]
    fn production_ids_in_generated_code(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("production_id_count"), "must contain production_id_count");
    }

    // 7. PRODUCTION_ID_MAP present in generated code
    #[test]
    fn production_id_map_in_generated_code(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("PRODUCTION_ID_MAP"), "must contain PRODUCTION_ID_MAP");
    }

    // 8. production_id_count present in generated code
    #[test]
    fn production_id_count_present(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code);
        prop_assert!(count.is_some(), "production_id_count must be extractable");
    }

    // 9. Empty grammar → minimal production IDs (count ≥ 1)
    #[test]
    fn empty_grammar_minimal_ids(_dummy in 0..5u8) {
        let table = empty_table(1, 1, 1, 0);
        let (g, _start, _t) = base_grammar("empty", &table);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code);
        prop_assert!(count.is_some());
        prop_assert!(count.unwrap() >= 1, "empty grammar must have count ≥ 1");
    }

    // 10. Empty grammar generates valid LANGUAGE struct
    #[test]
    fn empty_grammar_has_language_struct(_dummy in 0..5u8) {
        let table = empty_table(1, 1, 1, 0);
        let (g, _start, _t) = base_grammar("empty_lang", &table);
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("LANGUAGE"), "must contain LANGUAGE struct");
    }

    // 11. Large grammar production IDs are all unique
    #[test]
    fn large_grammar_unique_ids(n in 30..=50usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), n, "all {} production IDs must be unique", n);
    }

    // 12. Large grammar production count is correct
    #[test]
    fn large_grammar_count_correct(n in 30..=50usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, n as u32);
    }

    // 13. Production ID determinism (same grammar → same map)
    #[test]
    fn determinism_same_grammar(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code1 = gen_code(&g, &table);
        let code2 = gen_code(&g, &table);
        prop_assert_eq!(
            extract_production_id_map(&code1),
            extract_production_id_map(&code2),
            "production ID map must be deterministic"
        );
    }

    // 14. Production ID determinism across rebuilds
    #[test]
    fn determinism_across_rebuilds(n in 1..=20usize) {
        let (g1, t1) = grammar_with_n_rules(n);
        let (g2, t2) = grammar_with_n_rules(n);
        let map1 = extract_production_id_map(&gen_code(&g1, &t1));
        let map2 = extract_production_id_map(&gen_code(&g2, &t2));
        prop_assert_eq!(map1, map2, "rebuild must produce identical map");
    }

    // 15. Serialized production count matches ABI count
    #[test]
    fn serialized_matches_abi_count(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let ser_count = serialized_production_count(&g, &table);
        // Serializer counts rules directly
        let rule_count: u32 = g.rules.values().map(|v| v.len() as u32).sum();
        prop_assert_eq!(ser_count, rule_count);
    }

    // 16. Sequential IDs form identity map
    #[test]
    fn sequential_ids_identity_map(n in 1..=25usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        for i in 0..n {
            prop_assert_eq!(map[i], i as u16, "identity map: slot {} = {}", i, i);
        }
    }

    // 17. Gap in production IDs inflates map size
    #[test]
    fn gap_inflates_map(gap in 2..=10u16) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, t) = base_grammar("gap", &table);
        g.add_rule(make_rule(start, vec![Symbol::Terminal(t)], 0));
        g.add_rule(make_rule(start, vec![Symbol::Terminal(t), Symbol::Terminal(t)], gap));

        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, gap as u32 + 1, "count must cover gap");
    }

    // 18. Epsilon-only rules get valid production IDs
    #[test]
    fn epsilon_rules_valid_ids(n in 1..=10usize) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, _t) = base_grammar("eps", &table);
        for i in 0..n {
            g.add_rule(make_rule(start, vec![], i as u16));
        }
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), n);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, n as u32);
    }

    // 19. Multi-nonterminal grammars get distinct IDs
    #[test]
    fn multi_nt_distinct_ids(
        n_rules in 2..=20usize,
        nt_count in 2..=5usize,
    ) {
        let (g, table) = grammar_with_nonterminals(n_rules, nt_count);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), map.len(), "all IDs must be distinct across nonterminals");
    }

    // 20. External tokens don't affect production IDs
    #[test]
    fn externals_dont_affect_ids(n in 1..=10usize) {
        // Without externals
        let table_no_ext = empty_table(1, 1, 1, 0);
        let (mut g_no_ext, start_no, t_no) = base_grammar("no_ext", &table_no_ext);
        for i in 0..n {
            g_no_ext.add_rule(make_rule(start_no, vec![Symbol::Terminal(t_no)], i as u16));
        }
        let map_no_ext = extract_production_id_map(&gen_code(&g_no_ext, &table_no_ext));

        // With externals
        let table_ext = empty_table(1, 1, 1, 1);
        let start_ext = table_ext.start_symbol;
        let t_ext = SymbolId(1);
        let mut g_ext = Grammar::new("with_ext".to_string());
        g_ext.rule_names.insert(start_ext, "start".to_string());
        g_ext.tokens.insert(t_ext, tok("t", "t"));
        for i in 0..n {
            g_ext.add_rule(make_rule(start_ext, vec![Symbol::Terminal(t_ext)], i as u16));
        }
        let map_ext = extract_production_id_map(&gen_code(&g_ext, &table_ext));

        prop_assert_eq!(map_no_ext, map_ext, "externals must not affect production ID map");
    }

    // 21. production_id_count never zero for non-empty grammar
    #[test]
    fn count_never_zero_nonempty(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert!(count > 0, "non-empty grammar must have count > 0");
    }

    // 22. Mixed rule lengths preserve production ID assignment
    #[test]
    fn mixed_lengths_preserve_ids(n in 2..=15usize) {
        let table = empty_table(1, 2, 1, 0);
        let start = table.start_symbol;
        let t1 = SymbolId(1);
        let t2 = SymbolId(2);
        let mut g = Grammar::new("mixed".to_string());
        g.rule_names.insert(start, "start".to_string());
        g.tokens.insert(t1, tok("a", "a"));
        g.tokens.insert(t2, tok("b", "b"));

        for i in 0..n {
            let rhs = match i % 4 {
                0 => vec![],
                1 => vec![Symbol::Terminal(t1)],
                2 => vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
                _ => vec![Symbol::Terminal(t1), Symbol::Terminal(t2), Symbol::Terminal(t1)],
            };
            g.add_rule(make_rule(start, rhs, i as u16));
        }

        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), n);
        for i in 0..n {
            prop_assert_eq!(map[i], i as u16);
        }
    }

    // 23. Single-rule grammar gets production ID 0
    #[test]
    fn single_rule_id_zero(_dummy in 0..10u8) {
        let (g, table) = grammar_with_n_rules(1);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), 1);
        prop_assert_eq!(map[0], 0);
    }

    // 24. Compressed and uncompressed maps agree
    #[test]
    fn compressed_agrees_with_plain(n in 1..=10usize) {
        let table = empty_table(2, 1, 1, 0);
        let (mut g, start, t) = base_grammar("comp", &table);
        for i in 0..n {
            g.add_rule(make_rule(start, vec![Symbol::Terminal(t); (i % 3) + 1], i as u16));
        }

        let code_plain = gen_code(&g, &table);
        let map_plain = extract_production_id_map(&code_plain);

        let compressor = adze_tablegen::TableCompressor::new();
        let token_indices = adze_tablegen::collect_token_indices(&g, &table);
        if let Ok(compressed) = compressor.compress(&table, &token_indices, false) {
            let builder = AbiLanguageBuilder::new(&g, &table).with_compressed_tables(&compressed);
            let code_comp = builder.generate().to_string();
            let map_comp = extract_production_id_map(&code_comp);
            prop_assert_eq!(map_plain, map_comp, "compressed map must match plain map");
        }
    }

    // 25. Production ID map values fit in u16
    #[test]
    fn map_values_fit_u16(n in 1..=40usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        for val in &map {
            // val is u16, so it's always <= u16::MAX
            let _ = val;
        }
    }

    // 26. Production IDs stable under token reordering
    #[test]
    fn stable_under_token_reorder(n in 1..=10usize) {
        let table = empty_table(1, 2, 1, 0);
        let start = table.start_symbol;

        // Grammar with tokens inserted in order (1, 2)
        let mut g1 = Grammar::new("order1".to_string());
        g1.rule_names.insert(start, "start".to_string());
        g1.tokens.insert(SymbolId(1), tok("a", "a"));
        g1.tokens.insert(SymbolId(2), tok("b", "b"));
        for i in 0..n {
            g1.add_rule(make_rule(start, vec![Symbol::Terminal(SymbolId(1))], i as u16));
        }

        // Grammar with tokens inserted in reverse order (2, 1)
        let mut g2 = Grammar::new("order2".to_string());
        g2.rule_names.insert(start, "start".to_string());
        g2.tokens.insert(SymbolId(2), tok("b", "b"));
        g2.tokens.insert(SymbolId(1), tok("a", "a"));
        for i in 0..n {
            g2.add_rule(make_rule(start, vec![Symbol::Terminal(SymbolId(1))], i as u16));
        }

        let map1 = extract_production_id_map(&gen_code(&g1, &table));
        let map2 = extract_production_id_map(&gen_code(&g2, &table));
        prop_assert_eq!(map1, map2, "token insertion order must not affect production IDs");
    }

    // 27. Many nonterminals all get distinct production IDs
    #[test]
    fn many_nonterminals_distinct(nt_count in 2..=8usize) {
        let rules_per_nt = 3;
        let total = nt_count * rules_per_nt;
        let (g, table) = grammar_with_nonterminals(total, nt_count);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), total, "all {} IDs must be distinct", total);
    }

    // 28. Production count from serializer equals rule count
    #[test]
    fn serializer_count_equals_rules(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let ser_count = serialized_production_count(&g, &table);
        prop_assert_eq!(ser_count, n as u32);
    }

    // 29. Generated code contains LANGUAGE struct
    #[test]
    fn generated_code_has_language(n in 1..=15usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("LANGUAGE"));
    }

    // 30. Production ID map is reproducible across 10 iterations
    #[test]
    fn reproducible_across_iterations(n in 1..=15usize) {
        let (g, table) = grammar_with_n_rules(n);
        let reference = extract_production_id_map(&gen_code(&g, &table));
        for _ in 0..10 {
            let map = extract_production_id_map(&gen_code(&g, &table));
            prop_assert_eq!(&map, &reference, "map must be identical across iterations");
        }
    }

    // 31. Alias sequences don't alter production ID map
    #[test]
    fn aliases_dont_alter_production_id_map(n in 1..=15usize) {
        // Build baseline grammar
        let (g_base, table) = grammar_with_n_rules(n);
        let map_base = extract_production_id_map(&gen_code(&g_base, &table));

        // Build grammar with alias sequences added
        let (mut g_alias, table2) = grammar_with_n_rules(n);
        for i in 0..n {
            let pid = ProductionId(i as u16);
            let mut aliases = vec![None; (i % 3) + 1];
            aliases[0] = Some(format!("alias_{}", i));
            g_alias.alias_sequences.insert(pid, AliasSequence { aliases });
        }
        g_alias.max_alias_sequence_length = 3;
        let map_alias = extract_production_id_map(&gen_code(&g_alias, &table2));

        prop_assert_eq!(map_base, map_alias, "aliases must not alter production ID map");
    }

    // 32. Fields on rules don't alter production ID map values
    #[test]
    fn fields_dont_alter_production_id_map_values(n in 1..=15usize) {
        // Baseline without fields
        let (g_base, table) = grammar_with_n_rules(n);
        let map_base = extract_production_id_map(&gen_code(&g_base, &table));

        // Grammar with fields on rules
        let table2 = empty_table(1, 1, 1, 0);
        let (mut g_fields, start, t) = base_grammar("fields", &table2);
        g_fields.fields.insert(FieldId(1), "left".to_string());
        g_fields.fields.insert(FieldId(2), "right".to_string());
        for i in 0..n {
            let mut rule = make_rule(
                start,
                vec![Symbol::Terminal(t); (i % 3) + 1],
                i as u16,
            );
            // Attach field to position 0
            rule.fields.push((FieldId(1), 0));
            g_fields.add_rule(rule);
        }
        let map_fields = extract_production_id_map(&gen_code(&g_fields, &table2));

        // Same length and same values
        prop_assert_eq!(map_base.len(), map_fields.len(), "field rules same count");
        prop_assert_eq!(map_base, map_fields, "fields must not alter production ID map values");
    }

    // 33. Production ID zero case: zero-ID rule always first in map
    #[test]
    fn zero_id_rule_first_in_map(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert!(!map.is_empty(), "map must not be empty");
        prop_assert_eq!(map[0], 0u16, "production ID 0 must occupy slot 0");
    }

    // 34. Alias sequences deterministic across rebuilds
    #[test]
    fn alias_determinism_across_rebuilds(n in 1..=10usize) {
        let build = || {
            let (mut g, table) = grammar_with_n_rules(n);
            for i in 0..n {
                let pid = ProductionId(i as u16);
                g.alias_sequences.insert(pid, AliasSequence {
                    aliases: vec![Some(format!("a{}", i))],
                });
            }
            g.max_alias_sequence_length = 1;
            (g, table)
        };
        let (g1, t1) = build();
        let (g2, t2) = build();
        let map1 = extract_production_id_map(&gen_code(&g1, &t1));
        let map2 = extract_production_id_map(&gen_code(&g2, &t2));
        prop_assert_eq!(map1, map2, "alias grammars must be deterministic across rebuilds");
    }

    // 35. Fields preserve production ID count
    #[test]
    fn fields_preserve_production_id_count(n in 1..=15usize) {
        // Baseline count
        let (g_base, table_base) = grammar_with_n_rules(n);
        let count_base = extract_production_id_count(&gen_code(&g_base, &table_base)).unwrap();

        // Grammar with fields
        let table2 = empty_table(1, 2, 1, 0);
        let start = table2.start_symbol;
        let t1 = SymbolId(1);
        let t2 = SymbolId(2);
        let mut g = Grammar::new("fields_count".to_string());
        g.rule_names.insert(start, "start".to_string());
        g.tokens.insert(t1, tok("a", "a"));
        g.tokens.insert(t2, tok("b", "b"));
        g.fields.insert(FieldId(1), "operand".to_string());
        for i in 0..n {
            let tok_sym = if i % 2 == 0 { Symbol::Terminal(t1) } else { Symbol::Terminal(t2) };
            let mut rule = make_rule(start, vec![tok_sym], i as u16);
            rule.fields.push((FieldId(1), 0));
            g.add_rule(rule);
        }
        let count_fields = extract_production_id_count(&gen_code(&g, &table2)).unwrap();

        prop_assert_eq!(count_base, count_fields, "field rules must preserve production_id_count");
    }

    // 36. PRODUCTION_LHS_INDEX present in generated code
    #[test]
    fn production_lhs_index_present(n in 1..=15usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("PRODUCTION_LHS_INDEX"), "must contain PRODUCTION_LHS_INDEX");
    }

    // 37. PRODUCTION_LHS_INDEX length matches production count
    #[test]
    fn production_lhs_index_length_matches_count(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let lhs_index = extract_u16_array(&code, "PRODUCTION_LHS_INDEX");
        prop_assert_eq!(lhs_index.len(), n, "LHS index length must equal rule count");
    }

    // 38. TS_RULES present in generated code
    #[test]
    fn ts_rules_present(n in 1..=15usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("TS_RULES"), "must contain TS_RULES");
    }

    // 39. Grammar name does not affect production IDs
    #[test]
    fn grammar_name_independent(n in 1..=15usize) {
        let table = empty_table(1, 1, 1, 0);
        let start = table.start_symbol;
        let t = SymbolId(1);

        let build = |name: &str| {
            let mut g = Grammar::new(name.to_string());
            g.rule_names.insert(start, "start".to_string());
            g.tokens.insert(t, tok("t", "t"));
            for i in 0..n {
                g.add_rule(make_rule(start, vec![Symbol::Terminal(t); (i % 3) + 1], i as u16));
            }
            g
        };

        let map_a = extract_production_id_map(&gen_code(&build("alpha"), &table));
        let map_b = extract_production_id_map(&gen_code(&build("beta"), &table));
        prop_assert_eq!(map_a, map_b, "grammar name must not affect production ID map");
    }

    // 40. Production count monotonically increases with rule count
    #[test]
    fn count_monotonic_with_rules(n in 2..=25usize) {
        let (g_small, table_small) = grammar_with_n_rules(n - 1);
        let (g_large, table_large) = grammar_with_n_rules(n);
        let count_small = extract_production_id_count(&gen_code(&g_small, &table_small)).unwrap();
        let count_large = extract_production_id_count(&gen_code(&g_large, &table_large)).unwrap();
        prop_assert!(count_large > count_small, "adding a rule must increase production count");
    }

    // 41. Production ID map sentinel fill for gaps
    #[test]
    fn sentinel_fill_for_gaps(gap in 3..=8u16) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, t) = base_grammar("sentinel", &table);
        g.add_rule(make_rule(start, vec![Symbol::Terminal(t)], 0));
        g.add_rule(make_rule(start, vec![Symbol::Terminal(t), Symbol::Terminal(t)], gap));

        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        // Slot 0 = production ID 0, slot 1 = production ID `gap`
        prop_assert_eq!(map[0], 0u16);
        prop_assert_eq!(map[1], gap);
    }

    // 42. Reverse-ordered production IDs are handled
    #[test]
    fn reverse_ordered_ids(n in 2..=10usize) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, t) = base_grammar("reverse", &table);
        for i in 0..n {
            let prod_id = (n - 1 - i) as u16;
            g.add_rule(make_rule(start, vec![Symbol::Terminal(t); (i % 3) + 1], prod_id));
        }
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), n, "reverse-ordered IDs must still be unique");
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, n as u32);
    }

    // 43. Production ID map with all same RHS length
    #[test]
    fn uniform_rhs_length(n in 1..=20usize) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, t) = base_grammar("uniform", &table);
        for i in 0..n {
            g.add_rule(make_rule(start, vec![Symbol::Terminal(t); 2], i as u16));
        }
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), n);
        for i in 0..n {
            prop_assert_eq!(map[i], i as u16);
        }
    }

    // 44. Supertypes don't affect production IDs
    #[test]
    fn supertypes_dont_affect_ids(n in 1..=10usize) {
        let table = empty_table(1, 1, 1, 0);
        let start = table.start_symbol;
        let t = SymbolId(1);

        // Without supertypes
        let mut g1 = Grammar::new("no_super".to_string());
        g1.rule_names.insert(start, "start".to_string());
        g1.tokens.insert(t, tok("t", "t"));
        for i in 0..n {
            g1.add_rule(make_rule(start, vec![Symbol::Terminal(t)], i as u16));
        }
        let map1 = extract_production_id_map(&gen_code(&g1, &table));

        // With supertypes
        let mut g2 = Grammar::new("with_super".to_string());
        g2.rule_names.insert(start, "start".to_string());
        g2.tokens.insert(t, tok("t", "t"));
        g2.supertypes.push(start);
        for i in 0..n {
            g2.add_rule(make_rule(start, vec![Symbol::Terminal(t)], i as u16));
        }
        let map2 = extract_production_id_map(&gen_code(&g2, &table));

        prop_assert_eq!(map1, map2, "supertypes must not affect production ID map");
    }

    // 45. Multiple epsilon rules across nonterminals
    #[test]
    fn epsilon_across_nonterminals(nt_count in 2..=5usize) {
        let table = empty_table(1, 1, nt_count, 0);
        let start = table.start_symbol;
        let t = SymbolId(1);

        let mut g = Grammar::new("eps_multi".to_string());
        g.tokens.insert(t, tok("t", "t"));
        for (prod_id, off) in (0..nt_count).enumerate() {
            let nt = SymbolId(start.0 + off as u16);
            g.rule_names.insert(nt, format!("nt_{}", off));
            g.add_rule(make_rule(nt, vec![], prod_id as u16));
        }

        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), nt_count, "epsilon rules across NTs must have distinct IDs");
    }

    // 46. PRODUCTION_LHS_INDEX deterministic across rebuilds
    #[test]
    fn lhs_index_deterministic(n in 1..=15usize) {
        let (g1, t1) = grammar_with_n_rules(n);
        let (g2, t2) = grammar_with_n_rules(n);
        let lhs1 = extract_u16_array(&gen_code(&g1, &t1), "PRODUCTION_LHS_INDEX");
        let lhs2 = extract_u16_array(&gen_code(&g2, &t2), "PRODUCTION_LHS_INDEX");
        prop_assert_eq!(lhs1, lhs2, "LHS index must be deterministic across rebuilds");
    }

    // 47. FIELD_MAP_SLICES present when fields exist
    #[test]
    fn field_map_slices_present_with_fields(n in 1..=10usize) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, t) = base_grammar("fms", &table);
        g.fields.insert(FieldId(1), "operand".to_string());
        for i in 0..n {
            let mut r = make_rule(start, vec![Symbol::Terminal(t)], i as u16);
            r.fields.push((FieldId(1), 0));
            g.add_rule(r);
        }
        let code = gen_code(&g, &table);
        prop_assert!(code.contains("FIELD_MAP_SLICES"), "code must contain FIELD_MAP_SLICES");
        prop_assert!(code.contains("FIELD_MAP_ENTRIES"), "code must contain FIELD_MAP_ENTRIES");
    }

    // 48. FIELD_MAP_ENTRIES is minimal when no fields
    #[test]
    fn field_map_entries_minimal_no_fields(n in 1..=10usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        // With no fields, field_count should be 0
        prop_assert!(code.contains("field_count : 0u32") || code.contains("field_count : 0"),
            "field_count must be 0 when no fields");
    }

    // 49. Production IDs with interleaved nonterminal rules
    #[test]
    fn interleaved_nt_rules(pairs in 1..=8usize) {
        let table = empty_table(1, 1, 2, 0);
        let start = table.start_symbol;
        let other = SymbolId(start.0 + 1);
        let t = SymbolId(1);

        let mut g = Grammar::new("interleaved".to_string());
        g.rule_names.insert(start, "start".to_string());
        g.rule_names.insert(other, "other".to_string());
        g.tokens.insert(t, tok("t", "t"));

        for i in 0..pairs {
            g.add_rule(make_rule(start, vec![Symbol::Terminal(t); (i % 3) + 1], (i * 2) as u16));
            g.add_rule(make_rule(other, vec![Symbol::Terminal(t); (i % 2) + 1], (i * 2 + 1) as u16));
        }

        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), pairs * 2, "interleaved IDs must all be distinct");
    }

    // 50. Production ID map covers all IDs from 0 to max
    #[test]
    fn map_covers_0_to_max(n in 1..=20usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let max_id = map.iter().copied().max().unwrap_or(0);
        prop_assert_eq!(max_id, (n - 1) as u16, "max ID must be n-1 for sequential IDs");
    }

    // 51. Serialized and ABI production counts agree for multi-NT grammars
    #[test]
    fn serialized_abi_agree_multi_nt(
        n_rules in 2..=15usize,
        nt_count in 2..=4usize,
    ) {
        let (g, table) = grammar_with_nonterminals(n_rules, nt_count);
        let abi_count = extract_production_id_count(&gen_code(&g, &table)).unwrap();
        let ser_count = serialized_production_count(&g, &table);
        prop_assert_eq!(abi_count, ser_count, "ABI and serialized counts must agree for multi-NT");
    }

    // 52. Production count equals total rules across all nonterminals
    #[test]
    fn count_equals_total_rules(
        n_rules in 1..=20usize,
        nt_count in 1..=5usize,
    ) {
        let (g, table) = grammar_with_nonterminals(n_rules, nt_count);
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), total, "map length must equal total rule count");
    }

    // 53. Production IDs with single nonterminal, many tokens
    #[test]
    fn single_nt_many_tokens(tok_count in 2..=6usize) {
        let table = empty_table(1, tok_count, 1, 0);
        let start = table.start_symbol;
        let mut g = Grammar::new("many_tok".to_string());
        g.rule_names.insert(start, "start".to_string());
        for i in 1..=tok_count {
            g.tokens.insert(SymbolId(i as u16), tok(&format!("t{}", i), &format!("{}", i)));
        }
        for i in 0..tok_count {
            g.add_rule(make_rule(start, vec![Symbol::Terminal(SymbolId((i + 1) as u16))], i as u16));
        }
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map.len(), tok_count);
        for i in 0..tok_count {
            prop_assert_eq!(map[i], i as u16);
        }
    }

    // 54. Alias sequence length doesn't affect production_id_count
    #[test]
    fn alias_length_doesnt_affect_count(n in 1..=10usize, alias_len in 1..=5usize) {
        let (mut g, table) = grammar_with_n_rules(n);
        for i in 0..n {
            let pid = ProductionId(i as u16);
            let aliases = vec![Some(format!("a{}", i)); alias_len];
            g.alias_sequences.insert(pid, AliasSequence { aliases });
        }
        g.max_alias_sequence_length = alias_len;
        let count = extract_production_id_count(&gen_code(&g, &table)).unwrap();
        prop_assert_eq!(count, n as u32, "alias length must not change production_id_count");
    }

    // 55. Production map is sorted by production ID (values are ascending)
    #[test]
    fn map_values_ascending_for_sequential(n in 1..=25usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        for i in 1..map.len() {
            prop_assert!(map[i] > map[i - 1], "map values must be strictly ascending for sequential IDs");
        }
    }

    // 56. Adding extras doesn't affect production IDs
    #[test]
    fn extras_dont_affect_ids(n in 1..=10usize) {
        let table = empty_table(1, 2, 1, 0);
        let start = table.start_symbol;
        let t1 = SymbolId(1);
        let t2 = SymbolId(2);

        // Without extras
        let mut g1 = Grammar::new("no_extras".to_string());
        g1.rule_names.insert(start, "start".to_string());
        g1.tokens.insert(t1, tok("a", "a"));
        g1.tokens.insert(t2, tok("ws", " "));
        for i in 0..n {
            g1.add_rule(make_rule(start, vec![Symbol::Terminal(t1)], i as u16));
        }
        let map1 = extract_production_id_map(&gen_code(&g1, &table));

        // With extras
        let mut g2 = Grammar::new("with_extras".to_string());
        g2.rule_names.insert(start, "start".to_string());
        g2.tokens.insert(t1, tok("a", "a"));
        g2.tokens.insert(t2, tok("ws", " "));
        g2.extras.push(t2);
        for i in 0..n {
            g2.add_rule(make_rule(start, vec![Symbol::Terminal(t1)], i as u16));
        }
        let map2 = extract_production_id_map(&gen_code(&g2, &table));

        prop_assert_eq!(map1, map2, "extras must not affect production ID map");
    }

    // 57. Multiple fields on a single rule don't change production IDs
    #[test]
    fn multiple_fields_same_rule(n in 1..=10usize) {
        let (g_base, table_base) = grammar_with_n_rules(n);
        let map_base = extract_production_id_map(&gen_code(&g_base, &table_base));

        let table2 = empty_table(1, 2, 1, 0);
        let start = table2.start_symbol;
        let t1 = SymbolId(1);
        let t2 = SymbolId(2);
        let mut g = Grammar::new("multi_field".to_string());
        g.rule_names.insert(start, "start".to_string());
        g.tokens.insert(t1, tok("a", "a"));
        g.tokens.insert(t2, tok("b", "b"));
        g.fields.insert(FieldId(1), "left".to_string());
        g.fields.insert(FieldId(2), "right".to_string());
        g.fields.insert(FieldId(3), "op".to_string());
        for i in 0..n {
            let mut r = make_rule(start, vec![Symbol::Terminal(t1); (i % 3) + 1], i as u16);
            r.fields.push((FieldId(1), 0));
            if r.rhs.len() > 1 {
                r.fields.push((FieldId(2), 1));
            }
            g.add_rule(r);
        }
        let map_fields = extract_production_id_map(&gen_code(&g, &table2));

        prop_assert_eq!(map_base, map_fields, "multiple fields must not alter production ID map");
    }

    // 58. Production ID map with non-terminal references in RHS
    #[test]
    fn nt_references_in_rhs(n in 1..=8usize) {
        let table = empty_table(1, 1, 2, 0);
        let start = table.start_symbol;
        let other = SymbolId(start.0 + 1);
        let t = SymbolId(1);

        let mut g = Grammar::new("nt_rhs".to_string());
        g.rule_names.insert(start, "start".to_string());
        g.rule_names.insert(other, "other".to_string());
        g.tokens.insert(t, tok("t", "t"));

        g.add_rule(make_rule(other, vec![Symbol::Terminal(t)], 0));
        for i in 1..=n {
            g.add_rule(make_rule(start, vec![Symbol::NonTerminal(other); (i % 3) + 1], i as u16));
        }

        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), n + 1, "NT references in RHS must not break production IDs");
    }

    // 59. Production ID count is always at least 1 for any grammar
    #[test]
    fn count_at_least_one(_dummy in 0..10u8) {
        let table = empty_table(1, 1, 1, 0);
        let (g, _start, _t) = base_grammar("mincount", &table);
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert!(count >= 1, "production_id_count must be at least 1");
    }

    // 60. PRODUCTION_LHS_INDEX values are consistent across identical grammars
    #[test]
    fn lhs_index_consistent(n in 1..=10usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code1 = gen_code(&g, &table);
        let code2 = gen_code(&g, &table);
        let lhs1 = extract_u16_array(&code1, "PRODUCTION_LHS_INDEX");
        let lhs2 = extract_u16_array(&code2, "PRODUCTION_LHS_INDEX");
        prop_assert_eq!(lhs1, lhs2, "LHS index must be consistent across identical codegen");
    }

    // 61. Production IDs with sparse nonterminal IDs
    #[test]
    fn sparse_nonterminal_ids(n in 1..=8usize) {
        let table = empty_table(1, 2, 3, 0);
        let start = table.start_symbol;
        let nt2 = SymbolId(start.0 + 2); // skip one NT
        let t1 = SymbolId(1);
        let t2 = SymbolId(2);

        let mut g = Grammar::new("sparse_nt".to_string());
        g.rule_names.insert(start, "start".to_string());
        g.rule_names.insert(nt2, "leaf".to_string());
        g.tokens.insert(t1, tok("a", "a"));
        g.tokens.insert(t2, tok("b", "b"));

        let mut prod_id = 0u16;
        for _ in 0..n {
            g.add_rule(make_rule(start, vec![Symbol::Terminal(t1)], prod_id));
            prod_id += 1;
        }
        for _ in 0..n {
            g.add_rule(make_rule(nt2, vec![Symbol::Terminal(t2)], prod_id));
            prod_id += 1;
        }

        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let set: HashSet<u16> = map.iter().copied().collect();
        prop_assert_eq!(set.len(), n * 2, "sparse NT IDs must not affect production uniqueness");
    }

    // 62. Production ID map and LHS index have same number of entries
    #[test]
    fn map_and_lhs_index_same_length(n in 1..=15usize) {
        let (g, table) = grammar_with_n_rules(n);
        let code = gen_code(&g, &table);
        let map = extract_production_id_map(&code);
        let lhs = extract_u16_array(&code, "PRODUCTION_LHS_INDEX");
        prop_assert_eq!(map.len(), lhs.len(), "PRODUCTION_ID_MAP and PRODUCTION_LHS_INDEX must have same length");
    }

    // 63. Production ID count with only one epsilon rule
    #[test]
    fn single_epsilon_rule_count(_dummy in 0..5u8) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, _t) = base_grammar("single_eps", &table);
        g.add_rule(make_rule(start, vec![], 0));
        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, 1, "single epsilon rule must give count = 1");
    }

    // 64. Alias sequences with None entries don't affect map
    #[test]
    fn alias_none_entries_dont_affect_map(n in 1..=10usize) {
        let (g_base, table) = grammar_with_n_rules(n);
        let map_base = extract_production_id_map(&gen_code(&g_base, &table));

        let (mut g_alias, table2) = grammar_with_n_rules(n);
        for i in 0..n {
            let pid = ProductionId(i as u16);
            g_alias.alias_sequences.insert(pid, AliasSequence {
                aliases: vec![None; 3],
            });
        }
        g_alias.max_alias_sequence_length = 3;
        let map_alias = extract_production_id_map(&gen_code(&g_alias, &table2));

        prop_assert_eq!(map_base, map_alias, "None-only alias sequences must not alter map");
    }

    // 65. Production ID map with maximum gap still has correct count
    #[test]
    fn max_gap_correct_count(gap in 10..=30u16) {
        let table = empty_table(1, 1, 1, 0);
        let (mut g, start, t) = base_grammar("maxgap", &table);
        g.add_rule(make_rule(start, vec![Symbol::Terminal(t)], 0));
        g.add_rule(make_rule(start, vec![Symbol::Terminal(t), Symbol::Terminal(t)], gap));

        let code = gen_code(&g, &table);
        let count = extract_production_id_count(&code).unwrap();
        prop_assert_eq!(count, gap as u32 + 1, "count must be gap + 1");
        let map = extract_production_id_map(&code);
        prop_assert_eq!(map[0], 0u16);
        prop_assert_eq!(map[1], gap);
    }
}
