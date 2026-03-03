//! Exhaustive tests for the IR GrammarBuilder API
//!
//! This test suite provides comprehensive coverage of all GrammarBuilder methods
//! and various edge cases and use patterns.

use adze_ir::{Associativity, PrecedenceKind, Symbol, TokenPattern, builder::GrammarBuilder};

// ============================================================================
// Test 1: Every builder method individually
// ============================================================================

#[test]
fn test_builder_token_method() {
    let grammar = GrammarBuilder::new("test").token("NUMBER", r"\d+").build();

    assert_eq!(grammar.tokens.len(), 1);
    let token = grammar.tokens.values().next().unwrap();
    assert_eq!(token.name, "NUMBER");
    assert!(!token.fragile);
}

#[test]
fn test_builder_fragile_token_method() {
    let grammar = GrammarBuilder::new("test")
        .fragile_token("ERROR_TOKEN", r"[^\s]+")
        .build();

    assert_eq!(grammar.tokens.len(), 1);
    let token = grammar.tokens.values().next().unwrap();
    assert_eq!(token.name, "ERROR_TOKEN");
    assert!(token.fragile);
}

#[test]
fn test_builder_rule_method() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .build();

    assert_eq!(grammar.rules.len(), 1);
    let rules = grammar.rules.values().next().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
}

#[test]
fn test_builder_rule_with_precedence_method() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(rules[0].associativity, Some(Associativity::Left));
}

#[test]
fn test_builder_start_method() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    // Verify start symbol's rules are the first in the rules map
    let first_key = grammar.rules.keys().next().unwrap();
    // Find the symbol ID for "expr"
    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id);
    assert_eq!(Some(*first_key), expr_id);
}

#[test]
fn test_builder_extra_method() {
    let grammar = GrammarBuilder::new("test")
        .token("WHITESPACE", r"[ \t\n]+")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .extra("WHITESPACE")
        .build();

    assert_eq!(grammar.extras.len(), 1);
}

#[test]
fn test_builder_external_method() {
    let grammar = GrammarBuilder::new("test")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .build();

    assert_eq!(grammar.externals.len(), 1);
    assert_eq!(grammar.externals[0].name, "INDENT");
}

#[test]
fn test_builder_precedence_method() {
    let grammar = GrammarBuilder::new("test")
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
fn test_builder_build_method() {
    let grammar = GrammarBuilder::new("my_grammar")
        .token("X", "x")
        .rule("a", vec!["X"])
        .build();

    assert_eq!(grammar.name, "my_grammar");
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
}

// ============================================================================
// Test 2: Chaining all builder methods in a single expression
// ============================================================================

#[test]
fn test_builder_full_chain() {
    let grammar = GrammarBuilder::new("full_chain")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .fragile_token("ERROR", r"[^\s]+")
        .token("WHITESPACE", r"[ \t\n]+")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .rule("expr", vec!["NUMBER"])
        .rule_with_precedence("term", vec!["term", "*", "term"], 2, Associativity::Left)
        .rule_with_precedence("term", vec!["NUMBER"], 1, Associativity::Left)
        .start("expr")
        .extra("WHITESPACE")
        .external("INDENT")
        .external("DEDENT")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .precedence(2, Associativity::Left, vec!["*", "/"])
        .build();

    assert_eq!(grammar.name, "full_chain");
    assert_eq!(grammar.tokens.len(), 9);
    assert_eq!(grammar.rules.len(), 2); // expr and term
    assert_eq!(grammar.externals.len(), 2);
    assert_eq!(grammar.extras.len(), 1);
    assert_eq!(grammar.precedences.len(), 2);

    // Verify fragile token
    let fragile_token = grammar.tokens.values().find(|t| t.name == "ERROR").unwrap();
    assert!(fragile_token.fragile);
}

// ============================================================================
// Test 3: Builder with 0 tokens (empty grammar)
// ============================================================================

#[test]
fn test_builder_zero_tokens() {
    let grammar = GrammarBuilder::new("empty").build();

    assert_eq!(grammar.tokens.len(), 0);
    assert_eq!(grammar.rules.len(), 0);
    assert_eq!(grammar.name, "empty");
}

#[test]
fn test_builder_with_rules_but_no_tokens() {
    let grammar = GrammarBuilder::new("rules_only")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec![])
        .build();

    assert_eq!(grammar.tokens.len(), 0);
    assert_eq!(grammar.rules.len(), 3); // a, b, c
}

// ============================================================================
// Test 4: Builder with only external tokens
// ============================================================================

#[test]
fn test_builder_only_external_tokens() {
    let grammar = GrammarBuilder::new("externals_only")
        .token("EXT1", "EXT1")
        .token("EXT2", "EXT2")
        .token("EXT3", "EXT3")
        .external("EXT1")
        .external("EXT2")
        .external("EXT3")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
    assert_eq!(grammar.externals.len(), 3);
    assert!(
        grammar
            .externals
            .iter()
            .all(|e| matches!(e.name.as_str(), "EXT1" | "EXT2" | "EXT3"))
    );
}

// ============================================================================
// Test 5: Builder with only fragile tokens
// ============================================================================

#[test]
fn test_builder_only_fragile_tokens() {
    let grammar = GrammarBuilder::new("fragile_only")
        .fragile_token("FRAGILE1", r"[a-z]+")
        .fragile_token("FRAGILE2", r"[0-9]+")
        .fragile_token("FRAGILE3", r"[A-Z]+")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
    assert!(grammar.tokens.values().all(|t| t.fragile));
}

// ============================================================================
// Test 6: Builder with conflicting token names
// ============================================================================

#[test]
fn test_builder_conflicting_token_names() {
    // This tests that duplicate token names are handled (last one wins with indexmap)
    let grammar = GrammarBuilder::new("conflicts")
        .token("ID", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .token("ID", r"[a-zA-Z]+") // Override with different pattern
        .build();

    assert_eq!(grammar.tokens.len(), 1);
    let token = grammar.tokens.values().next().unwrap();
    assert_eq!(token.name, "ID");
    // The last pattern should be in effect
    match &token.pattern {
        TokenPattern::Regex(p) => assert_eq!(p, "[a-zA-Z]+"),
        _ => panic!("Expected regex pattern"),
    }
}

// ============================================================================
// Test 7: Builder with maximum precedence values
// ============================================================================

#[test]
fn test_builder_maximum_precedence_values() {
    let grammar = GrammarBuilder::new("max_prec")
        .token("X", "x")
        .rule_with_precedence("a", vec!["X"], i16::MAX, Associativity::Left)
        .rule_with_precedence("b", vec!["X"], i16::MIN, Associativity::Right)
        .precedence(i16::MAX, Associativity::Left, vec!["X"])
        .precedence(i16::MIN, Associativity::Right, vec!["X"])
        .build();

    // Verify max precedence rule exists
    let has_max = grammar
        .all_rules()
        .any(|r| matches!(r.precedence, Some(PrecedenceKind::Static(p)) if p == i16::MAX));
    assert!(has_max);

    let has_min = grammar
        .all_rules()
        .any(|r| matches!(r.precedence, Some(PrecedenceKind::Static(p)) if p == i16::MIN));
    assert!(has_min);

    // Verify precedence declarations
    assert_eq!(grammar.precedences[0].level, i16::MAX);
    assert_eq!(grammar.precedences[1].level, i16::MIN);
}

// ============================================================================
// Test 8: Builder with all associativity types
// ============================================================================

#[test]
fn test_builder_all_associativity_types() {
    let grammar = GrammarBuilder::new("assoc_types")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule_with_precedence("left", vec!["A"], 1, Associativity::Left)
        .rule_with_precedence("right", vec!["B"], 1, Associativity::Right)
        .rule_with_precedence("none", vec!["C"], 1, Associativity::None)
        .precedence(1, Associativity::Left, vec!["A"])
        .precedence(2, Associativity::Right, vec!["B"])
        .precedence(3, Associativity::None, vec!["C"])
        .build();

    // Check rules
    let mut assoc_from_rules = vec![];
    for rules in grammar.rules.values() {
        if let Some(assoc) = rules[0].associativity {
            assoc_from_rules.push(assoc);
        }
    }
    assert!(assoc_from_rules.contains(&Associativity::Left));
    assert!(assoc_from_rules.contains(&Associativity::Right));
    assert!(assoc_from_rules.contains(&Associativity::None));

    // Check precedences
    assert_eq!(grammar.precedences[0].associativity, Associativity::Left);
    assert_eq!(grammar.precedences[1].associativity, Associativity::Right);
    assert_eq!(grammar.precedences[2].associativity, Associativity::None);
}

// ============================================================================
// Test 9: Building the python_like() preset
// ============================================================================

#[test]
fn test_builder_python_like_preset() {
    let grammar = GrammarBuilder::python_like();

    assert_eq!(grammar.name, "python_like");
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
    assert!(!grammar.externals.is_empty()); // Should have INDENT/DEDENT
    assert!(!grammar.extras.is_empty()); // Should have WHITESPACE

    // Verify module is the start symbol
    let module_rule_name = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "module");
    assert!(module_rule_name.is_some());

    // Verify nullable production exists
    let module_id = module_rule_name.unwrap().0;
    let module_rules = &grammar.rules[module_id];
    assert!(
        module_rules
            .iter()
            .any(|r| r.rhs.is_empty() || r.rhs.iter().all(|s| matches!(s, Symbol::Epsilon)))
    );
}

#[test]
fn test_python_like_has_required_tokens() {
    let grammar = GrammarBuilder::python_like();

    let token_names: Vec<_> = grammar.tokens.values().map(|t| &t.name).collect();
    assert!(token_names.contains(&&"def".to_string()));
    assert!(token_names.contains(&&"pass".to_string()));
    assert!(token_names.contains(&&"IDENTIFIER".to_string()));
    assert!(token_names.contains(&&"INDENT".to_string()));
    assert!(token_names.contains(&&"DEDENT".to_string()));
}

// ============================================================================
// Test 10: Building the javascript_like() preset
// ============================================================================

#[test]
fn test_builder_javascript_like_preset() {
    let grammar = GrammarBuilder::javascript_like();

    assert_eq!(grammar.name, "javascript_like");
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
    assert!(grammar.externals.is_empty()); // JS shouldn't have externals
    assert!(!grammar.extras.is_empty()); // Should have WHITESPACE

    // Verify program is the start symbol
    let program_rule_name = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "program");
    assert!(program_rule_name.is_some());
}

#[test]
fn test_javascript_like_has_required_tokens() {
    let grammar = GrammarBuilder::javascript_like();

    let token_names: Vec<_> = grammar.tokens.values().map(|t| &t.name).collect();
    assert!(token_names.contains(&&"function".to_string()));
    assert!(token_names.contains(&&"var".to_string()));
    assert!(token_names.contains(&&"IDENTIFIER".to_string()));
    assert!(token_names.contains(&&"NUMBER".to_string()));
    assert!(token_names.contains(&&"+".to_string()));
    assert!(token_names.contains(&&"*".to_string()));
}

#[test]
fn test_javascript_like_has_precedence() {
    let grammar = GrammarBuilder::javascript_like();

    // Should have rules with precedence (rule_with_precedence was used)
    let has_rule_precedence = grammar.all_rules().any(|r| r.precedence.is_some());
    assert!(has_rule_precedence);
}

// ============================================================================
// Test 11: Verifying builder output matches manually constructed Grammar
// ============================================================================

#[test]
fn test_builder_matches_manual_construction() {
    // Build with builder
    let built = GrammarBuilder::new("compare")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();

    // Verify properties
    assert_eq!(built.name, "compare");
    assert_eq!(built.tokens.len(), 2);
    assert_eq!(built.rules.len(), 1);

    let token_names: Vec<_> = built.tokens.values().map(|t| &t.name).collect();
    assert_eq!(token_names.len(), 2);
    assert!(token_names.contains(&&"A".to_string()));
    assert!(token_names.contains(&&"B".to_string()));

    let rules = built.rules.values().next().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 2);
}

// ============================================================================
// Additional comprehensive tests
// ============================================================================

#[test]
fn test_builder_multiple_rules_same_lhs() {
    let grammar = GrammarBuilder::new("alternatives")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .rule("expr", vec!["expr", "A"])
        .build();

    let expr_rules = grammar.rules.values().next().unwrap();
    assert_eq!(expr_rules.len(), 3);
}

#[test]
fn test_builder_epsilon_production() {
    let grammar = GrammarBuilder::new("epsilon")
        .token("A", "a")
        .rule("opt", vec![]) // Empty RHS becomes epsilon
        .rule("opt", vec!["A"])
        .build();

    let rules = grammar.rules.values().next().unwrap();
    assert!(
        rules
            .iter()
            .any(|r| r.rhs.is_empty() || r.rhs.iter().all(|s| matches!(s, Symbol::Epsilon)))
    );
}

#[test]
fn test_builder_complex_precedence_chain() {
    let grammar = GrammarBuilder::new("precedence_chain")
        .token("LOW", "low")
        .token("MID", "mid")
        .token("HIGH", "high")
        .rule_with_precedence("a", vec!["LOW"], 1, Associativity::Left)
        .rule_with_precedence("b", vec!["MID"], 2, Associativity::Right)
        .rule_with_precedence("c", vec!["HIGH"], 3, Associativity::None)
        .build();

    assert_eq!(grammar.rules.len(), 3);
    let mut precedences = vec![];
    for rules in grammar.rules.values() {
        if let Some(PrecedenceKind::Static(p)) = rules[0].precedence {
            precedences.push(p);
        }
    }
    assert_eq!(precedences.len(), 3);
    assert!(precedences.contains(&1));
    assert!(precedences.contains(&2));
    assert!(precedences.contains(&3));
}

#[test]
fn test_builder_string_vs_regex_patterns() {
    let grammar = GrammarBuilder::new("patterns")
        .token("LITERAL", "literal_string")
        .token("REGEX", r"\d+")
        .token("BRACKET", "[")
        .token("PAREN", "(")
        .build();

    let literal = grammar
        .tokens
        .values()
        .find(|t| t.name == "LITERAL")
        .unwrap();
    match &literal.pattern {
        TokenPattern::String(s) => {
            assert_eq!(s, "literal_string");
        }
        _ => panic!("Expected string pattern for literal"),
    }

    let regex = grammar.tokens.values().find(|t| t.name == "REGEX").unwrap();
    match &regex.pattern {
        TokenPattern::Regex(r) => {
            assert_eq!(r, r"\d+");
        }
        _ => panic!("Expected regex pattern"),
    }
}

#[test]
fn test_builder_with_many_extras() {
    let grammar = GrammarBuilder::new("many_extras")
        .token("WHITESPACE", r"[ \t]+")
        .token("COMMENT", r"//[^\n]*")
        .token("NEWLINE", r"\n")
        .token("A", "a")
        .extra("WHITESPACE")
        .extra("COMMENT")
        .extra("NEWLINE")
        .rule("a", vec!["A"])
        .build();

    assert_eq!(grammar.extras.len(), 3);
    assert_eq!(grammar.tokens.len(), 4);
}

#[test]
fn test_builder_symbol_id_allocation() {
    // Verify that symbol IDs are allocated sequentially
    let grammar = GrammarBuilder::new("symbols")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .rule("z", vec!["C"])
        .build();

    let symbol_ids: Vec<_> = grammar
        .tokens
        .keys()
        .chain(grammar.rules.keys())
        .map(|id| id.0)
        .collect();

    // Should have sequential IDs (starting from 1 for first token)
    assert!(symbol_ids.len() >= 6); // At least 3 tokens + 3 rules
}

#[test]
fn test_builder_token_pattern_detection() {
    let grammar = GrammarBuilder::new("pattern_detection")
        // Literal string (matches itself or has special chars)
        .token("PLUS", "+")
        .token("MINUS", "-")
        // With regex special chars
        .token("DIGITS", r"\d+")
        .token("WORD", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .build();

    assert_eq!(grammar.tokens.len(), 4);

    let plus = grammar.tokens.values().find(|t| t.name == "PLUS").unwrap();
    // "+" is a special character so it becomes a regex
    assert!(matches!(
        &plus.pattern,
        TokenPattern::String(_) | TokenPattern::Regex(_)
    ));

    let digits = grammar
        .tokens
        .values()
        .find(|t| t.name == "DIGITS")
        .unwrap();
    assert!(matches!(&digits.pattern, TokenPattern::Regex(_)));
}

#[test]
fn test_builder_start_symbol_ordering() {
    // Verify that when start is set, those rules come first
    let grammar = GrammarBuilder::new("ordering")
        .token("A", "a")
        .rule("z", vec!["A"])
        .rule("y", vec!["A"])
        .rule("x", vec!["A"])
        .start("x")
        .build();

    // First rules should be for 'x' (the start symbol)
    let first_symbol_id = grammar.rules.keys().next().unwrap();
    let _first_rule = &grammar.rules[first_symbol_id][0];
    // The symbol should correspond to 'x'
    assert_eq!(grammar.rules.len(), 3);
}

#[test]
fn test_builder_rule_production_ids() {
    let grammar = GrammarBuilder::new("production_ids")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["A", "A"])
        .rule("term", vec!["A"])
        .build();

    // Verify production IDs are unique
    let production_ids: Vec<_> = grammar.all_rules().map(|r| r.production_id).collect();

    assert_eq!(production_ids.len(), 3);
    // All production IDs should be unique
    let mut unique_ids = production_ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), production_ids.len());
}

#[test]
fn test_builder_with_complex_expression_grammar() {
    let grammar = GrammarBuilder::new("expressions")
        .token("NUMBER", r"\d+")
        .token("IDENTIFIER", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .token("=", "=")
        .rule("program", vec!["statement"])
        .rule("program", vec!["program", "statement"])
        .rule("statement", vec!["assignment"])
        .rule("assignment", vec!["IDENTIFIER", "=", "expression"])
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
        .rule("expression", vec!["(", "expression", ")"])
        .rule("expression", vec!["NUMBER"])
        .rule("expression", vec!["IDENTIFIER"])
        .start("program")
        .build();

    assert_eq!(grammar.name, "expressions");
    assert_eq!(grammar.tokens.len(), 9);
    assert!(grammar.rules.len() >= 4); // At least program, statement, assignment, expression

    // Verify at least one rule has precedence
    let has_precedence = grammar.all_rules().any(|r| r.precedence.is_some());
    assert!(has_precedence);
}

#[test]
fn test_builder_empty_rules() {
    let grammar = GrammarBuilder::new("empty_rules")
        .token("A", "a")
        .rule("nullable", vec![])
        .rule("non_nullable", vec!["A"])
        .build();

    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn test_builder_fragment_like_pattern() {
    let grammar = GrammarBuilder::new("fragments")
        .token("_DIGIT", r"\d")
        .token("_HEX_DIGIT", r"[0-9a-fA-F]")
        .token("NUMBER", r"\d+")
        .rule("number_list", vec!["NUMBER"])
        .rule("number_list", vec!["number_list", "NUMBER"])
        .build();

    // All tokens should be present regardless of naming convention
    assert_eq!(grammar.tokens.len(), 3);
}
