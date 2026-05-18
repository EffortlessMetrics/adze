use super::*;
use adze_ir::{Symbol, TokenPattern};

#[test]
fn test_simple_conversion() {
    let mut grammar_js = GrammarJs::new("test".to_string());

    grammar_js.rules.insert(
        "expression".to_string(),
        JsRule::Choice {
            members: vec![
                JsRule::Symbol {
                    name: "number".to_string(),
                },
                JsRule::Symbol {
                    name: "identifier".to_string(),
                },
            ],
        },
    );

    grammar_js.rules.insert(
        "number".to_string(),
        JsRule::Pattern {
            value: r"\d+".to_string(),
        },
    );

    grammar_js.rules.insert(
        "identifier".to_string(),
        JsRule::Pattern {
            value: r"[a-zA-Z]+".to_string(),
        },
    );

    let converter = GrammarJsConverter::new(grammar_js);
    let grammar = converter.convert().unwrap();

    assert_eq!(grammar.name, "test");
    assert!(!grammar.rules.is_empty());
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn string_wrapper_symbols_lower_to_terminals_when_referenced() {
    let mut grammar_js = GrammarJs::new("string_wrapper".to_string());

    grammar_js.rules.insert(
        "source_file".to_string(),
        JsRule::Seq {
            members: vec![JsRule::Symbol {
                name: "keyword_if".to_string(),
            }],
        },
    );
    grammar_js.rules.insert(
        "keyword_if".to_string(),
        JsRule::String {
            value: "if".to_string(),
        },
    );

    let grammar = GrammarJsConverter::new(grammar_js).convert().unwrap();
    let source_file = grammar
        .rule_names
        .iter()
        .find_map(|(id, name)| (name == "source_file").then_some(*id))
        .expect("source_file symbol should exist");
    let keyword_if = grammar
        .rule_names
        .iter()
        .find_map(|(id, name)| (name == "keyword_if").then_some(*id))
        .expect("keyword_if symbol should exist");
    let if_token = grammar
        .tokens
        .iter()
        .find_map(|(id, token)| (token.name == "if").then_some(*id))
        .expect("literal token should exist");

    let source_rule = grammar
        .rules
        .get(&source_file)
        .and_then(|rules| rules.first())
        .expect("source_file rule should exist");

    assert_eq!(source_rule.rhs, vec![Symbol::Terminal(if_token)]);
    assert!(
        !source_rule.rhs.contains(&Symbol::NonTerminal(keyword_if)),
        "string leaf wrappers must not hide token lookahead behind nonterminals"
    );
}

#[test]
fn fielded_seq_preserves_fields_on_lowered_token_symbols() {
    let mut grammar_js = GrammarJs::new("fielded_seq".to_string());

    grammar_js.rules.insert(
        "source_file".to_string(),
        JsRule::Seq {
            members: vec![JsRule::Symbol {
                name: "pair".to_string(),
            }],
        },
    );
    grammar_js.rules.insert(
        "pair".to_string(),
        JsRule::Seq {
            members: vec![
                JsRule::Field {
                    name: "left".to_string(),
                    content: Box::new(JsRule::Symbol {
                        name: "pair_left".to_string(),
                    }),
                },
                JsRule::Field {
                    name: "right".to_string(),
                    content: Box::new(JsRule::Symbol {
                        name: "pair_right".to_string(),
                    }),
                },
            ],
        },
    );
    grammar_js.rules.insert(
        "pair_left".to_string(),
        JsRule::Pattern {
            value: r"\d+".to_string(),
        },
    );
    grammar_js.rules.insert(
        "pair_right".to_string(),
        JsRule::String {
            value: "+".to_string(),
        },
    );

    let grammar = GrammarJsConverter::new(grammar_js).convert().unwrap();
    let pair = grammar
        .rule_names
        .iter()
        .find_map(|(id, name)| (name == "pair").then_some(*id))
        .expect("pair symbol should exist");
    let pair_left = grammar
        .rule_names
        .iter()
        .find_map(|(id, name)| (name == "pair_left").then_some(*id))
        .expect("pair_left symbol should exist");
    let pair_right = grammar
        .rule_names
        .iter()
        .find_map(|(id, name)| (name == "pair_right").then_some(*id))
        .expect("pair_right symbol should exist");
    let left = grammar
        .fields
        .iter()
        .find_map(|(id, name)| (name == "left").then_some(*id))
        .expect("left field should exist");
    let right = grammar
        .fields
        .iter()
        .find_map(|(id, name)| (name == "right").then_some(*id))
        .expect("right field should exist");

    let pair_rule = grammar
        .rules
        .get(&pair)
        .and_then(|rules| rules.first())
        .expect("pair rule should exist");

    assert!(
        matches!(
            pair_rule.rhs.as_slice(),
            [Symbol::Terminal(_), Symbol::Terminal(_)]
        ),
        "token-backed field wrapper references should keep parser productions terminal-backed"
    );
    assert_eq!(pair_rule.fields, vec![(left, 0), (right, 1)]);

    let left_rule = grammar
        .rules
        .get(&pair_left)
        .and_then(|rules| rules.first())
        .expect("pair_left rule should exist");
    assert!(
        matches!(left_rule.rhs.as_slice(), [Symbol::Terminal(_)]),
        "field wrapper rules should still lower to terminal-backed productions"
    );
    let right_rule = grammar
        .rules
        .get(&pair_right)
        .and_then(|rules| rules.first())
        .expect("pair_right rule should exist");
    assert!(
        matches!(right_rule.rhs.as_slice(), [Symbol::Terminal(_)]),
        "field wrapper rules should still lower to terminal-backed productions"
    );
}

#[test]
fn fielded_seq_skips_generated_tuple_field_metadata() {
    let mut grammar_js = GrammarJs::new("generated_tuple_fields".to_string());

    grammar_js.rules.insert(
        "source_file".to_string(),
        JsRule::Seq {
            members: vec![JsRule::Symbol {
                name: "expr".to_string(),
            }],
        },
    );
    grammar_js.rules.insert(
        "expr".to_string(),
        JsRule::Seq {
            members: vec![
                JsRule::Field {
                    name: "Expr_Add_0".to_string(),
                    content: Box::new(JsRule::Pattern {
                        value: r"\d+".to_string(),
                    }),
                },
                JsRule::Field {
                    name: "Expr_Add_1".to_string(),
                    content: Box::new(JsRule::String {
                        value: "+".to_string(),
                    }),
                },
            ],
        },
    );

    let grammar = GrammarJsConverter::new(grammar_js).convert().unwrap();
    let expr = grammar
        .rule_names
        .iter()
        .find_map(|(id, name)| (name == "expr").then_some(*id))
        .expect("expr symbol should exist");
    let expr_rule = grammar
        .rules
        .get(&expr)
        .and_then(|rules| rules.first())
        .expect("expr rule should exist");

    assert!(
        expr_rule.fields.is_empty(),
        "generated tuple field names are extraction scaffolding and should not alter existing generated AST tree shape"
    );
}

#[test]
fn pattern_wrapper_tokens_keep_human_readable_hidden_names() {
    let mut grammar_js = GrammarJs::new("pattern_wrapper".to_string());

    grammar_js.rules.insert(
        "source_file".to_string(),
        JsRule::Seq {
            members: vec![JsRule::Symbol {
                name: "identifier".to_string(),
            }],
        },
    );
    grammar_js.rules.insert(
        "identifier".to_string(),
        JsRule::Pattern {
            value: r"[a-z]+".to_string(),
        },
    );

    let grammar = GrammarJsConverter::new(grammar_js).convert().unwrap();
    let token = grammar
        .tokens
        .values()
        .find(
            |token| matches!(&token.pattern, TokenPattern::Regex(pattern) if pattern == r"[a-z]+"),
        )
        .expect("wrapped pattern token should exist");

    assert_eq!(
        token.name, "_/[a-z]+/",
        "wrapped pattern tokens should remain hidden while preserving a diagnostic name"
    );
}
