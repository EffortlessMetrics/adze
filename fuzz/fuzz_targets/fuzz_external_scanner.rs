#![no_main]

use adze_ir::validation::GrammarValidator;
use adze_ir::{
    ExternalToken, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 || data.len() > 2_000 {
        return;
    }

    // Use first byte to select grammar construction strategy
    let strategy = data[0] % 3;
    let payload = &data[1..];

    match strategy {
        0 => fuzz_grammar_names(payload),
        1 => fuzz_external_tokens(payload),
        _ => fuzz_validator(payload),
    }
});

/// Feed arbitrary bytes as grammar/symbol names to Grammar::new() and validate
fn fuzz_grammar_names(data: &[u8]) {
    let name = String::from_utf8_lossy(data).to_string();
    let mut grammar = Grammar::new(name);

    // Add tokens with fuzz-derived names
    for (i, chunk) in data.chunks(4).enumerate().take(20) {
        let token_name = String::from_utf8_lossy(chunk).to_string();
        let pattern = if i % 2 == 0 {
            TokenPattern::String(token_name.clone())
        } else {
            TokenPattern::Regex(token_name.clone())
        };
        grammar.tokens.insert(
            SymbolId(i as u16 + 1),
            Token {
                name: token_name,
                pattern,
                fragile: i % 3 == 0,
            },
        );
    }

    // Must never panic
    let _ = grammar.validate();
    let _ = grammar.check_empty_terminals();
    let _ = grammar.normalize();
}

/// Feed arbitrary bytes to construct external token declarations and validate
fn fuzz_external_tokens(data: &[u8]) {
    let mut grammar = Grammar::new("fuzz_ext".to_string());

    // Add a base terminal for rules to reference
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "tok".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    // Add external tokens from fuzz data
    for (i, chunk) in data.chunks(3).enumerate().take(30) {
        let ext_name = String::from_utf8_lossy(chunk).to_string();
        let ext_id = SymbolId(100 + i as u16);
        grammar.externals.push(ExternalToken {
            name: ext_name,
            symbol_id: ext_id,
        });

        // Add a rule referencing this external token
        grammar.add_rule(Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::External(ext_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }

    grammar
        .rule_names
        .insert(SymbolId(10), "start".to_string());

    // Must never panic
    let _ = grammar.validate();
    let _ = grammar.normalize();
}

/// Run the full GrammarValidator on fuzz-constructed grammars
fn fuzz_validator(data: &[u8]) {
    let mut grammar = Grammar::new("fuzz_validate".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    // Use fuzz bytes to build rules with varying structure
    let mut prod_id = 0u16;
    for chunk in data.chunks(2).take(50) {
        let lhs_id = SymbolId(10 + (chunk[0] % 5) as u16);
        let rhs_choice = chunk.get(1).unwrap_or(&0) % 5;

        let rhs = match rhs_choice {
            0 => vec![Symbol::Terminal(SymbolId(1))],
            1 => vec![Symbol::Terminal(SymbolId(2))],
            2 => vec![Symbol::NonTerminal(SymbolId(10 + (chunk[0] % 3) as u16))],
            3 => vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::NonTerminal(SymbolId(10)),
            ],
            _ => vec![Symbol::Epsilon],
        };

        let name = format!("rule_{}", lhs_id.0);
        grammar.rule_names.entry(lhs_id).or_insert(name);
        grammar.add_rule(Rule {
            lhs: lhs_id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        });
        prod_id = prod_id.wrapping_add(1);
    }

    // Must never panic
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&grammar);
    let _ = grammar.validate();
    let _ = grammar.normalize();
}
