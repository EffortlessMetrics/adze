use anyhow::Result;
use rust_sitter_tool::grammar_js::{GrammarJsParserV3, GrammarJsConverter};
use std::fs;

pub fn test_local_grammars() -> Result<()> {
    println!("Testing local grammar files...\n");
    
    // Test 1: Simple arithmetic grammar
    let arithmetic_grammar = r#"
module.exports = grammar({
  name: 'arithmetic',
  
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
});
"#;
    
    test_grammar_string("arithmetic", arithmetic_grammar)?;
    
    // Test 2: C-style grammar with precedence
    let c_style_grammar = r#"
module.exports = grammar({
  name: 'c_style',
  
  word: $ => $.identifier,
  
  rules: {
    translation_unit: $ => repeat($._external_declaration),
    
    _external_declaration: $ => choice(
      $.function_definition,
      $.declaration
    ),
    
    declaration: $ => seq(
      $._declaration_specifiers,
      optional($.init_declarator_list),
      ';'
    ),
    
    _declaration_specifiers: $ => repeat1(
      choice(
        $.type_specifier,
        $.storage_class_specifier
      )
    ),
    
    init_declarator_list: $ => commaSep1($.init_declarator),
    
    init_declarator: $ => seq(
      $.declarator,
      optional(seq('=', $.initializer))
    ),
    
    declarator: $ => choice(
      $.identifier,
      seq('*', $.declarator),
      seq($.declarator, '[', optional($.expression), ']'),
      seq($.declarator, '(', optional($.parameter_list), ')')
    ),
    
    function_definition: $ => seq(
      optional($._declaration_specifiers),
      $.declarator,
      $.compound_statement
    ),
    
    compound_statement: $ => seq(
      '{',
      repeat($._block_item),
      '}'
    ),
    
    _block_item: $ => choice(
      $.declaration,
      $.statement
    ),
    
    statement: $ => choice(
      $.expression_statement,
      $.compound_statement,
      $.if_statement
    ),
    
    expression_statement: $ => seq(
      optional($.expression),
      ';'
    ),
    
    if_statement: $ => seq(
      'if',
      '(',
      $.expression,
      ')',
      $.statement,
      optional(seq('else', $.statement))
    ),
    
    expression: $ => choice(
      $.identifier,
      $.number,
      $.binary_expression,
      $.assignment_expression
    ),
    
    binary_expression: $ => choice(
      prec.left(1, seq($.expression, '||', $.expression)),
      prec.left(2, seq($.expression, '&&', $.expression)),
      prec.left(3, seq($.expression, '==', $.expression)),
      prec.left(3, seq($.expression, '!=', $.expression)),
      prec.left(4, seq($.expression, '<', $.expression)),
      prec.left(4, seq($.expression, '>', $.expression)),
      prec.left(5, seq($.expression, '+', $.expression)),
      prec.left(5, seq($.expression, '-', $.expression)),
      prec.left(6, seq($.expression, '*', $.expression)),
      prec.left(6, seq($.expression, '/', $.expression))
    ),
    
    assignment_expression: $ => prec.right(seq(
      $.expression,
      '=',
      $.expression
    )),
    
    type_specifier: $ => choice(
      'void',
      'char',
      'int',
      'float',
      'double'
    ),
    
    storage_class_specifier: $ => choice(
      'static',
      'extern',
      'auto',
      'register'
    ),
    
    parameter_list: $ => commaSep1($.parameter_declaration),
    
    parameter_declaration: $ => seq(
      $._declaration_specifiers,
      optional($.declarator)
    ),
    
    initializer: $ => choice(
      $.expression,
      seq('{', commaSep($.initializer), optional(','), '}')
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    number: $ => /\d+/
  }
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

function commaSep(rule) {
  return optional(commaSep1(rule));
}
"#;
    
    test_grammar_string("c_style", c_style_grammar)?;
    
    // Test 3: Grammar with externals (should parse but not build)
    let python_style_grammar = r#"
module.exports = grammar({
  name: 'python_style',
  
  externals: $ => [
    $._newline,
    $._indent,
    $._dedent
  ],
  
  rules: {
    module: $ => repeat($.statement),
    
    statement: $ => choice(
      $.expression_statement,
      $.if_statement
    ),
    
    expression_statement: $ => seq(
      $.expression,
      $._newline
    ),
    
    if_statement: $ => seq(
      'if',
      $.expression,
      ':',
      $._newline,
      $._indent,
      repeat1($.statement),
      $._dedent
    ),
    
    expression: $ => choice(
      $.identifier,
      $.number
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    
    number: $ => /\d+/
  }
});
"#;
    
    test_grammar_string("python_style", python_style_grammar)?;
    
    println!("\nLocal grammar tests complete!");
    Ok(())
}

fn test_grammar_string(name: &str, content: &str) -> Result<()> {
    println!("Testing {} grammar...", name);
    
    // Parse
    let mut parser = GrammarJsParserV3::new(content.to_string());
    match parser.parse() {
        Ok(grammar_js) => {
            println!("  ✅ Parsed successfully");
            println!("    Rules: {}", grammar_js.rules.len());
            
            // Check features
            let mut features = vec![];
            if !grammar_js.externals.is_empty() {
                features.push(format!("externals({})", grammar_js.externals.len()));
            }
            if grammar_js.word.is_some() {
                features.push("word".to_string());
            }
            if !grammar_js.conflicts.is_empty() {
                features.push("conflicts".to_string());
            }
            if !features.is_empty() {
                println!("    Features: {}", features.join(", "));
            }
            
            // Convert to IR
            let converter = GrammarJsConverter::new(grammar_js);
            match converter.convert() {
                Ok(ir) => {
                    println!("  ✅ Converted to IR");
                    println!("    IR rules: {}, tokens: {}", ir.rules.len(), ir.tokens.len());
                    
                    // For grammars with externals, we expect build to fail
                    if !ir.externals.is_empty() {
                        println!("  ⚠️  Has external scanner - build would fail at runtime");
                    }
                }
                Err(e) => {
                    println!("  ❌ Convert failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parse failed: {}", e);
        }
    }
    
    println!();
    Ok(())
}