#![no_main]

use adze::{glr_lexer::GLRLexer, glr_parser::GLRParser};
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
use std::thread;

fn create_test_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("fuzz_concurrent".to_string());

    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let star_id = SymbolId(3);

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

    grammar.tokens.insert(
        star_id,
        Token {
            name: "star".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

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
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(star_id),
            Symbol::NonTerminal(expr_id),
        ],
        production_id: ProductionId(1),
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: None,
        fields: vec![],
    });

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        production_id: ProductionId(2),
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
    inputs: Vec<Vec<u8>>,
}

fn parse_one(input: &str, grammar: &Arc<Grammar>, table: &adze_glr_core::ParseTable) {
    let Ok(mut lexer) = GLRLexer::new(grammar, input.to_string()) else {
        return;
    };
    let tokens = lexer.tokenize_all();

    let mut parser = GLRParser::new(table.clone(), (**grammar).clone());
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
    }
    let _ = parser.finish();
}

fuzz_target!(|input: FuzzInput| {
    // Limit to a small number of threads and small inputs.
    let inputs: Vec<String> = input
        .inputs
        .iter()
        .take(4)
        .filter_map(|bytes| {
            let s = String::from_utf8_lossy(bytes);
            if s.trim().is_empty() || s.len() > 1_000 {
                None
            } else {
                Some(s.into_owned())
            }
        })
        .collect();

    if inputs.is_empty() {
        return;
    }

    let grammar = Arc::clone(&TEST_GRAMMAR);
    let table = PARSE_TABLE.clone();

    // Spawn threads that parse concurrently, sharing the grammar and table.
    let handles: Vec<_> = inputs
        .into_iter()
        .map(|text| {
            let g = Arc::clone(&grammar);
            let t = table.clone();
            thread::spawn(move || {
                parse_one(&text, &g, &t);
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }
});
