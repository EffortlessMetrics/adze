use adze::parser_v4::ParseNode;
use adze::query::{PredicateContext, QueryError, compile_query};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use std::collections::HashMap;

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    let identifier = SymbolId(1);
    let expression = SymbolId(10);

    grammar.tokens.insert(
        identifier,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex("[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            fragile: false,
        },
    );

    grammar.rules.entry(expression).or_default().push(Rule {
        lhs: expression,
        rhs: vec![Symbol::Terminal(identifier)],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
    });

    grammar
        .rule_names
        .insert(expression, "expression".to_string());

    grammar
}

fn make_node(symbol: SymbolId, start: usize, end: usize) -> ParseNode {
    ParseNode {
        symbol,
        symbol_id: symbol,
        start_byte: start,
        end_byte: end,
        field_name: None,
        children: vec![],
    }
}

#[test]
fn test_eq_predicate_matches_literal_value() {
    let grammar = create_test_grammar();
    let query = compile_query("(expression @expr) (#eq? @expr \"hello\")", &grammar)
        .expect("eq predicate query should compile");

    assert_eq!(query.patterns.len(), 1);
    assert_eq!(query.patterns[0].predicates.len(), 1);

    let mut captures = HashMap::new();
    captures.insert(0, make_node(SymbolId(10), 0, 5));

    let predicate_ctx = PredicateContext::new("hello");
    assert!(predicate_ctx.evaluate(&query.patterns[0].predicates[0], &captures));
}

#[test]
fn test_unsupported_predicate_returns_clear_error() {
    let grammar = create_test_grammar();
    let err = compile_query("(expression @expr) (#foo? @expr \"hello\")", &grammar)
        .expect_err("unsupported predicate should return an error");

    match err {
        QueryError::InvalidPredicate(message) => {
            assert!(message.contains("foo?"));
            assert!(message.contains("Unsupported predicate"));
        }
        other => panic!("expected InvalidPredicate, got {other:?}"),
    }
}
