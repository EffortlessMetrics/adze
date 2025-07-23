module.exports = grammar({
  name: 'mylang',

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => repeat($._definition),

    _definition: $ => choice(
      // TODO: add choices here
    ),

    // TODO: add other rules
  }
});
