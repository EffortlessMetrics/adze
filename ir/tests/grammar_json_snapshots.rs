//! Insta snapshot tests for Grammar → JSON output.
//!
//! Run with: `INSTA_UPDATE=always cargo test -p adze-ir --test grammar_json_snapshots`

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

/// Serialize a grammar to pretty-printed JSON for snapshot comparison.
fn grammar_json(grammar: &Grammar) -> String {
    serde_json::to_string_pretty(grammar).expect("Grammar serialization should not fail")
}

// ---------------------------------------------------------------------------
// 1. Minimal grammar (1 rule, 1 terminal)
// ---------------------------------------------------------------------------

#[test]
fn minimal_grammar() {
    let grammar = GrammarBuilder::new("minimal")
        .token("NUMBER", r"\d+")
        .rule("value", vec!["NUMBER"])
        .start("value")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 2. Arithmetic grammar (add, multiply, number)
// ---------------------------------------------------------------------------

#[test]
fn arithmetic_grammar() {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUMBER"])
        .start("expr")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 3. Grammar with precedence declarations
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_precedence_declarations() {
    let grammar = GrammarBuilder::new("precedence_demo")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 4. Grammar with associativity (on rules)
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_associativity() {
    let grammar = GrammarBuilder::new("assoc_demo")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 5. Grammar with external tokens
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_external_tokens() {
    let grammar = GrammarBuilder::new("external_demo")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token(":", ":")
        .token("pass", "pass")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .rule("suite", vec!["NEWLINE", "INDENT", "statement", "DEDENT"])
        .rule("statement", vec!["pass"])
        .start("suite")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 6. Grammar with field mappings
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_field_mappings() {
    let mut grammar = GrammarBuilder::new("field_demo")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("binary_expr", vec!["NUMBER", "+", "NUMBER"])
        .start("binary_expr")
        .build();

    // Add field mappings manually (builder doesn't expose field API)
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "operator".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());

    // Attach fields to the rule
    if let Some(rules) = grammar.rules.values_mut().next()
        && let Some(rule) = rules.first_mut()
    {
        rule.fields = vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)];
    }

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 7. Grammar with extras (whitespace)
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_extras() {
    let grammar = GrammarBuilder::new("extras_demo")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token(";", ";")
        .token("WHITESPACE", r"[ \t\n\r]+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WHITESPACE")
        .extra("COMMENT")
        .rule("statement", vec!["IDENTIFIER", ";"])
        .start("statement")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 8. Grammar after normalization
// ---------------------------------------------------------------------------

#[test]
fn grammar_after_normalization() {
    let mut grammar = Grammar::new("normalize_demo".to_string());

    let expr_id = SymbolId(1);
    let num_id = SymbolId(2);
    let plus_id = SymbolId(3);

    grammar.tokens.insert(
        num_id,
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(expr_id, "expr".to_string());

    // expr -> NUMBER ('+' NUMBER)*
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Repeat(Box::new(Symbol::Sequence(vec![
                Symbol::Terminal(plus_id),
                Symbol::Terminal(num_id),
            ]))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.normalize();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 9. Grammar with all symbol types
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_all_symbol_types() {
    let mut grammar = Grammar::new("all_symbols".to_string());

    let start_id = SymbolId(1);
    let a_id = SymbolId(2);
    let b_id = SymbolId(3);
    let c_id = SymbolId(4);
    let ext_id = SymbolId(5);
    let inner_id = SymbolId(6);

    grammar.tokens.insert(
        a_id,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        b_id,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        c_id,
        Token {
            name: "C".to_string(),
            pattern: TokenPattern::String("c".to_string()),
            fragile: false,
        },
    );
    grammar.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: ext_id,
    });

    grammar.rule_names.insert(start_id, "start".to_string());
    grammar.rule_names.insert(inner_id, "inner".to_string());

    // Rule with Terminal, NonTerminal, External, Optional, Repeat, RepeatOne,
    // Choice, Sequence, Epsilon
    grammar.add_rule(Rule {
        lhs: start_id,
        rhs: vec![
            Symbol::Terminal(a_id),
            Symbol::NonTerminal(inner_id),
            Symbol::External(ext_id),
            Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
            Symbol::Repeat(Box::new(Symbol::Terminal(c_id))),
            Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id))),
            Symbol::Choice(vec![Symbol::Terminal(b_id), Symbol::Terminal(c_id)]),
            Symbol::Sequence(vec![Symbol::Terminal(a_id), Symbol::Terminal(b_id)]),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // inner -> ε
    grammar.add_rule(Rule {
        lhs: inner_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 10. Grammar with alias sequences
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_alias_sequences() {
    let mut grammar = GrammarBuilder::new("alias_demo")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token("=", "=")
        .token(";", ";")
        .rule("assignment", vec!["IDENTIFIER", "=", "IDENTIFIER", ";"])
        .start("assignment")
        .build();

    // Add alias sequences
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![
                Some("target".to_string()),
                None,
                Some("value".to_string()),
                None,
            ],
        },
    );
    grammar.max_alias_sequence_length = 4;

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 11. Grammar with supertypes
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_supertypes() {
    let mut grammar = GrammarBuilder::new("supertype_demo")
        .token("NUMBER", r"\d+")
        .token("STRING", r#""[^"]*""#)
        .token("true", "true")
        .token("false", "false")
        .rule("expression", vec!["literal"])
        .rule("literal", vec!["NUMBER"])
        .rule("literal", vec!["STRING"])
        .rule("literal", vec!["true"])
        .rule("literal", vec!["false"])
        .start("expression")
        .build();

    // Mark "literal" as a supertype
    let literal_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "literal")
        .map(|(id, _)| *id)
        .expect("literal should exist");
    grammar.supertypes.push(literal_id);

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 12. Grammar with inline rules
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_inline_rules() {
    let mut grammar = GrammarBuilder::new("inline_demo")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["_binary_op"])
        .rule("expr", vec!["NUMBER"])
        .rule("_binary_op", vec!["expr", "+", "expr"])
        .rule("_binary_op", vec!["expr", "*", "expr"])
        .start("expr")
        .build();

    // Mark _binary_op as inline
    let inline_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "_binary_op")
        .map(|(id, _)| *id)
        .expect("_binary_op should exist");
    grammar.inline_rules.push(inline_id);

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 13. Large grammar (20+ rules) → snapshot first 50 lines
// ---------------------------------------------------------------------------

#[test]
fn large_grammar_first_50_lines() {
    let grammar = GrammarBuilder::new("large_grammar")
        .token("NUMBER", r"\d+")
        .token("STRING", r#""[^"]*""#)
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("=", "=")
        .token("==", "==")
        .token("!=", "!=")
        .token("<", "<")
        .token(">", ">")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token(";", ";")
        .token(",", ",")
        .token("if", "if")
        .token("else", "else")
        .token("while", "while")
        .token("return", "return")
        .token("fn", "fn")
        .token("let", "let")
        // Rules (20+)
        .rule("program", vec!["declaration_list"])
        .rule("declaration_list", vec!["declaration"])
        .rule("declaration_list", vec!["declaration_list", "declaration"])
        .rule("declaration", vec!["function_decl"])
        .rule("declaration", vec!["var_decl"])
        .rule(
            "function_decl",
            vec!["fn", "IDENTIFIER", "(", "param_list", ")", "block"],
        )
        .rule("function_decl", vec!["fn", "IDENTIFIER", "(", ")", "block"])
        .rule("param_list", vec!["IDENTIFIER"])
        .rule("param_list", vec!["param_list", ",", "IDENTIFIER"])
        .rule("block", vec!["{", "statement_list", "}"])
        .rule("block", vec!["{", "}"])
        .rule("statement_list", vec!["statement"])
        .rule("statement_list", vec!["statement_list", "statement"])
        .rule("statement", vec!["expr_stmt"])
        .rule("statement", vec!["var_decl"])
        .rule("statement", vec!["return_stmt"])
        .rule("statement", vec!["if_stmt"])
        .rule("statement", vec!["while_stmt"])
        .rule("var_decl", vec!["let", "IDENTIFIER", "=", "expr", ";"])
        .rule("expr_stmt", vec!["expr", ";"])
        .rule("return_stmt", vec!["return", "expr", ";"])
        .rule("return_stmt", vec!["return", ";"])
        .rule("if_stmt", vec!["if", "(", "expr", ")", "block"])
        .rule(
            "if_stmt",
            vec!["if", "(", "expr", ")", "block", "else", "block"],
        )
        .rule("while_stmt", vec!["while", "(", "expr", ")", "block"])
        .rule("expr", vec!["IDENTIFIER"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["STRING"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .rule("expr", vec!["expr", "==", "expr"])
        .rule("expr", vec!["expr", "!=", "expr"])
        .rule("expr", vec!["expr", "<", "expr"])
        .rule("expr", vec!["expr", ">", "expr"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["call_expr"])
        .rule("call_expr", vec!["IDENTIFIER", "(", "arg_list", ")"])
        .rule("call_expr", vec!["IDENTIFIER", "(", ")"])
        .rule("arg_list", vec!["expr"])
        .rule("arg_list", vec!["arg_list", ",", "expr"])
        .start("program")
        .build();

    let json = grammar_json(&grammar);
    let first_50: String = json.lines().take(50).collect::<Vec<_>>().join("\n");
    insta::assert_snapshot!(first_50);
}

// ---------------------------------------------------------------------------
// 14. Empty grammar serializes cleanly
// ---------------------------------------------------------------------------

#[test]
fn empty_grammar_json() {
    let grammar = Grammar::new("empty".to_string());
    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 15. Grammar with conflicts (GLR declarations)
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_conflicts() {
    let mut grammar = GrammarBuilder::new("conflict_demo")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .token("*", "*")
        .rule("expr", vec!["IDENTIFIER"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["(", "type_expr", ")", "expr"])
        .rule("type_expr", vec!["IDENTIFIER"])
        .start("expr")
        .build();

    // Declare a GLR conflict between cast and multiplication
    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let type_expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, name)| name.as_str() == "type_expr")
        .map(|(id, _)| *id)
        .unwrap();

    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id, type_expr_id],
        resolution: ConflictResolution::GLR,
    });

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 16. Grammar with fragile tokens
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_fragile_tokens() {
    let grammar = GrammarBuilder::new("fragile_demo")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .fragile_token("KEYWORD_IF", "if")
        .fragile_token("KEYWORD_ELSE", "else")
        .token(";", ";")
        .rule("statement", vec!["KEYWORD_IF", "IDENTIFIER", ";"])
        .start("statement")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 17. Grammar with dynamic precedence
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_dynamic_precedence() {
    let mut grammar = Grammar::new("dynamic_prec".to_string());

    let expr_id = SymbolId(1);
    let num_id = SymbolId(2);
    let plus_id = SymbolId(3);

    grammar.tokens.insert(
        num_id,
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(expr_id, "expr".to_string());

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(PrecedenceKind::Dynamic(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 18. Grammar roundtrip (serialize then deserialize)
// ---------------------------------------------------------------------------

#[test]
fn grammar_json_roundtrip() {
    let grammar = GrammarBuilder::new("roundtrip")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();

    let json = grammar_json(&grammar);
    let deserialized: Grammar =
        serde_json::from_str(&json).expect("Deserialization should succeed");
    let json2 = grammar_json(&deserialized);

    // Snapshot the re-serialized output — must be identical
    insta::assert_snapshot!(json2);
}

// ---------------------------------------------------------------------------
// 19. Grammar with regex and string token patterns
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_mixed_token_patterns() {
    let grammar = GrammarBuilder::new("token_patterns")
        .token("NUMBER", r"\d+(\.\d+)?")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .token("STRING", r#""([^"\\]|\\.)*""#)
        .token("+", "+")
        .token(";", ";")
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["IDENTIFIER"])
        .rule("value", vec!["STRING"])
        .start("value")
        .build();

    insta::assert_snapshot!(grammar_json(&grammar));
}

// ---------------------------------------------------------------------------
// 20. Grammar with production IDs mapped
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_production_ids() {
    let mut grammar = GrammarBuilder::new("production_ids")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();

    grammar.production_ids.insert(RuleId(0), ProductionId(0));
    grammar.production_ids.insert(RuleId(1), ProductionId(1));
    grammar.production_ids.insert(RuleId(2), ProductionId(2));

    insta::assert_snapshot!(grammar_json(&grammar));
}
