# The Empty Rule Bug: How To Not Fail At Tree-sitter Grammar Generation

## The Journey

When building a macro-driven Tree-sitter grammar generator in Rust, I encountered one of the most notorious pitfalls in parser generation: the "empty production rule" problem. This post documents the journey of discovering, understanding, and solving this issue in adze.

## The Problem Manifests

It started innocently enough. I was porting a Python grammar to adze when the build failed with:

```
Error: EmptyString("Module")
```

My first thought was "surely a module can be empty?" After all, an empty Python file is valid Python. But Tree-sitter had other ideas.

## Understanding the Root Cause

After hours of debugging, I discovered that Tree-sitter's parsing algorithm fundamentally cannot handle rules that match zero tokens. Why? Because LR parsers need to consume input to make progress. An empty rule would cause infinite loops.

Consider this innocent-looking Rust struct:

```rust
#[adze::language]
pub struct Module {
    #[adze::repeat]
    pub statements: Vec<Statement>,
}
```

This generates a grammar where `Module` can match nothing when the Vec is empty. Tree-sitter rejects this at grammar generation time.

## The Investigation Process

1. **Initial Confusion**: Why does Tree-sitter reject valid language constructs?
2. **Deep Dive**: Traced through macro expansion, IR generation, and Tree-sitter's grammar validation
3. **Realization**: This isn't a bug—it's a fundamental constraint of LR parsing
4. **Challenge**: How to maintain ergonomic Rust APIs while satisfying Tree-sitter's requirements?

## Solutions Discovered

### Pattern 1: Mandatory Elements
```rust
#[adze::repeat(non_empty = true)]
pub statements: Vec<Statement>,
```

### Pattern 2: Whitespace Anchors
```rust
#[adze::language]
pub struct ListExpression {
    #[adze::leaf(text = "[")]
    _open: (),
    
    // These ensure the rule is never empty
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws1: (),
    
    #[adze::repeat]
    pub elements: Vec<Expression>,
    
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws2: (),
    
    #[adze::leaf(text = "]")]
    _close: (),
}
```

### Pattern 3: Enum Modeling
```rust
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

## Why This Matters

1. **It's Universal**: Every Tree-sitter grammar author will hit this
2. **It's Subtle**: The error messages don't clearly indicate the solution
3. **It's Architectural**: You can't patch around it—you must design for it

## Lessons Learned

1. **Parser constraints shape API design**: The theoretical limitations of LR parsing directly impact how we structure our grammar DSLs
2. **Documentation is critical**: This edge case needs to be front-and-center for new users
3. **Macro systems can help**: We could potentially detect and auto-fix these patterns at compile time

## Impact on adze

This investigation led to:
- Comprehensive documentation for grammar authors
- Clear patterns for handling empty productions
- A deeper understanding of Tree-sitter's constraints
- Ideas for future macro improvements

## The Bigger Picture

This experience reinforces that building developer tools isn't just about implementing algorithms—it's about understanding the fundamental constraints of your underlying systems and designing APIs that guide users toward success.

When you're building on top of complex systems like Tree-sitter, you inherit not just their capabilities but also their limitations. The art is in presenting those limitations as features, not bugs.

## What's Next?

Future versions of adze could:
- Detect empty rules at macro expansion time
- Automatically insert whitespace tokens
- Provide clearer error messages with suggested fixes

But for now, understanding and documenting these patterns is a major step forward for the adze ecosystem.

---

*Building adze has been a journey through the depths of parser theory, Rust macros, and developer experience design. The empty rule problem is just one of many challenges, but solving it properly sets the foundation for a robust, production-ready parser generator.*