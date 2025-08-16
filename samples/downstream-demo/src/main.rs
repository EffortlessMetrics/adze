use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_tablegen::compress::TableCompressor;
use rust_sitter_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};

fn main() {
    println!("=== Downstream Demo: Testing rust-sitter integration ===\n");

    // Create a nullable-start grammar: "module ::= ε | IDENT"
    let grammar = GrammarBuilder::new("demo")
        .token("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("module", vec![]) // ε (empty production)
        .rule("module", vec!["IDENT"])
        .start("module")
        .build();

    println!("Grammar created: {}", grammar.name);
    println!("Number of rules: {}", grammar.rules.len());

    // Compute FIRST/FOLLOW sets
    let first_follow = FirstFollowSets::compute(&grammar);
    println!("✓ FIRST/FOLLOW sets computed");

    // Build LR(1) parse table
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");
    println!(
        "✓ Parse table built with {} states",
        parse_table.state_count
    );

    // Check if start symbol is nullable
    let nullable_start = eof_accepts_or_reduces(&parse_table);
    println!("✓ Start symbol nullable? {}", nullable_start);

    // Collect token indices
    let token_indices = collect_token_indices(&grammar, &parse_table);
    println!("✓ Token indices collected: {} tokens", token_indices.len());

    // Compress the table
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&parse_table, &token_indices, nullable_start)
        .expect("Failed to compress table");

    println!("\n=== Compression Results ===");
    println!(
        "Action table compressed size: {} bytes",
        compressed.action_table.data.len()
    );
    println!(
        "Goto table compressed size: {} bytes",
        compressed.goto_table.data.len()
    );
    println!(
        "Small table threshold: {}",
        compressed.small_table_threshold
    );

    println!("\n✅ Demo completed successfully!");
    println!("This proves the rust-sitter crates integrate correctly.");
}
