#![no_main]

use adze_glr_core::FirstFollowSets;
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use libfuzzer_sys::fuzz_target;

/// Build a small arithmetic grammar, then mutate it using fuzzer bytes
/// to exercise normalization and FIRST/FOLLOW computation.
fn build_grammar_from_fuzz(data: &[u8]) -> Grammar {
    let mut grammar = Grammar::new("fuzz_parser".to_string());

    // Fixed token definitions
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "star".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    // Non-terminal IDs
    let expr = SymbolId(10);
    let term = SymbolId(11);

    grammar.rule_names.insert(expr, "expr".to_string());
    grammar.rule_names.insert(term, "term".to_string());

    // Base rules: expr -> term, term -> number
    grammar.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::NonTerminal(term)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: term,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Use fuzzer bytes to add more rules with complex symbols
    let mut prod_id = 2u16;
    for chunk in data.chunks(3) {
        if prod_id > 200 {
            break;
        }
        let kind = chunk[0] % 6;
        let lhs = if chunk.get(1).unwrap_or(&0) % 2 == 0 { expr } else { term };
        let rhs = match kind {
            0 => {
                // expr -> expr + term
                vec![
                    Symbol::NonTerminal(expr),
                    Symbol::Terminal(SymbolId(2)),
                    Symbol::NonTerminal(term),
                ]
            }
            1 => {
                // Optional symbol
                vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))]
            }
            2 => {
                // Repeat symbol
                vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))))]
            }
            3 => {
                // Choice
                vec![Symbol::Choice(vec![
                    Symbol::Terminal(SymbolId(1)),
                    Symbol::Terminal(SymbolId(2)),
                ])]
            }
            4 => {
                // Sequence
                vec![Symbol::Sequence(vec![
                    Symbol::Terminal(SymbolId(1)),
                    Symbol::Terminal(SymbolId(3)),
                    Symbol::Terminal(SymbolId(1)),
                ])]
            }
            _ => {
                // Epsilon
                vec![Symbol::Epsilon]
            }
        };

        grammar.add_rule(Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        });
        prod_id += 1;
    }

    grammar
}

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_000 {
        return;
    }

    let mut grammar = build_grammar_from_fuzz(data);

    // Exercise normalization - must never panic
    let _ = grammar.normalize();

    // Exercise validation - must never panic
    let _ = grammar.validate();
    let _ = grammar.check_empty_terminals();

    // Exercise FIRST/FOLLOW computation - must never panic
    let _ = FirstFollowSets::compute(&grammar);
});
