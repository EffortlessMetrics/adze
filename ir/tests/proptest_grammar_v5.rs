//! Property-based tests for GrammarBuilder and Grammar APIs.
//!
//! 40+ proptest properties covering construction, normalization,
//! symbol lookup, rule counts, and token uniqueness.

use proptest::prelude::*;

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, Symbol, SymbolId};

// ---------------------------------------------------------------------------
// Helpers: safe name generation (alphabetic only, no Rust 2024 keywords)
// ---------------------------------------------------------------------------

const RESERVED: &[&str] = &[
    "gen", "do", "abstract", "become", "final", "override", "priv", "typeof", "unsized", "virtual",
    "box", "macro", "try", "yield", "as", "break", "const", "continue", "crate", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "dyn",
];

fn is_reserved(s: &str) -> bool {
    RESERVED.contains(&s)
}

/// Strategy producing safe identifiers: 2–8 lowercase letters, never a keyword.
fn safe_ident() -> impl Strategy<Value = String> {
    "[a-z]{2,8}".prop_filter("must not be a reserved keyword", |s| !is_reserved(s))
}

/// Strategy for grammar names.
fn grammar_name() -> impl Strategy<Value = String> {
    safe_ident()
}

/// Strategy for token names: uppercase 2–6 chars.
fn token_name() -> impl Strategy<Value = String> {
    "[A-Z]{2,6}"
}

/// Strategy for simple regex patterns.
fn token_pattern() -> impl Strategy<Value = String> {
    "[a-z]{1,4}"
}

/// Build a minimal grammar with one token, one rule, and a start symbol.
fn minimal_grammar(name: &str, tok: &str, rule_sym: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token(tok, tok)
        .rule(rule_sym, vec![tok])
        .start(rule_sym)
        .build()
}

// ===========================================================================
// Section 1 — Grammar construction properties (10 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1
    #[test]
    fn grammar_name_preserved(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .start("root")
            .build();
        prop_assert_eq!(g.name, name);
    }

    // 2
    #[test]
    fn single_token_grammar_has_one_token(name in grammar_name(), tok in token_name()) {
        let g = GrammarBuilder::new(&name)
            .token(&tok, "a")
            .rule("root", vec![&tok])
            .start("root")
            .build();
        prop_assert_eq!(g.tokens.len(), 1);
    }

    // 3
    #[test]
    fn single_rule_grammar_has_one_lhs(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .start("root")
            .build();
        prop_assert_eq!(g.rules.len(), 1);
    }

    // 4
    #[test]
    fn empty_grammar_has_no_rules(name in grammar_name()) {
        let g = GrammarBuilder::new(&name).build();
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.tokens.is_empty());
    }

    // 5
    #[test]
    fn eof_symbol_is_zero(_dummy in 0u8..1) {
        prop_assert_eq!(SymbolId(0).0, 0);
    }

    // 6
    #[test]
    fn builder_assigns_ids_starting_at_one(tok in token_name()) {
        let g = GrammarBuilder::new("test")
            .token(&tok, "a")
            .rule("root", vec![&tok])
            .start("root")
            .build();
        // All symbol IDs > 0 (0 is reserved for EOF)
        for id in g.tokens.keys() {
            prop_assert!(id.0 >= 1);
        }
        for id in g.rules.keys() {
            prop_assert!(id.0 >= 1);
        }
    }

    // 7
    #[test]
    fn multiple_tokens_count(
        t1 in token_name(),
        t2 in token_name(),
        t3 in token_name(),
    ) {
        let names: Vec<&str> = vec![&t1, &t2, &t3];
        let mut builder = GrammarBuilder::new("multi");
        for n in &names {
            builder = builder.token(n, "x");
        }
        let g = builder.build();
        // Unique token names determine count
        let unique: std::collections::HashSet<&str> = names.into_iter().collect();
        prop_assert_eq!(g.tokens.len(), unique.len());
    }

    // 8
    #[test]
    fn start_symbol_rules_come_first(rule_name in safe_ident()) {
        let g = GrammarBuilder::new("ordered")
            .token("TOK", "x")
            .rule("other", vec!["TOK"])
            .rule(&rule_name, vec!["TOK"])
            .start(&rule_name)
            .build();
        let first_lhs = g.rules.keys().next();
        prop_assert!(first_lhs.is_some());
        if let Some(first_id) = first_lhs {
            let first_name = g.rule_names.get(first_id);
            prop_assert_eq!(first_name.map(|s| s.as_str()), Some(rule_name.as_str()));
        }
    }

    // 9
    #[test]
    fn rule_alternatives_accumulate(n in 1usize..6) {
        let mut builder = GrammarBuilder::new("alts")
            .token("TOK", "x");
        for _ in 0..n {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let g = builder.start("root").build();
        let root_id = g.find_symbol_by_name("root").unwrap();
        let rules = g.rules.get(&root_id).unwrap();
        prop_assert_eq!(rules.len(), n);
    }

    // 10
    #[test]
    fn epsilon_rule_from_empty_rhs(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .rule("nullable", vec![])
            .start("nullable")
            .build();
        let nid = g.find_symbol_by_name("nullable").unwrap();
        let rules = g.rules.get(&nid).unwrap();
        prop_assert!(rules.iter().any(|r| r.rhs.contains(&Symbol::Epsilon)));
    }
}

// ===========================================================================
// Section 2 — Normalize idempotency (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // 11
    #[test]
    fn normalize_idempotent_simple(name in grammar_name()) {
        let mut g1 = minimal_grammar(&name, "TOK", "root");
        g1.normalize();
        let count_after_first = g1.all_rules().count();

        g1.normalize();
        let count_after_second = g1.all_rules().count();
        prop_assert_eq!(count_after_first, count_after_second);
    }

    // 12
    #[test]
    fn normalize_preserves_rule_count_for_flat_grammar(name in grammar_name()) {
        let mut g = minimal_grammar(&name, "TOK", "root");
        let before = g.all_rules().count();
        g.normalize();
        let after = g.all_rules().count();
        // Flat grammars (no Optional/Repeat) should remain unchanged
        prop_assert_eq!(before, after);
    }

    // 13
    #[test]
    fn normalize_never_removes_start_symbol(name in grammar_name()) {
        let mut g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .rule("root", vec![])
            .start("root")
            .build();
        g.normalize();
        let root_id = g.find_symbol_by_name("root");
        prop_assert!(root_id.is_some());
        let rules = g.rules.get(&root_id.unwrap());
        prop_assert!(rules.is_some());
        prop_assert!(!rules.unwrap().is_empty());
    }

    // 14
    #[test]
    fn normalize_keeps_token_count(name in grammar_name()) {
        let mut g = minimal_grammar(&name, "TOK", "root");
        let tok_count_before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), tok_count_before);
    }

    // 15
    #[test]
    fn normalize_retains_grammar_name(name in grammar_name()) {
        let mut g = minimal_grammar(&name, "TOK", "root");
        g.normalize();
        prop_assert_eq!(g.name, name);
    }

    // 16
    #[test]
    fn normalize_idempotent_multi_rule(n in 1usize..5) {
        let mut builder = GrammarBuilder::new("multi")
            .token("TOK", "x");
        for _ in 0..n {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let mut g = builder.start("root").build();
        g.normalize();
        let after_first = g.all_rules().count();
        g.normalize();
        let after_second = g.all_rules().count();
        prop_assert_eq!(after_first, after_second);
    }

    // 17
    #[test]
    fn normalize_epsilon_rule_survives(name in grammar_name()) {
        let mut g = GrammarBuilder::new(&name)
            .rule("nullable", vec![])
            .start("nullable")
            .build();
        g.normalize();
        let nid = g.find_symbol_by_name("nullable").unwrap();
        let rules = g.rules.get(&nid).unwrap();
        prop_assert!(rules.iter().any(|r| r.rhs.contains(&Symbol::Epsilon)));
    }

    // 18
    #[test]
    fn normalize_twice_same_rules_set(name in grammar_name()) {
        let mut g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .rule("root", vec![])
            .start("root")
            .build();
        g.normalize();
        let rules_first: Vec<_> = g.all_rules().cloned().collect();
        g.normalize();
        let rules_second: Vec<_> = g.all_rules().cloned().collect();
        prop_assert_eq!(rules_first.len(), rules_second.len());
    }
}

// ===========================================================================
// Section 3 — Symbol lookup consistency (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 19
    #[test]
    fn find_symbol_returns_consistent_id(rule_name in safe_ident()) {
        let g = GrammarBuilder::new("lookup")
            .token("TOK", "x")
            .rule(&rule_name, vec!["TOK"])
            .start(&rule_name)
            .build();
        let id1 = g.find_symbol_by_name(&rule_name);
        let id2 = g.find_symbol_by_name(&rule_name);
        prop_assert_eq!(id1, id2);
    }

    // 20
    #[test]
    fn find_symbol_missing_returns_none(name in safe_ident()) {
        let g = GrammarBuilder::new("empty").build();
        prop_assert!(g.find_symbol_by_name(&name).is_none());
    }

    // 21
    #[test]
    fn find_symbol_present_returns_some(rule_name in safe_ident()) {
        let g = GrammarBuilder::new("present")
            .token("TOK", "x")
            .rule(&rule_name, vec!["TOK"])
            .start(&rule_name)
            .build();
        prop_assert!(g.find_symbol_by_name(&rule_name).is_some());
    }

    // 22
    #[test]
    fn rule_names_contain_all_nonterminals(
        r1 in safe_ident(),
        r2 in safe_ident(),
    ) {
        let g = GrammarBuilder::new("names")
            .token("TOK", "x")
            .rule(&r1, vec!["TOK"])
            .rule(&r2, vec!["TOK"])
            .start(&r1)
            .build();
        let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
        prop_assert!(names.contains(&r1.as_str()));
        prop_assert!(names.contains(&r2.as_str()));
    }

    // 23
    #[test]
    fn find_symbol_id_matches_rules_key(rule_name in safe_ident()) {
        let g = GrammarBuilder::new("match")
            .token("TOK", "x")
            .rule(&rule_name, vec!["TOK"])
            .start(&rule_name)
            .build();
        let id = g.find_symbol_by_name(&rule_name).unwrap();
        prop_assert!(g.rules.contains_key(&id));
    }

    // 24
    #[test]
    fn symbol_id_zero_not_in_rule_names(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("eof")
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .start("root")
            .build();
        // SymbolId(0) is EOF — never assigned by builder
        prop_assert!(!g.rule_names.contains_key(&SymbolId(0)));
    }

    // 25
    #[test]
    fn two_different_names_get_different_ids(
        r1 in safe_ident(),
        r2 in safe_ident(),
    ) {
        prop_assume!(r1 != r2);
        let g = GrammarBuilder::new("diff")
            .token("TOK", "x")
            .rule(&r1, vec!["TOK"])
            .rule(&r2, vec!["TOK"])
            .start(&r1)
            .build();
        let id1 = g.find_symbol_by_name(&r1).unwrap();
        let id2 = g.find_symbol_by_name(&r2).unwrap();
        prop_assert_ne!(id1, id2);
    }

    // 26
    #[test]
    fn get_rules_for_symbol_matches_find(rule_name in safe_ident()) {
        let g = GrammarBuilder::new("roundtrip")
            .token("TOK", "x")
            .rule(&rule_name, vec!["TOK"])
            .start(&rule_name)
            .build();
        let id = g.find_symbol_by_name(&rule_name).unwrap();
        let rules = g.get_rules_for_symbol(id);
        prop_assert!(rules.is_some());
        prop_assert!(!rules.unwrap().is_empty());
    }
}

// ===========================================================================
// Section 4 — Rule count invariants (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 27
    #[test]
    fn all_rules_count_equals_sum_of_alternatives(n in 1usize..6) {
        let mut builder = GrammarBuilder::new("count")
            .token("TOK", "x");
        for _ in 0..n {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let g = builder.start("root").build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(g.all_rules().count(), total);
    }

    // 28
    #[test]
    fn adding_rules_increases_count(base in 1usize..4, extra in 1usize..4) {
        let mut builder = GrammarBuilder::new("grow")
            .token("TOK", "x");
        for _ in 0..base {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let g1 = builder.start("root").build();

        let mut builder2 = GrammarBuilder::new("grow")
            .token("TOK", "x");
        for _ in 0..(base + extra) {
            builder2 = builder2.rule("root", vec!["TOK"]);
        }
        let g2 = builder2.start("root").build();
        prop_assert!(g2.all_rules().count() > g1.all_rules().count());
    }

    // 29
    #[test]
    fn lhs_count_bounded_by_distinct_names(
        r1 in safe_ident(),
        r2 in safe_ident(),
    ) {
        let g = GrammarBuilder::new("bound")
            .token("TOK", "x")
            .rule(&r1, vec!["TOK"])
            .rule(&r2, vec!["TOK"])
            .start(&r1)
            .build();
        let distinct: std::collections::HashSet<&str> = [r1.as_str(), r2.as_str()].into();
        prop_assert!(g.rules.len() <= distinct.len());
    }

    // 30
    #[test]
    fn same_lhs_alternatives_share_key(n in 2usize..6) {
        let mut builder = GrammarBuilder::new("shared")
            .token("TOK", "x");
        for _ in 0..n {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let g = builder.start("root").build();
        // Only one LHS key for "root"
        prop_assert_eq!(g.rules.len(), 1);
    }

    // 31
    #[test]
    fn all_rules_lhs_matches_map_keys(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .rule("other", vec!["TOK"])
            .start("root")
            .build();
        for rule in g.all_rules() {
            prop_assert!(g.rules.contains_key(&rule.lhs));
        }
    }

    // 32
    #[test]
    fn production_ids_are_unique(n in 1usize..8) {
        let mut builder = GrammarBuilder::new("prodids")
            .token("TOK", "x");
        for _ in 0..n {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let g = builder.start("root").build();
        let ids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        prop_assert_eq!(ids.len(), unique.len());
    }

    // 33
    #[test]
    fn rule_rhs_terminal_ids_exist_in_tokens(name in grammar_name()) {
        let g = minimal_grammar(&name, "TOK", "root");
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                if let Symbol::Terminal(tid) = sym {
                    prop_assert!(g.tokens.contains_key(tid));
                }
            }
        }
    }

    // 34
    #[test]
    fn empty_grammar_all_rules_empty(name in grammar_name()) {
        let g = GrammarBuilder::new(&name).build();
        prop_assert_eq!(g.all_rules().count(), 0);
    }
}

// ===========================================================================
// Section 5 — Token uniqueness and properties (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 35
    #[test]
    fn duplicate_token_name_overwrites(tok in token_name()) {
        let g = GrammarBuilder::new("dup")
            .token(&tok, "first")
            .token(&tok, "second")
            .build();
        // Same name → same symbol ID → one entry
        prop_assert_eq!(g.tokens.len(), 1);
    }

    // 36
    #[test]
    fn distinct_tokens_get_distinct_ids(
        t1 in token_name(),
        t2 in token_name(),
    ) {
        prop_assume!(t1 != t2);
        let g = GrammarBuilder::new("dist")
            .token(&t1, "a")
            .token(&t2, "b")
            .build();
        let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
        prop_assert_eq!(ids.len(), 2);
        prop_assert_ne!(ids[0], ids[1]);
    }

    // 37
    #[test]
    fn token_name_matches_registration(tok in token_name()) {
        let g = GrammarBuilder::new("reg")
            .token(&tok, "pat")
            .build();
        let token_entry = g.tokens.values().next().unwrap();
        prop_assert_eq!(token_entry.name.as_str(), tok.as_str());
    }

    // 38
    #[test]
    fn token_count_matches_unique_names(
        t1 in token_name(),
        t2 in token_name(),
        t3 in token_name(),
    ) {
        let names = vec![&t1, &t2, &t3];
        let unique: std::collections::HashSet<&&String> = names.iter().collect();
        let mut builder = GrammarBuilder::new("cnt");
        for n in &names {
            builder = builder.token(n, "x");
        }
        let g = builder.build();
        prop_assert_eq!(g.tokens.len(), unique.len());
    }

    // 39
    #[test]
    fn tokens_survive_normalize(tok in token_name()) {
        let mut g = GrammarBuilder::new("surv")
            .token(&tok, "pat")
            .rule("root", vec![&tok])
            .start("root")
            .build();
        g.normalize();
        prop_assert!(g.tokens.values().any(|t| t.name == tok));
    }

    // 40
    #[test]
    fn token_id_never_zero(tok in token_name()) {
        let g = GrammarBuilder::new("nz")
            .token(&tok, "a")
            .build();
        for id in g.tokens.keys() {
            prop_assert_ne!(id.0, 0, "token SymbolId must not be 0 (EOF)");
        }
    }

    // 41
    #[test]
    fn token_not_fragile_by_default(tok in token_name()) {
        let g = GrammarBuilder::new("frag")
            .token(&tok, "a")
            .build();
        for t in g.tokens.values() {
            prop_assert!(!t.fragile);
        }
    }

    // 42
    #[test]
    fn token_pattern_preserved(tok in token_name(), pat in token_pattern()) {
        let g = GrammarBuilder::new("pat")
            .token(&tok, &pat)
            .build();
        let token_entry = g.tokens.values().next().unwrap();
        // Pattern is stored — verify it contains the original pattern text
        let stored = match &token_entry.pattern {
            adze_ir::TokenPattern::String(s) => s.as_str(),
            adze_ir::TokenPattern::Regex(r) => r.as_str(),
        };
        prop_assert_eq!(stored, pat.as_str());
    }
}

// ===========================================================================
// Section 6 — Miscellaneous cross-cutting properties (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // 43
    #[test]
    fn grammar_clone_equals_original(name in grammar_name()) {
        let g = minimal_grammar(&name, "TOK", "root");
        let g2 = g.clone();
        prop_assert_eq!(g, g2);
    }

    // 44
    #[test]
    fn grammar_default_is_empty(_dummy in 0u8..1) {
        let g = Grammar::default();
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.tokens.is_empty());
        prop_assert!(g.name.is_empty());
    }

    // 45
    #[test]
    fn symbol_id_copy_semantics(a in 0u16..1000) {
        let id = SymbolId(a);
        let id2 = id; // Copy, not move
        prop_assert_eq!(id, id2);
        prop_assert_eq!(id.0, a);
        prop_assert_eq!(id2.0, a);
    }

    // 46
    #[test]
    fn all_rules_consistent_after_add_rule(name in grammar_name()) {
        let mut g = GrammarBuilder::new(&name).build();
        let rule = adze_ir::Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(0),
        };
        g.add_rule(rule);
        prop_assert_eq!(g.all_rules().count(), 1);
        prop_assert!(g.rules.contains_key(&SymbolId(1)));
    }

    // 47
    #[test]
    fn normalize_then_all_rules_nonempty(name in grammar_name()) {
        let mut g = minimal_grammar(&name, "TOK", "root");
        g.normalize();
        prop_assert!(g.all_rules().count() > 0);
    }
}
