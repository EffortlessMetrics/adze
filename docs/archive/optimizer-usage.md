# Grammar Optimizer Usage Guide

The adze grammar optimizer is an optional feature that can significantly improve parser performance by applying various optimization passes to your grammar.

## Enabling the Optimizer

The optimizer is disabled by default to ensure stability. To enable it, use the `optimize` feature flag:

### In Cargo.toml

```toml
[dependencies]
adze-tool = { version = "0.5.0-beta", features = ["optimize"] }
```

### Via Command Line

```bash
cargo build --features adze-tool/optimize
```

## What the Optimizer Does

When enabled, the optimizer applies several transformation passes:

1. **Unit Rule Elimination**: Removes intermediate rules like `A -> B` by inlining them
2. **Symbol Inlining**: Inlines simple, non-recursive rules to reduce indirection
3. **Token Merging**: Combines tokens with identical patterns
4. **Left Recursion Optimization** (Enhanced PR #4): Transforms left-recursive rules with comprehensive metadata preservation
   - Preserves conflict declarations for both original and auxiliary symbols
   - Adjusts field indices when removing left-recursive symbols from rules
   - Uses proper Grammar API for cleaner rule manipulation
   - Creates readable auxiliary symbol names (e.g., "expr__rec" for recursive part)
5. **Dead Code Removal**: Removes unreferenced symbols and rules
6. **Symbol Renumbering**: Compacts symbol IDs for better cache locality

## Performance Impact

The optimizer can provide:
- **15-20% faster parsing** for typical grammars
- **Smaller parse tables** due to rule elimination
- **Better cache performance** from symbol renumbering

## Important Notes

1. **Tree-sitter Compatibility**: The optimizer preserves Tree-sitter semantics
2. **source_file Preservation**: The special `source_file` symbol is never optimized away
3. **FIRST/FOLLOW Safety**: Optimizations preserve FIRST/FOLLOW sets for GLR parsing
4. **Debug Artifacts**: Use `ADZE_EMIT_ARTIFACTS=true` to inspect optimization results

## Example

Before optimization:
```
source_file -> statement
statement -> expression  
expression -> sum
sum -> product | sum + product
product -> primary | product * primary
primary -> NUMBER
```

After optimization (unit rules eliminated):
```
source_file -> sum + product | product * primary | NUMBER
sum -> product * primary | NUMBER
product -> NUMBER
```

## Testing (Enhanced in PR #4)

The optimizer includes comprehensive regression tests, particularly for left recursion transformation:

```bash
# Run all optimizer tests
cargo test -p adze-ir

# Run specific left recursion transformation test
cargo test -p adze-ir test_transform_left_recursion_rewrites_grammar
```

The test `test_transform_left_recursion_rewrites_grammar` validates:
- Proper transformation of left-recursive rules (A -> A + b becomes A -> b A', A' -> + b A' | ε)
- Field index adjustment (field at position 2 becomes position 1 after removing first symbol)
- Conflict declaration propagation (conflicts on original symbol extend to auxiliary symbol)
- Precedence and associativity preservation
- Readable auxiliary symbol naming

## Troubleshooting

If you encounter issues with the optimizer:

1. **Disable optimization**: Remove the `optimize` feature to compare behavior
2. **Check artifacts**: Set `ADZE_EMIT_ARTIFACTS=true` to see the IR before/after
3. **Run regression tests**: Use `cargo test -p adze-ir` to verify optimizer behavior
4. **Report issues**: File a bug with the grammar that causes problems

The optimizer is designed to be conservative and should never change the language accepted by your grammar.