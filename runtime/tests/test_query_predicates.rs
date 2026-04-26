use adze::query::{QueryError, compile_query};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex("[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            fragile: false,
        },
    );

    // Minimal nonterminal so query parser can resolve both token and rule names.
    grammar.rules.entry(SymbolId(100)).or_default().push(Rule {
        lhs: SymbolId(100),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
    });
    grammar
        .rule_names
        .insert(SymbolId(100), "expression".to_string());

    grammar
}

#[test]
fn test_eq_predicate_canary_compiles() {
    let grammar = create_test_grammar();
    let query_source = r#"
        (identifier @id)
        (#eq? @id "name")
    "#;

    let query = compile_query(query_source, &grammar).expect("query should compile");

    assert_eq!(query.pattern_count(), 1);
    assert_eq!(query.capture_index("id"), Some(0));
    assert_eq!(query.patterns[0].predicates.len(), 1);
}

#[test]
fn test_unsupported_predicate_returns_clear_error() {
    let grammar = create_test_grammar();
    let query_source = r#"
        (identifier @id)
        (#contains? @id "x")
    "#;

    let err = compile_query(query_source, &grammar).expect_err("predicate should be rejected");

    match err {
        QueryError::InvalidPredicate(message) => {
            assert!(message.contains("#contains?"), "message was: {message}");
            assert!(
                message.contains("supported"),
                "message should list supported predicates: {message}"
            );
        }
        other => panic!("unexpected error kind: {other:?}"),
    }
}
