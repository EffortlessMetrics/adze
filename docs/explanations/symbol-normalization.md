# Symbol Normalization in adze

## Overview

Symbol normalization is a crucial preprocessing step in the adze GLR parser generation pipeline. It converts complex grammar symbols (like `Optional`, `Repeat`, `Sequence`, `Choice`) into equivalent simple productions that only contain `Terminal`, `NonTerminal`, `External`, and `Epsilon` symbols.

## Why Normalization is Needed

The GLR core's FIRST/FOLLOW computation algorithms expect all grammar symbols to be in normalized form. Complex symbols like `Repeat(Sequence([Terminal(a), Terminal(b)]))` cannot be directly processed by the mathematical algorithms used for:

1. **FIRST set computation** - Computing which terminal symbols can start a production
2. **FOLLOW set computation** - Computing which terminal symbols can follow a non-terminal
3. **LR(1) item generation** - Building the parser state machine

## How Normalization Works

The normalization process transforms complex symbols into auxiliary non-terminal symbols with equivalent grammar rules:

### Optional Symbols
```rust
// Before: rule -> symbol?
rule -> optional_symbol

// After: (creates auxiliary rules)
rule -> _aux1001
_aux1001 -> symbol
_aux1001 -> ε
```

### Repeat Symbols  
```rust
// Before: rule -> symbol*
rule -> repeat_symbol

// After: (creates auxiliary rules)
rule -> _aux1002  
_aux1002 -> _aux1002 symbol    // left-recursive for efficiency
_aux1002 -> ε
```

### RepeatOne Symbols
```rust
// Before: rule -> symbol+
rule -> repeat_one_symbol

// After: (creates auxiliary rules)  
rule -> _aux1003
_aux1003 -> _aux1003 symbol    // left-recursive
_aux1003 -> symbol
```

### Choice Symbols
```rust
// Before: rule -> (symbol1 | symbol2 | symbol3)
rule -> choice_symbol

// After: (creates auxiliary rules)
rule -> _aux1004
_aux1004 -> symbol1
_aux1004 -> symbol2
_aux1004 -> symbol3
```

### Sequence Symbols
```rust
// Before: rule -> (symbol1 symbol2 symbol3)
rule -> sequence_symbol

// After: (flattens or creates auxiliary rule)
// For multi-element sequences:
rule -> _aux1005
_aux1005 -> symbol1 symbol2 symbol3

// For single-element sequences:
rule -> symbol1  // directly flattened
```

## Recursive Normalization

The normalization process handles nested complex symbols recursively:

```rust
// Before: rule -> (symbol*)?
rule -> optional_repeat_symbol

// After: (creates multiple auxiliary rules)
rule -> _aux1006           // For the Optional
_aux1006 -> _aux1007       // For the Repeat  
_aux1006 -> ε
_aux1007 -> _aux1007 symbol
_aux1007 -> ε
```

## Implementation Details

### Auxiliary Symbol ID Assignment

- Auxiliary symbols are assigned IDs starting from `max_existing_id + 1000`
- This avoids conflicts with user-defined symbols
- IDs are assigned sequentially during normalization
- The upper bound is `u16::MAX` (65535) to fit the `SymbolId` type

### Grammar Rule Names

- Auxiliary symbols get generated rule names like `_aux1001`, `_aux1002`, etc.
- These names are added to the grammar's `rule_names` mapping
- They appear in debug output and error messages

### Production ID Management

- Each auxiliary rule gets a unique `ProductionId`
- Production IDs are managed to avoid conflicts
- The counter increments for each new rule created

## API Usage

### Automatic Normalization

Normalization happens automatically during `FirstFollowSets::compute()`:

```rust
use adze_glr_core::FirstFollowSets;

let mut grammar = create_grammar_with_complex_symbols();

// Normalization happens automatically here
let first_follow = FirstFollowSets::compute(&mut grammar)?;
```

### Manual Normalization  

You can also normalize grammars explicitly:

```rust
use adze_ir::Grammar;

let mut grammar = create_grammar_with_complex_symbols();

// Manually normalize the grammar
grammar.normalize()?;

// Now all symbols are simple
assert!(all_symbols_are_simple(&grammar));
```

### Idempotency

Normalization is idempotent - calling it multiple times has no effect:

```rust
let mut grammar = create_grammar_with_complex_symbols();

grammar.normalize()?;
let after_first = grammar.clone();

grammar.normalize()?;  // Second call does nothing
let after_second = grammar.clone();

assert_eq!(after_first, after_second);
```

## Testing

The normalization functionality is comprehensively tested:

```bash
# Run normalization-specific tests
cargo test -p adze-ir --test test_normalization

# Run integration test that was originally failing
cargo test test_json_language_generation -p adze-tablegen
```

## Error Handling

Normalization can fail with `GrammarError` in these cases:

- **Symbol ID overflow** - Too many auxiliary symbols created (exceeds u16 range)  
- **Recursive symbol definitions** - Self-referencing complex symbols (future work)
- **Memory constraints** - Extremely large grammars (unlikely in practice)

## Performance Considerations

- **Memory usage** - Each complex symbol may create 1-3 auxiliary rules
- **Parse table size** - More rules = larger parse tables, but equivalent functionality
- **Runtime performance** - No impact on parsing speed, only affects compilation time
- **Compilation time** - Minimal overhead, runs once during grammar processing

## Backward Compatibility

The normalization feature maintains full backward compatibility:

- Existing grammars with simple symbols work unchanged
- Complex symbols are transparently normalized  
- Generated parser behavior is identical
- API remains the same except for the mutability requirement in `FirstFollowSets::compute()`

## Debugging

### Debug Output

Set environment variables to see normalization in action:

```bash
RUST_LOG=trace cargo test test_json_language_generation
```

### Inspecting Normalized Grammars

After normalization, you can inspect the generated auxiliary rules:

```rust
grammar.normalize()?;

for (symbol_id, rules) in &grammar.rules {
    if symbol_id.0 > 1000 {  // Auxiliary symbols start at 1000+
        println!("Auxiliary symbol {}: {} rules", symbol_id.0, rules.len());
        for rule in rules {
            println!("  {:?}", rule);
        }
    }
}
```

### Common Debug Scenarios

1. **Complex symbols not normalized** - Check that `FirstFollowSets::compute()` is called with `&mut Grammar`
2. **Unexpected auxiliary rules** - Verify the original grammar structure
3. **Symbol ID conflicts** - Ensure no user symbols have IDs > 1000

## Future Improvements

Potential enhancements to the normalization system:

1. **Symbol ID optimization** - Better auxiliary ID assignment strategies
2. **Rule deduplication** - Merge equivalent auxiliary rules to reduce grammar size  
3. **Error reporting** - Better error messages for normalization failures
4. **Performance optimization** - Faster normalization for very large grammars
5. **Rule naming** - More descriptive names for auxiliary rules in debug output