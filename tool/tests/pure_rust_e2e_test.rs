// End-to-end test for pure-Rust parser generation

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
use std::fs;
use tempfile::TempDir;

#[test]
#[ignore]
fn test_json_grammar_generation() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'json',
  
  rules: {
    document: $ => $._value,
    
    _value: $ => choice(
      $.object,
      $.array,
      $.string,
      $.number,
      $.true,
      $.false,
      $.null
    ),
    
    object: $ => seq(
      '{',
      optional($.pair),
      repeat(seq(',', $.pair)),
      '}'
    ),
    
    pair: $ => seq(
      field('key', $.string),
      ':',
      field('value', $._value)
    ),
    
    array: $ => seq(
      '[',
      optional($._value),
      repeat(seq(',', $._value)),
      ']'
    ),
    
    string: $ => /\"[^\"]*\"/,
    number: $ => /-?\d+(\.\d+)?([eE][+-]?\d+)?/,
    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null'
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

    // Verify result
    assert_eq!(result.grammar_name, "json");

    // Check generated code
    assert!(result.parser_code.contains("tree_sitter_json"));
    assert!(result.parser_code.contains("TSLanguage"));

    // Check NODE_TYPES
    let node_types: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(node_types.is_array());

    // Verify parser file was created
    let parser_file = std::path::Path::new(&result.parser_path);
    assert!(parser_file.exists());
}

#[test]
fn test_expression_grammar_with_precedence() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'calc',
  
  rules: {
    expression: $ => choice(
      $.binary_expression,
      $.unary_expression,
      $.number,
      $.parenthesized_expression
    ),
    
    binary_expression: $ => choice(
      prec.left(2, seq(
        field('left', $.expression),
        field('operator', '*'),
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        field('operator', '/'),
        field('right', $.expression)
      )),
      prec.left(1, seq(
        field('left', $.expression),
        field('operator', '+'),
        field('right', $.expression)
      )),
      prec.left(1, seq(
        field('left', $.expression),
        field('operator', '-'),
        field('right', $.expression)
      ))
    ),
    
    unary_expression: $ => prec(3, seq(
      field('operator', '-'),
      field('operand', $.expression)
    )),
    
    parenthesized_expression: $ => seq(
      '(',
      $.expression,
      ')'
    ),
    
    number: $ => /\d+(\.\d+)?/
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

    // Verify result
    assert_eq!(result.grammar_name, "calc");

    // Check that precedence is handled
    assert!(result.parser_code.contains("TSLanguage"));

    // Verify artifacts were emitted
    let grammar_dir = temp_dir.path().join("grammar_calc");
    assert!(grammar_dir.exists());

    let ir_file = grammar_dir.join("grammar.ir.json");
    assert!(ir_file.exists());

    let node_types_file = grammar_dir.join("NODE_TYPES.json");
    assert!(node_types_file.exists());
}

#[test]
fn test_compressed_vs_uncompressed() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'simple',
  
  rules: {
    program: $ => repeat($.statement),
    statement: $ => choice(
      $.assignment,
      $.expression_statement
    ),
    assignment: $ => seq(
      field('left', $.identifier),
      '=',
      field('right', $.expression)
    ),
    expression_statement: $ => $.expression,
    expression: $ => choice(
      $.identifier,
      $.number
    ),
    identifier: $ => /[a-zA-Z_]\w*/,
    number: $ => /\d+/
  }
});
    "#;

    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_js).unwrap();

    // Build with compression
    let compressed_options = BuildOptions {
        out_dir: temp_dir
            .path()
            .join("compressed")
            .to_string_lossy()
            .to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };

    let compressed_result =
        build_parser_from_grammar_js(&grammar_path, compressed_options).unwrap();

    // Build without compression
    let uncompressed_options = BuildOptions {
        out_dir: temp_dir
            .path()
            .join("uncompressed")
            .to_string_lossy()
            .to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };

    let uncompressed_result =
        build_parser_from_grammar_js(&grammar_path, uncompressed_options).unwrap();

    // Both should produce valid parsers
    assert!(compressed_result.parser_code.contains("TSLanguage"));
    assert!(uncompressed_result.parser_code.contains("TSLanguage"));

    // Compressed version should have PARSE_TABLE
    assert!(compressed_result.parser_code.contains("PARSE_TABLE"));
}
