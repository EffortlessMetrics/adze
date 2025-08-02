// Demo of pure-Rust Tree-sitter implementation

use rust_sitter_runtime::pure_parser::Parser;
use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pure-Rust Tree-sitter Demo ===\n");

    // Step 1: Define a grammar
    let grammar_js = r#"
module.exports = grammar({
  name: 'calc',
  
  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => seq(
      $.expression,
      optional(';')
    ),
    
    expression: $ => choice(
      $.number,
      $.identifier,
      $.binary_expression,
      $.function_call,
      $.parenthesized_expression
    ),
    
    binary_expression: $ => choice(
      prec.left(1, seq($.expression, '+', $.expression)),
      prec.left(1, seq($.expression, '-', $.expression)),
      prec.left(2, seq($.expression, '*', $.expression)),
      prec.left(2, seq($.expression, '/', $.expression)),
      prec.right(3, seq($.expression, '^', $.expression))
    ),
    
    function_call: $ => seq(
      field('name', $.identifier),
      '(',
      optional(seq(
        $.expression,
        repeat(seq(',', $.expression))
      )),
      ')'
    ),
    
    parenthesized_expression: $ => seq('(', $.expression, ')'),
    
    number: $ => /\d+(\.\d+)?/,
    
    identifier: $ => /[a-zA-Z_]\w*/
  }
});
    "#;

    // Step 2: Generate parser
    println!("Generating parser from grammar...");

    let temp_dir = Path::new("target/pure_rust_demo");
    fs::create_dir_all(temp_dir)?;

    let grammar_path = temp_dir.join("grammar.js");
    fs::write(&grammar_path, grammar_js)?;

    let options = BuildOptions {
        out_dir: temp_dir.to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };

    let build_result = build_parser_from_grammar_js(&grammar_path, options)?;
    println!(
        "✓ Generated parser for language: {}",
        build_result.grammar_name
    );
    println!("✓ Parser saved to: {}", build_result.parser_path);

    // Step 3: Parse some code
    println!("\nParsing expressions...\n");

    let test_expressions = vec![
        "42",
        "x + y",
        "2 * 3 + 4",
        "(5 + 6) * 7",
        "sin(3.14)",
        "max(10, 20 + 30)",
        "2 ^ 3 ^ 4", // Right associative
        "a + b * c + d",
    ];

    let mut parser = Parser::new();

    for expr in test_expressions {
        println!("Input: {}", expr);

        let result = parser.parse_string(expr);

        if let Some(root) = &result.root {
            println!("  ✓ Parsed successfully");
            print_tree_structure(root, 2);
        } else {
            println!("  ✗ Parse failed");
        }

        if !result.errors.is_empty() {
            println!("  Errors:");
            for error in &result.errors {
                println!(
                    "    - At position {}: expected symbols {:?}, found {}",
                    error.position, error.expected, error.found
                );
            }
        }

        println!();
        parser.reset();
    }

    // Step 4: Show error recovery
    println!("Testing error recovery...\n");

    let error_cases = vec![
        ("1 + + 2", "Double operator"),
        ("(3 + 4", "Missing closing parenthesis"),
        ("5 * (6 + )", "Missing operand"),
        ("sin(", "Incomplete function call"),
        ("7 8", "Missing operator between numbers"),
    ];

    for (expr, description) in error_cases {
        println!("Input: {} ({})", expr, description);

        let result = parser.parse_string(expr);

        if result.errors.is_empty() {
            println!("  ⚠ Unexpectedly parsed without errors");
        } else {
            println!("  ✓ Detected {} error(s)", result.errors.len());
            for error in &result.errors {
                println!(
                    "    - At {}:{}: expected {:?}",
                    error.point.row + 1,
                    error.point.column + 1,
                    error.expected
                );
            }
        }

        println!();
        parser.reset();
    }

    // Step 5: Performance test
    println!("Performance test...\n");

    let large_expr = generate_large_expression(100);
    println!(
        "Parsing expression with ~{} tokens...",
        large_expr.split_whitespace().count()
    );

    let start = std::time::Instant::now();
    let result = parser.parse_string(&large_expr);
    let elapsed = start.elapsed();

    println!("  ✓ Parsed in {:?}", elapsed);
    if let Some(root) = &result.root {
        println!("  ✓ Tree has {} nodes", count_nodes(root));
    }
    println!("  ✓ Found {} errors", result.errors.len());

    Ok(())
}

fn print_tree_structure(node: &rust_sitter_runtime::pure_parser::ParsedNode, indent: usize) {
    let indent_str = " ".repeat(indent);
    println!(
        "{}├─ Symbol {} [{}-{}]",
        indent_str,
        node.symbol(),
        node.start_byte(),
        node.end_byte()
    );

    for (i, child) in node.children().iter().enumerate() {
        if i == node.children().len() - 1 {
            print_tree_structure(child, indent + 2);
        } else {
            print_tree_structure(child, indent + 2);
        }
    }
}

fn count_nodes(node: &rust_sitter_runtime::pure_parser::ParsedNode) -> usize {
    1 + node.children().iter().map(count_nodes).sum::<usize>()
}

fn generate_large_expression(depth: usize) -> String {
    if depth == 0 {
        "x".to_string()
    } else {
        format!(
            "({} + {} * {})",
            generate_large_expression(depth - 1),
            depth,
            generate_large_expression(depth - 1)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs() {
        main().unwrap();
    }
}
