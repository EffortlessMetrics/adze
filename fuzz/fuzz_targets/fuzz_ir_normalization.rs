#![no_main]

use adze_ir::{
    Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    let mut grammar = Grammar::new("fuzz".to_string());
    let mut cursor = 0;

    // Determine counts from first bytes
    let num_tokens = (data[cursor] % 10) as u16 + 1;
    cursor += 1;
    let num_rules = (data[cursor] % 10) as u16 + 1;
    cursor += 1;

    // Create tokens (IDs start at 100 to leave room for non-terminals)
    for i in 0..num_tokens {
        let token_id = SymbolId(100 + i);
        let name = format!("T{}", i);
        let pattern = if cursor < data.len() {
            let b = data[cursor];
            cursor += 1;
            let len = ((b % 4) as usize) + 1;
            let end = (cursor + len).min(data.len());
            let slice = &data[cursor..end];
            cursor = end;
            String::from_utf8_lossy(slice).into_owned()
        } else {
            format!("t{}", i)
        };
        grammar.tokens.insert(
            token_id,
            Token {
                name,
                pattern: TokenPattern::String(pattern),
                fragile: false,
            },
        );
    }

    // Register non-terminal rule names
    for i in 0..num_rules {
        let lhs_id = SymbolId(i);
        grammar.rule_names.insert(lhs_id, format!("rule_{}", i));
    }

    // Create rules with potentially complex symbols
    let mut prod_counter: u16 = 0;
    for i in 0..num_rules {
        let lhs_id = SymbolId(i);

        // Build RHS from fuzz bytes
        let rhs_len = if cursor < data.len() {
            (data[cursor] % 5) as usize + 1
        } else {
            1
        };
        cursor += 1;

        let mut rhs = Vec::new();
        for _ in 0..rhs_len {
            if cursor >= data.len() {
                rhs.push(Symbol::Terminal(SymbolId(100)));
                continue;
            }
            let sym_byte = data[cursor];
            cursor += 1;
            let sym = make_symbol(sym_byte, num_tokens, num_rules, data, &mut cursor);
            rhs.push(sym);
        }

        grammar.add_rule(Rule {
            lhs: lhs_id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_counter),
        });
        prod_counter = prod_counter.wrapping_add(1);
    }

    // The main thing we're fuzzing: normalize must not panic
    let _normalized = grammar.normalize();

    // Optional: validate doesn't panic either (may return errors, that's fine)
    let _ = grammar.validate();
});

/// Build a Symbol from fuzz bytes, potentially creating nested complex symbols.
fn make_symbol(
    byte: u8,
    num_tokens: u16,
    num_rules: u16,
    data: &[u8],
    cursor: &mut usize,
) -> Symbol {
    match byte % 10 {
        // Terminal
        0..=2 => {
            let token_idx = if *cursor < data.len() {
                let b = data[*cursor];
                *cursor += 1;
                (b as u16) % num_tokens
            } else {
                0
            };
            Symbol::Terminal(SymbolId(100 + token_idx))
        }
        // NonTerminal
        3..=4 => {
            let rule_idx = if *cursor < data.len() {
                let b = data[*cursor];
                *cursor += 1;
                (b as u16) % num_rules
            } else {
                0
            };
            Symbol::NonTerminal(SymbolId(rule_idx))
        }
        // Optional
        5 => {
            let inner = make_leaf_symbol(data, cursor, num_tokens, num_rules);
            Symbol::Optional(Box::new(inner))
        }
        // Repeat
        6 => {
            let inner = make_leaf_symbol(data, cursor, num_tokens, num_rules);
            Symbol::Repeat(Box::new(inner))
        }
        // RepeatOne
        7 => {
            let inner = make_leaf_symbol(data, cursor, num_tokens, num_rules);
            Symbol::RepeatOne(Box::new(inner))
        }
        // Choice
        8 => {
            let count = if *cursor < data.len() {
                let b = data[*cursor];
                *cursor += 1;
                (b % 3) as usize + 2
            } else {
                2
            };
            let choices: Vec<Symbol> = (0..count)
                .map(|_| make_leaf_symbol(data, cursor, num_tokens, num_rules))
                .collect();
            Symbol::Choice(choices)
        }
        // Sequence
        9 => {
            let count = if *cursor < data.len() {
                let b = data[*cursor];
                *cursor += 1;
                (b % 3) as usize + 2
            } else {
                2
            };
            let seq: Vec<Symbol> = (0..count)
                .map(|_| make_leaf_symbol(data, cursor, num_tokens, num_rules))
                .collect();
            Symbol::Sequence(seq)
        }
        _ => Symbol::Epsilon,
    }
}

/// Build a simple (non-recursive) leaf symbol to bound depth.
fn make_leaf_symbol(
    data: &[u8],
    cursor: &mut usize,
    num_tokens: u16,
    num_rules: u16,
) -> Symbol {
    if *cursor >= data.len() {
        return Symbol::Terminal(SymbolId(100));
    }
    let b = data[*cursor];
    *cursor += 1;
    match b % 3 {
        0 => Symbol::Terminal(SymbolId(100 + (b as u16 / 3) % num_tokens)),
        1 => Symbol::NonTerminal(SymbolId((b as u16 / 3) % num_rules)),
        _ => Symbol::Epsilon,
    }
}
