# adze-python

Python grammar definition for Adze.

## Overview

This crate provides a complete Python grammar definition using Adze's macro-based
grammar specification. It generates a Tree-sitter-compatible parser for Python source
code at build time.

## Features

- Full Python grammar coverage
- Build-time parser generation via `adze-tool`
- Snapshot testing with `insta` for parse tree verification
- Compatible with both the standard Tree-sitter runtime and Adze's pure-Rust runtime

## Usage

```rust
use adze_python::language;

let lang = language();
// Use with adze's Parser API to parse Python source code
```

## Testing

```bash
cargo test -p adze-python
```

## License

MIT OR Apache-2.0
