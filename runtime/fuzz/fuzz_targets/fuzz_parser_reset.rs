#![no_main]

use adze::{glr_lexer::GLRLexer, glr_parser::GLRParser};
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fn create_test_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("fuzz_reset".to_string());

    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);

    grammar.rule_names.insert(expr_id, "expression".to_string());

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    // E -> E + E | number
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        production_id: ProductionId(0),
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: None,
        fields: vec![],
    });

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        production_id: ProductionId(1),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    Arc::new(grammar)
}

lazy_static::lazy_static! {
    static ref TEST_GRAMMAR: Arc<Grammar> = create_test_grammar();
    static ref PARSE_TABLE: adze_glr_core::ParseTable = {
        let ff_sets = FirstFollowSets::compute(&TEST_GRAMMAR).unwrap();
        build_lr1_automaton(&TEST_GRAMMAR, &ff_sets)
            .expect("Failed to build parse table for test grammar")
    };
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    first_input: Vec<u8>,
    second_input: Vec<u8>,
}

/// Parse a string, returning whether it succeeded.
fn try_parse(input: &str) -> bool {
    let lexer_result = GLRLexer::new(&TEST_GRAMMAR, input.to_string());
    let Ok(mut lexer) = lexer_result else {
        return false;
    };
    let tokens = lexer.tokenize_all();

    let mut parser = GLRParser::new(PARSE_TABLE.clone(), (**TEST_GRAMMAR).clone());
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
    }
    parser.finish().is_ok()
}

fuzz_target!(|input: FuzzInput| {
    let first = String::from_utf8_lossy(&input.first_input);
    let second = String::from_utf8_lossy(&input.second_input);

    if first.len() > 10_000 || second.len() > 10_000 {
        return;
    }

    // Parse first input
    let _ = try_parse(&first);

    // Parse second input (reusing nothing — tests that creating a fresh parser after
    // a previous parse doesn't corrupt state)
    let _ = try_parse(&second);

    // Now test actual reset: create one parser, parse, reset, parse again
    let lexer1 = GLRLexer::new(&TEST_GRAMMAR, first.to_string());
    let lexer2 = GLRLexer::new(&TEST_GRAMMAR, second.to_string());

    if let (Ok(mut l1), Ok(mut l2)) = (lexer1, lexer2) {
        let tokens1 = l1.tokenize_all();
        let tokens2 = l2.tokenize_all();

        let mut parser = GLRParser::new(PARSE_TABLE.clone(), (**TEST_GRAMMAR).clone());

        // First parse
        for token in &tokens1 {
            parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        }
        let _result1 = parser.finish();

        // Reset
        parser.reset();

        // Second parse after reset
        for token in &tokens2 {
            parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        }
        let _result2 = parser.finish();
    }
});
