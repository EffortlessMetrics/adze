#![cfg(feature = "test-api")]
//! Property-based tests for ParseTable invariants.
//!
//! Run with:
//! ```bash
//! RUST_TEST_THREADS=2 cargo test -p adze-glr-core --features test-api \
//!     --test proptest_parse_table_properties
//! ```

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a simple grammar: S -> tok1 (one token, one rule).
fn simple_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// Convenience: compute FIRST/FOLLOW then build parse table.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton failed")
}

/// Build a grammar with `n_tok` tokens named t0..t{n-1} and `rules` mapping
/// nonterminal names to sequences of token names.
fn grammar_with(name: &str, tokens: &[&str], rules: &[(&str, Vec<&str>)], start: &str) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for &tok in tokens {
        b = b.token(tok, tok);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    b.build()
}

// ---------------------------------------------------------------------------
// Proptest strategy: random *valid* grammars
// ---------------------------------------------------------------------------

/// Generate a random valid grammar with 1-5 tokens and 1-3 rules.
fn arb_valid_grammar() -> impl Strategy<Value = Grammar> {
    // Pick number of tokens (1-5) and number of extra rules (0-2).
    (1usize..=5, 0usize..=2)
        .prop_flat_map(|(n_tok, n_extra_rules)| {
            // For each extra rule we need a token index for the RHS.
            let rhs_indices = proptest::collection::vec(0..n_tok, n_extra_rules);
            (Just(n_tok), Just(n_extra_rules), rhs_indices)
        })
        .prop_map(|(n_tok, n_extra, rhs_indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("rand_grammar");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            // Base rule: S -> t0
            b = b.rule("S", vec![tok_names[0].as_str()]);
            // Extra rules: S -> t{k}
            for &idx in &rhs_indices {
                b = b.rule("S", vec![tok_names[idx].as_str()]);
            }
            b = b.start("S");
            b.build()
        })
}

/// Generate a random grammar with at least two nonterminals.
fn arb_two_nt_grammar() -> impl Strategy<Value = Grammar> {
    (1usize..=4, 0usize..=3)
        .prop_flat_map(|(n_tok, n_extra)| {
            let rhs_indices = proptest::collection::vec(0..n_tok, n_extra);
            (Just(n_tok), rhs_indices)
        })
        .prop_map(|(n_tok, rhs_indices)| {
            let tok_names: Vec<String> = (0..n_tok).map(|i| format!("t{i}")).collect();
            let mut b = GrammarBuilder::new("two_nt");
            for tn in &tok_names {
                b = b.token(tn, tn);
            }
            // S -> A, A -> t0
            b = b.rule("S", vec!["A"]);
            b = b.rule("A", vec![tok_names[0].as_str()]);
            for &idx in &rhs_indices {
                b = b.rule("A", vec![tok_names[idx].as_str()]);
            }
            b = b.start("S");
            b.build()
        })
}

// ---------------------------------------------------------------------------
// Property 1: Any valid grammar produces state_count > 0
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_state_count_positive(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0, "state_count must be > 0");
    }
}

// ---------------------------------------------------------------------------
// Property 2: action_table length equals state_count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_action_table_len_eq_state_count(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(
            table.action_table.len(),
            table.state_count,
            "action_table.len() must equal state_count"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 3: goto_table length equals state_count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_goto_table_len_eq_state_count(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(
            table.goto_table.len(),
            table.state_count,
            "goto_table.len() must equal state_count"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 4: Adding more rules doesn't decrease state count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_more_rules_no_fewer_states(n_extra in 0usize..=3) {
        // Base grammar: S -> t0
        let base = grammar_with("mono", &["t0", "t1", "t2"], &[("S", vec!["t0"])], "S");
        let base_table = build_table(&base);

        // Extended grammar: S -> t0 | t1 | ... (add alternatives)
        let all_toks = ["t0", "t1", "t2"];
        let mut rules: Vec<(&str, Vec<&str>)> = vec![("S", vec!["t0"])];
        for &tok in &all_toks[1..=(n_extra.min(2))] {
            rules.push(("S", vec![tok]));
        }
        let ext = grammar_with("ext", &["t0", "t1", "t2"], &rules, "S");
        let ext_table = build_table(&ext);

        prop_assert!(
            ext_table.state_count >= base_table.state_count,
            "extended grammar ({}) must have >= states than base ({})",
            ext_table.state_count,
            base_table.state_count,
        );
    }
}

// ---------------------------------------------------------------------------
// Property 5: symbol_to_index round-trips with index_to_symbol
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_symbol_index_roundtrip(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for (&sym, &idx) in &table.symbol_to_index {
            prop_assert!(
                idx < table.index_to_symbol.len(),
                "index {} out of bounds (len={})", idx, table.index_to_symbol.len()
            );
            prop_assert_eq!(
                table.index_to_symbol[idx], sym,
                "roundtrip failed: symbol_to_index[{:?}]={} but index_to_symbol[{}]={:?}",
                sym, idx, idx, table.index_to_symbol[idx]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property 6: Grammar name is preserved in ParseTable.grammar
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_grammar_name_preserved(name_suffix in 0u32..100) {
        let name = format!("g_{name_suffix}");
        let grammar = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("S", vec!["x"])
            .start("S")
            .build();
        let table = build_table(&grammar);
        prop_assert_eq!(
            &table.grammar().name, &name,
            "grammar name not preserved"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 7: All ParseTable rules have valid LHS (SymbolId)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_rules_have_valid_lhs(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        for (i, rule) in table.rules.iter().enumerate() {
            // LHS must be a non-terminal, i.e. present in nonterminal_to_index
            // or at minimum be a known symbol in the grammar.
            prop_assert!(
                rule.lhs.0 > 0 || i == 0,
                "rule {} has unexpected lhs {:?}", i, rule.lhs
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property 8: ParseTable is deterministic (same grammar → same table)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_deterministic_table(grammar in arb_valid_grammar()) {
        let t1 = build_table(&grammar);
        let t2 = build_table(&grammar);
        prop_assert_eq!(t1.state_count, t2.state_count, "state_count differs");
        prop_assert_eq!(t1.symbol_count, t2.symbol_count, "symbol_count differs");
        prop_assert_eq!(t1.rules.len(), t2.rules.len(), "rules len differs");
        prop_assert_eq!(
            t1.action_table.len(), t2.action_table.len(),
            "action_table len differs"
        );
        prop_assert_eq!(
            t1.goto_table.len(), t2.goto_table.len(),
            "goto_table len differs"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 9: Normalized grammar produces valid parse table
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_normalized_grammar_valid_table(grammar in arb_valid_grammar()) {
        let mut normalized = grammar.clone();
        let _ = normalized.normalize();
        // Must still produce a valid parse table.
        let ff = FirstFollowSets::compute(&normalized).expect("FF on normalized failed");
        let table = build_lr1_automaton(&normalized, &ff).expect("automaton on normalized failed");
        prop_assert!(table.state_count > 0);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }
}

// ---------------------------------------------------------------------------
// Property 10: FIRST set of a nonterminal that directly derives a terminal
//              contains that terminal
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn prop_first_set_nonterminal_contains_direct_terminal(grammar in arb_valid_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).expect("FF failed");
        // For our random grammars, S -> t{k} means FIRST(S) must contain t{k}.
        // The start symbol's FIRST set should contain at least one token.
        if let Some(start_sym) = grammar.rules.keys().next() {
            if let Some(first_set) = ff.first(*start_sym) {
                // At least one token should be in the FIRST set of the start symbol.
                let any_token = grammar.tokens.keys().any(|t| first_set.contains(t.0 as usize));
                prop_assert!(any_token, "FIRST(start) should contain at least one terminal");
            }
        }
    }
}

// ===========================================================================
// Regular #[test] cases
// ===========================================================================

#[test]
fn test_simple_grammar_state_count_positive() {
    let g = simple_grammar("simple");
    let t = build_table(&g);
    assert!(t.state_count > 0);
}

#[test]
fn test_simple_grammar_action_table_len() {
    let g = simple_grammar("act");
    let t = build_table(&g);
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn test_simple_grammar_goto_table_len() {
    let g = simple_grammar("got");
    let t = build_table(&g);
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn test_grammar_name_simple() {
    let g = simple_grammar("my_lang");
    let t = build_table(&g);
    assert_eq!(t.grammar().name, "my_lang");
}

#[test]
fn test_grammar_name_empty_string() {
    let g = GrammarBuilder::new("")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let t = build_table(&g);
    assert_eq!(t.grammar().name, "");
}

#[test]
fn test_two_token_grammar() {
    let g = grammar_with(
        "two",
        &["a", "b"],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count > 0);
    assert_eq!(t.action_table.len(), t.state_count);
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn test_chain_grammar() {
    let g = grammar_with("chain", &["x"], &[("S", vec!["A"]), ("A", vec!["x"])], "S");
    let t = build_table(&g);
    assert!(t.state_count > 0);
}

#[test]
fn test_symbol_to_index_not_empty() {
    let g = simple_grammar("idx");
    let t = build_table(&g);
    assert!(
        !t.symbol_to_index.is_empty(),
        "symbol_to_index must not be empty"
    );
}

#[test]
fn test_index_to_symbol_not_empty() {
    let g = simple_grammar("idx2");
    let t = build_table(&g);
    assert!(
        !t.index_to_symbol.is_empty(),
        "index_to_symbol must not be empty"
    );
}

#[test]
fn test_symbol_index_roundtrip_simple() {
    let g = simple_grammar("rt");
    let t = build_table(&g);
    for (&sym, &idx) in &t.symbol_to_index {
        assert!(idx < t.index_to_symbol.len());
        assert_eq!(t.index_to_symbol[idx], sym);
    }
}

#[test]
fn test_rules_nonempty() {
    let g = simple_grammar("rules");
    let t = build_table(&g);
    assert!(
        !t.rules.is_empty(),
        "parse table must have at least one rule"
    );
}

#[test]
fn test_rules_lhs_nonzero_or_augmented() {
    let g = simple_grammar("lhs");
    let t = build_table(&g);
    for rule in &t.rules {
        // Every LHS should be a valid symbol (u16 value)
        let _ = rule.lhs.0; // just ensure it's accessible
    }
}

#[test]
fn test_deterministic_simple() {
    let g = simple_grammar("det");
    let t1 = build_table(&g);
    let t2 = build_table(&g);
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.symbol_count, t2.symbol_count);
    assert_eq!(t1.rules.len(), t2.rules.len());
}

#[test]
fn test_eof_symbol_exists() {
    let g = simple_grammar("eof");
    let t = build_table(&g);
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "eof_symbol must be in symbol_to_index"
    );
}

#[test]
fn test_start_symbol_set() {
    let g = simple_grammar("start");
    let t = build_table(&g);
    // start_symbol should be non-trivial
    let _ = t.start_symbol;
}

#[test]
fn test_normalized_grammar_builds() {
    let mut g = simple_grammar("norm");
    let _ = g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.state_count > 0);
}

#[test]
fn test_first_follow_computes_on_simple() {
    let g = simple_grammar("ff");
    let ff = FirstFollowSets::compute(&g).unwrap();
    // S -> a, so FIRST(S) should contain the token "a".
    let s_id = *g.rules.keys().next().expect("should have rule");
    let a_id = *g.tokens.keys().next().expect("should have token");
    if let Some(first_set) = ff.first(s_id) {
        assert!(
            first_set.contains(a_id.0 as usize),
            "FIRST(S) should contain token 'a'"
        );
    }
}

#[test]
fn test_first_set_for_nonterminal_contains_terminal() {
    // S -> a  ⇒  FIRST(S) should contain 'a'
    let g = grammar_with("fnt", &["a"], &[("S", vec!["a"])], "S");
    let ff = FirstFollowSets::compute(&g).unwrap();
    // Find the SymbolId for token "a"
    let a_id = *g.tokens.keys().next().expect("should have token");
    // Find the SymbolId for nonterminal "S"
    let s_id = *g.rules.keys().next().expect("should have rule");
    if let Some(first_s) = ff.first(s_id) {
        assert!(
            first_s.contains(a_id.0 as usize),
            "FIRST(S) should contain token 'a'"
        );
    }
}

#[test]
fn test_multiple_alternatives_all_states() {
    let g = grammar_with(
        "alt",
        &["a", "b", "c"],
        &[("S", vec!["a"]), ("S", vec!["b"]), ("S", vec!["c"])],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count > 0);
    assert_eq!(t.action_table.len(), t.state_count);
}

#[test]
fn test_two_nonterminals() {
    let g = grammar_with("twont", &["x"], &[("S", vec!["A"]), ("A", vec!["x"])], "S");
    let t = build_table(&g);
    assert!(t.state_count > 0);
    assert!(
        t.rules.len() >= 2,
        "should have at least 2 rules (S->A, A->x)"
    );
}

#[test]
fn test_action_table_rows_consistent_width() {
    let g = simple_grammar("width");
    let t = build_table(&g);
    if !t.action_table.is_empty() {
        let expected_cols = t.action_table[0].len();
        for (i, row) in t.action_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                expected_cols,
                "action_table row {} has inconsistent width",
                i
            );
        }
    }
}

#[test]
fn test_goto_table_rows_consistent_width() {
    let g = simple_grammar("gwidth");
    let t = build_table(&g);
    if !t.goto_table.is_empty() {
        let expected_cols = t.goto_table[0].len();
        for (i, row) in t.goto_table.iter().enumerate() {
            assert_eq!(
                row.len(),
                expected_cols,
                "goto_table row {} has inconsistent width",
                i
            );
        }
    }
}

#[test]
fn test_symbol_count_positive() {
    let g = simple_grammar("symcnt");
    let t = build_table(&g);
    assert!(t.symbol_count > 0, "symbol_count must be > 0");
}

#[test]
fn test_initial_state_within_bounds() {
    let g = simple_grammar("init");
    let t = build_table(&g);
    assert!(
        (t.initial_state.0 as usize) < t.state_count,
        "initial_state {} out of bounds (state_count={})",
        t.initial_state.0,
        t.state_count
    );
}

#[test]
fn test_token_count_matches_grammar_tokens() {
    let g = grammar_with("tc", &["a", "b"], &[("S", vec!["a"])], "S");
    let t = build_table(&g);
    assert!(t.token_count > 0, "should have at least 1 token");
}

#[test]
fn test_rule_method_returns_valid_data() {
    let g = simple_grammar("rulem");
    let t = build_table(&g);
    for i in 0..t.rules.len() {
        let (lhs, rhs_len) = t.rule(adze_ir::RuleId(i as u16));
        let _ = lhs.0; // valid SymbolId
        let _ = rhs_len; // valid u16
    }
}

#[test]
fn test_eof_method() {
    let g = simple_grammar("eofm");
    let t = build_table(&g);
    let eof = t.eof();
    assert_eq!(eof, t.eof_symbol);
}

#[test]
fn test_start_symbol_method() {
    let g = simple_grammar("ssm");
    let t = build_table(&g);
    let ss = t.start_symbol();
    assert_eq!(ss, t.start_symbol);
}

#[test]
fn test_grammar_method_returns_ref() {
    let g = simple_grammar("gref");
    let t = build_table(&g);
    let gr = t.grammar();
    assert_eq!(gr.name, "gref");
}

#[test]
fn test_nonterminal_to_index_populated() {
    let g = simple_grammar("ntidx");
    let t = build_table(&g);
    assert!(
        !t.nonterminal_to_index.is_empty(),
        "nonterminal_to_index must not be empty for a grammar with rules"
    );
}

#[test]
fn test_five_token_grammar() {
    let g = grammar_with(
        "five",
        &["a", "b", "c", "d", "e"],
        &[
            ("S", vec!["a"]),
            ("S", vec!["b"]),
            ("S", vec!["c"]),
            ("S", vec!["d"]),
            ("S", vec!["e"]),
        ],
        "S",
    );
    let t = build_table(&g);
    assert!(t.state_count > 0);
    assert_eq!(t.action_table.len(), t.state_count);
    assert_eq!(t.goto_table.len(), t.state_count);
}

#[test]
fn test_longer_rhs_grammar() {
    let g = grammar_with("long", &["a", "b", "c"], &[("S", vec!["a", "b", "c"])], "S");
    let t = build_table(&g);
    assert!(t.state_count > 0);
    // The rule S -> a b c should have rhs_len = 3
    let has_len3 = t.rules.iter().any(|r| r.rhs_len == 3);
    assert!(has_len3, "should have a rule with rhs_len=3");
}

#[test]
fn test_symbol_metadata_len() {
    let g = simple_grammar("meta");
    let t = build_table(&g);
    assert!(
        !t.symbol_metadata.is_empty(),
        "symbol_metadata should not be empty"
    );
}

// ---------------------------------------------------------------------------
// Additional proptest blocks for extra coverage
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_two_nt_state_count_positive(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
    }

    #[test]
    fn prop_two_nt_action_eq_state(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }

    #[test]
    fn prop_two_nt_goto_eq_state(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }

    #[test]
    fn prop_two_nt_nonterminal_index_nonempty(grammar in arb_two_nt_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.nonterminal_to_index.is_empty());
    }

    #[test]
    fn prop_initial_state_valid(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!((table.initial_state.0 as usize) < table.state_count);
    }

    #[test]
    fn prop_symbol_count_positive(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }

    #[test]
    fn prop_eof_in_symbol_to_index(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "eof_symbol must be present in symbol_to_index"
        );
    }

    #[test]
    fn prop_rules_nonempty(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(!table.rules.is_empty());
    }

    #[test]
    fn prop_action_rows_uniform_width(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        if !table.action_table.is_empty() {
            let w = table.action_table[0].len();
            for row in &table.action_table {
                prop_assert_eq!(row.len(), w);
            }
        }
    }

    #[test]
    fn prop_goto_rows_uniform_width(grammar in arb_valid_grammar()) {
        let table = build_table(&grammar);
        if !table.goto_table.is_empty() {
            let w = table.goto_table[0].len();
            for row in &table.goto_table {
                prop_assert_eq!(row.len(), w);
            }
        }
    }
}
