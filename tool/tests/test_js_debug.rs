use adze_tool::grammar_js::parse_grammar_js_v2;

#[test]
fn test_javascript_debug() {
    // Test just the problematic part
    let grammar = r#"
module.exports = grammar({
  name: 'javascript',

  extras: $ => [
    /\s/,
    $.comment
  ],

  inline: $ => [
    $.statement,
    $.expression
  ],

  precedences: $ => [
    ['member', 'call', 'unary', 'binary'],
    ['assign', 'ternary']
  ],

  conflicts: $ => [
    [$.primary_expression, $.pattern],
    [$.primary_expression, $.rest_pattern]
  ],

  word: $ => $.identifier,

  rules: {
    program: $ => repeat($.statement),

    statement: $ => choice(
      $.expression_statement,
      $.declaration
    ),

    expression_statement: $ => seq(
      $.expression,
      ';'
    ),

    declaration: $ => choice(
      $.variable_declaration,
      $.function_declaration
    ),

    variable_declaration: $ => seq(
      choice('let', 'const', 'var'),
      $.identifier,
      optional(seq('=', $.expression)),
      ';'
    ),

    function_declaration: $ => seq(
      'function',
      field('name', $.identifier),
      '(',
      optional($.parameters),
      ')',
      $.block_statement
    ),

    parameters: $ => seq(
      $.identifier,
      repeat(seq(',', $.identifier))
    ),

    return_statement: $ => seq(
      'return',
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

    while_statement: $ => seq(
      'while',
      '(',
      $.expression,
      ')',
      $.statement
    ),

    for_statement: $ => seq(
      'for',
      '(',
      optional(choice($.variable_declaration, $.expression)),
      ';',
      optional($.expression),
      ';',
      optional($.expression),
      ')',
      $.statement
    ),

    block_statement: $ => seq(
      '{',
      repeat($.statement),
      '}'
    ),

    expression: $ => choice(
      $.assignment_expression,
      $.binary_expression,
      $.unary_expression,
      $.call_expression,
      $.member_expression,
      $.primary_expression
    ),

    assignment_expression: $ => prec.right(1, seq(
      field('left', $.expression),
      '=',
      field('right', $.expression)
    )),

    binary_expression: $ => choice(
      prec.left(1, seq(
        field('left', $.expression),
        '&&',
        field('right', $.expression)
      )),
      prec.left(1, seq(
        field('left', $.expression),
        '||',
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        '+',
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        '-',
        field('right', $.expression)
      ))
    ),

    identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/
  }
});
"#;

    eprintln!("Grammar length: {}", grammar.len());
    eprintln!("Grammar content:\n{}", grammar);

    let result = parse_grammar_js_v2(grammar);
    match result {
        Ok(parsed) => {
            eprintln!("Successfully parsed!");
            eprintln!("Name: {}", parsed.name);
            eprintln!("Inline rules: {:?}", parsed.inline);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            panic!("Failed to parse: {}", e);
        }
    }
}
