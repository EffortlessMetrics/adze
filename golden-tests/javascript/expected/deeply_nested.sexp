(program [0, 0] - [22, 0]
  (function_declaration [0, 0] - [21, 1]
    name: (identifier [0, 9] - [0, 10])
    parameters: (formal_parameters [0, 10] - [0, 12])
    body: (statement_block [0, 13] - [21, 1]
      (function_declaration [1, 2] - [20, 3]
        name: (identifier [1, 11] - [1, 12])
        parameters: (formal_parameters [1, 12] - [1, 14])
        body: (statement_block [1, 15] - [20, 3]
          (function_declaration [2, 4] - [19, 5]
            name: (identifier [2, 13] - [2, 14])
            parameters: (formal_parameters [2, 14] - [2, 16])
            body: (statement_block [2, 17] - [19, 5]
              (function_declaration [3, 6] - [18, 7]
                name: (identifier [3, 15] - [3, 16])
                parameters: (formal_parameters [3, 16] - [3, 18])
                body: (statement_block [3, 19] - [18, 7]
                  (function_declaration [4, 8] - [17, 9]
                    name: (identifier [4, 17] - [4, 18])
                    parameters: (formal_parameters [4, 18] - [4, 20])
                    body: (statement_block [4, 21] - [17, 9]
                      (if_statement [5, 10] - [16, 11]
                        condition: (parenthesized_expression [5, 13] - [5, 19]
                          (true [5, 14] - [5, 18]))
                        consequence: (statement_block [5, 20] - [16, 11]
                          (if_statement [6, 12] - [15, 13]
                            condition: (parenthesized_expression [6, 15] - [6, 21]
                              (true [6, 16] - [6, 20]))
                            consequence: (statement_block [6, 22] - [15, 13]
                              (if_statement [7, 14] - [14, 15]
                                condition: (parenthesized_expression [7, 17] - [7, 23]
                                  (true [7, 18] - [7, 22]))
                                consequence: (statement_block [7, 24] - [14, 15]
                                  (for_statement [8, 16] - [13, 17]
                                    initializer: (lexical_declaration [8, 21] - [8, 31]
                                      (variable_declarator [8, 25] - [8, 30]
                                        name: (identifier [8, 25] - [8, 26])
                                        value: (number [8, 29] - [8, 30])))
                                    condition: (binary_expression [8, 32] - [8, 38]
                                      left: (identifier [8, 32] - [8, 33])
                                      right: (number [8, 36] - [8, 38]))
                                    increment: (update_expression [8, 40] - [8, 43]
                                      argument: (identifier [8, 40] - [8, 41]))
                                    body: (statement_block [8, 45] - [13, 17]
                                      (for_statement [9, 18] - [12, 19]
                                        initializer: (lexical_declaration [9, 23] - [9, 33]
                                          (variable_declarator [9, 27] - [9, 32]
                                            name: (identifier [9, 27] - [9, 28])
                                            value: (number [9, 31] - [9, 32])))
                                        condition: (binary_expression [9, 34] - [9, 40]
                                          left: (identifier [9, 34] - [9, 35])
                                          right: (number [9, 38] - [9, 40]))
                                        increment: (update_expression [9, 42] - [9, 45]
                                          argument: (identifier [9, 42] - [9, 43]))
                                        body: (statement_block [9, 47] - [12, 19]
                                          (lexical_declaration [10, 20] - [10, 44]
                                            (variable_declarator [10, 24] - [10, 43]
                                              name: (identifier [10, 24] - [10, 25])
                                              value: (array [10, 28] - [10, 43]
                                                (array [10, 29] - [10, 42]
                                                  (array [10, 30] - [10, 41]
                                                    (array [10, 31] - [10, 40]
                                                      (array [10, 32] - [10, 39]
                                                        (binary_expression [10, 33] - [10, 38]
                                                          left: (identifier [10, 33] - [10, 34])
                                                          right: (identifier [10, 37] - [10, 38])))))))))
                                          (return_statement [11, 20] - [11, 29]
                                            (identifier [11, 27] - [11, 28])))))))))))))))))))))))
