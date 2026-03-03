#![allow(clippy::needless_range_loop)]

//! Property-based tests for individual grammar optimization passes in adze-ir.
//!
//! Each test targets a specific pass (remove_unused_symbols, inline_simple_rules,
//! merge_equivalent_tokens, optimize_left_recursion, eliminate_unit_rules,
//! renumber_symbols) and verifies its invariants in isolation and in combination.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::{
    Associativity, ExternalToken, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
    Token, TokenPattern,
};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar with the builder API. Each rule has a two-symbol RHS so
/// it is never a trivial unit rule. The first rule references the second to
/// keep it reachable.
fn make_grammar(
    name: &str,
    tok_names: &[String],
    rule_names: &[String],
    ext_names: &[String],
) -> Grammar {
    let mut builder = GrammarBuilder::new(name);

    let toks: Vec<String> = tok_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("tok_{i}_{n}"))
        .collect();
    for t in &toks {
        builder = builder.token(t, t);
    }

    for (i, en) in ext_names.iter().enumerate() {
        builder = builder.external(&format!("ext_{i}_{en}"));
    }

    let rules: Vec<String> = rule_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("rl_{i}_{n}"))
        .collect();

    let first_tok = &toks[0];
    let second_tok = if toks.len() > 1 { &toks[1] } else { &toks[0] };

    for (i, rn) in rules.iter().enumerate() {
        if i == 0 && rules.len() > 1 {
            builder = builder.rule(rn, vec![first_tok, &rules[1]]);
        } else {
            builder = builder.rule(rn, vec![first_tok, second_tok]);
        }
    }

    builder = builder.start(&rules[0]);
    builder.build()
}

/// Build a grammar containing duplicate tokens (same pattern, different names).
fn make_dup_token_grammar(name: &str, dup_count: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // One canonical token.
    let canon = SymbolId(1);
    grammar.tokens.insert(
        canon,
        Token {
            name: "tok_canon".to_string(),
            pattern: TokenPattern::String("dup_pattern".to_string()),
            fragile: false,
        },
    );

    // Duplicate tokens sharing the same pattern.
    for i in 0..dup_count {
        let id = SymbolId((i as u16) + 10);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("tok_dup_{i}"),
                pattern: TokenPattern::String("dup_pattern".to_string()),
                fragile: false,
            },
        );
    }

    let start = SymbolId(100);
    grammar.rule_names.insert(start, "start".to_string());
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(canon), Symbol::Terminal(canon)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar
}

/// Build a grammar with a unit rule chain: start -> mid, mid -> tok tok.
fn make_unit_rule_grammar(name: &str, chain_len: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    let mut prev_id = None;
    let mut prod = 0u16;
    for i in 0..=chain_len {
        let id = SymbolId((i as u16) + 50);
        let rname = format!("r{i}");
        grammar.rule_names.insert(id, rname);

        if i == chain_len {
            // Terminal production at the end of the chain.
            grammar.add_rule(Rule {
                lhs: id,
                rhs: vec![Symbol::Terminal(tok), Symbol::Terminal(tok)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod),
            });
        } else {
            // Unit rule pointing to the next symbol.
            let next = SymbolId(((i + 1) as u16) + 50);
            grammar.add_rule(Rule {
                lhs: id,
                rhs: vec![Symbol::NonTerminal(next)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod),
            });
        }
        prod += 1;
        if i == 0 {
            prev_id = Some(id);
        }
    }

    // Ensure the first rule key is the start symbol.
    if let Some(start_id) = prev_id {
        let mut ordered = indexmap::IndexMap::new();
        if let Some(rules) = grammar.rules.shift_remove(&start_id) {
            ordered.insert(start_id, rules);
        }
        for (k, v) in grammar.rules.drain(..) {
            ordered.insert(k, v);
        }
        grammar.rules = ordered;
    }

    grammar
}

/// Build a left-recursive grammar: expr -> expr op number | number.
fn make_left_rec_grammar(name: &str, op_count: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    let num = SymbolId(1);
    grammar.tokens.insert(
        num,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let mut ops = Vec::new();
    for i in 0..op_count.max(1) {
        let id = SymbolId((i as u16) + 2);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("op{i}"),
                pattern: TokenPattern::String(format!("op{i}")),
                fragile: false,
            },
        );
        ops.push(id);
    }

    let expr = SymbolId(100);
    grammar.rule_names.insert(expr, "expr".to_string());

    let mut pid = 0u16;
    for op in &ops {
        grammar.add_rule(Rule {
            lhs: expr,
            rhs: vec![
                Symbol::NonTerminal(expr),
                Symbol::Terminal(*op),
                Symbol::Terminal(num),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        });
        pid += 1;
    }

    // Base case.
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(pid),
    });

    grammar
}

/// Collect every terminal SymbolId referenced inside any rule RHS.
fn collect_terminal_refs(g: &Grammar) -> HashSet<SymbolId> {
    let mut set = HashSet::new();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            collect_from_symbol(sym, &mut set);
        }
    }
    set
}

fn collect_from_symbol(sym: &Symbol, out: &mut HashSet<SymbolId>) {
    match sym {
        Symbol::Terminal(id) => {
            out.insert(*id);
        }
        Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon => {}
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            collect_from_symbol(inner, out);
        }
        Symbol::Choice(choices) => {
            for s in choices {
                collect_from_symbol(s, out);
            }
        }
        Symbol::Sequence(seq) => {
            for s in seq {
                collect_from_symbol(s, out);
            }
        }
    }
}

/// Count total productions across all rule groups.
fn total_productions(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn name_strat() -> impl Strategy<Value = String> {
    "[a-z]{1,6}"
}

fn tok_strat() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strat(), 1..5usize)
}

fn rule_strat() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strat(), 1..5usize)
}

fn ext_strat() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strat(), 0..3usize)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    // ===================================================================
    // 1–5: Empty / trivial grammar behaviour
    // ===================================================================

    /// 1. Optimizing an empty grammar produces an empty grammar without panic.
    #[test]
    fn pass_empty_grammar_no_panic(gn in name_strat()) {
        let g = Grammar::new(gn);
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(opt.rules.is_empty());
        prop_assert!(opt.tokens.is_empty());
    }

    /// 2. Stats for an empty grammar are all zero.
    #[test]
    fn pass_empty_grammar_stats_zero(gn in name_strat()) {
        let mut g = Grammar::new(gn);
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut g);
        prop_assert_eq!(stats.total(), 0);
    }

    /// 3. Minimal single-rule grammar survives all passes.
    #[test]
    fn pass_minimal_grammar_survives(gn in name_strat()) {
        let g = GrammarBuilder::new(&gn)
            .token("x", "x")
            .rule("s", vec!["x", "x"])
            .start("s")
            .build();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(!opt.rules.is_empty(), "rules removed from minimal grammar");
        prop_assert!(!opt.tokens.is_empty(), "tokens removed from minimal grammar");
    }

    /// 4. Already-optimal grammar (no duplicates, no unused, no unit rules)
    /// has idempotent serialization even on first pass.
    #[test]
    fn pass_already_optimal_idempotent(gn in name_strat()) {
        let g = GrammarBuilder::new(&gn)
            .token("a", "a")
            .token("b", "b")
            .rule("root", vec!["a", "b"])
            .start("root")
            .build();
        let once = optimize_grammar(g).unwrap();
        let json1 = serde_json::to_string(&once).unwrap();
        let twice = optimize_grammar(once).unwrap();
        let json2 = serde_json::to_string(&twice).unwrap();
        prop_assert_eq!(json1, json2);
    }

    /// 5. Already-optimal grammar yields zero inlined rules and merged tokens.
    #[test]
    fn pass_already_optimal_stats(gn in name_strat()) {
        let mut g = GrammarBuilder::new(&gn)
            .token("a", "a")
            .token("b", "b")
            .rule("root", vec!["a", "b"])
            .start("root")
            .build();
        let mut opt = GrammarOptimizer::new();
        let stats = opt.optimize(&mut g);
        prop_assert_eq!(stats.inlined_rules, 0);
        prop_assert_eq!(stats.merged_tokens, 0);
        prop_assert_eq!(stats.optimized_left_recursion, 0);
    }

    // ===================================================================
    // 6–10: merge_equivalent_tokens pass
    // ===================================================================

    /// 6. Duplicate tokens are merged so token count drops.
    #[test]
    fn pass_merge_reduces_dup_tokens(
        gn in name_strat(),
        dups in 1usize..5,
    ) {
        let g = make_dup_token_grammar(&gn, dups);
        let before = g.tokens.len();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(
            opt.tokens.len() < before,
            "expected fewer tokens: before={}, after={}",
            before,
            opt.tokens.len()
        );
    }

    /// 7. After merging, no two tokens share the same pattern string.
    #[test]
    fn pass_merge_no_dup_patterns(
        gn in name_strat(),
        dups in 1usize..5,
    ) {
        let g = make_dup_token_grammar(&gn, dups);
        let opt = optimize_grammar(g).unwrap();
        let patterns: HashSet<String> = opt
            .tokens
            .values()
            .map(|t| match &t.pattern {
                TokenPattern::String(s) => s.clone(),
                TokenPattern::Regex(r) => r.clone(),
            })
            .collect();
        prop_assert_eq!(patterns.len(), opt.tokens.len());
    }

    /// 8. Merging tokens preserves at least one token with the duplicated pattern.
    #[test]
    fn pass_merge_preserves_canonical_pattern(
        gn in name_strat(),
        dups in 1usize..5,
    ) {
        let g = make_dup_token_grammar(&gn, dups);
        let opt = optimize_grammar(g).unwrap();
        let has_pattern = opt.tokens.values().any(|t| {
            matches!(&t.pattern, TokenPattern::String(s) if s == "dup_pattern")
        });
        prop_assert!(has_pattern, "canonical pattern lost after merge");
    }

    /// 9. Total optimisations are positive for a grammar with duplicate tokens.
    #[test]
    fn pass_merge_stats_positive_for_dups(
        gn in name_strat(),
        dups in 1usize..4,
    ) {
        let mut g = make_dup_token_grammar(&gn, dups);
        let before = g.tokens.len();
        let mut opt = GrammarOptimizer::new();
        let stats = opt.optimize(&mut g);
        let after = g.tokens.len();
        // Earlier passes (remove_unused_symbols) may remove unreferenced
        // duplicate tokens before merge runs, so we check the combined effect.
        prop_assert!(
            stats.total() > 0 || after < before,
            "expected some optimisation for grammar with {} duplicates (before={}, after={})",
            dups,
            before,
            after,
        );
    }

    /// 10. Merging idempotent: second pass merges zero additional tokens.
    #[test]
    fn pass_merge_idempotent(
        gn in name_strat(),
        dups in 1usize..4,
    ) {
        let g = make_dup_token_grammar(&gn, dups);
        let once = optimize_grammar(g).unwrap();
        let mut second = once.clone();
        let mut opt2 = GrammarOptimizer::new();
        let stats2 = opt2.optimize(&mut second);
        prop_assert_eq!(stats2.merged_tokens, 0, "second pass merged more tokens");
    }

    // ===================================================================
    // 11–15: eliminate_unit_rules pass
    // ===================================================================

    /// 11. Unit-rule chain is resolved so terminal production is reachable from start.
    #[test]
    fn pass_unit_chain_resolved(
        gn in name_strat(),
        len in 1usize..4,
    ) {
        let g = make_unit_rule_grammar(&gn, len);
        let opt = optimize_grammar(g).unwrap();
        // After elimination, the start symbol should directly or indirectly
        // reach a terminal production.
        let refs = collect_terminal_refs(&opt);
        prop_assert!(!refs.is_empty(), "no terminal refs after unit elimination");
    }

    /// 12. All terminal refs are valid after unit-rule elimination.
    #[test]
    fn pass_unit_terminals_valid(
        gn in name_strat(),
        len in 1usize..4,
    ) {
        let g = make_unit_rule_grammar(&gn, len);
        let opt = optimize_grammar(g).unwrap();
        for id in collect_terminal_refs(&opt) {
            prop_assert!(
                opt.tokens.contains_key(&id),
                "dangling terminal {:?} after unit elimination",
                id,
            );
        }
    }

    /// 13. Unit elimination is idempotent.
    #[test]
    fn pass_unit_idempotent(
        gn in name_strat(),
        len in 1usize..3,
    ) {
        let g = make_unit_rule_grammar(&gn, len);
        let once = optimize_grammar(g).unwrap();
        let json1 = serde_json::to_string(&once).unwrap();
        let twice = optimize_grammar(once).unwrap();
        let json2 = serde_json::to_string(&twice).unwrap();
        prop_assert_eq!(json1, json2, "unit elimination not idempotent");
    }

    /// 14. Unit chain grammar still has at least one rule after optimisation.
    #[test]
    fn pass_unit_rules_survive(
        gn in name_strat(),
        len in 1usize..4,
    ) {
        let g = make_unit_rule_grammar(&gn, len);
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(!opt.rules.is_empty(), "all rules removed from unit-chain grammar");
    }

    /// 15. Unit-chain grammar is modified by optimisation (rules may be
    /// inlined, eliminated, or renumbered).
    #[test]
    fn pass_unit_grammar_modified(
        gn in name_strat(),
        len in 1usize..4,
    ) {
        let g = make_unit_rule_grammar(&gn, len);
        let before_json = serde_json::to_string(&g).unwrap();
        let opt = optimize_grammar(g).unwrap();
        let after_json = serde_json::to_string(&opt).unwrap();
        prop_assert_ne!(
            before_json, after_json,
            "expected grammar to change for chain len {}",
            len,
        );
    }

    // ===================================================================
    // 16–20: optimize_left_recursion pass
    // ===================================================================

    /// 16. Left-recursive grammar retains rules for the original symbol after transform.
    #[test]
    fn pass_lr_retains_original_symbol(
        gn in name_strat(),
        ops in 1usize..4,
    ) {
        let g = make_left_rec_grammar(&gn, ops);
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(!opt.rules.is_empty(), "no rules after LR optimisation");
    }

    /// 17. Left-recursion transform creates at least one new helper symbol (A').
    #[test]
    fn pass_lr_creates_helper_symbol(
        gn in name_strat(),
        ops in 1usize..3,
    ) {
        let g = make_left_rec_grammar(&gn, ops);
        let before_keys: HashSet<_> = g.rules.keys().copied().collect();
        let opt = optimize_grammar(g).unwrap();
        // Renumbering changes IDs, but the count of distinct rule-LHS entries
        // must be >= 2 (original + helper).
        prop_assert!(
            opt.rules.len() >= 2,
            "expected >= 2 rule groups, got {}",
            opt.rules.len(),
        );
    }

    /// 18. After LR transform, no direct left-recursion remains.
    #[test]
    fn pass_lr_no_direct_left_recursion(
        gn in name_strat(),
        ops in 1usize..4,
    ) {
        let g = make_left_rec_grammar(&gn, ops);
        let opt = optimize_grammar(g).unwrap();
        for (lhs, rules) in &opt.rules {
            for rule in rules {
                if let Some(Symbol::NonTerminal(id)) = rule.rhs.first() {
                    prop_assert_ne!(
                        *id, *lhs,
                        "direct left recursion on {:?} still present",
                        lhs,
                    );
                }
            }
        }
    }

    /// 19. Total productions are non-decreasing after LR transform (epsilon rules added).
    #[test]
    fn pass_lr_productions_nondecreasing(
        gn in name_strat(),
        ops in 1usize..4,
    ) {
        let g = make_left_rec_grammar(&gn, ops);
        let before = total_productions(&g);
        let opt = optimize_grammar(g).unwrap();
        let after = total_productions(&opt);
        prop_assert!(
            after >= before,
            "productions decreased: {} -> {}",
            before,
            after,
        );
    }

    /// 20. LR transform is idempotent (no further left recursion to transform).
    #[test]
    fn pass_lr_idempotent(
        gn in name_strat(),
        ops in 1usize..3,
    ) {
        let g = make_left_rec_grammar(&gn, ops);
        let once = optimize_grammar(g).unwrap();
        let json1 = serde_json::to_string(&once).unwrap();
        let twice = optimize_grammar(once).unwrap();
        let json2 = serde_json::to_string(&twice).unwrap();
        prop_assert_eq!(json1, json2, "LR optimisation not idempotent");
    }

    // ===================================================================
    // 21–25: Externals preservation across all passes
    // ===================================================================

    /// 21. External token names are preserved through optimisation.
    #[test]
    fn pass_externals_names_preserved(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
        en in ext_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &en);
        let before_names: HashSet<String> =
            g.externals.iter().map(|e| e.name.clone()).collect();
        let opt = optimize_grammar(g).unwrap();
        let after_names: HashSet<String> =
            opt.externals.iter().map(|e| e.name.clone()).collect();
        prop_assert_eq!(before_names, after_names, "external names changed");
    }

    /// 22. External symbol IDs are still unique after renumbering.
    #[test]
    fn pass_externals_unique_ids(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
        en in ext_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &en);
        let opt = optimize_grammar(g).unwrap();
        let ids: HashSet<SymbolId> = opt.externals.iter().map(|e| e.symbol_id).collect();
        prop_assert_eq!(
            ids.len(),
            opt.externals.len(),
            "duplicate external symbol IDs",
        );
    }

    /// 23. External IDs do not collide with token IDs after renumbering.
    #[test]
    fn pass_externals_no_token_collision(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
        en in ext_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &en);
        let opt = optimize_grammar(g).unwrap();
        let tok_ids: HashSet<SymbolId> = opt.tokens.keys().copied().collect();
        for ext in &opt.externals {
            prop_assert!(
                !tok_ids.contains(&ext.symbol_id),
                "external {:?} collides with token",
                ext.symbol_id,
            );
        }
    }

    /// 24. External IDs do not collide with rule LHS IDs after renumbering.
    #[test]
    fn pass_externals_no_rule_collision(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
        en in ext_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &en);
        let opt = optimize_grammar(g).unwrap();
        let rule_ids: HashSet<SymbolId> = opt.rules.keys().copied().collect();
        for ext in &opt.externals {
            prop_assert!(
                !rule_ids.contains(&ext.symbol_id),
                "external {:?} collides with rule LHS",
                ext.symbol_id,
            );
        }
    }

    /// 25. Grammar with only externals (no rules) does not panic.
    #[test]
    fn pass_externals_only_no_panic(gn in name_strat()) {
        let mut g = Grammar::new(gn);
        g.externals.push(ExternalToken {
            name: "indent".to_string(),
            symbol_id: SymbolId(10),
        });
        g.externals.push(ExternalToken {
            name: "dedent".to_string(),
            symbol_id: SymbolId(11),
        });
        let opt = optimize_grammar(g).unwrap();
        prop_assert_eq!(opt.externals.len(), 2);
    }

    // ===================================================================
    // 26–30: Renumber / validity across passes
    // ===================================================================

    /// 26. All rule LHS IDs are contiguous starting from 1 after optimisation.
    #[test]
    fn pass_renumber_ids_contiguous(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        let mut all_ids: Vec<u16> = opt
            .rules
            .keys()
            .chain(opt.tokens.keys())
            .map(|id| id.0)
            .collect();
        all_ids.sort();
        all_ids.dedup();
        // IDs should form a dense range starting at 1.
        if !all_ids.is_empty() {
            prop_assert_eq!(all_ids[0], 1, "IDs don't start at 1");
            for i in 1..all_ids.len() {
                prop_assert!(
                    all_ids[i] <= all_ids[i - 1] + 1,
                    "gap between {} and {}",
                    all_ids[i - 1],
                    all_ids[i],
                );
            }
        }
    }

    /// 27. Production IDs are unique across the entire optimised grammar.
    #[test]
    fn pass_production_ids_unique(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        let mut pids = HashSet::new();
        for rule in opt.all_rules() {
            prop_assert!(
                pids.insert(rule.production_id),
                "duplicate production_id {:?}",
                rule.production_id,
            );
        }
    }

    /// 28. rule_names keys are a subset of rule keys ∪ token keys after optimisation.
    #[test]
    fn pass_rule_names_subset_of_known_symbols(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        let known: HashSet<SymbolId> = opt
            .rules
            .keys()
            .chain(opt.tokens.keys())
            .copied()
            .collect();
        for id in opt.rule_names.keys() {
            prop_assert!(
                known.contains(id),
                "rule_names has {:?} but no matching rule or token",
                id,
            );
        }
    }

    /// 29. Grammar with extras: extras IDs are valid after renumbering.
    #[test]
    fn pass_extras_valid_after_renumber(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
    ) {
        let mut g = make_grammar(&gn, &tn, &rn, &[]);
        // Add first token as an extra.
        if let Some(&first) = g.tokens.keys().next() {
            g.extras.push(first);
        }
        let opt = optimize_grammar(g).unwrap();
        let tok_ids: HashSet<SymbolId> = opt.tokens.keys().copied().collect();
        for &extra_id in &opt.extras {
            prop_assert!(
                tok_ids.contains(&extra_id),
                "extras has {:?} which is not a token",
                extra_id,
            );
        }
    }

    /// 30. Grammar with precedence annotations: precedence survives optimisation.
    #[test]
    fn pass_precedence_survives(gn in name_strat()) {
        let g = GrammarBuilder::new(&gn)
            .token("a", "a")
            .token("b", "b")
            .rule_with_precedence("expr", vec!["a", "b"], 2, Associativity::Left)
            .rule("expr", vec!["a", "a"])
            .start("expr")
            .build();
        let opt = optimize_grammar(g).unwrap();
        let has_prec = opt.all_rules().any(|r| r.precedence.is_some());
        prop_assert!(has_prec, "all precedence annotations lost");
    }

    // ===================================================================
    // 31–33: Cross-pass interaction
    // ===================================================================

    /// 31. Grammar with both duplicates and unit rules optimises without panic.
    #[test]
    fn pass_combined_dup_and_unit(gn in name_strat()) {
        let mut g = Grammar::new(gn);

        let tok = SymbolId(1);
        g.tokens.insert(
            tok,
            Token {
                name: "t".to_string(),
                pattern: TokenPattern::String("t".to_string()),
                fragile: false,
            },
        );
        let dup_tok = SymbolId(2);
        g.tokens.insert(
            dup_tok,
            Token {
                name: "t_dup".to_string(),
                pattern: TokenPattern::String("t".to_string()),
                fragile: false,
            },
        );

        let start = SymbolId(50);
        let mid = SymbolId(51);
        g.rule_names.insert(start, "start".to_string());
        g.rule_names.insert(mid, "mid".to_string());

        // Unit rule: start -> mid.
        g.add_rule(Rule {
            lhs: start,
            rhs: vec![Symbol::NonTerminal(mid)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        // Terminal production: mid -> tok dup_tok.
        g.add_rule(Rule {
            lhs: mid,
            rhs: vec![Symbol::Terminal(tok), Symbol::Terminal(dup_tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        });

        let opt = optimize_grammar(g).unwrap();
        prop_assert!(!opt.rules.is_empty());
        // After merge, dup_tok should be gone.
        let patterns: HashSet<String> = opt
            .tokens
            .values()
            .map(|t| match &t.pattern {
                TokenPattern::String(s) => s.clone(),
                TokenPattern::Regex(r) => r.clone(),
            })
            .collect();
        prop_assert_eq!(patterns.len(), opt.tokens.len());
    }

    /// 32. Randomised grammar: total rule-LHS + token count never increases.
    #[test]
    fn pass_random_total_never_increases(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
        en in ext_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &en);
        let before = g.rules.len() + g.tokens.len();
        let opt = optimize_grammar(g).unwrap();
        let after = opt.rules.len() + opt.tokens.len();
        prop_assert!(after <= before, "total increased: {} -> {}", before, after);
    }

    /// 33. Three consecutive passes converge to the same serialization.
    #[test]
    fn pass_triple_convergence(
        gn in name_strat(),
        tn in tok_strat(),
        rn in rule_strat(),
    ) {
        let g = make_grammar(&gn, &tn, &rn, &[]);
        let once = optimize_grammar(g).unwrap();
        let twice = optimize_grammar(once.clone()).unwrap();
        let thrice = optimize_grammar(twice.clone()).unwrap();
        let j2 = serde_json::to_string(&twice).unwrap();
        let j3 = serde_json::to_string(&thrice).unwrap();
        prop_assert_eq!(j2, j3, "not converged after three passes");
    }
}
