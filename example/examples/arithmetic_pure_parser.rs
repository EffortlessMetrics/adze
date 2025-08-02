// Example using the pure-Rust parser with arithmetic grammar
use rust_sitter::pure_parser::Parser;
use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Arithmetic Grammar - Pure Rust Parser Example");
    println!("============================================\n");

    // Define a simple arithmetic grammar
    let grammar_js = r#"
module.exports = grammar({
  name: 'arithmetic',
  
  rules: {
    source_file: $ => $._expression,
    
    _expression: $ => choice(
      $.number,
      $.binary_expression,
      $.parenthesized_expression
    ),
    
    number: $ => /\d+/,
    
    binary_expression: $ => choice(
      prec.left(1, seq($._expression, '+', $._expression)),
      prec.left(1, seq($._expression, '-', $._expression)),
      prec.left(2, seq($._expression, '*', $._expression)),
      prec.left(2, seq($._expression, '/', $._expression))
    ),
    
    parenthesized_expression: $ => seq('(', $._expression, ')')
  }
});
    "#;

    // Create a temporary directory for outputs
    let temp_dir = TempDir::new()?;
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js)?;

    println!("Step 1: Building parser from grammar.js...");

    // Build the parser
    let options = BuildOptions {
        out_dir: temp_dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    let result = build_parser_from_grammar_js(&grammar_path, options)?;

    println!("✓ Parser built successfully!");
    println!("  Grammar name: {}", result.grammar_name);
    println!("  Parser module: {}", result.parser_path);

    // Show generated NODE_TYPES
    println!("\nStep 2: Generated NODE_TYPES.json:");
    let node_types: serde_json::Value = serde_json::from_str(&result.node_types_json)?;
    println!("{}", serde_json::to_string_pretty(&node_types)?);

    // Test expressions
    let test_expressions = vec![
        "42",
        "1 + 2",
        "3 * 4 + 5",
        "6 + 7 * 8",
        "(9 + 10) * 11",
        "12 / (13 - 14)",
    ];

    println!("\nStep 3: Testing expressions:");
    println!("(Note: Actual parsing would require loading the generated language)");

    for expr in test_expressions {
        println!("\n  Expression: '{}'", expr);
        // In a real implementation, you would:
        // 1. Load the generated language module
        // 2. Create a parser and set the language
        // 3. Parse the expression
        // 4. Print the parse tree
    }

    println!("\n✓ Example completed successfully!");
    println!("\nKey Benefits of Pure-Rust Implementation:");
    println!("- No C compiler required");
    println!("- Fully compatible with Tree-sitter ecosystem");
    println!("- Can be compiled to WASM");
    println!("- Type-safe and memory-safe");

    Ok(())
}
