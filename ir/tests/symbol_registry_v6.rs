//! SymbolRegistry integration tests via GrammarBuilder.
//!
//! 60+ tests across 8 categories covering the full SymbolRegistry API
//! when constructed through `Grammar::build_registry()`.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId, SymbolRegistry};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a grammar and return its freshly-built registry.
fn registry_for(grammar: &Grammar) -> SymbolRegistry {
    grammar.build_registry()
}

/// Convenience: arithmetic grammar (3 tokens, 2 nonterminals).
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", r"\+")
        .token("*", r"\*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUMBER"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Registry contains all tokens after build (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_tokens_present_number() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("NUMBER").is_some());
}

#[test]
fn test_tokens_present_plus() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("+").is_some());
}

#[test]
fn test_tokens_present_star() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("*").is_some());
}

#[test]
fn test_tokens_metadata_is_terminal() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("NUMBER").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.terminal);
}

#[test]
fn test_tokens_metadata_visible() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("+").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.visible);
}

#[test]
fn test_tokens_three_distinct_ids() {
    let reg = registry_for(&arith_grammar());
    let a = reg.get_id("NUMBER").unwrap();
    let b = reg.get_id("+").unwrap();
    let c = reg.get_id("*").unwrap();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

#[test]
fn test_tokens_single_token_grammar() {
    let g = GrammarBuilder::new("one_tok")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let reg = registry_for(&g);
    assert!(reg.get_id("ID").is_some());
}

#[test]
fn test_tokens_many_tokens() {
    let g = GrammarBuilder::new("many_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("s", vec!["A", "B", "C", "D", "E"])
        .start("s")
        .build();
    let reg = registry_for(&g);
    for name in ["A", "B", "C", "D", "E"] {
        assert!(reg.get_id(name).is_some(), "missing token {name}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Registry contains all nonterminals (rule LHS) after build (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_nonterminals_present_expr() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("expr").is_some());
}

#[test]
fn test_nonterminals_present_term() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("term").is_some());
}

#[test]
fn test_nonterminals_metadata_not_terminal() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
}

#[test]
fn test_nonterminals_metadata_named() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("term").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.named);
}

#[test]
fn test_nonterminals_distinct_from_tokens() {
    let reg = registry_for(&arith_grammar());
    let tok_id = reg.get_id("NUMBER").unwrap();
    let nt_id = reg.get_id("expr").unwrap();
    assert_ne!(tok_id, nt_id);
}

#[test]
fn test_nonterminals_single_rule() {
    let g = GrammarBuilder::new("single_nt")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let reg = registry_for(&g);
    assert!(reg.get_id("root").is_some());
}

#[test]
fn test_nonterminals_many_rules() {
    let g = GrammarBuilder::new("many_nt")
        .token("X", "x")
        .rule("alpha", vec!["X"])
        .rule("beta", vec!["alpha"])
        .rule("gamma", vec!["beta"])
        .rule("delta", vec!["gamma"])
        .start("alpha")
        .build();
    let reg = registry_for(&g);
    for name in ["alpha", "beta", "gamma", "delta"] {
        assert!(reg.get_id(name).is_some(), "missing nonterminal {name}");
    }
}

#[test]
fn test_nonterminals_visible_by_default() {
    let g = GrammarBuilder::new("vis")
        .token("T", "t")
        .rule("visible_rule", vec!["T"])
        .start("visible_rule")
        .build();
    let reg = registry_for(&g);
    let id = reg.get_id("visible_rule").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.visible);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Name-to-ID lookup is consistent (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_name_to_id_deterministic() {
    let g = arith_grammar();
    let r1 = registry_for(&g);
    let r2 = registry_for(&g);
    assert_eq!(r1.get_id("NUMBER"), r2.get_id("NUMBER"));
}

#[test]
fn test_name_to_id_all_tokens_deterministic() {
    let g = arith_grammar();
    let r1 = registry_for(&g);
    let r2 = registry_for(&g);
    for name in ["+", "*", "NUMBER"] {
        assert_eq!(r1.get_id(name), r2.get_id(name));
    }
}

#[test]
fn test_name_to_id_all_nonterminals_deterministic() {
    let g = arith_grammar();
    let r1 = registry_for(&g);
    let r2 = registry_for(&g);
    for name in ["expr", "term"] {
        assert_eq!(r1.get_id(name), r2.get_id(name));
    }
}

#[test]
fn test_name_to_id_missing_returns_none() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("nonexistent").is_none());
}

#[test]
fn test_name_to_id_eof_always_zero() {
    let reg = registry_for(&arith_grammar());
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_name_to_id_case_sensitive() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("number").is_none()); // "NUMBER" exists, "number" does not
}

#[test]
fn test_name_to_id_empty_string_returns_none() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_id("").is_none());
}

#[test]
fn test_name_to_id_contains_id_roundtrip() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("NUMBER").unwrap();
    assert!(reg.contains_id(id));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. ID-to-name lookup is consistent (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_id_to_name_eof() {
    let reg = registry_for(&arith_grammar());
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn test_id_to_name_token() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("NUMBER").unwrap();
    assert_eq!(reg.get_name(id), Some("NUMBER"));
}

#[test]
fn test_id_to_name_nonterminal() {
    let reg = registry_for(&arith_grammar());
    let id = reg.get_id("expr").unwrap();
    assert_eq!(reg.get_name(id), Some("expr"));
}

#[test]
fn test_id_to_name_roundtrip_all() {
    let reg = registry_for(&arith_grammar());
    for name in ["end", "NUMBER", "+", "*", "expr", "term"] {
        let id = reg.get_id(name).unwrap();
        assert_eq!(reg.get_name(id), Some(name));
    }
}

#[test]
fn test_id_to_name_missing_id_returns_none() {
    let reg = registry_for(&arith_grammar());
    assert!(reg.get_name(SymbolId(9999)).is_none());
}

#[test]
fn test_id_to_name_deterministic() {
    let g = arith_grammar();
    let r1 = registry_for(&g);
    let r2 = registry_for(&g);
    let id1 = r1.get_id("+").unwrap();
    let id2 = r2.get_id("+").unwrap();
    assert_eq!(r1.get_name(id1), r2.get_name(id2));
}

#[test]
fn test_id_to_name_every_id_has_name() {
    let reg = registry_for(&arith_grammar());
    for (_name, info) in reg.iter() {
        assert!(reg.get_name(info.id).is_some());
    }
}

#[test]
fn test_id_to_name_unique_names() {
    let reg = registry_for(&arith_grammar());
    let names: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    let mut dedup = names.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(names.len(), dedup.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Registry after normalize (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_normalize_preserves_tokens() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    assert!(reg.get_id("NUMBER").is_some());
}

#[test]
fn test_normalize_preserves_nonterminals() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    assert!(reg.get_id("expr").is_some());
}

#[test]
fn test_normalize_preserves_eof() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_normalize_name_to_id_roundtrip() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    for name in ["NUMBER", "+", "*", "expr", "term"] {
        let id = reg.get_id(name).unwrap();
        assert_eq!(reg.get_name(id), Some(name));
    }
}

#[test]
fn test_normalize_token_metadata_unchanged() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    let id = reg.get_id("NUMBER").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.terminal);
}

#[test]
fn test_normalize_nonterminal_metadata_unchanged() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
    assert!(meta.named);
}

#[test]
fn test_normalize_registry_not_empty() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    assert!(!reg.is_empty());
}

#[test]
fn test_normalize_registry_at_least_original_size() {
    let g = arith_grammar();
    let pre_size = registry_for(&g).len();
    let mut g2 = arith_grammar();
    let _aux = g2.normalize();
    let post_size = registry_for(&g2).len();
    // normalize may only add auxiliary rules; token/nonterminal count is >= original
    assert!(post_size >= pre_size);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Registry after optimize (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_optimize_preserves_tokens() {
    let mut g = arith_grammar();
    g.optimize();
    let reg = registry_for(&g);
    assert!(reg.get_id("NUMBER").is_some());
}

#[test]
fn test_optimize_preserves_nonterminals() {
    let mut g = arith_grammar();
    g.optimize();
    let reg = registry_for(&g);
    assert!(reg.get_id("expr").is_some());
}

#[test]
fn test_optimize_preserves_eof() {
    let mut g = arith_grammar();
    g.optimize();
    let reg = registry_for(&g);
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_optimize_roundtrip() {
    let mut g = arith_grammar();
    g.optimize();
    let reg = registry_for(&g);
    for name in ["NUMBER", "+", "*", "expr", "term"] {
        let id = reg.get_id(name).unwrap();
        assert_eq!(reg.get_name(id), Some(name));
    }
}

#[test]
fn test_optimize_metadata_terminals() {
    let mut g = arith_grammar();
    g.optimize();
    let reg = registry_for(&g);
    for name in ["NUMBER", "+", "*"] {
        let id = reg.get_id(name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(meta.terminal, "{name} should be terminal after optimize");
    }
}

#[test]
fn test_optimize_metadata_nonterminals() {
    let mut g = arith_grammar();
    g.optimize();
    let reg = registry_for(&g);
    for name in ["expr", "term"] {
        let id = reg.get_id(name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(
            !meta.terminal,
            "{name} should be nonterminal after optimize"
        );
    }
}

#[test]
fn test_optimize_size_unchanged() {
    let g = arith_grammar();
    let pre = registry_for(&g).len();
    let mut g2 = arith_grammar();
    g2.optimize();
    let post = registry_for(&g2).len();
    assert_eq!(pre, post);
}

#[test]
fn test_optimize_then_normalize_preserves_all() {
    let mut g = arith_grammar();
    g.optimize();
    let _aux = g.normalize();
    let reg = registry_for(&g);
    for name in ["NUMBER", "+", "*", "expr", "term"] {
        assert!(reg.get_id(name).is_some(), "missing {name} after opt+norm");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Registry size matches expected (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_size_arith_grammar() {
    // end(1) + 3 tokens + 2 nonterminals = 6
    let reg = registry_for(&arith_grammar());
    assert_eq!(reg.len(), 6);
}

#[test]
fn test_size_one_token_one_rule() {
    // end(1) + 1 token + 1 nonterminal = 3
    let g = GrammarBuilder::new("tiny")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let reg = registry_for(&g);
    assert_eq!(reg.len(), 3);
}

#[test]
fn test_size_multiple_tokens_only() {
    // end(1) + 4 tokens + 1 nonterminal (from rule) = 6
    let g = GrammarBuilder::new("tok_only")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("s", vec!["A", "B", "C", "D"])
        .start("s")
        .build();
    let reg = registry_for(&g);
    assert_eq!(reg.len(), 6);
}

#[test]
fn test_size_matches_iter_count() {
    let reg = registry_for(&arith_grammar());
    assert_eq!(reg.len(), reg.iter().count());
}

#[test]
fn test_size_index_map_len() {
    let reg = registry_for(&arith_grammar());
    assert_eq!(reg.len(), reg.to_index_map().len());
}

#[test]
fn test_size_symbol_map_len() {
    let reg = registry_for(&arith_grammar());
    assert_eq!(reg.len(), reg.to_symbol_map().len());
}

#[test]
fn test_size_five_nonterminals() {
    // end(1) + 1 token + 5 nonterminals = 7
    let g = GrammarBuilder::new("five_nt")
        .token("X", "x")
        .rule("a", vec!["X"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .rule("e", vec!["d"])
        .start("a")
        .build();
    let reg = registry_for(&g);
    assert_eq!(reg.len(), 7);
}

#[test]
fn test_size_after_double_build() {
    let g = arith_grammar();
    let r1 = registry_for(&g);
    let r2 = registry_for(&g);
    assert_eq!(r1.len(), r2.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Edge cases: empty grammar, single symbol, many symbols (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_edge_empty_grammar_has_eof() {
    let g = GrammarBuilder::new("empty").build();
    let reg = registry_for(&g);
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_edge_empty_grammar_size_is_one() {
    let g = GrammarBuilder::new("empty").build();
    let reg = registry_for(&g);
    // Only EOF ("end") symbol
    assert_eq!(reg.len(), 1);
}

#[test]
fn test_edge_empty_grammar_not_empty() {
    // Registry always has EOF, so is_empty is false
    let g = GrammarBuilder::new("empty").build();
    let reg = registry_for(&g);
    assert!(!reg.is_empty());
}

#[test]
fn test_edge_single_token_no_rule() {
    let g = GrammarBuilder::new("lone_tok")
        .token("ONLY", "only")
        .build();
    let reg = registry_for(&g);
    // end + ONLY = 2
    assert_eq!(reg.len(), 2);
    assert!(reg.get_id("ONLY").is_some());
}

#[test]
fn test_edge_many_symbols() {
    let mut builder = GrammarBuilder::new("large");
    let mut rhs = Vec::new();
    for i in 0..20 {
        let tok_name = format!("T{i}");
        // Build chain: each token added, then used in rule rhs
        builder = builder.token(&tok_name, &format!("t{i}"));
        rhs.push(tok_name);
    }
    let rhs_refs: Vec<&str> = rhs.iter().map(|s| s.as_str()).collect();
    builder = builder.rule("mega", rhs_refs).start("mega");
    let g = builder.build();
    let reg = registry_for(&g);
    // end(1) + 20 tokens + 1 nonterminal = 22
    assert_eq!(reg.len(), 22);
}

#[test]
fn test_edge_eof_cannot_be_overwritten() {
    // Building a registry always starts with "end" = SymbolId(0)
    let g = arith_grammar();
    let reg = registry_for(&g);
    let eof_id = reg.get_id("end").unwrap();
    assert_eq!(eof_id, SymbolId(0));
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn test_edge_iter_order_starts_with_eof() {
    let reg = registry_for(&arith_grammar());
    let first = reg.iter().next().unwrap();
    assert_eq!(first.0, "end");
    assert_eq!(first.1.id, SymbolId(0));
}

#[test]
fn test_edge_get_or_build_registry_caches() {
    let mut g = arith_grammar();
    // First call builds it
    let len1 = g.get_or_build_registry().len();
    // Second call returns cached version
    let len2 = g.get_or_build_registry().len();
    assert_eq!(len1, len2);
}
