use adze::query::{QueryError, compile_query};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

fn create_query_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("query-test".to_string());

    let identifier = SymbolId(1);
    let number = SymbolId(2);
    let expression = SymbolId(10);

    grammar.tokens.insert(
        identifier,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex("[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        number,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex("[0-9]+".to_string()),
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

#[test]
fn test_eq_predicate_compiles_as_tree_sitter_style_canary() {
    let grammar = create_query_test_grammar();
    let query_source = r#"
        (identifier @id)
        (#eq? @id "if")
    "#;

    let query = compile_query(query_source, &grammar).expect("#eq? should compile");

    assert_eq!(query.patterns.len(), 1);
    assert_eq!(query.patterns[0].predicates.len(), 1);
    assert_eq!(query.capture_index("id"), Some(0));
}

#[test]
fn test_unsupported_predicate_reports_clear_error() {
    let grammar = create_query_test_grammar();
    let query_source = r#"
        (identifier @id)
        (#contains? @id "foo")
    "#;

    let err = compile_query(query_source, &grammar).expect_err("unsupported predicate should fail");

    match err {
        QueryError::InvalidPredicate(message) => {
            assert!(
                message.contains("contains?"),
                "error should include predicate name, got: {message}"
            );
            assert!(
                message.contains("Supported predicates"),
                "error should list supported predicates, got: {message}"
            );
        }
        other => panic!("expected InvalidPredicate, got {other:?}"),
    }
}
