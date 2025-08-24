#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_sitter::{glr_lexer::GLRLexer, glr_parser::GLRParser};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{
    Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use std::sync::Arc;

// Create a test grammar that can handle various inputs
fn create_test_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("fuzz_test".to_string());

    // Symbol IDs
    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let ident_id = SymbolId(2);
    let plus_id = SymbolId(3);
    let star_id = SymbolId(4);
    let lparen_id = SymbolId(5);
    let rparen_id = SymbolId(6);

    // Mark expr as start symbol
    grammar.rule_names.insert(expr_id, "expression".to_string());

    // Define tokens with patterns that could match fuzzer input
    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        ident_id,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
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

    grammar.tokens.insert(
        lparen_id,
        Token {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        rparen_id,
        Token {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );

    // Grammar rules: E -> E + E | E * E | ( E ) | number | identifier
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
        rhs: vec![
            Symbol::Terminal(lparen_id),
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(rparen_id),
        ],
        production_id: ProductionId(2),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        production_id: ProductionId(3),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(ident_id)],
        production_id: ProductionId(4),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    Arc::new(grammar)
}

// Create static parse table to avoid rebuilding on every fuzz iteration
lazy_static::lazy_static! {
    static ref TEST_GRAMMAR: Arc<Grammar> = create_test_grammar();
    static ref PARSE_TABLE: rust_sitter_glr_core::ParseTable = {
        let ff_sets = FirstFollowSets::compute(&TEST_GRAMMAR);
        build_lr1_automaton(&TEST_GRAMMAR, &ff_sets)
            .expect("Failed to build parse table for test grammar")
    };
}

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string (ignore invalid UTF-8)
    let input = String::from_utf8_lossy(data);

    // Skip empty input
    if input.trim().is_empty() {
        return;
    }

    // Try to tokenize the input
    let lexer_result = GLRLexer::new(&TEST_GRAMMAR, input.to_string());

    match lexer_result {
        Ok(mut lexer) => {
            // Tokenize the input
            let tokens = lexer.tokenize_all();

            // Create parser and try to parse
            let mut glr_parser = GLRParser::new(PARSE_TABLE.clone(), (**TEST_GRAMMAR).clone());

            // Parse should not panic, even with malformed input
            // Process each token
            for token in &tokens {
                glr_parser.process_token(token.symbol_id, &token.text, token.byte_offset);
            }

            // Finish parsing
            let _result = glr_parser.finish();

            // We don't care about the result, just that it doesn't panic
        }
        Err(_) => {
            // Lexer error is fine, we're testing robustness
        }
    }
});
