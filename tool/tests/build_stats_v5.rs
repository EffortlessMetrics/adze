//! Tests for `BuildStats` and build output analysis in adze-tool.
//!
//! 64 tests across 8 categories:
//! 1. BuildStats fields are non-negative
//! 2. Symbol count reflects grammar tokens
//! 3. State count from parse table
//! 4. Symbol count covers tokens and nonterminals
//! 5. Complex grammars produce higher counts
//! 6. BuildStats correlates with grammar complexity
//! 7. parser_code length scales with grammar size
//! 8. Edge cases: minimal, large, and special grammars

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

/// Two-token grammar: `number` and `ident`, both alternatives for `source_file`.
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

/// Multi-operator grammar: extends expr with `*`.
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

/// Three-operator grammar: extends multi_op with `-`.
fn three_op_grammar(name: &str) -> Grammar {
    let mut g = multi_op_grammar(name);
    let minus = SymbolId(6);
    let expr = SymbolId(4);
    g.tokens.insert(
        minus,
        Token {
            name: "minus".into(),
            pattern: TokenPattern::String("-".into()),
            fragile: false,
        },
    );
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(minus),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(3),
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

/// Sequence grammar: `source_file -> tok_0 tok_1 ... tok_{n-1}`.
fn sequence_grammar(name: &str, n: usize) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    let src = SymbolId(200);
    g.rule_names.insert(src, "source_file".into());
    let mut rhs = Vec::new();
    for i in 0..n {
        let tok = SymbolId((i + 1) as u16);
        g.tokens.insert(
            tok,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("s{i}")),
                fragile: false,
            },
        );
        rhs.push(Symbol::Terminal(tok));
    }
    g.rules.entry(src).or_default().push(Rule {
        lhs: src,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn do_build(g: Grammar) -> BuildResult {
    let (_dir, opts) = tmp_opts();
    build_parser(g, opts).expect("build should succeed")
}

fn get_stats(g: Grammar) -> BuildStats {
    do_build(g).build_stats
}

// =========================================================================
// 1. BuildStats fields are non-negative (8 tests)
// =========================================================================

#[test]
fn nonneg_state_count_minimal() {
    let s = get_stats(minimal_grammar("nn_st_min"));
    assert!(
        s.state_count > 0,
        "state_count must be positive for any grammar"
    );
}

#[test]
fn nonneg_symbol_count_minimal() {
    let s = get_stats(minimal_grammar("nn_sym_min"));
    assert!(s.symbol_count > 0, "symbol_count must be positive");
}

#[test]
fn nonneg_conflict_cells_minimal() {
    let s = get_stats(minimal_grammar("nn_cc_min"));
    // usize is always >= 0; verify it's bounded
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

#[test]
fn nonneg_state_count_two_token() {
    let s = get_stats(two_token_grammar("nn_st_2t"));
    assert!(s.state_count > 0);
}

#[test]
fn nonneg_symbol_count_two_token() {
    let s = get_stats(two_token_grammar("nn_sym_2t"));
    assert!(s.symbol_count > 0);
}

#[test]
fn nonneg_state_count_expr() {
    let s = get_stats(expr_grammar("nn_st_expr"));
    assert!(s.state_count > 0);
}

#[test]
fn nonneg_symbol_count_expr() {
    let s = get_stats(expr_grammar("nn_sym_expr"));
    assert!(s.symbol_count > 0);
}

#[test]
fn nonneg_all_fields_multi_op() {
    let s = get_stats(multi_op_grammar("nn_all_mop"));
    assert!(s.state_count > 0);
    assert!(s.symbol_count > 0);
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

// =========================================================================
// 2. Symbol count reflects grammar tokens (8 tests)
// =========================================================================

#[test]
fn symcount_ge_token_count_minimal() {
    let g = minimal_grammar("tc_min");
    let n_tokens = g.tokens.len();
    let s = get_stats(g);
    assert!(
        s.symbol_count >= n_tokens,
        "symbol_count {} must be >= token count {}",
        s.symbol_count,
        n_tokens,
    );
}

#[test]
fn symcount_ge_token_count_two_token() {
    let g = two_token_grammar("tc_2t");
    let n_tokens = g.tokens.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens);
}

#[test]
fn symcount_ge_token_count_expr() {
    let g = expr_grammar("tc_expr");
    let n_tokens = g.tokens.len();
    let s = get_stats(g);
    assert!(
        s.symbol_count >= n_tokens,
        "expr: symbol_count {} < token count {}",
        s.symbol_count,
        n_tokens,
    );
}

#[test]
fn symcount_ge_token_count_multi_op() {
    let g = multi_op_grammar("tc_mop");
    let n_tokens = g.tokens.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens);
}

#[test]
fn symcount_ge_five_for_five_tokens() {
    let s = get_stats(n_token_grammar("tc_n5", 5));
    assert!(
        s.symbol_count >= 5,
        "5 tokens → ≥5 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn symcount_ge_ten_for_ten_tokens() {
    let s = get_stats(n_token_grammar("tc_n10", 10));
    assert!(
        s.symbol_count >= 10,
        "10 tokens → ≥10 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn symcount_ge_twenty_for_twenty_tokens() {
    let s = get_stats(n_token_grammar("tc_n20", 20));
    assert!(
        s.symbol_count >= 20,
        "20 tokens → ≥20 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn symcount_ge_token_count_three_op() {
    let g = three_op_grammar("tc_3op");
    let n_tokens = g.tokens.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens);
}

// =========================================================================
// 3. State count from parse table (8 tests)
// =========================================================================

#[test]
fn states_minimal_at_least_two() {
    // Even the simplest grammar needs at least an initial and accept state
    let s = get_stats(minimal_grammar("st_min"));
    assert!(
        s.state_count >= 2,
        "minimal grammar needs ≥2 states, got {}",
        s.state_count
    );
}

#[test]
fn states_two_token_at_least_two() {
    let s = get_stats(two_token_grammar("st_2t"));
    assert!(s.state_count >= 2);
}

#[test]
fn states_expr_at_least_three() {
    let s = get_stats(expr_grammar("st_expr"));
    assert!(
        s.state_count >= 3,
        "expr grammar (with recursion) needs ≥3 states, got {}",
        s.state_count,
    );
}

#[test]
fn states_multi_op_at_least_expr() {
    let s_expr = get_stats(expr_grammar("st_expr2"));
    let s_multi = get_stats(multi_op_grammar("st_mop"));
    assert!(
        s_multi.state_count >= s_expr.state_count,
        "multi-op ≥ single-op states: {} vs {}",
        s_multi.state_count,
        s_expr.state_count,
    );
}

#[test]
fn states_deterministic_same_grammar() {
    let s1 = get_stats(expr_grammar("st_det_a"));
    let s2 = get_stats(expr_grammar("st_det_b"));
    assert_eq!(s1.state_count, s2.state_count);
}

#[test]
fn states_sequence_grows_with_length() {
    let s2 = get_stats(sequence_grammar("st_seq2", 2));
    let s5 = get_stats(sequence_grammar("st_seq5", 5));
    assert!(
        s5.state_count >= s2.state_count,
        "longer sequence → more states: {} vs {}",
        s5.state_count,
        s2.state_count,
    );
}

#[test]
fn states_n_tokens_grows() {
    let s3 = get_stats(n_token_grammar("st_n3", 3));
    let s15 = get_stats(n_token_grammar("st_n15", 15));
    assert!(s15.state_count >= s3.state_count);
}

#[test]
fn states_three_op_at_least_multi_op() {
    let s_multi = get_stats(multi_op_grammar("st_mop2"));
    let s_three = get_stats(three_op_grammar("st_3op"));
    assert!(
        s_three.state_count >= s_multi.state_count,
        "three-op ≥ multi-op states: {} vs {}",
        s_three.state_count,
        s_multi.state_count,
    );
}

// =========================================================================
// 4. Symbol count covers tokens and nonterminals (8 tests)
// =========================================================================

#[test]
fn sym_covers_tokens_and_nonterminals_minimal() {
    let g = minimal_grammar("cv_min");
    let n_tokens = g.tokens.len();
    let n_rules = g.rule_names.len();
    let s = get_stats(g);
    assert!(
        s.symbol_count >= n_tokens + n_rules,
        "symbol_count {} < tokens({}) + nonterminals({})",
        s.symbol_count,
        n_tokens,
        n_rules,
    );
}

#[test]
fn sym_covers_tokens_and_nonterminals_two_token() {
    let g = two_token_grammar("cv_2t");
    let n_tokens = g.tokens.len();
    let n_rules = g.rule_names.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens + n_rules);
}

#[test]
fn sym_covers_tokens_and_nonterminals_expr() {
    let g = expr_grammar("cv_expr");
    let n_tokens = g.tokens.len();
    let n_rules = g.rule_names.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens + n_rules);
}

#[test]
fn sym_covers_tokens_and_nonterminals_multi_op() {
    let g = multi_op_grammar("cv_mop");
    let n_tokens = g.tokens.len();
    let n_rules = g.rule_names.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens + n_rules);
}

#[test]
fn sym_count_deterministic() {
    let s1 = get_stats(multi_op_grammar("cv_det_a"));
    let s2 = get_stats(multi_op_grammar("cv_det_b"));
    assert_eq!(s1.symbol_count, s2.symbol_count);
}

#[test]
fn sym_count_more_tokens_more_symbols() {
    let s5 = get_stats(n_token_grammar("cv_n5", 5));
    let s10 = get_stats(n_token_grammar("cv_n10", 10));
    assert!(
        s10.symbol_count > s5.symbol_count,
        "10 tokens should have more symbols than 5: {} vs {}",
        s10.symbol_count,
        s5.symbol_count,
    );
}

#[test]
fn sym_count_monotonic_with_tokens() {
    let counts: Vec<usize> = [2, 5, 10, 20]
        .iter()
        .enumerate()
        .map(|(i, &n)| get_stats(n_token_grammar(&format!("cv_mono_{i}"), n)).symbol_count)
        .collect();
    for pair in counts.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "symbol_count should be monotonic: {counts:?}",
        );
    }
}

#[test]
fn sym_covers_tokens_and_nonterminals_three_op() {
    let g = three_op_grammar("cv_3op");
    let n_tokens = g.tokens.len();
    let n_rules = g.rule_names.len();
    let s = get_stats(g);
    assert!(s.symbol_count >= n_tokens + n_rules);
}

// =========================================================================
// 5. Complex grammars produce higher counts (8 tests)
// =========================================================================

#[test]
fn complex_expr_more_states_than_minimal() {
    let s_min = get_stats(minimal_grammar("cx_min"));
    let s_expr = get_stats(expr_grammar("cx_expr"));
    assert!(s_expr.state_count > s_min.state_count);
}

#[test]
fn complex_expr_more_symbols_than_minimal() {
    let s_min = get_stats(minimal_grammar("cx_sym_min"));
    let s_expr = get_stats(expr_grammar("cx_sym_expr"));
    assert!(s_expr.symbol_count > s_min.symbol_count);
}

#[test]
fn complex_multi_op_more_symbols_than_expr() {
    let s_expr = get_stats(expr_grammar("cx_expr2"));
    let s_multi = get_stats(multi_op_grammar("cx_mop"));
    assert!(
        s_multi.symbol_count >= s_expr.symbol_count,
        "multi-op symbols {} < expr symbols {}",
        s_multi.symbol_count,
        s_expr.symbol_count,
    );
}

#[test]
fn complex_three_op_more_symbols_than_multi_op() {
    let s_multi = get_stats(multi_op_grammar("cx_mop2"));
    let s_three = get_stats(three_op_grammar("cx_3op"));
    assert!(s_three.symbol_count >= s_multi.symbol_count);
}

#[test]
fn complex_twenty_tokens_more_than_five() {
    let s5 = get_stats(n_token_grammar("cx_n5", 5));
    let s20 = get_stats(n_token_grammar("cx_n20", 20));
    assert!(s20.symbol_count > s5.symbol_count);
    assert!(s20.state_count >= s5.state_count);
}

#[test]
fn complex_sequence_ten_more_states_than_two() {
    let s2 = get_stats(sequence_grammar("cx_seq2", 2));
    let s10 = get_stats(sequence_grammar("cx_seq10", 10));
    assert!(s10.state_count > s2.state_count);
}

#[test]
fn complex_two_token_more_symbols_than_one() {
    let s1 = get_stats(minimal_grammar("cx_1t"));
    let s2 = get_stats(two_token_grammar("cx_2t"));
    assert!(s2.symbol_count >= s1.symbol_count);
}

#[test]
fn complex_thirty_tokens_more_than_ten() {
    let s10 = get_stats(n_token_grammar("cx_n10", 10));
    let s30 = get_stats(n_token_grammar("cx_n30", 30));
    assert!(s30.symbol_count > s10.symbol_count);
}

// =========================================================================
// 6. BuildStats correlates with grammar complexity (8 tests)
// =========================================================================

#[test]
fn corr_conflict_bounded_by_table_size_expr() {
    let s = get_stats(expr_grammar("cor_cf_expr"));
    let upper = s.state_count * s.symbol_count;
    assert!(s.conflict_cells <= upper);
}

#[test]
fn corr_conflict_bounded_by_table_size_multi_op() {
    let s = get_stats(multi_op_grammar("cor_cf_mop"));
    let upper = s.state_count * s.symbol_count;
    assert!(s.conflict_cells <= upper);
}

#[test]
fn corr_conflict_bounded_by_table_size_three_op() {
    let s = get_stats(three_op_grammar("cor_cf_3op"));
    let upper = s.state_count * s.symbol_count;
    assert!(s.conflict_cells <= upper);
}

#[test]
fn corr_state_monotonic_across_sizes() {
    let counts: Vec<usize> = [2, 4, 8, 16]
        .iter()
        .enumerate()
        .map(|(i, &n)| get_stats(n_token_grammar(&format!("cor_sm_{i}"), n)).state_count)
        .collect();
    for pair in counts.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "state_count should be monotonic: {counts:?}"
        );
    }
}

#[test]
fn corr_symbol_monotonic_across_sizes() {
    let counts: Vec<usize> = [2, 4, 8, 16]
        .iter()
        .enumerate()
        .map(|(i, &n)| get_stats(n_token_grammar(&format!("cor_sym_{i}"), n)).symbol_count)
        .collect();
    for pair in counts.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "symbol_count should be monotonic: {counts:?}"
        );
    }
}

#[test]
fn corr_sequence_states_monotonic() {
    let counts: Vec<usize> = [2, 4, 6, 8]
        .iter()
        .enumerate()
        .map(|(i, &n)| get_stats(sequence_grammar(&format!("cor_seq_{i}"), n)).state_count)
        .collect();
    for pair in counts.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "seq state_count should be monotonic: {counts:?}"
        );
    }
}

#[test]
fn corr_conflict_cells_deterministic() {
    let s1 = get_stats(multi_op_grammar("cor_det_a"));
    let s2 = get_stats(multi_op_grammar("cor_det_b"));
    assert_eq!(s1.conflict_cells, s2.conflict_cells);
}

#[test]
fn corr_all_stats_deterministic() {
    let s1 = get_stats(three_op_grammar("cor_all_a"));
    let s2 = get_stats(three_op_grammar("cor_all_b"));
    assert_eq!(s1.state_count, s2.state_count);
    assert_eq!(s1.symbol_count, s2.symbol_count);
    assert_eq!(s1.conflict_cells, s2.conflict_cells);
}

// =========================================================================
// 7. parser_code length scales with grammar size (8 tests)
// =========================================================================

#[test]
fn code_nonempty_minimal() {
    let r = do_build(minimal_grammar("pc_min"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn code_nonempty_expr() {
    let r = do_build(expr_grammar("pc_expr"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn code_nonempty_multi_op() {
    let r = do_build(multi_op_grammar("pc_mop"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn code_expr_longer_than_minimal() {
    let r_min = do_build(minimal_grammar("pc_min2"));
    let r_expr = do_build(expr_grammar("pc_expr2"));
    assert!(
        r_expr.parser_code.len() >= r_min.parser_code.len(),
        "expr code {} should be ≥ minimal code {}",
        r_expr.parser_code.len(),
        r_min.parser_code.len(),
    );
}

#[test]
fn code_multi_op_longer_than_expr() {
    let r_expr = do_build(expr_grammar("pc_expr3"));
    let r_multi = do_build(multi_op_grammar("pc_mop2"));
    assert!(
        r_multi.parser_code.len() >= r_expr.parser_code.len(),
        "multi-op code {} should be ≥ expr code {}",
        r_multi.parser_code.len(),
        r_expr.parser_code.len(),
    );
}

#[test]
fn code_twenty_tokens_longer_than_five() {
    let r5 = do_build(n_token_grammar("pc_n5", 5));
    let r20 = do_build(n_token_grammar("pc_n20", 20));
    assert!(
        r20.parser_code.len() > r5.parser_code.len(),
        "20-token code {} should be > 5-token code {}",
        r20.parser_code.len(),
        r5.parser_code.len(),
    );
}

#[test]
fn code_length_monotonic() {
    let lengths: Vec<usize> = [2, 5, 10, 20]
        .iter()
        .enumerate()
        .map(|(i, &n)| {
            do_build(n_token_grammar(&format!("pc_mono_{i}"), n))
                .parser_code
                .len()
        })
        .collect();
    for pair in lengths.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "parser_code length should be monotonic: {lengths:?}",
        );
    }
}

#[test]
fn code_sequence_longer_than_minimal() {
    let r_min = do_build(minimal_grammar("pc_seq_min"));
    let r_seq = do_build(sequence_grammar("pc_seq5", 5));
    assert!(
        r_seq.parser_code.len() >= r_min.parser_code.len(),
        "sequence code {} should be ≥ minimal code {}",
        r_seq.parser_code.len(),
        r_min.parser_code.len(),
    );
}

// =========================================================================
// 8. Edge cases: minimal, large, and special grammars (8 tests)
// =========================================================================

#[test]
fn edge_string_token_grammar() {
    let mut g = Grammar::new("edge_str_v5".to_string());
    let tok = SymbolId(1);
    let src = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "kw_return".into(),
            pattern: TokenPattern::String("return".into()),
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
fn edge_thirty_token_grammar() {
    let s = get_stats(n_token_grammar("edge_30_v5", 30));
    assert!(s.symbol_count >= 30);
    assert!(s.state_count > 0);
}

#[test]
fn edge_fifty_token_grammar() {
    let s = get_stats(n_token_grammar("edge_50_v5", 50));
    assert!(s.symbol_count >= 50);
}

#[test]
fn edge_node_types_json_valid_minimal() {
    let r = do_build(minimal_grammar("edge_json_v5"));
    let parsed: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json must be valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn edge_node_types_json_valid_expr() {
    let r = do_build(expr_grammar("edge_json_expr_v5"));
    let parsed: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json must be valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

#[test]
fn edge_grammar_name_preserved() {
    let r = do_build(minimal_grammar("edge_name_v5"));
    assert_eq!(r.grammar_name, "edge_name_v5");
}

#[test]
fn edge_parser_path_nonempty() {
    let r = do_build(minimal_grammar("edge_path_v5"));
    assert!(!r.parser_path.is_empty());
}

#[test]
fn edge_debug_build_result_contains_name() {
    let r = do_build(minimal_grammar("edge_dbg_v5"));
    let dbg = format!("{r:?}");
    assert!(dbg.contains("edge_dbg_v5"));
}
