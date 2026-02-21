# Empty Production Rules in adze

Tree-sitter does not support empty production rules. This means that a grammar rule cannot match zero characters without matching at least one token (even if it's an empty string literal).

## The Problem

When you have a struct with only a `Vec<T>` field that can be empty, adze generates a grammar rule that can match zero tokens, which causes tree-sitter-generate to fail with an `EmptyString` error.

```rust
// This will cause an EmptyString error
#[adze::language]
pub struct Module {
    #[adze::repeat]
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
#[adze::language]
pub struct Module {
    #[adze::repeat(non_empty = true)]
    pub statements: Vec<Statement>,
}
```

### Solution 2: Add a Token Field
Add another field that always matches something (even if skipped):

```rust
#[adze::language]
pub struct Module {
    // Match optional whitespace to ensure non-empty rule
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _whitespace: (),
    
    #[adze::repeat]
    pub statements: Vec<Statement>,
}
```

### Solution 3: Use Enum Variants
For cases where you need to distinguish between empty and non-empty:

```rust
#[adze::language]
pub enum Module {
    Empty(#[adze::leaf(pattern = r"\s*")] String),
    WithStatements {
        #[adze::repeat(non_empty = true)]
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
    #[adze::repeat]  // Can be empty!
    rest: Vec<DottedPart>,
}

// Use:
#[adze::language]
pub enum DottedName {
    Single(Identifier),
    Dotted {
        first: Identifier,
        #[adze::repeat(non_empty = true)]
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
#[adze::language]
pub struct ListExpression {
    #[adze::leaf(text = "[")]
    _open: (),
    #[adze::repeat]
    pub elements: Vec<Expression>,
    #[adze::leaf(text = "]")]
    _close: (),
}

// Solution: Add whitespace tokens
#[adze::language]
pub struct ListExpression {
    #[adze::leaf(text = "[")]
    _open: (),
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws1: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub elements: Vec<Expression>,
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws2: (),
    #[adze::leaf(text = "]")]
    _close: (),
}
```

### Function Parameters
```rust
// Solution for optional parameter lists
#[adze::language]
pub struct Parameters {
    #[adze::leaf(text = "(")]
    _open: (),
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws1: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub params: Vec<Parameter>,
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws2: (),
    #[adze::leaf(text = ")")]
    _close: (),
}
```

## Debugging Empty Rule Errors

When you encounter an `EmptyString` error:

1. **Enable artifact emission** to see the generated grammar:
   ```bash
   ADZE_EMIT_ARTIFACTS=true cargo build
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

A future version of adze may automatically handle empty production rules by:
- Detecting structs with only Vec fields during macro expansion
- Automatically inserting whitespace tokens or wrapper rules
- Providing better error messages with suggested fixes

Until then, grammar authors need to be aware of this limitation and design their grammars accordingly.