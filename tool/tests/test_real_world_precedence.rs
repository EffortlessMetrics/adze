use rust_sitter_tool::grammar_js::{GrammarJsConverter, GrammarJsParserV3};
use rust_sitter_tool::pure_rust_builder::{build_parser_from_grammar_js, BuildOptions};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_arithmetic_grammar_with_precedence() {
    // Create a grammar similar to what real grammars use
    let grammar_content = r#"
module.exports = grammar({
  name: 'arithmetic',
  
  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => seq(
      $.expression,
      ';'
    ),
    
    expression: $ => choice(
      $.number,
      $.identifier,
      $.binary_expression,
      $.unary_expression,
      $.parenthesized_expression
    ),
    
    binary_expression: $ => {
      const table = [
        [prec.left, '+', 1],
        [prec.left, '-', 1],
        [prec.left, '*', 2],
        [prec.left, '/', 2],
        [prec.right, '^', 3],
      ];

      return choice(...table.map(([fn, operator, precedence]) => fn(precedence,
        seq(
          field('left', $.expression),
          field('operator', operator),
          field('right', $.expression)
        )
      )));
    },
    
    unary_expression: $ => prec(4, choice(
      seq('-', $.expression),
      seq('+', $.expression)
    )),
    
    parenthesized_expression: $ => seq('(', $.expression, ')'),
    
    number: $ => /\d+(\.\d+)?/,
    
    identifier: $ => /[a-zA-Z_]\w*/
  }
});
"#;

    // Write to temp file
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_content).unwrap();

    // Try to parse with v3 parser
    println!("\nTesting arithmetic grammar with precedence...");
    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar_js) => {
            println!("✓ Successfully parsed grammar!");
            println!("  Rules: {:?}", grammar_js.rules.keys().collect::<Vec<_>>());

            // Try to convert to IR
            let converter = GrammarJsConverter::new(grammar_js.clone());
            match converter.convert() {
                Ok(ir_grammar) => {
                    println!("✓ Successfully converted to IR!");
                    println!(
                        "  IR has {} rules, {} tokens",
                        ir_grammar.rules.len(),
                        ir_grammar.tokens.len()
                    );

                    // Try to build the parser
                    let options = BuildOptions {
                        out_dir: temp_dir.path().to_str().unwrap().to_string(),
                        emit_artifacts: false,
                        compress_tables: true,
                    };

                    match build_parser_from_grammar_js(&grammar_path, options) {
                        Ok(_) => {
                            println!("✓ Successfully built parser!");
                        }
                        Err(e) => {
                            println!("✗ Failed to build parser: {:#}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to convert to IR: {:#}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse grammar: {:#}", e);
        }
    }
}

#[test]
fn test_c_style_precedence() {
    // Test C-style operator precedence which is common in many grammars
    let grammar_content = r#"
module.exports = grammar({
  name: 'c_style',
  
  rules: {
    expression: $ => choice(
      $.primary_expression,
      $.assignment_expression,
      $.conditional_expression,
      $.logical_or_expression,
      $.logical_and_expression,
      $.equality_expression,
      $.relational_expression,
      $.additive_expression,
      $.multiplicative_expression
    ),
    
    primary_expression: $ => choice(
      $.identifier,
      $.number,
      seq('(', $.expression, ')')
    ),
    
    assignment_expression: $ => prec.right(1, seq(
      field('left', $.expression),
      '=',
      field('right', $.expression)
    )),
    
    conditional_expression: $ => prec.right(2, seq(
      field('condition', $.expression),
      '?',
      field('consequence', $.expression),
      ':',
      field('alternative', $.expression)
    )),
    
    logical_or_expression: $ => prec.left(3, seq(
      field('left', $.expression),
      '||',
      field('right', $.expression)
    )),
    
    logical_and_expression: $ => prec.left(4, seq(
      field('left', $.expression),
      '&&',
      field('right', $.expression)
    )),
    
    equality_expression: $ => prec.left(5, seq(
      field('left', $.expression),
      choice('==', '!='),
      field('right', $.expression)
    )),
    
    relational_expression: $ => prec.left(6, seq(
      field('left', $.expression),
      choice('<', '>', '<=', '>='),
      field('right', $.expression)
    )),
    
    additive_expression: $ => prec.left(7, seq(
      field('left', $.expression),
      choice('+', '-'),
      field('right', $.expression)
    )),
    
    multiplicative_expression: $ => prec.left(8, seq(
      field('left', $.expression),
      choice('*', '/', '%'),
      field('right', $.expression)
    )),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    number: $ => /\d+/
  }
});
"#;

    // Write to temp file
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("grammar.js");
    fs::write(&grammar_path, grammar_content).unwrap();

    // Test parsing
    println!("\nTesting C-style precedence grammar...");
    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar_js) => {
            println!("✓ Successfully parsed C-style grammar!");
            println!("  Rules: {} total", grammar_js.rules.len());

            // Try to convert to IR
            let converter = GrammarJsConverter::new(grammar_js.clone());
            match converter.convert() {
                Ok(_ir_grammar) => {
                    println!("✓ Successfully converted to IR!");

                    // Try to build the parser
                    let options = BuildOptions {
                        out_dir: temp_dir.path().to_str().unwrap().to_string(),
                        emit_artifacts: false,
                        compress_tables: true,
                    };

                    match build_parser_from_grammar_js(&grammar_path, options) {
                        Ok(_) => {
                            println!("✓ Successfully built C-style parser!");
                        }
                        Err(e) => {
                            println!("✗ Failed to build parser: {:#}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to convert to IR: {:#}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse grammar: {:#}", e);
        }
    }
}
