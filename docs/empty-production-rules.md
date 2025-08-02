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

## Future Work

A future version of rust-sitter may automatically handle empty production rules by inserting appropriate workarounds during grammar generation. Until then, grammar authors need to be aware of this limitation and design their grammars accordingly.