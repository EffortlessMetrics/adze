#![no_main]

use adze::{
    glr_incremental::{GLREdit, GLRToken, IncrementalGLRParser},
    glr_lexer::GLRLexer,
};
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

// Reuse a small arithmetic grammar to keep fuzz iterations fast.
fn create_test_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("fuzz_test".to_string());

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

#[derive(Debug)]
struct FuzzInput {
    initial_text: String,
    edits: Vec<FuzzEdit>,
}

#[derive(Debug)]
struct FuzzEdit {
    start_byte: usize,
    old_end_byte: usize,
    new_text: String,
}

impl<'a> arbitrary::Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let nums: Vec<u8> = u.arbitrary()?;
        let initial_text = if nums.is_empty() {
            "1".to_string()
        } else {
            nums.iter()
                .take(10)
                .map(|n| (n % 10).to_string())
                .collect::<Vec<_>>()
                .join(" + ")
        };

        let num_edits: usize = u.int_in_range(0..=5)?;
        let mut edits = Vec::new();

        for _ in 0..num_edits {
            let text_len = initial_text.len();
            if text_len == 0 {
                continue;
            }

            let start = u.int_in_range(0..=text_len)?;
            let old_end = u.int_in_range(start..=text_len)?;

            let edit_type: u8 = u.int_in_range(0..=3)?;
            let new_text = match edit_type {
                0 => "".to_string(),
                1 => u.int_in_range(0..=99)?.to_string(),
                2 => " + ".to_string(),
                _ => format!("{} + {}", u.int_in_range(0..=9)?, u.int_in_range(0..=9)?),
            };

            edits.push(FuzzEdit {
                start_byte: start,
                old_end_byte: old_end,
                new_text,
            });
        }

        Ok(FuzzInput {
            initial_text,
            edits,
        })
    }
}

fn tokenize_to_glr(grammar: &Grammar, text: &str) -> Option<Vec<GLRToken>> {
    let mut lexer = GLRLexer::new(grammar, text.to_string()).ok()?;
    let tokens = lexer.tokenize_all();

    Some(
        tokens
            .into_iter()
            .map(|token| GLRToken {
                symbol: token.symbol_id,
                text: token.text.into_bytes(),
                start_byte: token.byte_offset,
                end_byte: token.byte_offset + token.byte_length,
            })
            .collect(),
    )
}

fuzz_target!(|input: FuzzInput| {
    if input.initial_text.trim().is_empty() {
        return;
    }

    let mut current_text = input.initial_text.clone();
    let mut current_tokens = match tokenize_to_glr(&TEST_GRAMMAR, &current_text) {
        Some(tokens) if !tokens.is_empty() => tokens,
        _ => return,
    };

    let mut incremental = IncrementalGLRParser::new((**TEST_GRAMMAR).clone(), PARSE_TABLE.clone());

    let mut current_forest = match incremental.parse_incremental(&current_tokens, &[]) {
        Ok(forest) => forest,
        Err(_) => return,
    };

    for fuzz_edit in input.edits {
        if fuzz_edit.start_byte > current_text.len()
            || fuzz_edit.old_end_byte > current_text.len()
            || fuzz_edit.start_byte > fuzz_edit.old_end_byte
            || !current_text.is_char_boundary(fuzz_edit.start_byte)
            || !current_text.is_char_boundary(fuzz_edit.old_end_byte)
        {
            continue;
        }

        let mut new_text = String::with_capacity(
            current_text.len() + fuzz_edit.new_text.len()
                - (fuzz_edit.old_end_byte - fuzz_edit.start_byte),
        );
        new_text.push_str(&current_text[..fuzz_edit.start_byte]);
        new_text.push_str(&fuzz_edit.new_text);
        new_text.push_str(&current_text[fuzz_edit.old_end_byte..]);

        let new_tokens = match tokenize_to_glr(&TEST_GRAMMAR, &new_text) {
            Some(tokens) if !tokens.is_empty() => tokens,
            _ => continue,
        };

        // Conservative edit descriptor for fuzzing robustness: replace the old token stream.
        let glr_edit = GLREdit {
            old_range: fuzz_edit.start_byte..fuzz_edit.old_end_byte,
            new_text: fuzz_edit.new_text.as_bytes().to_vec(),
            old_token_range: 0..current_tokens.len(),
            new_tokens: new_tokens.clone(),
            old_tokens: current_tokens.clone(),
            old_forest: Some(current_forest.clone()),
        };

        if let Ok(new_forest) = incremental.parse_incremental(&new_tokens, &[glr_edit]) {
            current_text = new_text;
            current_tokens = new_tokens;
            current_forest = new_forest;
        }
    }
});
