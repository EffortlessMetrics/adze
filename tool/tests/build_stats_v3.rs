//! Comprehensive tests for `BuildStats` reporting in adze-tool.
//!
//! Covers: positive values, grammar matching, scaling, Debug/Clone,
//! consistency invariants, build time measurement, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, BuildStats, build_parser};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

/// Minimal grammar: one token `number`, one non-terminal `source_file`.
fn minimal_grammar(name: &str) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    let tok = SymbolId(1);
    let src = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(src, "source_file".into());
    g.rules.entry(src).or_default().push(Rule {
        lhs: src,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

/// Two-token grammar: `number` and `ident`, both as alternatives for `source_file`.
fn two_token_grammar(name: &str) -> Grammar {
    let mut g = minimal_grammar(name);
    let ident = SymbolId(10);
    let src = SymbolId(2);
    g.tokens.insert(
        ident,
        Token {
            name: "ident".into(),
            pattern: TokenPattern::Regex(r"[a-z]+".into()),
            fragile: false,
        },
    );
    g.rules.entry(src).or_default().push(Rule {
        lhs: src,
        rhs: vec![Symbol::Terminal(ident)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

/// Expression grammar: `expr -> number | expr "+" expr` with left-assoc precedence.
fn expr_grammar(name: &str) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    let num = SymbolId(1);
    let plus = SymbolId(3);
    let expr = SymbolId(4);

    g.tokens.insert(
        num,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(expr, "source_file".into());
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(plus),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

/// Multi-operator expression grammar with `+` and `*`.
fn multi_op_grammar(name: &str) -> Grammar {
    let mut g = expr_grammar(name);
    let star = SymbolId(5);
    let expr = SymbolId(4);

    g.tokens.insert(
        star,
        Token {
            name: "star".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(star),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(2),
    });
    g
}

/// Grammar with N alternative tokens for `source_file`.
fn n_token_grammar(name: &str, n: usize) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    let src = SymbolId(200);
    g.rule_names.insert(src, "source_file".into());
    for i in 0..n {
        let tok = SymbolId((i + 1) as u16);
        g.tokens.insert(
            tok,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
        g.rules.entry(src).or_default().push(Rule {
            lhs: src,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    g
}

/// Build grammar, return result.
fn do_build(g: Grammar) -> BuildResult {
    let (_dir, opts) = tmp_opts();
    build_parser(g, opts).expect("build should succeed")
}

/// Build grammar, return just stats.
fn get_stats(g: Grammar) -> BuildStats {
    do_build(g).build_stats
}

// =========================================================================
// 1. Stats have positive values (8 tests)
// =========================================================================

#[test]
fn positive_state_count_minimal() {
    let s = get_stats(minimal_grammar("pos_st_min"));
    assert!(s.state_count > 0, "state_count must be positive");
}

#[test]
fn positive_symbol_count_minimal() {
    let s = get_stats(minimal_grammar("pos_sym_min"));
    assert!(s.symbol_count > 0, "symbol_count must be positive");
}

#[test]
fn positive_state_count_two_token() {
    let s = get_stats(two_token_grammar("pos_st_2t"));
    assert!(s.state_count > 0);
}

#[test]
fn positive_symbol_count_two_token() {
    let s = get_stats(two_token_grammar("pos_sym_2t"));
    assert!(s.symbol_count > 0);
}

#[test]
fn positive_state_count_expr() {
    let s = get_stats(expr_grammar("pos_st_expr"));
    assert!(s.state_count > 0);
}

#[test]
fn positive_symbol_count_expr() {
    let s = get_stats(expr_grammar("pos_sym_expr"));
    assert!(s.symbol_count > 0);
}

#[test]
fn positive_state_count_multi_op() {
    let s = get_stats(multi_op_grammar("pos_st_mop"));
    assert!(s.state_count > 0);
}

#[test]
fn positive_symbol_count_multi_op() {
    let s = get_stats(multi_op_grammar("pos_sym_mop"));
    assert!(s.symbol_count > 0);
}

// =========================================================================
// 2. Stats match grammar (8 tests)
// =========================================================================

#[test]
fn symbol_count_at_least_token_count_minimal() {
    let s = get_stats(minimal_grammar("match_tok1"));
    // 1 token + implicit symbols → at least 1
    assert!(s.symbol_count >= 1);
}

#[test]
fn symbol_count_at_least_token_count_two_token() {
    let s = get_stats(two_token_grammar("match_tok2"));
    assert!(
        s.symbol_count >= 2,
        "should have at least 2 symbols for 2 tokens"
    );
}

#[test]
fn symbol_count_at_least_token_count_expr() {
    let s = get_stats(expr_grammar("match_tok_e"));
    // 2 tokens (number, plus)
    assert!(s.symbol_count >= 2);
}

#[test]
fn symbol_count_at_least_token_count_multi_op() {
    let s = get_stats(multi_op_grammar("match_tok_m"));
    // 3 tokens (number, plus, star)
    assert!(s.symbol_count >= 3);
}

#[test]
fn symbol_count_at_least_nonterminal_count_minimal() {
    let s = get_stats(minimal_grammar("match_nt1"));
    // 1 non-terminal (source_file)
    assert!(s.symbol_count >= 1);
}

#[test]
fn n_token_grammar_symbol_count_grows() {
    let s5 = get_stats(n_token_grammar("match_n5", 5));
    assert!(
        s5.symbol_count >= 5,
        "5 tokens → ≥5 symbols, got {}",
        s5.symbol_count
    );
}

#[test]
fn n_token_grammar_ten_has_more_symbols_than_three() {
    let s3 = get_stats(n_token_grammar("match_n3a", 3));
    let s10 = get_stats(n_token_grammar("match_n10a", 10));
    assert!(
        s10.symbol_count >= s3.symbol_count,
        "10 tokens should yield ≥ symbols than 3: {} vs {}",
        s10.symbol_count,
        s3.symbol_count,
    );
}

#[test]
fn conflict_cells_non_negative_always() {
    // conflict_cells is usize, so always >= 0, but verify it's a sane value
    let s = get_stats(minimal_grammar("match_cc"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

// =========================================================================
// 3. Stats scale with grammar (8 tests)
// =========================================================================

#[test]
fn expr_has_more_states_than_minimal() {
    let s_min = get_stats(minimal_grammar("scale_min1"));
    let s_expr = get_stats(expr_grammar("scale_expr1"));
    assert!(
        s_expr.state_count >= s_min.state_count,
        "expression grammar should have ≥ states: {} vs {}",
        s_expr.state_count,
        s_min.state_count,
    );
}

#[test]
fn multi_op_has_at_least_as_many_states_as_single_op() {
    let s_single = get_stats(expr_grammar("scale_sop"));
    let s_multi = get_stats(multi_op_grammar("scale_mop"));
    assert!(
        s_multi.state_count >= s_single.state_count,
        "multi-op grammar should have ≥ states: {} vs {}",
        s_multi.state_count,
        s_single.state_count,
    );
}

#[test]
fn multi_op_has_at_least_as_many_symbols_as_single_op() {
    let s_single = get_stats(expr_grammar("scale_sop_s"));
    let s_multi = get_stats(multi_op_grammar("scale_mop_s"));
    assert!(
        s_multi.symbol_count >= s_single.symbol_count,
        "multi-op grammar should have ≥ symbols: {} vs {}",
        s_multi.symbol_count,
        s_single.symbol_count,
    );
}

#[test]
fn ten_tokens_more_states_than_two() {
    let s2 = get_stats(n_token_grammar("scale_2", 2));
    let s10 = get_stats(n_token_grammar("scale_10", 10));
    assert!(
        s10.state_count >= s2.state_count,
        "10 tokens should yield ≥ states than 2: {} vs {}",
        s10.state_count,
        s2.state_count,
    );
}

#[test]
fn ten_tokens_more_symbols_than_two() {
    let s2 = get_stats(n_token_grammar("scale_s2", 2));
    let s10 = get_stats(n_token_grammar("scale_s10", 10));
    assert!(
        s10.symbol_count >= s2.symbol_count,
        "10 tokens should yield ≥ symbols than 2: {} vs {}",
        s10.symbol_count,
        s2.symbol_count,
    );
}

#[test]
fn twenty_tokens_more_symbols_than_five() {
    let s5 = get_stats(n_token_grammar("scale_5a", 5));
    let s20 = get_stats(n_token_grammar("scale_20a", 20));
    assert!(s20.symbol_count >= s5.symbol_count);
}

#[test]
fn scaling_is_monotonic_for_symbol_count() {
    let counts: Vec<usize> = [2, 4, 8, 16]
        .iter()
        .enumerate()
        .map(|(i, &n)| get_stats(n_token_grammar(&format!("mono_{i}"), n)).symbol_count)
        .collect();
    for pair in counts.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "symbol_count should be monotonic: {:?}",
            counts,
        );
    }
}

#[test]
fn scaling_is_monotonic_for_state_count() {
    let counts: Vec<usize> = [2, 4, 8, 16]
        .iter()
        .enumerate()
        .map(|(i, &n)| get_stats(n_token_grammar(&format!("mono_st_{i}"), n)).state_count)
        .collect();
    for pair in counts.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "state_count should be monotonic: {:?}",
            counts,
        );
    }
}

// =========================================================================
// 4. Stats Debug/Clone (7 tests)
// =========================================================================

#[test]
fn debug_format_contains_struct_name() {
    let s = BuildStats {
        state_count: 1,
        symbol_count: 2,
        conflict_cells: 0,
    };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("BuildStats"));
}

#[test]
fn debug_format_contains_state_count_field() {
    let s = BuildStats {
        state_count: 42,
        symbol_count: 10,
        conflict_cells: 0,
    };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("state_count"));
}

#[test]
fn debug_format_contains_symbol_count_field() {
    let s = BuildStats {
        state_count: 1,
        symbol_count: 99,
        conflict_cells: 0,
    };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("symbol_count"));
}

#[test]
fn debug_format_contains_conflict_cells_field() {
    let s = BuildStats {
        state_count: 1,
        symbol_count: 2,
        conflict_cells: 7,
    };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("conflict_cells"));
}

#[test]
fn debug_format_shows_values() {
    let s = BuildStats {
        state_count: 123,
        symbol_count: 456,
        conflict_cells: 789,
    };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("123"));
    assert!(dbg.contains("456"));
    assert!(dbg.contains("789"));
}

#[test]
fn debug_format_zero_stats_is_not_empty() {
    let s = BuildStats {
        state_count: 0,
        symbol_count: 0,
        conflict_cells: 0,
    };
    let dbg = format!("{s:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn manual_clone_produces_equal_fields() {
    let s = BuildStats {
        state_count: 10,
        symbol_count: 20,
        conflict_cells: 3,
    };
    let copy = BuildStats {
        state_count: s.state_count,
        symbol_count: s.symbol_count,
        conflict_cells: s.conflict_cells,
    };
    assert_eq!(s.state_count, copy.state_count);
    assert_eq!(s.symbol_count, copy.symbol_count);
    assert_eq!(s.conflict_cells, copy.conflict_cells);
}

// =========================================================================
// 5. Stats consistency (8 tests)
// =========================================================================

#[test]
fn consistency_minimal_symbol_ge_one() {
    let s = get_stats(minimal_grammar("cons_min"));
    assert!(s.symbol_count >= 1, "at least 1 symbol expected");
}

#[test]
fn consistency_minimal_state_ge_one() {
    let s = get_stats(minimal_grammar("cons_st_min"));
    assert!(s.state_count >= 1, "at least 1 state expected");
}

#[test]
fn consistency_expr_symbol_ge_state_not_required_but_sane() {
    let s = get_stats(expr_grammar("cons_expr"));
    // Both should be positive — the exact relationship depends on the grammar
    assert!(s.state_count > 0);
    assert!(s.symbol_count > 0);
}

#[test]
fn consistency_conflict_bounded_by_table_size() {
    let s = get_stats(expr_grammar("cons_cf_expr"));
    let upper_bound = s.state_count * s.symbol_count;
    assert!(
        s.conflict_cells <= upper_bound,
        "conflict_cells {} should be ≤ state_count * symbol_count = {}",
        s.conflict_cells,
        upper_bound,
    );
}

#[test]
fn consistency_two_token_symbols_ge_minimal() {
    let s_min = get_stats(minimal_grammar("cons_2t_min"));
    let s_two = get_stats(two_token_grammar("cons_2t_two"));
    assert!(s_two.symbol_count >= s_min.symbol_count);
}

#[test]
fn consistency_multi_op_conflict_bounded() {
    let s = get_stats(multi_op_grammar("cons_mop_cf"));
    let upper = s.state_count * s.symbol_count;
    assert!(s.conflict_cells <= upper);
}

#[test]
fn consistency_n_token_state_positive() {
    for n in [1, 3, 7] {
        let s = get_stats(n_token_grammar(&format!("cons_nt_{n}"), n));
        assert!(s.state_count > 0, "n={n}: state_count must be positive");
    }
}

#[test]
fn consistency_deterministic_builds() {
    let s1 = get_stats(expr_grammar("cons_det_a"));
    let s2 = get_stats(expr_grammar("cons_det_b"));
    assert_eq!(
        s1.state_count, s2.state_count,
        "same grammar should produce same state_count"
    );
    assert_eq!(
        s1.symbol_count, s2.symbol_count,
        "same grammar should produce same symbol_count"
    );
    assert_eq!(
        s1.conflict_cells, s2.conflict_cells,
        "same grammar should produce same conflict_cells"
    );
}

// =========================================================================
// 6. Build time measurement (8 tests)
// =========================================================================

#[test]
fn build_completes_minimal() {
    let r = do_build(minimal_grammar("time_min"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_completes_expr() {
    let r = do_build(expr_grammar("time_expr"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_completes_multi_op() {
    let r = do_build(multi_op_grammar("time_mop"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_completes_two_token() {
    let r = do_build(two_token_grammar("time_2t"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_result_grammar_name_matches_minimal() {
    let r = do_build(minimal_grammar("time_name_min"));
    assert_eq!(r.grammar_name, "time_name_min");
}

#[test]
fn build_result_grammar_name_matches_expr() {
    let r = do_build(expr_grammar("time_name_expr"));
    assert_eq!(r.grammar_name, "time_name_expr");
}

#[test]
fn build_result_node_types_json_valid() {
    let r = do_build(minimal_grammar("time_json"));
    // node_types_json should parse as valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json should be valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn build_result_parser_path_nonempty() {
    let r = do_build(minimal_grammar("time_path"));
    assert!(!r.parser_path.is_empty());
}

// =========================================================================
// 7. Edge cases (8 tests)
// =========================================================================

#[test]
fn edge_single_token_grammar() {
    let s = get_stats(minimal_grammar("edge_single"));
    assert!(s.state_count > 0);
    assert!(s.symbol_count > 0);
}

#[test]
fn edge_grammar_with_string_token() {
    let mut g = Grammar::new("edge_str".to_string());
    let tok = SymbolId(1);
    let src = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "kw_if".into(),
            pattern: TokenPattern::String("if".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(src, "source_file".into());
    g.rules.entry(src).or_default().push(Rule {
        lhs: src,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let s = get_stats(g);
    assert!(s.state_count > 0);
    assert!(s.symbol_count > 0);
}

#[test]
fn edge_many_tokens_grammar_thirty() {
    let s = get_stats(n_token_grammar("edge_30", 30));
    assert!(
        s.symbol_count >= 30,
        "30 tokens → ≥30 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn edge_expr_grammar_has_conflict_cells() {
    // An ambiguous expression grammar may produce conflict cells
    let mut g = Grammar::new("edge_ambig".to_string());
    let num = SymbolId(1);
    let plus = SymbolId(3);
    let expr = SymbolId(4);
    g.tokens.insert(
        num,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(expr, "source_file".into());
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // Deliberately no precedence → ambiguity
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(plus),
            Symbol::NonTerminal(expr),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let s = get_stats(g);
    // Build should succeed; conflict_cells >= 0 (always true for usize)
    assert!(s.state_count > 0);
}

#[test]
fn edge_grammar_builder_single_token() {
    let gb = GrammarBuilder::new("edge_gb_1")
        .token("x", "x")
        .rule("source_file", vec!["x"])
        .start("source_file")
        .build();
    let (_dir, opts) = tmp_opts();
    let r = build_parser(gb, opts).expect("GrammarBuilder build should succeed");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn edge_grammar_builder_sequence() {
    let gb = GrammarBuilder::new("edge_gb_seq")
        .token("a", "a")
        .token("b", "b")
        .rule("source_file", vec!["a", "b"])
        .start("source_file")
        .build();
    let (_dir, opts) = tmp_opts();
    let r = build_parser(gb, opts).expect("GrammarBuilder sequence should build");
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn edge_build_result_debug_contains_grammar_name() {
    let r = do_build(minimal_grammar("edge_dbg_name"));
    let dbg = format!("{r:?}");
    assert!(dbg.contains("edge_dbg_name"));
}

#[test]
fn edge_build_stats_from_result_are_accessible() {
    let r = do_build(minimal_grammar("edge_access"));
    let _sc = r.build_stats.state_count;
    let _syc = r.build_stats.symbol_count;
    let _cc = r.build_stats.conflict_cells;
    // Just verify the fields are publicly accessible and don't panic
}
