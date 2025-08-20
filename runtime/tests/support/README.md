# Test Support Infrastructure

This directory contains helper modules for testing the pure-Rust parser implementation.

## Current Status

### What's Working
- **Grammar Definition**: `json_grammar.rs` defines a complete JSON grammar with proper rules and tokens
- **Language Building**: `language_builder.rs` can construct a TSLanguage struct from grammar and parse table
- **Type Compatibility**: All types compile and match the expected FFI-compatible structures

### What Needs Work
- **Parse Table Generation**: The manually constructed parse table in `json_grammar.rs` is incomplete
  - Action table indices don't match the symbol IDs returned by the lexer
  - States are simplified and don't represent a complete LR(1) automaton
  - Need to integrate with `glr-core` to generate proper parse tables

- **Lexer Integration**: The lexer needs to:
  - Properly tokenize input according to the grammar's token definitions
  - Return symbol IDs that match the parse table's expectations
  - Handle whitespace and other extras correctly

- **Symbol Registration**: Need proper mapping between:
  - Token names in the grammar
  - Symbol IDs in the parse table
  - Indices in the action/goto tables

## Next Steps

To get the pure-Rust golden tests working:

1. **Use GLR-Core for Table Generation**: Instead of manually building the parse table, use the algorithms in `glr-core` to generate proper LR(1) states and transitions

2. **Fix Symbol Mapping**: Ensure that:
   - Tokens get consistent symbol IDs across grammar, lexer, and parse table
   - The `symbol_to_index` map correctly maps symbol IDs to action table columns
   - The lexer returns the correct symbol IDs for each token

3. **Complete the Parse Table**: Add all necessary states and transitions for parsing JSON, including:
   - Proper shift/reduce actions for all states
   - Goto transitions for non-terminals
   - Accept action for the end-of-input

4. **Test with Simple Cases First**: Start with minimal JSON like `{}` or `1` before testing complex structures

## Files

- `json_grammar.rs`: Defines a JSON grammar for testing
- `language_builder.rs`: Builds TSLanguage structs from grammar and parse table
- `unified_json_helper.rs`: Entry point for pure-Rust JSON language generation