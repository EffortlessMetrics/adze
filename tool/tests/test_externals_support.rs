use rust_sitter_tool::grammar_js::{GrammarJsConverter, GrammarJsParserV3};
use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_basic_externals() {
    // Test basic external scanner support
    let grammar_content = r#"
module.exports = grammar({
  name: 'test_externals',
  
  externals: $ => [
    $.string_content,
    $.comment
  ],
  
  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => choice(
      $.string,
      $.comment,
      $.expression
    ),
    
    string: $ => seq(
      '"',
      optional($.string_content),
      '"'
    ),
    
    expression: $ => /[a-z]+/
  }
});
"#;

    println!("\nTesting grammar with externals...");
    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar_js) => {
            println!("✓ Successfully parsed grammar!");
            println!("  Rules: {:?}", grammar_js.rules.keys().collect::<Vec<_>>());
            println!("  Externals: {} defined", grammar_js.externals.len());

            // Try to convert to IR
            let converter = GrammarJsConverter::new(grammar_js.clone());
            match converter.convert() {
                Ok(ir_grammar) => {
                    println!("✓ Successfully converted to IR!");
                    println!(
                        "  IR has {} rules, {} tokens, {} externals",
                        ir_grammar.rules.len(),
                        ir_grammar.tokens.len(),
                        ir_grammar.externals.len()
                    );
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
fn test_python_style_externals() {
    // Test Python-style externals (commonly used pattern)
    let grammar_content = r#"
module.exports = grammar({
  name: 'python_style',
  
  externals: $ => [
    $._newline,
    $._indent,
    $._dedent,
    $.string_content,
    $.comment
  ],
  
  extras: $ => [
    /\s/,
    $.comment
  ],
  
  rules: {
    module: $ => repeat($._statement),
    
    _statement: $ => choice(
      $.expression_statement,
      $.if_statement,
      $.function_definition
    ),
    
    expression_statement: $ => seq(
      $._expression,
      $._newline
    ),
    
    if_statement: $ => seq(
      'if',
      $._expression,
      ':',
      $._newline,
      $._indent,
      repeat1($._statement),
      $._dedent
    ),
    
    function_definition: $ => seq(
      'def',
      $.identifier,
      '(',
      ')',
      ':',
      $._newline,
      $._indent,
      repeat1($._statement),
      $._dedent
    ),
    
    _expression: $ => choice(
      $.identifier,
      $.string,
      $.number
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    string: $ => seq(
      choice('"', "'"),
      optional($.string_content),
      choice('"', "'")
    ),
    
    number: $ => /\d+/
  }
});
"#;

    println!("\nTesting Python-style grammar with indent/dedent externals...");
    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar_js) => {
            println!("✓ Successfully parsed grammar!");
            println!("  Externals: {:?}", grammar_js.externals.len());

            // Convert to IR
            let converter = GrammarJsConverter::new(grammar_js.clone());
            match converter.convert() {
                Ok(ir_grammar) => {
                    println!("✓ Successfully converted to IR!");
                    println!("  External symbols: {}", ir_grammar.externals.len());
                    for external in &ir_grammar.externals {
                        println!("    - External symbol ID: {}", external.symbol_id.0);
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
fn test_markdown_style_externals() {
    // Test Markdown-style externals (complex external scanner)
    let grammar_content = r#"
module.exports = grammar({
  name: 'markdown_style',
  
  externals: $ => [
    $._block_continuation,
    $._block_close,
    $._fenced_code_block_delimiter,
    $._html_block_end,
    $.line_break
  ],
  
  precedences: $ => [
    ['inline', 'block']
  ],
  
  rules: {
    document: $ => repeat($.block),
    
    block: $ => choice(
      $.paragraph,
      $.heading,
      $.code_block,
      $.list
    ),
    
    paragraph: $ => prec('block', seq(
      repeat1($.inline),
      $._block_close
    )),
    
    heading: $ => seq(
      /#{1,6}/,
      /\s+/,
      repeat($.inline),
      $._block_close
    ),
    
    code_block: $ => seq(
      $._fenced_code_block_delimiter,
      optional($.language),
      repeat($.code_line),
      $._fenced_code_block_delimiter,
      $._block_close
    ),
    
    language: $ => /\w+/,
    
    code_line: $ => /[^\n]+\n/,
    
    list: $ => repeat1($.list_item),
    
    list_item: $ => seq(
      choice('-', '*', '+'),
      /\s+/,
      repeat($.inline),
      $._block_close
    ),
    
    inline: $ => prec('inline', choice(
      $.text,
      $.emphasis,
      $.code_span
    )),
    
    text: $ => /[^\n*`]+/,
    
    emphasis: $ => seq(
      '*',
      repeat1(choice($.text, $.code_span)),
      '*'
    ),
    
    code_span: $ => seq(
      '`',
      /[^`\n]+/,
      '`'
    )
  }
});
"#;

    println!("\nTesting Markdown-style grammar with complex externals...");
    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar_js) => {
            println!("✓ Successfully parsed grammar!");
            println!("  Has {} externals", grammar_js.externals.len());
            println!("  Has {} precedence groups", grammar_js.precedences.len());

            // Try to convert
            let converter = GrammarJsConverter::new(grammar_js.clone());
            match converter.convert() {
                Ok(_ir_grammar) => {
                    println!("✓ Successfully converted to IR!");

                    // Try to build (will likely fail without scanner implementation)
                    let temp_dir = TempDir::new().unwrap();
                    let grammar_path = temp_dir.path().join("grammar.js");
                    fs::write(&grammar_path, grammar_content).unwrap();

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
                            if e.to_string().contains("external scanner") {
                                println!("  (This is expected - external scanner not implemented)");
                            }
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
