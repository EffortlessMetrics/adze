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

    string: $ => seq(
      '"',
      repeat(choice(
        /[^"\\]/,
        /\\["\\\/bfnrt]/,
        /\\u[0-9a-fA-F]{4}/
      )),
      '"'
    ),

    number: $ => /\-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null'
  }
});