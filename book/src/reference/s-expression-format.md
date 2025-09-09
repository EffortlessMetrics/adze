# S-Expression Format Reference

rust-sitter uses S-expression format for parse tree serialization, maintaining compatibility with Tree-sitter's standard output format. This format provides a human-readable representation of parse trees used in golden tests, debugging, and tree analysis.

## Format Overview

S-expressions represent parse trees as nested symbolic expressions where:
- **Parentheses** indicate node boundaries and hierarchy
- **Node types** specify the grammatical category
- **Source text** appears in quoted strings for leaf nodes
- **Indentation** shows tree structure visually

### Basic Structure

```lisp
(node_type
  (child_node "text")
  (another_child
    (nested_node "more_text")))
```

## Node Representation

### Named Nodes

Named nodes represent grammatical constructs defined in the grammar:

```lisp
(module
  (function_definition
    (identifier) "hello"
    (parameters)
    (block
      (return_statement
        (string) "\"world\""))))
```

**Characteristics:**
- Enclosed in parentheses: `(node_type ...)`
- Node type appears first: `function_definition`, `identifier`, etc.
- May contain child nodes or text content
- Correspond to grammar rules and AST nodes

### Anonymous Nodes

Anonymous nodes represent literal tokens and punctuation:

```lisp
(binary_expression
  (number) "1"
  "+"
  (number) "2")
```

**Characteristics:**  
- Appear as quoted strings: `"+"`, `"def"`, `"("`
- No surrounding parentheses
- Represent exact source text
- Include keywords, operators, and punctuation

### Text Content

Leaf nodes include their source text content:

```lisp
(identifier) "variable_name"
(number) "42"
(string) "\"hello world\""
```

**Text Escaping:**
- Double quotes: `"\"quoted\""`
- Backslashes: `"\\path\\to\\file"`
- Newlines: `"line1\nline2"`
- Control characters: `"\t\r\n"`
- Unicode: `"\u{1F600}"` (for 😀)

## Tree Structure Examples

### Python Function

**Source Code:**
```python
def greet(name):
    return f"Hello, {name}!"
```

**S-Expression:**
```lisp
(module
  (function_definition
    "def"
    (identifier) "greet"
    (parameters
      "("
      (identifier) "name"
      ")")
    ":"
    (block
      (return_statement
        "return"
        (f_string
          "f"
          "\""
          "Hello, "
          (interpolation
            "{"
            (identifier) "name"
            "}")
          "!\""))))
```

### JavaScript Expression

**Source Code:**
```javascript
const result = add(x, y);
```

**S-Expression:**
```lisp
(program
  (variable_declaration
    "const"
    (variable_declarator
      (identifier) "result"
      "="
      (call_expression
        (identifier) "add"
        (arguments
          "("
          (identifier) "x"
          ","
          (identifier) "y"
          ")")))))
```

### Arithmetic Expression

**Source Code:**
```
1 + 2 * 3
```

**S-Expression:**
```lisp
(expression
  (binary_expression
    (number) "1"
    "+"
    (binary_expression
      (number) "2"
      "*"
      (number) "3")))
```

## Formatting Rules

### Indentation

S-expressions use consistent 2-space indentation:

```lisp
(module
  (class_definition
    "class"
    (identifier) "Example"
    ":"
    (block
      (expression_statement
        (call
          (identifier) "print"
          (argument_list
            "("
            (string) "\"Hello\""
            ")"))))))
```

### Line Breaks

- **Named nodes**: New line after opening parenthesis if children exist
- **Leaf nodes**: Single line format
- **Closing parentheses**: Aligned with opening node

**Multi-line Example:**
```lisp
(function_definition
  "def"
  (identifier) "process"
  (parameters
    "("
    (identifier) "data"
    ")")
  ":"
  (block
    (return_statement
      "return"
      (call
        (identifier) "transform"
        (argument_list
          "("
          (identifier) "data"
          ")")))))
```

**Single-line Example:**
```lisp
(identifier) "simple_name"
```

### Whitespace Handling

Source code whitespace is preserved in anonymous nodes:

**Source:**
```python
x    =     42
```

**S-Expression:**
```lisp
(assignment
  (identifier) "x"
  "    =     "
  (number) "42")
```

## Field Information

Standard S-expressions don't include field names. For debugging with fields, use extended format:

**Standard Format:**
```lisp
(binary_expression
  (identifier) "a"
  "+"
  (identifier) "b")
```

**Extended Format (debugging):**
```lisp
(binary_expression
  left: (identifier) "a"
  operator: "+"
  right: (identifier) "b")
```

## Error Nodes

Parse errors appear as special error nodes:

**Source (with syntax error):**
```python
def foo(
    pass
```

**S-Expression:**
```lisp
(module
  (function_definition
    "def"
    (identifier) "foo"
    "("
    (ERROR)
    (block
      "pass")))
```

**Error Node Characteristics:**
- Node type: `ERROR` or `MISSING`
- May contain partial content
- Indicates parser recovery points
- Shows where parsing failed

## Generating S-Expressions

### From rust-sitter

```rust
use rust_sitter::tree_to_sexp;

let source = "def hello(): pass";
let tree = python_parser.parse(source)?;
let sexp = tree_to_sexp(&tree, source);
println!("{}", sexp);
```

### From Tree-sitter CLI

```bash
echo "def hello(): pass" | tree-sitter parse --quiet
```

### From Golden Tests

```bash
cd golden-tests
./generate_references.sh
cat python/expected/simple_program.sexp
```

## Practical Applications

### Golden Test Validation

S-expressions enable precise compatibility testing:

```rust
#[test]
fn test_python_parsing() {
    let source = load_fixture("example.py");
    let actual_sexp = parse_with_rust_sitter(source)?;
    let expected_sexp = load_expected("example.sexp")?;
    assert_eq!(actual_sexp, expected_sexp);
}
```

### Debugging Parse Trees

S-expressions provide readable tree inspection:

```bash
# Debug parsing issues
echo "problematic_code" | rust-sitter parse --sexp

# Compare implementations
echo "test_input" | tree-sitter parse --quiet > expected.sexp
echo "test_input" | rust-sitter parse --sexp > actual.sexp
diff expected.sexp actual.sexp
```

### Tree Analysis

S-expressions facilitate pattern matching and analysis:

```python
# Parse S-expression with external tools
import sexpdata

sexp = "(function_definition (identifier) 'hello' ...)"
tree = sexpdata.loads(sexp)
functions = find_all_functions(tree)
```

## Format Variations

### Compact vs. Pretty-Printed

**Compact (single line):**
```lisp
(module (function_definition "def" (identifier) "foo" (parameters) (block (pass_statement "pass"))))
```

**Pretty-printed (multi-line):**
```lisp
(module
  (function_definition
    "def"
    (identifier) "foo"
    (parameters)
    (block
      (pass_statement
        "pass"))))
```

### Language-Specific Variations

Different languages may have specific node types:

**Python-specific nodes:**
```lisp
(f_string
  (interpolation
    (expression_list)))
```

**JavaScript-specific nodes:**
```lisp
(template_string
  (template_substitution
    (expression)))
```

**Rust-specific nodes:**
```lisp
(match_expression
  (match_arm
    (match_pattern)))
```

## Implementation Details

### Character Escaping

S-expression strings follow standard escaping rules:

```rust
fn escape_sexp_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            c if c.is_control() => {
                format!("\\u{{{:04x}}}", c as u32).chars().collect()
            }
            c => vec![c],
        })
        .collect()
}
```

### Tree Traversal

S-expression generation uses depth-first traversal:

```rust
fn node_to_sexp(node: &ParsedNode, source: &str, indent: usize) -> String {
    let spaces = " ".repeat(indent);
    
    if node.is_named() {
        if node.child_count() == 0 {
            // Leaf node
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            format!("{}({} \"{}\")", spaces, node.kind(), escape_string(text))
        } else {
            // Internal node  
            let mut result = format!("{}({}\n", spaces, node.kind());
            for child in node.children() {
                result.push_str(&node_to_sexp(&child, source, indent + 2));
                result.push('\n');
            }
            result.push_str(&format!("{})", spaces));
            result
        }
    } else {
        // Anonymous node
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        format!("{}\"{}\"", spaces, escape_string(text))
    }
}
```

## Related Documentation

- **[Golden Tests Guide](../development/golden-tests.md)**: Using S-expressions for compatibility testing
- **[Testing Guide](../development/testing.md)**: Comprehensive testing strategies
- **[API Documentation](api.md)**: Programmatic S-expression generation
- **[Architecture](../development/architecture.md)**: Parse tree internal representation

S-expressions provide a standardized, human-readable format for parse tree representation, enabling effective debugging, testing, and analysis across the rust-sitter ecosystem.