# HYPOTHESIS: Enum Variants Create Implicit Disambiguation

## Date: 2025-11-19

## Status: Investigation in Progress

## Hypothesis

The way rust-sitter maps Rust enum variants to grammar productions **naturally creates unambiguous grammars** even when the user intends to create ambiguity.

## Background

We've proven that:
1. ✅ The LR(1) automaton builder works correctly
2. ✅ Manually-built ambiguous grammars generate conflicts
3. ❌ Enum-based grammars generate ZERO conflicts

## The Key Difference

### Manually-Built Ambiguous Grammar (WORKS)
Using `GrammarBuilder`:
```rust
let grammar = GrammarBuilder::new("ambiguous")
    .rule("expr", vec!["binary"])
    .rule("expr", vec!["NUMBER"])
    .rule("binary", vec!["expr", "OP", "expr"])  // Ambiguous!
    .build();
```

Productions:
```
Expr → Binary
Expr → Number
Binary → Expr Op Expr  # Creates recursion through Expr
```

**Result**: Shift/reduce conflict detected ✅

###Enum-Based Grammar (FAILS)
Using Rust enums:
```rust
#[rust_sitter::language]
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),
    Number(i32),
}
```

**How rust-sitter translates this**:
```
Expr → Expr_Binary       # Separate non-terminal for Binary variant
Expr → Expr_Number       # Separate non-terminal for Number variant
Expr_Binary → Expr Op Expr
Expr_Number → NUMBER
```

Or possibly inline to:
```
Expr → Expr Op Expr      # Inline Binary variant
Expr → NUMBER            # Inline Number variant
```

## Analysis

### If Variants Are Separate Non-Terminals

The LR(1) parser can use lookahead to distinguish:
- When it sees `NUMBER`, reduce to `Expr_Number`, then to `Expr`
- When it sees complex expression start, it eventually reduces to `Expr_Binary`, then to `Expr`

**Problem**: The extra layer of non-terminals provides disambiguation points!

### If Variants Are Inlined

This SHOULD create ambiguity:
```
Expr → Expr Op Expr
Expr → NUMBER
```

After parsing "Expr Op Expr", on lookahead "Op":
- **SHIFT**: Continue to parse → "(Expr Op Expr) Op ..."
- **REDUCE**: Complete the production → "Expr Op (Expr Op ...)"

This MUST create a conflict!

## Test Plan

1. **Examine Generated Grammar.json**
   - Look at the actual productions generated from `ambiguous_expr.rs`
   - Determine if variants are separate symbols or inlined
   - Check if there are hidden intermediate symbols

2. **Add Debug Logging to Extraction**
   - Instrument `tool/src/expansion.rs` to print productions
   - See exactly how enum variants are processed

3. **Compare Production Structures**
   - Manual grammar (works): Count productions and symbols
   - Enum grammar (fails): Count productions and symbols
   - Look for structural differences

## Expected Findings

### Scenario A: Variants Are Separate Symbols
If each variant becomes a separate non-terminal, then the enum-based approach **CANNOT** create truly ambiguous grammars. This would be a fundamental architectural limitation.

**Implication**: Need to allow users to write direct productions without enum variants, OR automatically inline variant productions for ambiguous cases.

### Scenario B: Variants Are Inlined
If variants are inlined directly into Expr productions, then something ELSE is adding disambiguation:
- Implicit precedence?
- Symbol ordering?
- Special handling for recursive variants?

## Next Steps

1. Enable grammar.json emission
2. Compare generated grammar structures
3. Trace through production generation in expansion.rs
4. Identify where disambiguation is introduced
5. Fix or document the limitation

## Related Files

- `example/src/ambiguous_expr.rs` - Enum-based grammar
- `glr-core/tests/test_ambiguous_expr_lr1_construction.rs` - Working manual grammar
- `tool/src/expansion.rs` - Enum to grammar conversion
- `target/debug/build/.../grammar_ambiguous_expr/` - Generated artifacts

## Timeline

- Investigation Start: 2025-11-19
- Hypothesis Formed: 2025-11-19
- Target Resolution: Within 1-2 days

## Impact

If enum variants inherently create disambiguation, this has major implications:
- May need alternative syntax for truly ambiguous grammars
- Documentation must explain this limitation
- GLR validation tests need manually-built grammars
- Users creating ambiguous grammars need different approach
