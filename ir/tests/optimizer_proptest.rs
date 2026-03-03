#![allow(clippy::needless_range_loop)]

//! Property-based tests for the grammar optimizer in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{optimize_grammar, GrammarOptimizer};
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a well-formed grammar from generated names.
///
/// Tokens are named `t_<idx>_<name>`, rules are named `r_<idx>_<name>`.
/// Each rule uses at least two RHS symbols so they are not trivial unit rules.
/// The start rule additionally references a subsequent rule to keep it reachable.
fn build_grammar(
    grammar_name: &str,
    token_names: &[String],
    rule_names: &[String],
    external_names: &[String],
) -> Grammar {
    let mut builder = GrammarBuilder::new(grammar_name);

    let tok_ids: Vec<String> = token_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("t_{i}_{n}"))
        .collect();
    for tok in &tok_ids {
        builder = builder.token(tok, tok);
    }

    for (i, ename) in external_names.iter().enumerate() {
        let ext = format!("e_{i}_{ename}");
        builder = builder.external(&ext);
    }

    let rule_ids: Vec<String> = rule_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("r_{i}_{n}"))
        .collect();

    let first_tok = &tok_ids[0];
    let second_tok = if tok_ids.len() > 1 {
        &tok_ids[1]
    } else {
        &tok_ids[0]
    };

    for (i, rname) in rule_ids.iter().enumerate() {
        if i == 0 && rule_ids.len() > 1 {
            builder = builder.rule(rname, vec![first_tok, &rule_ids[1]]);
        } else {
            builder = builder.rule(rname, vec![first_tok, second_tok]);
        }
    }

    builder = builder.start(&rule_ids[0]);
    builder.build()
}

/// Build a grammar with left-recursive rules.
fn build_left_recursive_grammar(grammar_name: &str, op_count: usize) -> Grammar {
    let mut grammar = Grammar::new(grammar_name.to_string());

    let num_id = SymbolId(1);
    grammar.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let mut next_tok = 2u16;
    let mut op_ids = Vec::new();
    for i in 0..op_count.max(1) {
        let id = SymbolId(next_tok);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("op{}", i),
                pattern: TokenPattern::String(format!("op{}", i)),
                fragile: false,
            },
        );
        op_ids.push(id);
        next_tok += 1;
    }

    let expr_id = SymbolId(100);
    grammar.rule_names.insert(expr_id, "expr".to_string());

    let mut prod_id = 0u16;
    for op in &op_ids {
        grammar.add_rule(Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(*op),
                Symbol::Terminal(num_id),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        });
        prod_id += 1;
    }

    // Base case: expr -> number
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod_id),
    });

    grammar
}

/// Build a grammar with complex symbols for normalization tests.
fn build_complex_grammar(token_count: usize) -> Grammar {
    let tc = token_count.max(2);
    let mut grammar = Grammar::new("complex".to_string());

    for i in 1..=tc {
        grammar.tokens.insert(
            SymbolId(i as u16),
            Token {
                name: format!("t{}", i),
                pattern: TokenPattern::String(format!("tok{}", i)),
                fragile: false,
            },
        );
    }

    let start = SymbolId(100);
    grammar.rule_names.insert(start, "start".to_string());

    let t1 = SymbolId(1);
    let t2 = SymbolId(2);

    // start -> t1 Optional(t2) t1
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![
            Symbol::Terminal(t1),
            Symbol::Optional(Box::new(Symbol::Terminal(t2))),
            Symbol::Terminal(t1),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // start -> Repeat(t1)
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(t1)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    grammar
}

/// Total symbol count: distinct rule LHS entries + token entries.
fn total_symbol_count(g: &Grammar) -> usize {
    g.rules.len() + g.tokens.len()
}

/// Recursively visit a symbol, collecting terminal and nonterminal IDs.
fn visit_symbol(sym: &Symbol, nt: &mut HashSet<SymbolId>, t: &mut HashSet<SymbolId>) {
    match sym {
        Symbol::Terminal(id) => {
            t.insert(*id);
        }
        Symbol::NonTerminal(id) => {
            nt.insert(*id);
        }
        Symbol::External(_) => {}
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            visit_symbol(inner, nt, t);
        }
        Symbol::Choice(choices) => {
            for s in choices {
                visit_symbol(s, nt, t);
            }
        }
        Symbol::Sequence(seq) => {
            for s in seq {
                visit_symbol(s, nt, t);
            }
        }
        Symbol::Epsilon => {}
    }
}

/// Collect all terminal IDs referenced in any rule RHS.
fn referenced_terminal_ids(g: &Grammar) -> HashSet<SymbolId> {
    let mut nt = HashSet::new();
    let mut t = HashSet::new();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            visit_symbol(sym, &mut nt, &mut t);
        }
    }
    t
}

/// Check whether any rule RHS contains complex symbols.
fn has_complex_symbols(g: &Grammar) -> bool {
    fn is_complex(sym: &Symbol) -> bool {
        matches!(
            sym,
            Symbol::Optional(_)
                | Symbol::Repeat(_)
                | Symbol::RepeatOne(_)
                | Symbol::Choice(_)
                | Symbol::Sequence(_)
        )
    }
    g.all_rules().any(|r| r.rhs.iter().any(is_complex))
}

/// Collect the set of token pattern strings.
fn token_patterns(g: &Grammar) -> HashSet<String> {
    g.tokens
        .values()
        .map(|t| match &t.pattern {
            TokenPattern::String(s) => s.clone(),
            TokenPattern::Regex(r) => r.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn name_strat() -> impl Strategy<Value = String> {
    "[a-z]{1,8}"
}

fn token_names_strat() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strat(), 1..5usize)
}

fn rule_names_strat() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strat(), 1..5usize)
}

fn external_names_strat() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(name_strat(), 0..4usize)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // =======================================================================
    // Core preservation properties
    // =======================================================================

    /// 1. Grammar name is preserved through optimization.
    #[test]
    fn opt_preserves_grammar_name(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let name = g.name.clone();
        let opt = optimize_grammar(g).unwrap();
        prop_assert_eq!(opt.name, name);
    }

    /// 2. Start symbol still exists after optimization.
    #[test]
    fn opt_preserves_start_symbol_existence(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        prop_assert!(g.start_symbol().is_some());
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(
            opt.start_symbol().is_some(),
            "start symbol lost after optimization"
        );
    }

    /// 3. The start symbol has at least one production after optimization.
    #[test]
    fn opt_start_rule_has_productions(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        if let Some(start) = opt.start_symbol() {
            let rules = opt.get_rules_for_symbol(start);
            prop_assert!(
                rules.is_some() && !rules.unwrap().is_empty(),
                "start symbol has no productions"
            );
        }
    }

    /// 4. Total symbol count (rules + tokens) never increases.
    #[test]
    fn opt_never_increases_total_symbol_count(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let before = total_symbol_count(&g);
        let opt = optimize_grammar(g).unwrap();
        let after = total_symbol_count(&opt);
        prop_assert!(after <= before, "symbol count grew: {} -> {}", before, after);
    }

    /// 5. Distinct rule-LHS count never increases.
    #[test]
    fn opt_never_increases_rule_lhs_count(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let before = g.rules.len();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(opt.rules.len() <= before, "rule LHS count increased");
    }

    /// 6. Token count never increases.
    #[test]
    fn opt_tokens_dont_increase(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let before = g.tokens.len();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(opt.tokens.len() <= before, "token count increased");
    }

    /// 7. External token count is unchanged.
    #[test]
    fn opt_preserves_external_count(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
        en in external_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &en);
        let before = g.externals.len();
        let opt = optimize_grammar(g).unwrap();
        prop_assert_eq!(opt.externals.len(), before);
    }

    // =======================================================================
    // Idempotency properties
    // =======================================================================

    /// 8. Optimizing twice yields the same serialization as once.
    #[test]
    fn opt_idempotent_serialization(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let once = optimize_grammar(g).unwrap();
        let json_once = serde_json::to_string(&once).unwrap();
        let twice = optimize_grammar(once).unwrap();
        let json_twice = serde_json::to_string(&twice).unwrap();
        prop_assert_eq!(json_once, json_twice, "optimization not idempotent");
    }

    /// 9. Rule-LHS count is stable after the first optimization pass.
    #[test]
    fn opt_idempotent_rule_count(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let once = optimize_grammar(g).unwrap();
        let c1 = once.rules.len();
        let twice = optimize_grammar(once).unwrap();
        prop_assert_eq!(twice.rules.len(), c1);
    }

    /// 10. Token count is stable after the first optimization pass.
    #[test]
    fn opt_idempotent_token_count(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let once = optimize_grammar(g).unwrap();
        let tc1 = once.tokens.len();
        let twice = optimize_grammar(once).unwrap();
        prop_assert_eq!(twice.tokens.len(), tc1);
    }

    /// 11. Start symbol is stable after the first optimization pass.
    #[test]
    fn opt_idempotent_start_symbol(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let once = optimize_grammar(g).unwrap();
        let s1 = once.start_symbol();
        let twice = optimize_grammar(once).unwrap();
        let s2 = twice.start_symbol();
        prop_assert_eq!(s1, s2, "start symbol changed across passes");
    }

    // =======================================================================
    // Symbol validity properties
    // =======================================================================

    /// 12. Every Terminal ID in rule RHS has a matching token entry.
    #[test]
    fn opt_all_terminal_refs_have_token_entry(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        let t_ids = referenced_terminal_ids(&opt);
        for id in t_ids {
            prop_assert!(
                opt.tokens.contains_key(&id),
                "terminal {:?} has no token entry",
                id
            );
        }
    }

    /// 13. Every rule.lhs matches the key it is stored under.
    #[test]
    fn opt_rule_lhs_matches_key(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        for (key, rules) in &opt.rules {
            for rule in rules {
                prop_assert_eq!(rule.lhs, *key, "rule.lhs != key");
            }
        }
    }

    /// 14. No SymbolId(0) appears as a rule key or token key (0 is reserved for EOF).
    #[test]
    fn opt_all_symbol_ids_nonzero(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        for key in opt.rules.keys() {
            prop_assert!(key.0 > 0, "rule key has SymbolId(0)");
        }
        for key in opt.tokens.keys() {
            prop_assert!(key.0 > 0, "token key has SymbolId(0)");
        }
    }

    /// 15. The convenience function produces the same result as manual invocation.
    #[test]
    fn opt_convenience_fn_matches_manual(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g1 = build_grammar(&gn, &tn, &rn, &[]);
        let g2 = g1.clone();

        let opt1 = optimize_grammar(g1).unwrap();

        let mut opt2 = g2;
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut opt2);

        prop_assert_eq!(
            serde_json::to_string(&opt1).unwrap(),
            serde_json::to_string(&opt2).unwrap(),
        );
    }

    /// 16. No rule vector in the map is empty.
    #[test]
    fn opt_no_empty_rule_vectors(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        for (key, rules) in &opt.rules {
            prop_assert!(
                !rules.is_empty(),
                "symbol {:?} has empty rule vector",
                key
            );
        }
    }

    // =======================================================================
    // Token properties
    // =======================================================================

    /// 17. After optimization, no two tokens share the same pattern.
    #[test]
    fn opt_no_duplicate_token_patterns(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        let patterns = token_patterns(&opt);
        prop_assert_eq!(
            patterns.len(),
            opt.tokens.len(),
            "duplicate token patterns remain"
        );
    }

    /// 18. Every token has a non-empty pattern after optimization.
    #[test]
    fn opt_all_token_patterns_nonempty(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        for (id, tok) in &opt.tokens {
            let pat = match &tok.pattern {
                TokenPattern::String(s) => s,
                TokenPattern::Regex(r) => r,
            };
            prop_assert!(!pat.is_empty(), "token {:?} has empty pattern", id);
        }
    }

    /// 19. Every token has a non-empty name after optimization.
    #[test]
    fn opt_token_names_nonempty(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let opt = optimize_grammar(g).unwrap();
        for (id, tok) in &opt.tokens {
            prop_assert!(!tok.name.is_empty(), "token {:?} has empty name", id);
        }
    }

    // =======================================================================
    // Stats properties
    // =======================================================================

    /// 20. stats.total() equals the sum of all individual fields.
    #[test]
    fn opt_stats_total_equals_sum(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let mut g = build_grammar(&gn, &tn, &rn, &[]);
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut g);
        let sum = stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules;
        prop_assert_eq!(stats.total(), sum);
    }

    /// 21. Total optimizations are bounded by a reasonable multiple of grammar size.
    #[test]
    fn opt_stats_bounded(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let g = build_grammar(&gn, &tn, &rn, &[]);
        let original_size = g.rules.len() + g.tokens.len() + g.externals.len();
        let mut g2 = g;
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut g2);
        prop_assert!(
            stats.total() <= original_size * 10,
            "stats.total()={} for grammar of size {}",
            stats.total(),
            original_size
        );
    }

    // =======================================================================
    // Left-recursive grammar properties
    // =======================================================================

    /// 22. Left-recursive grammar still has rules after optimization.
    #[test]
    fn opt_left_recursive_preserves_rules(
        gn in name_strat(),
        ops in 1usize..4,
    ) {
        let g = build_left_recursive_grammar(&gn, ops);
        prop_assert!(!g.rules.is_empty());
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(
            !opt.rules.is_empty(),
            "all rules removed from left-recursive grammar"
        );
    }

    /// 23. Terminal references are valid after optimizing left-recursive grammar.
    #[test]
    fn opt_left_recursive_terminals_valid(
        gn in name_strat(),
        ops in 1usize..4,
    ) {
        let g = build_left_recursive_grammar(&gn, ops);
        let opt = optimize_grammar(g).unwrap();
        let t_ids = referenced_terminal_ids(&opt);
        for id in t_ids {
            prop_assert!(
                opt.tokens.contains_key(&id),
                "dangling terminal {:?} in left-recursive grammar",
                id
            );
        }
    }

    /// 24. Left-recursive grammar optimization is idempotent.
    #[test]
    fn opt_left_recursive_idempotent(
        gn in name_strat(),
        ops in 1usize..4,
    ) {
        let g = build_left_recursive_grammar(&gn, ops);
        let once = optimize_grammar(g).unwrap();
        let json_once = serde_json::to_string(&once).unwrap();
        let twice = optimize_grammar(once).unwrap();
        let json_twice = serde_json::to_string(&twice).unwrap();
        prop_assert_eq!(json_once, json_twice);
    }

    // =======================================================================
    // Specialized grammar patterns
    // =======================================================================

    /// 25. Grammar with precedence still has rules after optimization.
    #[test]
    fn opt_with_precedence_preserves_rules(
        gn in name_strat(),
        tn in token_names_strat(),
    ) {
        let tok_ids: Vec<String> = tn
            .iter()
            .enumerate()
            .map(|(i, n)| format!("t_{i}_{n}"))
            .collect();
        let mut builder = GrammarBuilder::new(&gn);
        for tok in &tok_ids {
            builder = builder.token(tok, tok);
        }
        let first = &tok_ids[0];
        let second = if tok_ids.len() > 1 {
            &tok_ids[1]
        } else {
            &tok_ids[0]
        };
        builder =
            builder.rule_with_precedence("expr", vec![first, second], 1, Associativity::Left);
        builder = builder.rule("expr", vec![first, second]);
        builder = builder.start("expr");
        let g = builder.build();

        let opt = optimize_grammar(g).unwrap();
        prop_assert!(!opt.rules.is_empty());
    }

    /// 26. Extras count does not increase after optimization.
    #[test]
    fn opt_with_extras_count_stable(
        gn in name_strat(),
        tn in token_names_strat(),
        rn in rule_names_strat(),
    ) {
        let mut g = build_grammar(&gn, &tn, &rn, &[]);
        if let Some(&first_tok) = g.tokens.keys().next() {
            g.extras.push(first_tok);
        }
        let extras_before = g.extras.len();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(
            opt.extras.len() <= extras_before,
            "extras count increased"
        );
    }

    /// 27. Minimal grammar survives optimization with rules intact.
    #[test]
    fn opt_minimal_grammar_survives(gn in name_strat()) {
        let g = GrammarBuilder::new(&gn)
            .token("t", "t")
            .rule("r", vec!["t", "t"])
            .start("r")
            .build();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(!opt.rules.is_empty());
        prop_assert!(!opt.tokens.is_empty());
    }

    // =======================================================================
    // Normalization interop
    // =======================================================================

    /// 28. Optimize then normalize leaves no complex symbols.
    #[test]
    fn opt_then_normalize_no_complex_symbols(tc in 2usize..6) {
        let g = build_complex_grammar(tc);
        let mut opt = optimize_grammar(g).unwrap();
        let _ = opt.normalize();
        prop_assert!(
            !has_complex_symbols(&opt),
            "complex symbols remain after optimize + normalize"
        );
    }

    /// 29. Normalize then optimize still has rules.
    #[test]
    fn normalize_then_optimize_preserves_rules(tc in 2usize..6) {
        let mut g = build_complex_grammar(tc);
        let _ = g.normalize();
        let opt = optimize_grammar(g).unwrap();
        prop_assert!(
            !opt.rules.is_empty(),
            "all rules removed after normalize + optimize"
        );
    }

    /// 30. Terminal references valid after optimize then normalize.
    #[test]
    fn opt_then_normalize_terminals_valid(tc in 2usize..6) {
        let g = build_complex_grammar(tc);
        let mut opt = optimize_grammar(g).unwrap();
        let _ = opt.normalize();
        let t_ids = referenced_terminal_ids(&opt);
        for id in t_ids {
            prop_assert!(
                opt.tokens.contains_key(&id),
                "dangling terminal {:?} after optimize + normalize",
                id
            );
        }
    }

    /// 31. Terminal references valid after normalize then optimize.
    #[test]
    fn normalize_then_optimize_terminals_valid(tc in 2usize..6) {
        let mut g = build_complex_grammar(tc);
        let _ = g.normalize();
        let opt = optimize_grammar(g).unwrap();
        let t_ids = referenced_terminal_ids(&opt);
        for id in t_ids {
            prop_assert!(
                opt.tokens.contains_key(&id),
                "dangling terminal {:?} after normalize + optimize",
                id
            );
        }
    }

    /// 32. Both orderings (opt+norm vs norm+opt) produce grammars with no complex symbols.
    #[test]
    fn both_orderings_no_complex_symbols(tc in 2usize..6) {
        // Order 1: optimize then normalize
        let g1 = build_complex_grammar(tc);
        let mut opt1 = optimize_grammar(g1).unwrap();
        let _ = opt1.normalize();

        // Order 2: normalize then optimize
        let mut g2 = build_complex_grammar(tc);
        let _ = g2.normalize();
        let opt2 = optimize_grammar(g2).unwrap();

        prop_assert!(
            !has_complex_symbols(&opt1),
            "complex symbols in opt+norm ordering"
        );
        prop_assert!(
            !has_complex_symbols(&opt2),
            "complex symbols in norm+opt ordering"
        );
    }
}
