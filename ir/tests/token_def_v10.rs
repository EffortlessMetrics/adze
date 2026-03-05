//! Comprehensive tests for Token and token management in adze-ir Grammar.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Token, TokenPattern};

// ── Helper ──────────────────────────────────────────────────────────────────

fn single_token_grammar(name: &str, tok_name: &str, pattern: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token(tok_name, pattern)
        .rule("start", vec![tok_name])
        .start("start")
        .build()
}

// ── 1–5: Single token basics ────────────────────────────────────────────────

#[test]
fn td_v10_single_token_count() {
    let g = single_token_grammar("td_v10_single_count", "NUM", r"\d+");
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn td_v10_single_token_name() {
    let g = single_token_grammar("td_v10_single_name", "NUM", r"\d+");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.name, "NUM");
}

#[test]
fn td_v10_single_token_pattern_regex() {
    let g = single_token_grammar("td_v10_single_pat", "NUM", r"\d+");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\d+".to_string()));
}

#[test]
fn td_v10_single_token_not_fragile() {
    let g = single_token_grammar("td_v10_single_frag", "NUM", r"\d+");
    let tok = g.tokens.values().next().unwrap();
    assert!(!tok.fragile);
}

#[test]
fn td_v10_single_token_symbol_id_consistent() {
    let g = single_token_grammar("td_v10_single_sid", "NUM", r"\d+");
    let (sid, tok) = g.tokens.iter().next().unwrap();
    assert_eq!(tok.name, "NUM");
    // SymbolId should be retrievable as map key
    assert!(g.tokens.contains_key(sid));
}

// ── 6–10: Token access and indexing ─────────────────────────────────────────

#[test]
fn td_v10_tokens_indexed_by_symbol_id() {
    let g = single_token_grammar("td_v10_idx", "IDENT", r"[a-z]+");
    let sid = *g.tokens.keys().next().unwrap();
    let tok = &g.tokens[&sid];
    assert_eq!(tok.name, "IDENT");
}

#[test]
fn td_v10_token_names_are_unique() {
    let g = GrammarBuilder::new("td_v10_unique")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let names: Vec<_> = g.tokens.values().map(|t| t.name.as_str()).collect();
    let mut dedup = names.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(names.len(), dedup.len());
}

#[test]
fn td_v10_grammar_with_regex_patterns() {
    let g = GrammarBuilder::new("td_v10_regex")
        .token("INT", r"\d+")
        .token("FLOAT", r"\d+\.\d+")
        .rule("start", vec!["INT"])
        .start("start")
        .build();
    for tok in g.tokens.values() {
        assert!(matches!(tok.pattern, TokenPattern::Regex(_)));
    }
}

#[test]
fn td_v10_token_patterns_with_special_chars() {
    let g = GrammarBuilder::new("td_v10_special")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("start", vec!["+"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn td_v10_token_ordering_preserved() {
    let g = GrammarBuilder::new("td_v10_order")
        .token("ALPHA", r"[a-z]+")
        .token("BETA", r"[A-Z]+")
        .token("GAMMA", r"\d+")
        .rule("start", vec!["ALPHA"])
        .start("start")
        .build();
    let names: Vec<_> = g.tokens.values().map(|t| t.name.clone()).collect();
    assert_eq!(names, vec!["ALPHA", "BETA", "GAMMA"]);
}

// ── 11–15: Clone and transforms ─────────────────────────────────────────────

#[test]
fn td_v10_clone_preserves_tokens() {
    let g = GrammarBuilder::new("td_v10_clone")
        .token("X", r"\w+")
        .rule("start", vec!["X"])
        .start("start")
        .build();
    let g2 = g.clone();
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.tokens, g2.tokens);
}

#[test]
fn td_v10_clone_token_names_match() {
    let g = GrammarBuilder::new("td_v10_clone_names")
        .token("FOO", "foo")
        .token("BAR", "bar")
        .rule("start", vec!["FOO"])
        .start("start")
        .build();
    let g2 = g.clone();
    let orig: Vec<_> = g.tokens.values().map(|t| &t.name).collect();
    let cloned: Vec<_> = g2.tokens.values().map(|t| &t.name).collect();
    assert_eq!(orig, cloned);
}

#[test]
fn td_v10_tokens_after_normalize() {
    let mut g = GrammarBuilder::new("td_v10_norm")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();
    let before = g.tokens.len();
    let _ = g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn td_v10_tokens_after_optimize() {
    let mut g = GrammarBuilder::new("td_v10_opt")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();
    let before = g.tokens.len();
    g.optimize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn td_v10_token_access_by_symbol_id_key() {
    let g = GrammarBuilder::new("td_v10_access")
        .token("K", "k")
        .rule("start", vec!["K"])
        .start("start")
        .build();
    for (sid, tok) in &g.tokens {
        assert_eq!(&g.tokens[sid], tok);
    }
}

// ── 16–30: Various token counts ─────────────────────────────────────────────

macro_rules! token_count_test {
    ($name:ident, $grammar_name:expr, $count:expr) => {
        #[test]
        fn $name() {
            let mut b = GrammarBuilder::new($grammar_name);
            for i in 0..$count {
                let tok_name = format!("T{i}");
                let pat = format!("[{i}]+");
                b = b.token(&tok_name, &pat);
            }
            b = b.rule("start", vec!["T0"]).start("start");
            let g = b.build();
            assert_eq!(g.tokens.len(), $count);
        }
    };
}

token_count_test!(td_v10_count_1, "td_v10_cnt1", 1);
token_count_test!(td_v10_count_2, "td_v10_cnt2", 2);
token_count_test!(td_v10_count_3, "td_v10_cnt3", 3);
token_count_test!(td_v10_count_4, "td_v10_cnt4", 4);
token_count_test!(td_v10_count_5, "td_v10_cnt5", 5);
token_count_test!(td_v10_count_6, "td_v10_cnt6", 6);
token_count_test!(td_v10_count_7, "td_v10_cnt7", 7);
token_count_test!(td_v10_count_8, "td_v10_cnt8", 8);
token_count_test!(td_v10_count_9, "td_v10_cnt9", 9);
token_count_test!(td_v10_count_10, "td_v10_cnt10", 10);
token_count_test!(td_v10_count_11, "td_v10_cnt11", 11);
token_count_test!(td_v10_count_12, "td_v10_cnt12", 12);
token_count_test!(td_v10_count_13, "td_v10_cnt13", 13);
token_count_test!(td_v10_count_14, "td_v10_cnt14", 14);
token_count_test!(td_v10_count_15, "td_v10_cnt15", 15);

// ── 31–40: TokenPattern variants ────────────────────────────────────────────

#[test]
fn td_v10_string_pattern_literal() {
    let g = single_token_grammar("td_v10_str_lit", "SEMI", ";");
    let tok = g.tokens.values().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::String(_)));
}

#[test]
fn td_v10_regex_pattern_digits() {
    let g = single_token_grammar("td_v10_rx_dig", "DIGIT", r"\d");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\d".to_string()));
}

#[test]
fn td_v10_regex_pattern_word_chars() {
    let g = single_token_grammar("td_v10_rx_word", "WORD", r"\w+");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\w+".to_string()));
}

#[test]
fn td_v10_regex_pattern_char_class() {
    let g = single_token_grammar("td_v10_rx_class", "HEX", r"[0-9a-fA-F]+");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(
        tok.pattern,
        TokenPattern::Regex(r"[0-9a-fA-F]+".to_string())
    );
}

#[test]
fn td_v10_string_pattern_keyword() {
    let g = single_token_grammar("td_v10_str_kw", "IF", "if");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("if".to_string()));
}

#[test]
fn td_v10_string_pattern_multi_char() {
    let g = single_token_grammar("td_v10_str_mc", "RETURN", "return");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("return".to_string()));
}

#[test]
fn td_v10_regex_pattern_alternation() {
    let g = single_token_grammar("td_v10_rx_alt", "BOOL", r"true|false");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"true|false".to_string()));
}

#[test]
fn td_v10_regex_pattern_quantifier_star() {
    let g = single_token_grammar("td_v10_rx_star", "WS", r"\s*");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\s*".to_string()));
}

#[test]
fn td_v10_regex_pattern_quantifier_question() {
    let g = single_token_grammar("td_v10_rx_q", "OPT", r"\d?");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\d?".to_string()));
}

#[test]
fn td_v10_regex_pattern_dot() {
    let g = single_token_grammar("td_v10_rx_dot", "ANY", r".+");
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r".+".to_string()));
}

// ── 41–50: Fragile tokens ───────────────────────────────────────────────────

#[test]
fn td_v10_fragile_token_flag() {
    let g = GrammarBuilder::new("td_v10_frag_flag")
        .fragile_token("ERR", r"[^\s]+")
        .rule("start", vec!["ERR"])
        .start("start")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

#[test]
fn td_v10_fragile_token_name() {
    let g = GrammarBuilder::new("td_v10_frag_name")
        .fragile_token("ERR", r"[^\s]+")
        .rule("start", vec!["ERR"])
        .start("start")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.name, "ERR");
}

#[test]
fn td_v10_fragile_token_pattern() {
    let g = GrammarBuilder::new("td_v10_frag_pat")
        .fragile_token("ERR", r"[^\s]+")
        .rule("start", vec!["ERR"])
        .start("start")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"[^\s]+".to_string()));
}

#[test]
fn td_v10_mixed_fragile_and_normal() {
    let g = GrammarBuilder::new("td_v10_mix_frag")
        .token("OK", "ok")
        .fragile_token("ERR", r"[^\s]+")
        .rule("start", vec!["OK"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 2);
    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);
}

#[test]
fn td_v10_multiple_fragile_tokens() {
    let g = GrammarBuilder::new("td_v10_multi_frag")
        .fragile_token("ERR1", r"[^\s]+")
        .fragile_token("ERR2", r".+")
        .rule("start", vec!["ERR1"])
        .start("start")
        .build();
    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 2);
}

#[test]
fn td_v10_fragile_string_pattern() {
    let g = GrammarBuilder::new("td_v10_frag_str")
        .fragile_token("MISSING", "MISSING")
        .rule("start", vec!["MISSING"])
        .start("start")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert!(tok.fragile);
    assert!(matches!(tok.pattern, TokenPattern::String(_)));
}

#[test]
fn td_v10_normal_token_not_fragile() {
    let g = GrammarBuilder::new("td_v10_norm_nofrag")
        .token("NUM", r"\d+")
        .fragile_token("ERR", r".+")
        .rule("start", vec!["NUM"])
        .start("start")
        .build();
    let num = g.tokens.values().find(|t| t.name == "NUM").unwrap();
    assert!(!num.fragile);
}

#[test]
fn td_v10_fragile_clone_preserved() {
    let g = GrammarBuilder::new("td_v10_frag_clone")
        .fragile_token("ERR", r".+")
        .rule("start", vec!["ERR"])
        .start("start")
        .build();
    let g2 = g.clone();
    let tok = g2.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

#[test]
fn td_v10_fragile_after_normalize() {
    let mut g = GrammarBuilder::new("td_v10_frag_norm")
        .fragile_token("ERR", r".+")
        .token("OK", "ok")
        .rule("start", vec!["OK"])
        .start("start")
        .build();
    let _ = g.normalize();
    let err = g.tokens.values().find(|t| t.name == "ERR").unwrap();
    assert!(err.fragile);
}

#[test]
fn td_v10_fragile_after_optimize() {
    let mut g = GrammarBuilder::new("td_v10_frag_opt")
        .fragile_token("ERR", r".+")
        .token("OK", "ok")
        .rule("start", vec!["OK"])
        .start("start")
        .build();
    g.optimize();
    let err = g.tokens.values().find(|t| t.name == "ERR").unwrap();
    assert!(err.fragile);
}

// ── 51–60: Multiple tokens combined ─────────────────────────────────────────

#[test]
fn td_v10_two_tokens_both_present() {
    let g = GrammarBuilder::new("td_v10_two")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 2);
    let names: Vec<_> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"A"));
    assert!(names.contains(&"B"));
}

#[test]
fn td_v10_three_tokens_all_present() {
    let g = GrammarBuilder::new("td_v10_three")
        .token("X", "x")
        .token("Y", "y")
        .token("Z", "z")
        .rule("start", vec!["X"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn td_v10_tokens_with_rules() {
    let g = GrammarBuilder::new("td_v10_with_rules")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn td_v10_tokens_with_precedence() {
    let g = GrammarBuilder::new("td_v10_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUM"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn td_v10_tokens_with_extras() {
    let g = GrammarBuilder::new("td_v10_extras")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn td_v10_token_symbol_ids_distinct() {
    let g = GrammarBuilder::new("td_v10_ids_dist")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let ids: Vec<_> = g.tokens.keys().collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }
}

#[test]
fn td_v10_token_iter_count_matches_len() {
    let g = GrammarBuilder::new("td_v10_iter_len")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.iter().count(), g.tokens.len());
}

#[test]
fn td_v10_token_values_iter_count() {
    let g = GrammarBuilder::new("td_v10_vals_cnt")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.values().count(), 2);
}

#[test]
fn td_v10_token_keys_iter_count() {
    let g = GrammarBuilder::new("td_v10_keys_cnt")
        .token("P", r"\d+")
        .token("Q", r"\w+")
        .token("R", r"\s+")
        .rule("start", vec!["P"])
        .start("start")
        .build();
    assert_eq!(g.tokens.keys().count(), 3);
}

// ── 61–70: Token equality and pattern matching ──────────────────────────────

#[test]
fn td_v10_token_eq_same_fields() {
    let a = Token {
        name: "T".to_string(),
        pattern: TokenPattern::String("t".to_string()),
        fragile: false,
    };
    let b = Token {
        name: "T".to_string(),
        pattern: TokenPattern::String("t".to_string()),
        fragile: false,
    };
    assert_eq!(a, b);
}

#[test]
fn td_v10_token_ne_different_name() {
    let a = Token {
        name: "A".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    let b = Token {
        name: "B".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    assert_ne!(a, b);
}

#[test]
fn td_v10_token_ne_different_pattern() {
    let a = Token {
        name: "T".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    };
    let b = Token {
        name: "T".to_string(),
        pattern: TokenPattern::Regex("a".to_string()),
        fragile: false,
    };
    assert_ne!(a, b);
}

#[test]
fn td_v10_token_ne_different_fragile() {
    let a = Token {
        name: "T".to_string(),
        pattern: TokenPattern::String("t".to_string()),
        fragile: false,
    };
    let b = Token {
        name: "T".to_string(),
        pattern: TokenPattern::String("t".to_string()),
        fragile: true,
    };
    assert_ne!(a, b);
}

#[test]
fn td_v10_token_clone_eq() {
    let a = Token {
        name: "T".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: true,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn td_v10_token_pattern_string_eq() {
    let a = TokenPattern::String("hello".to_string());
    let b = TokenPattern::String("hello".to_string());
    assert_eq!(a, b);
}

#[test]
fn td_v10_token_pattern_regex_eq() {
    let a = TokenPattern::Regex(r"\d+".to_string());
    let b = TokenPattern::Regex(r"\d+".to_string());
    assert_eq!(a, b);
}

#[test]
fn td_v10_token_pattern_string_ne_regex() {
    let a = TokenPattern::String("abc".to_string());
    let b = TokenPattern::Regex("abc".to_string());
    assert_ne!(a, b);
}

#[test]
fn td_v10_token_debug_contains_name() {
    let tok = Token {
        name: "MY_TOK".to_string(),
        pattern: TokenPattern::String("my".to_string()),
        fragile: false,
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("MY_TOK"));
}

#[test]
fn td_v10_token_pattern_debug_variant() {
    let s = TokenPattern::String("x".to_string());
    let r = TokenPattern::Regex("x".to_string());
    let sd = format!("{s:?}");
    let rd = format!("{r:?}");
    assert!(sd.contains("String"));
    assert!(rd.contains("Regex"));
}

// ── 71–80: Edge cases and integration ───────────────────────────────────────

#[test]
fn td_v10_token_with_empty_name_builds() {
    let g = GrammarBuilder::new("td_v10_empty_name")
        .token("", "")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    // Builder should handle it — token count may be 1 or 2 depending on dedup
    assert!(!g.tokens.is_empty());
}

#[test]
fn td_v10_token_long_pattern() {
    let long_pat = r"[a-zA-Z_][a-zA-Z0-9_]*";
    let g = single_token_grammar("td_v10_long_pat", "IDENT", long_pat);
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(long_pat.to_string()));
}

#[test]
fn td_v10_token_single_char_string() {
    let g = single_token_grammar("td_v10_single_ch", ";", ";");
    let tok = g.tokens.values().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::String(_)));
}

#[test]
fn td_v10_tokens_survive_double_normalize() {
    let mut g = GrammarBuilder::new("td_v10_dbl_norm")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();
    let _ = g.normalize();
    let _ = g.normalize();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn td_v10_tokens_survive_optimize_then_normalize() {
    let mut g = GrammarBuilder::new("td_v10_opt_norm")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    g.optimize();
    let _ = g.normalize();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn td_v10_grammar_name_independent_of_tokens() {
    let g = GrammarBuilder::new("td_v10_gname")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.name, "td_v10_gname");
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn td_v10_same_token_re_added_no_duplicate() {
    let g = GrammarBuilder::new("td_v10_readd")
        .token("A", "a")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    // Re-adding same name overwrites — count stays 1
    let count = g.tokens.values().filter(|t| t.name == "A").count();
    assert_eq!(count, 1);
}

#[test]
fn td_v10_token_used_in_multiple_rules() {
    let g = GrammarBuilder::new("td_v10_multi_rule")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn td_v10_tokens_with_external() {
    let g = GrammarBuilder::new("td_v10_ext")
        .token("A", "a")
        .external("INDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 1);
    assert!(!g.externals.is_empty());
}

#[test]
fn td_v10_token_contains_key_true() {
    let g = single_token_grammar("td_v10_ckey", "K", "k");
    let sid = *g.tokens.keys().next().unwrap();
    assert!(g.tokens.contains_key(&sid));
}

// ── 81–85: Additional coverage ──────────────────────────────────────────────

#[test]
fn td_v10_all_tokens_have_nonempty_name() {
    let g = GrammarBuilder::new("td_v10_nonempty")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    for tok in g.tokens.values() {
        assert!(!tok.name.is_empty());
    }
}

#[test]
fn td_v10_token_map_get_returns_some() {
    let g = single_token_grammar("td_v10_get_some", "Z", "z");
    let sid = *g.tokens.keys().next().unwrap();
    assert!(g.tokens.get(&sid).is_some());
}

#[test]
fn td_v10_token_map_get_missing_returns_none() {
    let g = single_token_grammar("td_v10_get_none", "Z", "z");
    let missing = adze_ir::SymbolId(9999);
    assert!(g.tokens.get(&missing).is_none());
}

#[test]
fn td_v10_operator_tokens_patterns() {
    let g = GrammarBuilder::new("td_v10_ops")
        .token("==", "==")
        .token("!=", "!=")
        .token("<=", "<=")
        .token(">=", ">=")
        .rule("start", vec!["=="])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 4);
}

#[test]
fn td_v10_keyword_tokens() {
    let g = GrammarBuilder::new("td_v10_kw")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("WHILE", "while")
        .token("FOR", "for")
        .token("RETURN", "return")
        .rule("start", vec!["IF"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 5);
    for tok in g.tokens.values() {
        assert!(matches!(tok.pattern, TokenPattern::String(_)));
    }
}
