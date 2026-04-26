use adze::query::{QueryError, compile_query};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

fn create_query_grammar() -> Grammar {
    let mut grammar = Grammar::new("query-canary".to_string());

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
        .insert(identifier, "identifier".to_string());
    grammar
        .rule_names
        .insert(expression, "expression".to_string());

    grammar
}

#[test]
fn test_eq_predicate_compiles_for_known_predicate() {
    let grammar = create_query_grammar();
    let query_src = r#"
        (identifier @id)
        (#eq? @id "alpha")
    "#;

    let query = compile_query(query_src, &grammar).expect("#eq? should compile");

    assert_eq!(query.patterns.len(), 1);
    assert_eq!(query.patterns[0].predicates.len(), 1);
    assert_eq!(query.capture_index("id"), Some(0));
}

#[test]
fn test_unsupported_predicate_returns_clear_error() {
    let grammar = create_query_grammar();
    let query_src = r#"
        (identifier @id)
        (#contains? @id "alp")
    "#;

    let err = compile_query(query_src, &grammar).expect_err("unsupported predicate should fail");

    match err {
        QueryError::InvalidPredicate(message) => {
            assert!(
                message.contains("#contains?"),
                "unexpected message: {message}"
            );
            assert!(
                message.contains("Unsupported predicate"),
                "unexpected message: {message}"
            );
        }
        other => panic!("expected InvalidPredicate, got {other:?}"),
    }
}
