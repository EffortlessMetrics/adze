#![no_main]

use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

/// Fuzzer-controlled symbol to generate complex grammar shapes.
#[derive(Debug, Arbitrary)]
enum FuzzSymbol {
    Terminal(u8),
    NonTerminal(u8),
    Optional(Box<FuzzSymbol>),
    Repeat(Box<FuzzSymbol>),
    RepeatOne(Box<FuzzSymbol>),
    Choice(Vec<FuzzSymbol>),
    Sequence(Vec<FuzzSymbol>),
    Epsilon,
}

impl FuzzSymbol {
    /// Convert to an IR Symbol, capping IDs to the token/rule range.
    fn to_symbol(&self, max_terminal: u16, max_nonterminal: u16) -> Symbol {
        match self {
            FuzzSymbol::Terminal(id) => {
                let capped = (*id as u16) % max_terminal.max(1);
                Symbol::Terminal(SymbolId(capped))
            }
            FuzzSymbol::NonTerminal(id) => {
                let capped = (*id as u16) % max_nonterminal.max(1);
                Symbol::NonTerminal(SymbolId(capped))
            }
            FuzzSymbol::Optional(inner) => {
                Symbol::Optional(Box::new(inner.to_symbol(max_terminal, max_nonterminal)))
            }
            FuzzSymbol::Repeat(inner) => {
                Symbol::Repeat(Box::new(inner.to_symbol(max_terminal, max_nonterminal)))
            }
            FuzzSymbol::RepeatOne(inner) => {
                Symbol::RepeatOne(Box::new(inner.to_symbol(max_terminal, max_nonterminal)))
            }
            FuzzSymbol::Choice(items) => {
                let items: Vec<_> = items
                    .iter()
                    .take(8)
                    .map(|s| s.to_symbol(max_terminal, max_nonterminal))
                    .collect();
                if items.is_empty() {
                    Symbol::Epsilon
                } else {
                    Symbol::Choice(items)
                }
            }
            FuzzSymbol::Sequence(items) => {
                let items: Vec<_> = items
                    .iter()
                    .take(8)
                    .map(|s| s.to_symbol(max_terminal, max_nonterminal))
                    .collect();
                if items.is_empty() {
                    Symbol::Epsilon
                } else {
                    Symbol::Sequence(items)
                }
            }
            FuzzSymbol::Epsilon => Symbol::Epsilon,
        }
    }

    /// Depth of nesting (to prevent overly deep grammars).
    fn depth(&self) -> usize {
        match self {
            FuzzSymbol::Terminal(_) | FuzzSymbol::NonTerminal(_) | FuzzSymbol::Epsilon => 0,
            FuzzSymbol::Optional(inner)
            | FuzzSymbol::Repeat(inner)
            | FuzzSymbol::RepeatOne(inner) => 1 + inner.depth(),
            FuzzSymbol::Choice(items) | FuzzSymbol::Sequence(items) => {
                1 + items.iter().map(|i| i.depth()).max().unwrap_or(0)
            }
        }
    }
}

#[derive(Debug, Arbitrary)]
struct FuzzRule {
    lhs_idx: u8,
    rhs: Vec<FuzzSymbol>,
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    num_terminals: u8,
    num_nonterminals: u8,
    rules: Vec<FuzzRule>,
}

fuzz_target!(|input: FuzzInput| {
    let num_terminals = (input.num_terminals as u16).clamp(1, 16);
    let num_nonterminals = (input.num_nonterminals as u16).clamp(1, 16);

    // Cap number of rules.
    if input.rules.len() > 64 {
        return;
    }

    let mut grammar = Grammar::new("fuzz_normalize".to_string());

    // Create terminals.
    for i in 0..num_terminals {
        grammar.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("t{}", i),
                pattern: TokenPattern::String(format!("t{}", i)),
                fragile: false,
            },
        );
    }

    // Create non-terminal rule names (IDs start after terminals).
    let nt_base = num_terminals;
    for i in 0..num_nonterminals {
        grammar
            .rule_names
            .insert(SymbolId(nt_base + i), format!("nt{}", i));
    }

    // Add rules.
    for (idx, fuzz_rule) in input.rules.iter().enumerate() {
        let lhs = SymbolId(nt_base + (fuzz_rule.lhs_idx as u16 % num_nonterminals));

        // Skip rules with deeply nested symbols.
        let too_deep = fuzz_rule.rhs.iter().any(|s| s.depth() > 6);
        if too_deep || fuzz_rule.rhs.len() > 16 {
            continue;
        }

        let rhs: Vec<Symbol> = fuzz_rule
            .rhs
            .iter()
            .map(|s| s.to_symbol(num_terminals, num_nonterminals))
            .collect();

        if rhs.is_empty() {
            continue;
        }

        grammar.add_rule(Rule {
            lhs,
            rhs,
            production_id: ProductionId(idx as u16),
            precedence: None,
            associativity: None,
            fields: vec![],
        });
    }

    // Normalize should not panic on any well-formed grammar.
    let _new_rules = grammar.normalize();
});
