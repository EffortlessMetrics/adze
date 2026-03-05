# adze-grammar-json-core

Tiny SRP microcrate for reading token patterns from Tree-sitter `grammar.json` files and mapping them to `adze_ir` token pattern types.

## API

- `load_patterns_from_grammar_json(path)`
- `load_patterns_with_symbol_map(path, symbol_names)`
