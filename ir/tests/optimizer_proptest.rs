use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::GrammarOptimizer;
use proptest::prelude::*;

/// Build a grammar from generated names using GrammarBuilder.
///
/// Tokens are named `t_<idx>_<name>` and rules are named `r_<idx>_<name>`.
/// Each rule uses at least two tokens in its RHS to avoid unit-rule elimination.
/// The start rule additionally references subsequent rules to keep them reachable.
fn build_grammar(
    grammar_name: &str,
    token_names: &[String],
    rule_names: &[String],
    external_names: &[String],
) -> Grammar {
    let mut builder = GrammarBuilder::new(grammar_name);

    // Unique token names (index-prefixed to avoid collisions from duplicate generated names)
    let tok_ids: Vec<String> = token_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("t_{i}_{n}"))
        .collect();
    for tok in &tok_ids {
        builder = builder.token(tok, tok);
    }

    // External tokens
    for (i, ename) in external_names.iter().enumerate() {
        let ext = format!("e_{i}_{ename}");
        builder = builder.external(&ext);
    }

    // Unique rule names
    let rule_ids: Vec<String> = rule_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("r_{i}_{n}"))
        .collect();

    // Build rules with at least two RHS symbols so they are not trivial unit rules.
    let first_tok = &tok_ids[0];
    let second_tok = if tok_ids.len() > 1 {
        &tok_ids[1]
    } else {
        &tok_ids[0]
    };

    for (i, rname) in rule_ids.iter().enumerate() {
        if i == 0 && rule_ids.len() > 1 {
            // Start rule references the second rule and a token
            builder = builder.rule(rname, vec![first_tok, &rule_ids[1]]);
        } else {
            // Other rules use two tokens to avoid being unit rules
            builder = builder.rule(rname, vec![first_tok, second_tok]);
        }
    }

    // Start symbol is the first rule
    builder = builder.start(&rule_ids[0]);

    builder.build()
}

/// Count total symbols: distinct rule LHS entries + token entries.
fn total_symbol_count(g: &Grammar) -> usize {
    g.rules.len() + g.tokens.len()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // ---------------------------------------------------------------
    // 1. Optimization never increases total symbol count
    // ---------------------------------------------------------------
    #[test]
    fn optimization_never_increases_symbol_count(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let grammar = build_grammar(&grammar_name, &token_names, &rule_names, &[]);
        let before = total_symbol_count(&grammar);

        let mut optimized = grammar;
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut optimized);

        let after = total_symbol_count(&optimized);
        prop_assert!(
            after <= before,
            "symbol count increased: {before} -> {after}"
        );
    }

    // ---------------------------------------------------------------
    // 2. Optimization never removes the start symbol
    // ---------------------------------------------------------------
    #[test]
    fn optimization_preserves_start_symbol(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let grammar = build_grammar(&grammar_name, &token_names, &rule_names, &[]);
        let start_before = grammar.start_symbol();
        prop_assert!(start_before.is_some(), "grammar has no start symbol before optimization");

        let mut optimized = grammar;
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut optimized);

        let start_after = optimized.start_symbol();
        prop_assert!(
            start_after.is_some(),
            "start symbol was removed by optimization"
        );
    }

    // ---------------------------------------------------------------
    // 3. Optimization preserves token count
    // ---------------------------------------------------------------
    #[test]
    fn optimization_preserves_token_count(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let grammar = build_grammar(&grammar_name, &token_names, &rule_names, &[]);
        let tokens_before = grammar.tokens.len();

        let mut optimized = grammar;
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut optimized);

        let tokens_after = optimized.tokens.len();
        prop_assert!(
            tokens_after <= tokens_before,
            "token count increased: {tokens_before} -> {tokens_after}"
        );
    }

    // ---------------------------------------------------------------
    // 4. Optimization preserves external token count
    // ---------------------------------------------------------------
    #[test]
    fn optimization_preserves_external_token_count(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        external_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let grammar = build_grammar(&grammar_name, &token_names, &rule_names, &external_names);
        let ext_before = grammar.externals.len();

        let mut optimized = grammar;
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut optimized);

        let ext_after = optimized.externals.len();
        prop_assert_eq!(
            ext_after,
            ext_before,
            "external token count changed: {} -> {}",
            ext_before, ext_after
        );
    }

    // ---------------------------------------------------------------
    // 5. Optimization is idempotent
    // ---------------------------------------------------------------
    #[test]
    fn optimization_is_idempotent(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let grammar = build_grammar(&grammar_name, &token_names, &rule_names, &[]);

        // First optimization pass
        let mut once = grammar;
        let mut opt1 = GrammarOptimizer::new();
        opt1.optimize(&mut once);
        let json_once = serde_json::to_string(&once).unwrap();

        // Second optimization pass on already-optimized grammar
        let mut twice = once;
        let mut opt2 = GrammarOptimizer::new();
        opt2.optimize(&mut twice);
        let json_twice = serde_json::to_string(&twice).unwrap();

        prop_assert_eq!(
            json_once,
            json_twice,
            "optimization is not idempotent"
        );
    }

    // ---------------------------------------------------------------
    // 6. OptimizationStats fields are non-negative
    // ---------------------------------------------------------------
    #[test]
    fn optimization_stats_fields_are_nonnegative(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let mut grammar = build_grammar(&grammar_name, &token_names, &rule_names, &[]);
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut grammar);

        // All fields are usize so they can't be negative, but verify
        // total() equals the sum (no underflow or mismatch).
        let expected = stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules;
        let total = stats.total();
        prop_assert_eq!(
            total,
            expected,
            "stats.total() ({}) != sum of fields ({})",
            total, expected
        );
    }

    // ---------------------------------------------------------------
    // 7. Optimization preserves grammar name
    // ---------------------------------------------------------------
    #[test]
    fn optimization_preserves_grammar_name(
        grammar_name in "[a-z]{1,8}",
        token_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
        rule_names in prop::collection::vec("[a-z]{1,8}", 1..5usize),
    ) {
        let grammar = build_grammar(&grammar_name, &token_names, &rule_names, &[]);
        let name_before = grammar.name.clone();

        let mut optimized = grammar;
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut optimized);

        prop_assert_eq!(
            optimized.name,
            name_before,
            "optimizer changed grammar name"
        );
    }
}
