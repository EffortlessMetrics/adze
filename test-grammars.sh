#!/bin/bash
# Test script for rust-sitter grammars

set -e

echo "Building rust-sitter grammars..."

# Build all grammars
cargo build -p rust-sitter-javascript
cargo build -p rust-sitter-python
cargo build -p rust-sitter-go

echo "Running grammar tests..."

# Test JavaScript
echo "Testing JavaScript grammar..."
cargo test -p rust-sitter-javascript

# Test Python
echo "Testing Python grammar..."
cargo test -p rust-sitter-python

# Test Go
echo "Testing Go grammar..."
cargo test -p rust-sitter-go

echo "All grammar tests passed!"

# Run compatibility tests if testing framework is built
if cargo build -p rust-sitter-testing 2>/dev/null; then
    echo "Running compatibility tests..."
    
    # Create test directories
    mkdir -p grammars/javascript/test/corpus
    mkdir -p grammars/python/test/corpus
    mkdir -p grammars/go/test/corpus
    
    # Create sample test files
    cat > grammars/javascript/test/corpus/basic.txt << 'EOF'
================
Variable Declaration
================

let x = 42;
const y = "hello";
var z = true;

---

(program
  (variable_declaration
    (variable_declarator (identifier) (number)))
  (variable_declaration
    (variable_declarator (identifier) (string)))
  (variable_declaration
    (variable_declarator (identifier) (true))))
EOF

    cat > grammars/python/test/corpus/basic.txt << 'EOF'
================
Function Definition
================

def hello(name):
    print(f"Hello, {name}!")
    return name

---

(module
  (function_definition
    (identifier)
    (parameters (identifier))
    (block
      (expression_statement
        (call
          (identifier)
          (argument_list (f_string))))
      (return_statement (identifier)))))
EOF

    cat > grammars/go/test/corpus/basic.txt << 'EOF'
================
Package and Function
================

package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}

---

(source_file
  (package_clause (package_identifier))
  (import_declaration (import_spec (interpreted_string_literal)))
  (function_declaration
    (identifier)
    (parameter_list)
    (block
      (expression_statement
        (call_expression
          (selector_expression
            (identifier)
            (field_identifier))
          (argument_list (interpreted_string_literal)))))))
EOF

    echo "Test corpus files created."
else
    echo "Testing framework not available, skipping compatibility tests."
fi

echo "Grammar development complete!"