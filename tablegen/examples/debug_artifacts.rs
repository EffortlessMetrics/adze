use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{SymbolId, builder::GrammarBuilder};
use rust_sitter_tablegen::{collect_token_indices, eof_accepts_or_reduces};
use std::collections::BTreeMap;

fn main() {
    // Create a simple test grammar with nullable start
    let grammar = GrammarBuilder::new("test_grammar")
        // Add some tokens
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("NUM", r"\d+")
        // Build a simple grammar: start -> expr*, expr -> term ('+' term)*, term -> NUM ('*' NUM)*
        .rule("start", vec![]) // Nullable start symbol (empty rule)
        .rule("start", vec!["expr"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("term", vec!["NUM"])
        .rule("term", vec!["term", "STAR", "NUM"])
        .build();

    // Generate parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");

    // Display symbol->col mappings (first 10)
    println!("=== Symbol → Column Mappings (first 10) ===");
    let sorted_mappings: BTreeMap<_, _> = parse_table
        .symbol_to_index
        .iter()
        .map(|(k, v)| (k.0, *v))
        .collect();
    for (i, (symbol_id, col)) in sorted_mappings.iter().take(10).enumerate() {
        println!("  {} → {}", symbol_id, col);
        if i >= 9 {
            break;
        }
    }

    // Get token indices
    let token_indices = collect_token_indices(&grammar, &parse_table);
    println!("\n=== Token Indices from collect_token_indices ===");
    let mut sorted_indices: Vec<_> = token_indices.to_vec();
    sorted_indices.sort();
    println!("  {:?}", sorted_indices);

    // Check if start can be empty
    let start_nullable = eof_accepts_or_reduces(&parse_table);
    println!("\n=== Start Symbol Nullable Check ===");
    println!("  start_can_be_empty: {}", start_nullable);

    // Display state 0 action cells (first 12 non-empty)
    println!("\n=== State 0 Action Cells (first 12 non-empty) ===");
    if let Some(state_0_row) = parse_table.action_table.first() {
        let mut shown = 0;
        for (col, actions) in state_0_row.iter().enumerate() {
            if !actions.is_empty() && shown < 12 {
                println!("  col {}: {:?}", col, actions);
                shown += 1;
            }
        }
    }

    // Show EOF column explicitly
    let eof_col = parse_table.symbol_to_index.get(&SymbolId(0));
    println!("\n=== EOF Column Location ===");
    println!("  EOF (SymbolId(0)) is at column: {:?}", eof_col);
}
