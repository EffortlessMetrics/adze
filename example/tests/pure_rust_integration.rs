// Integration test for pure-Rust parser with generated languages

use rust_sitter_tool::pure_rust_builder::{build_parser_from_grammar_js, BuildOptions};
use rust_sitter_runtime::pure_parser::{Parser, ParseResult};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_arithmetic_parser() {
    // Define a simple arithmetic grammar
    let grammar_js = r#"
module.exports = grammar({
  name: 'arithmetic',
  
  rules: {
    expression: $ => choice(
      $.number,
      $.binary_expression,
      $.parenthesized_expression
    ),
    
    binary_expression: $ => choice(
      prec.left(1, seq(
        field('left', $.expression),
        '+',
        field('right', $.expression)
      )),
      prec.left(1, seq(
        field('left', $.expression),
        '-',
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        '*',
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        '/',
        field('right', $.expression)
      ))
    ),
    
    parenthesized_expression: $ => seq(
      '(',
      $.expression,
      ')'
    ),
    
    number: $ => /\d+/
  }
});
    "#;
    
    // Generate parser
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js).unwrap();
    
    let options = BuildOptions {
        out_dir: temp_dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    
    let result = build_parser_from_grammar_js(&grammar_path, options).unwrap();
    assert_eq!(result.grammar_name, "arithmetic");
    
    // Test cases
    let test_cases = vec![
        ("42", true),
        ("1 + 2", true),
        ("3 * 4 + 5", true),
        ("(6 + 7) * 8", true),
        ("9 + (10 * 11)", true),
        ("((12 + 13) * 14) - 15", true),
        ("1 + + 2", false), // Error: double operator
        ("(1 + 2", false),  // Error: missing closing paren
        ("+ 3", false),     // Error: leading operator
    ];
    
    for (input, should_succeed) in test_cases {
        println!("Testing: {}", input);
        
        let mut parser = Parser::new();
        let result = parser.parse_string(input);
        
        if should_succeed {
            assert!(result.root.is_some(), "Failed to parse valid input: {}", input);
            assert!(result.errors.is_empty(), "Unexpected errors for: {}", input);
        } else {
            assert!(!result.errors.is_empty(), "Expected errors for invalid input: {}", input);
        }
    }
}

#[test]
fn test_json_parser() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'json',
  
  rules: {
    value: $ => choice(
      $.null,
      $.boolean,
      $.number,
      $.string,
      $.array,
      $.object
    ),
    
    null: $ => 'null',
    
    boolean: $ => choice('true', 'false'),
    
    number: $ => /-?\d+(\.\d+)?([eE][+-]?\d+)?/,
    
    string: $ => /"([^"\\]|\\.)*"/,
    
    array: $ => seq(
      '[',
      optional(seq(
        $.value,
        repeat(seq(',', $.value))
      )),
      ']'
    ),
    
    object: $ => seq(
      '{',
      optional(seq(
        $.pair,
        repeat(seq(',', $.pair))
      )),
      '}'
    ),
    
    pair: $ => seq(
      field('key', $.string),
      ':',
      field('value', $.value)
    )
  }
});
    "#;
    
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js).unwrap();
    
    let options = BuildOptions {
        out_dir: temp_dir.path().to_string_lossy().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    
    let result = build_parser_from_grammar_js(&grammar_path, options).unwrap();
    assert_eq!(result.grammar_name, "json");
    
    // Test JSON parsing
    let test_cases = vec![
        (r#"null"#, true),
        (r#"true"#, true),
        (r#"false"#, true),
        (r#"42"#, true),
        (r#"-3.14"#, true),
        (r#"1.23e-4"#, true),
        (r#""hello""#, true),
        (r#""escaped \"quotes\"""#, true),
        (r#"[]"#, true),
        (r#"[1, 2, 3]"#, true),
        (r#"{}"#, true),
        (r#"{"key": "value"}"#, true),
        (r#"{"a": 1, "b": [2, 3], "c": {"d": null}}"#, true),
        (r#"{invalid}"#, false), // Missing quotes on key
        (r#"[1, 2,]"#, false),    // Trailing comma
    ];
    
    for (input, should_succeed) in test_cases {
        println!("Testing JSON: {}", input);
        
        let mut parser = Parser::new();
        let result = parser.parse_string(input);
        
        if should_succeed {
            assert!(result.root.is_some(), "Failed to parse valid JSON: {}", input);
        } else {
            assert!(!result.errors.is_empty(), "Expected errors for invalid JSON: {}", input);
        }
    }
}

#[test]
fn test_error_recovery() {
    // Test error recovery capabilities
    let grammar_js = r#"
module.exports = grammar({
  name: 'statements',
  
  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => choice(
      $.assignment,
      $.expression_statement
    ),
    
    assignment: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $.expression),
      ';'
    ),
    
    expression_statement: $ => seq(
      $.expression,
      ';'
    ),
    
    expression: $ => choice(
      $.identifier,
      $.number,
      $.binary_expression
    ),
    
    binary_expression: $ => prec.left(seq(
      field('left', $.expression),
      choice('+', '-', '*', '/'),
      field('right', $.expression)
    )),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    number: $ => /\d+/
  }
});
    "#;
    
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js).unwrap();
    
    let options = BuildOptions {
        out_dir: temp_dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    
    let _result = build_parser_from_grammar_js(&grammar_path, options).unwrap();
    
    // Test error recovery
    let error_prone_code = r#"
        x = 42;
        y = ;        // Missing value
        z = 10 + 20;
        a b = 30;    // Missing operator
        c = 40 + ;   // Missing operand
        d = 50;
    "#;
    
    let mut parser = Parser::new();
    let result = parser.parse_string(error_prone_code);
    
    // Should have errors but still produce some parse tree
    assert!(!result.errors.is_empty(), "Expected errors in error-prone code");
    println!("Found {} errors", result.errors.len());
    
    for error in &result.errors {
        println!("Error at {}:{} - expected {:?}, found {}", 
            error.point.row, 
            error.point.column,
            error.expected,
            error.found
        );
    }
}

#[test]
fn test_incremental_parsing_preparation() {
    // Test that parser can be reused for multiple parses
    let mut parser = Parser::new();
    
    // Parse multiple inputs with same parser
    let inputs = vec![
        "1 + 2",
        "3 * 4",
        "5 - 6",
        "(7 + 8) * 9",
    ];
    
    for input in inputs {
        parser.reset();
        let result = parser.parse_string(input);
        
        // Basic validation - in real implementation would check tree structure
        assert_eq!(result.errors.is_empty(), true);
    }
}

// Helper function to print parse tree (for debugging)
fn print_tree(result: &ParseResult, indent: usize) {
    if let Some(root) = &result.root {
        print_node(root, indent);
    }
    
    if !result.errors.is_empty() {
        println!("Errors:");
        for error in &result.errors {
            println!("  - At {}:{}: expected {:?}, found {}",
                error.point.row,
                error.point.column,
                error.expected,
                error.found
            );
        }
    }
}

fn print_node(node: &rust_sitter_runtime::pure_parser::ParsedNode, indent: usize) {
    println!("{}{} [{}-{}]",
        " ".repeat(indent),
        node.symbol(),
        node.start_byte(),
        node.end_byte()
    );
    
    for child in node.children() {
        print_node(child, indent + 2);
    }
}