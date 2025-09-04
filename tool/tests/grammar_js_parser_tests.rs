//! Tests for grammar.js parser

use rust_sitter_tool::grammar_js::{parse_grammar_js_v2, Rule};

#[test]
fn test_javascript_like_grammar() {
    let grammar_js = r#"
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
    [6, 5, 4, 3],
    [1, 0]
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
      $.declaration,
      $.return_statement,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.block_statement
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

    if_statement: $ => prec.right(0, seq(
      'if',
      '(',
      $.expression,
      ')',
      $.statement,
      optional(seq('else', $.statement))
    )),

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
      optional($.expression),
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
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '<',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '>',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '<=',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '>=',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '==',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '!=',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '===',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '!==',
        field('right', $.expression)
      ))
    ),

    unary_expression: $ => prec.left(4, choice(
      seq('-', $.expression),
      seq('+', $.expression),
      seq('!', $.expression),
      seq('~', $.expression),
      seq('typeof', $.expression),
      seq('void', $.expression),
      seq('delete', $.expression)
    )),

    call_expression: $ => prec(5, seq(
      field('function', $.expression),
      '(',
      optional($.arguments),
      ')'
    )),

    arguments: $ => seq(
      $.expression,
      repeat(seq(',', $.expression))
    ),

    member_expression: $ => prec(6, seq(
      field('object', $.expression),
      '.',
      field('property', $.identifier)
    )),

    primary_expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
      $.template_string,
      $.true,
      $.false,
      $.null,
      $.undefined,
      $.array,
      $.object,
      seq('(', $.expression, ')')
    ),

    identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,

    number: $ => /\d+(\.\d+)?/,

    string: $ => choice(
      /"[^"]*"/,
      /'[^']*'/
    ),

    template_string: $ => /`[^`]*`/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null',
    undefined: $ => 'undefined',

    array: $ => seq(
      '[',
      optional(seq(
        $.expression,
        repeat(seq(',', $.expression))
      )),
      ']'
    ),

    object: $ => seq(
      '{',
      optional(seq(
        $.property,
        repeat(seq(',', $.property))
      )),
      '}'
    ),

    property: $ => seq(
      field('key', choice($.identifier, $.string)),
      ':',
      field('value', $.expression)
    ),

    pattern: $ => choice(
      $.identifier,
      $.array_pattern,
      $.object_pattern
    ),

    array_pattern: $ => seq(
      '[',
      optional(seq(
        $.pattern,
        repeat(seq(',', $.pattern))
      )),
      ']'
    ),

    object_pattern: $ => seq(
      '{',
      optional(seq(
        $.pattern_property,
        repeat(seq(',', $.pattern_property))
      )),
      '}'
    ),

    pattern_property: $ => seq(
      field('key', $.identifier),
      optional(seq(':', field('value', $.pattern)))
    ),

    rest_pattern: $ => seq('...', $.identifier),

    comment: $ => choice(
      /\/\/[^\n]*/,
      /\/\*[^*]*\*\//
    )
  }
});
    "#;

    // Debug check
    eprintln!(
        "Grammar contains $.statement: {}",
        grammar_js.contains("$.statement")
    );
    eprintln!(
        "Grammar contains $.state': {}",
        grammar_js.contains("$.state'")
    );

    let result = parse_grammar_js_v2(grammar_js);

    match result {
        Ok(grammar) => {
            assert_eq!(grammar.name, "javascript");
            assert_eq!(grammar.word, Some("identifier".to_string()));
            assert!(!grammar.extras.is_empty());
            assert_eq!(grammar.inline.len(), 2);
            assert_eq!(grammar.conflicts.len(), 2); // Both conflicts are valid since all referenced rules exist

            // Verify the conflicts are correctly parsed
            assert_eq!(grammar.conflicts[0], vec!["primary_expression", "pattern"]);
            assert_eq!(
                grammar.conflicts[1],
                vec!["primary_expression", "rest_pattern"]
            );
            assert!(!grammar.rules.is_empty());

            // Check specific rules
            assert!(grammar.rules.contains_key("program"));
            assert!(grammar.rules.contains_key("expression"));
            assert!(grammar.rules.contains_key("identifier"));

            // Check field rules
            if let Some(Rule::Seq { members }) = grammar.rules.get("function_declaration") {
                let has_field = members.iter().any(|m| matches!(m, Rule::Field { .. }));
                assert!(has_field, "function_declaration should have field");
            }

            println!("Successfully parsed JavaScript-like grammar!");
        }
        Err(e) => {
            panic!("Failed to parse grammar: {}", e);
        }
    }
}

#[test]
fn test_simple_arithmetic_grammar() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'arithmetic',

  rules: {
    expression: $ => choice(
      $.number,
      $.binary_expression,
      seq('(', $.expression, ')')
    ),

    binary_expression: $ => choice(
      prec.left(2, seq($.expression, '*', $.expression)),
      prec.left(2, seq($.expression, '/', $.expression)),
      prec.left(1, seq($.expression, '+', $.expression)),
      prec.left(1, seq($.expression, '-', $.expression))
    ),

    number: $ => /\d+/
  }
});
    "#;

    let grammar = parse_grammar_js_v2(grammar_js).unwrap();
    assert_eq!(grammar.name, "arithmetic");
    assert_eq!(grammar.rules.len(), 3);
}

#[test]
fn test_json_grammar() {
    let grammar_js = r#"
module.exports = grammar({
  name: 'json',

  extras: $ => [
    /\s/
  ],

  rules: {
    document: $ => $._value,

    _value: $ => choice(
      $.object,
      $.array,
      $.number,
      $.string,
      $.true,
      $.false,
      $.null
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
      field('value', $._value)
    ),

    array: $ => seq(
      '[',
      optional(seq(
        $._value,
        repeat(seq(',', $._value))
      )),
      ']'
    ),

    string: $ => /("[^"]*")/,

    number: $ => /-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null'
  }
});
    "#;

    let grammar = parse_grammar_js_v2(grammar_js).unwrap();
    assert_eq!(grammar.name, "json");
    assert!(grammar.rules.contains_key("document"));
    assert!(grammar.rules.contains_key("_value"));
}
