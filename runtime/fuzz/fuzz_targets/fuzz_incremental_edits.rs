#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_sitter::{
    glr_incremental::{Edit, IncrementalGLRParser, Position},
    glr_lexer::GLRLexer,
    glr_parser::GLRParser,
};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{
    Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use std::sync::Arc;

// Reuse the test grammar from the other fuzz target
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

    // Simple grammar: E -> E + E | number
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
    static ref PARSE_TABLE: rust_sitter_glr_core::ParseTable = {
        let ff_sets = FirstFollowSets::compute(&TEST_GRAMMAR);
        build_lr1_automaton(&TEST_GRAMMAR, &ff_sets)
            .expect("Failed to build parse table for test grammar")
    };
}

// Fuzzer input structure
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
        // Generate initial text (simple arithmetic expression)
        let nums: Vec<u8> = u.arbitrary()?;
        let initial_text = if nums.is_empty() {
            "1".to_string()
        } else {
            nums.iter()
                .take(10) // Limit size
                .map(|n| (n % 10).to_string())
                .collect::<Vec<_>>()
                .join(" + ")
        };

        // Generate 0-5 edits
        let num_edits: usize = u.int_in_range(0..=5)?;
        let mut edits = Vec::new();

        for _ in 0..num_edits {
            let text_len = initial_text.len();
            if text_len == 0 {
                continue;
            }

            let start = u.int_in_range(0..=text_len)?;
            let old_end = u.int_in_range(start..=text_len)?;

            // Generate new text for the edit
            let edit_type: u8 = u.int_in_range(0..=3)?;
            let new_text = match edit_type {
                0 => "".to_string(),                      // Deletion
                1 => u.int_in_range(0..=99)?.to_string(), // Insert number
                2 => " + ".to_string(),                   // Insert operator
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

fuzz_target!(|input: FuzzInput| {
    // Skip empty input
    if input.initial_text.trim().is_empty() {
        return;
    }

    // Parse initial text
    let lexer_result = GLRLexer::new(&TEST_GRAMMAR, input.initial_text.clone());

    let initial_tokens = match lexer_result {
        Ok(mut lexer) => lexer.tokenize_all(),
        Err(_) => return, // Invalid initial input, skip
    };

    // Create incremental parser
    let glr_parser = GLRParser::new(PARSE_TABLE.clone(), (**TEST_GRAMMAR).clone());
    let mut incremental = IncrementalGLRParser::new(glr_parser, TEST_GRAMMAR.clone());

    // Parse initial text
    let tree_result = incremental.parse_incremental(&initial_tokens, &[], None);

    let mut current_tree = match tree_result {
        Ok(tree) => tree,
        Err(_) => return, // Initial parse failed, skip
    };

    let mut current_text = input.initial_text.clone();

    // Apply edits incrementally
    for fuzz_edit in input.edits {
        // Skip invalid edits
        if fuzz_edit.start_byte > current_text.len()
            || fuzz_edit.old_end_byte > current_text.len()
            || fuzz_edit.start_byte > fuzz_edit.old_end_byte
        {
            continue;
        }

        // Apply edit to text
        let new_text = format!(
            "{}{}{}",
            &current_text[..fuzz_edit.start_byte],
            &fuzz_edit.new_text,
            &current_text[fuzz_edit.old_end_byte..]
        );

        // Create Edit struct
        let edit = Edit {
            start_byte: fuzz_edit.start_byte,
            old_end_byte: fuzz_edit.old_end_byte,
            new_end_byte: fuzz_edit.start_byte + fuzz_edit.new_text.len(),
            start_position: Position {
                line: 0,
                column: fuzz_edit.start_byte,
            },
            old_end_position: Position {
                line: 0,
                column: fuzz_edit.old_end_byte,
            },
            new_end_position: Position {
                line: 0,
                column: fuzz_edit.start_byte + fuzz_edit.new_text.len(),
            },
        };

        // Tokenize new text
        let new_lexer_result = GLRLexer::new(&TEST_GRAMMAR, new_text.clone());

        let new_tokens = match new_lexer_result {
            Ok(mut lexer) => lexer.tokenize_all(),
            Err(_) => continue, // Invalid edit result, skip
        };

        // Parse incrementally - should not panic
        let new_tree_result =
            incremental.parse_incremental(&new_tokens, &[edit], Some(current_tree.clone()));

        match new_tree_result {
            Ok(new_tree) => {
                current_tree = new_tree;
                current_text = new_text;
            }
            Err(_) => {
                // Parse error is fine, we're testing robustness
            }
        }
    }
});
