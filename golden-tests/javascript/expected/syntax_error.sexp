(program [0, 0] - [3, 0]
  (lexical_declaration [0, 0] - [0, 10]
    (variable_declarator [0, 4] - [0, 9]
      name: (identifier [0, 4] - [0, 5])
      value: (number [0, 8] - [0, 9])))
  (ERROR [1, 0] - [2, 9]
    (identifier [1, 9] - [1, 12])
    (object_assignment_pattern [2, 0] - [2, 9]
      left: (shorthand_property_identifier_pattern [2, 0] - [2, 3])
      (ERROR [2, 4] - [2, 5]
        (identifier [2, 4] - [2, 5]))
      right: (number [2, 8] - [2, 9])))
  (empty_statement [2, 9] - [2, 10]))
/home/steven/code/rust-sitter/golden-tests/javascript/fixtures/syntax_error.js	Parse:    0.14 ms	   273 bytes/ms	(ERROR [1, 0] - [2, 9])
