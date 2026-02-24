use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, Rule, Symbol, SymbolId, Token, TokenPattern};

#[test]
fn rhs_only_terminal_stays_in_terminal_band() {
    let mut grammar = Grammar::new("rhs_only_terminal".to_string());

    let number_id = SymbolId(1);
    let plus_id = SymbolId(2); // Intentionally omitted from grammar.tokens
    let expr_id = SymbolId(10);

    grammar.tokens.insert(
        number_id,
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(expr_id, "expr".to_string());

    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::Terminal(number_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(1),
    });

    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should compute");
    let table = build_lr1_automaton(&grammar, &ff).expect("table build should succeed");

    let plus_col = *table
        .symbol_to_index
        .get(&plus_id)
        .expect("RHS-only terminal must be indexed");
    let expr_col = *table
        .symbol_to_index
        .get(&expr_id)
        .expect("nonterminal must be indexed");

    assert!(
        plus_col < table.token_count,
        "RHS-only terminal '+' must be in token band (col={}, token_count={})",
        plus_col,
        table.token_count
    );
    assert!(
        expr_col >= table.token_count + table.external_token_count,
        "nonterminal must be after terminal band (expr_col={}, token_count={}, external={})",
        expr_col,
        table.token_count,
        table.external_token_count
    );

    // EOF + NUMBER + '+'.
    assert_eq!(table.token_count, 3);
    assert_eq!(table.external_token_count, 0);
}
