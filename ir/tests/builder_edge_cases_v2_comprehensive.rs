//! Comprehensive edge-case tests for GrammarBuilder.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol, TokenPattern};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Look up a SymbolId by rule-name string.
fn find_rule_id(g: &adze_ir::Grammar, name: &str) -> adze_ir::SymbolId {
    g.rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("rule name `{name}` not found"))
}

/// Look up a SymbolId by token name.
fn find_token_id(g: &adze_ir::Grammar, name: &str) -> adze_ir::SymbolId {
    g.tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token `{name}` not found"))
}

// ═══════════════════════════════════════════════════════════════════════════
// 1.  Unicode token names and grammar names
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn unicode_grammar_name() {
    let g = GrammarBuilder::new("日本語文法")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "日本語文法");
}

#[test]
fn unicode_token_name_chinese() {
    let g = GrammarBuilder::new("t")
        .token("数字", "123")
        .rule("s", vec!["数字"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "数字");
    assert!(g.tokens.contains_key(&tid));
}

#[test]
fn unicode_token_name_emoji() {
    let g = GrammarBuilder::new("emoji")
        .token("🔥", "fire")
        .rule("s", vec!["🔥"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn unicode_rule_name_cyrillic() {
    let g = GrammarBuilder::new("кир")
        .token("a", "a")
        .rule("правило", vec!["a"])
        .start("правило")
        .build();
    let rid = find_rule_id(&g, "правило");
    assert_eq!(g.rules[&rid].len(), 1);
}

#[test]
fn unicode_mixed_script_grammar() {
    let g = GrammarBuilder::new("混合grammar")
        .token("αβγ", "abc")
        .token("数", "num")
        .rule("начало", vec!["αβγ", "数"])
        .start("начало")
        .build();
    assert_eq!(g.tokens.len(), 2);
    assert_eq!(g.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.  Very long rule names
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn long_rule_name_256_chars() {
    let long = "r".repeat(256);
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule(&long, vec!["a"])
        .start(&long)
        .build();
    let rid = find_rule_id(&g, &long);
    assert_eq!(g.rules[&rid].len(), 1);
}

#[test]
fn long_rule_name_1024_chars() {
    let long = "x".repeat(1024);
    let g = GrammarBuilder::new("t")
        .token("b", "b")
        .rule(&long, vec!["b"])
        .start(&long)
        .build();
    assert_eq!(g.name, "t");
    let rid = find_rule_id(&g, &long);
    assert!(!g.rules[&rid].is_empty());
}

#[test]
fn long_grammar_name() {
    let long = "g".repeat(512);
    let g = GrammarBuilder::new(&long)
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name.len(), 512);
}

#[test]
fn long_token_name() {
    let long = "tok".repeat(200);
    let g = GrammarBuilder::new("t")
        .token(&long, "pattern")
        .rule("s", vec![&*long])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.  Many rules for the same LHS (10+)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ten_alternatives_same_lhs() {
    let mut b = GrammarBuilder::new("t");
    for i in 0..10 {
        let tok_name = format!("t{i}");
        b = b.token(&tok_name, &tok_name);
    }
    for i in 0..10 {
        let tok_name = format!("t{i}");
        b = b.rule("s", vec![&*tok_name]);
    }
    let g = b.start("s").build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid].len(), 10);
}

#[test]
fn fifteen_alternatives_same_lhs() {
    let mut b = GrammarBuilder::new("t");
    let names: Vec<String> = (0..15).map(|i| format!("tok{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    for n in &names {
        b = b.rule("expr", vec![n]);
    }
    let g = b.start("expr").build();
    let rid = find_rule_id(&g, "expr");
    assert_eq!(g.rules[&rid].len(), 15);
}

#[test]
fn twenty_alternatives_all_have_unique_production_ids() {
    let mut b = GrammarBuilder::new("t");
    let names: Vec<String> = (0..20).map(|i| format!("a{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    for n in &names {
        b = b.rule("root", vec![n]);
    }
    let g = b.start("root").build();
    let rid = find_rule_id(&g, "root");
    let ids: Vec<_> = g.rules[&rid].iter().map(|r| r.production_id).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 20);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.  Deeply nested nonterminal chains (10+ levels)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn chain_depth_10() {
    let mut b = GrammarBuilder::new("chain").token("leaf", "leaf");
    for i in 0..10 {
        let lhs = format!("n{i}");
        let rhs = if i == 0 {
            "leaf".to_string()
        } else {
            format!("n{}", i - 1)
        };
        b = b.rule(&lhs, vec![&*rhs]);
    }
    let g = b.start("n9").build();
    assert_eq!(g.rules.len(), 10);
}

#[test]
fn chain_depth_15() {
    let mut b = GrammarBuilder::new("deep").token("x", "x");
    for i in 0..15 {
        let lhs = format!("level{i}");
        let rhs = if i == 0 {
            "x".to_string()
        } else {
            format!("level{}", i - 1)
        };
        b = b.rule(&lhs, vec![&*rhs]);
    }
    let g = b.start("level14").build();
    assert_eq!(g.rules.len(), 15);
}

#[test]
fn chain_depth_20_start_is_first_rule() {
    let mut b = GrammarBuilder::new("deep20").token("z", "z");
    for i in 0..20 {
        let lhs = format!("d{i}");
        let rhs = if i == 0 {
            "z".to_string()
        } else {
            format!("d{}", i - 1)
        };
        b = b.rule(&lhs, vec![&*rhs]);
    }
    let g = b.start("d19").build();
    // The start symbol's rules should be first in the ordered map.
    let first_lhs = *g.rules.keys().next().unwrap();
    let start_id = find_rule_id(&g, "d19");
    assert_eq!(first_lhs, start_id);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.  Rules with many symbols in RHS (10+)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rhs_10_symbols() {
    let mut b = GrammarBuilder::new("t");
    let names: Vec<String> = (0..10).map(|i| format!("t{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let g = b.rule("s", refs).start("s").build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid][0].rhs.len(), 10);
}

#[test]
fn rhs_20_symbols() {
    let mut b = GrammarBuilder::new("t");
    let names: Vec<String> = (0..20).map(|i| format!("w{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let g = b.rule("s", refs).start("s").build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid][0].rhs.len(), 20);
}

#[test]
fn rhs_mixed_terminals_nonterminals() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("inner", vec!["a"])
        .rule(
            "s",
            vec![
                "a", "inner", "b", "inner", "c", "inner", "a", "b", "c", "inner",
            ],
        )
        .start("s")
        .build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid][0].rhs.len(), 10);
    // Check mix of Terminal and NonTerminal
    let terminals = g.rules[&rid][0]
        .rhs
        .iter()
        .filter(|s| matches!(s, Symbol::Terminal(_)))
        .count();
    let nonterminals = g.rules[&rid][0]
        .rhs
        .iter()
        .filter(|s| matches!(s, Symbol::NonTerminal(_)))
        .count();
    assert!(terminals > 0);
    assert!(nonterminals > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.  Multiple start symbols (last wins)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn start_called_twice_last_wins() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("first", vec!["a"])
        .rule("second", vec!["b"])
        .start("first")
        .start("second")
        .build();
    // The last start() should be first in rules map.
    let first_lhs = *g.rules.keys().next().unwrap();
    let second_id = find_rule_id(&g, "second");
    assert_eq!(first_lhs, second_id);
}

#[test]
fn start_called_three_times() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .start("r1")
        .start("r2")
        .start("r3")
        .build();
    let first_lhs = *g.rules.keys().next().unwrap();
    let r3_id = find_rule_id(&g, "r3");
    assert_eq!(first_lhs, r3_id);
}

#[test]
fn start_same_symbol_twice_is_idempotent() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .start("s")
        .build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid].len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7.  Duplicate token names
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn duplicate_token_name_last_pattern_wins() {
    let g = GrammarBuilder::new("t")
        .token("NUM", "first_pattern")
        .token("NUM", "second_pattern")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
    let tid = find_token_id(&g, "NUM");
    match &g.tokens[&tid].pattern {
        TokenPattern::String(s) => assert_eq!(s, "second_pattern"),
        TokenPattern::Regex(s) => assert_eq!(s, "second_pattern"),
    }
}

#[test]
fn duplicate_token_reuses_symbol_id() {
    let g = GrammarBuilder::new("t")
        .token("X", "x1")
        .token("X", "x2")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    // Only one symbol ID for "X"
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn overwrite_normal_token_with_fragile() {
    let g = GrammarBuilder::new("t")
        .token("WS", "ws_pattern")
        .fragile_token("WS", "ws_pattern")
        .rule("s", vec!["WS"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "WS");
    assert!(g.tokens[&tid].fragile);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.  Token patterns with regex special chars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_pattern_with_backslash_d() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "NUM");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_pattern_with_brackets() {
    let g = GrammarBuilder::new("t")
        .token("ALPHA", "[a-zA-Z]+")
        .rule("s", vec!["ALPHA"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "ALPHA");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_pattern_with_pipe_alternation() {
    let g = GrammarBuilder::new("t")
        .token("KW", "if|else|while")
        .rule("s", vec!["KW"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "KW");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_pattern_with_dot_star() {
    let g = GrammarBuilder::new("t")
        .token("ANY", ".*")
        .rule("s", vec!["ANY"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "ANY");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_pattern_with_caret_dollar() {
    let g = GrammarBuilder::new("t")
        .token("LINE", "^hello$")
        .rule("s", vec!["LINE"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "LINE");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_pattern_with_question_mark() {
    let g = GrammarBuilder::new("t")
        .token("OPT", "colou?r")
        .rule("s", vec!["OPT"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "OPT");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_pattern_plain_string() {
    let g = GrammarBuilder::new("t")
        .token("hello", "hello")
        .rule("s", vec!["hello"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "hello");
    assert!(matches!(&g.tokens[&tid].pattern, TokenPattern::String(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 9.  Builder method chaining order variations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn start_before_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .start("s")
        .rule("s", vec!["a"])
        .build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid].len(), 1);
}

#[test]
fn rules_before_tokens_nonterminal_fallback() {
    // When rule is added before the token, symbol is treated as NonTerminal
    let g = GrammarBuilder::new("t")
        .rule("s", vec!["a"])
        .token("a", "a")
        .start("s")
        .build();
    let rid = find_rule_id(&g, "s");
    // "a" was not a token when the rule was added, so it's NonTerminal
    assert!(matches!(g.rules[&rid][0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn extra_before_token_definition() {
    let g = GrammarBuilder::new("t")
        .extra("WS")
        .token("WS", r"[ \t]+")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn external_before_token() {
    let g = GrammarBuilder::new("t")
        .external("INDENT")
        .token("INDENT", "INDENT")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn precedence_declaration_before_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .precedence(1, Associativity::Left, vec!["a"])
        .precedence(2, Associativity::Right, vec!["b"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[1].level, 2);
}

#[test]
fn interleaved_tokens_and_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a", "inner"])
        .token("b", "b")
        .rule("inner", vec!["b"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 2);
    assert_eq!(g.rules.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10.  Large grammars (50+ tokens, 50+ rules)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fifty_tokens() {
    let mut b = GrammarBuilder::new("large");
    let names: Vec<String> = (0..50).map(|i| format!("tok{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    b = b.rule("s", vec![&*names[0]]).start("s");
    let g = b.build();
    assert_eq!(g.tokens.len(), 50);
}

#[test]
fn fifty_rules_distinct_lhs() {
    let mut b = GrammarBuilder::new("large");
    b = b.token("leaf", "leaf");
    let rule_names: Vec<String> = (0..50).map(|i| format!("rule{i}")).collect();
    for (i, n) in rule_names.iter().enumerate() {
        let rhs = if i == 0 {
            "leaf".to_string()
        } else {
            rule_names[i - 1].clone()
        };
        b = b.rule(n, vec![&*rhs]);
    }
    let g = b.start("rule49").build();
    assert_eq!(g.rules.len(), 50);
}

#[test]
fn hundred_tokens_and_rules() {
    let mut b = GrammarBuilder::new("huge");
    let tok_names: Vec<String> = (0..100).map(|i| format!("t{i}")).collect();
    for n in &tok_names {
        b = b.token(n, n);
    }
    // 100 rules, each using a different token
    for (i, n) in tok_names.iter().enumerate() {
        let rule_name = format!("r{i}");
        b = b.rule(&rule_name, vec![n]);
    }
    let g = b.start("r0").build();
    assert_eq!(g.tokens.len(), 100);
    assert_eq!(g.rules.len(), 100);
}

#[test]
fn large_grammar_start_symbol_ordering() {
    let mut b = GrammarBuilder::new("big");
    b = b.token("leaf", "leaf");
    let names: Vec<String> = (0..60).map(|i| format!("nt{i}")).collect();
    for n in &names {
        b = b.rule(n, vec!["leaf"]);
    }
    // Start should be the last defined nonterminal.
    let g = b.start("nt59").build();
    let first_lhs = *g.rules.keys().next().unwrap();
    let start_id = find_rule_id(&g, "nt59");
    assert_eq!(first_lhs, start_id);
}

#[test]
fn large_grammar_production_id_uniqueness() {
    let mut b = GrammarBuilder::new("unique");
    b = b.token("a", "a");
    for i in 0..50 {
        let name = format!("r{i}");
        b = b.rule(&name, vec!["a"]);
    }
    let g = b.start("r0").build();
    let mut all_pids = Vec::new();
    for rules in g.rules.values() {
        for rule in rules {
            all_pids.push(rule.production_id);
        }
    }
    let set: std::collections::HashSet<_> = all_pids.iter().collect();
    assert_eq!(set.len(), all_pids.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 11.  Empty / minimal grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_no_tokens_no_rules() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.tokens.len(), 0);
    assert_eq!(g.rules.len(), 0);
}

#[test]
fn grammar_with_only_tokens() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .build();
    assert_eq!(g.tokens.len(), 2);
    assert_eq!(g.rules.len(), 0);
}

#[test]
fn grammar_with_epsilon_rule() {
    let g = GrammarBuilder::new("t")
        .rule("empty_rule", vec![])
        .start("empty_rule")
        .build();
    let rid = find_rule_id(&g, "empty_rule");
    assert_eq!(g.rules[&rid][0].rhs.len(), 1);
    assert!(matches!(g.rules[&rid][0].rhs[0], Symbol::Epsilon));
}

// ═══════════════════════════════════════════════════════════════════════════
// 12.  Extras and externals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_extras() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"[ \t]+")
        .token("NL", r"\n")
        .token("a", "a")
        .extra("WS")
        .extra("NL")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn multiple_externals() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.externals.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 13.  Precedence and associativity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rule_with_precedence_left() {
    let g = GrammarBuilder::new("t")
        .token("n", "n")
        .token("op", "op")
        .rule_with_precedence("e", vec!["e", "op", "e"], 5, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let rid = find_rule_id(&g, "e");
    let prec_rule = &g.rules[&rid][0];
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn rule_with_precedence_right() {
    let g = GrammarBuilder::new("t")
        .token("n", "n")
        .token("op", "op")
        .rule_with_precedence("e", vec!["e", "op", "e"], 3, Associativity::Right)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let rid = find_rule_id(&g, "e");
    let prec_rule = &g.rules[&rid][0];
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn negative_precedence_level() {
    let g = GrammarBuilder::new("t")
        .token("n", "n")
        .token("op", "op")
        .rule_with_precedence("e", vec!["e", "op", "e"], -10, Associativity::Left)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let rid = find_rule_id(&g, "e");
    assert_eq!(
        g.rules[&rid][0].precedence,
        Some(PrecedenceKind::Static(-10))
    );
}

#[test]
fn multiple_precedence_levels() {
    let g = GrammarBuilder::new("t")
        .token("n", "n")
        .token("plus", "plus")
        .token("star", "star")
        .token("exp", "exp")
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "exp", "e"], 3, Associativity::Right)
        .rule("e", vec!["n"])
        .start("e")
        .build();
    let rid = find_rule_id(&g, "e");
    assert_eq!(g.rules[&rid].len(), 4);
}

// ═══════════════════════════════════════════════════════════════════════════
// 14.  Fragile tokens
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fragile_token_flag() {
    let g = GrammarBuilder::new("t")
        .fragile_token("ERR", "error_pattern")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "ERR");
    assert!(g.tokens[&tid].fragile);
}

#[test]
fn non_fragile_token_flag() {
    let g = GrammarBuilder::new("t")
        .token("OK", "ok_pattern")
        .rule("s", vec!["OK"])
        .start("s")
        .build();
    let tid = find_token_id(&g, "OK");
    assert!(!g.tokens[&tid].fragile);
}

// ═══════════════════════════════════════════════════════════════════════════
// 15.  Symbol identity and reuse
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn same_symbol_referenced_in_multiple_rules() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["a"])
        .rule("r3", vec!["a", "a"])
        .start("r1")
        .build();
    // "a" should map to the same SymbolId everywhere
    let r1_id = find_rule_id(&g, "r1");
    let r2_id = find_rule_id(&g, "r2");
    let r3_id = find_rule_id(&g, "r3");
    let sym1 = &g.rules[&r1_id][0].rhs[0];
    let sym2 = &g.rules[&r2_id][0].rhs[0];
    let sym3a = &g.rules[&r3_id][0].rhs[0];
    let sym3b = &g.rules[&r3_id][0].rhs[1];
    assert_eq!(sym1, sym2);
    assert_eq!(sym1, sym3a);
    assert_eq!(sym1, sym3b);
}

#[test]
fn nonterminal_reused_in_rhs_and_lhs() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner", "inner"])
        .start("outer")
        .build();
    let oid = find_rule_id(&g, "outer");
    let rhs = &g.rules[&oid][0].rhs;
    assert_eq!(rhs[0], rhs[1]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 16.  No start symbol specified
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn no_start_symbol() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    // Grammar should build fine; rules are in insertion order
    assert_eq!(g.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 17.  Self-referencing / recursive rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn direct_left_recursion() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid].len(), 2);
}

#[test]
fn indirect_recursion_a_b_a() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("a", vec!["b", "x"])
        .rule("b", vec!["a", "x"])
        .rule("a", vec!["x"])
        .start("a")
        .build();
    assert_eq!(g.rules.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 18.  Special-character token names (punctuation tokens)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn punctuation_token_names() {
    let g = GrammarBuilder::new("t")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token("[", "[")
        .token("]", "]")
        .token(";", ";")
        .token(":", ":")
        .token(",", ",")
        .token("a", "a")
        .rule("s", vec!["(", "a", ")"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 10);
}

// ═══════════════════════════════════════════════════════════════════════════
// 19.  Grammar name edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_name() {
    let g = GrammarBuilder::new("")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "");
}

#[test]
fn grammar_name_with_spaces() {
    let g = GrammarBuilder::new("my grammar name")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "my grammar name");
}

// ═══════════════════════════════════════════════════════════════════════════
// 20.  Stress: large RHS + many alternatives combined
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn stress_many_alts_with_long_rhs() {
    let mut b = GrammarBuilder::new("stress");
    let tok_names: Vec<String> = (0..12).map(|i| format!("tok{i}")).collect();
    for n in &tok_names {
        b = b.token(n, n);
    }
    // 10 alternatives for "s", each with 12-token RHS
    for i in 0..10 {
        // Rotate the token list for variety
        let mut rhs: Vec<&str> = tok_names.iter().map(|s| s.as_str()).collect();
        rhs.rotate_left(i % tok_names.len());
        b = b.rule("s", rhs);
    }
    let g = b.start("s").build();
    let rid = find_rule_id(&g, "s");
    assert_eq!(g.rules[&rid].len(), 10);
    for rule in &g.rules[&rid] {
        assert_eq!(rule.rhs.len(), 12);
    }
}

#[test]
fn stress_wide_and_deep() {
    let mut b = GrammarBuilder::new("wd");
    // 30 tokens
    let toks: Vec<String> = (0..30).map(|i| format!("t{i}")).collect();
    for t in &toks {
        b = b.token(t, t);
    }
    // 20-deep nonterminal chain
    for i in 0..20 {
        let lhs = format!("n{i}");
        let rhs = if i == 0 {
            "t0".to_string()
        } else {
            format!("n{}", i - 1)
        };
        b = b.rule(&lhs, vec![&*rhs]);
    }
    // 15 alternatives for top-level
    for i in 0..15 {
        let tok = format!("t{i}");
        b = b.rule("top", vec![&*tok]);
    }
    let g = b.start("top").build();
    assert_eq!(g.tokens.len(), 30);
    assert_eq!(g.rules.len(), 21); // 20 chain + 1 top
}
