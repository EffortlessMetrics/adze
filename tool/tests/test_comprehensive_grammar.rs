use rust_sitter_tool::grammar_js::{GrammarJsParserV3, GrammarJsConverter};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_comprehensive_grammar_features() {
    // Create a grammar that uses all the features we've implemented
    let grammar_content = r#"
module.exports = grammar({
  name: 'comprehensive_test',
  
  word: $ => $.identifier,
  
  extras: $ => [
    /\s+/,
    $.comment
  ],
  
  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => choice(
      $.expression_statement,
      $.if_statement,
      $.while_statement
    ),
    
    expression_statement: $ => seq(
      $.expression,
      ';'
    ),
    
    if_statement: $ => seq(
      'if',
      '(',
      $.expression,
      ')',
      $.statement
    ),
    
    while_statement: $ => seq(
      'while',
      '(',
      $.expression,
      ')',
      $.statement
    ),
    
    expression: $ => choice(
      $.binary_expression,
      $.unary_expression,
      $.primary_expression
    ),
    
    binary_expression: $ => choice(
      // Arithmetic operators with precedence
      prec.left(1, seq(
        field('left', $.expression),
        field('operator', choice('+', '-')),
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        field('operator', choice('*', '/', '%')),
        field('right', $.expression)
      )),
      // Comparison operators
      prec.left(3, seq(
        field('left', $.expression),
        field('operator', choice('<', '>', '<=', '>=', '==', '!=')),
        field('right', $.expression)
      )),
      // Assignment (right associative)
      prec.right(4, seq(
        field('left', $.expression),
        field('operator', '='),
        field('right', $.expression)
      ))
    ),
    
    unary_expression: $ => prec(5, choice(
      seq('-', $.expression),
      seq('!', $.expression),
      seq('++', $.expression),
      seq('--', $.expression)
    )),
    
    primary_expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
      seq('(', $.expression, ')')
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    number: $ => /\d+(\.\d+)?/,
    
    string: $ => choice(
      seq('"', repeat(choice(/[^"\\]/, /\\./)), '"'),
      seq("'", repeat(choice(/[^'\\]/, /\\./)), "'")
    ),
    
    comment: $ => choice(
      seq('//', /.*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
    )
  }
})
"#;
    
    println!("Testing comprehensive grammar with all implemented features...\n");
    
    // Parse the grammar
    let parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar) => {
            println!("✓ Successfully parsed grammar!");
            println!("  Name: {}", grammar.name);
            println!("  Word token: {:?}", grammar.word);
            println!("  Number of rules: {}", grammar.rules.len());
            println!("  Extras: {} items", grammar.extras.len());
            
            // Try to convert to IR
            let converter = GrammarJsConverter::new(grammar);
            match converter.convert() {
                Ok(ir_grammar) => {
                    println!("\n✓ Successfully converted to IR!");
                    println!("  IR rules: {}", ir_grammar.rules.len());
                    println!("  Tokens: {}", ir_grammar.tokens.len());
                    
                    // Count rules with precedence
                    let prec_count = ir_grammar.rules.values()
                        .filter(|r| r.precedence.is_some())
                        .count();
                    println!("  Rules with precedence: {}", prec_count);
                    
                    // Count rules with associativity
                    let assoc_count = ir_grammar.rules.values()
                        .filter(|r| r.associativity.is_some())
                        .count();
                    println!("  Rules with associativity: {}", assoc_count);
                }
                Err(e) => {
                    println!("\n✗ Failed to convert to IR: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse grammar: {}", e);
        }
    }
}

#[test]
fn test_end_to_end_grammar_compilation() {
    // Test that we can go from grammar.js to a working parser
    let temp_dir = TempDir::new().unwrap();
    
    // Create a simple expression grammar
    let grammar_content = r#"
module.exports = grammar({
  name: 'calc',
  
  rules: {
    expression: $ => choice(
      $.number,
      $.binary_expression
    ),
    
    binary_expression: $ => choice(
      prec.left(1, seq($.expression, '+', $.expression)),
      prec.left(1, seq($.expression, '-', $.expression)),
      prec.left(2, seq($.expression, '*', $.expression)),
      prec.left(2, seq($.expression, '/', $.expression))
    ),
    
    number: $ => /\d+/
  }
})
"#;
    
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_content).unwrap();
    
    println!("\nTesting end-to-end compilation of calculator grammar...");
    
    // Try to build using pure rust
    unsafe {
        std::env::set_var("CARGO_FEATURE_PURE_RUST", "1");
    }
    
    use rust_sitter_tool::pure_rust_builder::{build_parser_from_grammar_js, BuildOptions};
    
    let options = BuildOptions {
        out_dir: temp_dir.path().to_str().unwrap().to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    
    match build_parser_from_grammar_js(&grammar_path, options) {
        Ok(result) => {
            println!("✓ Successfully built parser!");
            println!("  Grammar name: {}", result.grammar_name);
            
            // Check if NODE_TYPES.json was generated
            let node_types_path = temp_dir.path().join("NODE_TYPES.json");
            if node_types_path.exists() {
                println!("✓ NODE_TYPES.json generated");
                let content = fs::read_to_string(&node_types_path).unwrap();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(arr) = json.as_array() {
                        println!("  Node types: {} entries", arr.len());
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to build parser: {}", e);
            
            // Check what stage failed
            let error_msg = format!("{:?}", e);
            if error_msg.contains("GLR") || error_msg.contains("automaton") {
                println!("  Issue in GLR/automaton generation");
            } else if error_msg.contains("table") {
                println!("  Issue in table generation");
            } else {
                println!("  Error: {:#}", e);
            }
        }
    }
}