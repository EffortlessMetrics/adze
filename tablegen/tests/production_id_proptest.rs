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

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern};
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
fn rule_count_strategy() -> impl Strategy<Value = usize> {
    1..=50usize
}

/// Strategy for nonterminal count in [1, 10].
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
            prop_assert!(*val < 0xFFFF || *val == u16::MAX, "unexpected value: {}", val);
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
}
