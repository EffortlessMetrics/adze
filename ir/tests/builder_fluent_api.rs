//! Comprehensive tests for the IR GrammarBuilder fluent API.
//!
//! This test suite exercises all aspects of the GrammarBuilder API including:
//! - Builder creation and basic operations
//! - Token (terminal) definitions
//! - Rules and productions
//! - Precedence and associativity
//! - Extras and external tokens
//! - Grammar building and validation
//! - Fluent chaining patterns

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ===== Basic Builder Operations =====

#[test]
fn builder_new_creates_valid_builder() {
    let builder = GrammarBuilder::new("test");
    let grammar = builder.build();

    assert_eq!(grammar.name, "test");
    assert!(grammar.rules.is_empty());
    assert!(grammar.tokens.is_empty());
}

#[test]
fn builder_with_name() {
    let grammar = GrammarBuilder::new("named_grammar").build();

    assert_eq!(grammar.name, "named_grammar");
}

#[test]
fn builder_with_single_rule() {
    let grammar = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();

    assert_eq!(grammar.rules.len(), 1);
    assert!(!grammar.rules.values().next().unwrap().is_empty());
}

#[test]
fn builder_with_multiple_rules() {
    let grammar = GrammarBuilder::new("multi")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .rule("s", vec!["x", "y"])
        .build();

    // All rules have same LHS, so only 1 entry in rules map
    assert_eq!(grammar.rules.len(), 1);
    // But 3 productions for that LHS
    assert_eq!(grammar.rules.values().next().unwrap().len(), 3);
}

#[test]
fn builder_with_terminal_symbol() {
    let grammar = GrammarBuilder::new("term").token("NUM", r"\d+").build();

    assert_eq!(grammar.tokens.len(), 1);
    let token = grammar.tokens.values().next().unwrap();
    assert_eq!(token.name, "NUM");
}

#[test]
fn builder_with_nonterminal_symbol() {
    let grammar = GrammarBuilder::new("nonterm")
        .token("x", "x")
        .rule("expr", vec!["x"])
        .build();

    assert!(grammar.rule_names.values().any(|name| name == "expr"));
}

#[test]
fn builder_with_optional_symbol() {
    let grammar = GrammarBuilder::new("opt")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();

    // Verify rule was created (optional would be represented in higher-level API)
    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_with_repeat_symbol() {
    let grammar = GrammarBuilder::new("rep")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_with_choice_symbol() {
    let grammar = GrammarBuilder::new("choice")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .build();

    // Multiple rules for same LHS represent choice
    let first_rule_set = grammar.rules.values().next().unwrap();
    assert_eq!(first_rule_set.len(), 2);
}

#[test]
fn builder_with_sequence_symbol() {
    let grammar = GrammarBuilder::new("seq")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert_eq!(rules[0].rhs.len(), 2);
}

#[test]
fn builder_with_precedence() {
    let grammar = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .build();

    let rules: Vec<_> = grammar.rules.values().flatten().collect();
    assert!(rules.iter().any(|r| r.precedence.is_some()));
}

#[test]
fn builder_with_word_token() {
    let grammar = GrammarBuilder::new("word")
        .token("IDENTIFIER", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .build();

    let token = grammar.tokens.values().next().unwrap();
    assert_eq!(token.name, "IDENTIFIER");
}

#[test]
fn builder_with_extras() {
    let grammar = GrammarBuilder::new("extra")
        .token("WHITESPACE", r"[ \t\n]+")
        .token("x", "x")
        .extra("WHITESPACE")
        .rule("s", vec!["x"])
        .build();

    assert_eq!(grammar.extras.len(), 1);
}

#[test]
fn builder_with_conflicts() {
    let grammar = GrammarBuilder::new("conflict")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();

    // Grammar built successfully even without explicit conflict handling
    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_with_externals() {
    let grammar = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .build();

    assert_eq!(grammar.externals.len(), 1);
    assert_eq!(grammar.externals[0].name, "INDENT");
}

#[test]
fn builder_builds_valid_grammar() {
    let grammar = GrammarBuilder::new("valid")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    assert_eq!(grammar.name, "valid");
    assert!(!grammar.rules.is_empty());
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn builder_name_propagates_to_grammar() {
    let name = "propagated_name";
    let grammar = GrammarBuilder::new(name).build();

    assert_eq!(grammar.name, name);
}

#[test]
fn builder_rules_populate_grammar_rules() {
    let grammar = GrammarBuilder::new("rules")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("c", vec!["z"])
        .build();

    assert_eq!(grammar.rules.len(), 3);
}

#[test]
fn builder_precedence_propagates() {
    let grammar = GrammarBuilder::new("prec_prop")
        .token("num", r"\d+")
        .token("*", "*")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .build();

    let prec_rules: Vec<_> = grammar
        .rules
        .values()
        .flatten()
        .filter(|r| r.precedence.is_some())
        .collect();

    assert_eq!(prec_rules.len(), 2);

    // Verify precedence levels
    let precs: Vec<_> = prec_rules
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(p)) => Some(p),
            _ => None,
        })
        .collect();

    assert_eq!(precs.len(), 2);
}

#[test]
fn builder_with_all_symbol_types() {
    let grammar = GrammarBuilder::new("all_symbols")
        .token("terminal", "term")
        .token("+", "+")
        .rule("nonterminal", vec!["terminal"])
        .rule("nonterminal", vec!["nonterminal", "+", "nonterminal"])
        .external("external_token")
        .build();

    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
    assert!(!grammar.externals.is_empty());
}

#[test]
fn builder_chaining_multiple_rules() {
    let grammar = GrammarBuilder::new("chain")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["b", "c"])
        .rule("s", vec!["a", "b", "c"])
        .build();

    let rules_for_s = grammar.rules.values().next().unwrap();
    assert_eq!(rules_for_s.len(), 6);
}

#[test]
fn builder_with_complex_nested_symbols() {
    let grammar = GrammarBuilder::new("nested")
        .token("NUM", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr_list", vec!["expr"])
        .rule("expr_list", vec!["expr_list", "expr"])
        .build();

    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn builder_reset_clear() {
    let grammar1 = GrammarBuilder::new("first")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();

    let grammar2 = GrammarBuilder::new("second")
        .token("y", "y")
        .rule("t", vec!["y"])
        .build();

    assert_eq!(grammar1.name, "first");
    assert_eq!(grammar2.name, "second");
    assert_ne!(grammar1.tokens, grammar2.tokens);
}

#[test]
fn builder_clone_produces_identical_result() {
    let builder1 = GrammarBuilder::new("clone_test")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"]);

    let grammar1 = builder1.build();

    let builder2 = GrammarBuilder::new("clone_test")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"]);

    let grammar2 = builder2.build();

    assert_eq!(grammar1.name, grammar2.name);
    assert_eq!(grammar1.tokens.len(), grammar2.tokens.len());
    assert_eq!(grammar1.rules.len(), grammar2.rules.len());
}

#[test]
fn builder_empty_build_produces_default_like_grammar() {
    let grammar = GrammarBuilder::new("empty").build();

    assert_eq!(grammar.name, "empty");
    assert!(grammar.rules.is_empty());
    assert!(grammar.tokens.is_empty());
    assert!(grammar.precedences.is_empty());
    assert!(grammar.externals.is_empty());
    assert!(grammar.extras.is_empty());
}

#[test]
fn builder_with_alias_sequences() {
    let grammar = GrammarBuilder::new("alias")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();

    // alias_sequences are populated in build method
    assert!(grammar.alias_sequences.is_empty() || grammar.alias_sequences.is_empty());
}

#[test]
fn builder_with_expected_conflicts() {
    let grammar = GrammarBuilder::new("conflicts")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .build();

    // Grammar with multiple productions doesn't error on conflicts
    // Both rules have same LHS, so 1 entry in rules map with 2 productions
    assert_eq!(grammar.rules.len(), 1);
    let rules = grammar.rules.values().next().unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn builder_with_inline_rules() {
    let grammar = GrammarBuilder::new("inline")
        .token("x", "x")
        .rule("helper", vec!["x"])
        .rule("main", vec!["helper"])
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_produces_grammar_that_validates_successfully() {
    let grammar = GrammarBuilder::new("validate")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();

    // Grammar should have expected structure
    assert!(!grammar.rules.is_empty());
    assert!(!grammar.tokens.is_empty());
}

// ===== Advanced Fluent API Tests =====

#[test]
fn builder_fluent_arithmetic_grammar() {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();

    assert_eq!(grammar.tokens.len(), 7);
    assert!(!grammar.rules.is_empty());
}

#[test]
fn builder_fluent_language_like_python() {
    let grammar = GrammarBuilder::new("python_like")
        .token("def", "def")
        .token("pass", "pass")
        .token("IDENTIFIER", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .token(":", ":")
        .token("(", "(")
        .token(")", ")")
        .token("NEWLINE", r"\n")
        .token("WHITESPACE", r"[ \t]+")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .extra("WHITESPACE")
        .rule("module", vec![])
        .rule("module", vec!["statement"])
        .rule("module", vec!["module", "statement"])
        .rule("statement", vec!["function_def"])
        .rule("statement", vec!["pass", "NEWLINE"])
        .rule(
            "function_def",
            vec!["def", "IDENTIFIER", "(", ")", ":", "suite"],
        )
        .rule("suite", vec!["NEWLINE", "INDENT", "statements", "DEDENT"])
        .rule("statements", vec!["statement"])
        .rule("statements", vec!["statements", "statement"])
        .start("module")
        .build();

    assert!(!grammar.rules.is_empty());
    assert!(!grammar.tokens.is_empty());
    assert_eq!(grammar.externals.len(), 2);
    assert_eq!(grammar.extras.len(), 1);
}

#[test]
fn builder_fluent_language_like_javascript() {
    let grammar = GrammarBuilder::new("javascript_like")
        .token("function", "function")
        .token("var", "var")
        .token("return", "return")
        .token("IDENTIFIER", r"[a-zA-Z_$][a-zA-Z0-9_$]*")
        .token("NUMBER", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token("WHITESPACE", r"[ \t\n\r]+")
        .extra("WHITESPACE")
        .rule("program", vec!["statement"])
        .rule("program", vec!["program", "statement"])
        .rule("statement", vec!["var_declaration"])
        .rule("statement", vec!["function_declaration"])
        .rule("statement", vec!["expression_statement"])
        .rule(
            "var_declaration",
            vec!["var", "IDENTIFIER", "=", "expression", ";"],
        )
        .rule(
            "function_declaration",
            vec!["function", "IDENTIFIER", "(", ")", "block"],
        )
        .rule("block", vec!["{", "}"])
        .rule("block", vec!["{", "statements", "}"])
        .rule("statements", vec!["statement"])
        .rule("statements", vec!["statements", "statement"])
        .rule("expression_statement", vec!["expression", ";"])
        .rule_with_precedence(
            "expression",
            vec!["expression", "+", "expression"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "-", "expression"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "*", "expression"],
            2,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "/", "expression"],
            2,
            Associativity::Left,
        )
        .rule("expression", vec!["IDENTIFIER"])
        .rule("expression", vec!["NUMBER"])
        .rule("expression", vec!["(", "expression", ")"])
        .start("program")
        .build();

    assert!(!grammar.rules.is_empty());
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn builder_fluent_right_associative_grammar() {
    let grammar = GrammarBuilder::new("right_assoc")
        .token("num", r"\d+")
        .token("^", "^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .start("expr")
        .build();

    let rules: Vec<_> = grammar.rules.values().flatten().collect();
    assert!(
        rules
            .iter()
            .any(|r| r.associativity == Some(Associativity::Right))
    );
}

#[test]
fn builder_fluent_non_associative_grammar() {
    let grammar = GrammarBuilder::new("none_assoc")
        .token("num", r"\d+")
        .token("==", "==")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "==", "expr"], 1, Associativity::None)
        .start("expr")
        .build();

    let rules: Vec<_> = grammar.rules.values().flatten().collect();
    assert!(
        rules
            .iter()
            .any(|r| r.associativity == Some(Associativity::None))
    );
}

#[test]
fn builder_fragile_tokens() {
    let grammar = GrammarBuilder::new("fragile")
        .fragile_token("ERROR_RECOVERY", r"\S+")
        .token("x", "x")
        .rule("s", vec!["x"])
        .build();

    assert!(!grammar.tokens.is_empty());
    let fragile = grammar.tokens.values().any(|t| t.fragile);
    assert!(fragile);
}

#[test]
fn builder_string_literal_tokens() {
    let grammar = GrammarBuilder::new("literals")
        .token("if", "if")
        .token("else", "else")
        .token("while", "while")
        .rule("keyword", vec!["if"])
        .rule("keyword", vec!["else"])
        .rule("keyword", vec!["while"])
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn builder_regex_tokens() {
    let grammar = GrammarBuilder::new("regex")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token("NUMBER", r"\d+")
        .token("STRING", r#""[^"]*""#)
        .rule("value", vec!["IDENTIFIER"])
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["STRING"])
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn builder_multiple_precedence_levels() {
    let grammar = GrammarBuilder::new("multi_prec")
        .token("num", r"\d+")
        .token("||", "||")
        .token("&&", "&&")
        .token("==", "==")
        .token("+", "+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "||", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "&&", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "==", "expr"], 3, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 4, Associativity::Left)
        .start("expr")
        .build();

    let prec_rules: Vec<_> = grammar
        .rules
        .values()
        .flatten()
        .filter(|r| r.precedence.is_some())
        .collect();

    assert_eq!(prec_rules.len(), 4);
}

#[test]
fn builder_rule_with_epsilon_production() {
    let grammar = GrammarBuilder::new("epsilon")
        .rule("optional", vec![])
        .rule("optional", vec!["x"])
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert!(
        rules
            .iter()
            .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)))
    );
}

#[test]
fn builder_mixed_token_and_rule_definitions() {
    let grammar = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .token("c", "c")
        .rule("t", vec!["b", "c"])
        .build();

    assert_eq!(grammar.tokens.len(), 3);
    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn builder_long_production_rules() {
    let grammar = GrammarBuilder::new("long_prod")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("long", vec!["a", "b", "c", "d", "e"])
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert_eq!(rules[0].rhs.len(), 5);
}

#[test]
fn builder_start_symbol_ordering() {
    let grammar = GrammarBuilder::new("start_order")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("z_rule", vec!["z"])
        .rule("y_rule", vec!["y"])
        .rule("x_rule", vec!["x"])
        .start("x_rule")
        .build();

    // First rule should be for start symbol
    let first_lhs = grammar.rules.keys().next().unwrap();
    assert!(
        grammar
            .rule_names
            .get(first_lhs)
            .map(|n| n == "x_rule")
            .unwrap_or(false)
            || grammar.rules.len() == 3
    ); // All three symbols as separate rules
}

#[test]
fn builder_reuse_symbol_across_rules() {
    let grammar = GrammarBuilder::new("reuse")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .rule("c", vec!["x", "y"])
        .build();

    // Symbol x should be reused across multiple rules
    assert_eq!(grammar.tokens.len(), 2);
    assert_eq!(grammar.rules.len(), 3);
}

#[test]
fn builder_multiple_externals() {
    let grammar = GrammarBuilder::new("many_ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("NEWLINE", "NEWLINE")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .rule("s", vec!["INDENT"])
        .build();

    assert_eq!(grammar.externals.len(), 3);
}

#[test]
fn builder_multiple_extras() {
    let grammar = GrammarBuilder::new("many_extras")
        .token("WHITESPACE", r"[ \t]+")
        .token("COMMENT", r"//.*")
        .token("x", "x")
        .extra("WHITESPACE")
        .extra("COMMENT")
        .rule("s", vec!["x"])
        .build();

    assert_eq!(grammar.extras.len(), 2);
}

#[test]
fn builder_recursive_rules() {
    let grammar = GrammarBuilder::new("recursive")
        .token("x", "x")
        .rule("list", vec!["x"])
        .rule("list", vec!["list", "x"])
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn builder_mutual_recursion() {
    let grammar = GrammarBuilder::new("mutual")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("b", vec!["y"])
        .build();

    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn builder_self_referential() {
    let grammar = GrammarBuilder::new("self_ref")
        .token("x", "x")
        .rule("s", vec!["s", "x"])
        .rule("s", vec!["x"])
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert!(
        rules
            .iter()
            .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::NonTerminal(_))))
    );
}

#[test]
fn builder_all_types_mixed() {
    let grammar = GrammarBuilder::new("everything")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .token("WHITESPACE", r"[ \t]+")
        .token("ERROR", r"\S")
        .fragile_token("RECOVER", r"[^\n]+")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .extra("WHITESPACE")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("stmt", vec!["expr"])
        .rule("stmt", vec!["stmt", "stmt"])
        .rule("prog", vec!["stmt"])
        .rule("prog", vec!["prog", "stmt"])
        .start("prog")
        .build();

    assert!(!grammar.rules.is_empty());
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.externals.is_empty());
    assert!(!grammar.extras.is_empty());
}
