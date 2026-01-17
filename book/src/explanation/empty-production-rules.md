# Empty Production Rules in rust-sitter

Tree-sitter does not support empty production rules. This means that a grammar rule cannot match zero characters without matching at least one token (even if it's an empty string literal).

## The Problem

When you have a struct with only a `Vec<T>` field that can be empty, rust-sitter generates a grammar rule that can match zero tokens, which causes tree-sitter-generate to fail with an `EmptyString` error.

```rust
// This will cause an EmptyString error
#[rust_sitter::language]
pub struct Module {
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
}
```

The generated grammar would have a rule like:
```json
{
  "Module": {
    "type": "FIELD",
    "name": "statements",
    "content": {
      "type": "SYMBOL",
      "name": "Module_statements_vec_contents"
    }
  },
  "Module_statements_vec_contents": {
    "type": "REPEAT",  // Can match zero occurrences
    "content": {
      "type": "SYMBOL",
      "name": "Statement"
    }
  }
}
```

## Solutions

### Solution 1: Use `non_empty = true`
Require at least one element in the Vec:

```rust
#[rust_sitter::language]
pub struct Module {
    #[rust_sitter::repeat(non_empty = true)]
    pub statements: Vec<Statement>,
}
```

### Solution 2: Add a Token Field
Add another field that always matches something (even if skipped):

```rust
#[rust_sitter::language]
pub struct Module {
    // Match optional whitespace to ensure non-empty rule
    #[rust_sitter::leaf(pattern = r"\s*")]
    #[rust_sitter::skip]
    _whitespace: (),
    
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
}
```

### Solution 3: Use Enum Variants
For cases where you need to distinguish between empty and non-empty:

```rust
#[rust_sitter::language]
pub enum Module {
    Empty(#[rust_sitter::leaf(pattern = r"\s*")] String),
    WithStatements {
        #[rust_sitter::repeat(non_empty = true)]
        statements: Vec<Statement>,
    }
}
```

### Solution 4: Handle Optional Suffixes with Enums
For structures like dotted names (`foo` vs `foo.bar.baz`):

```rust
// Instead of:
pub struct DottedName {
    first: Identifier,
    #[rust_sitter::repeat]  // Can be empty!
    rest: Vec<DottedPart>,
}

// Use:
#[rust_sitter::language]
pub enum DottedName {
    Single(Identifier),
    Dotted {
        first: Identifier,
        #[rust_sitter::repeat(non_empty = true)]
        rest: Vec<DottedPart>,
    }
}
```

## Best Practices

1. Always test your grammar with empty inputs to catch EmptyString errors early
2. Consider whether empty cases are actually needed in your language
3. Use enums to explicitly model different structural variants
4. Add delimiter tokens or whitespace patterns when appropriate

## Common Patterns and Examples

### Container Expressions (Lists, Tuples, Dicts)
When implementing container literals that can be empty:

```rust
// Problem: Empty list [] causes EmptyString error
#[rust_sitter::language]
pub struct ListExpression {
    #[rust_sitter::leaf(text = "[")]
    _open: (),
    #[rust_sitter::repeat]
    pub elements: Vec<Expression>,
    #[rust_sitter::leaf(text = "]")]
    _close: (),
}

// Solution: Add whitespace tokens
#[rust_sitter::language]
pub struct ListExpression {
    #[rust_sitter::leaf(text = "[")]
    _open: (),
    #[rust_sitter::leaf(pattern = r"\s*")]
    #[rust_sitter::skip]
    _ws1: (),
    #[rust_sitter::repeat]
    #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
    pub elements: Vec<Expression>,
    #[rust_sitter::leaf(pattern = r"\s*")]
    #[rust_sitter::skip]
    _ws2: (),
    #[rust_sitter::leaf(text = "]")]
    _close: (),
}
```

### Function Parameters
```rust
// Solution for optional parameter lists
#[rust_sitter::language]
pub struct Parameters {
    #[rust_sitter::leaf(text = "(")]
    _open: (),
    #[rust_sitter::leaf(pattern = r"\s*")]
    #[rust_sitter::skip]
    _ws1: (),
    #[rust_sitter::repeat]
    #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
    pub params: Vec<Parameter>,
    #[rust_sitter::leaf(pattern = r"\s*")]
    #[rust_sitter::skip]
    _ws2: (),
    #[rust_sitter::leaf(text = ")")]
    _close: (),
}
```

## Debugging Empty Rule Errors

When you encounter an `EmptyString` error:

1. **Enable artifact emission** to see the generated grammar:
   ```bash
   RUST_SITTER_EMIT_ARTIFACTS=true cargo build
   ```

2. **Look for the error message** which will indicate the problematic rule:
   ```
   Error: EmptyString("Module")
   ```

3. **Check the generated grammar** in `target/debug/build/*/out/grammar.json`

4. **Trace back** to find which struct has only empty Vec fields

## Technical Background

Tree-sitter's parsing algorithm requires every grammar rule to consume at least one token to make forward progress. Empty rules would cause the parser to get stuck in infinite loops. This is a fundamental limitation of the LR parsing approach used by Tree-sitter.

## Future Work

A future version of rust-sitter may automatically handle empty production rules by:
- Detecting structs with only Vec fields during macro expansion
- Automatically inserting whitespace tokens or wrapper rules
- Providing better error messages with suggested fixes

Until then, grammar authors need to be aware of this limitation and design their grammars accordingly.