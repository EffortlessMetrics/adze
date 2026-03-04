//! Comprehensive tests for advanced GrammarBuilder usage patterns.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol, SymbolId, TokenPattern};

// ============================================================================
// 1. Basic builder: token, rule, start, build
// ============================================================================

#[test]
fn basic_single_token_grammar() {
    let grammar = GrammarBuilder::new("single_tok")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    assert_eq!(grammar.name, "single_tok");
    assert_eq!(grammar.tokens.len(), 1);
    assert_eq!(grammar.rules.len(), 1);
}

#[test]
fn basic_builder_name_propagation() {
    let grammar = GrammarBuilder::new("my_lang").build();
    assert_eq!(grammar.name, "my_lang");
}

#[test]
fn basic_token_pattern_string_literal() {
    let grammar = GrammarBuilder::new("g").token("def", "def").build();

    let tok = grammar.tokens.values().next().unwrap();
    assert_eq!(tok.name, "def");
    assert!(matches!(tok.pattern, TokenPattern::String(ref s) if s == "def"));
    assert!(!tok.fragile);
}

#[test]
fn basic_token_pattern_regex() {
    let grammar = GrammarBuilder::new("g").token("NUMBER", r"\d+").build();

    let tok = grammar.tokens.values().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::Regex(ref r) if r == r"\d+"));
}

#[test]
fn basic_start_symbol_is_first_rule() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("beta", vec!["A"])
        .rule("alpha", vec!["beta"])
        .start("alpha")
        .build();

    // start symbol rules should come first in the ordered map
    let first_lhs = *grammar.rules.keys().next().unwrap();
    let first_name = grammar.rule_names.get(&first_lhs).unwrap();
    assert_eq!(first_name, "alpha");
}

#[test]
fn basic_fragile_token() {
    let grammar = GrammarBuilder::new("g").fragile_token("SEMI", ";").build();

    let tok = grammar.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

// ============================================================================
// 2. Multi-rule grammars
// ============================================================================

#[test]
fn multi_rule_same_lhs() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn multi_rule_different_lhs() {
    let grammar = GrammarBuilder::new("g")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("atom", vec!["NUM"])
        .rule("sum", vec!["atom", "+", "atom"])
        .start("sum")
        .build();

    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn multi_rule_all_rules_iterator() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("y", vec!["A", "B"])
        .start("x")
        .build();

    assert_eq!(grammar.all_rules().count(), 3);
}

#[test]
fn multi_rule_chain_of_nonterminals() {
    let grammar = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("a", vec!["X"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("c")
        .build();

    assert_eq!(grammar.rules.len(), 3);
    let c_id = grammar.find_symbol_by_name("c").unwrap();
    let c_rules = grammar.get_rules_for_symbol(c_id).unwrap();
    assert_eq!(c_rules.len(), 1);
    // rhs should be NonTerminal referencing b
    assert!(matches!(c_rules[0].rhs[0], Symbol::NonTerminal(_)));
}

// ============================================================================
// 3. Precedence and associativity
// ============================================================================

#[test]
fn precedence_static_values() {
    let grammar = GrammarBuilder::new("calc")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();

    let add = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .unwrap();
    assert_eq!(add.associativity, Some(Associativity::Left));

    let mul = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .unwrap();
    assert_eq!(mul.associativity, Some(Associativity::Left));
}

#[test]
fn precedence_right_associativity() {
    let grammar = GrammarBuilder::new("pow")
        .token("N", r"\d+")
        .token("^", r"\^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    let pow_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(pow_rule.associativity, Some(Associativity::Right));
}

#[test]
fn precedence_none_associativity() {
    let grammar = GrammarBuilder::new("cmp")
        .token("N", r"\d+")
        .token("<", r"\<")
        .rule_with_precedence("expr", vec!["expr", "<", "expr"], 0, Associativity::None)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    let cmp_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(cmp_rule.associativity, Some(Associativity::None));
}

#[test]
fn precedence_no_precedence_on_plain_rule() {
    let grammar = GrammarBuilder::new("g")
        .token("N", r"\d+")
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert!(rules[0].precedence.is_none());
    assert!(rules[0].associativity.is_none());
}

#[test]
fn precedence_declaration_via_builder() {
    let grammar = GrammarBuilder::new("g")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();

    assert_eq!(grammar.precedences.len(), 2);
    assert_eq!(grammar.precedences[0].level, 1);
    assert_eq!(grammar.precedences[1].level, 2);
}

#[test]
fn precedence_mixed_levels() {
    let grammar = GrammarBuilder::new("mixed")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", r"\^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    // 3 precedence rules + 1 base rule = 4
    assert_eq!(rules.len(), 4);
}

// ============================================================================
// 4. python_like() and javascript_like() presets
// ============================================================================

#[test]
fn python_like_name() {
    let grammar = GrammarBuilder::python_like();
    assert_eq!(grammar.name, "python_like");
}

#[test]
fn python_like_has_externals() {
    let grammar = GrammarBuilder::python_like();
    assert!(grammar.externals.len() >= 2);
    let external_names: Vec<&str> = grammar.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(external_names.contains(&"INDENT"));
    assert!(external_names.contains(&"DEDENT"));
}

#[test]
fn python_like_has_extras() {
    let grammar = GrammarBuilder::python_like();
    assert!(!grammar.extras.is_empty());
}

#[test]
fn python_like_nullable_start() {
    let grammar = GrammarBuilder::python_like();
    let module_id = grammar.find_symbol_by_name("module").unwrap();
    let module_rules = grammar.get_rules_for_symbol(module_id).unwrap();
    // There should be an epsilon production
    assert!(
        module_rules
            .iter()
            .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon))
    );
}

#[test]
fn python_like_has_function_def() {
    let grammar = GrammarBuilder::python_like();
    assert!(grammar.find_symbol_by_name("function_def").is_some());
}

#[test]
fn javascript_like_name() {
    let grammar = GrammarBuilder::javascript_like();
    assert_eq!(grammar.name, "javascript_like");
}

#[test]
fn javascript_like_non_nullable_start() {
    let grammar = GrammarBuilder::javascript_like();
    let program_id = grammar.find_symbol_by_name("program").unwrap();
    let program_rules = grammar.get_rules_for_symbol(program_id).unwrap();
    // No epsilon production
    assert!(
        !program_rules
            .iter()
            .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Epsilon))
    );
}

#[test]
fn javascript_like_has_precedence_rules() {
    let grammar = GrammarBuilder::javascript_like();
    let expr_id = grammar.find_symbol_by_name("expression").unwrap();
    let expr_rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    // At least some rules should have precedence
    let with_prec = expr_rules.iter().filter(|r| r.precedence.is_some()).count();
    assert!(
        with_prec >= 4,
        "expected >= 4 precedence rules, got {with_prec}"
    );
}

#[test]
fn javascript_like_has_block_rules() {
    let grammar = GrammarBuilder::javascript_like();
    let block_id = grammar.find_symbol_by_name("block").unwrap();
    let block_rules = grammar.get_rules_for_symbol(block_id).unwrap();
    assert!(block_rules.len() >= 2);
}

// ============================================================================
// 5. Grammar with externals
// ============================================================================

#[test]
fn externals_added_to_grammar() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    assert_eq!(grammar.externals.len(), 2);
    assert_eq!(grammar.externals[0].name, "INDENT");
    assert_eq!(grammar.externals[1].name, "DEDENT");
}

#[test]
fn externals_have_symbol_ids() {
    let grammar = GrammarBuilder::new("g")
        .external("INDENT")
        .external("DEDENT")
        .build();

    // Each external should have a distinct symbol id
    let id0 = grammar.externals[0].symbol_id;
    let id1 = grammar.externals[1].symbol_id;
    assert_ne!(id0, id1);
}

// ============================================================================
// 6. Grammar validation after build
// ============================================================================

#[test]
fn check_empty_terminals_ok() {
    let grammar = GrammarBuilder::new("g")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    assert!(grammar.check_empty_terminals().is_ok());
}

#[test]
fn all_rules_have_production_ids() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("y", vec!["A", "B"])
        .start("x")
        .build();

    let ids: Vec<_> = grammar.all_rules().map(|r| r.production_id).collect();
    // All production IDs should be unique
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids.len(), sorted.len());
}

// ============================================================================
// 7. Symbol lookup by name
// ============================================================================

#[test]
fn find_symbol_by_name_found() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    assert!(grammar.find_symbol_by_name("expr").is_some());
}

#[test]
fn find_symbol_by_name_not_found() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    assert!(grammar.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn uppercase_names_not_in_rule_names() {
    let grammar = GrammarBuilder::new("g")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    // "NUMBER" is all uppercase, so it should NOT be in rule_names
    assert!(grammar.find_symbol_by_name("NUMBER").is_none());
    // "expr" is lowercase, so it should be in rule_names
    assert!(grammar.find_symbol_by_name("expr").is_some());
}

#[test]
fn punctuation_tokens_not_in_rule_names() {
    let grammar = GrammarBuilder::new("g")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .build();

    assert!(grammar.find_symbol_by_name("+").is_none());
    assert!(grammar.find_symbol_by_name("(").is_none());
    assert!(grammar.find_symbol_by_name(")").is_none());
}

// ============================================================================
// 8. Rule counting and enumeration
// ============================================================================

#[test]
fn rule_count_matches_insertions() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("y", vec!["C"])
        .start("x")
        .build();

    // 2 distinct LHS symbols
    assert_eq!(grammar.rules.len(), 2);
    // 3 total rules
    assert_eq!(grammar.all_rules().count(), 3);
}

#[test]
fn token_count_matches_insertions() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn extras_count() {
    let grammar = GrammarBuilder::new("g")
        .token("WS", r"[ \t]+")
        .token("NL", r"\n")
        .extra("WS")
        .extra("NL")
        .build();

    assert_eq!(grammar.extras.len(), 2);
}

// ============================================================================
// 9. Edge cases
// ============================================================================

#[test]
fn empty_grammar_has_no_rules() {
    let grammar = GrammarBuilder::new("empty").build();
    assert!(grammar.rules.is_empty());
    assert!(grammar.tokens.is_empty());
    assert!(grammar.externals.is_empty());
    assert!(grammar.extras.is_empty());
}

#[test]
fn epsilon_rule_via_empty_rhs() {
    let grammar = GrammarBuilder::new("g")
        .rule("empty", vec![])
        .start("empty")
        .build();

    let id = grammar.find_symbol_by_name("empty").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn deeply_nested_nonterminal_chain() {
    let mut builder = GrammarBuilder::new("deep").token("X", "x");
    // Create a chain: level0 -> level1 -> ... -> level9 -> X
    builder = builder.rule("level0", vec!["X"]);
    for i in 1..10 {
        let prev = format!("level{}", i - 1);
        let curr = format!("level{}", i);
        // We need to use the builder pattern with owned strings
        // Since rule() takes &str, we use references to our owned strings
        builder = builder.rule(&curr, vec![&*prev.leak()]);
    }
    builder = builder.start("level9");
    let grammar = builder.build();

    assert_eq!(grammar.rules.len(), 10);
    assert!(grammar.find_symbol_by_name("level9").is_some());
    assert!(grammar.find_symbol_by_name("level0").is_some());
}

#[test]
fn symbol_id_starts_at_one() {
    // SymbolId(0) is reserved for EOF
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    for (&id, _) in &grammar.tokens {
        assert!(id.0 >= 1, "token symbol id should be >= 1, got {}", id.0);
    }
    for (&id, _) in &grammar.rules {
        assert!(id.0 >= 1, "rule symbol id should be >= 1, got {}", id.0);
    }
}

#[test]
fn reuse_same_symbol_across_rules() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("x", vec!["A"])
        .rule("y", vec!["A"])
        .start("x")
        .build();

    // Both rules reference the same terminal symbol for "A"
    let x_id = grammar.find_symbol_by_name("x").unwrap();
    let y_id = grammar.find_symbol_by_name("y").unwrap();
    let x_rules = grammar.get_rules_for_symbol(x_id).unwrap();
    let y_rules = grammar.get_rules_for_symbol(y_id).unwrap();

    let x_terminal = match &x_rules[0].rhs[0] {
        Symbol::Terminal(id) => *id,
        _ => panic!("expected terminal"),
    };
    let y_terminal = match &y_rules[0].rhs[0] {
        Symbol::Terminal(id) => *id,
        _ => panic!("expected terminal"),
    };
    assert_eq!(x_terminal, y_terminal);
}

#[test]
fn many_alternatives_same_lhs() {
    let mut builder = GrammarBuilder::new("g");
    for i in 0..20 {
        let tok_name: &str = Box::leak(format!("T{i}").into_boxed_str());
        builder = builder.token(tok_name, tok_name);
        builder = builder.rule("expr", vec![tok_name]);
    }
    builder = builder.start("expr");
    let grammar = builder.build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 20);
}

#[test]
fn start_symbol_method_returns_first_rule_lhs() {
    let grammar = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("second", vec!["A"])
        .rule("first", vec!["second"])
        .start("first")
        .build();

    // start() reorders rules so "first" comes first
    let first_key = *grammar.rules.keys().next().unwrap();
    let name = grammar.rule_names.get(&first_key).unwrap();
    assert_eq!(name, "first");
}

#[test]
fn get_rules_for_missing_symbol_returns_none() {
    let grammar = GrammarBuilder::new("g").build();
    assert!(grammar.get_rules_for_symbol(SymbolId(999)).is_none());
}

#[test]
fn build_registry_includes_tokens_and_rules() {
    let mut grammar = GrammarBuilder::new("g")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();

    let registry = grammar.get_or_build_registry();
    // Registry should have been built
    assert!(!registry.is_empty());
}
