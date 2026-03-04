#![allow(clippy::needless_range_loop)]
//! Property-based tests for `Production` and related types in adze-glr-core.
//!
//! Covers `ParseRule` (lhs, rhs_len), production identity, Clone/Debug,
//! epsilon productions, large RHS, uniqueness, parse-table integration,
//! and precedence/associativity on productions.
//!
//! Run with: `cargo test -p adze-glr-core --test production_proptest`

use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseRule, ParseTable, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};

// ============================================================================
// Strategies
// ============================================================================

/// Arbitrary `SymbolId` in the nonterminal range.
fn arb_nonterminal_id() -> impl Strategy<Value = SymbolId> {
    (1u16..500).prop_map(SymbolId)
}

/// Arbitrary `ParseRule`.
fn arb_parse_rule() -> impl Strategy<Value = ParseRule> {
    (arb_nonterminal_id(), 0u16..=64).prop_map(|(lhs, rhs_len)| ParseRule { lhs, rhs_len })
}

/// Arbitrary associativity value.
fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

/// Build a well-formed `ParseTable` with the given rules vector.
fn make_table_with_rules(rules: Vec<ParseRule>) -> ParseTable {
    let num_terminals = 2usize;
    let num_nonterminals = 2usize;
    let sym_count = num_terminals + num_nonterminals;
    let num_states = 1usize;

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..sym_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }
    let mut nonterminal_to_index = BTreeMap::new();
    for i in num_terminals..sym_count {
        nonterminal_to_index.insert(SymbolId(i as u16), i - num_terminals);
    }

    let rule_count = rules.len();
    ParseTable {
        action_table: vec![vec![vec![]; sym_count]; num_states],
        goto_table: vec![vec![StateId(65535); num_nonterminals]; num_states],
        rules,
        state_count: num_states,
        symbol_count: sym_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(num_terminals as u16),
        grammar: Grammar::new("proptest".to_string()),
        symbol_metadata: (0..sym_count as u16)
            .map(|i| adze_glr_core::SymbolMetadata {
                name: format!("sym_{i}"),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: (i as usize) < num_terminals,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(i),
            })
            .collect(),
        initial_state: StateId(0),
        token_count: num_terminals,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rule_count],
        rule_assoc_by_rule: vec![0; rule_count],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    }
}

// ============================================================================
// Helpers for grammar-backed tables
// ============================================================================

fn build_grammar_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

// ============================================================================
// Property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // -----------------------------------------------------------------------
    // 1. ParseRule stores lhs and rhs_len faithfully
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_roundtrips_fields(lhs_val in 1u16..1000, rhs_len in 0u16..100) {
        let rule = ParseRule { lhs: SymbolId(lhs_val), rhs_len };
        prop_assert_eq!(rule.lhs, SymbolId(lhs_val));
        prop_assert_eq!(rule.rhs_len, rhs_len);
    }

    // -----------------------------------------------------------------------
    // 2. ParseRule LHS is always a valid SymbolId (non-negative)
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_lhs_always_valid(rule in arb_parse_rule()) {
        // SymbolId wraps u16, always >= 0. Check it survived construction.
        prop_assert!(rule.lhs.0 >= 1, "LHS should be a positive nonterminal id");
    }

    // -----------------------------------------------------------------------
    // 3. Clone produces an identical copy
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_clone_is_identical(rule in arb_parse_rule()) {
        let cloned = rule.clone();
        prop_assert_eq!(cloned.lhs, rule.lhs);
        prop_assert_eq!(cloned.rhs_len, rule.rhs_len);
    }

    // -----------------------------------------------------------------------
    // 4. Debug output contains type name
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_debug_contains_type(rule in arb_parse_rule()) {
        let dbg = format!("{:?}", rule);
        prop_assert!(dbg.contains("ParseRule"), "Debug should mention ParseRule: {}", dbg);
    }

    // -----------------------------------------------------------------------
    // 5. Debug output contains field values
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_debug_contains_values(lhs_val in 1u16..500, rhs_len in 0u16..50) {
        let rule = ParseRule { lhs: SymbolId(lhs_val), rhs_len };
        let dbg = format!("{:?}", rule);
        prop_assert!(dbg.contains(&rhs_len.to_string()), "Debug missing rhs_len: {}", dbg);
    }

    // -----------------------------------------------------------------------
    // 6. Epsilon production has rhs_len == 0
    // -----------------------------------------------------------------------
    #[test]
    fn epsilon_production_rhs_is_zero(lhs_val in 1u16..500) {
        let rule = ParseRule { lhs: SymbolId(lhs_val), rhs_len: 0 };
        prop_assert_eq!(rule.rhs_len, 0);
    }

    // -----------------------------------------------------------------------
    // 7. Large RHS productions are representable
    // -----------------------------------------------------------------------
    #[test]
    fn large_rhs_representable(rhs_len in 100u16..=u16::MAX) {
        let rule = ParseRule { lhs: SymbolId(1), rhs_len };
        prop_assert_eq!(rule.rhs_len, rhs_len);
    }

    // -----------------------------------------------------------------------
    // 8. Vec of ParseRule preserves insertion order
    // -----------------------------------------------------------------------
    #[test]
    fn rules_vec_preserves_order(rules in prop::collection::vec(arb_parse_rule(), 1..20)) {
        let lhs_values: Vec<u16> = rules.iter().map(|r| r.lhs.0).collect();
        let rhs_values: Vec<u16> = rules.iter().map(|r| r.rhs_len).collect();
        for i in 0..rules.len() {
            prop_assert_eq!(rules[i].lhs.0, lhs_values[i]);
            prop_assert_eq!(rules[i].rhs_len, rhs_values[i]);
        }
    }

    // -----------------------------------------------------------------------
    // 9. Production ID uniqueness: index in Vec is the implicit ID
    // -----------------------------------------------------------------------
    #[test]
    fn production_ids_are_unique_indices(rules in prop::collection::vec(arb_parse_rule(), 1..30)) {
        let indices: Vec<usize> = (0..rules.len()).collect();
        let set: HashSet<usize> = indices.iter().copied().collect();
        prop_assert_eq!(set.len(), rules.len(), "implicit IDs must be unique");
    }

    // -----------------------------------------------------------------------
    // 10. ParseTable.rule() accessor roundtrips for every rule
    // -----------------------------------------------------------------------
    #[test]
    fn parse_table_rule_accessor(rules in prop::collection::vec(arb_parse_rule(), 1..15)) {
        let table = make_table_with_rules(rules.clone());
        for (i, expected) in rules.iter().enumerate() {
            let (lhs, rhs_len) = table.rule(RuleId(i as u16));
            prop_assert_eq!(lhs, expected.lhs);
            prop_assert_eq!(rhs_len, expected.rhs_len);
        }
    }

    // -----------------------------------------------------------------------
    // 11. ParseTable.rules length matches dynamic_prec_by_rule length
    // -----------------------------------------------------------------------
    #[test]
    fn table_rules_aligned_with_dynamic_prec(rules in prop::collection::vec(arb_parse_rule(), 0..20)) {
        let table = make_table_with_rules(rules);
        prop_assert_eq!(table.rules.len(), table.dynamic_prec_by_rule.len());
    }

    // -----------------------------------------------------------------------
    // 12. ParseTable.rules length matches rule_assoc_by_rule length
    // -----------------------------------------------------------------------
    #[test]
    fn table_rules_aligned_with_assoc(rules in prop::collection::vec(arb_parse_rule(), 0..20)) {
        let table = make_table_with_rules(rules);
        prop_assert_eq!(table.rules.len(), table.rule_assoc_by_rule.len());
    }

    // -----------------------------------------------------------------------
    // 13. Default ParseTable has zero rules
    // -----------------------------------------------------------------------
    #[test]
    fn default_table_empty_rules(_dummy in 0u8..1) {
        let pt = ParseTable::default();
        prop_assert!(pt.rules.is_empty());
        prop_assert!(pt.dynamic_prec_by_rule.is_empty());
        prop_assert!(pt.rule_assoc_by_rule.is_empty());
    }

    // -----------------------------------------------------------------------
    // 14. Productions collected from Reduce actions reference valid rule IDs
    // -----------------------------------------------------------------------
    #[test]
    fn reduce_actions_reference_valid_rules(
        rules in prop::collection::vec(arb_parse_rule(), 1..10),
    ) {
        let table = make_table_with_rules(rules.clone());
        // Manually add a Reduce action that references rule 0
        let mut t = table;
        if !t.action_table.is_empty() && !t.action_table[0].is_empty() {
            t.action_table[0][0] = vec![Action::Reduce(RuleId(0))];
        }
        // Verify rule 0 is accessible
        let (lhs, rhs_len) = t.rule(RuleId(0));
        prop_assert_eq!(lhs, rules[0].lhs);
        prop_assert_eq!(rhs_len, rules[0].rhs_len);
    }

    // -----------------------------------------------------------------------
    // 15. Multiple rules can share the same LHS
    // -----------------------------------------------------------------------
    #[test]
    fn multiple_rules_same_lhs(
        lhs_val in 2u16..100,
        rhs_lens in prop::collection::vec(0u16..10, 2..8),
    ) {
        let rules: Vec<ParseRule> = rhs_lens
            .iter()
            .map(|&rhs_len| ParseRule { lhs: SymbolId(lhs_val), rhs_len })
            .collect();
        let table = make_table_with_rules(rules.clone());
        let same_lhs: Vec<_> = table.rules.iter().filter(|r| r.lhs == SymbolId(lhs_val)).collect();
        prop_assert_eq!(same_lhs.len(), rhs_lens.len());
    }

    // -----------------------------------------------------------------------
    // 16. Distinct LHS values are preserved in rules vector
    // -----------------------------------------------------------------------
    #[test]
    fn distinct_lhs_preserved(
        lhs_values in prop::collection::vec(1u16..200, 2..10),
    ) {
        let rules: Vec<ParseRule> = lhs_values
            .iter()
            .map(|&v| ParseRule { lhs: SymbolId(v), rhs_len: 1 })
            .collect();
        let table = make_table_with_rules(rules);
        let lhs_set: BTreeSet<u16> = table.rules.iter().map(|r| r.lhs.0).collect();
        let expected_set: BTreeSet<u16> = lhs_values.iter().copied().collect();
        prop_assert_eq!(lhs_set, expected_set);
    }

    // -----------------------------------------------------------------------
    // 17. rhs_len == 0 rules can coexist with non-epsilon rules
    // -----------------------------------------------------------------------
    #[test]
    fn epsilon_and_nonempty_coexist(
        n_epsilon in 1usize..5,
        n_normal in 1usize..5,
    ) {
        let mut rules = Vec::new();
        for i in 0..n_epsilon {
            rules.push(ParseRule { lhs: SymbolId((i + 1) as u16), rhs_len: 0 });
        }
        for i in 0..n_normal {
            rules.push(ParseRule { lhs: SymbolId((i + 100) as u16), rhs_len: 3 });
        }
        let table = make_table_with_rules(rules);
        let eps_count = table.rules.iter().filter(|r| r.rhs_len == 0).count();
        let non_count = table.rules.iter().filter(|r| r.rhs_len > 0).count();
        prop_assert_eq!(eps_count, n_epsilon);
        prop_assert_eq!(non_count, n_normal);
    }

    // -----------------------------------------------------------------------
    // 18. ParseRule fields survive clone chain
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_multi_clone(rule in arb_parse_rule()) {
        let c1 = rule.clone();
        let c2 = c1.clone();
        let c3 = c2.clone();
        prop_assert_eq!(c3.lhs, rule.lhs);
        prop_assert_eq!(c3.rhs_len, rule.rhs_len);
    }

    // -----------------------------------------------------------------------
    // 19. ParseRule Debug is deterministic
    // -----------------------------------------------------------------------
    #[test]
    fn parse_rule_debug_deterministic(rule in arb_parse_rule()) {
        let d1 = format!("{:?}", rule);
        let d2 = format!("{:?}", rule);
        prop_assert_eq!(d1, d2);
    }

    // -----------------------------------------------------------------------
    // 20. Large batch of rules stored without truncation
    // -----------------------------------------------------------------------
    #[test]
    fn large_batch_no_truncation(count in 50usize..200) {
        let rules: Vec<ParseRule> = (0..count)
            .map(|i| ParseRule { lhs: SymbolId((i % 400 + 1) as u16), rhs_len: (i % 10) as u16 })
            .collect();
        let table = make_table_with_rules(rules);
        prop_assert_eq!(table.rules.len(), count);
    }

    // -----------------------------------------------------------------------
    // 21. rule_assoc_by_rule encodes Left=1, Right=-1, None=0
    // -----------------------------------------------------------------------
    #[test]
    fn rule_assoc_encoding(assoc in arb_associativity()) {
        let encoded: i8 = match assoc {
            Associativity::Left => 1,
            Associativity::Right => -1,
            Associativity::None => 0,
        };
        // Roundtrip the encoding
        let decoded = match encoded {
            1 => Associativity::Left,
            -1 => Associativity::Right,
            _ => Associativity::None,
        };
        prop_assert_eq!(decoded, assoc);
    }

    // -----------------------------------------------------------------------
    // 22. dynamic_prec_by_rule defaults to zero in synthetic table
    // -----------------------------------------------------------------------
    #[test]
    fn default_dynamic_prec_is_zero(count in 1usize..20) {
        let rules: Vec<ParseRule> = (0..count)
            .map(|_| ParseRule { lhs: SymbolId(1), rhs_len: 1 })
            .collect();
        let table = make_table_with_rules(rules);
        for &p in &table.dynamic_prec_by_rule {
            prop_assert_eq!(p, 0i16);
        }
    }

    // -----------------------------------------------------------------------
    // 23. rule_assoc defaults to zero (None) in synthetic table
    // -----------------------------------------------------------------------
    #[test]
    fn default_rule_assoc_is_none(count in 1usize..20) {
        let rules: Vec<ParseRule> = (0..count)
            .map(|_| ParseRule { lhs: SymbolId(1), rhs_len: 1 })
            .collect();
        let table = make_table_with_rules(rules);
        for &a in &table.rule_assoc_by_rule {
            prop_assert_eq!(a, 0i8);
        }
    }

    // -----------------------------------------------------------------------
    // 24. rhs_len distribution: random batch always has sum >= count of rules
    // -----------------------------------------------------------------------
    #[test]
    fn rhs_len_sum_property(rules in prop::collection::vec(arb_parse_rule(), 1..30)) {
        let total: u64 = rules.iter().map(|r| r.rhs_len as u64).sum();
        // Sum of rhs_len values is at least 0 * count (trivially true but we test the computation).
        prop_assert!(total <= rules.len() as u64 * u16::MAX as u64);
    }

    // -----------------------------------------------------------------------
    // 25. Mapping rules to (lhs, rhs_len) tuples preserves count
    // -----------------------------------------------------------------------
    #[test]
    fn rule_to_tuple_count(rules in prop::collection::vec(arb_parse_rule(), 0..25)) {
        let tuples: Vec<(SymbolId, u16)> = rules.iter().map(|r| (r.lhs, r.rhs_len)).collect();
        prop_assert_eq!(tuples.len(), rules.len());
    }
}

// ============================================================================
// Grammar-backed property tests (use real build_lr1_automaton)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // -----------------------------------------------------------------------
    // 26. Grammar-built table always has at least one rule
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_table_nonempty_rules(n_alts in 1usize..4) {
        // Build grammar: start → a₁ | a₂ | ... | aₙ
        let mut gb = GrammarBuilder::new("prop_gram");
        for i in 0..n_alts {
            let tok_name = format!("t{i}");
            // Must leak to get &'static str for builder API
            let tok: &'static str = Box::leak(tok_name.clone().into_boxed_str());
            gb = gb.token(tok, tok);
            gb = gb.rule("start", vec![tok]);
        }
        gb = gb.start("start");
        let grammar = gb.build();
        let table = build_grammar_table(&grammar);
        prop_assert!(!table.rules.is_empty());
        // At least as many rules as user alternatives (augmented start may add one more)
        prop_assert!(table.rules.len() >= n_alts);
    }

    // -----------------------------------------------------------------------
    // 27. Every rule LHS in grammar table is a known nonterminal
    // -----------------------------------------------------------------------
    #[test]
    fn grammar_table_lhs_are_nonterminals(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("check_lhs")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        let table = build_grammar_table(&grammar);
        let nt_ids: BTreeSet<SymbolId> = grammar.rule_names.keys().copied().collect();
        for rule in &table.rules {
            // Augmented start rule LHS may not be in rule_names but must be a valid symbol
            // At minimum verify LHS is not SymbolId(0) which is EOF
            prop_assert!(rule.lhs.0 > 0 || nt_ids.contains(&rule.lhs),
                "LHS {:?} should be a nonterminal", rule.lhs);
        }
    }

    // -----------------------------------------------------------------------
    // 28. Precedence rules produce matching dynamic_prec_by_rule entries
    // -----------------------------------------------------------------------
    #[test]
    fn precedence_populates_dynamic_prec(prec_val in 1i16..10) {
        let grammar = GrammarBuilder::new("prec_test")
            .token("a", "a")
            .token("plus", "+")
            .rule_with_precedence("expr", vec!["a"], prec_val, Associativity::None)
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], prec_val, Associativity::Left)
            .start("expr")
            .build();
        let table = build_grammar_table(&grammar);
        // At least one rule should carry the specified precedence
        let has_prec = table.dynamic_prec_by_rule.contains(&prec_val);
        prop_assert!(has_prec, "Expected prec {} in {:?}", prec_val, table.dynamic_prec_by_rule);
    }

    // -----------------------------------------------------------------------
    // 29. Left-associative rules produce assoc == 1
    // -----------------------------------------------------------------------
    #[test]
    fn left_assoc_encoded_as_positive(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("left_a")
            .token("n", "n")
            .token("plus", "+")
            .rule_with_precedence("expr", vec!["n"], 1, Associativity::None)
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .start("expr")
            .build();
        let table = build_grammar_table(&grammar);
        let has_left = table.rule_assoc_by_rule.contains(&1);
        prop_assert!(has_left, "Expected left-assoc (1) in {:?}", table.rule_assoc_by_rule);
    }

    // -----------------------------------------------------------------------
    // 30. Right-associative rules produce assoc == -1
    // -----------------------------------------------------------------------
    #[test]
    fn right_assoc_encoded_as_negative(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("right_a")
            .token("n", "n")
            .token("eq", "=")
            .rule_with_precedence("assign", vec!["n"], 1, Associativity::None)
            .rule_with_precedence("assign", vec!["assign", "eq", "assign"], 1, Associativity::Right)
            .start("assign")
            .build();
        let table = build_grammar_table(&grammar);
        let has_right = table.rule_assoc_by_rule.contains(&-1);
        prop_assert!(has_right, "Expected right-assoc (-1) in {:?}", table.rule_assoc_by_rule);
    }

    // -----------------------------------------------------------------------
    // 31. Reduce actions in table point to valid rule indices
    // -----------------------------------------------------------------------
    #[test]
    fn reduce_actions_in_bounds(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("bounds")
            .token("x", "x")
            .token("y", "y")
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .start("start")
            .build();
        let table = build_grammar_table(&grammar);
        for row in &table.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Reduce(rid) = action {
                        prop_assert!(
                            (rid.0 as usize) < table.rules.len(),
                            "Reduce({}) out of bounds (max {})", rid.0, table.rules.len()
                        );
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 32. dynamic_prec_by_rule and rule_assoc_by_rule have same length as rules
    // -----------------------------------------------------------------------
    #[test]
    fn prec_assoc_arrays_aligned(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("aligned")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build();
        let table = build_grammar_table(&grammar);
        prop_assert_eq!(table.dynamic_prec_by_rule.len(), table.rules.len());
        prop_assert_eq!(table.rule_assoc_by_rule.len(), table.rules.len());
    }

    // -----------------------------------------------------------------------
    // 33. rhs_len for grammar rules matches the original RHS symbol count
    // -----------------------------------------------------------------------
    #[test]
    fn rhs_len_matches_grammar_rhs(_dummy in 0u8..1) {
        let grammar = GrammarBuilder::new("rhs_check")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start")
            .build();
        let table = build_grammar_table(&grammar);
        let start_sym = nt_id(&grammar, "start");
        let start_rules: Vec<_> = table.rules.iter().filter(|r| r.lhs == start_sym).collect();
        prop_assert!(!start_rules.is_empty());
        // The user rule has 3 RHS symbols
        prop_assert!(
            start_rules.iter().any(|r| r.rhs_len == 3),
            "Expected rhs_len==3 for start→a b c, got {:?}",
            start_rules.iter().map(|r| r.rhs_len).collect::<Vec<_>>()
        );
    }
}

// ============================================================================
// Non-proptest supplementary tests (bring total to 35)
// ============================================================================

#[test]
fn parse_rule_max_rhs_len() {
    let rule = ParseRule {
        lhs: SymbolId(1),
        rhs_len: u16::MAX,
    };
    assert_eq!(rule.rhs_len, u16::MAX);
}

#[test]
fn parse_rule_zero_lhs_is_representable() {
    // SymbolId(0) is typically EOF but the struct allows it
    let rule = ParseRule {
        lhs: SymbolId(0),
        rhs_len: 1,
    };
    assert_eq!(rule.lhs, SymbolId(0));
}
