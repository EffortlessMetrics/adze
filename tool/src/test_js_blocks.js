// Test file to understand JavaScript function blocks in grammar.js

// Example 1: Simple function block
rule1: $ => {
  const items = ['a', 'b', 'c'];
  return choice(...items.map(item => $.identifier));
},

// Example 2: Helper function usage
rule2: $ => {
  const commaSep = (rule) => optional(seq(rule, repeat(seq(',', rule))));
  return commaSep($.expression);
},

// Example 3: Complex logic
rule3: $ => {
  const precedences = [
    ['member', 'call'],
    ['unary', 'binary'],
  ];
  
  const table = precedences.map(([name, assoc]) => {
    return [name, prec[assoc]];
  });
  
  return choice(
    ...table.map(([name, fn]) => fn(name, $.expression))
  );
}