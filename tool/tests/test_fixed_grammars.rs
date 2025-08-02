use rust_sitter_tool::grammar_js::parse_grammar_js_v2;

#[test]
fn test_simple_json_grammar() {
    let grammar = r#"
module.exports = grammar({
  name: 'json',

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

    number: $ => /-?\d+(\.\d+)?/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null'
  }
});
"#;

    let result = parse_grammar_js_v2(grammar);
    assert!(
        result.is_ok(),
        "Failed to parse JSON grammar: {:?}",
        result.err()
    );

    let parsed = result.unwrap();
    assert_eq!(parsed.name, "json");
    assert!(parsed.rules.contains_key("document"));
    assert!(parsed.rules.contains_key("_value"));

    println!("Successfully parsed simple JSON grammar!");
}

#[test]
fn test_simple_javascript_grammar() {
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

  word: $ => $.identifier,

  rules: {
    program: $ => repeat($.statement),

    statement: $ => choice(
      $.expression_statement,
      $.block_statement
    ),

    expression_statement: $ => seq(
      $.expression,
      ';'
    ),

    block_statement: $ => seq(
      '{',
      repeat($.statement),
      '}'
    ),

    expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
      $.binary_expression,
      $.assignment_expression
    ),

    binary_expression: $ => choice(
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
      prec.left(3, seq(
        field('left', $.expression),
        '*',
        field('right', $.expression)
      )),
      prec.left(3, seq(
        field('left', $.expression),
        '/',
        field('right', $.expression)
      ))
    ),

    assignment_expression: $ => prec.right(1, seq(
      field('left', $.expression),
      '=',
      field('right', $.expression)
    )),

    identifier: $ => /[a-zA-Z_]\w*/,
    number: $ => /\d+/,
    string: $ => /"[^"]*"/,
    comment: $ => /\/\/[^\n]*/
  }
});
"#;

    let result = parse_grammar_js_v2(grammar);
    assert!(
        result.is_ok(),
        "Failed to parse JavaScript grammar: {:?}",
        result.err()
    );

    let parsed = result.unwrap();
    assert_eq!(parsed.name, "javascript");
    assert_eq!(parsed.word, Some("identifier".to_string()));
    assert!(!parsed.extras.is_empty());
    assert_eq!(parsed.inline.len(), 2);

    // Check specific rules
    assert!(parsed.rules.contains_key("program"));
    assert!(parsed.rules.contains_key("expression"));
    assert!(parsed.rules.contains_key("identifier"));

    println!("Successfully parsed simple JavaScript grammar!");
}
