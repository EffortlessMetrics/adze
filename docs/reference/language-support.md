# Adze Language Support

## Built-in Grammars

The following grammars are maintained within the Adze repository and serve as reference implementations:

| Language | Location | Features Demonstrated |
|----------|----------|-----------------------|
| **Python** | `grammars/python` | External scanners (indentation), Complex rules |
| **JavaScript** | `grammars/javascript` | Large grammar, GLR conflict resolution |
| **Go** | `grammars/go` | Standard grammar structure |
| **Python (Simple)** | `grammars/python-simple` | Simplified subset for testing |

## Importing Tree-sitter Grammars (Experimental)

Adze includes a tool called `ts-bridge` that can generate Adze bindings for existing Tree-sitter grammars. This allows you to leverage the vast ecosystem of 150+ existing Tree-sitter grammars.

### Usage

**Note:** This feature is experimental and may require manual adjustments to the generated Rust code.

```bash
# Build the bridge tool
cargo build -p ts-bridge

# Run it against a tree-sitter grammar repo
cargo run -p ts-bridge -- /path/to/tree-sitter-rust
```

## Language Features Status

| Feature | Status | Notes |
|---------|--------|-------|
| **External Scanners** | ✅ Supported | Python indentation scanner is fully implemented in Rust |
| **GLR (Ambiguity)** | ✅ Supported | Used for handling conflicts in JS/Python |
| **Query System** | 🚧 Planned | Tree-sitter query compatibility is in progress |
| **LSP Generation** | 🚧 Experimental | Prototype available in `lsp-generator` crate |

## Contributing New Languages

We welcome contributions of new grammars! Please see the [Developer Guide](../DEVELOPER_GUIDE.md) for how to set up a new grammar crate in the `grammars/` directory.
